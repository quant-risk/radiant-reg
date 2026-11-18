//! Golden corpus — o loader do corpus de paridade Rust × v1.
//!
//! Os fixtures JSON vivem em `tests/golden/*.json` e são gerados por outro
//! agente; este módulo é o CONSUMIDOR (docs/blueprint/20-ARQUITETURA-RUST.md
//! §6 — paridade como gate, contagem honesta).
//!
//! Regra de ouro: o loader nunca inventa números. `GoldenInventory` traz os
//! counts declarados do gerador do corpus; o runner (`parity.rs`) recalcula
//! do materializado e a comparação é explícita. Skip de caso só existe no
//! estado DECLARADO (`RuleNotImplemented`) — nunca silêncio.

pub mod parity;

use reg_core::error::RegError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Um caso golden: XML de entrada + expectativa + a regra que o exerce.
/// O corpus real (tests/golden/fixtures_*.json) usa snake_case; os aliases
/// camelCase mantêm compatibilidade com a convenção declarada no blueprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoldenCase {
    /// Id estável do caso, ex. "3040-TestS04_CreditoALiberar_Zero-01".
    pub id: String,
    /// Nome legível do teste de origem (função Test do v1).
    #[serde(alias = "test_name")]
    pub test_name: String,
    /// Código da regra, ex. "S04", "DRM-01".
    #[serde(alias = "rule_code")]
    pub rule_code: String,
    /// XML canônico de entrada.
    pub xml: String,
    /// Expectativa do caso: `"nil"` (apply deve ser Ok) ou `"substring"`
    /// (apply Ok E o XML contém `substring`) — interpretadas pelo runner
    /// (`parity.rs`, classificação honesta). Qualquer outro valor não é
    /// interpretável pelo runner e é contado como `expect_nao_interpretadas`
    /// (estado DECLARADO — nunca contado como passed).
    pub expect: serde_json::Value,
    /// Substring que o XML deve conter na expectativa `"substring"`. Vazia =
    /// verificação só superficial (contada como `superficiais` no relatório).
    #[serde(default)]
    pub substring: String,
}

/// Uma regra do inventário (shape do gerador do corpus: objeto com code,
/// cadoc, sheet, severity, class). Campos descritivos têm default — o
/// consumidor de paridade só exige `code`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEntry {
    /// Código da regra, ex. "DRM-01", "S04".
    pub code: String,
    /// Cadoc de origem, ex. "2060", "3040".
    pub cadoc: String,
    /// Sheet de extração no .xls oficial.
    #[serde(default)]
    pub sheet: String,
    /// Severidade ("E" = eliminatória).
    #[serde(default)]
    pub severity: String,
    /// Classe de cobertura ("wired", "orphan", "stub").
    #[serde(default)]
    pub class: String,
}

/// Inventário declarado do corpus (`inventory.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenInventory {
    /// Regras cobertas pelo corpus, com metadata de extração.
    pub rules: Vec<RuleEntry>,
    /// Counts por classe de cobertura, declarados pelo gerador
    /// (wired/orphan/stub + `total`; total == rules.len() — validado).
    pub counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub source_commit: Option<String>,
}

impl GoldenInventory {
    /// Total declarado (counts["total"]) — validado contra rules.len().
    pub fn total(&self) -> Option<usize> {
        self.counts.get("total").copied()
    }

    /// Validação honesta: counts["total"] == número de regras listadas.
    pub fn validate(&self) -> Result<(), RegError> {
        const FIELD: &str = "golden.inventory";
        match self.total() {
            Some(total) if total != self.rules.len() => {
                return Err(RegError::InvalidValue {
                    field: FIELD.into(),
                    reason: format!("counts.total {total} ≠ {0} regras listadas", self.rules.len()),
                });
            }
            None => {
                return Err(RegError::InvalidValue {
                    field: FIELD.into(),
                    reason: "counts.total ausente — contagem dishonest é erro".into(),
                });
            }
            Some(_) => {}
        }
        Ok(())
    }
}

/// Carrega `inventory.json` de um diretório de fixtures.
pub fn load_inventory(dir: &Path) -> Result<GoldenInventory, RegError> {
    const FIELD: &str = "golden.inventory";
    let path = dir.join("inventory.json");
    let raw = std::fs::read_to_string(&path).map_err(|e| RegError::InvalidValue {
        field: FIELD.into(),
        reason: format!("ler {}: {e}", path.display()),
    })?;
    let inv: GoldenInventory = serde_json::from_str(&raw).map_err(|e| RegError::InvalidValue {
        field: FIELD.into(),
        reason: format!("JSON inválido em {}: {e}", path.display()),
    })?;
    inv.validate()?;
    Ok(inv)
}

