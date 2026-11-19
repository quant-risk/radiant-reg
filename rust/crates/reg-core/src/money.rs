//! Dinheiro — `Centavos(i128)`, zero float (doutrina: 20-ARQUITETURA §4.5).
//!
//! Motivação: o gen2030 do v1 acumulava concentrações em `float64`
//! (`generator.go:133-166`). Aqui não existe ponto flutuante: toda entrada
//! textual é parseada primeiro como texto (política texto-primeiro), depois
//! convertida em centavos com aritmética checked. Overflow é erro, nunca wrap.
//!
//! ## Contrato do parser (comportamento DEFINIDO, testado — nunca heurística muda)
//! - Sinal: `-` à esquerda é suportado (estornos/retificações redutoras);
//!   parênteses contábeis `(100,00)` são **rejeitados** com erro explícito
//!   (declare sinal, não convenção tipográfica).
//! - `Reais`: vírgula é sempre decimal (formato BR); ponto único é decimal
//!   (formato US) **exceto** quando seguido de exatamente 3 dígitos em um
//!   número maior que 4 caracteres — aí é separador de milhar BR
//!   ("1.234" = R$1.234,00; "1234.56" = R$1.234,56). Valores com 3 casas
//!   decimais não existem em BRL, por isso a heurística é segura e PINADA
//!   em testes. Múltiplos pontos = milhar ("1.234.567").
//! - `Centavos`: inteiro puro — qualquer separador é ERRO (célula formatada
//!   "1.234" nunca vira centavos errado silenciosamente).
//! - `MilReais`: aceita decimais ("1,5" = R$1.500,00) — escala ×1.000 após
//!   o parse em reais.
//! - Decimais com >2 casas: arredondamento half-up em UMA operação
//!   (sem double rounding cascata).

use serde::{Deserialize, Serialize};

use crate::error::RegError;

/// Valor monetário em centavos de BRL. Sempre inteiro exato.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Centavos(i128);

/// Unidade de entrada declarada no mapping (contrato tipado do mapping.lock).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonetaryUnit {
    /// "1.234,56" / "1234.56" / "123456" — reais com centavos.
    Reais,
    /// "123456" já em centavos — inteiro puro, separadores rejeitados.
    Centavos,
    /// Planilhas BACEN que informam "mil reais" (×1.000); aceita decimais.
    MilReais,
}

impl Centavos {
    pub const ZERO: Centavos = Centavos(0);

    /// Construtor canônico: texto primeiro, depois conversão exata.
    pub fn parse(raw: &str, unit: MonetaryUnit) -> Result<Centavos, RegError> {
        const FIELD: &str = "valor monetário";
        let invalid = |reason: String| RegError::InvalidValue {
            field: FIELD.into(),
            reason,
        };
        let cleaned: String = raw
            .chars()
            .filter(|c| !matches!(c, ' ' | '\u{a0}' | '\t'))
            .collect();
        if cleaned.is_empty() {
            return Err(invalid(format!("nenhum dígito em {raw:?}")));
        }
        if cleaned.starts_with('(') {
            return Err(invalid(
                "parênteses contábeis não suportados: declare o sinal com '-'".into(),
            ));
        }
        let (negativo, body) = match cleaned.strip_prefix('-') {
            Some(b) => (true, b.to_string()),
            None => (false, cleaned.clone()),
        };
        let mut cents: i128 = match unit {
            MonetaryUnit::Centavos => {
                if !body.chars().all(|c| c.is_ascii_digit()) {
                    return Err(invalid(format!(
                        "unit=centavos exige inteiro puro sem separadores: {raw:?}"
                    )));
                }
                body.parse::<i128>()
                    .map_err(|_| invalid(format!("magnitude fora de i128 em {raw:?}")))?
            }
            MonetaryUnit::Reais | MonetaryUnit::MilReais => {
                let (int_part, frac_part) = split_decimal(&body).ok_or_else(|| {
                    invalid(format!("separador decimal ambíguo ou ausente em {raw:?}"))
                })?;
                let non_digit = int_part
                    .chars()
                    .any(|c| !c.is_ascii_digit() && c != '.' && c != ',');
                if non_digit {
                    return Err(invalid(format!("caractere não numérico em {raw:?}")));
                }
                let digits_int: String = int_part.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits_int.is_empty() {
                    return Err(invalid(format!("nenhum dígito em {raw:?}")));
                }
                let mut c: i128 = digits_int
                    .parse::<i128>()
                    .map_err(|_| invalid(format!("magnitude fora de i128 em {raw:?}")))?
                    .checked_mul(100)
                    .ok_or_else(|| invalid("overflow ao converter reais".into()))?;
                if !frac_part.is_empty() {
                    if !frac_part.chars().all(|c| c.is_ascii_digit()) {
                        return Err(invalid(format!("decimais inválidos em {raw:?}")));
                    }
                    let mut f: i128 = frac_part
                        .parse()
                        .map_err(|_| invalid(format!("decimais inválidos em {raw:?}")))?;
                    let extra = frac_part.len().saturating_sub(2);
                    if extra > 0 {
                        // half-up em UMA operação (sem cascata de arredondamentos)
                        let d = i128::checked_pow(10, extra as u32)
                            .ok_or_else(|| invalid("decimais longos demais".into()))?;
                        f = (f + d / 2) / d;
                    } else if frac_part.len() == 1 {
                        f *= 10;
                    }
                    c = c
                        .checked_add(f)
                        .ok_or_else(|| invalid("overflow ao somar decimais".into()))?;
                }
                if unit == MonetaryUnit::MilReais {
                    c = c
                        .checked_mul(1_000)
                        .ok_or_else(|| invalid("overflow ao converter mil-reais".into()))?;
                }
                c
            }
        };
        if negativo {
            cents = -cents;
        }
        Ok(Centavos(cents))
    }

