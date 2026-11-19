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
                reason: format!(
                    "caractere inválido para formato {formato:?} (IN BCB 729/2026) em {raw:?}"
                ),
            });
        }
        // DV módulo-11 verificado para o formato numérico completo. O DV do
        // formato ALFANUMÉRICO (base 36) depende do algoritmo oficial da
        // IN BCB 729/2026 — pendência DECLARADA via `dv_pendente()`, nunca
        // skip silencioso.
        if formato == CnpjFormat::Numerico && len == 14 && !dv_numerico_valido(&s) {
            return Err(RegError::InvalidValue {
                field: "cnpj".into(),
                reason: format!("dígito verificador inválido para {raw:?} (módulo 11)"),
            });
        }
        Ok(CnpjIdentifier(s))
    }

    /// O DV deste identificador ainda não é verificável? (formato alfanumérico
    /// completo — algoritmo oficial da IN 729/2026 pendente de confirmação).
    pub fn dv_pendente(&self, data_base: DataBase) -> bool {
        formato_para(data_base) == CnpjFormat::Alfanumerico && self.is_completo()
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

/// DV módulo-11 do CNPJ numérico completo (14 dígitos): pesos 5..2 na 1ª
/// rodada e 6..2 na 2ª (algoritmo clássico da Receita Federal).
fn dv_numerico_valido(s: &str) -> bool {
    let digits: Vec<u8> = s
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect();
    if digits.len() != 14 {
        return false;
    }
    let (body, dv) = digits.split_at(12);
    let calc = |weights: &[u8; 12]| -> u8 {
        let sum: usize = body
            .iter()
            .zip(weights.iter())
            .map(|(d, w)| usize::from(*d) * usize::from(*w))
            .sum();
        let rest = sum % 11;
        if rest < 2 { 0 } else { (11 - rest) as u8 }
    };
    let d1 = calc(&[5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]);
    if dv[0] != d1 {
        return false;
    }
    let mut with_d1: Vec<u8> = body.to_vec();
    with_d1.push(d1);
    let weights2 = [6usize, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum: usize = with_d1
        .iter()
        .zip(weights2.iter())
        .map(|(d, w)| usize::from(*d) * *w)
        .sum();
    let rest = sum % 11;
    let d2 = if rest < 2 { 0 } else { (11 - rest) as u8 };
    dv[1] == d2
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
        assert!(CnpjIdentifier::new("11222333000181", db(2026, 6)).is_ok()); // DV válido
    }

    #[test]
    fn competencia_julho_2026_aceita_alfanumerico() {
        assert!(CnpjIdentifier::new("12ABC34501DE56", db(2026, 7)).is_ok());
        assert!(CnpjIdentifier::new("11222333000181", db(2026, 7)).is_ok()); // numérico segue válido
        assert!(CnpjIdentifier::new("12aBC345", db(2026, 7)).is_err()); // lowercase rejeitado
    }

    #[test]
    fn dv_modulo11_verificado_no_numerico() {
        // 11.222.333/0001-81 — par clássico válido da RFB
        assert!(CnpjIdentifier::new("11222333000181", db(2025, 1)).is_ok());
        // mesmos 12 dígitos com DV trocado → erro com "dígito verificador"
        let e = CnpjIdentifier::new("11222333000182", db(2025, 1)).unwrap_err();
        assert!(e.to_string().contains("dígito verificador"), "{e}");
    }

    #[test]
    fn dv_alfanumerico_eh_pendencia_declarada() {
        let c = CnpjIdentifier::new("12ABC34501DE56", db(2026, 7)).unwrap();
        assert!(c.dv_pendente(db(2026, 7)));
        let n = CnpjIdentifier::new("11222333000181", db(2025, 1)).unwrap();
        assert!(!n.dv_pendente(db(2025, 1)));
    }

    /// Bordas do corte temporal (achado do revisor: só testado indiretamente).
    #[test]
    fn formato_para_bordas() {
        assert_eq!(formato_para(db(2026, 6)), CnpjFormat::Numerico);
        assert_eq!(formato_para(db(2026, 7)), CnpjFormat::Alfanumerico);
        assert_eq!(formato_para(db(2025, 12)), CnpjFormat::Numerico);
        assert_eq!(formato_para(db(2027, 1)), CnpjFormat::Alfanumerico);
    }

    #[test]
    fn tamanho_errado_rejeitado() {
        assert!(CnpjIdentifier::new("1234567", db(2026, 7)).is_err());
        assert!(CnpjIdentifier::new("123456780001901", db(2026, 7)).is_err());
    }

    #[test]
    fn raiz_eh_os_oito_primeiros() {
        let c = CnpjIdentifier::new("11222333000181", db(2026, 7)).unwrap();
        assert_eq!(c.raiz(), "11222333");
        assert!(c.is_completo());
    }
}
