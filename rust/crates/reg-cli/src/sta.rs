//! sta — cliente do BACEN STA Web Services v1.5 (port de `backend/internal/sta/ws.go`,
//! `retry.go` e `ws_types.go`).
//!
//! Fontes do port (lidas antes da implementação, com linha):
//! - `backend/internal/sta/ws.go` — protocolo completo (Seções 5.x/6.x/7.1/8.1
//!   do Manual de Web Services do STA, v1.5 julho/2022);
//! - `backend/internal/sta/ws_types.go` — structs XML e enums tipados;
//! - `backend/internal/sta/retry.go` — política de retry transiente;
//! - `docs/blueprint/research/04-rust-stack.md` §R1 (66-83) — TLS via rustls
//!   (reqwest `rustls-tls` já no workspace); a nota §R1 sobre client-cert
//!   ICP-Brasil (A1/A3) fica para a fase de integração real: o port segue o
//!   ws.go, que autentica APENAS com Basic preemptivo + TLS.
//!
//! Operações (mapeamento 1:1 com ws.go):
//! | Go (ws.go:linha)              | Rust (sta.rs)            |
//! |-------------------------------|--------------------------|
//! | Submit (286)                  | `enviar`                 |
//! | requestProtocol (363)         | `solicitar_protocolo`    |
//! | uploadContent (421)           | `enviar_conteudo`        |
//! | StatusUpload (523)            | `posicao_upload`         |
//! | SubmitRange (1121)            | `enviar_range`           |
//! | Download (667)                | `baixar`                 |
//! | ListDisponiveis (909)         | `listar_disponiveis`     |
//! | AlterarSituacao (1010)        | `alterar_situacao`       |
//!
//! ALLOWLIST hard-coded (`HOSTS_PERMITIDOS`): sta-h.bcb.gov.br, sta.bcb.gov.br,
//! www9.bcb.gov.br, www3.bcb.gov.br. Qualquer outra URL é rejeitada com
//! `RegError::InvalidValue` no construtor — ANTES de qualquer conexão (o
//! equivalente Go era `AllowInsecureHTTP`; aqui o bypass existe só em
//! `para_testes`, `#[cfg(test)]` — produção não tem chave para desligar a
//! allowlist). URLs absolutas vindas do `atom:link` de paginação são
//! revalidadas contra a allowlist em `seguir_pagina`.
//!
//! Auth Basic PREEMPTIVO (RFC 7617, ws.go:264-267): o header `Authorization`
//! é enviado na primeira tentativa, sem esperar 401.
//!
//! Retry (port de retry.go:184-279): SÓ 5xx/timeout/network; 4xx, rejeição de
//! upload e hash divergente NUNCA; backoff `1s × 2^(n-1)` com jitter ±50%.
//! Desvio honesto: o jitter do v1 usava float (applyJitter, retry.go:267-279);
//! aqui é aritmética INTEIRA em pontos-básicos (doutrina zero-float, §4.5 do
//! blueprint — jitter não é matemática financeira, mas o lint de workspace
//! `float_arithmetic` veta float em todo o crate e o resultado é o mesmo
//! intervalo ±50%). Esgotadas as tentativas, o erro final é
//! `StaError::AposTentativas` (port do wrap `fmt.Errorf` de retry.go:171).
//!
//! Validação X-Content-Hash (ws.go:722-753, manual §6.1.1): OBRIGATÓRIA no
//! download — hash ausente = `StaError::ContentHashAusente`; header
//! malformado = `StaError::ContentHashHeaderMalformed`; divergente =
//! `StaError::ContentHashMismatch` (equivalentes aos sentinels Go
//! `ErrContentHash*`, ws.go:110-121). Nenhum dos três é retried.
//!
//! Cap defensivo de body: respostas pequenas 10 MiB, download 100 MiB,
//! chunk de upload 8 MiB (ws.go:96-107 + especificação do STA chunked).
//!
//! Exit codes do CLI (contrato do doc de módulo de main.rs):
//! 0 aceito · 1 transporte/integridade · 2 validação-cliente · 3 formal-BACEN.
//! O mapa está em main.rs (`codigo_saida_sta`/`codigo_saida_senhaws`).

use std::io::Read;
use std::sync::Mutex;
use std::time::Duration;

use quick_xml::events::{BytesStart, Event};
use reg_core::RegError;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::secrets::ProvedorSegredos;

/// Allowlist hard-coded dos hosts BACEN (especificação do STA client):
/// homolog/prod do STA WS e homolog/prod do senhaws. Qualquer host fora
/// desta lista é rejeitado com erro tipado ANTES de conectar.
pub const HOSTS_PERMITIDOS: [&str; 4] = [
    "sta-h.bcb.gov.br",
    "sta.bcb.gov.br",
    "www9.bcb.gov.br",
    "www3.bcb.gov.br",
];

/// Cap de respostas pequenas (XML de protocolo, posicaoupload, disponiveis,
/// erros) — ws.go:96.
pub(crate) const MAX_CORPO_RESPOSTA_BYTES: usize = 10 << 20;
/// Cap do body do download — ws.go:107.
pub(crate) const MAX_CORPO_DOWNLOAD_BYTES: usize = 100 << 20;
/// Cap do chunk de upload (especificação: chunk máximo de 8 MiB).
pub(crate) const MAX_CHUNK_UPLOAD_BYTES: usize = 8 << 20;
/// Timeout default por chamada HTTP — ws.go:234-236.
pub const TIMEOUT_PADRAO: Duration = Duration::from_secs(30);

/// Ambiente de destino — URLs hard-coded da allowlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Ambiente {
    /// Homologação: sta-h.bcb.gov.br/staws · www9.bcb.gov.br/senhaws.
    Homolog,
    /// Produção: sta.bcb.gov.br/staws · www3.bcb.gov.br/senhaws.
    Producao,
}

impl Ambiente {
    /// URL base do STA WS — ws.go:3-5.
    pub fn url_sta(self) -> &'static str {
        match self {
            Self::Homolog => "https://sta-h.bcb.gov.br/staws",
            Self::Producao => "https://sta.bcb.gov.br/staws",
        }
    }
    /// URL base do senhaws — senhaws.go:38-40.
    pub fn url_senhaws(self) -> &'static str {
        match self {
            Self::Homolog => "https://www9.bcb.gov.br/senhaws",
            Self::Producao => "https://www3.bcb.gov.br/senhaws",
        }
    }
}

// ============================================================
// Erros tipados (port de STAError ws.go:133-145 + sentinels 110-121)
// ============================================================

/// Erro do cliente STA. Validações de input do CLIENTE reutilizam
/// `RegError` de reg-core (variante `Validacao` — nunca um erro paralelo);
/// rejeições do BACEN, transporte e integridade são variantes próprias.
#[derive(Debug, thiserror::Error)]
pub enum StaError {
    /// Validação client-side (usuário, URL, tamanho de chunk, segredo).
    #[error(transparent)]
    Validacao(#[from] RegError),
    /// Rejeição formal do BACEN (4xx/5xx com XML Listagem 4) — port de
    /// `STAError` (ws.go:133-145) e `parseSTAError` (463-478).
    #[error("BACEN STA error {status} (protocolo={protocolo}, código={codigo}): {mensagem}")]
    ErroBacen {
        status: u16,
        codigo: String,
        mensagem: String,
        protocolo: String,
    },
    /// SHA-256 do body não bate com X-Content-Hash — ws.go:111-114
    /// (`ErrContentHashMismatch`). NÃO é retryable (corrupção permanente).
    #[error(
        "STA: X-Content-Hash do BACEN não bate com SHA-256 do body (esperado={esperado}, obtido={obtido})"
    )]
    ContentHashMismatch { esperado: String, obtido: String },
    /// Header X-Content-Hash existe mas não é `SHA-256 {64-hex}` — ws.go:117-120
    /// (`ErrContentHashHeaderMalformed`). NÃO é retryable.
    #[error("STA: header X-Content-Hash malformado (esperado: SHA-256 {{64-hex}}): {motivo}")]
    ContentHashHeaderMalformed { motivo: String },
    /// BACEN não mandou o header obrigatório — ws.go:722-734.
    #[error("STA: BACEN não retornou header X-Content-Hash (protocolo={protocolo})")]
    ContentHashAusente { protocolo: String },
    /// Erro de transporte HTTP (rede/timeout) — retryable conforme classificação.
    #[error("STA transporte: {0}")]
    Transporte(#[from] reqwest::Error),
    /// Falha de leitura do corpo (conexão caiu no meio) — retryable se o
    /// `ErrorKind` indicar problema de rede.
    #[error("STA falha de leitura do corpo da resposta: {0}")]
    LeituraCorpo(#[from] std::io::Error),
    /// Resposta que não parseia (XML mal-formado, protocolo vazio) —
    /// conservador: NÃO retryable (retry.go:211-212).
    #[error("STA resposta inválida: {motivo}")]
    RespostaInvalida { motivo: String },
    /// Todas as tentativas esgotadas — port do wrap de retry.go:171.
    #[error("STA falhou após {tentativas} tentativas: {causa}")]
    AposTentativas {
        tentativas: u32,
        causa: Box<StaError>,
    },
}

// ============================================================
// Tipos do domínio (port de ws_types.go)
// ============================================================

/// Par `[inicio, fim]` de `RangesRecebidos` — ws_types.go:144-149. Inclusivo
/// nos dois extremos: "0-3" = bytes 0,1,2,3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intervalo {
    pub inicio: i64,
    pub fim: i64,
}

/// Enum dos 3 valores oficiais de Situacao (manual §5.3.1) + Desconhecida —
/// port de `UploadSituacao` (ws_types.go:154-166).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SituacaoUpload {
    /// "Transmissão não iniciada".
    NaoIniciada,
    /// "Transmissão pendente".
    Pendente,
    /// "Transmissão finalizada".
    Finalizada,
    /// Valor fora dos 3 documentados (defesa — BACEN adicionou valor novo).
    Desconhecida,
}

impl SituacaoUpload {
    /// String canônica pt-BR do manual — port de `String()` (ws_types.go:169-180).
    pub fn como_str(self) -> &'static str {
        match self {
            Self::NaoIniciada => "Transmissão não iniciada",
            Self::Pendente => "Transmissão pendente",
            Self::Finalizada => "Transmissão finalizada",
            Self::Desconhecida => "Desconhecida",
        }
    }
}

/// Port de `parseUploadSituacao` (ws_types.go:184-195): valores fora dos 3
/// documentados viram `Desconhecida` (não erro — o cru fica em `situacao_cru`).
fn parsear_situacao_upload(cru: &str) -> SituacaoUpload {
    match cru {
        "Transmissão não iniciada" => SituacaoUpload::NaoIniciada,
        "Transmissão pendente" => SituacaoUpload::Pendente,
        "Transmissão finalizada" => SituacaoUpload::Finalizada,
        _ => SituacaoUpload::Desconhecida,
    }
}

/// SituacaoAtual de um arquivo na listagem — port de `SituacaoArquivo`
/// (ws_types.go:300-308): código 1 = Recebido, 3 = A receber, resto Desconhecida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SituacaoArquivo {
    /// Código 1.
    Recebido,
    /// Código 3.
    AReceber,
    /// Código fora de {1, 3}.
    Desconhecida,
}

impl SituacaoArquivo {
    /// Port de `String()` (ws_types.go:311-320).
    pub fn como_str(self) -> &'static str {
        match self {
            Self::Recebido => "Recebido",
            Self::AReceber => "A receber",
            Self::Desconhecida => "Desconhecida",
        }
    }
}

/// Port de `parseSituacaoArquivo` (ws_types.go:324-333).
fn parsear_situacao_arquivo(codigo: i64) -> SituacaoArquivo {
    match codigo {
        1 => SituacaoArquivo::Recebido,
        3 => SituacaoArquivo::AReceber,
        _ => SituacaoArquivo::Desconhecida,
    }
}

/// Situacao da alteração (§7.1) — port de `SituacaoTransferencia`
/// (ws_types.go:343-363). Aqui SEM variante Desconhecida: o parse é
/// ESTRICTO (erro tipado em valor fora de {A_REC, REC}) — dispatch
/// exaustivo da doutrina: o v1 deixava Unknown passar até o BACEN rejeitar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SituacaoTransferencia {
    /// "A_REC" (a receber).
    AReceber,
    /// "REC" (recebido).
    Recebido,
}

