//! Testes black-box do subcomando `reg-cli validate` (contrato de exit codes
//! 0/1/2/3 documentado no main.rs). Roda o BINÁRIO real via
//! `CARGO_BIN_EXE_reg-cli` — sem shell, argv direto.
//!
//! Os testes de `--chancela` usam um ORÁCULO-FAKE (script POSIX que emula o
//! CLI oficial `validador_linha_comando.sh` criando o `<stem>_VALIDADO.zip`
//! copiando a entrada) — NUNCA dependem do `.jar` real do BCB. Por isso são
//! `#[cfg(unix)]`; cobertura Windows (fake `.bat`) fica pendente de runner.

#![allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
#![allow(clippy::expect_used)] // testes

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const XML_OK: &str = r#"<?xml version="1.0" encoding="UTF-8"?><Doc3040 data-base="2026-11"/>"#;

/// Fake do CLI oficial em Unix: argc == 1 e `<stem>_VALIDADO.zip` = cópia da
/// entrada (Manual do Validador SCR3040, seção D).
const FAKE_ORACULO_OK: &str = r#"
[ "$#" -eq 1 ] || exit 9
cp "$1" "${1%.xml}_VALIDADO.zip" || exit 8
exit 0
"#;

/// Fake que rejeita: exit 3 com crítica no stderr.
const FAKE_ORACULO_REJEITA: &str = r#"
echo "ERRO: crítica semântica 3040-campo-saldo" >&2
exit 3
"#;

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_reg-cli")
}

/// Roda o reg-cli com argv direto (sem shell) e devolve (exit, stdout, stderr).
fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin_path()).args(args).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn escrever(dir: &TempDir, nome: &str, conteudo: &str) -> String {
    let p = dir.path().join(nome);
    std::fs::write(&p, conteudo).unwrap();
    p.display().to_string()
}

