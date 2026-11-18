//! reg-cli — Camada 3: composition root do wedge V1.
//!
//! CLI determinística (exit codes 0 aceito / 1 rejeição-transporte /
//! 2 validação-cliente / 3 erro-formal-BACEN), `--offline` por padrão,
//! `--json` para pipelines. Subcomandos implementados: `validate`
//! (pré-validação de forma + chancela opcional via oráculo oficial
//! B02/ADR-0018), os subcomandos de REDE `sta`/`senhaws` (S6 — port do
//! backend/internal/sta/ws.go, retry.go e backend/internal/senhaws/senhaws.go;
//! doc de módulo de `sta`/`senhaws` detalha o protocolo) e os subcomandos
//! LOCAIS `dossier` (S5 — RC Conjunta 18/2025; doc de módulo de
//! `dossier.rs`) e `shadow-diff` (S5 — gate de desligamento do legado; doc
//! de módulo de `shadow.rs`). Os demais (`generate`, `xray`, `mutate`)
//! chegam nas fases seguintes.
//!
//! Mapeamento de exit codes do `validate` (contrato do cabeçalho):
//! - **0 — aceito**: XML bem-formado; com `--chancela`, oráculo oficial aceitou.
//! - **1 — rejeição-transporte**: arquivo de entrada ausente/ilegível.
//! - **2 — validação-cliente**: problema do AMBIENTE de validação do cliente
//!   (CADOC sem suporte, `--jar-dir`/`--jre` ausentes, JRE/jar ausentes no
//!   builder, timeout do oráculo, zip chancela não produzido) — nunca é culpa
//!   do documento.
//! - **3 — erro-formal-BACEN**: XML mal-formado/sem declaração, ou o oráculo
//!   oficial REJEITOU o documento (`ValidadorRejeitou`).
//!
//! O MESMO contrato vale para `sta`/`senhaws` (funções `codigo_saida_sta` e
//! `codigo_saida_senhaws`): 2 = config/ambiente/`--offline`/segredo ausente;
//! 1 = transporte (rede, timeout, leitura) e INTEGRIDADE em trânsito
//! (X-Content-Hash ausente/divergente — o dado chegou corrompido, não é
//! rejeição formal); 3 = rejeição formal do BACEN (XML Listagem 4, 4xx/5xx)
//! e resposta XML mal-formada — mesmo código que XML mal-formado em
//! `validate`.
//!
//! `--offline` e os subcomandos: a flag BLOQUEIA `sta`/`senhaws` com erro
//! explícito (exit 2) — rede é sempre opt-out explícito. O validador oficial
//! é processo LOCAL headless e continua funcionando sob `--offline` (doc de
//! módulo do `reg_cadoc::validator` detalha a doutrina e a allowlist de argv).
//! `dossier` e `shadow-diff` são 100% LOCAIS (sem rede) e funcionam sob
//! `--offline`. Contrato deles: `dossier` usa 0 (dossiê gerado) / 1
//! (rejeição-transporte: mapping.toml ilegível, --out não criado/escrito) /
//! 2 (validação-cliente: mapping.toml inválido, cadoc sem suporte no wedge
//! V1, seed ausente/inválida, ledger indisponível) — 3 não se aplica (não há
//! oráculo BACEN). `shadow-diff` usa 0 (SMER 100%, gate passa) / 1
//! (transporte) / 2 (XML mal-formado ou fora do subset declarado) / 3 (gate
//! REPROVADO, SMER < 100% — a autoridade formal deste subcomando é o
//! próprio gate).

mod dossier;
mod senhaws;
mod secrets;
mod shadow;
mod sta;

use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use reg_cadoc::validator::{BacenValidatorAdapter, JrePath, ValidationError};
use reg_ledger::Chain;
use sha2::Digest as _;
use zeroize::Zeroizing;

use crate::dossier::{ENV_SEED_1, ENV_SEED_2};
use crate::secrets::ProvedorEnv;
use crate::sta::{Ambiente, RetryPolicy, SituacaoTransferencia, StaError};
use crate::senhaws::SenhawsError;

/// Timeout por chamada HTTP dos subcomandos de rede — default do v1
/// (ws.go:234-236).
const TIMEOUT_REDE: Duration = sta::TIMEOUT_PADRAO;

#[derive(Parser, Debug)]
#[command(
    name = "reg-cli",
    about = "Radiant Reg — motor de CADOC determinístico (wedge V1)",
    version
)]
struct Cli {
    /// Rodar sem qualquer acesso de rede (padrão de produção on-perimeter).
    /// Desligue com `--offline=false` — a saída da rede é sempre opt-out
    /// explícito, nunca o contrário. (`require_equals`: `--offline` pelado não
    /// engole o subcomando seguinte.)
    #[arg(
        long,
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1,
        require_equals = true,
        action = clap::ArgAction::Set
    )]
    offline: bool,
    /// Saída em JSON estruturado (para Airflow/Dagster).
    #[arg(long)]
    json: bool,
    #[command(subcommand)]
    comando: Option<Comando>,
}

