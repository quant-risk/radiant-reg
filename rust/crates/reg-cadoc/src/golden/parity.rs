//! Runner de paridade Rust × corpus golden.
//!
//! Classificação exaustiva de cada `GoldenCase` (docs/blueprint/
//! 20-ARQUITETURA-RUST.md §6 — "skip DECLARADO, nunca silêncio"):
//! `passed` = regra no registry e caso passou; `failed` = regra no registry
//! e caso FALHOU (REGRESSÃO — gate vermelho); `skipped_not_implemented` =
//! regra fora do registry, estado DECLARADO no relatório por regra.
//!
//! O registry é um `&[&dyn Rule]` do próprio crate/CLI: uma regra só é
//! "implementada" se estiver lá — ausência é declarada, nunca inferida.

use reg_core::error::RegError;

use super::GoldenCase;

/// Uma regra de validação 3040/2030. O `code` bate com `rule_code` dos
/// casos golden; `apply` valida o XML canônico (usa `RegError` de reg-core —
/// não existe error paralelo nesta camada).
pub trait Rule {
    fn code(&self) -> &str;
    fn apply(&self, xml: &str) -> Result<(), RegError>;
}

/// Relatório de paridade — contagem honesta, sem fração silenciosa.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParityReport {
    /// Total de casos consumidos.
    pub total: usize,
    /// Casos cuja regra está implementada (passed + failed).
    pub implemented: usize,
    /// Casos implementados que passaram.
    pub passed: usize,
    /// Casos implementados que FALHARAM = regressão (gate vermelho).
    pub failed: usize,
    /// Casos cuja regra não está no registry — skip DECLARADO.
    pub skipped_not_implemented: usize,
    /// Ids dos casos com falha (para o dossiê de regressão).
    pub failed_ids: Vec<String>,
    /// Counts por regra de casos não implementados (skip declarado por regra).
    pub skipped_by_rule: std::collections::BTreeMap<String, usize>,
}

impl ParityReport {
    /// O gate de paridade é verde só quando: total accounted + zero falhas.
    pub fn is_green(&self) -> bool {
        self.total == self.implemented + self.skipped_not_implemented
            && self.failed == 0
    }
}

/// Executa todos os casos contra o registry de regras implementadas.
pub fn run_parity(cases: &[GoldenCase], registry: &[&dyn Rule]) -> ParityReport {
    let mut report = ParityReport::default();
    for case in cases {
        report.total += 1;
        match registry.iter().find(|r| r.code() == case.rule_code) {
            Some(rule) => {
                report.implemented += 1;
                if rule.apply(&case.xml).is_ok() {
                    report.passed += 1;
                } else {
                    report.failed += 1;
                    report.failed_ids.push(case.id.clone());
                }
            }
            None => {
                report.skipped_not_implemented += 1;
                *report.skipped_by_rule.entry(case.rule_code.clone()).or_default() += 1;
            }
        }
    }
    report
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Regra-fantasma que sempre passa.
    struct AlwaysPass;
    impl Rule for AlwaysPass {
        fn code(&self) -> &str {
            "DUM-PASS"
        }
        fn apply(&self, _xml: &str) -> Result<(), RegError> {
            Ok(())
        }
    }

    /// Regra-fantasma que sempre falha (tipado via RegError de reg-core).
    struct AlwaysFail;
    impl Rule for AlwaysFail {
        fn code(&self) -> &str {
            "DUM-FAIL"
        }
        fn apply(&self, _xml: &str) -> Result<(), RegError> {
            Err(RegError::InvalidValue { field: "xml".into(), reason: "dummy".into() })
        }
    }

    fn case(id: &str, rule: &str) -> GoldenCase {
        serde_json::from_str(&format!(
            r#"{{"id":"{id}","testName":"t","ruleCode":"{rule}","xml":"<x/>","expect":null}}"#
        ))
        .unwrap()
    }

    #[test]
    fn classificacao_exaustiva_e_honesta() {
        let cases = vec![
            case("p1", "DUM-PASS"),
            case("p2", "DUM-PASS"),
            case("f1", "DUM-FAIL"),
            case("s1", "DUM-NAO-IMPLEMENTADA"),
            case("s2", "DUM-NAO-IMPLEMENTADA"),
        ];
        let registry: [&dyn Rule; 2] = [&AlwaysPass, &AlwaysFail];
        let report = run_parity(&cases, &registry);

        assert_eq!(report.total, 5);
        assert_eq!(report.implemented, 3);
        assert_eq!(report.passed, 2);
        assert_eq!(report.failed, 1);
        assert_eq!(report.skipped_not_implemented, 2);
        assert_eq!(report.failed_ids, vec!["f1".to_string()]);
        assert_eq!(report.skipped_by_rule.get("DUM-NAO-IMPLEMENTADA"), Some(&2));
        // Regressão → vermelho, mesmo com skip declarado.
        assert!(!report.is_green());
    }

    #[test]
    fn tudo_implementado_e_passando_e_verde() {
        let cases = vec![case("p1", "DUM-PASS"), case("p2", "DUM-PASS")];
        let registry: [&dyn Rule; 1] = [&AlwaysPass];
        let report = run_parity(&cases, &registry);
        assert!(report.is_green());
        assert_eq!(report.skipped_not_implemented, 0);
    }

    #[test]
    fn registry_vazio_declara_todo_o_skip() {
        let cases = vec![case("p1", "DUM-PASS")];
        let report = run_parity(&cases, &[]);
        assert_eq!(report.total, 1);
        assert_eq!(report.skipped_not_implemented, 1);
        assert!(report.is_green()); // skip declarado não é regressão
    }
}
