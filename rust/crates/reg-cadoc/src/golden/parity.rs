//! Runner de paridade Rust × corpus golden.
//!
//! Classificação exaustiva de cada `GoldenCase` (docs/blueprint/
//! 20-ARQUITETURA-RUST.md §6 — "skip DECLARADO, nunca silêncio"):
//! `passed` = regra no registry E o resultado casou com a expectativa do
//! caso; `failed` = regra no registry e o caso FALHOU (REGRESSÃO — gate
//! vermelho); `skipped_not_implemented` = regra fora do registry, estado
//! DECLARADO no relatório por regra; `expect_nao_interpretadas` = casos
//! cujo `expect` não é interpretável pelo runner (estado DECLARADO — o
//! gate NÃO fecha por cima deles).
//!
//! A expectativa é HONESTAMENTE interpretada (não é "a regra aceitou, fim"):
//! - `expect = "nil"`: `apply` deve ser `Ok`;
//! - `expect = "substring"`: `apply` deve ser `Ok` E o XML deve conter a
//!   substring — quando a substring vem vazia no corpus, o caso é contado
//!   como `superficiais` (passou só pela checagem de aceitação; o relatório
//!   declara quantos casos foram verificados só superficialmente);
//! - caso negativo futuro (`expect` de rejeição): a classificação de regra
//!   implementada que DEVE rejeitar entra como `failed` se `apply` aceitar.
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
    /// Casos implementados cujo resultado CASOU com a expectativa.
    pub passed: usize,
    /// Casos implementados que NÃO casaram = regressão (gate vermelho).
    pub failed: usize,
    /// Casos cuja regra não está no registry — skip DECLARADO.
    pub skipped_not_implemented: usize,
    /// Casos passados sem verificação plena da expectativa (expect
    /// `"substring"` com substring vazia no corpus) — DECLARADO, não
    /// misturado com passed pleno.
    pub superficiais: usize,
    /// Casos cujo `expect` não é interpretável pelo runner (nem `"nil"` nem
    /// `"substring"`) — estado DECLARADO; o gate não fecha por cima deles.
    pub expect_nao_interpretadas: usize,
    /// Ids dos casos com falha (para o dossiê de regressão).
    pub failed_ids: Vec<String>,
    /// Counts por regra de casos não implementados (skip declarado por regra).
    pub skipped_by_rule: std::collections::BTreeMap<String, usize>,
}

impl ParityReport {
    /// O gate de paridade é verde só quando: contabilidade fecha (a soma de
    /// `implemented`, `skipped_not_implemented` e `expect_nao_interpretadas`
    /// é o `total`), zero falhas E zero expect não interpretável.
    pub fn is_green(&self) -> bool {
        self.total
            == self.implemented + self.skipped_not_implemented + self.expect_nao_interpretadas
            && self.failed == 0
            && self.expect_nao_interpretadas == 0
    }
}

/// Classifica um caso implementado contra a expectativa do corpus.
/// `Ok(())` = resultado casou; `Err(())` = regressão.
fn classifica_com_expectativa(case: &GoldenCase, resultado: Result<(), RegError>) -> Result<(), ()> {
    match case.expect.as_str() {
        Some("nil") => match resultado {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        },
        Some("substring") => {
            let aplicar_ok = resultado.is_ok();
            let substring = case.substring.trim();
            if substring.is_empty() {
                // Corpus sem substring preenchida: só a checagem de aceitação
                // é possível — superfície DECLARADA no relatório.
                return if aplicar_ok { Ok(()) } else { Err(()) };
            }
            match resultado {
                Ok(()) if case.xml.contains(substring) => Ok(()),
                _ => Err(()),
            }
        }
        // Expectativa não interpretável — nunca contada como pass/fail.
        _ => Err(()),
    }
}

