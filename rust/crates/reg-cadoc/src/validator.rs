//! BacenValidatorAdapter — ORÁCULO OFICIAL de submissão B02 (ADR-0018,
//! docs/blueprint/20-ARQUITETURA-RUST.md §17).
//!
//! # Doutrina
//! - O `.jar` oficial do validador SCR3040 (BCB) é a autoridade final de
//!   submissão: **nenhum ZIP sai para o STA sem chancela** — B02: "Arquivo
//!   .ZIP deve ser gerado pelo aplicativo validador".
//! - O SHA-256 do artefato é calculado **EXCLUSIVAMENTE sobre os bytes do
//!   `*_VALIDADO.zip`** produzido pelo validador — nunca sobre o XML/ZIP de
//!   pré-validação.
//! - `--max-part-mb` conservador em 512–1024 MiB (ADR-0018).
//!
//! # CLI real do validador oficial (descoberta do agente S3)
//! A interface de linha de comando distribuída pelo BCB é
//! `validador_linha_comando.bat CaminhoArquivo [no_warn]`, exigindo
//! Java ≥ 1.8, e a saída de sucesso é o arquivo `<stem>_VALIDADO.zip` gravado
//! ao lado da entrada. Este adapter invoca EXATAMENTE esse CLI:
//! - **Windows**: o wrapper é `validador_linha_comando.bat`, tratado como
//!   script — `std::process::Command` executa `.bat` diretamente (o std faz a
//!   ponte via cmd.exe com escaping estrito de argumentos, pós
//!   CVE-2024-24576). O adapter NUNCA monta string de shell nem passa por
//!   `sh -c`/`cmd /C` explícito.
//! - **Unix**: o wrapper é `validador_linha_comando.sh` executável
//!   (equivalente a `java -jar validador.jar "$1"` — o BCB só distribui o
//!   `.bat`; o operador provisiona o `.sh` junto à distribuição).
//! - A JRE validada no builder é colocada À FRENTE do `PATH` do processo
//!   filho, para que o wrapper resolva exatamente o `java` que o adapter
//!   validou (fail-fast no builder, nunca `Option` silencioso).
//!
//! PENDÊNCIA declarada (S1 re-download): a ordem de argumentos interna do
//! `.bat`/`.jar` re-baixado precisa ser reconfirmada. O nome do `.jar` não é
//! checado porque só será conhecido no re-download — a existência executável
//! do CLI wrapper é o proxy de "distribuição completa".
//!
//! # Sandbox honesto (`--offline` do CLI bloqueia REDE, não subprocesso local)
//! O validador oficial é um processo LOCAL headless que não abre conexão de
//! rede; chancelar sob `reg-cli --offline` é compatível — a flag desliga saída
//! de rede, não invocação de subprocesso local. Defesas em profundidade:
//! - **Allowlist de argv**: os ÚNICOS valores interpolados são (a) o path do
//!   wrapper (resolvido e validado no builder), (b) o path do arquivo de
//!   entrada (escrito pelo próprio adapter em temp dir, com stem sanitizado
//!   contra path traversal) e (c) o literal `no_warn`. Nenhum input cru do
//!   usuário entra no argv; os bytes do XML nunca trafegam por argv/env.
//! - **Working dir isolado** em `tempfile::TempDir` (removido no drop).
//! - **Timeout HARD**: poll + `kill` — o processo é morto; no timeout, as
//!   threads de drenagem de pipe NÃO são esperadas (netos órfãos podem segurar
//!   o fd do pipe; o output é irrelevante no timeout).

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use thiserror::Error;

use reg_core::error::RegError;