#[derive(Subcommand, Debug)]
enum Comando {
    /// Pré-valida a forma do CADOC e, com --chancela, submete ao oráculo
    /// oficial do BCB (validador_linha_comando — B02/ADR-0018).
    Validate {
        /// CADOC alvo (wedge V1: só 3040).
        #[arg(long)]
        cadoc: String,
        /// Caminho do arquivo XML de entrada.
        #[arg(long)]
        input: PathBuf,
        /// Encaminhar ao validador oficial (chancela de submissão). Sem esta
        /// flag: pré-validação honesta de forma — NUNCA é chancela.
        #[arg(long, default_value_t = false)]
        chancela: bool,
        /// Diretório da distribuição oficial do validador (contém
        /// validador_linha_comando.bat/.sh). Exigido por --chancela.
        #[arg(long, requires = "chancela")]
        jar_dir: Option<PathBuf>,
        /// Executável java da JRE (≥ 1.8). Exigido por --chancela.
        #[arg(long, requires = "chancela")]
        jre: Option<PathBuf>,
        /// Passa o token `no_warn` ao CLI oficial (Manual SCR3040, seção F).
        #[arg(long, requires = "chancela")]
        no_warn: bool,
    },
    /// Operações contra o BACEN STA Web Services (REDE — bloqueado por
    /// --offline). Hosts: allowlist hard-coded (doc de módulo de `sta`).
    Sta {
        #[command(subcommand)]
        operacao: StaOperacao,
    },
    /// Gerência de senha Sisbacen no BACEN senhaws (REDE — bloqueado por
    /// --offline). Hosts: allowlist hard-coded (doc de módulo de `senhaws`).
    Senhaws {
        #[command(subcommand)]
        operacao: SenhawsOperacao,
    },
    /// Dossiê RC Conjunta 18/2025 — JSON assinado Ed25519 (dupla alçada,
    /// 4-olhos) + HTML A4 editorial. 100% LOCAL — funciona sob --offline.
    /// As seeds vêm de env (doc de módulo de `dossier`), nunca de argv.
    Dossier {
        #[command(subcommand)]
        operacao: DossierOperacao,
    },
    /// Gate de desligamento do legado: compara o XML do v1 com o XML do
    /// Rust, classifica as divergências e emite o SMER. Exit 0 SOMENTE com
    /// SMER 100%. 100% LOCAL — funciona sob --offline.
    ShadowDiff {
        /// XML emitido pelo legado (v1 Go).
        #[arg(long)]
        legacy: PathBuf,
        /// XML emitido pelo Radiant Reg.
        #[arg(long)]
        rust: PathBuf,
    },
}

/// Operações do dossiê (doc de módulo de `dossier`).
#[derive(Subcommand, Debug)]
enum DossierOperacao {
    /// Constrói e assina o dossiê: payload JCS → dupla alçada Ed25519 (2
    /// chaves distintas de env) → custódia do SHA-256 no ledger →
    /// dossie.json + dossie.html em --out.
    Build {
        /// mapping.toml custodiado (fonte do dicionário/linhagem e do
        /// SHA-256 assinado).
        #[arg(long)]
        mapping: PathBuf,
        /// Diretório de saída — recebe dossie.json, dossie.html e, por
        /// default, o ledger da custódia.
        #[arg(long)]
        out: PathBuf,
        /// Ledger SQLite da custódia (default: <out>/ledger.sqlite).
        #[arg(long)]
        ledger: Option<PathBuf>,
    },
}

/// Operações do STA WS (port de ws.go; ver doc de módulo de `sta`).
#[derive(Subcommand, Debug)]
enum StaOperacao {
    /// Fluxo de envio em 2 fases (manual §5.1+§5.2): POST /arquivos →
    /// protocolo; PUT /arquivos/{p}/conteudo → 200. Entrada: o
    /// *_VALIDADO.zip do oráculo oficial (B02/ADR-0018) — nada sai para o
    /// STA sem chancela.
    Enviar {
        /// Código do documento CADOC (IdentificadorDocumento).
        #[arg(long)]
        cadoc: String,
        /// Arquivo de entrada (ZIP chancelado pelo oráculo ou XML cru).
        #[arg(long)]
        input: PathBuf,
        /// Usuário Sisbacen UUUUUDDDD.operador (ou UUUUU/DDDD.operador).
        #[arg(long)]
        usuario: String,
        /// Ambiente BACEN (URL da allowlist).
        #[arg(long)]
        ambiente: Ambiente,
    },
    /// GET /arquivos/{p}/posicaoupload (manual §5.3.1) — status + ranges
    /// recebidos de um envio em andamento.
    PosicaoUpload {
        #[arg(long)]
        protocolo: String,
        #[arg(long)]
        usuario: String,
        #[arg(long)]
        ambiente: Ambiente,
    },
    /// PUT /arquivos/{p}/conteudo com Content-Range (manual §5.6) — envia UM
    /// chunk (máx 8 MiB) do arquivo.
    EnviarRange {
        #[arg(long)]
        protocolo: String,
        /// Arquivo com os bytes do chunk.
        #[arg(long)]
        input: PathBuf,
        /// Byte inicial do range (inclusivo).
        #[arg(long)]
        inicio: u64,
        /// Byte final do range (inclusivo).
        #[arg(long)]
        fim: u64,
        /// Tamanho total do arquivo completo.
        #[arg(long)]
        total: u64,
        #[arg(long)]
        usuario: String,
        #[arg(long)]
        ambiente: Ambiente,
    },
    /// GET /arquivos/{p}/conteudo (manual §6.1.1) — download com validação
    /// OBRIGATÓRIA de X-Content-Hash (sha256 do body).
    Baixar {
        #[arg(long)]
        protocolo: String,
        /// Caminho de saída do conteúdo baixado.
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        usuario: String,
        #[arg(long)]
        ambiente: Ambiente,
    },
    /// GET /arquivos/disponiveis (manual §8.1.1) — arquivos que o BACEN
    /// disponibilizou; segue atom:link enquanto houver próxima página
    /// (≤1000 registros/página).
    Disponiveis {
        /// Data-hora inicial "yyyy-MM-ddTHH:mm:ss.SSS" (obrigatório, Tabela 4).
        #[arg(long)]
        data_hora_inicio: String,
        #[arg(long)]
        identificador_documento: Option<String>,
        /// Até 100 sistemas separados por ";".
        #[arg(long)]
        sistemas: Option<String>,
        #[arg(long)]
        dependencia: Option<String>,
        #[arg(long)]
        usuario: String,
        #[arg(long)]
        ambiente: Ambiente,
    },
    /// PUT /arquivos/situacao (manual §7.1) — muda protocolos para A_REC/REC.
    Situacao {
        /// Protocolos separados por vírgula.
        #[arg(long, value_delimiter = ',')]
        protocolos: Vec<String>,
        /// "A_REC" (a receber) ou "REC" (recebido) — parse estrito.
        #[arg(long)]
        situacao: String,
        #[arg(long)]
        usuario: String,
        #[arg(long)]
        ambiente: Ambiente,
    },
}

