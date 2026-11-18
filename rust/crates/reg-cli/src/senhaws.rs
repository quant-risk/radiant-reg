//! senhaws — cliente do serviço senhaws do BACEN STA (port de
//! `backend/internal/senhaws/senhaws.go`, manual v1.5 §9).
//!
//! Serviço SEPARADO do STA WS (host diferente): homologação
//! `www9.bcb.gov.br/senhaws`, produção `www3.bcb.gov.br/senhaws`
//! (senhaws.go:38-40) — ambos na mesma allowlist hard-coded de
//! `crate::sta::HOSTS_PERMITIDOS`.
//!
//! Operações (mapeamento com senhaws.go):
//! | Go (senhaws.go:linha)      | Rust (senhaws.rs)     |
//! |----------------------------|-----------------------|
//! | AlterarSenha (152)         | `alterar_senha`       |
//! | ConsultarVencimento (209)  | `consultar_vencimento`|
//! | GerarSenhaRandom (256)     | `gerar_senha_hex`     |
//!
//! Port fiel — incluindo os DESVIOS documentados:
//! - SEM retry (o v1 não tem RetryingClient no senhaws — senhaws.go inteiro
//!   não referencia retry.go);
//! - validação client-side da nova senha: 8..=128 CHARS (bytes, `len()` do
//!   Go) e ≠ senha atual → `RegError::InvalidValue` (senhaws.go:152-164);
//! - PUT /senha espera 204 com Content-Type application/xml (senhaws.go:177-198);
//! - GET /senha/vencimento espera 200 + `<DiasVencimentoSenha>` inteiro ≥ 0
//!   (senhaws.go:209-244);
//! - cap de resposta 1 MiB (senhaws.go:339).
//!
//! DESVIO HONESTO em `gerar_senha_hex`: o v1 usava `math/rand` (senhaws.go:246-263,
//! explicitamente não-cryptográfica!) — o port lê 16 bytes de `/dev/urandom`
//! (força criptográfica real, std-only, sem crate novo). A senha gerada
//! retorna em `Zeroizing<String>`.
//!
//! Após `alterar_senha` com sucesso, a senha atual do PROVEDOR de segredos
//! fica desatualizada — o caller DEVE atualizar
//! `RADIANT_SECRET_BACEN_SENHA_<user>` antes da próxima chamada STA, senão
//! todas as chamadas retornam 401 (senhaws.go:142-144).

use reg_core::RegError;
use zeroize::Zeroizing;

use crate::secrets::ProvedorSegredos;
use crate::sta::{
    ler_corpo_limitado, truncar_bytes, validar_url_permitida, validar_usuario_sisbacen, xml_parsear,
    StaError,
};

/// Cap de respostas do senhaws — 1 MiB (senhaws.go:339, responses minúsculas).
const MAX_CORPO_SENHAWS_BYTES: usize = 1 << 20;

/// O helper `ler_corpo_limitado` é do módulo `sta` e retorna `StaError` —
/// converte preservando a categoria (dispatch exaustivo, sem wildcard).
fn mapear_erro_sta(e: StaError) -> SenhawsError {
    match e {
        StaError::Transporte(inner) => SenhawsError::Transporte(inner),
        StaError::LeituraCorpo(inner) => SenhawsError::LeituraCorpo(inner),
        StaError::ErroBacen { status, codigo, mensagem, .. } => {
            SenhawsError::ErroBacen { status, codigo, mensagem }
        }
        StaError::Validacao(_)
        | StaError::ContentHashMismatch { .. }
        | StaError::ContentHashHeaderMalformed { .. }
        | StaError::ContentHashAusente { .. }
        | StaError::RespostaInvalida { .. }
        | StaError::AposTentativas { .. } => SenhawsError::RespostaInvalida {
            motivo: e.to_string(),
        },
    }
}