/// Erro tipado do adapter — nenhum `unwrap`, nenhuma falha silenciosa.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// JRE declarada no builder não existe (ou não é executável).
    #[error("JRE ausente ou não executável em {path}")]
    JreAusente { path: String },
    /// Distribuição do validador oficial incompleta: o CLI wrapper
    /// (`validador_linha_comando.bat`/`.sh`) não existe (ou não é executável)
    /// no diretório declarado.
    #[error("distribuição do validador oficial incompleta — CLI ausente em {path}")]
    JarAusente { path: String },
    /// Configuração do builder fora do range conservador do ADR-0018.
    #[error("configuração inválida do validador: {motivo}")]
    ConfigInvalida { motivo: String },
    /// O validador oficial rejeitou o documento (exit != 0). `codigo` é o
    /// exit code; `mensagem` é a cauda de stderr/stdout do processo.
    #[error("validador oficial rejeitou o documento (código {codigo}): {mensagem}")]
    ValidadorRejeitou { codigo: i32, mensagem: String },
    /// Timeout HARD: o processo foi morto (kill) após exceder o limite.
    #[error("validador excedeu o timeout de {timeout:?} e foi morto (kill)")]
    Timeout { timeout: Duration },
    /// O validador encerrou com sucesso mas não produziu o `*_VALIDADO.zip`.
    #[error("validador encerrou ok mas o zip chancelado {esperado} não foi encontrado")]
    ZipChancelaNaoEncontrado { esperado: String },
    /// Falha de I/O (spawn, temp dir, leitura de pipes/zip).
    #[error("erro de I/O no adapter do validador: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<ValidationError> for RegError {
    fn from(e: ValidationError) -> Self {
        RegError::InvalidValue {
            field: "validador_bacen".to_string(),
            reason: e.to_string(),
        }
    }
}

/// Path do executável `java` da JRE (≥ 1.8). Newtype para nunca confundir
/// com o path do wrapper nem aceitar `Option` silencioso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JrePath(PathBuf);

impl JrePath {
    pub fn new(path: PathBuf) -> Self {
        JrePath(path)
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// Chancela oficial: os bytes do `*_VALIDADO.zip` + o SHA-256 hex calculado
/// EXCLUSIVAMENTE sobre esses bytes (ADR-0018 — nunca sobre o ZIP/XML de
/// pré-validação).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedZip {
    /// Bytes completos do zip chancelado (artefato que sai para o STA).
    pub zip_bytes: Vec<u8>,
    /// SHA-256 hex dos bytes do zip chancelado.
    pub sha256: String,
}

/// Adapter do oráculo oficial. Construa via [`BacenValidatorAdapter::builder`].
#[derive(Debug, Clone)]
pub struct BacenValidatorAdapter {
    /// Diretório da distribuição oficial (contém o CLI wrapper + .jar).
    jar_dir: PathBuf,
    /// CLI wrapper resolvido no builder (`validador_linha_comando.bat` no
    /// Windows / `validador_linha_comando.sh` em Unix).
    cli_script: PathBuf,
    jre: JrePath,
    max_part_mb: u32,
    timeout: Duration,
    no_warn: bool,
}

/// Limites do ADR-0018: `--max-part-mb` conservador em 512–1024 MiB.
const MAX_PART_MB_MIN: u32 = 512;
const MAX_PART_MB_MAX: u32 = 1024;

/// Nome do CLI wrapper por plataforma (descoberta do agente S3).
#[cfg(windows)]
const CLI_SCRIPT_NAME: &str = "validador_linha_comando.bat";
#[cfg(not(windows))]
const CLI_SCRIPT_NAME: &str = "validador_linha_comando.sh";

impl BacenValidatorAdapter {
    /// Builder: valida existência/executabilidade da JRE e do CLI wrapper da
    /// distribuição — erro tipado, nunca `Option` silencioso.
    pub fn builder(jar_dir: PathBuf, jre: JrePath) -> Result<Builder, ValidationError> {
        if !is_executable_file(jre.as_path()) {
            return Err(ValidationError::JreAusente {
                path: jre.as_path().display().to_string(),
            });
        }
        let cli_script = jar_dir.join(CLI_SCRIPT_NAME);
        if !is_executable_file(&cli_script) {
            return Err(ValidationError::JarAusente {
                path: cli_script.display().to_string(),
            });
        }
        Ok(Builder {
            jar_dir,
            cli_script,
            jre,
            max_part_mb: MAX_PART_MB_MIN,
            timeout: Duration::from_secs(900),
            no_warn: false,
        })
    }

