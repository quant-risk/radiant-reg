//! Testes black-box do subcomando `reg-cli extract-catalog` (100% local,
//! compatível com `--offline`).

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_reg-cli")
}

fn repo_3040(file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../3040")
        .join(file)
}

fn run(args: &[&str]) -> Result<(i32, String, String), Box<dyn std::error::Error>> {
    let out = Command::new(bin_path()).args(args).output()?;
    Ok((
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    ))
}

#[test]
fn extract_catalog_offline_gera_jsons_canonicos_3040() -> Result<(), Box<dyn std::error::Error>> {
    let criticas = repo_3040("SCR3040_Criticas.xls");
    let leiaute = repo_3040("SCR3040_Leiaute.xls");
    assert!(criticas.exists(), "fixture ausente: {}", criticas.display());
    assert!(leiaute.exists(), "fixture ausente: {}", leiaute.display());

    let out_dir = TempDir::new()?;
    let criticas_s = criticas.display().to_string();
    let leiaute_s = leiaute.display().to_string();
    let out_s = out_dir.path().display().to_string();

    let (code, stdout, stderr) = run(&[
        "--offline",
        "--json",
        "extract-catalog",
        "--criticas",
        &criticas_s,
        "--leiaute",
        &leiaute_s,
        "--out-dir",
        &out_s,
    ])?;
    assert_eq!(code, 0, "stdout={stdout}\nstderr={stderr}");

    let resumo: serde_json::Value = serde_json::from_str(stdout.trim())?;
    assert_eq!(resumo["resultado"], "aceito");
    assert_eq!(resumo["criticas"]["total"], 255);
    assert_eq!(resumo["criticas"]["habilitado_s"], 226);
    assert_eq!(resumo["criticas"]["habilitado_n"], 29);

    let criticas_json_path = out_dir.path().join("criticas_3040.json");
    let leiaute_json_path = out_dir.path().join("leiaute_3040_campos.json");
    assert!(criticas_json_path.exists());
    assert!(leiaute_json_path.exists());

    let criticas_doc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&criticas_json_path)?)?;
    let lista = criticas_doc["criticas"]
        .as_array()
        .ok_or("criticas nao eh array")?;
    assert_eq!(lista.len(), 255);
    for codigo in ["B12", "C56b", "S118", "C71"] {
        assert!(
            lista.iter().any(|c| c["codigo"] == codigo),
            "regra {codigo} ausente"
        );
    }

    let leiaute_doc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&leiaute_json_path)?)?;
    assert!(
        leiaute_doc["campos"]
            .as_array()
            .is_some_and(|a| !a.is_empty())
    );
    assert!(
        leiaute_doc["historico_atualizacoes"]
            .as_array()
            .is_some_and(|a| !a.is_empty())
    );
    Ok(())
}

#[test]
fn extract_catalog_arquivo_ausente_exit_1() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = TempDir::new()?;
    let missing = out_dir.path().join("nao-existe.xls").display().to_string();
    let leiaute = repo_3040("SCR3040_Leiaute.xls").display().to_string();
    let out_s = out_dir.path().display().to_string();

    let (code, _stdout, stderr) = run(&[
        "extract-catalog",
        "--criticas",
        &missing,
        "--leiaute",
        &leiaute,
        "--out-dir",
        &out_s,
    ])?;
    assert_eq!(code, 1, "stderr={stderr}");
    assert!(stderr.contains("rejeição-transporte"), "stderr={stderr}");
    Ok(())
}

#[test]
fn extract_catalog_xls_invalido_exit_2() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = TempDir::new()?;
    let fake_xls = out_dir.path().join("corrompido.xls");
    std::fs::write(&fake_xls, b"nao eh um arquivo xls")?;
    let fake_s = fake_xls.display().to_string();
    let leiaute = repo_3040("SCR3040_Leiaute.xls").display().to_string();
    let out_s = out_dir.path().display().to_string();

    let (code, _stdout, stderr) = run(&[
        "extract-catalog",
        "--criticas",
        &fake_s,
        "--leiaute",
        &leiaute,
        "--out-dir",
        &out_s,
    ])?;
    assert_eq!(code, 2, "stderr={stderr}");
    assert!(stderr.contains("validação-cliente"), "stderr={stderr}");
    Ok(())
}