/// Rejeição do senhaws — port de `SenhaError` (senhaws.go:281-289). Validação
/// client-side reutiliza `RegError` (nunca erro paralelo).
#[derive(Debug, thiserror::Error)]
pub enum SenhawsError {
    #[error(transparent)]
    Validacao(#[from] RegError),
    /// Erro formal do BACEN (4xx/5xx com XML Listagem 4 — mesmo formato do
    /// STA WS) — port de `parseSenhaError` (senhaws.go:315-336).
    #[error("BACEN senhaws error {status}: {mensagem}")]
    ErroBacen {
        status: u16,
        codigo: String,
        mensagem: String,
    },
    #[error("senhaws transporte: {0}")]
    Transporte(#[from] reqwest::Error),
    #[error("senhaws falha de leitura do corpo da resposta: {0}")]
    LeituraCorpo(#[from] std::io::Error),
    #[error("senhaws resposta inválida: {motivo}")]
    RespostaInvalida { motivo: String },
}

/// Cliente do senhaws BACEN — port de `SenhawsClient` (senhaws.go:70-128).
/// SEM retry (port fiel — o v1 não retenta no senhaws).
#[derive(Debug)]
pub struct SenhawsClient {
    http: reqwest::blocking::Client,
    base_url: String,
    usuario: String,
    senha_atual: Zeroizing<String>,
}

impl SenhawsClient {
    /// Construtor de produção — port das validações de `NewSenhawsClient`
    /// (senhaws.go:86-128) + a allowlist hard-coded do workspace (mais forte
    /// que o v1, que aceitava qualquer host HTTPS).
    pub fn nova(
        base_url: &str,
        usuario: &str,
        segredos: &dyn ProvedorSegredos,
        timeout: std::time::Duration,
    ) -> Result<Self, SenhawsError> {
        validar_url_permitida(base_url)?;
        validar_usuario_sisbacen(usuario)?;
        let senha_atual = segredos.obter_senha_bacen(usuario)?;
        let http = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .http1_only()
            .build()
            .map_err(|e| RegError::InvalidValue {
                field: "http_client".to_string(),
                reason: format!("falha ao construir cliente HTTP: {e}"),
            })?;
        Ok(Self {
            http,
            base_url: base_url.to_string(),
            usuario: usuario.to_string(),
            senha_atual,
        })
    }

    /// Construtor de TESTE (aponta ao mock local em 127.0.0.1 — a allowlist
    /// de produção não tem bypass em builds release).
    #[cfg(test)]
    pub(crate) fn para_testes(
        base_url: &str,
        usuario: &str,
        senha: &str,
        timeout: std::time::Duration,
    ) -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .http1_only()
            .build()
            .unwrap_or_else(|e| panic!("cliente HTTP de teste: {e}"));
        Self {
            http,
            base_url: base_url.to_string(),
            usuario: usuario.to_string(),
            senha_atual: Zeroizing::new(senha.to_string()),
        }
    }

    /// Auth Basic preemptiva (senhaws.go:130-135) — o header vai na 1ª tentativa.
    fn requisicao(&self, metodo: reqwest::Method, url: &str) -> reqwest::blocking::RequestBuilder {
        self.http
            .request(metodo, url)
            .basic_auth(self.usuario.as_str(), Some(self.senha_atual.as_str()))
    }

    /// Port de `AlterarSenha` (senhaws.go:152-198, manual §9.1): PUT /senha
    /// com `<Parametros><Senha>/<NovaSenha>/<ConfirmacaoNovaSenha>`,
    /// Content-Type application/xml, espera 204.
    ///
    /// Após sucesso a senha atual aqui NÃO muda — o caller atualiza o
    /// provedor de segredos (senhaws.go:142-144).
    pub fn alterar_senha(&self, nova_senha: &str) -> Result<(), SenhawsError> {
        let invalido = |motivo: String| RegError::InvalidValue {
            field: "novaSenha".to_string(),
            reason: motivo,
        };
        if nova_senha.is_empty() {
            return Err(SenhawsError::Validacao(invalido(
                "não pode ser vazia".to_string(),
            )));
        }
        if nova_senha.len() < 8 {
            return Err(SenhawsError::Validacao(invalido(
                "deve ter no mínimo 8 chars".to_string(),
            )));
        }
        if nova_senha.len() > 128 {
            return Err(SenhawsError::Validacao(invalido(
                "deve ter no máximo 128 chars".to_string(),
            )));
        }
        if nova_senha == self.senha_atual.as_str() {
            return Err(SenhawsError::Validacao(invalido(
                "deve ser diferente da senha atual".to_string(),
            )));
        }

        let corpo = xml_parametros_senha(&self.senha_atual, nova_senha);
        let url = format!("{}/senha", self.base_url);
        let mut resp = self
            .requisicao(reqwest::Method::PUT, &url)
            .header(reqwest::header::CONTENT_TYPE, "application/xml")
            .header(reqwest::header::ACCEPT, "application/xml")
            .body(corpo.into_bytes())
            .send()?;
        let resp_corpo =
            ler_corpo_limitado(&mut resp, MAX_CORPO_SENHAWS_BYTES as u64).map_err(mapear_erro_sta)?;
        let status = resp.status().as_u16();
        if status != 204 {
            return Err(erro_senhaws_de_corpo(status, &resp_corpo));
        }
        Ok(())
    }

    /// Port de `ConsultarVencimento` (senhaws.go:209-244, manual §9.2): GET
    /// /senha/vencimento, 200 + `<DiasVencimentoSenha>` inteiro ≥ 0.
    pub fn consultar_vencimento(&self) -> Result<u32, SenhawsError> {
        let url = format!("{}/senha/vencimento", self.base_url);
        let mut resp = self.requisicao(reqwest::Method::GET, &url).send()?;
        let resp_corpo =
            ler_corpo_limitado(&mut resp, MAX_CORPO_SENHAWS_BYTES as u64).map_err(mapear_erro_sta)?;
        let status = resp.status().as_u16();
        if status != 200 {
            return Err(erro_senhaws_de_corpo(status, &resp_corpo));
        }
        let doc = xml_parsear(&resp_corpo).map_err(mapear_erro_sta)?;
        let cru = doc
            .primeiro_texto("Resultado/DiasVencimentoSenha")
            .unwrap_or("")
            .trim()
            .to_string();
        if cru.is_empty() {
            return Err(SenhawsError::RespostaInvalida {
                motivo: "BACEN retornou 200 mas <DiasVencimentoSenha> vazio".to_string(),
            });
        }
        let dias: i64 = cru.parse().map_err(|_| SenhawsError::RespostaInvalida {
            motivo: format!("DiasVencimentoSenha não é inteiro válido (got {cru:?})"),
        })?;
        if dias < 0 {
            return Err(SenhawsError::RespostaInvalida {
                motivo: format!("DiasVencimentoSenha negativo (got {dias})"),
            });
        }
        u32::try_from(dias).map_err(|_| SenhawsError::RespostaInvalida {
            motivo: format!("DiasVencimentoSenha fora do range suportado (got {dias})"),
        })
    }
}

/// Port endurecido de `GerarSenhaRandom` (senhaws.go:256-263): 16 bytes
/// criptográficos de `/dev/urandom` → 32 chars hex (o v1 usava `math/rand`
/// explicitamente não-cryptográfica — upgrade documentado no doc de módulo).
/// A senha retorna zeroizada.
pub fn gerar_senha_hex() -> Result<Zeroizing<String>, RegError> {
    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut bytes))
        .map_err(|e| RegError::InvalidValue {
            field: "gerar_senha".to_string(),
            reason: format!("falha ao ler 16 bytes de /dev/urandom: {e}"),
        })?;
    Ok(Zeroizing::new(hex::encode(bytes)))
}