/// Operações do senhaws (port de senhaws.go; ver doc de módulo de `senhaws`).
#[derive(Subcommand, Debug)]
enum SenhawsOperacao {
    /// PUT /senha (manual §9.1) — rotaciona a senha Sisbacen. Informe
    /// --nova-senha OU --gerar (gera 16 bytes hex = 32 chars). A senha
    /// atual vem de RADIANT_SECRET_BACEN_SENHA_<USUARIO>.
    AlterarSenha {
        #[arg(long)]
        usuario: String,
        #[arg(long)]
        ambiente: Ambiente,
        /// Nova senha explícita (8-128 chars, ≠ atual). Mutuamente exclusiva
        /// com --gerar.
        #[arg(long)]
        nova_senha: Option<String>,
        /// Gera nova senha aleatória de 16 bytes hex e a exibe UMA vez.
        #[arg(long)]
        gerar: bool,
    },
    /// GET /senha/vencimento (manual §9.2) — dias restantes da senha.
    Vencimento {
        #[arg(long)]
        usuario: String,
        #[arg(long)]
        ambiente: Ambiente,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    // --offline BLOQUEIA os subcomandos de rede com erro explícito (exit 2 =
    // problema do ambiente do cliente). O validador local (`validate`) é
    // subprocesso headless, NÃO rede — continua funcionando sob --offline.
    if cli.offline
        && matches!(
            cli.comando,
            Some(Comando::Sta { .. }) | Some(Comando::Senhaws { .. })
        )
    {
        erro(
            cli.json,
            2,
            "--offline bloqueia subcomandos de rede (sta, senhaws). Use --offline=false para \
             transmitir, ou rode o validador local com `validate`."
                .to_string(),
        );
        return ExitCode::from(2);
    }
    match cli.comando {
        Some(Comando::Validate {
            cadoc,
            input,
            chancela,
            jar_dir,
            jre,
            no_warn,
        }) => run_validate(cli.json, cadoc, input, chancela, jar_dir, jre, no_warn),
        Some(Comando::Sta { operacao }) => run_sta(cli.json, operacao),
        Some(Comando::Senhaws { operacao }) => run_senhaws(cli.json, operacao),
        Some(Comando::Dossier { operacao }) => run_dossier_operacao(cli.json, operacao),
        Some(Comando::ShadowDiff { legacy, rust }) => run_shadow(cli.json, &legacy, &rust),
        None => {
            eprintln!(
                "reg-cli: esqueleto — nenhum subcomando informado (implementados: validate, \
                 sta, senhaws, dossier, shadow-diff)"
            );
            ExitCode::from(2)
        }
    }
}

/// Erro → stderr (humano) ou JSON estruturado (pipelines). O exit code é
/// escolhido pelo chamador; `_codigo` entra no JSON para o pipeline decidir.
fn erro(json: bool, codigo: u8, mensagem: String) {
    if json {
        eprintln!(
            "{}",
            serde_json::json!({ "resultado": "rejeitado", "exit_code": codigo, "erro": mensagem })
        );
    } else {
        eprintln!("{mensagem}");
    }
}

fn run_validate(
    json: bool,
    cadoc: String,
    input: PathBuf,
    chancela: bool,
    jar_dir: Option<PathBuf>,
    jre: Option<PathBuf>,
    no_warn: bool,
) -> ExitCode {
    // validação-cliente: CADOC sem suporte no wedge V1.
    if cadoc != "3040" {
        erro(json, 2, format!("--cadoc {cadoc} sem suporte no wedge V1 (só 3040)"));
        return ExitCode::from(2);
    }

    // rejeição-transporte: o arquivo nem chegou.
    let xml_bytes = match std::fs::read(&input) {
        Ok(bytes) => bytes,
        Err(e) => {
            erro(
                json,
                1,
                format!("rejeição-transporte: não foi possível ler {}: {e}", input.display()),
            );
            return ExitCode::from(1);
        }
    };

    // erro formal: XML mal-formado ou sem declaração (pré-validação de forma).
    if let Err(motivo) = checar_xml_bem_formado(&xml_bytes) {
        erro(
            json,
            3,
            format!("erro formal: XML mal-formado em {}: {motivo}", input.display()),
        );
        return ExitCode::from(3);
    }

    // Sem --chancela: pré-validação honesta — aviso explícito de que NÃO é
    // chancela do oráculo (nenhum ZIP sai para o STA assim, B02/ADR-0018).
    if !chancela {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "resultado": "aceito",
                    "modo": "pre_validacao",
                    "arquivo": input.display().to_string(),
                    "sha256": serde_json::Value::Null,
                })
            );
        } else {
            println!(
                "pré-validação OK: XML bem-formado (declaração presente + sintaxe quick-xml) — {}",
                input.display()
            );
        }
        eprintln!(
            "AVISO: pré-validação NÃO é chancela (B02/ADR-0018). Nenhum ZIP sai para o STA sem o \
             *_VALIDADO.zip do validador oficial do BCB — rode com --chancela --jar-dir <dist> --jre <java>."
        );
        return ExitCode::from(0);
    }

    // --chancela exige --jar-dir e --jre (clap já exige o inverso; esta é a
    // defesa em profundidade para --chancela sem paths).
    let (jar_dir, jre) = match (jar_dir, jre) {
        (Some(jd), Some(jr)) => (jd, jr),
        (None, None) | (None, Some(_)) | (Some(_), None) => {
            erro(json, 2, "--chancela exige --jar-dir <dist> e --jre <java>".to_string());
            return ExitCode::from(2);
        }
    };

    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "documento".to_string());
    // Builder valida existência/executabilidade — erro tipado, exit 2
    // (ambiente do cliente), nunca Option silencioso.
    let adapter = match BacenValidatorAdapter::builder(jar_dir, JrePath::new(jre)) {
        Ok(builder) => builder.no_warn(no_warn).build(),
        Err(e) => {
            erro(json, 2, format!("erro de ambiente do oráculo oficial: {e}"));
            return ExitCode::from(2);
        }
    };

    match adapter.validate(&xml_bytes, &stem) {
        Ok(chancela_zip) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "resultado": "aceito",
                        "modo": "chancela",
                        "arquivo": input.display().to_string(),
                        "sha256": chancela_zip.sha256,
                        "zip_bytes": chancela_zip.zip_bytes.len(),
                    })
                );
            } else {
                println!(
                    "chancela OK: oráculo oficial (B02/ADR-0018) aceitou o documento — {}",
                    input.display()
                );
                println!("sha256(chancela): {}", chancela_zip.sha256);
            }
            ExitCode::from(0)
        }
        // Rejeição do ORÁCULO é erro formal BACEN (exit 3) — o validador
        // oficial é a autoridade de submissão.
        Err(ValidationError::ValidadorRejeitou { codigo, mensagem }) => {
            erro(
                json,
                3,
                format!("erro formal BACEN: oráculo oficial rejeitou o documento (código {codigo}): {mensagem}"),
            );
            ExitCode::from(3)
        }
        // Demais falhas são do ambiente de validação do cliente (exit 2).
        Err(e @ ValidationError::JreAusente { .. })
        | Err(e @ ValidationError::JarAusente { .. })
        | Err(e @ ValidationError::ConfigInvalida { .. })
        | Err(e @ ValidationError::Timeout { .. })
        | Err(e @ ValidationError::ZipChancelaNaoEncontrado { .. })
        | Err(e @ ValidationError::IoError(_)) => {
            erro(json, 2, format!("erro de ambiente do oráculo oficial: {e}"));
            ExitCode::from(2)
        }
    }
}

