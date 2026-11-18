//! Testes do BacenValidatorAdapter com EXECUTÁVEL-FAKE (nunca o `.jar` real).
//!
//! Doutrina: o "oráculo" dos testes é um script POSIX gerado em temp dir que
//! emula o CLI oficial (`validador_linha_comando.sh` em Unix — o nome real
//! resolvido pelo adapter); a "JRE" é um script fake que só precisa existir e
//! ser executável. Os fakes provam o CONTRATO (argv allowlist, chancela,
//! timeout HARD) por comportamento, não por espionar argv:
//! - argc errado → exit 9 (allowlist de cardinalidade);
//! - token inesperado → exit 7 (allowlist de conteúdo);
//! - sucesso → `<stem>_VALIDADO.zip` criado copiando a entrada (Manual
//!   SCR3040, seção D).
//!
//! Cobertura Windows (fake `.bat`) pendente de um runner Windows — os fakes
//! aqui são POSIX; declarado, não silêncio.
#![allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
#![allow(clippy::expect_used)] // testes
#![cfg(unix)]

use std::time::Duration;

use tempfile::TempDir;

use super::{BacenValidatorAdapter, JrePath, ValidationError};
use reg_core::error::RegError;

const XML_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?><Doc3040 data-base="2026-11"/>"#;
const STEM: &str = "doc3040_2026-11";

/// Fake de sucesso estrito: argc == 1 E cria `<stem>_VALIDADO.zip` copiando
/// a entrada.
const FAKE_OK: &str = r#"
[ "$#" -eq 1 ] || exit 9
cp "$1" "${1%.xml}_VALIDADO.zip" || exit 8
exit 0
"#;

/// Fake de sucesso com `no_warn`: argc == 2 E 2º argumento é o literal
/// `no_warn` (provam a allowlist por comportamento).
const FAKE_OK_NO_WARN: &str = r#"
[ "$#" -eq 2 ] || exit 9
[ "$2" = "no_warn" ] || exit 7
cp "$1" "${1%.xml}_VALIDADO.zip" || exit 8
exit 0
"#;

/// Fake de rejeição: exit != 0 com crítica no stderr.
const FAKE_REJEITA: &str = r#"
echo "ERRO: crítica semântica 3040-campo-saldo" >&2
exit 3
"#;

/// Fake lento: dorme além do timeout HARD (saída redirecionada para não
/// segurar o fd do pipe além da morte do wrapper).
const FAKE_LENTO: &str = r#"
sleep 30 >/dev/null 2>&1
exit 0
"#;

/// Fake sem chancela: encerra ok mas não produz o *_VALIDADO.zip.
const FAKE_SEM_ZIP: &str = r#"
exit 0
"#;