// ============================================================
// Helpers locais (port de senhaws.go:267-347)
// ============================================================

/// Corpo do PUT /senha — port de `senhaParams` (senhaws.go:267-272).
fn xml_parametros_senha(senha_atual: &str, nova_senha: &str) -> String {
    // Escape via quick-xml — senha pode conter & e < (senhas hex não, mas a
    // via manual do --nova-senha sim).
    let esc = |v: &str| quick_xml::escape::escape(v).into_owned();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Parametros><Senha>{}\
</Senha><NovaSenha>{}</NovaSenha><ConfirmacaoNovaSenha>{}</ConfirmacaoNovaSenha></Parametros>",
        esc(senha_atual),
        esc(nova_senha),
        esc(nova_senha),
    )
}

/// Port de `parseSenhaError` (senhaws.go:315-336).
fn erro_senhaws_de_corpo(status: u16, corpo: &[u8]) -> SenhawsError {
    if let Ok(doc) = xml_parsear(corpo)
        && let Some(codigo_cru) = doc.primeiro_texto("Resultado/Erro/Codigo")
        && let Ok(codigo_num) = codigo_cru.trim().parse::<i64>()
        && codigo_num != 0
    {
        let mensagem = doc
            .primeiro_texto("Resultado/Erro/Descricao")
            .unwrap_or("")
            .to_string();
        return SenhawsError::ErroBacen {
            status,
            codigo: codigo_num.to_string(),
            mensagem,
        };
    }
    SenhawsError::ErroBacen {
        status,
        codigo: format!("HTTP_{status}"),
        mensagem: truncar_bytes(corpo, 200),
    }
}