// ============================================================
// sta — subcomandos de rede (port de ws.go; doc de módulo de sta.rs)
// ============================================================

/// Erro do STA → exit code (contrato do doc de módulo):
/// 2 = validação-cliente (config/ambiente/segredo); 3 = formal-BACEN
/// (Listagem 4) e resposta XML mal-formada; 1 = transporte E integridade
/// (hash ausente/divergente — dado corrompido em trânsito não é rejeição
/// formal); esgotamento de retry herda o código da causa.
fn codigo_saida_sta(e: &StaError) -> u8 {
    match e {
        StaError::Validacao(_) => 2,
        StaError::ErroBacen { .. } | StaError::RespostaInvalida { .. } => 3,
        StaError::Transporte(_)
        | StaError::LeituraCorpo(_)
        | StaError::ContentHashMismatch { .. }
        | StaError::ContentHashHeaderMalformed { .. }
        | StaError::ContentHashAusente { .. } => 1,
        StaError::AposTentativas { causa, .. } => codigo_saida_sta(causa),
    }
}

fn falha_sta(json: bool, e: &StaError) -> ExitCode {
    let codigo = codigo_saida_sta(e);
    erro(json, codigo, format!("{e}"));
    ExitCode::from(codigo)
}

fn cliente_sta(usuario: &str, ambiente: Ambiente) -> Result<sta::StaClient, StaError> {
    sta::StaClient::nova(ambiente.url_sta(), usuario, &ProvedorEnv, TIMEOUT_REDE, RetryPolicy::padrao())
}

fn run_sta(json: bool, operacao: StaOperacao) -> ExitCode {
    match operacao {
        StaOperacao::Enviar { cadoc, input, usuario, ambiente } => {
            sta_enviar(json, &cadoc, &input, &usuario, ambiente)
        }
        StaOperacao::PosicaoUpload { protocolo, usuario, ambiente } => {
            sta_posicao_upload(json, &protocolo, &usuario, ambiente)
        }
        StaOperacao::EnviarRange { protocolo, input, inicio, fim, total, usuario, ambiente } => {
            sta_enviar_range(json, &protocolo, &input, inicio, fim, total, &usuario, ambiente)
        }
        StaOperacao::Baixar { protocolo, output, usuario, ambiente } => {
            sta_baixar(json, &protocolo, &output, &usuario, ambiente)
        }
        StaOperacao::Disponiveis {
            data_hora_inicio,
            identificador_documento,
            sistemas,
            dependencia,
            usuario,
            ambiente,
        } => sta_disponiveis(
            json,
            data_hora_inicio,
            identificador_documento,
            sistemas,
            dependencia,
            &usuario,
            ambiente,
        ),
        StaOperacao::Situacao { protocolos, situacao, usuario, ambiente } => {
            sta_situacao(json, protocolos, situacao, &usuario, ambiente)
        }
    }
}