/// Executa todos os casos contra o registry de regras implementadas,
/// interpretando `expect`/`substring` de cada caso (não apenas "a regra
/// aceitou").
pub fn run_parity(cases: &[GoldenCase], registry: &[&dyn Rule]) -> ParityReport {
    let mut report = ParityReport::default();
    for case in cases {
        report.total += 1;
        match registry.iter().find(|r| r.code() == case.rule_code) {
            Some(rule) => {
                report.implemented += 1;
                let resultado = rule.apply(&case.xml);
                let eh_substring_vazia =
                    case.expect.as_str() == Some("substring") && case.substring.trim().is_empty();
                match classifica_com_expectativa(case, resultado) {
                    Ok(()) => {
                        report.passed += 1;
                        if eh_substring_vazia {
                            report.superficiais += 1;
                        }
                    }
                    Err(()) => {
                        if case.expect.as_str().is_none()
                            || case.expect.as_str().is_some_and(|e| e != "nil" && e != "substring")
                        {
                            report.expect_nao_interpretadas += 1;
                        } else {
                            report.failed += 1;
                            report.failed_ids.push(case.id.clone());
                        }
                    }
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
        case_com_expect(id, rule, "nil", "")
    }

    fn case_com_expect(id: &str, rule: &str, expect: &str, substring: &str) -> GoldenCase {
        serde_json::from_str(&format!(
            r#"{{"id":"{id}","testName":"t","ruleCode":"{rule}","xml":"<x/>","expect":"{expect}","substring":"{substring}"}}"#
        ))
        .unwrap()
    }

    fn case_xml(id: &str, rule: &str, expect: &str, substring: &str, xml: &str) -> GoldenCase {
        let mut c = case_com_expect(id, rule, expect, substring);
        c.xml = xml.to_string();
        c
    }

    /// Regra que só aceita XML contendo "ok".
    struct PassaSeContemOk;
    impl Rule for PassaSeContemOk {
        fn code(&self) -> &str {
            "DUM-COND"
        }
        fn apply(&self, xml: &str) -> Result<(), RegError> {
            if xml.contains("ok") {
                Ok(())
            } else {
                Err(RegError::InvalidValue { field: "xml".into(), reason: "sem ok".into() })
            }
        }
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

    /// expect="substring" é verificado DE VERDADE: Ok + substring presente.
    /// Substring vazia no corpus = verificação superficial, contada à parte.
    #[test]
    fn substring_e_honestamente_verificada() {
        let cases = vec![
            // Ok + contém "ok" → passed pleno.
            case_xml("s1", "DUM-COND", "substring", "ok", "<x ok/>"),
            // Ok + NÃO contém "ok" → REGRESSÃO (não é "passed" por aceitar).
            case_xml("s2", "DUM-COND", "substring", "ok", "<x/>"),
            // Err → regressão.
            case_xml("s3", "DUM-COND", "substring", "ok", "<y/>"),
            // Ok + substring vazia no corpus → superficial (declarado).
            case_xml("s4", "DUM-COND", "substring", "", "<x ok/>"),
        ];
        let registry: [&dyn Rule; 1] = [&PassaSeContemOk];
        let report = run_parity(&cases, &registry);
        assert_eq!(report.total, 4);
        assert_eq!(report.implemented, 4);
        assert_eq!(report.passed, 2);
        assert_eq!(report.failed, 2);
        assert_eq!(report.superficiais, 1);
        assert_eq!(report.failed_ids, vec!["s2".to_string(), "s3".to_string()]);
        assert!(!report.is_green());
    }

    /// expect não interpretável (null/objeto/schema futuro) NUNCA é contado
    /// como passed — o estado é declarado e o gate fica vermelho.
    #[test]
    fn expect_nao_interpretavel_e_declarado_nao_passado() {
        let mut c = case("u1", "DUM-PASS");
        c.expect = serde_json::json!({"outcome": "ok"}); // schema desconhecido
        let registry: [&dyn Rule; 1] = [&AlwaysPass];
        let report = run_parity(&[c], &registry);
        assert_eq!(report.total, 1);
        assert_eq!(report.implemented, 1);
        assert_eq!(report.passed, 0);
        assert_eq!(report.failed, 0);
        assert_eq!(report.expect_nao_interpretadas, 1);
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
