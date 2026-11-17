//! CNPJ — identificador **temporal** conforme IN BCB 729/2026.
//!
//! A Receita Federal emite CNPJ **alfanumérico** `[0-9A-Z]{8|14}` para novas
//! inscrições desde jul/2026; leiautes BCB anteriores esperavam `[0-9]{8|14}`.
//! O v1 fixava `Cnpj8(u32)` numérico — bug-em-espera. Aqui o formato é
//! selecionado pela **data-base** da competência, nunca por heurística.

use serde::{Deserialize, Serialize};

use crate::error::RegError;
use crate::period::DataBase;

/// Formato aceito para uma data-base, fixado pela IN BCB 729/2026.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CnpjFormat {
    /// Competências anteriores a jul/2026: apenas dígitos `[0-9]{8|14}`.
    Numerico,
    /// Competências de jul/2026 em diante: `[0-9A-Z]{8|14}` (base 36, RFB).
    Alfanumerico,
}

/// CNPJ da IF remetente ou do cliente — 8 (raiz) ou 14 (completo) caracteres.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CnpjIdentifier(String);

impl CnpjIdentifier {
    /// Valida contra o formato vigente na data-base (IN BCB 729/2026).
    pub fn new(raw: &str, data_base: DataBase) -> Result<Self, RegError> {
        let s: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
        let formato = formato_para(data_base);
        let len = s.len();
        if len != 8 && len != 14 {
            return Err(RegError::InvalidValue {
                field: "cnpj".into(),
                reason: format!("esperado 8 ou 14 caracteres, recebido {len}"),
            });
        }
        let valido = match formato {
            CnpjFormat::Numerico => s.chars().all(|c| c.is_ascii_digit()),
            CnpjFormat::Alfanumerico => s
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase()),
        };
        if !valido {
            return Err(RegError::InvalidValue {
                field: "cnpj".into(),
                reason: format!("caractere inválido para formato {formato:?} (IN BCB 729/2026) em {raw:?}"),
            });
        }
        Ok(CnpjIdentifier(s))
    }

    /// Raiz (8 primeiros caracteres).
    pub fn raiz(&self) -> &str {
        &self.0[..8]
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_completo(&self) -> bool {
        self.0.len() == 14
    }
}

impl std::fmt::Display for CnpjIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Corte temporal oficial: a RFB começou a emitir alfanuméricos em jul/2026;
/// a IN BCB 729/2026 alinhou os leiautes BCB à mudança.
#[must_use]
pub const fn formato_para(data_base: DataBase) -> CnpjFormat {
    if data_base.ano > 2026 || (data_base.ano == 2026 && data_base.mes >= 7) {
        CnpjFormat::Alfanumerico
    } else {
        CnpjFormat::Numerico
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;

    fn db(ano: u16, mes: u8) -> DataBase {
        DataBase::new(ano, mes).unwrap()
    }

    #[test]
    fn competencia_antiga_exige_numerico() {
        assert!(CnpjIdentifier::new("12345678", db(2026, 6)).is_ok());
        assert!(CnpjIdentifier::new("123456780001AB", db(2026, 6)).is_err()); // alfa ainda não existe
        assert!(CnpjIdentifier::new("12345678000190", db(2026, 6)).is_ok());
    }

    #[test]
    fn competencia_julho_2026_aceita_alfanumerico() {
        assert!(CnpjIdentifier::new("12ABC34501DE56", db(2026, 7)).is_ok());
        assert!(CnpjIdentifier::new("12345678000190", db(2026, 7)).is_ok()); // numérico segue válido
        assert!(CnpjIdentifier::new("12aBC345", db(2026, 7)).is_err()); // lowercase rejeitado
    }

    #[test]
    fn tamanho_errado_rejeitado() {
        assert!(CnpjIdentifier::new("1234567", db(2026, 7)).is_err());
        assert!(CnpjIdentifier::new("123456780001901", db(2026, 7)).is_err());
    }

    #[test]
    fn raiz_eh_os_oito_primeiros() {
        let c = CnpjIdentifier::new("12345678000190", db(2026, 7)).unwrap();
        assert_eq!(c.raiz(), "12345678");
        assert!(c.is_completo());
    }
}