fn ler_entrada(json: bool, input: &PathBuf) -> Result<Vec<u8>, ExitCode> {
    match std::fs::read(input) {
        Ok(bytes) => Ok(bytes),
        Err(e) => {
            erro(
                json,
                1,
                format!("rejeição-transporte: não foi possível ler {}: {e}", input.display()),
            );
            Err(ExitCode::from(1))
        }
    }
}

fn sta_enviar(json: bool, cadoc: &str, input: &PathBuf, usuario: &str, ambiente: Ambiente) -> ExitCode {
    let payload = match ler_entrada(json, input) {
        Ok(bytes) => bytes,
        Err(codigo) => return codigo,
    };
    let cliente = match cliente_sta(usuario, ambiente) {
        Ok(c) => c,
        Err(e) => return falha_sta(json, &e),
    };
    match cliente.enviar(cadoc, &payload) {
        Ok(envio) if envio.aceito => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "resultado": "aceito",
                        "protocolo": envio.protocolo,
                        "bytes": payload.len(),
                        "sha256": hex::encode(sha2::Sha256::digest(&payload)),
                    })
                );
            } else {
                println!("envio aceito pelo BACEN STA — protocolo {}", envio.protocolo);
            }
            ExitCode::from(0)
        }
        Ok(envio) => {
            // Rejeição de upload: protocolo preservado para forensic trail.
            match envio.rejeicao {
                Some(rejeicao) => erro(
                    json,
                    3,
                    format!(
                        "BACEN recusou o conteúdo (protocolo={}, código={}: {})",
                        envio.protocolo, rejeicao.codigo, rejeicao.mensagem
                    ),
                ),
                None => erro(
                    json,
                    3,
                    format!("BACEN recusou o conteúdo (protocolo={})", envio.protocolo),
                ),
            }
            ExitCode::from(3)
        }
        Err(e) => falha_sta(json, &e),
    }
}

fn sta_posicao_upload(json: bool, protocolo: &str, usuario: &str, ambiente: Ambiente) -> ExitCode {
    let cliente = match cliente_sta(usuario, ambiente) {
        Ok(c) => c,
        Err(e) => return falha_sta(json, &e),
    };
    match cliente.posicao_upload(protocolo) {
        Ok(status) => {
            let ranges: Vec<serde_json::Value> = status
                .ranges_recebidos
                .iter()
                .map(|i| serde_json::json!({"inicio": i.inicio, "fim": i.fim}))
                .collect();
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "resultado": "aceito",
                        "protocolo": status.protocolo,
                        "situacao": status.situacao.como_str(),
                        "situacao_cru": status.situacao_cru,
                        "ranges_recebidos": ranges,
                        "ranges_recebidos_cru": status.ranges_recebidos_cru,
                    })
                );
            } else {
                println!(
                    "protocolo {}: {} (ranges recebidos: {})",
                    status.protocolo,
                    status.situacao.como_str(),
                    status.ranges_recebidos_cru
                );
            }
            ExitCode::from(0)
        }
        Err(e) => falha_sta(json, &e),
    }
}

#[allow(clippy::too_many_arguments)]
fn sta_enviar_range(
    json: bool,
    protocolo: &str,
    input: &PathBuf,
    inicio: u64,
    fim: u64,
    total: u64,
    usuario: &str,
    ambiente: Ambiente,
) -> ExitCode {
    let chunk = match ler_entrada(json, input) {
        Ok(bytes) => bytes,
        Err(codigo) => return codigo,
    };
    let cliente = match cliente_sta(usuario, ambiente) {
        Ok(c) => c,
        Err(e) => return falha_sta(json, &e),
    };
    match cliente.enviar_range(protocolo, inicio, fim, total, &chunk) {
        Ok(()) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "resultado": "aceito",
                        "protocolo": protocolo,
                        "range": format!("{inicio}-{fim}/{total}"),
                        "bytes": chunk.len(),
                    })
                );
            } else {
                println!("range {inicio}-{fim}/{total} aceito para o protocolo {protocolo}");
            }
            ExitCode::from(0)
        }
        Err(e) => falha_sta(json, &e),
    }
}

fn sta_baixar(json: bool, protocolo: &str, output: &PathBuf, usuario: &str, ambiente: Ambiente) -> ExitCode {
    let cliente = match cliente_sta(usuario, ambiente) {
        Ok(c) => c,
        Err(e) => return falha_sta(json, &e),
    };
    match cliente.baixar(protocolo) {
        Ok(download) => {
            if let Err(e) = std::fs::write(output, &download.conteudo) {
                erro(
                    json,
                    1,
                    format!("rejeição-transporte: não foi possível escrever {}: {e}", output.display()),
                );
                return ExitCode::from(1);
            }
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "resultado": "aceito",
                        "protocolo": protocolo,
                        "arquivo": output.display().to_string(),
                        "bytes": download.conteudo.len(),
                        "sha256": download.content_hash,
                        "x_content_hash": download.content_hash_header,
                        "etag": download.etag,
                        "last_modified": download.last_modified,
                    })
                );
            } else {
                println!(
                    "baixado {} ({} bytes) — SHA-256 verificado contra X-Content-Hash: {}",
                    output.display(),
                    download.conteudo.len(),
                    download.content_hash
                );
            }
            ExitCode::from(0)
        }
        Err(e) => falha_sta(json, &e),
    }
}

