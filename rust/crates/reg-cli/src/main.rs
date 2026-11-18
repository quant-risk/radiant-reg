//! reg-cli — Camada 3: composition root do wedge V1.
//!
//! CLI determinística (exit codes 0 aceito / 1 rejeição-transporte /
//! 2 validação-cliente / 3 erro-formal-BACEN), `--offline` por padrão,
//! `--json` para pipelines. Subcomando implementado: `validate` (pré-validação
//! de forma + chancela opcional via oráculo oficial B02/ADR-0018). Os demais
//! (`generate`, `xray`, `dossier`, `sta`, `senhaws`, `shadow-diff`, `mutate`)
//! chegam em S2-S6.
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
//! `--offline` e o oráculo: a flag bloqueia REDE, não subprocesso local — o
//! validador oficial é um processo local headless (doc de módulo do
//! `reg_cadoc::validator` detalha a doutrina e a allowlist de argv).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use reg_cadoc::validator::{BacenValidatorAdapter, JrePath, ValidationError};

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
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.comando {
        Some(Comando::Validate {
            cadoc,
            input,
            chancela,
            jar_dir,
            jre,
            no_warn,
        }) => run_validate(cli.json, cadoc, input, chancela, jar_dir, jre, no_warn),
        None => {
            eprintln!("reg-cli: esqueleto da Fase 1 — nenhum subcomando informado (implementados: validate)");
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