    /// Diretório da distribuição oficial declarado no builder (diagnóstico).
    pub fn jar_dir(&self) -> &Path {
        &self.jar_dir
    }

    /// Cap de partição configurado, em MiB (diagnóstico; default 512,
    /// range conservador do ADR-0018).
    pub fn max_part_mb(&self) -> u32 {
        self.max_part_mb
    }

    /// Roda o validador oficial headless sobre `xml_bytes` e devolve a
    /// chancela: o `<stem>_VALIDADO.zip` + SHA-256 exclusivamente sobre os
    /// bytes dele.
    ///
    /// `stem` é o nome-base do arquivo de entrada (sem extensão); é
    /// sanitizado contra path traversal — a chancela refere-se ao nome
    /// sanitizado. Contrato de sandbox: processo local, sem rede; argv por
    /// allowlist; timeout HARD (kill) por chamada; working dir isolado.
    pub fn validate(&self, xml_bytes: &[u8], stem: &str) -> Result<ValidatedZip, ValidationError> {
        let workdir = tempfile::TempDir::new()?;
        let stem_limpo = sanitize_stem(stem);
        self.validar_em(workdir.path(), &stem_limpo, xml_bytes)
    }

    /// Núcleo: valida dentro de um workdir isolado já criado.
    fn validar_em(
        &self,
        workdir: &Path,
        stem: &str,
        xml_bytes: &[u8],
    ) -> Result<ValidatedZip, ValidationError> {
        // Allowlist de argv: flags fixas + paths validados + literal "no_warn".
        let input_path = workdir.join(format!("{stem}.xml"));
        std::fs::write(&input_path, xml_bytes)?;

        let mut cmd = Command::new(&self.cli_script);
        cmd.arg(&input_path);
        if self.no_warn {
            cmd.arg("no_warn");
        }
        // O wrapper oficial resolve `java` do PATH — prepend do diretório da
        // JRE validada no builder (garante que o java usado é o validado).
        if let Some(bin_dir) = self.jre.as_path().parent()
            && let Ok(path_prependido) = std::env::join_paths(
                std::iter::once(bin_dir.to_path_buf()).chain(std::env::split_paths(
                    &std::env::var_os("PATH").unwrap_or_default(),
                )),
            )
        {
            cmd.env("PATH", path_prependido);
        }
        // SEM shell: o script é executado diretamente (no Windows, o std
        // trata .bat como script com escaping estrito — ver doc de módulo).
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn()?;

        // Pipes drenados em threads dedicadas — evita deadlock de pipe cheio
        // durante o poll de timeout e mantém o caminho livre de `unwrap`.
        let out_pipe = child.stdout.take();
        let err_pipe = child.stderr.take();
        let out_handle = std::thread::spawn(move || {
            let mut buf = String::new();
            if let Some(mut p) = out_pipe {
                let _ = p.read_to_string(&mut buf);
            }
            buf
        });
        let err_handle = std::thread::spawn(move || {
            let mut buf = String::new();
            if let Some(mut p) = err_pipe {
                let _ = p.read_to_string(&mut buf);
            }
            buf
        });

        // Timeout HARD por chamada: poll + kill (sem shell). No timeout o
        // processo é morto e NÃO esperamos as threads de drenagem: netos
        // órfãos (ex. java filho do wrapper) podem segurar o fd do pipe por
        // tempo indeterminado — o output é irrelevante no caminho de timeout.
        let start = Instant::now();
        let status = loop {
            match child.try_wait()? {
                Some(st) => break st,
                None if start.elapsed() >= self.timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ValidationError::Timeout {
                        timeout: self.timeout,
                    });
                }
                None => std::thread::sleep(Duration::from_millis(10)),
            }
        };