#[allow(clippy::too_many_arguments)]
fn sta_disponiveis(
    json: bool,
    data_hora_inicio: String,
    identificador_documento: Option<String>,
    sistemas: Option<String>,
    dependencia: Option<String>,
    usuario: &str,
    ambiente: Ambiente,
) -> ExitCode {
    let cliente = match cliente_sta(usuario, ambiente) {
        Ok(c) => c,
        Err(e) => return falha_sta(json, &e),
    };
    let opcoes = sta::OpcoesDisponiveis {
        data_hora_inicio,
        identificador_documento,
        sistemas,
        dependencia,
    };
    let mut pagina = match cliente.listar_disponiveis(&opcoes) {
        Ok(p) => p,
        Err(e) => return falha_sta(json, &e),
    };
    let mut todos = std::mem::take(&mut pagina.arquivos);
    // Paginação via atom:link — cada URL é revalidada contra a allowlist
    // em seguir_pagina. Guarda defensivo contra loop de paginação infinito.
    let mut paginas = 1u32;
    while let Some(ref url) = pagina.proxima_pagina_url {
        paginas += 1;
        if paginas > 10_000 {
            erro(json, 2, format!("limite defensivo de {paginas} páginas excedido"));
            return ExitCode::from(2);
        }
        pagina = match cliente.seguir_pagina(url) {
            Ok(p) => p,
            Err(e) => return falha_sta(json, &e),
        };
        todos.extend(pagina.arquivos);
    }
    if json {
        let arquivos: Vec<serde_json::Value> = todos
            .iter()
            .map(|a| {
                serde_json::json!({
                    "protocolo": a.protocolo,
                    "tipo_arquivo": a.tipo_arquivo,
                    "codigo_documento": a.codigo_documento,
                    "sistema": a.sistema,
                    "tamanho_arquivo": a.tamanho_arquivo,
                    "hash": a.hash,
                    "situacao": a.situacao_atual.como_str(),
                    "situacao_cru": a.situacao_atual_cru,
                    "data_hora_disponibilizacao": a.data_hora_disponibilizacao,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({
                "resultado": "aceito",
                "total": todos.len(),
                "paginas": paginas,
                "tem_proxima_pagina": pagina.tem_proxima_pagina,
                "data_hora_proxima_consulta": pagina.data_hora_proxima_consulta,
                "arquivos": arquivos,
            })
        );
    } else {
        println!(
            "{} arquivo(s) disponível(eis) em {paginas} página(s); próxima consulta: {}",
            todos.len(),
            pagina.data_hora_proxima_consulta
        );
        for a in &todos {
            println!(
                "  {} — {} ({}) — {} — hash {}",
                a.protocolo, a.codigo_documento, a.sistema, a.situacao_atual.como_str(), a.hash
            );
        }
    }
    ExitCode::from(0)
}

fn sta_situacao(
    json: bool,
    protocolos: Vec<String>,
    situacao: String,
    usuario: &str,
    ambiente: Ambiente,
) -> ExitCode {
    // Parse ESTRITO de A_REC|REC — valor inválido é validação-cliente (exit 2),
    // nunca viaja até o BACEN rejeitar (dispatch exaustivo da doutrina).
    let situacao = match SituacaoTransferencia::parsear(&situacao) {
        Ok(s) => s,
        Err(e) => {
            erro(json, 2, format!("{e}"));
            return ExitCode::from(2);
        }
    };
    let cliente = match cliente_sta(usuario, ambiente) {
        Ok(c) => c,
        Err(e) => return falha_sta(json, &e),
    };
    match cliente.alterar_situacao(&protocolos, situacao) {
        Ok(()) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "resultado": "aceito",
                        "protocolos": protocolos,
                        "situacao": situacao.como_str(),
                    })
                );
            } else {
                println!("situação alterada para {} nos protocolo(s) informado(s)", situacao.como_str());
            }
            ExitCode::from(0)
        }
        Err(e) => falha_sta(json, &e),
    }
}

// ============================================================
// senhaws — subcomandos de rede (port de senhaws.go; doc de módulo de senhaws.rs)
// ============================================================

/// Erro do senhaws → exit code (mesmo contrato de `codigo_saida_sta`; o
/// senhaws não tem validação de hash — não há variantes de integridade).
fn codigo_saida_senhaws(e: &SenhawsError) -> u8 {
    match e {
        SenhawsError::Validacao(_) => 2,
        SenhawsError::ErroBacen { .. } | SenhawsError::RespostaInvalida { .. } => 3,
        SenhawsError::Transporte(_) | SenhawsError::LeituraCorpo(_) => 1,
    }
}

fn falha_senhaws(json: bool, e: &SenhawsError) -> ExitCode {
    let codigo = codigo_saida_senhaws(e);
    erro(json, codigo, format!("{e}"));
    ExitCode::from(codigo)
}

fn cliente_senhaws(usuario: &str, ambiente: Ambiente) -> Result<senhaws::SenhawsClient, SenhawsError> {
    senhaws::SenhawsClient::nova(ambiente.url_senhaws(), usuario, &ProvedorEnv, TIMEOUT_REDE)
}

fn run_senhaws(json: bool, operacao: SenhawsOperacao) -> ExitCode {
    match operacao {
        SenhawsOperacao::AlterarSenha { usuario, ambiente, nova_senha, gerar } => {
            senhaws_alterar_senha(json, &usuario, ambiente, nova_senha, gerar)
        }
        SenhawsOperacao::Vencimento { usuario, ambiente } => {
            senhaws_vencimento(json, &usuario, ambiente)
        }
    }
}