/// Carrega todos os casos de um diretório (todos os `*.json` exceto
/// `inventory.json`), ordenados por id para determinismo.
///
/// O corpus real é particionado por cadoc: cada `fixtures_<cadoc>.json` é um
/// objeto `{ "cases": [...], counts, ... }`. Arquivos JSON sem a chave
/// `cases` (ex. `not_extracted.json`, manifesto de funções não extraídas)
/// são pulados como NÃO-CASOS — decisão estrutural documentada, não skip de
/// dado. Um arquivo COM `cases` que não parseia é erro tipado.
pub fn load_cases(dir: &Path) -> Result<Vec<GoldenCase>, RegError> {
    const FIELD: &str = "golden.cases";
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| RegError::InvalidValue { field: FIELD.into(), reason: format!("dir {}: {e}", dir.display()) })?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .filter(|p| p.file_name().is_some_and(|n| n != "inventory.json"))
        .collect();
    files.sort();

    let mut cases = Vec::new();
    for f in &files {
        let raw = std::fs::read_to_string(f).map_err(|e| RegError::InvalidValue {
            field: FIELD.into(),
            reason: format!("ler {}: {e}", f.display()),
        })?;
        let value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
            RegError::InvalidValue { field: FIELD.into(), reason: format!("JSON inválido em {}: {e}", f.display()) }
        })?;
        let Some(cases_json) = value.get("cases").cloned() else {
            // manifesto (not_extracted.json etc.), não é arquivo de casos
            continue;
        };
        let file_cases: Vec<GoldenCase> = serde_json::from_value(cases_json).map_err(|e| {
            RegError::InvalidValue { field: FIELD.into(), reason: format!("casos inválidos em {}: {e}", f.display()) }
        })?;
        cases.extend(file_cases);
    }
    cases.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(cases)
}

/// Counts HONESTOS de uma lista materializada de casos — recalculados do
/// conteúdo, não do inventário. O teste de paridade compara os dois lados.
pub fn honest_counts(cases: &[GoldenCase]) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for c in cases {
        *counts.entry(c.rule_code.clone()).or_default() += 1;
    }
    counts
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn golden_case_parseia_camel_case() {
        let json = r#"{
            "id": "3040-tit-001",
            "testName": "saldo zero com vigencia valida",
            "ruleCode": "3040-TIT-002",
            "xml": "<Doc3040/>",
            "expect": {"outcome": "ok"}
        }"#;
        let case: GoldenCase = serde_json::from_str(json).unwrap();
        assert_eq!(case.id, "3040-tit-001");
        assert_eq!(case.test_name, "saldo zero com vigencia valida");
        assert_eq!(case.rule_code, "3040-TIT-002");
    }

    #[test]
    fn inventory_inconsistente_erra() {
        let entry = |c: &str| RuleEntry {
            code: c.into(),
            cadoc: "3040".into(),
            sheet: String::new(),
            severity: String::new(),
            class: String::new(),
        };
        let inv = GoldenInventory {
            rules: vec![entry("B01")],
            counts: BTreeMap::from([("total".to_string(), 5)]),
            generated_at: None,
            source_commit: None,
        };
        assert!(inv.validate().is_err());
        let mut honest_inv = inv.clone();
        honest_inv.counts.insert("total".to_string(), 1);
        assert!(honest_inv.validate().is_ok());
    }

    #[test]
    fn loader_de_diretorio_com_contagem_honesta() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("inventory.json"),
            r#"{"rules":[{"code":"A1","cadoc":"3040"},{"code":"B2","cadoc":"3040"}],"counts":{"wired":3,"orphan":0,"stub":0,"total":2},"generated_at":"2026-01-01T00:00:00Z","source_commit":"abc"}"#,
        ).unwrap();
        std::fs::write(
            dir.path().join("fixtures_x.json"),
            r#"{"cases":[
                {"id":"c2","testName":"t","ruleCode":"A1","xml":"<x/>","expect":null},
                {"id":"c1","testName":"t","ruleCode":"B2","xml":"<x/>","expect":null}
            ]}"#,
        ).unwrap();
        std::fs::write(
            dir.path().join("fixtures_y.json"),
            r#"{"cases":[{"id":"c0","testName":"t","ruleCode":"B2","xml":"<x/>","expect":null}]}"#,
        ).unwrap();
        // Manifesto sem "cases" — pulado como não-caso.
        std::fs::write(dir.path().join("not_extracted.json"), r#"{"not_extracted":[]}"#).unwrap();

        let inv = load_inventory(dir.path()).unwrap();
        assert_eq!(inv.total(), Some(2));
        let cases = load_cases(dir.path()).unwrap();
        assert_eq!(cases.len(), 3);
        // Ordenado por id: c0, c1, c2.
        assert_eq!(cases[0].id, "c0");
        let honest = honest_counts(&cases);
        assert_eq!(honest.get("B2"), Some(&2));
    }

    #[test]
    fn json_corrompido_erra_tipado() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bad.json"), "nao-json").unwrap();
        assert!(load_cases(dir.path()).is_err());
    }
}
