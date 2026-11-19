//! IPOC — Identificador da Operação de Crédito (Doc 3040).
//!
//! Regra I14 do catálogo: 15 a 67 caracteres, alfanumérico, **sem** o campo
//! OrigemRec (que era emitido no v1 e invalida o identificador).

use serde::{Deserialize, Serialize};

use crate::error::RegError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ipoc(String);

impl Ipoc {
    pub fn new(raw: &str) -> Result<Self, RegError> {
        let s = raw.trim();
        let len = s.chars().count();
        if !(15..=67).contains(&len) {
            return Err(RegError::InvalidValue {
                field: "ipoc".into(),
                reason: format!("esperado 15..=67 caracteres, recebido {len} (regra I14)"),
            });
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase() || c == '-')
        {
            return Err(RegError::InvalidValue {
                field: "ipoc".into(),
                reason: "caractere fora de [0-9A-Z-] (regra I14)".into(),
            });
        }
        Ok(Ipoc(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;

    #[test]
    fn aceita_15_a_67() {
        assert!(Ipoc::new("123456789012345").is_ok()); // 15
        assert!(Ipoc::new(&"A".repeat(67)).is_ok()); // 67
        assert!(Ipoc::new("12345678901234").is_err()); // 14
        assert!(Ipoc::new(&"A".repeat(68)).is_err()); // 68
    }

    #[test]
    fn rejeita_nao_alfanumerico() {
        assert!(Ipoc::new("1234567890123á5").is_err());
        assert!(Ipoc::new("123456789012 45").is_err());
    }
}