fn senhaws_alterar_senha(
    json: bool,
    usuario: &str,
    ambiente: Ambiente,
    nova_senha: Option<String>,
    gerar: bool,
) -> ExitCode {
    let nova: Zeroizing<String> = match (nova_senha, gerar) {
        (Some(s), false) => Zeroizing::new(s),
        (None, true) => match senhaws::gerar_senha_hex() {
            Ok(s) => s,
            Err(e) => {
                erro(json, 2, format!("{e}"));
                return ExitCode::from(2);
            }
        },
        (Some(_), true) => {
            erro(json, 2, "--nova-senha e --gerar são mutuamente exclusivos".to_string());
            return ExitCode::from(2);
        }
        (None, false) => {
            erro(
                json,
                2,
                "informe --nova-senha <senha> (8-128 chars, ≠ atual) ou --gerar (16 bytes hex)".to_string(),
            );
            return ExitCode::from(2);
        }
    };
    let cliente = match cliente_senhaws(usuario, ambiente) {
        Ok(c) => c,
        Err(e) => return falha_senhaws(json, &e),
    };
    match cliente.alterar_senha(&nova) {
        Ok(()) => {
            // A nova senha é exibida UMA única vez — o caller DEVE armazená-la
            // em RADIANT_SECRET_BACEN_SENHA_<USUARIO> antes da próxima chamada
            // STA, senão todas retornam 401 (senhaws.go:142-144).
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "resultado": "aceito",
                        "acao": "alterar_senha",
                        "nova_senha": nova.as_str(),
                    })
                );
            } else {
                println!(
                    "senha alterada no BACEN. Nova senha (32 hex, exibida uma única vez): {}",
                    nova.as_str()
                );
            }
            ExitCode::from(0)
        }
        Err(e) => falha_senhaws(json, &e),
    }
}

fn senhaws_vencimento(json: bool, usuario: &str, ambiente: Ambiente) -> ExitCode {
    let cliente = match cliente_senhaws(usuario, ambiente) {
        Ok(c) => c,
        Err(e) => return falha_senhaws(json, &e),
    };
    match cliente.consultar_vencimento() {
        Ok(dias) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"resultado": "aceito", "dias_vencimento_senha": dias})
                );
            } else {
                println!("senha vence em {dias} dia(s)");
            }
            ExitCode::from(0)
        }
        Err(e) => falha_senhaws(json, &e),
    }
}

// ============================================================
// dossier — subcomando LOCAL (RC Conjunta 18/2025; doc de módulo de dossier.rs)
// ============================================================

/// Erro do dossier → exit code (contrato do doc de módulo): I/O = 1
/// (rejeição-transporte); validação-cliente = 2 (mapping inválido, seed
/// ausente/inválida, ledger indisponível, erro de assinatura/serialização).
/// 3 não se aplica — não há oráculo BACEN neste subcomando.
fn codigo_saida_dossie(e: &dossier::DossieErro) -> u8 {
    match e {
        dossier::DossieErro::Io(_) => 1,
        dossier::DossieErro::Reg(_)
        | dossier::DossieErro::Ledger(_)
        | dossier::DossieErro::Assinatura(_)
        | dossier::DossieErro::Jcs(_)
        | dossier::DossieErro::Serde(_) => 2,
    }
}

fn run_dossier_operacao(json: bool, operacao: DossierOperacao) -> ExitCode {
    match operacao {
        DossierOperacao::Build { mapping, out, ledger } => {
            run_dossier_build(json, &mapping, &out, ledger.as_deref())
        }
    }
}

fn run_dossier_build(json: bool, mapping: &Path, out: &Path, ledger: Option<&Path>) -> ExitCode {
    let mapping_bytes = match std::fs::read(mapping) {
        Ok(bytes) => bytes,
        Err(e) => {
            erro(
                json,
                1,
                format!("rejeição-transporte: não foi possível ler {}: {e}", mapping.display()),
            );
            return ExitCode::from(1);
        }
    };
    if let Err(e) = std::fs::create_dir_all(out) {
        erro(
            json,
            1,
            format!("rejeição-transporte: não foi possível criar {}: {e}", out.display()),
        );
        return ExitCode::from(1);
    }
    // Ledger da custódia: default <out>/ledger.sqlite — explicitável com
    // --ledger (a hash-chain é por arquivo; o dossiê referencia a entry).
    let ledger_path = ledger
        .map(Path::to_path_buf)
        .unwrap_or_else(|| out.join("ledger.sqlite"));
    let ledger = match Chain::open(&ledger_path) {
        Ok(chain) => chain,
        Err(e) => {
            erro(
                json,
                2,
                format!("erro de ambiente: ledger {} indisponível: {e}", ledger_path.display()),
            );
            return ExitCode::from(2);
        }
    };
    // Seeds NUNCA em argv (aparecem no `ps`/histórico — doutrina de
    // secrets.rs): vêm de RADIANT_SECRET_DOSSIER_SEED_1/2.
    let seeds = match (dossier::seed_do_ambiente(ENV_SEED_1), dossier::seed_do_ambiente(ENV_SEED_2)) {
        (Ok(s1), Ok(s2)) => (s1, s2),
        (Err(e), _) | (_, Err(e)) => {
            erro(
                json,
                2,
                format!(
                    "validação-cliente: {e} — exporte {} e {} (hex Ed25519 de 64 chars, DUAS \
                     chaves distintas)",
                    ENV_SEED_1, ENV_SEED_2
                ),
            );
            return ExitCode::from(2);
        }
    };
    let artefato = match dossier::construir(&mapping_bytes, &ledger, &seeds.0, &seeds.1) {
        Ok(a) => a,
        Err(e) => {
            let codigo = codigo_saida_dossie(&e);
            erro(json, codigo, format!("{e}"));
            return ExitCode::from(codigo);
        }
    };
    let (caminho_json, caminho_html) = match dossier::gravar(out, &artefato) {
        Ok(paths) => paths,
        Err(e) => {
            erro(
                json,
                1,
                format!("rejeição-transporte: não foi possível escrever o dossiê em {}: {e}", out.display()),
            );
            return ExitCode::from(1);
        }
    };
    if json {
        println!(
            "{}",
            serde_json::json!({
                "resultado": "aceito",
                "dossie_json": caminho_json.display().to_string(),
                "dossie_html": caminho_html.display().to_string(),
                "payload_sha256": artefato.payload_sha256,
                "entry_id": artefato.entry_id,
                "entry_hash": artefato.entry_hash,
            })
        );
    } else {
        println!(
            "dossiê gerado e assinado (dupla alçada Ed25519): {} + {}",
            caminho_json.display(),
            caminho_html.display()
        );
        println!("sha256(payload JCS): {}", artefato.payload_sha256);
        println!(
            "custódia no ledger {}: entry #{} — entry_hash {}",
            ledger_path.display(),
            artefato.entry_id,
            artefato.entry_hash
        );
    }
    ExitCode::from(0)
}