// ============================================================
// TESTES — mock HTTP reutilizado de sta::tests::http_mock (TcpListener com
// respostas HTTP escritas à mão; NUNCA conecta ao BACEN real).
// ============================================================
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::sta::tests::{base64_padrao, resposta_http, ServidorHttp};
    use std::sync::Arc;
    use std::time::Duration;

    const USUARIO: &str = "123450001.fulano";

    fn cliente_para(url: &str) -> SenhawsClient {
        SenhawsClient::para_testes(url, USUARIO, "senha-atual", Duration::from_secs(5))
    }

    fn xml_erro(status: u16, descricao: &str) -> String {
        format!("<Resultado><Erro><Codigo>{status}</Codigo><Descricao>{descricao}</Descricao></Erro></Resultado>")
    }

    #[test]
    fn alterar_senha_204_happy_path_com_auth_e_xml() {
        // Port de senhaws_test.go:178 (TestSenhawsClient_AlterarSenha_HappyPath).
        let auth_esperada = format!("Basic {}", base64_padrao(b"123450001.fulano:senha-atual"));
        let servidor = ServidorHttp::iniciar(Arc::new(move |captura| {
            assert_eq!(captura.metodo, "PUT");
            assert_eq!(captura.caminho, "/senha");
            assert_eq!(
                captura.cabecalho("content-type"),
                Some("application/xml"),
                "Content-Type application/xml exigido (senhaws.go:183)"
            );
            assert_eq!(
                captura.cabecalho("authorization"),
                Some(auth_esperada.as_str()),
                "Auth Basic preemptiva base64(user:senha_atual)"
            );
            resposta_http(204, "No Content", &[], b"")
        }));
        let cliente = cliente_para(&servidor.url);
        cliente.alterar_senha("new-password-12345").unwrap();

        let corpo = String::from_utf8_lossy(&servidor.capturas()[0].corpo).into_owned();
        assert!(corpo.contains("<Senha>senha-atual</Senha>"), "{corpo}");
        assert!(corpo.contains("<NovaSenha>new-password-12345</NovaSenha>"), "{corpo}");
        assert!(
            corpo.contains("<ConfirmacaoNovaSenha>new-password-12345</ConfirmacaoNovaSenha>"),
            "{corpo}"
        );
    }

    #[test]
    fn alterar_senha_400_e_erro_bacen() {
        // Port de senhaws_test.go:225.
        let servidor = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(
                400,
                "Bad Request",
                &[("Content-Type", "application/xml")],
                xml_erro(400, "Nova senha não atende requisitos").as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let err = cliente.alterar_senha("weak-pwd-1").unwrap_err();
        let SenhawsError::ErroBacen { status, mensagem, .. } = &err else {
            panic!("esperava ErroBacen 400, got {err:?}")
        };
        assert_eq!(*status, 400);
        assert!(mensagem.contains("Nova senha"), "{mensagem}");
    }

    #[test]
    fn alterar_senha_validacoes_locais_sem_rede() {
        // Port de senhaws_test.go:273 (8-128 chars, ≠ atual, não vazia).
        let servidor = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(204, "No Content", &[], b"")
        }));
        let cliente = cliente_para(&servidor.url);
        let longa = "x".repeat(129);
        for nova in ["", "curta", longa.as_str(), "senha-atual"] {
            let err = cliente.alterar_senha(nova).unwrap_err();
            let SenhawsError::Validacao(_) = &err else {
                panic!("senha {nova:?} deveria falhar validação, got {err:?}")
            };
        }
        assert_eq!(
            servidor.total_capturas(),
            0,
            "validação local falha ANTES da rede"
        );
        // Limite inferior válido (8 chars exatos) passa a validação local:
        let servidor_ok = ServidorHttp::iniciar(Arc::new(|_| resposta_http(204, "No Content", &[], b"")));
        cliente_para(&servidor_ok.url)
            .alterar_senha("12345678")
            .unwrap();
    }

    #[test]
    fn consultar_vencimento_happy_path() {
        // Port de senhaws_test.go:366 — XML multi-linha com whitespace.
        let xml = concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
            "<Resultado>\n<DiasVencimentoSenha>30</DiasVencimentoSenha>\n</Resultado>"
        );
        let servidor = ServidorHttp::iniciar(Arc::new(move |captura| {
            assert_eq!(captura.caminho, "/senha/vencimento");
            resposta_http(200, "OK", &[("Content-Type", "application/xml")], xml.as_bytes())
        }));
        let cliente = cliente_para(&servidor.url);
        assert_eq!(cliente.consultar_vencimento().unwrap(), 30);
    }

    #[test]
    fn consultar_vencimento_erros_tipados() {
        // 400 formal (port de senhaws_test.go:389):
        let servidor_400 = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(
                400,
                "Bad Request",
                &[("Content-Type", "application/xml")],
                xml_erro(400, "Usuário não autenticado").as_bytes(),
            )
        }));
        let err = cliente_para(&servidor_400.url).consultar_vencimento().unwrap_err();
        let SenhawsError::ErroBacen { status, .. } = &err else {
            panic!("esperava ErroBacen 400, got {err:?}")
        };
        assert_eq!(*status, 400);
        // XML válido com valor vazio (port de senhaws_test.go:433):
        let servidor_vazio = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(200, "OK", &[], b"<Resultado><DiasVencimentoSenha></DiasVencimentoSenha></Resultado>")
        }));
        let err = cliente_para(&servidor_vazio.url).consultar_vencimento().unwrap_err();
        let SenhawsError::RespostaInvalida { motivo } = &err else {
            panic!("esperava RespostaInvalida, got {err:?}")
        };
        assert!(motivo.contains("vazio"), "{motivo}");
        // Não-inteiro (port de senhaws_test.go:454):
        let servidor_abc = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(200, "OK", &[], b"<Resultado><DiasVencimentoSenha>abc</DiasVencimentoSenha></Resultado>")
        }));
        let err = cliente_para(&servidor_abc.url).consultar_vencimento().unwrap_err();
        let SenhawsError::RespostaInvalida { motivo } = &err else {
            panic!("esperava RespostaInvalida, got {err:?}")
        };
        assert!(motivo.contains("não é inteiro válido"), "{motivo}");
        // Negativo (port de senhaws_test.go:475):
        let servidor_neg = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(200, "OK", &[], b"<Resultado><DiasVencimentoSenha>-5</DiasVencimentoSenha></Resultado>")
        }));
        let err = cliente_para(&servidor_neg.url).consultar_vencimento().unwrap_err();
        let SenhawsError::RespostaInvalida { motivo } = &err else {
            panic!("esperava RespostaInvalida, got {err:?}")
        };
        assert!(motivo.contains("negativo"), "{motivo}");
    }

    #[test]
    fn gerar_senha_hex_32_chars_lowercase() {
        // Port de senhaws_test.go:496: 16 bytes hex = 32 chars [0-9a-f].
        for _ in 0..10 {
            let senha = gerar_senha_hex().unwrap();
            assert_eq!(senha.len(), 32);
            assert!(
                senha.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
                "chars não-hex em {senha:?}"
            );
            // Zeroizing<String> no retorno — a cópia é zerada no Drop.
        }
    }
}