fn chmod_755(p: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// Fake da JRE: só precisa existir e ser executável para o builder.
fn fake_jre(dir: &TempDir) -> JrePath {
    let path = dir.path().join("java");
    std::fs::write(&path, "#!/bin/sh\n# fake da JRE\nexit 0\n").unwrap();
    chmod_755(&path);
    JrePath::new(path)
}

/// Fake da distribuição oficial: escreve o CLI wrapper com o NOME real que o
/// adapter resolve (`validador_linha_comando.sh` em Unix) e devolve jar_dir.
fn fake_jar_dir(dir: &TempDir, body: &str) -> std::path::PathBuf {
    let wrapper = dir.path().join(super::CLI_SCRIPT_NAME);
    std::fs::write(&wrapper, format!("#!/bin/sh\n{body}\n")).unwrap();
    chmod_755(&wrapper);
    dir.path().to_path_buf()
}

fn adapter(dir: &TempDir, body: &str, timeout: Duration) -> BacenValidatorAdapter {
    BacenValidatorAdapter::builder(fake_jar_dir(dir, body), fake_jre(dir))
        .unwrap()
        .timeout(timeout)
        .build()
}

#[test]
fn builder_rejeita_jar_ausente() {
    let dir = TempDir::new().unwrap();
    let err = BacenValidatorAdapter::builder(dir.path().join("nao-existe"), fake_jre(&dir)).unwrap_err();
    assert!(matches!(err, ValidationError::JarAusente { .. }), "obtido: {err:?}");
    let ValidationError::JarAusente { path } = err else {
        panic!("inacessível — matches! acima garante a variante")
    };
    assert!(path.contains(super::CLI_SCRIPT_NAME), "path: {path}");
}

#[test]
fn builder_rejeita_wrapper_sem_permissao_de_execucao() {
    let dir = TempDir::new().unwrap();
    let wrapper = dir.path().join(super::CLI_SCRIPT_NAME);
    std::fs::write(&wrapper, format!("#!/bin/sh\n{FAKE_OK}\n")).unwrap();
    // sem chmod — não executável
    let err = BacenValidatorAdapter::builder(dir.path().to_path_buf(), fake_jre(&dir)).unwrap_err();
    assert!(matches!(err, ValidationError::JarAusente { .. }), "obtido: {err:?}");
    let ValidationError::JarAusente { path } = err else {
        panic!("inacessível — matches! acima garante a variante")
    };
    assert!(path.contains(super::CLI_SCRIPT_NAME), "path: {path}");
}

#[test]
fn builder_rejeita_jre_ausente() {
    let dir = TempDir::new().unwrap();
    let err = BacenValidatorAdapter::builder(fake_jar_dir(&dir, FAKE_OK), JrePath::new(dir.path().join("sem-java")))
        .unwrap_err();
    assert!(matches!(err, ValidationError::JreAusente { .. }), "obtido: {err:?}");
    let ValidationError::JreAusente { path } = err else {
        panic!("inacessível — matches! acima garante a variante")
    };
    assert!(path.contains("sem-java"), "path: {path}");
}

#[test]
fn max_part_mb_default_512_e_range_conservador_adr_0018() {
    let dir = TempDir::new().unwrap();
    let err = BacenValidatorAdapter::builder(fake_jar_dir(&dir, FAKE_OK), fake_jre(&dir))
        .unwrap()
        .max_part_mb(2048)
        .unwrap_err();
    assert!(matches!(err, ValidationError::ConfigInvalida { .. }), "obtido: {err:?}");
    let ValidationError::ConfigInvalida { motivo } = err else {
        panic!("inacessível — matches! acima garante a variante")
    };
    assert!(motivo.contains("ADR-0018"), "motivo: {motivo}");

    // 512 (default) e 1024 são aceitos; getter expõe o valor configurado.
    let a512 = BacenValidatorAdapter::builder(fake_jar_dir(&dir, FAKE_OK), fake_jre(&dir))
        .unwrap()
        .max_part_mb(512)
        .unwrap()
        .build();
    assert_eq!(a512.max_part_mb(), 512);
    assert_eq!(a512.jar_dir(), dir.path());
    let a1024 = BacenValidatorAdapter::builder(fake_jar_dir(&dir, FAKE_OK), fake_jre(&dir))
        .unwrap()
        .max_part_mb(1024)
        .unwrap()
        .build();
    assert_eq!(a1024.max_part_mb(), 1024);
}

#[test]
fn sucesso_gera_zip_chancelado_com_sha256_correto() {
    let dir = TempDir::new().unwrap();
    let adapter = adapter(&dir, FAKE_OK, Duration::from_secs(30));
    let validated = adapter.validate(XML_DOC.as_bytes(), STEM).unwrap();

    // O zip chancelado contém exatamente os bytes da entrada (fake copia).
    assert_eq!(validated.zip_bytes, XML_DOC.as_bytes());
    // SHA-256 EXCLUSIVAMENTE sobre os bytes do zip chancelado — checagem
    // independente com sha2 direto (allow de unwrap só neste mod de testes).
    use sha2::{Digest, Sha256};
    let esperado = hex::encode(Sha256::digest(XML_DOC.as_bytes()));
    assert_eq!(validated.sha256, esperado);
    assert_eq!(validated.sha256.len(), 64);
    assert!(validated.sha256.bytes().all(|b| b.is_ascii_hexdigit()));
}

#[test]
fn allowlist_no_warn_ativa_envia_o_literal_como_segundo_arg() {
    let dir = TempDir::new().unwrap();
    let adapter = BacenValidatorAdapter::builder(fake_jar_dir(&dir, FAKE_OK_NO_WARN), fake_jre(&dir))
        .unwrap()
        .no_warn(true)
        .timeout(Duration::from_secs(30))
        .build();
    // Fake exige argc == 2 e "$2" == "no_warn" — sucesso prova os dois.
    assert!(adapter.validate(XML_DOC.as_bytes(), STEM).is_ok());
}

#[test]
fn allowlist_sem_no_warn_nao_envia_segundo_arg() {
    let dir = TempDir::new().unwrap();
    // Mesmo fake estrito de argc==2, mas com no_warn DESLIGADO: o wrapper
    // recebe 1 argumento → exit 9 → prova de que o literal NÃO é enviado.
    let adapter = adapter(&dir, FAKE_OK_NO_WARN, Duration::from_secs(30));
    let err = adapter.validate(XML_DOC.as_bytes(), STEM).unwrap_err();
    assert!(
        matches!(err, ValidationError::ValidadorRejeitou { .. }),
        "obtido: {err:?}"
    );
    let ValidationError::ValidadorRejeitou { codigo, mensagem } = err else {
        panic!("inacessível — matches! acima garante a variante")
    };
    assert_eq!(codigo, 9, "mensagem: {mensagem}");
}

#[test]
fn rejeicao_vira_validador_rejeitou_com_codigo_e_mensagem() {
    let dir = TempDir::new().unwrap();
    let adapter = adapter(&dir, FAKE_REJEITA, Duration::from_secs(30));
    let err = adapter.validate(XML_DOC.as_bytes(), STEM).unwrap_err();
    assert!(
        matches!(err, ValidationError::ValidadorRejeitou { .. }),
        "obtido: {err:?}"
    );
    let ValidationError::ValidadorRejeitou { codigo, mensagem } = err else {
        panic!("inacessível — matches! acima garante a variante")
    };
    assert_eq!(codigo, 3);
    assert!(mensagem.contains("3040-campo-saldo"), "mensagem: {mensagem}");
}

#[test]
fn processo_lento_vira_timeout_com_kill() {
    let dir = TempDir::new().unwrap();
    let adapter = adapter(&dir, FAKE_LENTO, Duration::from_millis(300));
    let err = adapter.validate(XML_DOC.as_bytes(), STEM).unwrap_err();
    let ValidationError::Timeout { timeout } = err else {
        panic!("esperado Timeout, obtido: {err:?}")
    };
    assert_eq!(timeout, Duration::from_millis(300));
}

#[test]
fn saida_ok_sem_zip_vira_zip_chancela_nao_encontrado() {
    let dir = TempDir::new().unwrap();
    let adapter = adapter(&dir, FAKE_SEM_ZIP, Duration::from_secs(30));
    let err = adapter.validate(XML_DOC.as_bytes(), STEM).unwrap_err();
    assert!(
        matches!(err, ValidationError::ZipChancelaNaoEncontrado { .. }),
        "obtido: {err:?}"
    );
    let ValidationError::ZipChancelaNaoEncontrado { esperado } = err else {
        panic!("inacessível — matches! acima garante a variante")
    };
    assert!(esperado.contains("_VALIDADO.zip"), "esperado: {esperado}");
}

#[test]
fn stem_eh_sanitizado_contra_path_traversal() {
    let dir = TempDir::new().unwrap();
    let adapter = adapter(&dir, FAKE_OK, Duration::from_secs(30));
    // Separadores não sobrevivem à sanitização — nada escapa do temp dir.
    assert!(!super::sanitize_stem("../../etc/passwd").contains('/'));
    assert!(!super::sanitize_stem("..\\..\\windows").contains('\\'));
    assert_eq!(super::sanitize_stem(""), "documento");
    // E o caminho completo funciona: validação Ok com stem malicioso.
    let validated = adapter.validate(XML_DOC.as_bytes(), "../../etc/passwd").unwrap();
    assert_eq!(validated.zip_bytes, XML_DOC.as_bytes());
    // Stem vazio cai no fallback sanitizado e valida igualmente.
    assert!(adapter.validate(XML_DOC.as_bytes(), "").is_ok());
}

#[test]
fn validation_error_vira_reg_error_da_camada_0() {
    let e = ValidationError::Timeout {
        timeout: Duration::from_secs(900),
    };
    let reg = RegError::from(e);
    assert!(matches!(reg, RegError::InvalidValue { .. }), "obtido: {reg:?}");
    let RegError::InvalidValue { field, reason } = reg else {
        panic!("inacessível — matches! acima garante a variante")
    };
    assert_eq!(field, "validador_bacen");
    assert!(reason.contains("timeout"), "reason: {reason}");
}