impl SituacaoTransferencia {
    /// String que vai no XML (manual linha 789).
    pub fn como_str(self) -> &'static str {
        match self {
            Self::AReceber => "A_REC",
            Self::Recebido => "REC",
        }
    }
    /// Parse estrito de "A_REC" | "REC".
    pub fn parsear(cru: &str) -> Result<Self, RegError> {
        match cru {
            "A_REC" => Ok(Self::AReceber),
            "REC" => Ok(Self::Recebido),
            outro => Err(RegError::InvalidValue {
                field: "situacao".to_string(),
                reason: format!("esperado A_REC|REC, got {outro:?}"),
            }),
        }
    }
}

/// Resultado de `posicao_upload` — port de `UploadStatus` (ws_types.go:137-142)
/// com o cru de `RangesRecebidos` preservado para audit (o v1 descartava).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusUpload {
    pub protocolo: String,
    pub ranges_recebidos: Vec<Intervalo>,
    /// Valor cru de `RangesRecebidos` (ex. "0-3;5-8") — audit/debug.
    pub ranges_recebidos_cru: String,
    pub situacao: SituacaoUpload,
    /// Valor cru de `Situacao` — ws_types.go:136 (defesa contra valor novo).
    pub situacao_cru: String,
}

/// Resultado de `baixar` — port de `DownloadResult` (ws_types.go:239-245).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultadoDownload {
    pub conteudo: Vec<u8>,
    /// SHA-256 hex computado pelo CLIENTE (defesa contra BACEN bug) — já
    /// verificado contra `X-Content-Hash`.
    pub content_hash: String,
    pub etag: String,
    pub last_modified: String,
    /// Valor cru do header X-Content-Hash (debug/audit).
    pub content_hash_header: String,
}

/// Rejeição de upload — port de `Rejection{Code: "UPLOAD_FAILED"}` (ws.go:333-341).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejecao {
    pub codigo: String,
    pub mensagem: String,
}

/// Resultado de `enviar` — port de `Result` do v1: protocolo é PRESERVADO
/// mesmo quando o upload falha (forensic trail, ws.go:277-280).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultadoEnvio {
    pub protocolo: String,
    pub aceito: bool,
    pub rejeicao: Option<Rejecao>,
}

/// Parâmetros de `listar_disponiveis` (Tabela 4) — port de `ListDisponiveisOpts`
/// (ws_types.go:259-264).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OpcoesDisponiveis {
    /// OBRIGATÓRIO — formato "yyyy-MM-ddTHH:mm:ss.SSS" (BACEN responde 400 se malformado).
    pub data_hora_inicio: String,
    pub identificador_documento: Option<String>,
    pub sistemas: Option<String>,
    pub dependencia: Option<String>,
}

/// Port de `ListDisponiveisResult` (ws_types.go:272-277).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultadoDisponiveis {
    pub arquivos: Vec<ArquivoDisponivel>,
    pub data_hora_proxima_consulta: String,
    /// URL completa da próxima página (atom:link) — presente só com >1000 registros.
    pub proxima_pagina_url: Option<String>,
    pub tem_proxima_pagina: bool,
}

/// Port de `ArquivoDisponivel` (ws_types.go:280-290).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArquivoDisponivel {
    pub protocolo: String,
    pub tipo_arquivo: String,
    pub codigo_documento: String,
    pub sistema: String,
    pub tamanho_arquivo: i64,
    /// SHA-256 hex do arquivo completo (para validar download range).
    pub hash: String,
    pub situacao_atual: SituacaoArquivo,
    pub situacao_atual_cru: String,
    pub data_hora_disponibilizacao: String,
}

// ============================================================
// Validações de configuração (port de ws.go:155/212-232 e senhaws.go:82)
// ============================================================

/// Regex do v1 portada para validação manual (sem crate regex no workspace):
/// `^([0-9]{5}[0-9]{4}|[0-9]{5}/[0-9]{4})\.[A-Za-z0-9_-]+$` — ws.go:147-155.
pub(crate) fn validar_usuario_sisbacen(usuario: &str) -> Result<(), RegError> {
    let invalido = |motivo: String| RegError::InvalidValue {
        field: "usuario".to_string(),
        reason: format!(
            "formato Sisbacen exato UUUUUDDDD.operador (ou UUUUU/DDDD.operador) inválido: {motivo} (got {usuario:?})"
        ),
    };
    let Some((prefixo, operador)) = usuario.split_once('.') else {
        return Err(invalido("sem '.' separador".to_string()));
    };
    if operador.is_empty()
        || !operador
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(invalido(format!(
            "operador deve casar com [A-Za-z0-9_-]+ (got {operador:?})"
        )));
    }
    let prefixo_ok = match prefixo.split_once('/') {
        Some((if_code, dependencia)) => {
            digitos_exatos(if_code, 5) && digitos_exatos(dependencia, 4)
        }
        None => digitos_exatos(prefixo, 9),
    };
    if !prefixo_ok {
        return Err(invalido(
            "prefixo deve ser 5+4 dígitos (UUUUUDDDD) ou UUUUU/DDDD".to_string(),
        ));
    }
    Ok(())
}

fn digitos_exatos(s: &str, n: usize) -> bool {
    s.len() == n && s.chars().all(|c| c.is_ascii_digit())
}

/// Allowlist hard-coded — erro tipado ANTES de conectar (port endurecido das
/// checagens de ws.go:213-221; o v1 aceitava qualquer host HTTPS).
pub(crate) fn validar_url_permitida(base_url: &str) -> Result<(), RegError> {
    let invalido = |motivo: String| RegError::InvalidValue {
        field: "base_url".to_string(),
        reason: motivo,
    };
    if base_url.is_empty() {
        return Err(invalido(
            "base_url requerida (ex.: https://sta-h.bcb.gov.br/staws)".to_string(),
        ));
    }
    if !base_url.starts_with("https://") {
        return Err(invalido(format!(
            "base_url deve usar HTTPS (got {base_url:?}); allowlist: {HOSTS_PERMITIDOS:?}"
        )));
    }
    if base_url.ends_with('/') {
        return Err(invalido("base_url não deve terminar com /".to_string()));
    }
    let url =
        reqwest::Url::parse(base_url).map_err(|e| invalido(format!("base_url inválida: {e}")))?;
    let Some(host) = url.host_str() else {
        return Err(invalido("base_url sem host".to_string()));
    };
    if !HOSTS_PERMITIDOS.contains(&host) {
        return Err(invalido(format!(
            "host {host:?} fora da allowlist hard-coded {HOSTS_PERMITIDOS:?}"
        )));
    }
    Ok(())
}

// ============================================================
// RetryPolicy — port de retry.go
// ============================================================

/// Port de `RetryConfig`/`RetryingClient` (retry.go:29-107) em aritmética
/// inteira: `jitter_bp` é o jitter em pontos-básicos (5000 = ±50%). NÃO é
/// Clone — o estado do PRNG é mutável e não queremos cópias duplicando
/// semente silenciosamente.
#[derive(Debug)]
pub struct RetryPolicy {
    max_tentativas: u32,
    backoff_base: Duration,
    jitter_bp: u32,
    estado_prng: Mutex<u64>,
}

impl RetryPolicy {
    /// Defaults da especificação: 3 tentativas, backoff 1s × 2^n, jitter ±50%
    /// (retry.go:78-97). Delega a `nova` — os defaults passam pela MESMA
    /// validação do port de `NewRetryingClient`; o `panic!` é inalcançável
    /// por construção (valores dentro do range validado).
    pub fn padrao() -> Self {
        Self::nova(3, Duration::from_secs(1), 5_000)
            .unwrap_or_else(|e| panic!("defaults da RetryPolicy violam a própria validação: {e}"))
    }

    /// Port das validações de `NewRetryingClient` (retry.go:74-107):
    /// tentativas em [1, 10], jitter em [0, 10000] bp (±100%).
    pub fn nova(
        max_tentativas: u32,
        backoff_base: Duration,
        jitter_bp: u32,
    ) -> Result<Self, RegError> {
        if !(1..=10).contains(&max_tentativas) {
            return Err(RegError::InvalidValue {
                field: "max_tentativas".to_string(),
                reason: format!("deve estar em [1, 10] (got {max_tentativas})"),
            });
        }
        if jitter_bp > 10_000 {
            return Err(RegError::InvalidValue {
                field: "jitter_bp".to_string(),
                reason: format!("deve estar em [0, 10000] bp (±100%), got {jitter_bp}"),
            });
        }
        Ok(Self {
            max_tentativas,
            backoff_base,
            jitter_bp,
            estado_prng: Mutex::new(semente_prng()),
        })
    }

    /// Port de `Submit` do RetryingClient (retry.go:120-172), generalizado
    /// para toda operação: sucesso retorna; erro não-retryable retorna CRU;
    /// esgotamento wrapa em `AposTentativas` (retry.go:171). Cancelamento de
    /// contexto não existe no CLI blocking — o sleep é direto.
    pub fn executar<T>(
        &self,
        mut rotina: impl FnMut() -> Result<T, StaError>,
    ) -> Result<T, StaError> {
        let mut tentativa: u32 = 1;
        loop {
            match rotina() {
                Ok(valor) => return Ok(valor),
                Err(erro) => {
                    if tentativa >= self.max_tentativas {
                        return Err(StaError::AposTentativas {
                            tentativas: self.max_tentativas,
                            causa: Box::new(erro),
                        });
                    }
                    if !self.retryavel(&erro) {
                        return Err(erro);
                    }
                    let espera = self.espera(tentativa);
                    tentativa += 1;
                    std::thread::sleep(espera);
                }
            }
        }
    }

    /// Port de `shouldRetry` (retry.go:184-213): 5xx → retry; hash/validação/
    /// resposta inválida → NUNCA; 4xx → NUNCA (caller bug); timeout/network → retry.
    fn retryavel(&self, erro: &StaError) -> bool {
        match erro {
            StaError::ErroBacen { status, .. } => (500..=599).contains(status),
            StaError::Transporte(e) => e.is_timeout() || e.is_connect() || e.is_request(),
            // Erro no meio da leitura: retryable só quando o kind indica rede
            // (port do string-match de retry.go:241-249 — comparação por kind,
            // sem match com wildcard por ser enum non_exhaustive).
            StaError::LeituraCorpo(e) => {
                let k = e.kind();
                k == std::io::ErrorKind::TimedOut
                    || k == std::io::ErrorKind::ConnectionReset
                    || k == std::io::ErrorKind::ConnectionAborted
                    || k == std::io::ErrorKind::UnexpectedEof
                    || k == std::io::ErrorKind::Interrupted
                    || k == std::io::ErrorKind::WouldBlock
            }
            StaError::Validacao(_)
            | StaError::ContentHashMismatch { .. }
            | StaError::ContentHashHeaderMalformed { .. }
            | StaError::ContentHashAusente { .. }
            | StaError::RespostaInvalida { .. }
            | StaError::AposTentativas { .. } => false,
        }
    }

    /// `computeBackoff` + `applyJitter` (retry.go:255-279) em inteiro:
    /// backoff_ms × 2^(n-1), depois ±jitter_bp/10000.
    fn espera(&self, tentativa: u32) -> Duration {
        let expoente = tentativa.saturating_sub(1).min(62);
        let base_ms = self.backoff_base.as_millis();
        let exp_ms = base_ms.checked_shl(expoente).unwrap_or(u128::MAX);
        let base_final = u64::try_from(exp_ms).unwrap_or(u64::MAX);
        let delta_bp: i64 = if self.jitter_bp == 0 {
            0
        } else {
            let faixa = 2 * (self.jitter_bp as u64) + 1;
            let r = self.proximo_u64() % faixa;
            r as i64 - self.jitter_bp as i64
        };
        let ms = base_final as i128 + (base_final as i128 * delta_bp as i128) / 10_000;
        if ms <= 0 {
            Duration::ZERO
        } else {
            Duration::from_millis(u64::try_from(ms).unwrap_or(u64::MAX))
        }
    }

