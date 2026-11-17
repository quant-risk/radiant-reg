//! Dinheiro — `Centavos(i128)`, zero float (doutrina: 20-ARQUITETURA §4.5).
//!
//! Motivação: o gen2030 do v1 acumulava concentrações em `float64`
//! (`generator.go:133-166`). Aqui não existe ponto flutuante: toda entrada
//! textual é parseada primeiro como texto (política texto-primeiro), depois
//! convertida em centavos com aritmética checked. Overflow é erro, nunca wrap.

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
    /// "123456" já em centavos.
    Centavos,
    /// Planilhas BACEN que informam "mil reais" (multiplica por 1.000).
    MilReais,
}

impl Centavos {
    pub const ZERO: Centavos = Centavos(0);

    /// Construtor canônico: texto primeiro, depois conversão exata.
    ///
    /// Aceita os formatos BR ("1.234,56") e US ("1234.56") quando `Reais`;
    /// rejeita qualquer dígito não-numérico (moeda, parênteses, espaços).
    pub fn parse(raw: &str, unit: MonetaryUnit) -> Result<Centavos, RegError> {
        let cleaned: String = raw
            .chars()
            .filter(|c| !matches!(c, ' ' | '\u{a0}' | '\t'))
            .collect();
        let (int_part, frac_part): (String, String) = match unit {
            MonetaryUnit::Centavos => (cleaned.clone(), String::new()),
            MonetaryUnit::Reais | MonetaryUnit::MilReais => {
                split_decimal(&cleaned).ok_or_else(|| RegError::InvalidValue {
                    field: "valor monetário".into(),
                    reason: format!("separador decimal ambíguo ou ausente em {raw:?}"),
                })?
            }
        };
        let digits_int: String = int_part.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits_int.is_empty() {
            return Err(RegError::InvalidValue {
                field: "valor monetário".into(),
                reason: format!("nenhum dígito em {raw:?}"),
            });
        }
        // Rejeita separadores de milhar malformados ("1.23.456,78" já teria
        // sido rejeitado em split_decimal; aqui rejeita não-dígitos residual).
        let non_digit = int_part
            .chars()
            .any(|c| !c.is_ascii_digit() && c != '.' && c != ',');
        if non_digit || frac_part.chars().any(|c| !c.is_ascii_digit()) {
            return Err(RegError::InvalidValue {
                field: "valor monetário".into(),
                reason: format!("caractere não numérico em {raw:?}"),
            });
        }
        let int_val: i128 = digits_int
            .parse()
            .map_err(|_| RegError::InvalidValue {
                field: "valor monetário".into(),
                reason: format!("magnitude fora de i128 em {raw:?}"),
            })?;
        let mut cents = match unit {
            MonetaryUnit::Centavos => int_val,
            MonetaryUnit::MilReais => int_val
                .checked_mul(100_000)
                .ok_or(RegError::InvalidValue {
                    field: "valor monetário".into(),
                    reason: "overflow ao converter mil-reais".into(),
                })?,
            MonetaryUnit::Reais => int_val
                .checked_mul(100)
                .ok_or(RegError::InvalidValue {
                    field: "valor monetário".into(),
                    reason: "overflow ao converter reais".into(),
                })?,
        };
        // Parte decimal (apenas Reais): arredondamento HALF-UP para >2 dígitos.
        if unit == MonetaryUnit::Reais && !frac_part.is_empty() {
            let mut f: i128 = frac_part.parse().map_err(|_| RegError::InvalidValue {
                field: "valor monetário".into(),
                reason: format!("decimais inválidos em {raw:?}"),
            })?;
            let extra = frac_part.len().saturating_sub(2);
            if extra > 0 {
                for _ in 0..extra {
                    let (q, r) = (f / 10, f % 10);
                    if r >= 5 {
                        f = q + 1;
                    } else {
                        f = q;
                    }
                }
            } else {
                f *= 10_i128.pow(2 - frac_part.len() as u32);
            }
            cents = cents.checked_add(f).ok_or(RegError::InvalidValue {
                field: "valor monetário".into(),
                reason: "overflow ao somar decimais".into(),
            })?;
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
        let raw = self
            .0
            .checked_mul(bp)
            .ok_or(RegError::InvalidValue {
                field: "multiplicação por percentual".into(),
                reason: "overflow i128".into(),
            })?;
        let (q, r) = (raw / 10_000, raw % 10_000); // sinal de r acompanha raw
        let bump: i128 = if r.abs() >= 5_000 {
            if raw < 0 {
                -1
            } else {
                1
            }
        } else {
            0
        };
        Ok(Centavos(q + bump))
    }
}

/// Divide "1.234,56" ou "1234.56" em (inteiro, decimais).
/// Regra: se há vírgula, ela é o separador decimal (formato BR) e pontos são
/// milhar; se só há ponto, é decimal (formato US); se não há separador, é inteiro.
fn split_decimal(cleaned: &str) -> Option<(String, String)> {
    let comma = cleaned.rfind(',');
    let dot = cleaned.rfind('.');
    match (comma, dot) {
        (Some(c), Some(d)) if c > d => Some((cleaned[..c].to_string(), cleaned[c + 1..].to_string())),
        (Some(_), Some(d)) => Some((cleaned[..d].to_string(), cleaned[d + 1..].to_string())),
        (Some(c), None) => Some((cleaned[..c].to_string(), cleaned[c + 1..].to_string())),
        (None, Some(d)) => {
            // ponto pode ser decimal OU milhar ("1.234"): é milhar se o grupo
            // após o ponto tem exatamente 3 dígitos e não há outro separador depois
            let after = &cleaned[d + 1..];
            if after.len() == 3 && after.chars().all(|c| c.is_ascii_digit()) && cleaned.matches('.').count() == 1 && cleaned.len() > 4 {
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
        assert_eq!(Centavos::parse("1.234,56", MonetaryUnit::Reais).unwrap().as_i128(), 123_456);
        assert_eq!(Centavos::parse("1234.56", MonetaryUnit::Reais).unwrap().as_i128(), 123_456);
        assert_eq!(Centavos::parse("123456", MonetaryUnit::Reais).unwrap().as_i128(), 12_345_600);
        assert_eq!(Centavos::parse("1.234", MonetaryUnit::Reais).unwrap().as_i128(), 123_400);
        assert_eq!(Centavos::parse("0,05", MonetaryUnit::Reais).unwrap().as_i128(), 5);
    }

    #[test]
    fn parse_centavos_e_mil_reais() {
        assert_eq!(Centavos::parse("123456", MonetaryUnit::Centavos).unwrap().as_i128(), 123_456);
        assert_eq!(Centavos::parse("2", MonetaryUnit::MilReais).unwrap().as_i128(), 200_000);
    }

    #[test]
    fn rejeita_moeda_e_lixo() {
        assert!(Centavos::parse("R$ 1.234,56", MonetaryUnit::Reais).is_err());
        assert!(Centavos::parse("abc", MonetaryUnit::Reais).is_err());
        assert!(Centavos::parse("", MonetaryUnit::Reais).is_err());
    }

    #[test]
    fn checked_overflow_eh_erro_nao_wrap() {
        let max = Centavos(i128::MAX);
        assert!(max.checked_add(Centavos(1)).is_err());
    }

    #[test]
    fn basis_points_half_up() {
        assert_eq!(Centavos(1_000).mul_basis_points(10_000).unwrap().as_i128(), 1_000); // 100%
        assert_eq!(Centavos(100).mul_basis_points(5_000).unwrap().as_i128(), 50); // 50%
        assert_eq!(Centavos(1).mul_basis_points(15_000).unwrap().as_i128(), 2); // 1,5 → 2 (half-up)
        assert_eq!(Centavos(105).mul_basis_points(10_000).unwrap().as_i128(), 105);
    }

    #[test]
    fn conservacao_contabil_zero_centavos() {
        // TravA 2 do mapping.lock: Σ partes == total, exato.
        let partes = ["100,00", "250,50", "0,99"];
        let soma = partes
            .iter()
            .try_fold(Centavos::ZERO, |acc, p| {
                acc.checked_add(Centavos::parse(p, MonetaryUnit::Reais)?)
            })
            .unwrap();
        assert_eq!(soma.as_i128(), 35_149);
    }
}