    pub const fn as_i128(self) -> i128 {
        self.0
    }

    pub fn checked_add(self, rhs: Centavos) -> Result<Centavos, RegError> {
        self.0
            .checked_add(rhs.0)
            .map(Centavos)
            .ok_or(RegError::InvalidValue {
                field: "soma monetária".into(),
                reason: "overflow i128".into(),
            })
    }

    pub fn checked_sub(self, rhs: Centavos) -> Result<Centavos, RegError> {
        self.0
            .checked_sub(rhs.0)
            .map(Centavos)
            .ok_or(RegError::InvalidValue {
                field: "subtração monetária".into(),
                reason: "overflow i128".into(),
            })
    }

    /// Multiplicação por percentual em basis points (1 bp = 0,01%).
    /// Arredondamento half-away-from-zero explícito — nunca implícito
    /// (o leiaute declara a estratégia; aqui é a default de cálculo interno).
    pub fn mul_basis_points(self, bp: i64) -> Result<Centavos, RegError> {
        let bp = i128::from(bp);
        let raw = self.0.checked_mul(bp).ok_or(RegError::InvalidValue {
            field: "multiplicação por percentual".into(),
            reason: "overflow i128".into(),
        })?;
        let (q, r) = (raw / 10_000, raw % 10_000); // sinal de r acompanha raw
        let bump: i128 = if r.abs() >= 5_000 {
            if raw < 0 { -1 } else { 1 }
        } else {
            0
        };
        Ok(Centavos(q + bump))
    }
}