// ============================================================
// shadow-diff — subcomando LOCAL (gate de desligamento do legado;
// doc de módulo de shadow.rs)
// ============================================================

fn run_shadow(json: bool, legacy: &Path, rust_doc: &Path) -> ExitCode {
    let legado_bytes = match std::fs::read(legacy) {
        Ok(bytes) => bytes,
        Err(e) => {
            erro(
                json,
                1,
                format!("rejeição-transporte: não foi possível ler {}: {e}", legacy.display()),
            );
            return ExitCode::from(1);
        }
    };
    let rust_bytes = match std::fs::read(rust_doc) {
        Ok(bytes) => bytes,
        Err(e) => {
            erro(
                json,
                1,
                format!("rejeição-transporte: não foi possível ler {}: {e}", rust_doc.display()),
            );
            return ExitCode::from(1);
        }
    };
    let relatorio = match shadow::diff(&legado_bytes, &rust_bytes) {
        Ok(rel) => rel,
        // XML mal-formado/fora do subset declarado = problema do documento
        // do cliente (exit 2; contrato no doc de módulo de shadow.rs).
        Err(e) => {
            erro(json, 2, format!("{e}"));
            return ExitCode::from(2);
        }
    };
    let gate = relatorio.gate_passa();
    if json {
        println!("{}", shadow::relatorio_json(&relatorio));
    } else {
        print!("{}", shadow::relatorio_humano(&relatorio));
    }
    if gate {
        ExitCode::from(0)
    } else {
        // O relatório já está no stdout; a rejeição do gate vai no stderr —
        // a autoridade formal deste subcomando é o próprio gate (exit 3).
        erro(
            json,
            3,
            format!(
                "gate REPROVADO: SMER {} < 100% — DATA_DRIFT {} + RADIANT_GAP {} (LEGACY_BUG {} \
                 e COSMETIC {} não reduzem o SMER; linhas no relatório)",
                shadow::smer_pct(relatorio.smer_bps),
                relatorio.categorias.data_drift,
                relatorio.categorias.radiant_gap,
                relatorio.categorias.legacy_bug,
                relatorio.categorias.cosmetic,
            ),
        );
        ExitCode::from(3)
    }
}

/// Pré-validação de FORMA (cliente, quick-xml 0.42):
/// (1) a declaração XML `<?xml ...?>` deve ser o primeiro evento do documento;
/// (2) o parse completo não pode produzir erro de sintaxe — com o default
/// `check_end_names = true`, tags de fechamento são comparadas contra a pilha
/// de abertura (tag fechada errada = erro).
///
/// Limite honesto declarado: quick-xml é parser sintático PULL — elemento
/// aberto que nunca fecha até o EOF não é detectado aqui, e referências de
/// entidade não resolvidas chegam como evento `GeneralRef` sem erro. A
/// autoridade estrutural/semântica é xmllint/oráculo oficial (ADR-0012/0018);
/// esta checagem é o fio elétrico barato ANTES de gastar o subprocesso.
fn checar_xml_bem_formado(bytes: &[u8]) -> Result<(), String> {
    let mut reader = quick_xml::Reader::from_reader(bytes);
    let mut buf = Vec::new();
    buf.clear();
    match reader.read_event_into(&mut buf) {
        Ok(quick_xml::events::Event::Decl(_)) => {}
        Ok(outro_primeiro_evento) => {
            return Err(format!(
                "a declaração XML <?xml ...?> deve ser o primeiro evento (obtido: {outro_primeiro_evento:?})"
            ));
        }
        Err(e) => return Err(format!("erro de sintaxe XML no início do documento: {e}")),
    }
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => return Ok(()),
            Ok(quick_xml::events::Event::Start(_))
            | Ok(quick_xml::events::Event::End(_))
            | Ok(quick_xml::events::Event::Empty(_))
            | Ok(quick_xml::events::Event::Text(_))
            | Ok(quick_xml::events::Event::CData(_))
            | Ok(quick_xml::events::Event::Comment(_))
            | Ok(quick_xml::events::Event::Decl(_))
            | Ok(quick_xml::events::Event::PI(_))
            | Ok(quick_xml::events::Event::DocType(_))
            | Ok(quick_xml::events::Event::GeneralRef(_)) => {}
            Err(e) => return Err(format!("erro de sintaxe XML: {e}")),
        }
    }
}