    /// PRNG xorshift64\* (estado no Mutex, sem crate rand no workspace).
    fn proximo_u64(&self) -> u64 {
        let mut estado = match self.estado_prng.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut x = *estado;
        if x == 0 {
            x = 0x9E37_79B9_7F4A_7C15;
        }
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        *estado = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

fn semente_prng() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    nanos ^ ((std::process::id() as u64) << 32)
}

// ============================================================
// StaClient — port de WSClient (ws.go:196-258)
// ============================================================

/// Cliente HTTP do BACEN STA WS. Imutável após construção; a senha vive em
/// `Zeroizing<String>` (zerada no Drop — higiene além do v1, que guardava
/// `String` crua).
#[derive(Debug)]
pub struct StaClient {
    http: reqwest::blocking::Client,
    base_url: String,
    usuario: String,
    senha: Zeroizing<String>,
    retry: RetryPolicy,
}

impl StaClient {
    /// Construtor de produção: valida allowlist + usuário, resolve a senha
    /// pelo `ProvedorSegredos` e força HTTP/1.1 sobre rustls (port de
    /// ws.go:212-249 — `ForceAttemptHTTP2: false`, `MinVersion TLS 1.2`;
    /// rustls 0.23 não negocia < TLS 1.2 por padrão).
    pub fn nova(
        base_url: &str,
        usuario: &str,
        segredos: &dyn ProvedorSegredos,
        timeout: Duration,
        retry: RetryPolicy,
    ) -> Result<Self, StaError> {
        validar_url_permitida(base_url)?;
        validar_usuario_sisbacen(usuario)?;
        let senha = segredos.obter_senha_bacen(usuario)?;
        let http = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .http1_only()
            // Redirects NUNCA são seguidos: o default do reqwest 0.12 é
            // `Policy::limited(10)` e seguiria 3xx para host ARBITRÁRIO —
            // burlando a allowlist (o payload do CADOC viajaria no corpo).
            // A URL seguida é SEMPRE a validada aqui/por `seguir_pagina`;
            // um 3xx do BACEN/proxy vira resposta crua, tratada como erro.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| RegError::InvalidValue {
                field: "http_client".to_string(),
                reason: format!("falha ao construir cliente HTTP: {e}"),
            })?;
        Ok(Self {
            http,
            base_url: base_url.to_string(),
            usuario: usuario.to_string(),
            senha,
            retry,
        })
    }

    /// Construtor de TESTE: pulap a allowlist para apontar ao mock local
    /// (equivalente do `AllowInsecureHTTP` do v1, ws.go:180-183 — mas aqui a
    /// chave nem existe em builds de produção: `#[cfg(test)]`).
    #[cfg(test)]
    pub(crate) fn para_testes(
        base_url: &str,
        usuario: &str,
        senha: &str,
        timeout: Duration,
        retry: RetryPolicy,
    ) -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .http1_only()
            // Mesma política do construtor de produção (nenhum redirect) —
            // os testes exercitam o comportamento REAL do cliente.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_else(|e| panic!("cliente HTTP de teste: {e}"));
        Self {
            http,
            base_url: base_url.to_string(),
            usuario: usuario.to_string(),
            senha: Zeroizing::new(senha.to_string()),
            retry,
        }
    }

    /// Request com Auth Basic PREEMPTIVO (RFC 7617) — o header vai na
    /// primeira tentativa, sem esperar 401 (ws.go:264-267, 385).
    fn requisicao(&self, metodo: reqwest::Method, url: &str) -> reqwest::blocking::RequestBuilder {
        self.http
            .request(metodo, url)
            .basic_auth(self.usuario.as_str(), Some(self.senha.as_str()))
    }

    // ------------------------------------------------------------
    // Fluxo de envio em 2 fases (ws.go:286-357, manual §5.1 + §5.2)
    // ------------------------------------------------------------

    /// Port de `Submit`: Fase 1 POST /arquivos → 201 + protocolo; Fase 2 PUT
    /// /arquivos/{p}/conteudo → 200. SHA-256 do payload é calculado ANTES do
    /// POST (ws.go:305-307, manual §2.4). Falha de upload NÃO é erro: retorna
    /// `aceito=false` + `Rejecao{UPLOAD_FAILED}` com o protocolo preservado
    /// (forensic trail, ws.go:327-341) — e rejeição NUNCA é retried.
    pub fn enviar(&self, cadoc: &str, payload: &[u8]) -> Result<ResultadoEnvio, StaError> {
        if payload.is_empty() {
            return Err(StaError::Validacao(RegError::InvalidValue {
                field: "payload".to_string(),
                reason: "STA submission vazia (sem XML nem ZIP)".to_string(),
            }));
        }
        let hash_hex = hex::encode(Sha256::digest(payload));
        let tamanho = payload.len() as u64;
        let protocolo = self.solicitar_protocolo(cadoc, &hash_hex, tamanho)?;
        match self.enviar_conteudo(&protocolo, payload) {
            Ok(()) => Ok(ResultadoEnvio {
                protocolo,
                aceito: true,
                rejeicao: None,
            }),
            Err(e) => Ok(ResultadoEnvio {
                protocolo,
                aceito: false,
                rejeicao: Some(Rejecao {
                    codigo: "UPLOAD_FAILED".to_string(),
                    mensagem: e.to_string(),
                }),
            }),
        }
    }

    /// Port de `requestProtocol` (ws.go:363-414): POST /arquivos com XML
    /// `<Parametros>`, espera 201 + `<Protocolo>` não-vazio.
    fn solicitar_protocolo(
        &self,
        cadoc: &str,
        hash_hex: &str,
        tamanho: u64,
    ) -> Result<String, StaError> {
        let url = format!("{}/arquivos", self.base_url);
        self.retry.executar(|| {
            let corpo = xml_parametros_protocolo(cadoc, hash_hex, tamanho);
            let mut resp = self
                .requisicao(reqwest::Method::POST, &url)
                .header(reqwest::header::CONTENT_TYPE, "application/xml")
                .header(reqwest::header::ACCEPT, "application/xml")
                .body(corpo.into_bytes())
                .send()?;
            let resp_corpo = ler_corpo_limitado(&mut resp, MAX_CORPO_RESPOSTA_BYTES as u64)?;
            let status = resp.status().as_u16();
            if status != 201 {
                return Err(erro_bacen_de_corpo(status, &resp_corpo, ""));
            }
            let doc = xml_parsear(&resp_corpo)?;
            let protocolo = doc
                .primeiro_texto("Resultado/Protocolo")
                .unwrap_or("")
                .to_string();
            if protocolo.is_empty() {
                return Err(StaError::RespostaInvalida {
                    motivo: format!(
                        "BACEN retornou 201 mas <Protocolo> vazio: {}",
                        truncar_bytes(&resp_corpo, 200)
                    ),
                });
            }
            Ok(protocolo)
        })
    }

    /// Port de `uploadContent` (ws.go:421-444): PUT /arquivos/{p}/conteudo,
    /// espera 200, Content-Type INTENCIONALMENTE OMITIDO (manual §5.2.1).
    fn enviar_conteudo(&self, protocolo: &str, payload: &[u8]) -> Result<(), StaError> {
        let url = format!("{}/arquivos/{protocolo}/conteudo", self.base_url);
        self.retry.executar(|| {
            let mut resp = self
                .requisicao(reqwest::Method::PUT, &url)
                .body(payload.to_vec())
                .send()?;
            let resp_corpo = ler_corpo_limitado(&mut resp, MAX_CORPO_RESPOSTA_BYTES as u64)?;
            let status = resp.status().as_u16();
            if status != 200 {
                return Err(erro_bacen_de_corpo(status, &resp_corpo, protocolo));
            }
            Ok(())
        })
    }

    // ------------------------------------------------------------
    // Read side (manual §5.3.1 / §6.1.1 / §8.1.1 / §7.1)
    // ------------------------------------------------------------

    /// Port de `StatusUpload` (ws.go:523-574): GET /arquivos/{p}/posicaoupload
    /// sem Content-Type (manual §5.3.1).
    pub fn posicao_upload(&self, protocolo: &str) -> Result<StatusUpload, StaError> {
        if protocolo.is_empty() {
            return Err(StaError::Validacao(RegError::MissingField {
                field: "protocolo".to_string(),
            }));
        }
        let url = format!("{}/arquivos/{protocolo}/posicaoupload", self.base_url);
        self.retry.executar(|| {
            let mut resp = self.requisicao(reqwest::Method::GET, &url).send()?;
            let resp_corpo = ler_corpo_limitado(&mut resp, MAX_CORPO_RESPOSTA_BYTES as u64)?;
            let status = resp.status().as_u16();
            if status != 200 {
                return Err(erro_bacen_de_corpo(status, &resp_corpo, protocolo));
            }
            let doc = xml_parsear(&resp_corpo)?;
            let ranges_cru = doc
                .primeiro_texto("Resultado/RangesRecebidos")
                .unwrap_or("")
                .to_string();
            let situacao_cru = doc
                .primeiro_texto("Resultado/Situacao")
                .unwrap_or("")
                .to_string();
            Ok(StatusUpload {
                protocolo: doc
                    .primeiro_texto("Resultado/Protocolo")
                    .unwrap_or("")
                    .to_string(),
                ranges_recebidos: parsear_ranges(&ranges_cru),
                ranges_recebidos_cru: ranges_cru,
                situacao: parsear_situacao_upload(&situacao_cru),
                situacao_cru,
            })
        })
    }

    /// Port de `SubmitRange` (ws.go:1121-1180, manual §5.6): PUT com
    /// `Content-Range: bytes i-f/total` (RFC 7233 §4.2), espera 200. Com as
    /// validações client-side do Go (inicio/fim/total/tamanho exato do chunk)
    /// MAIS o cap da especificação: chunk máximo de 8 MiB.
    pub fn enviar_range(
        &self,
        protocolo: &str,
        inicio: u64,
        fim: u64,
        total: u64,
        chunk: &[u8],
    ) -> Result<(), StaError> {
        let invalido = |motivo: String| {
            StaError::Validacao(RegError::InvalidValue {
                field: "chunk".to_string(),
                reason: motivo,
            })
        };
        if protocolo.is_empty() {
            return Err(StaError::Validacao(RegError::MissingField {
                field: "protocolo".to_string(),
            }));
        }
        if total == 0 {
            return Err(invalido("total deve ser > 0".to_string()));
        }
        let fim_mais_um = fim
            .checked_add(1)
            .ok_or_else(|| invalido("fim+1 estoura u64".to_string()))?;
        if total < fim_mais_um {
            return Err(invalido(format!(
                "total deve ser > 0 e >= fim+1 (got total={total}, fim={fim})"
            )));
        }
        if fim < inicio {
            return Err(invalido(format!(
                "fim deve ser >= inicio (inicio={inicio}, fim={fim})"
            )));
        }
        let esperado_len = fim - inicio + 1;
        if chunk.len() as u64 != esperado_len {
            return Err(invalido(format!(
                "len(chunk) deve ser fim-inicio+1={esperado_len} (got {})",
                chunk.len()
            )));
        }
        if chunk.len() > MAX_CHUNK_UPLOAD_BYTES {
            return Err(invalido(format!(
                "chunk de {} bytes excede o máximo de {} bytes (8 MiB)",
                chunk.len(),
                MAX_CHUNK_UPLOAD_BYTES
            )));
        }
        let content_range = format!("bytes {inicio}-{fim}/{total}");
        let url = format!("{}/arquivos/{protocolo}/conteudo", self.base_url);
        self.retry.executar(|| {
            let mut resp = self
                .requisicao(reqwest::Method::PUT, &url)
                .header(reqwest::header::CONTENT_RANGE, content_range.as_str())
                // Content-Type omitido — manual §5.6 (ws.go:1160).
                .body(chunk.to_vec())
                .send()?;
            let resp_corpo = ler_corpo_limitado(&mut resp, MAX_CORPO_RESPOSTA_BYTES as u64)?;
            let status = resp.status().as_u16();
            if status != 200 {
                return Err(erro_bacen_de_corpo(status, &resp_corpo, protocolo));
            }
            Ok(())
        })
    }

    /// Port de `Download` (ws.go:667-767, manual §6.1.1): GET
    /// /arquivos/{p}/conteudo com validação X-Content-Hash OBRIGATÓRIA —
    /// ausente → `ContentHashAusente`; malformado → `ContentHashHeaderMalformed`;
    /// divergente → `ContentHashMismatch`. Cap de 100 MiB → `ErroBacen{413}`
    /// (ws.go:694-713). Nenhum desses erros é retryable.
    pub fn baixar(&self, protocolo: &str) -> Result<ResultadoDownload, StaError> {
        if protocolo.is_empty() {
            return Err(StaError::Validacao(RegError::MissingField {
                field: "protocolo".to_string(),
            }));
        }
        let url = format!("{}/arquivos/{protocolo}/conteudo", self.base_url);
        self.retry.executar(|| {
            let mut resp = self.requisicao(reqwest::Method::GET, &url).send()?;
            // Lê o body com cap ANTES do status check — mesma ordem do
            // ws.go:696-719 (overflow vira 413 mesmo em resposta de erro).
            let corpo = ler_corpo_limitado(&mut resp, MAX_CORPO_DOWNLOAD_BYTES as u64)?;
            let status = resp.status().as_u16();
            if status != 200 {
                return Err(erro_bacen_de_corpo(status, &corpo, protocolo));
            }
            let header_cru = resp
                .headers()
                .get("X-Content-Hash")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| StaError::ContentHashAusente {
                    protocolo: protocolo.to_string(),
                })?;
            let esperado = parsear_x_content_hash(header_cru)?;
            let obtido = hex::encode(Sha256::digest(&corpo));
            if obtido != esperado {
                return Err(StaError::ContentHashMismatch { esperado, obtido });
            }
            Ok(ResultadoDownload {
                conteudo: corpo,
                content_hash: obtido,
                etag: valor_header(resp.headers(), "ETag"),
                last_modified: valor_header(resp.headers(), "Last-Modified"),
                content_hash_header: header_cru.to_string(),
            })
        })
    }

    /// Port de `ListDisponiveis` (ws.go:909-989, manual §8.1.1): uma página de
    /// até 1000 registros; páginas seguintes via `seguir_pagina`.
    pub fn listar_disponiveis(
        &self,
        opcoes: &OpcoesDisponiveis,
    ) -> Result<ResultadoDisponiveis, StaError> {
        if opcoes.data_hora_inicio.is_empty() {
            return Err(StaError::Validacao(RegError::MissingField {
                field: "dataHoraInicio".to_string(),
            }));
        }
        let mut consulta: Vec<(&str, &str)> =
            vec![("dataHoraInicio", opcoes.data_hora_inicio.as_str())];
        for (chave, valor) in [
            (
                "identificadorDocumento",
                opcoes.identificador_documento.as_deref(),
            ),
            ("sistemas", opcoes.sistemas.as_deref()),
            ("dependencia", opcoes.dependencia.as_deref()),
        ] {
            if let Some(v) = valor
                && !v.is_empty()
            {
                consulta.push((chave, v));
            }
        }
        let url = format!("{}/arquivos/disponiveis", self.base_url);
        self.retry.executar(|| {
            let mut resp = self
                .requisicao(reqwest::Method::GET, &url)
                .query(&consulta)
                .send()?;
            extrair_disponiveis(&mut resp)
        })
    }

    /// Segue a URL do `atom:link` da página anterior. A URL vinda do BACEN é
    /// revalidada contra a allowlist ANTES de conectar (defesa contra BACEN
    /// mal-comportado/deviance de proxy — o v1 seguia qualquer URL).
    pub fn seguir_pagina(&self, url_proxima: &str) -> Result<ResultadoDisponiveis, StaError> {
        validar_url_permitida(url_proxima)?;
        self.buscar_pagina(url_proxima)
    }

    /// Busca direta de página (privada — só alcançável via `listar_disponiveis`
    /// com URL validada ou `seguir_pagina` com allowlist).
    fn buscar_pagina(&self, url: &str) -> Result<ResultadoDisponiveis, StaError> {
        self.retry.executar(|| {
            let mut resp = self.requisicao(reqwest::Method::GET, url).send()?;
            extrair_disponiveis(&mut resp)
        })
    }

    /// Port de `AlterarSituacao` (ws.go:1010-1063, manual §7.1): PUT
    /// /arquivos/situacao, espera 204, e é o ÚNICO endpoint que exige
    /// Content-Type application/xml (ws.go:996-998).
    pub fn alterar_situacao(
        &self,
        protocolos: &[String],
        situacao: SituacaoTransferencia,
    ) -> Result<(), StaError> {
        if protocolos.is_empty() {
            return Err(StaError::Validacao(RegError::InvalidValue {
                field: "protocolos".to_string(),
                reason: "Protocolos não pode ser vazio (Seção 7.1)".to_string(),
            }));
        }
        let corpo = xml_parametros_situacao(&protocolos.join(";"), situacao.como_str());
        let url = format!("{}/arquivos/situacao", self.base_url);
        self.retry.executar(|| {
            let mut resp = self
                .requisicao(reqwest::Method::PUT, &url)
                .header(reqwest::header::CONTENT_TYPE, "application/xml")
                .header(reqwest::header::ACCEPT, "application/xml")
                .body(corpo.clone().into_bytes())
                .send()?;
            let resp_corpo = ler_corpo_limitado(&mut resp, MAX_CORPO_RESPOSTA_BYTES as u64)?;
            let status = resp.status().as_u16();
            if status != 204 {
                return Err(erro_bacen_de_corpo(status, &resp_corpo, ""));
            }
            Ok(())
        })
    }
}