fn chmod_755(p: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// Cria a distribuição fake do oráculo: jar_dir com o wrapper executável +
/// JRE fake executável (o adapter exige ambas).
fn oraculo_fake(dir: &TempDir, corpo_wrapper: &str) -> (String, String) {
    let wrapper = dir.path().join("validador_linha_comando.sh");
    std::fs::write(&wrapper, format!("#!/bin/sh\n{corpo_wrapper}\n")).unwrap();
    chmod_755(&wrapper);
    let java = dir.path().join("java");
    std::fs::write(&java, "#!/bin/sh\n# fake da JRE\nexit 0\n").unwrap();
    chmod_755(&java);
    (
        dir.path().display().to_string(),
        java.display().to_string(),
    )
}

#[test]
fn pre_validacao_xml_bem_formado_exit_0_com_aviso_honesto() {
    let dir = TempDir::new().unwrap();
    let xml = escrever(&dir, "doc3040_2026-11.xml", XML_OK);
    let (code, stdout, stderr) = run(&["--offline", "validate", "--cadoc", "3040", "--input", &xml]);
    assert_eq!(code, 0, "stdout: {stdout}\nstderr: {stderr}");
    assert!(stdout.contains("pré-validação OK"), "stdout: {stdout}");
    // Aviso honesto: pré-validação NÃO é chancela (B02/ADR-0018).
    assert!(stderr.contains("NÃO é chancela"), "stderr: {stderr}");
    assert!(stderr.contains("ADR-0018"), "stderr: {stderr}");
}

#[test]
fn pre_validacao_json_saida_estruturada() {
    let dir = TempDir::new().unwrap();
    let xml = escrever(&dir, "doc3040.xml", XML_OK);
    let (code, stdout, _stderr) = run(&["--json", "validate", "--cadoc", "3040", "--input", &xml]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(v["resultado"], "aceito");
    assert_eq!(v["modo"], "pre_validacao");
    assert!(v["sha256"].is_null());
}

#[test]
fn xml_mal_formado_tag_descasada_exit_3() {
    let dir = TempDir::new().unwrap();
    // check_end_names (default do quick-xml 0.42): End "Cli" ≠ topo "Doc3040".
    let xml = escrever(&dir, "mal.xml", "<?xml version=\"1.0\"?><Doc3040><Cli></Doc3040></Cli>");
    let (code, _stdout, stderr) = run(&["validate", "--cadoc", "3040", "--input", &xml]);
    assert_eq!(code, 3, "stderr: {stderr}");
    assert!(stderr.contains("erro formal"), "stderr: {stderr}");
}

#[test]
fn xml_sem_declaracao_exit_3() {
    let dir = TempDir::new().unwrap();
    let xml = escrever(&dir, "sem-decl.xml", "<Doc3040 data-base=\"2026-11\"/>");
    let (code, _stdout, stderr) = run(&["validate", "--cadoc", "3040", "--input", &xml]);
    assert_eq!(code, 3, "stderr: {stderr}");
    assert!(stderr.contains("declaração"), "stderr: {stderr}");
}

#[test]
fn arquivo_ausente_exit_1() {
    let dir = TempDir::new().unwrap();
    let xml = dir.path().join("nao-existe.xml").display().to_string();
    let (code, _stdout, stderr) = run(&["validate", "--cadoc", "3040", "--input", &xml]);
    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(stderr.contains("rejeição-transporte"), "stderr: {stderr}");
}

#[test]
fn cadoc_nao_suportado_exit_2() {
    let dir = TempDir::new().unwrap();
    let xml = escrever(&dir, "doc.xml", XML_OK);
    let (code, _stdout, stderr) = run(&["validate", "--cadoc", "3050", "--input", &xml]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stderr.contains("sem suporte"), "stderr: {stderr}");
}

#[test]
fn chancela_sem_jar_dir_e_jre_exit_2() {
    let dir = TempDir::new().unwrap();
    let xml = escrever(&dir, "doc.xml", XML_OK);
    let (code, _stdout, stderr) = run(&["validate", "--cadoc", "3040", "--input", &xml, "--chancela"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stderr.contains("--jar-dir"), "stderr: {stderr}");
}

#[cfg(unix)]
#[test]
fn chancela_com_oraculo_fake_exit_0_com_sha256_da_chancela() {
    let dir = TempDir::new().unwrap();
    let xml = escrever(&dir, "doc3040_2026-11.xml", XML_OK);
    let (jar_dir, jre) = oraculo_fake(&dir, FAKE_ORACULO_OK);
    let (code, stdout, stderr) = run(&[
        "validate",
        "--cadoc",
        "3040",
        "--input",
        &xml,
        "--chancela",
        "--jar-dir",
        &jar_dir,
        "--jre",
        &jre,
    ]);
    assert_eq!(code, 0, "stdout: {stdout}\nstderr: {stderr}");
    assert!(stdout.contains("chancela OK"), "stdout: {stdout}");
    // SHA-256 impresso = SHA-256 dos bytes do zip chancelado (fake copia a
    // entrada → os bytes do zip são os bytes do XML). Cálculo independente:
    use sha2::{Digest, Sha256};
    let esperado = hex::encode(Sha256::digest(XML_OK.as_bytes()));
    assert!(stdout.contains(&esperado), "stdout: {stdout}\nesperado: {esperado}");
}

#[cfg(unix)]
#[test]
fn chancela_com_oraculo_fake_que_rejeita_exit_3() {
    let dir = TempDir::new().unwrap();
    let xml = escrever(&dir, "doc3040.xml", XML_OK);
    let (jar_dir, jre) = oraculo_fake(&dir, FAKE_ORACULO_REJEITA);
    let (code, _stdout, stderr) = run(&[
        "validate",
        "--cadoc",
        "3040",
        "--input",
        &xml,
        "--chancela",
        "--jar-dir",
        &jar_dir,
        "--jre",
        &jre,
    ]);
    assert_eq!(code, 3, "stderr: {stderr}");
    // A crítica do oráculo chega no stderr do CLI (proveniência auditável).
    assert!(stderr.contains("3040-campo-saldo"), "stderr: {stderr}");
    assert!(stderr.contains("erro formal BACEN"), "stderr: {stderr}");
}

#[cfg(unix)]
#[test]
fn chancela_com_jre_inexistente_exit_2() {
    let dir = TempDir::new().unwrap();
    let xml = escrever(&dir, "doc3040.xml", XML_OK);
    let (jar_dir, _jre_real) = oraculo_fake(&dir, FAKE_ORACULO_OK);
    let jre_morta = dir.path().join("java-inexistente").display().to_string();
    let (code, _stdout, stderr) = run(&[
        "validate",
        "--cadoc",
        "3040",
        "--input",
        &xml,
        "--chancela",
        "--jar-dir",
        &jar_dir,
        "--jre",
        &jre_morta,
    ]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stderr.contains("JRE ausente"), "stderr: {stderr}");
}

#[test]
fn sem_subcomando_exit_2_esqueleto() {
    let (code, _stdout, stderr) = run(&[]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stderr.contains("esqueleto"), "stderr: {stderr}");
}