/// Divide o corpo (sem sinal) em (inteiro, decimais) sob o contrato documentado.
fn split_decimal(cleaned: &str) -> Option<(String, String)> {
    let comma = cleaned.rfind(',');
    let dot = cleaned.rfind('.');
    match (comma, dot) {
        // vírgula presente: sempre decimal BR; pontos são milhar
        (Some(c), _) => Some((cleaned[..c].to_string(), cleaned[c + 1..].to_string())),
        (None, Some(d)) => {
            let after = &cleaned[d + 1..];
            if cleaned.matches('.').count() > 1 {
                // "1.234.567" — milhar múltiplo, sem decimais
                return Some((cleaned.replace('.', ""), String::new()));
            }
            // ponto único: milhar BR se +3 dígitos exatos em número grande;
            // senão decimal US. Comportamento PINADO em testes (doc do módulo).
            if after.len() == 3 && after.chars().all(|c| c.is_ascii_digit()) && cleaned.len() > 4 {
                Some((cleaned.replace('.', ""), String::new()))
            } else {
                Some((cleaned[..d].to_string(), after.to_string()))
            }
        }
        (None, None) => Some((cleaned.to_string(), String::new())),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;

    #[test]
    fn parse_reais_br_e_us() {
        assert_eq!(
            Centavos::parse("1.234,56", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            123_456
        );
        assert_eq!(
            Centavos::parse("1234.56", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            123_456
        );
        assert_eq!(
            Centavos::parse("123456", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            12_345_600
        );
        assert_eq!(
            Centavos::parse("1.234", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            123_400
        );
        assert_eq!(
            Centavos::parse("0,05", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            5
        );
    }

    /// Contrato PINADO (doc do módulo): ponto único + 3 dígitos = milhar BR.
    /// "0.500" é milhar (R$500), não R$0,50 — BRL não tem 3 casas decimais.
    #[test]
    fn heuristica_milhar_br_pinada() {
        assert_eq!(
            Centavos::parse("0.500", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            50_000
        );
        assert_eq!(
            Centavos::parse("12.340", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            1_234_000
        );
        assert_eq!(
            Centavos::parse("1.234.567,89", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            123_456_789
        );
        assert_eq!(
            Centavos::parse("1.234.567", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            123_456_700
        );
    }

    #[test]
    fn parse_centavos_e_mil_reais() {
        assert_eq!(
            Centavos::parse("123456", MonetaryUnit::Centavos)
                .unwrap()
                .as_i128(),
            123_456
        );
        assert_eq!(
            Centavos::parse("2", MonetaryUnit::MilReais)
                .unwrap()
                .as_i128(),
            200_000
        );
        // MilReais COM decimais (achado do revisor: antes descartava "1,5"→1 mil)
        assert_eq!(
            Centavos::parse("1,5", MonetaryUnit::MilReais)
                .unwrap()
                .as_i128(),
            150_000
        );
        assert_eq!(
            Centavos::parse("1.250,75", MonetaryUnit::MilReais)
                .unwrap()
                .as_i128(),
            125_075_000
        );
    }

    #[test]
    fn centavos_rejeita_separadores() {
        assert!(Centavos::parse("12,5", MonetaryUnit::Centavos).is_err());
        assert!(Centavos::parse("1.234", MonetaryUnit::Centavos).is_err());
    }

    #[test]
    fn negativos_suportados_com_sinal() {
        assert_eq!(
            Centavos::parse("-100,00", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            -10_000
        );
        assert_eq!(
            Centavos::parse("-123456", MonetaryUnit::Centavos)
                .unwrap()
                .as_i128(),
            -123_456
        );
        assert_eq!(
            Centavos::parse("-1,5", MonetaryUnit::MilReais)
                .unwrap()
                .as_i128(),
            -150_000
        );
    }

    #[test]
    fn parenteses_contabeis_rejeitados_explicito() {
        let e = Centavos::parse("(100,00)", MonetaryUnit::Reais).unwrap_err();
        assert!(e.to_string().contains("parênteses"), "{e}");
    }

    #[test]
    fn rejeita_moeda_e_lixo() {
        assert!(Centavos::parse("R$ 1.234,56", MonetaryUnit::Reais).is_err());
        assert!(Centavos::parse("abc", MonetaryUnit::Reais).is_err());
        assert!(Centavos::parse("", MonetaryUnit::Reais).is_err());
        assert!(Centavos::parse("-", MonetaryUnit::Reais).is_err());
    }

    #[test]
    fn checked_overflow_eh_erro_nao_wrap() {
        let max = Centavos(i128::MAX);
        assert!(max.checked_add(Centavos(1)).is_err());
    }

    #[test]
    fn basis_points_half_away_from_zero() {
        assert_eq!(
            Centavos(1_000).mul_basis_points(10_000).unwrap().as_i128(),
            1_000
        ); // 100%
        assert_eq!(Centavos(100).mul_basis_points(5_000).unwrap().as_i128(), 50); // 50%
        assert_eq!(Centavos(1).mul_basis_points(15_000).unwrap().as_i128(), 2); // 1,5 → 2
        assert_eq!(
            Centavos(105).mul_basis_points(10_000).unwrap().as_i128(),
            105
        );
        // ramos negativos (achado do revisor: cobertura ausente)
        assert_eq!(Centavos(-1).mul_basis_points(15_000).unwrap().as_i128(), -2);
        assert_eq!(
            Centavos(-1_000)
                .mul_basis_points(-10_000)
                .unwrap()
                .as_i128(),
            1_000
        );
        assert_eq!(
            Centavos(-150).mul_basis_points(5_000).unwrap().as_i128(),
            -75
        );
    }

    #[test]
    fn decimais_3_casas_half_up_unico() {
        // "0,999" → 99,9 centavos → 100 (uma operação, sem cascata)
        assert_eq!(
            Centavos::parse("0,999", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            100
        );
        assert_eq!(
            Centavos::parse("0,994", MonetaryUnit::Reais)
                .unwrap()
                .as_i128(),
            99
        );
    }

    #[test]
    fn conservacao_contabil_zero_centavos() {
        let partes = ["100,00", "250,50", "0,99"];
        let soma = partes
            .iter()
            .try_fold(Centavos::ZERO, |acc, p| {
                acc.checked_add(Centavos::parse(p, MonetaryUnit::Reais)?)
            })
            .unwrap();
        assert_eq!(soma.as_i128(), 35_149);
    }

    /// Roundtrip serde: Centavos serializa como inteiro i128 (audit trail estável).
    #[test]
    fn serde_roundtrip() {
        let casos = [123_i128, -123, i128::MAX, i128::MIN + 1, 0];
        for v in casos {
            let c = Centavos(v);
            let json = serde_json::to_string(&c).unwrap();
            let de: Centavos = serde_json::from_str(&json).unwrap();
            assert_eq!(de, c, "roundtrip falhou para {v}");
        }
    }
}