        let out = out_handle.join().unwrap_or_default();
        let err = err_handle.join().unwrap_or_default();

        if !status.success() {
            let codigo = status.code().unwrap_or(-1);
            return Err(ValidationError::ValidadorRejeitou {
                codigo,
                mensagem: tail_message(&err, &out),
            });
        }

        // Chancela: o validador grava `<stem>_VALIDADO.zip` na mesma pasta do
        // arquivo de entrada (Manual do Validador SCR3040, seção D).
        let zip_path = workdir.join(format!("{stem}_VALIDADO.zip"));
        if !zip_path.is_file() {
            return Err(ValidationError::ZipChancelaNaoEncontrado {
                esperado: zip_path.display().to_string(),
            });
        }
        let zip_bytes = std::fs::read(&zip_path)?;

        // SHA-256 EXCLUSIVAMENTE sobre os bytes do zip chancelado (ADR-0018).
        let sha256 = hex::encode(Sha256::digest(&zip_bytes));
        Ok(ValidatedZip { zip_bytes, sha256 })
    }
}

/// Builder fluído do adapter (validação pesada acontece em `builder()`).
#[derive(Debug, Clone)]
pub struct Builder {
    jar_dir: PathBuf,
    cli_script: PathBuf,
    jre: JrePath,
    max_part_mb: u32,
    timeout: Duration,
    no_warn: bool,
}

impl Builder {
    /// Cap de partição em MiB — restringido ao range conservador do
    /// ADR-0018 (512–1024).
    pub fn max_part_mb(mut self, mb: u32) -> Result<Self, ValidationError> {
        if !(MAX_PART_MB_MIN..=MAX_PART_MB_MAX).contains(&mb) {
            return Err(ValidationError::ConfigInvalida {
                motivo: format!(
                    "max_part_mb={mb} fora do range conservador {MAX_PART_MB_MIN}–{MAX_PART_MB_MAX} MiB (ADR-0018)"
                ),
            });
        }
        self.max_part_mb = mb;
        Ok(self)
    }

    /// Timeout HARD por chamada (default 900s).
    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = t;
        self
    }

    /// Liga/desliga o token `no_warn` do CLI oficial (Manual SCR3040 §F).
    pub fn no_warn(mut self, enabled: bool) -> Self {
        self.no_warn = enabled;
        self
    }

    pub fn build(self) -> BacenValidatorAdapter {
        BacenValidatorAdapter {
            jar_dir: self.jar_dir,
            cli_script: self.cli_script,
            jre: self.jre,
            max_part_mb: self.max_part_mb,
            timeout: self.timeout,
            no_warn: self.no_warn,
        }
    }
}

/// Sanitiza o stem: apenas [A-Za-z0-9._-] — separadores de path não
/// sobrevivem (defesa contra path traversal no temp dir).
fn sanitize_stem(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        .collect();
    if cleaned.is_empty() {
        "documento".to_string()
    } else {
        cleaned
    }
}

fn is_executable_file(p: &Path) -> bool {
    p.is_file() && {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::metadata(p)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            true
        }
    }
}

/// Cauda da mensagem de erro do validador (stderr tem precedência).
/// Conta CHARS, não bytes — corte em limite de char nunca entra em pânico
/// com saída multibyte do validador.
fn tail_message(stderr: &str, stdout: &str) -> String {
    let raw = if stderr.trim().is_empty() { stdout } else { stderr };
    let trimmed = raw.trim();
    const MAX_CHARS: usize = 2000;
    let total = trimmed.chars().count();
    if total <= MAX_CHARS {
        trimmed.to_string()
    } else {
        trimmed.chars().skip(total - MAX_CHARS).collect()
    }
}

#[cfg(test)]
#[path = "validator_tests.rs"]
mod validator_tests;