// ============================================================
// Helpers de HTTP/XML (port de ws.go:463-503/777-802 e ws_types.go:197-226)
// ============================================================

/// Lê o corpo com `io::take`-equivalente (port do `io.LimitReader` de
/// ws.go:399/551/696): excedente → `ErroBacen{413}` (ws.go:703-713).
pub(crate) fn ler_corpo_limitado(
    resp: &mut reqwest::blocking::Response,
    capa: u64,
) -> Result<Vec<u8>, StaError> {
    let mut buf = Vec::new();
    resp.take(capa.saturating_add(1))
        .read_to_end(&mut buf)
        .map_err(StaError::LeituraCorpo)?;
    if buf.len() as u64 > capa {
        return Err(StaError::ErroBacen {
            status: 413,
            codigo: "HTTP_413".to_string(),
            mensagem: format!("body excede {capa} bytes (cap defensivo)"),
            protocolo: String::new(),
        });
    }
    Ok(buf)
}

/// Port de `parseSTAError`/`parseSTAErrorTyped` (ws.go:463-503): XML
/// `<Resultado><Erro><Codigo>N</Codigo><Descricao>MSG</Descricao></Erro></Resultado>`
/// → `ErroBacen` tipado; corpo não-parseável → fallback com cru truncado.
pub(crate) fn erro_bacen_de_corpo(status: u16, corpo: &[u8], protocolo: &str) -> StaError {
    if let Ok(doc) = xml_parsear(corpo)
        && let Some(codigo_cru) = doc.primeiro_texto("Resultado/Erro/Codigo")
        && let Ok(codigo_num) = codigo_cru.trim().parse::<i64>()
        && codigo_num != 0
    {
        let mensagem = doc
            .primeiro_texto("Resultado/Erro/Descricao")
            .unwrap_or("")
            .to_string();
        return StaError::ErroBacen {
            status,
            codigo: codigo_num.to_string(),
            mensagem,
            protocolo: protocolo.to_string(),
        };
    }
    StaError::ErroBacen {
        status,
        codigo: format!("HTTP_{status}"),
        mensagem: truncar_bytes(corpo, 200),
        protocolo: protocolo.to_string(),
    }
}

/// Port de `parseXContentHash` (ws.go:777-794): formato `SHA-256 {64-hex}`
/// (algoritmo case-insensitive, hash validado como hex e normalizado lowercase).
pub(crate) fn parsear_x_content_hash(header: &str) -> Result<String, StaError> {
    let malformado = |motivo: String| StaError::ContentHashHeaderMalformed {
        motivo: format!("{motivo} (header={header:?})"),
    };
    let header = header.trim();
    let Some((algo, hash)) = header.split_once(' ') else {
        return Err(malformado(format!(
            "formato esperado 'SHA-256 <hash>', got {header:?}"
        )));
    };
    if !algo.trim().eq_ignore_ascii_case("SHA-256") {
        return Err(malformado(format!(
            "algoritmo esperado SHA-256, got {algo:?}"
        )));
    }
    let hash = hash.trim();
    if hash.len() != 64 {
        return Err(malformado(format!(
            "hash deve ter 64 chars hex, got {}",
            hash.len()
        )));
    }
    if hex::decode(hash).is_err() {
        return Err(malformado(format!("hash não-hex ({} chars)", hash.len())));
    }
    Ok(hash.to_lowercase())
}

/// Port de `parseRanges` (ws_types.go:200-226): "0-3;5-8" → intervalos;
/// partes malformadas são descartadas (o cru fica preservado no caller —
/// melhor que o v1, que não guardava o original).
pub(crate) fn parsear_ranges(cru: &str) -> Vec<Intervalo> {
    let mut out = Vec::new();
    for parte in cru.split(';') {
        let parte = parte.trim();
        if parte.is_empty() {
            continue;
        }
        let Some((inicio_cru, fim_cru)) = parte.split_once('-') else {
            continue;
        };
        let (Ok(inicio), Ok(fim)) = (
            inicio_cru.trim().parse::<i64>(),
            fim_cru.trim().parse::<i64>(),
        ) else {
            continue;
        };
        if inicio < 0 || fim < inicio {
            continue;
        }
        out.push(Intervalo { inicio, fim });
    }
    out
}

/// Port de `truncate` (ws.go:797-802).
pub(crate) fn truncar_bytes(b: &[u8], n: usize) -> String {
    if b.len() <= n {
        String::from_utf8_lossy(b).into_owned()
    } else {
        format!("{}...", String::from_utf8_lossy(&b[..n]))
    }
}

