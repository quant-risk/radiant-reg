//! Tempo regulatório — `DataBase` (AAAA-MM) e `Vigencia` `[from, to)`.
//!
//! O v1 misturava `2006-01-02` no generator com `YYYY-MM` no parser
//! (bug documentado na spec de port §1.8.4). Aqui a data-base é sempre
//! AAAA-MM — o leiaute BCB declara competência mensal/semestral.

use serde::{Deserialize, Serialize};

use crate::error::RegError;

/// Data-base de competência — ano + mês (ex.: "2026-12" para o DRSAC da IN 694/2025).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DataBase {
    pub ano: u16,
    pub mes: u8,
}

impl DataBase {
    pub fn new(ano: u16, mes: u8) -> Result<Self, RegError> {
        if !(1..=12).contains(&mes) {
            return Err(RegError::InvalidValue {
                field: "data-base".into(),
                reason: format!("mês {mes} fora de 1..=12"),
            });
        }
        if !(1990..=2100).contains(&ano) {
            return Err(RegError::InvalidValue {
                field: "data-base".into(),
                reason: format!("ano {ano} fora do intervalo suportado"),
            });
        }
        Ok(DataBase { ano, mes })
    }

    /// Parse estrito de "AAAA-MM" — o formato canônico dos atributos `dataBase`.
    /// Zero-padding obrigatório: "2026-1" é rejeitado (parse e Display são
    /// simétricos).
    pub fn parse(s: &str) -> Result<Self, RegError> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2
            || parts[0].len() != 4
            || parts[1].len() != 2
            || !parts[0].chars().all(|c| c.is_ascii_digit())
            || !parts[1].chars().all(|c| c.is_ascii_digit())
        {
            return Err(RegError::InvalidValue {
                field: "data-base".into(),
                reason: format!("esperado AAAA-MM, recebido {s:?}"),
            });
        }
        let ano: u16 = parts[0].parse().map_err(|_| RegError::InvalidValue {
            field: "data-base".into(),
            reason: format!("ano não numérico em {s:?}"),
        })?;
        let mes: u8 = parts[1].parse().map_err(|_| RegError::InvalidValue {
            field: "data-base".into(),
            reason: format!("mês não numérico em {s:?}"),
        })?;
        DataBase::new(ano, mes)
    }
}

impl std::fmt::Display for DataBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}", self.ano, self.mes)
    }
}

/// Intervalo de vigência de um leiaute/regra: `[from, to)` — `to = None` é
/// "vigente indefinidamente". Resolve a versão vigente por data-base
/// (resolve(): a versão com maior `from <= db` vence; a anterior fica para
/// re-validação histórica, padrão schema_versions do v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vigencia {
    pub from: DataBase,
    pub to: Option<DataBase>,
}

impl Vigencia {
    pub fn desde(from: DataBase) -> Self {
        Vigencia { from, to: None }
    }

    /// Vigência fechada `[from, to)` — rejeita intervalo invertido/vazio
    /// (configuração errada nunca cria uma vigência que rejeita tudo
    /// silenciosamente).
    pub fn entre(from: DataBase, to: DataBase) -> Result<Self, RegError> {
        if from >= to {
            return Err(RegError::InvalidValue {
                field: "vigência".into(),
                reason: format!("intervalo invertido ou vazio: [{from}, {to})"),
            });
        }
        Ok(Vigencia { from, to: Some(to) })
    }

    /// A data-base está dentro da vigência?
    pub fn contem(&self, db: DataBase) -> bool {
        if db < self.from {
            return false;
        }
        match self.to {
            Some(to) => db < to,
            None => true,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;

    fn db(s: &str) -> DataBase {
        DataBase::parse(s).unwrap()
    }

    #[test]
    fn parse_estrito_aaaa_mm() {
        assert_eq!(db("2026-12"), DataBase::new(2026, 12).unwrap());
        assert!(DataBase::parse("2026-12-15").is_err()); // o bug do v1 (2006-01-02)
        assert!(DataBase::parse("2026/12").is_err());
        assert!(DataBase::parse("2026-13").is_err());
    }

    #[test]
    fn vigencia_intervalo_semiaberto() {
        let v = Vigencia::entre(db("2026-07"), db("2026-12")).unwrap();
        assert!(!v.contem(db("2026-06")));
        assert!(v.contem(db("2026-07"))); // from incluso
        assert!(v.contem(db("2026-11")));
        assert!(!v.contem(db("2026-12"))); // to exclusivo
    }

    /// Achado do revisor: intervalo invertido é erro, não vigência que
    /// rejeita tudo silenciosamente.
    #[test]
    fn vigencia_invertida_eh_erro() {
        assert!(Vigencia::entre(db("2026-12"), db("2026-07")).is_err());
        assert!(Vigencia::entre(db("2026-07"), db("2026-07")).is_err());
    }

    /// Achado do revisor: parse exige zero-padding (simétrico com Display).
    #[test]
    fn parse_exige_zero_padding() {
        assert!(DataBase::parse("2026-1").is_err());
        assert!(DataBase::parse("26-01").is_err());
        assert!(DataBase::parse("2026-+1").is_err());
        assert!(DataBase::parse("2026-01").is_ok());
    }

    #[test]
    fn vigencia_aberta() {
        let v = Vigencia::desde(db("2026-12"));
        assert!(v.contem(db("2030-01")));
        assert!(!v.contem(db("2026-11")));
    }

    #[test]
    fn ordenacao_cronologica() {
        assert!(db("2026-06") < db("2026-07"));
        assert!(db("2025-12") < db("2026-01"));
    }
}