fn valor_header(headers: &reqwest::header::HeaderMap, nome: &str) -> String {
    headers
        .get(nome)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

// ============================================================
// XML — construção (port dos marshal de ws.go:372-378/1030-1034)
// ============================================================

/// Escape XML de conteúdo de texto via quick-xml (o mesmo que
/// `BytesText::new` faz por baixo).
fn escapar(valor: &str) -> String {
    quick_xml::escape::escape(valor).into_owned()
}

/// Declaração exigida pelo manual (ws.go:376-378): `<?xml ... standalone="yes"?>`
/// + newline + `<Parametros>…`.
const DECLARACAO_XML: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";

/// Corpo do POST /arquivos — port de `requestProtocolParams` (ws_types.go:32-40
/// com omitempty de Observacao/Destinatarios, que não aplicam ao fluxo).
pub(crate) fn xml_parametros_protocolo(cadoc: &str, hash_hex: &str, tamanho: u64) -> String {
    format!(
        "{DECLARACAO_XML}<Parametros><IdentificadorDocumento>{}</IdentificadorDocumento><Hash>{}\
</Hash><Tamanho>{}</Tamanho><NomeArquivo>{}</NomeArquivo></Parametros>",
        escapar(cadoc),
        escapar(hash_hex),
        tamanho,
        escapar(&format!("{cadoc}.zip")),
    )
}

/// Corpo do PUT /arquivos/situacao — port de `situacaoParams` (ws_types.go:87-91).
pub(crate) fn xml_parametros_situacao(protocolos: &str, situacao: &str) -> String {
    format!(
        "{DECLARACAO_XML}<Parametros><Protocolos>{}</Protocolos><Situacao>{}</Situacao>\
</Parametros>",
        escapar(protocolos),
        escapar(situacao),
    )
}

// ============================================================
// XML — parsing (substituto do encoding/xml de ws_types.go)
// ============================================================

/// Item coletado na ordem do documento: abertura de elemento (com atributos)
/// ou texto de elemento. Caminhos são locais sem namespace (`atom:link` →
/// `link`), juntados por `/` (ex.: `Resultado/Erro/Codigo`).
#[derive(Debug, Clone, PartialEq, Eq)]
enum ItemXml {
    Abertura {
        caminho: String,
        atributos: Vec<(String, String)>,
    },
    Texto {
        caminho: String,
        valor: String,
    },
}

/// Documento XML plano em ordem — a base para os mesmos de-serializations
/// que o encoding/xml fazia em ws_types.go.
#[derive(Debug, Default)]
pub(crate) struct XmlDoc {
    itens: Vec<ItemXml>,
}

impl XmlDoc {
    /// Primeiro texto no caminho dado.
    pub(crate) fn primeiro_texto(&self, caminho: &str) -> Option<&str> {
        self.itens.iter().find_map(|item| match item {
            ItemXml::Abertura { .. } => None,
            ItemXml::Texto { caminho: c, valor } if c == caminho => Some(valor.as_str()),
            ItemXml::Texto { .. } => None,
        })
    }

    /// Valor de atributo da primeira abertura no caminho dado.
    fn atributo(&self, caminho: &str, nome_attr: &str) -> Option<&str> {
        self.itens.iter().find_map(|item| match item {
            ItemXml::Abertura {
                caminho: c,
                atributos,
            } if c == caminho => atributos.iter().find_map(|(n, v)| {
                if n == nome_attr {
                    Some(v.as_str())
                } else {
                    None
                }
            }),
            ItemXml::Abertura { .. } | ItemXml::Texto { .. } => None,
        })
    }

    /// Extrai os registros `<Arquivo>` da listagem (§8.1.1), agrupando os
    /// textos por abertura de `Resultado/Arquivo`.
    fn extrair_arquivos(&self) -> Result<Vec<ArquivoDisponivel>, StaError> {
        let mut registros: Vec<Vec<(String, String)>> = Vec::new();
        for item in &self.itens {
            match item {
                ItemXml::Abertura { caminho, .. } if caminho == "Resultado/Arquivo" => {
                    registros.push(Vec::new());
                }
                ItemXml::Texto { caminho, valor } => {
                    if let Some(campo) = caminho.strip_prefix("Resultado/Arquivo/")
                        && let Some(registro) = registros.last_mut()
                    {
                        registro.push((campo.to_string(), valor.clone()));
                    }
                }
                ItemXml::Abertura { .. } => {}
            }
        }
        registros
            .into_iter()
            .filter(|registro| !registro.is_empty())
            .map(|registro| registro_para_arquivo(&registro))
            .collect()
    }
}

/// Port do de-serialization de `arquivosDisponiveisResponse` (ws_types.go:101-122):
/// campos ausentes = zero-value; inteiro presente e inválido = erro (mesmo
/// comportamento do encoding/xml).
fn registro_para_arquivo(registro: &[(String, String)]) -> Result<ArquivoDisponivel, StaError> {
    let campo = |nome: &str| {
        registro
            .iter()
            .find(|(c, _)| c == nome)
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    };
    let inteiro = |nome: &str| -> Result<i64, StaError> {
        match registro.iter().find(|(c, _)| c == nome) {
            None => Ok(0),
            Some((_, v)) => v
                .trim()
                .parse::<i64>()
                .map_err(|_| StaError::RespostaInvalida {
                    motivo: format!("<{nome}> não é inteiro válido: {v:?}"),
                }),
        }
    };
    let codigo_situacao = inteiro("SituacaoAtual/Codigo")?;
    Ok(ArquivoDisponivel {
        protocolo: campo("Protocolo").to_string(),
        tipo_arquivo: campo("TipoArquivo").to_string(),
        codigo_documento: campo("CodigoDocumento").to_string(),
        sistema: campo("Sistema").to_string(),
        tamanho_arquivo: inteiro("TamanhoArquivo")?,
        hash: campo("Hash").to_string(),
        situacao_atual: parsear_situacao_arquivo(codigo_situacao),
        situacao_atual_cru: campo("SituacaoAtual/Descricao").to_string(),
        data_hora_disponibilizacao: campo("DataHoraDisponibilizacao").to_string(),
    })
}

/// Parseia o XML em `XmlDoc` ordenado. quick-xml 0.42 NÃO des-escapa texto:
/// `Text` chega cru (com entidades) e `GeneralRef` carrega `&ref;` — ambos
/// passam por `escape::unescape`. Nomes são locais (sem prefixo de namespace).
pub(crate) fn xml_parsear(bytes: &[u8]) -> Result<XmlDoc, StaError> {
    let mut reader = quick_xml::Reader::from_reader(bytes);
    let mut pilha: Vec<String> = Vec::new();
    let mut buf = Vec::new();
    let mut doc = XmlDoc::default();
    loop {
        buf.clear();
        let evento = reader
            .read_event_into(&mut buf)
            .map_err(|e| StaError::RespostaInvalida {
                motivo: format!("XML mal-formado na resposta do BACEN: {e}"),
            })?;
        match evento {
            Event::Start(e) => {
                let local = e.name().local_name().into_inner().to_string();
                let atributos = coletar_atributos(&e)?;
                let caminho = junta_caminho(&pilha, &local);
                doc.itens.push(ItemXml::Abertura {
                    caminho: caminho.clone(),
                    atributos,
                });
                pilha.push(local);
            }
            Event::End(_) => {
                pilha.pop();
            }
            Event::Empty(e) => {
                let local = e.name().local_name().into_inner().to_string();
                let atributos = coletar_atributos(&e)?;
                let caminho = junta_caminho(&pilha, &local);
                doc.itens.push(ItemXml::Abertura { caminho, atributos });
            }
            Event::Text(t) => {
                let texto = des_escapar(&t)?;
                if !texto.is_empty() {
                    doc.itens.push(ItemXml::Texto {
                        caminho: pilha.join("/"),
                        valor: texto,
                    });
                }
            }
            Event::GeneralRef(r) => {
                let texto = des_escapar(&r)?;
                if !texto.is_empty() {
                    doc.itens.push(ItemXml::Texto {
                        caminho: pilha.join("/"),
                        valor: texto,
                    });
                }
            }
            Event::Eof => break,
            Event::CData(_)
            | Event::Comment(_)
            | Event::Decl(_)
            | Event::PI(_)
            | Event::DocType(_) => {}
        }
    }
    Ok(doc)
}

fn des_escapar(cru: &str) -> Result<String, StaError> {
    let valor = quick_xml::escape::unescape(cru).map_err(|e| StaError::RespostaInvalida {
        motivo: format!("entidade XML inválida: {e}"),
    })?;
    Ok(valor.trim().to_string())
}

fn coletar_atributos(e: &BytesStart<'_>) -> Result<Vec<(String, String)>, StaError> {
    e.attributes()
        .map(|a| {
            let a = a.map_err(|err| StaError::RespostaInvalida {
                motivo: format!("atributo XML inválido: {err}"),
            })?;
            let nome = a.key.local_name().into_inner().to_string();
            // normalized_value resolve entidades predefinidas (&amp; em URLs
            // de atom:link) e normaliza whitespace conforme XML 1.0.
            let valor = a
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .map_err(|err| StaError::RespostaInvalida {
                    motivo: format!("valor de atributo XML inválido: {err}"),
                })?
                .into_owned();
            Ok((nome, valor))
        })
        .collect()
}

fn junta_caminho(pilha: &[String], local: &str) -> String {
    if pilha.is_empty() {
        local.to_string()
    } else {
        format!("{}/{}", pilha.join("/"), local)
    }
}

/// Extração compartilhada de `listar_disponiveis`/`buscar_pagina`.
fn extrair_disponiveis(
    resp: &mut reqwest::blocking::Response,
) -> Result<ResultadoDisponiveis, StaError> {
    let resp_corpo = ler_corpo_limitado(resp, MAX_CORPO_RESPOSTA_BYTES as u64)?;
    let status = resp.status().as_u16();
    if status != 200 {
        return Err(erro_bacen_de_corpo(status, &resp_corpo, ""));
    }
    let doc = xml_parsear(&resp_corpo)?;
    let arquivos = doc.extrair_arquivos()?;
    let href = doc
        .atributo("Resultado/link", "href")
        .unwrap_or("")
        .to_string();
    let tem_proxima_pagina = !href.is_empty();
    Ok(ResultadoDisponiveis {
        arquivos,
        data_hora_proxima_consulta: doc
            .primeiro_texto("Resultado/DataHoraProximaConsulta")
            .unwrap_or("")
            .to_string(),
        proxima_pagina_url: if tem_proxima_pagina { Some(href) } else { None },
        tem_proxima_pagina,
    })
}

// ============================================================
// TESTES — mock HTTP à mão sobre std::net::TcpListener (sem crate extra).
// NUNCA conectam ao BACEN real: o cliente é construído com `para_testes`
// apontando a http://127.0.0.1:0 (a allowlist de produção é exercida por
// testes dedicados SEM rede).
// ============================================================
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
pub(crate) mod tests {
    use super::*;
    use crate::secrets::ProvedorMemoria;
    use std::io::Write as _;
    use std::net::{TcpListener, TcpStream};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::{io::BufRead, sync::Arc};

    /// Captura de uma requisição recebida pelo mock.
    #[derive(Debug, Clone)]
    pub(crate) struct CapturaRequisicao {
        pub(crate) metodo: String,
        /// Path com query (ex. `/arquivos/disponiveis?dataHoraInicio=...`).
        pub(crate) caminho: String,
        pub(crate) cabecalhos: Vec<(String, String)>,
        pub(crate) corpo: Vec<u8>,
    }

    impl CapturaRequisicao {
        pub(crate) fn cabecalho(&self, nome: &str) -> Option<&str> {
            self.cabecalhos
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(nome))
                .map(|(_, v)| v.as_str())
        }
    }

    type Tratador = Arc<dyn Fn(&CapturaRequisicao) -> Vec<u8> + Send + Sync>;

    /// Servidor HTTP de mock local. Responde o que o tratador devolver
    /// (bytes HTTP crus) e grava cada requisição para as asserções dos testes.
    pub(crate) struct ServidorHttp {
        pub(crate) url: String,
        capturas: Arc<Mutex<Vec<CapturaRequisicao>>>,
        vivo: Arc<AtomicBool>,
        thread: Option<std::thread::JoinHandle<()>>,
    }

    impl ServidorHttp {
        pub(crate) fn iniciar(tratador: Tratador) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock em 127.0.0.1:0");
            let url = format!(
                "http://{}",
                listener.local_addr().expect("local_addr do mock")
            );
            let vivo = Arc::new(AtomicBool::new(true));
            let capturas = Arc::new(Mutex::new(Vec::new()));
            let vivo_thread = vivo.clone();
            let capturas_thread = capturas.clone();
            let thread = std::thread::spawn(move || {
                let _ = listener.set_nonblocking(true);
                while vivo_thread.load(Ordering::Relaxed) {
                    match listener.accept() {
                        // Cada conexão numa thread própria: o accept nunca
                        // fica preso atrás de uma resposta lenta e o hyper
                        // nunca vê conexão fechada por timeout do accept.
                        Ok((stream, _)) => {
                            let tratador = tratador.clone();
                            let capturas = capturas_thread.clone();
                            let _ = std::thread::spawn(move || {
                                // macOS/BSD: socket aceito HERDA O_NONBLOCK do
                                // listener — sem isto, read_line devolve
                                // WouldBlock antes dos bytes chegarem e o mock
                                // fecharia a conexão sem responder (fonte de
                                // IncompleteMessage flaky nos retries).
                                let _ = stream.set_nonblocking(false);
                                tratar_conexao(stream, &tratador, &capturas)
                            });
                        }
                        Err(_) => std::thread::sleep(Duration::from_millis(1)),
                    }
                }
            });
            Self {
                url,
                capturas,
                vivo,
                thread: Some(thread),
            }
        }

        pub(crate) fn capturas(&self) -> Vec<CapturaRequisicao> {
            self.capturas
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .clone()
        }

        pub(crate) fn total_capturas(&self) -> usize {
            self.capturas
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .len()
        }
    }

    impl Drop for ServidorHttp {
        fn drop(&mut self) {
            self.vivo.store(false, Ordering::Relaxed);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }

    /// Trata a conexão em loop de keep-alive: lê UMA requisição (linha de
    /// status + headers + body por Content-Length), registra, responde e
    /// AGUARDA a próxima requisição na MESMA conexão. Fecha apenas quando o
    /// cliente fecha (EOF). NUNCA o mock fecha primeiro — isso eliminava o
    /// race de reuso do pool do hyper (conexão morta no pool →
    /// `IncompleteMessage` flaky nos retries).
    fn tratar_conexao(
        mut stream: TcpStream,
        tratador: &Tratador,
        capturas: &Mutex<Vec<CapturaRequisicao>>,
    ) {
        let Ok(clone) = stream.try_clone() else {
            return;
        };
        let mut reader = std::io::BufReader::new(clone);
        let mut linha = String::new();
        loop {
            linha.clear();
            if reader.read_line(&mut linha).is_err()
                || linha.trim_end_matches(['\r', '\n']).is_empty()
            {
                return; // EOF ou requisição vazia — cliente fechou.
            }
            let mut partes = linha.split_whitespace();
            let metodo = partes.next().unwrap_or_default().to_string();
            let caminho = partes.next().unwrap_or_default().to_string();
            let mut content_length: usize = 0;
            let mut cabecalhos: Vec<(String, String)> = Vec::new();
            loop {
                linha.clear();
                if reader.read_line(&mut linha).is_err() {
                    return;
                }
                let l = linha.trim_end_matches(['\r', '\n']);
                if l.is_empty() {
                    break;
                }
                if let Some((nome, valor)) = l.split_once(':') {
                    cabecalhos.push((nome.trim().to_string(), valor.trim().to_string()));
                    if nome.trim().eq_ignore_ascii_case("content-length") {
                        content_length = valor.trim().parse().unwrap_or(0);
                    }
                }
            }
            let mut corpo = vec![0u8; content_length];
            if content_length > 0 && reader.read_exact(&mut corpo).is_err() {
                return;
            }
            let captura = CapturaRequisicao {
                metodo,
                caminho,
                cabecalhos,
                corpo,
            };
            capturas
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .push(captura.clone());
            let resposta = tratador(&captura);
            let _ = stream.write_all(&resposta);
            let _ = stream.flush();
        }
    }

    /// Monta bytes HTTP crus com Content-Length e `Connection: keep-alive`
    /// (a conexão é fechada pelo CLIENTE no fim do teste — o mock nunca
    /// fecha primeiro, ver doc de `tratar_conexao`).
    pub(crate) fn resposta_http(
        status: u16,
        motivo: &str,
        cabecalhos: &[(&str, &str)],
        corpo: &[u8],
    ) -> Vec<u8> {
        let mut s = format!("HTTP/1.1 {status} {motivo}\r\n");
        let mut tem_length = false;
        for (nome, valor) in cabecalhos {
            s.push_str(&format!("{nome}: {valor}\r\n"));
            if nome.eq_ignore_ascii_case("content-length") {
                tem_length = true;
            }
        }
        if !tem_length {
            s.push_str(&format!("Content-Length: {}\r\n", corpo.len()));
        }
        s.push_str("Connection: keep-alive\r\n\r\n");
        let mut bytes = s.into_bytes();
        bytes.extend_from_slice(corpo);
        bytes
    }

    /// base64 RFC 4648 std (com padding) — só para as asserções do header
    /// Authorization nos testes (o cliente usa `RequestBuilder::basic_auth`).
    pub(crate) fn base64_padrao(dados: &[u8]) -> String {
        const TABELA: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for trio in dados.chunks(3) {
            let b = [
                trio[0],
                *trio.get(1).unwrap_or(&0),
                *trio.get(2).unwrap_or(&0),
            ];
            let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
            out.push(TABELA[(n >> 18) as usize & 63] as char);
            out.push(TABELA[(n >> 12) as usize & 63] as char);
            out.push(if trio.len() > 1 {
                TABELA[(n >> 6) as usize & 63] as char
            } else {
                '='
            });
            out.push(if trio.len() > 2 {
                TABELA[n as usize & 63] as char
            } else {
                '='
            });
        }
        out
    }

    // ------------------------------------------------------------
    // helpers de construção de cliente/mock
    // ------------------------------------------------------------

    const USUARIO_TESTE: &str = "12345/0001.fulano";

    fn cliente_para(url: &str) -> StaClient {
        StaClient::para_testes(
            url,
            USUARIO_TESTE,
            "senha-test",
            Duration::from_secs(5),
            // Retry rápido para os testes contarem tentativas sem dormir 3s.
            RetryPolicy::nova(3, Duration::from_millis(2), 5_000).unwrap(),
        )
    }

    fn xml_protocolo(protocolo: &str) -> String {
        format!("<Resultado><Protocolo>{protocolo}</Protocolo></Resultado>")
    }

    fn xml_erro(status: u16, descricao: &str) -> String {
        format!(
            "<Resultado><Erro><Codigo>{status}</Codigo><Descricao>{descricao}</Descricao></Erro></Resultado>"
        )
    }

    // ------------------------------------------------------------
    // Fluxo de envio completo + auth preemptiva
    // ------------------------------------------------------------

    #[test]
    fn fluxo_completo_201_upload_200() {
        let servidor = ServidorHttp::iniciar(Arc::new(|captura| {
            if captura.metodo == "POST" && captura.caminho == "/arquivos" {
                // Content-Type application/xml no POST (ws.go:386).
                assert_eq!(captura.cabecalho("content-type"), Some("application/xml"));
                // Auth Basic PREEMPTIVA — presente na 1ª tentativa.
                assert!(
                    captura
                        .cabecalho("authorization")
                        .unwrap_or("")
                        .starts_with("Basic "),
                    "Authorization Basic preemptiva ausente no POST"
                );
                return resposta_http(
                    201,
                    "Created",
                    &[("Content-Type", "application/xml")],
                    xml_protocolo("123").as_bytes(),
                );
            }
            if captura.metodo == "PUT" && captura.caminho == "/arquivos/123/conteudo" {
                assert!(!captura.corpo.is_empty());
                // uploadContent NÃO envia Content-Type (ws.go:428-429).
                assert!(
                    captura.cabecalho("content-type").is_none(),
                    "Content-Type não deveria ser enviado no PUT de conteúdo"
                );
                return resposta_http(200, "OK", &[], b"");
            }
            resposta_http(404, "Not Found", &[], b"nao implementado no mock")
        }));
        let cliente = cliente_para(&servidor.url);
        let payload = b"PAYLOAD-ZIP-TESTE".to_vec();

        let resultado = cliente.enviar("3040", &payload).unwrap();

        assert!(
            resultado.aceito,
            "envio deveria ser aceito: {:?}",
            resultado
        );
        assert_eq!(resultado.protocolo, "123");
        assert!(resultado.rejeicao.is_none());

        let capturas = servidor.capturas();
        assert_eq!(capturas.len(), 2);
        let hash_esperado = hex::encode(Sha256::digest(&payload));
        let corpo_post = String::from_utf8_lossy(&capturas[0].corpo).into_owned();
        assert!(
            corpo_post.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"),
            "declaração XML ausente: {corpo_post}"
        );
        assert!(
            corpo_post.contains("<IdentificadorDocumento>3040</IdentificadorDocumento>"),
            "{corpo_post}"
        );
        assert!(
            corpo_post.contains(&format!("<Hash>{hash_esperado}</Hash>")),
            "{corpo_post}"
        );
        assert!(
            corpo_post.contains(&format!("<Tamanho>{}</Tamanho>", payload.len())),
            "{corpo_post}"
        );
        assert!(
            corpo_post.contains("<NomeArquivo>3040.zip</NomeArquivo>"),
            "{corpo_post}"
        );
        assert_eq!(capturas[1].metodo, "PUT");
        assert_eq!(capturas[1].caminho, "/arquivos/123/conteudo");
        assert_eq!(capturas[1].corpo, payload);
    }

    #[test]
    fn auth_basic_preemptiva_carrega_usuario_e_senha() {
        let auth_esperada = format!("Basic {}", base64_padrao(b"12345/0001.fulano:senha-test"));
        let servidor = ServidorHttp::iniciar(Arc::new(move |captura| {
            assert_eq!(
                captura.cabecalho("authorization"),
                Some(auth_esperada.as_str()),
                "header Authorization Basic não é base64(user:senha) preemptivo"
            );
            if captura.metodo == "PUT" {
                return resposta_http(200, "OK", &[], b"");
            }
            resposta_http(
                201,
                "Created",
                &[("Content-Type", "application/xml")],
                xml_protocolo("7").as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let resultado = cliente.enviar("3040", b"abc").unwrap();
        assert!(resultado.aceito);
        assert_eq!(servidor.total_capturas(), 2); // POST + PUT
    }

    #[test]
    fn envio_vazio_e_validacao_sem_rede() {
        let servidor = ServidorHttp::iniciar(Arc::new(|_| resposta_http(500, "Err", &[], b"")));
        let cliente = cliente_para(&servidor.url);
        let err = cliente.enviar("3040", b"").unwrap_err();
        let StaError::Validacao(RegError::InvalidValue { field, reason }) = &err else {
            panic!("esperava Validacao, got {err:?}")
        };
        assert_eq!(field, "payload");
        assert!(reason.contains("vazia"), "{reason}");
        assert_eq!(
            servidor.total_capturas(),
            0,
            "nenhuma requisição deveria sair"
        );
    }

    // ------------------------------------------------------------
    // Retry — 4xx NUNCA, 5xx sim (port de retry_test.go:191/255)
    // ------------------------------------------------------------

    #[test]
    fn enviar_400_sem_retry() {
        let servidor = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(
                400,
                "Bad Request",
                &[("Content-Type", "application/xml")],
                xml_erro(400, "Parâmetro 'IdentificadorDocumento' inválido").as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let err = cliente.enviar("3040", b"abc").unwrap_err();
        let StaError::ErroBacen {
            status, mensagem, ..
        } = &err
        else {
            panic!("esperava ErroBacen, got {err:?}")
        };
        assert_eq!(*status, 400);
        assert!(mensagem.contains("IdentificadorDocumento"), "{mensagem}");
        assert_eq!(servidor.total_capturas(), 1, "4xx NUNCA re-tenta");
    }

    #[test]
    fn enviar_503_reintenta_e_conta_tentativas() {
        let contador = Arc::new(AtomicUsize::new(0));
        let contador_handler = contador.clone();
        let servidor = ServidorHttp::iniciar(Arc::new(move |captura| {
            if captura.metodo == "PUT" {
                return resposta_http(200, "OK", &[], b"");
            }
            let n = contador_handler.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                resposta_http(
                    503,
                    "Service Unavailable",
                    &[("Content-Type", "application/xml")],
                    xml_erro(503, "Serviço indisponível").as_bytes(),
                )
            } else {
                resposta_http(
                    201,
                    "Created",
                    &[("Content-Type", "application/xml")],
                    xml_protocolo("123").as_bytes(),
                )
            }
        }));
        let cliente = cliente_para(&servidor.url);
        let resultado = cliente.enviar("3040", b"XYZ").unwrap();
        assert!(resultado.aceito);
        assert_eq!(
            contador.load(Ordering::SeqCst),
            3,
            "2×503 + 1×201 = 3 tentativas do POST"
        );
        assert_eq!(
            servidor.total_capturas(),
            4,
            "3 POSTs (retry) + 1 PUT da fase 2"
        );
    }

    #[test]
    fn enviar_503_esgota_tentativas_e_wrapa_erro() {
        let servidor = ServidorHttp::iniciar(Arc::new(|captura| {
            if captura.metodo == "PUT" {
                return resposta_http(200, "OK", &[], b"");
            }
            resposta_http(
                503,
                "Service Unavailable",
                &[("Content-Type", "application/xml")],
                xml_erro(503, "Serviço indisponível").as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let err = cliente.enviar("3040", b"XYZ").unwrap_err();
        let StaError::AposTentativas { tentativas, causa } = &err else {
            panic!("esperava AposTentativas, got {err:?}")
        };
        assert_eq!(*tentativas, 3);
        let StaError::ErroBacen { status, .. } = causa.as_ref() else {
            panic!("causa deveria ser ErroBacen 503, got {causa:?}")
        };
        assert_eq!(*status, 503);
        assert_eq!(servidor.total_capturas(), 3, "esgotou as 3 tentativas");
    }

    // ------------------------------------------------------------
    // Submit — rejeição de upload preserva protocolo (ws_test.go:392)
    // ------------------------------------------------------------

    #[test]
    fn upload_403_rejeicao_preserva_protocolo() {
        let servidor = ServidorHttp::iniciar(Arc::new(|captura| {
            if captura.metodo == "POST" {
                return resposta_http(
                    201,
                    "Created",
                    &[("Content-Type", "application/xml")],
                    xml_protocolo("456").as_bytes(),
                );
            }
            resposta_http(
                403,
                "Forbidden",
                &[("Content-Type", "application/xml")],
                xml_erro(403, "Protocolo não pertence à instituição").as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let resultado = cliente.enviar("3040", b"payload").unwrap();
        // Rejeição NÃO é erro (Result com protocolo preservado — ws.go:327-341).
        assert!(!resultado.aceito);
        assert_eq!(resultado.protocolo, "456");
        let rejeicao = resultado.rejeicao.expect("rejeicao populada");
        assert_eq!(rejeicao.codigo, "UPLOAD_FAILED");
        assert!(
            rejeicao.mensagem.contains("não pertence"),
            "{}",
            rejeicao.mensagem
        );
        assert_eq!(servidor.total_capturas(), 2);
    }

    // ------------------------------------------------------------
    // Download — X-Content-Hash obrigatório (ws_test.go:843-1055)
    // ------------------------------------------------------------

    #[test]
    fn baixar_sucesso_valida_hash() {
        let corpo = b"CONTEUDO-CADOC-REAL".to_vec();
        let hash = hex::encode(Sha256::digest(&corpo));
        let header_hash = format!("SHA-256 {hash}");
        let corpo_mock = corpo.clone();
        let servidor = ServidorHttp::iniciar(Arc::new(move |captura| {
            assert_eq!(captura.caminho, "/arquivos/999/conteudo");
            resposta_http(
                200,
                "OK",
                &[
                    ("Content-Type", "application/octet-stream"),
                    ("ETag", "\"etag-abc\""),
                    ("Last-Modified", "Wed, 12 Nov 2025 10:00:00 GMT"),
                    ("X-Content-Hash", header_hash.as_str()),
                ],
                &corpo_mock,
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let resultado = cliente.baixar("999").unwrap();
        assert_eq!(resultado.conteudo, corpo);
        assert_eq!(resultado.content_hash, hash);
        assert_eq!(resultado.etag, "\"etag-abc\"");
        assert_eq!(resultado.last_modified, "Wed, 12 Nov 2025 10:00:00 GMT");
        assert_eq!(resultado.content_hash_header, format!("SHA-256 {hash}"));
        assert_eq!(servidor.total_capturas(), 1);
    }

    #[test]
    fn baixar_hash_divergente_e_sem_retry() {
        let corpo = b"CONTEUDO-RECEBIDO".to_vec();
        let hash_errado = hex::encode(Sha256::digest(b"OUTRO-CONTEUDO"));
        let hash_assert = hash_errado.clone();
        let header_divergente = format!("SHA-256 {hash_errado}");
        let corpo_mock = corpo.clone();
        let servidor = ServidorHttp::iniciar(Arc::new(move |_| {
            resposta_http(
                200,
                "OK",
                &[("X-Content-Hash", header_divergente.as_str())],
                &corpo_mock,
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let err = cliente.baixar("999").unwrap_err();
        let StaError::ContentHashMismatch { esperado, obtido } = &err else {
            panic!("esperava ContentHashMismatch, got {err:?}")
        };
        assert_eq!(*esperado, hash_assert);
        assert_eq!(*obtido, hex::encode(Sha256::digest(&corpo)));
        assert_eq!(
            servidor.total_capturas(),
            1,
            "hash divergente NUNCA re-tenta"
        );
    }

    #[test]
    fn baixar_hash_ausente_e_sem_retry() {
        let servidor = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(200, "OK", &[], b"CONTEUDO-SEM-HEADER")
        }));
        let cliente = cliente_para(&servidor.url);
        let err = cliente.baixar("999").unwrap_err();
        let StaError::ContentHashAusente { protocolo } = &err else {
            panic!("esperava ContentHashAusente, got {err:?}")
        };
        assert_eq!(protocolo, "999");
        assert_eq!(servidor.total_capturas(), 1, "hash ausente NUNCA re-tenta");
    }

    #[test]
    fn baixar_410_e_erro_bacen() {
        let servidor = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(
                410,
                "Gone",
                &[("Content-Type", "application/xml")],
                xml_erro(410, "Arquivo não disponível").as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let err = cliente.baixar("999").unwrap_err();
        let StaError::ErroBacen {
            status,
            mensagem,
            protocolo,
            ..
        } = &err
        else {
            panic!("esperava ErroBacen 410, got {err:?}")
        };
        assert_eq!(*status, 410);
        assert_eq!(protocolo, "999");
        assert!(mensagem.contains("não disponível"), "{mensagem}");
        assert_eq!(servidor.total_capturas(), 1, "4xx NUNCA re-tenta");
    }

    // ------------------------------------------------------------
    // Range upload (manual §5.6, ws_test.go:1668-1758)
    // ------------------------------------------------------------

    #[test]
    fn enviar_range_content_range_e_validacoes_locais() {
        let servidor = ServidorHttp::iniciar(Arc::new(|captura| {
            assert_eq!(captura.metodo, "PUT");
            assert_eq!(captura.caminho, "/arquivos/55/conteudo");
            resposta_http(200, "OK", &[], b"")
        }));
        let cliente = cliente_para(&servidor.url);

        cliente.enviar_range("55", 0, 3, 10, b"abcd").unwrap();
        let capturas = servidor.capturas();
        assert_eq!(capturas.len(), 1);
        assert_eq!(capturas[0].cabecalho("content-range"), Some("bytes 0-3/10"));
        assert_eq!(capturas[0].corpo, b"abcd".to_vec());
        assert!(
            capturas[0].cabecalho("content-type").is_none(),
            "Content-Type omitido no range (manual §5.6)"
        );

        // Validações client-side — NENHUMA sai para a rede:
        assert!(
            cliente.enviar_range("55", 0, 3, 10, b"abc").is_err(),
            "len(chunk) != fim-inicio+1"
        );
        assert!(
            cliente.enviar_range("55", 0, 3, 2, b"abc").is_err(),
            "fim > total-1"
        );
        assert!(
            cliente.enviar_range("55", 3, 0, 10, b"abc").is_err(),
            "fim < inicio"
        );
        assert!(
            cliente.enviar_range("55", 0, 3, 0, b"abcd").is_err(),
            "total = 0"
        );
        assert!(
            cliente.enviar_range("", 0, 3, 10, b"abcd").is_err(),
            "protocolo vazio"
        );
        let grande = vec![0u8; MAX_CHUNK_UPLOAD_BYTES + 1];
        assert!(
            cliente
                .enviar_range(
                    "55",
                    0,
                    (grande.len() - 1) as u64,
                    grande.len() as u64,
                    &grande
                )
                .is_err(),
            "chunk acima de 8 MiB"
        );
        assert_eq!(servidor.total_capturas(), 1, "só o range válido foi à rede");
    }

    #[test]
    fn enviar_range_416_erro_bacen() {
        let servidor = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(
                416,
                "Range Not Satisfiable",
                &[("Content-Type", "application/xml")],
                xml_erro(416, "Range já enviado ou inválido").as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let err = cliente.enviar_range("55", 0, 3, 10, b"abcd").unwrap_err();
        let StaError::ErroBacen { status, .. } = &err else {
            panic!("esperava ErroBacen 416, got {err:?}")
        };
        assert_eq!(*status, 416);
        assert_eq!(servidor.total_capturas(), 1);
    }

    // ------------------------------------------------------------
    // StatusUpload (ws_test.go:686-841)
    // ------------------------------------------------------------

    #[test]
    fn posicao_upload_parseia_ranges_e_situacao() {
        let xml = concat!(
            "<Resultado><Protocolo>123</Protocolo>",
            "<RangesRecebidos>0-3;5-8</RangesRecebidos>",
            "<Situacao>Transmissão pendente</Situacao></Resultado>"
        );
        let servidor = ServidorHttp::iniciar(Arc::new(move |captura| {
            assert_eq!(captura.caminho, "/arquivos/123/posicaoupload");
            resposta_http(
                200,
                "OK",
                &[("Content-Type", "application/xml")],
                xml.as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let status = cliente.posicao_upload("123").unwrap();
        assert_eq!(status.protocolo, "123");
        assert_eq!(
            status.ranges_recebidos,
            vec![
                Intervalo { inicio: 0, fim: 3 },
                Intervalo { inicio: 5, fim: 8 }
            ]
        );
        assert_eq!(status.ranges_recebidos_cru, "0-3;5-8");
        assert_eq!(status.situacao, SituacaoUpload::Pendente);
        assert_eq!(status.situacao_cru, "Transmissão pendente");
    }

    #[test]
    fn posicao_upload_403_sem_retry() {
        let servidor = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(
                403,
                "Forbidden",
                &[("Content-Type", "application/xml")],
                xml_erro(403, "Usuário não autorizado").as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let err = cliente.posicao_upload("123").unwrap_err();
        let StaError::ErroBacen { status, .. } = &err else {
            panic!("esperava ErroBacen 403, got {err:?}")
        };
        assert_eq!(*status, 403);
        assert_eq!(servidor.total_capturas(), 1);
    }

    // ------------------------------------------------------------
    // ListDisponiveis + paginação via atom:link (ws_test.go:1125-1307)
    // ------------------------------------------------------------

    #[test]
    fn listar_disponiveis_e_seguir_pagina_atom_link() {
        let servidor = ServidorHttp::iniciar(Arc::new(|captura| {
            if captura.caminho.contains("pagina=2") {
                let xml = concat!(
                    "<Resultado xmlns:atom=\"http://www.w3.org/2005/Atom\">",
                    "<Arquivo><Protocolo>888</Protocolo><CodigoDocumento>3040</CodigoDocumento>",
                    "<Sistema>SCR</Sistema><TamanhoArquivo>7</TamanhoArquivo><Hash>aa</Hash>",
                    "<SituacaoAtual><Codigo>1</Codigo><Descricao>Recebido</Descricao></SituacaoAtual>",
                    "</Arquivo></Resultado>"
                );
                return resposta_http(
                    200,
                    "OK",
                    &[("Content-Type", "application/xml")],
                    xml.as_bytes(),
                );
            }
            let host = captura.cabecalho("host").unwrap_or("127.0.0.1:0");
            let proxima = format!("http://{host}/arquivos/disponiveis?pagina=2");
            let xml = format!(
                concat!(
                    "<Resultado xmlns:atom=\"http://www.w3.org/2005/Atom\">",
                    "<DataHoraProximaConsulta>2025-11-19T00:00:00.000</DataHoraProximaConsulta>",
                    "<Arquivo><Protocolo>111</Protocolo><CodigoDocumento>3040</CodigoDocumento>",
                    "<Sistema>SCR</Sistema><TamanhoArquivo>100</TamanhoArquivo><Hash>hh</Hash>",
                    "<SituacaoAtual><Codigo>3</Codigo><Descricao>A receber</Descricao></SituacaoAtual>",
                    "<DataHoraDisponibilizacao>2025-11-18T10:00:00.000</DataHoraDisponibilizacao>",
                    "</Arquivo><Arquivo><Protocolo>222</Protocolo></Arquivo>",
                    "<atom:link href=\"{}\" rel=\"disponiveis\" type=\"application/xml\"/>",
                    "</Resultado>"
                ),
                proxima
            );
            resposta_http(
                200,
                "OK",
                &[("Content-Type", "application/xml")],
                xml.as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let opcoes = OpcoesDisponiveis {
            data_hora_inicio: "2025-11-18T00:00:00.000".to_string(),
            identificador_documento: Some("3040".to_string()),
            sistemas: None,
            dependencia: None,
        };
        let pagina1 = cliente.listar_disponiveis(&opcoes).unwrap();
        assert_eq!(pagina1.arquivos.len(), 2);
        assert_eq!(pagina1.arquivos[0].protocolo, "111");
        assert_eq!(
            pagina1.arquivos[0].situacao_atual,
            SituacaoArquivo::AReceber
        );
        assert_eq!(pagina1.arquivos[0].tamanho_arquivo, 100);
        assert_eq!(
            pagina1.arquivos[1].situacao_atual,
            SituacaoArquivo::Desconhecida
        );
        assert!(pagina1.tem_proxima_pagina);
        // Página 2: via buscar_pagina (privada) porque o mock roda fora da
        // allowlist de produção; a pública seguir_pagina valida allowlist e
        // é exercida no teste de allowlist abaixo.
        let url2 = pagina1.proxima_pagina_url.clone().expect("atom:link href");
        let pagina2 = cliente.buscar_pagina(&url2).unwrap();
        assert_eq!(pagina2.arquivos.len(), 1);
        assert_eq!(pagina2.arquivos[0].protocolo, "888");
        assert_eq!(
            pagina2.arquivos[0].situacao_atual,
            SituacaoArquivo::Recebido
        );
        assert!(!pagina2.tem_proxima_pagina);
        assert!(pagina2.proxima_pagina_url.is_none());
        // dataHoraInicio foi como query no primeiro GET:
        let capturas = servidor.capturas();
        assert!(
            capturas[0]
                .caminho
                .contains("dataHoraInicio=2025-11-18T00%3A00%3A00.000"),
            "{}",
            capturas[0].caminho
        );
        assert!(
            capturas[0].caminho.contains("identificadorDocumento=3040"),
            "{}",
            capturas[0].caminho
        );
    }

    /// Allowlist não é burlável por redirect: um 3xx do host de origem
    /// apontando para OUTRO host NÃO é seguido (`Policy::none` — o default
    /// `limited(10)` do reqwest seguiria para host arbitrário e o payload
    /// viajaria). O destino NUNCA recebe requisição; o 3xx vira erro tipado.
    #[test]
    fn redirect_para_outro_host_nao_e_seguido() {
        let destino = ServidorHttp::iniciar(Arc::new(|_captura| {
            resposta_http(
                200,
                "OK",
                &[("Content-Type", "application/xml")],
                b"<Resultado>interceptado</Resultado>",
            )
        }));
        let url_destino = destino.url.clone();
        let origem = ServidorHttp::iniciar(Arc::new(move |_captura| {
            resposta_http(
                307,
                "Temporary Redirect",
                &[("Location", &url_destino.clone())],
                b"",
            )
        }));
        let cliente = cliente_para(&origem.url);
        let opcoes = OpcoesDisponiveis {
            data_hora_inicio: "2025-11-18T00:00:00.000".to_string(),
            identificador_documento: None,
            sistemas: None,
            dependencia: None,
        };
        let resultado = cliente.listar_disponiveis(&opcoes);
        assert!(
            resultado.is_err(),
            "3xx não seguido não pode parecer sucesso"
        );
        // O host de destino NUNCA recebeu nada — o corpo não vazou.
        assert_eq!(
            destino.total_capturas(),
            0,
            "redirect cross-host não é seguido"
        );
        // A origem recebeu UMA tentativa: o 3xx vira erro de status do BACEN
        // (não-retriável), então o RetryPolicy não re-tenta.
        assert_eq!(origem.total_capturas(), 1);
    }

    #[test]
    fn listar_disponiveis_data_hora_vazia_e_validacao() {
        let servidor = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(200, "OK", &[], b"<Resultado></Resultado>")
        }));
        let cliente = cliente_para(&servidor.url);
        let err = cliente
            .listar_disponiveis(&OpcoesDisponiveis {
                data_hora_inicio: String::new(),
                identificador_documento: None,
                sistemas: None,
                dependencia: None,
            })
            .unwrap_err();
        let StaError::Validacao(RegError::MissingField { field }) = &err else {
            panic!("esperava MissingField, got {err:?}")
        };
        assert_eq!(field, "dataHoraInicio");
        assert_eq!(servidor.total_capturas(), 0);
    }

    // ------------------------------------------------------------
    // AlterarSituacao — único endpoint com Content-Type (ws_test.go:1309-1415)
    // ------------------------------------------------------------

    #[test]
    fn alterar_situacao_204_exige_content_type() {
        let servidor = ServidorHttp::iniciar(Arc::new(|captura| {
            assert_eq!(captura.metodo, "PUT");
            assert_eq!(captura.caminho, "/arquivos/situacao");
            // ÚNICO endpoint do manual que exige Content-Type application/xml.
            assert_eq!(
                captura.cabecalho("content-type"),
                Some("application/xml"),
                "PUT /arquivos/situacao exige Content-Type application/xml"
            );
            resposta_http(204, "No Content", &[], b"")
        }));
        let cliente = cliente_para(&servidor.url);
        cliente
            .alterar_situacao(
                &["111".to_string(), "222".to_string()],
                SituacaoTransferencia::Recebido,
            )
            .unwrap();
        let corpo = String::from_utf8_lossy(&servidor.capturas()[0].corpo).into_owned();
        assert!(
            corpo.contains("<Protocolos>111;222</Protocolos>"),
            "{corpo}"
        );
        assert!(corpo.contains("<Situacao>REC</Situacao>"), "{corpo}");
        assert_eq!(servidor.total_capturas(), 1);
    }

    #[test]
    fn alterar_situacao_400_sem_retry_e_validacao_local() {
        let servidor = ServidorHttp::iniciar(Arc::new(|_| {
            resposta_http(
                400,
                "Bad Request",
                &[("Content-Type", "application/xml")],
                xml_erro(400, "Situacao inválida").as_bytes(),
            )
        }));
        let cliente = cliente_para(&servidor.url);
        let err = cliente
            .alterar_situacao(&["111".to_string()], SituacaoTransferencia::AReceber)
            .unwrap_err();
        let StaError::ErroBacen { status, .. } = &err else {
            panic!("esperava ErroBacen 400, got {err:?}")
        };
        assert_eq!(*status, 400);
        assert_eq!(servidor.total_capturas(), 1, "4xx NUNCA re-tenta");
        // Protocolos vazio: erro imediato, sem rede.
        let err = cliente
            .alterar_situacao(&[], SituacaoTransferencia::Recebido)
            .unwrap_err();
        let StaError::Validacao(RegError::InvalidValue { field, .. }) = &err else {
            panic!("esperava Validacao, got {err:?}")
        };
        assert_eq!(field, "protocolos");
        assert_eq!(servidor.total_capturas(), 1);
    }

    // ------------------------------------------------------------
    // Allowlist + usuário — erros ANTES de conectar (sem rede)
    // ------------------------------------------------------------

    #[test]
    fn allowlist_rejeita_url_antes_de_conectar() {
        let segredos = ProvedorMemoria::default().inserir(USUARIO_TESTE, "senha-teste");
        for url in [
            "https://evil.example.com/staws",
            "http://sta-h.bcb.gov.br/staws",
            "https://sta-h.bcb.gov.br/staws/",
            "https://203.0.113.10/staws",
            "",
        ] {
            match StaClient::nova(
                url,
                USUARIO_TESTE,
                &segredos,
                TIMEOUT_PADRAO,
                RetryPolicy::padrao(),
            ) {
                Err(StaError::Validacao(RegError::InvalidValue { field, reason })) => {
                    assert_eq!(field, "base_url", "url {url:?}");
                    assert!(!reason.is_empty());
                }
                Ok(_) => panic!("URL {url:?} fora da allowlist deveria ser rejeitada"),
                Err(outro) => panic!("esperava Validacao para {url:?}, got {outro:?}"),
            }
        }
        // Os 4 hosts da allowlist constroem SEM rede (nada é enviado aqui).
        for url in [
            "https://sta-h.bcb.gov.br/staws",
            "https://sta.bcb.gov.br/staws",
            "https://www9.bcb.gov.br/senhaws",
            "https://www3.bcb.gov.br/senhaws",
        ] {
            StaClient::nova(
                url,
                USUARIO_TESTE,
                &segredos,
                TIMEOUT_PADRAO,
                RetryPolicy::padrao(),
            )
            .unwrap_or_else(|e| panic!("URL {url:?} da allowlist deveria construir: {e}"));
        }
    }

    #[test]
    fn usuario_sisbacen_valida_formatos() {
        for ok in [
            "123450001.fulano",
            "12345/0001.fulano",
            "123450001.Fulano_1-x",
        ] {
            assert!(validar_usuario_sisbacen(ok).is_ok(), "{ok:?}");
        }
        for err in [
            "123450001",          // sem .operador
            "12345.0001.fulano",  // formato proibido (ws_test.go:221)
            "x.y",                // sem dígitos
            "12345/0001.",        // operador vazio
            "12345/0001.fu lano", // espaço
            "",                   // vazio
        ] {
            assert!(validar_usuario_sisbacen(err).is_err(), "{err:?}");
        }
    }

    // ------------------------------------------------------------
    // Parses unitários (port de ws_test.go:1541-1666)
    // ------------------------------------------------------------

    #[test]
    fn parsear_x_content_hash_casos() {
        let h64 = "a".repeat(64);
        let h64_maiusculo = h64.to_uppercase();
        assert_eq!(
            parsear_x_content_hash(&format!("SHA-256 {h64}")).unwrap(),
            h64
        );
        // Algoritmo case-insensitive; hex normalizado lowercase (ws.go:785/793).
        assert_eq!(
            parsear_x_content_hash(&format!("sha-256 {h64_maiusculo}")).unwrap(),
            h64
        );
        let sem_espaco = "a".repeat(64);
        for err in [
            "MD5 aaaa",          // algoritmo errado
            sem_espaco.as_str(), // sem espaço
            "SHA-256 abc",       // 3 chars
            "SHA-256",           // sem hash
            "SHA-256 gg",        // não-hex
        ] {
            match parsear_x_content_hash(err) {
                Err(StaError::ContentHashHeaderMalformed { .. }) => {}
                outro => panic!("{err:?} deveria ser HeaderMalformed, got {outro:?}"),
            }
        }
    }

    #[test]
    fn parsear_ranges_casos() {
        // port de ws_test.go:1541
        assert_eq!(parsear_ranges(""), Vec::new());
        assert_eq!(parsear_ranges("0-3"), vec![Intervalo { inicio: 0, fim: 3 }]);
        assert_eq!(
            parsear_ranges("0-3;5-8"),
            vec![
                Intervalo { inicio: 0, fim: 3 },
                Intervalo { inicio: 5, fim: 8 }
            ]
        );
        // Malformados descartados (ws_types.go:212-222):
        assert_eq!(
            parsear_ranges("x-y;5-8"),
            vec![Intervalo { inicio: 5, fim: 8 }]
        );
        assert_eq!(parsear_ranges("8-5"), Vec::new());
        assert_eq!(parsear_ranges("-3"), Vec::new());
    }

    #[test]
    fn truncar_bytes_casos() {
        assert_eq!(truncar_bytes(b"curto", 10), "curto");
        assert_eq!(truncar_bytes(b"abcdefghij", 5), "abcde...");
    }

    #[test]
    fn retry_policy_validacoes() {
        assert!(
            RetryPolicy::nova(0, Duration::from_secs(1), 0).is_err(),
            "0 tentativas"
        );
        assert!(
            RetryPolicy::nova(11, Duration::from_secs(1), 0).is_err(),
            "11 > 10"
        );
        assert!(
            RetryPolicy::nova(3, Duration::from_secs(1), 10_001).is_err(),
            "jitter > 100%"
        );
        assert!(RetryPolicy::nova(1, Duration::from_millis(1), 0).is_ok());
        assert!(RetryPolicy::nova(3, Duration::from_millis(1), 10_000).is_ok());
    }

    #[test]
    fn situacao_transferencia_parse_estrito() {
        assert_eq!(
            SituacaoTransferencia::parsear("A_REC").unwrap(),
            SituacaoTransferencia::AReceber
        );
        assert_eq!(
            SituacaoTransferencia::parsear("REC").unwrap(),
            SituacaoTransferencia::Recebido
        );
        // Dispatch exaustivo: valor fora do domínio é ERRO, nunca Unknown silencioso.
        assert!(SituacaoTransferencia::parsear("RECIBO").is_err());
        assert!(SituacaoTransferencia::parsear("").is_err());
    }
}
