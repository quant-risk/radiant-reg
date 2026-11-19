//! Ingestão estrita de planilhas BACEN (Camada 1).
//!
//! Fonte real: `.xls`/`.xlsx` lido via calamine (`xlsx.rs`), mapeado por
//! `mapping.toml` (`mapping.rs`) para o shape mínimo `CanonicalInput` —
//! o wedge entre o mundo sujo de planilhas e o gerador 3040 tipado.
//!
//! Princípios (docs/blueprint/20-ARQUITETURA-RUST.md §3.2, §5):
//! - Texto primeiro: toda célula vira `String` antes de qualquer conversão;
//!   `Centavos::parse` de reg-core decide a semântica numérica.
//! - Nada de default silencioso: header não detectado, coluna do mapping
//!   ausente ou unidade contratada faltando são `RegError` tipado.
//! - `Transform` e `MonetaryUnit` são enums exaustivos — dispatch sem
//!   wildcard, o skip silencioso do v1 é bug banido por lint do workspace.

mod mapping;
mod xlsx;

pub use mapping::{MappingDocument, MappingMeta, parse_mapping_toml};
pub use xlsx::{canonical_rows, detect_header_row, read_sheet_rows};

use reg_core::error::RegError;
use reg_core::money::MonetaryUnit;
use serde::{Deserialize, Serialize};

/// Re-export do contrato de unidade de reg-core — o mapping.toml fala a
/// mesma língua de `Centavos::parse`, sem conversão intermédia.
pub use reg_core::money::MonetaryUnit as MonetaryUnitReexport;

/// Entrada monetária crua: o texto da célula + a unidade contratada.
/// A conversão acontece APENAS em `Centavos::parse(raw, unit)` (reg-core) —
/// este crate nunca interpreta número por conta própria (zero-float, §4.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonetaryInput {
    /// Texto da célula, tal como veio da planilha (após `Transform`s declarados).
    pub raw: String,
    /// Unidade declarada no mapping — 1ª trava do contrato de unidade.
    pub unit: MonetaryUnit,
}

impl MonetaryInput {
    /// Converte para `Centavos` delegando inteiramente a reg-core.
    pub fn to_centavos(&self) -> Result<reg_core::money::Centavos, RegError> {
        reg_core::money::Centavos::parse(&self.raw, self.unit)
    }
}

/// Caminho do campo alvo, ex. `"Doc3040/titulo/saldo"`.
/// Newtype para o mapping.toml nunca carregar `String` solta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldPath(pub String);

impl FieldPath {
    /// Último segmento do caminho — usado para o contrato de unidade
    /// por campo (1ª trava, ver `mapping::MONETARY_FIELDS`).
    pub fn leaf(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }
}

/// Transform declarado no mapping — enum exaustivo, aplicado na ordem do TOML.
/// Cada variante é uma operação textual determinística; nenhuma interpreta
/// semântica monetária ou temporal por conta própria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transform {
    /// Remove espaços das pontas.
    Trim,
    /// Lowercase ASCII (headers BACEN variam "Campo"/"CAMPO").
    ToLower,
    /// Remove acentos latinos (mapeamento NFD-manual, sem dependência unicode).
    StripAccents,
    /// Converte "dd/mm/aaaa" (ou "dd/mm/aa") para AAAA-MM — `DataBase::parse`.
    DateToDataBase,
}

impl Transform {
    /// Aplica o transform a uma célula textual.
    pub fn apply(self, input: &str) -> String {
        match self {
            Transform::Trim => input.trim().to_string(),
            Transform::ToLower => input.to_lowercase(),
            Transform::StripAccents => strip_accents(input),
            Transform::DateToDataBase => date_to_data_base(input),
        }
    }
}

/// Mapeamento de UMA coluna da planilha para um campo canônico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnMapping {
    /// Header exatamente como aparece na planilha (matched após normalizar
    /// trim/lowercase/acentos — os transforms rodam também no header).
    pub source_header: String,
    /// Campo canônico de destino.
    pub target_field: FieldPath,
    /// Unidade contratada. Obrigatória quando `target_field.leaf()` está em
    /// [`mapping::MONETARY_FIELDS`] — 1ª trava, validada no parse do TOML
    /// e re-checada no ingest (`xlsx::canonical_rows`).
    pub unit: Option<MonetaryUnit>,
    /// Pipeline de transforms aplicado na ordem declarada.
    #[serde(default)]
    pub transform: Vec<Transform>,
}

/// Shape mínimo do wedge: o que o gerador 3040 precisa saber antes de qualquer
/// regra de negócio. Participante/Operação ficam crus aqui — tipagem forte
/// deles entra no gen3040 (S3), não na ingestão.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalInput {
    /// Competência do cadoc — cidadã de 1ª classe (reg-core).
    pub data_base: reg_core::period::DataBase,
    /// CNPJ do IF informante, já validado pela janela temporal IN 729.
    pub if_cnpj: reg_core::cnpj::CnpjIdentifier,
    /// Identificador do participante, cru (tipagem forte no gen3040).
    pub participante: String,
    /// Operação/código do cliente no IF, cru.
    pub operacao: String,
}

/// Remove acentos latinos básicos sem crate de unicode — tabela explícita
/// dos caracteres que aparecem em headers/valores BACEN (pt-BR).
fn strip_accents(input: &str) -> String {
    const MAP: &[(char, char)] = &[
        ('á', 'a'),
        ('à', 'a'),
        ('â', 'a'),
        ('ã', 'a'),
        ('ä', 'a'),
        ('é', 'e'),
        ('ê', 'e'),
        ('è', 'e'),
        ('ë', 'e'),
        ('í', 'i'),
        ('î', 'i'),
        ('ï', 'i'),
        ('ó', 'o'),
        ('ô', 'o'),
        ('õ', 'o'),
        ('ö', 'o'),
        ('ú', 'u'),
        ('û', 'u'),
        ('ü', 'u'),
        ('ç', 'c'),
        ('ñ', 'n'),
        ('Á', 'A'),
        ('À', 'A'),
        ('Â', 'A'),
        ('Ã', 'A'),
        ('Ä', 'A'),
        ('É', 'E'),
        ('Ê', 'E'),
        ('È', 'E'),
        ('Ë', 'E'),
        ('Í', 'I'),
        ('Î', 'I'),
        ('Ï', 'I'),
        ('Ó', 'O'),
        ('Ô', 'O'),
        ('Õ', 'O'),
        ('Ö', 'O'),
        ('Ú', 'U'),
        ('Û', 'U'),
        ('Ü', 'U'),
        ('Ç', 'C'),
        ('Ñ', 'N'),
    ];
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match MAP.iter().find(|(from, _)| *from == c) {
            Some((_, to)) => out.push(*to),
            None => out.push(c),
        }
    }
    out
}

/// "dd/mm/aaaa" (ou "dd/mm/aa") → "AAAA-MM" (o DIA é descartado — o
/// transform produz data-base de competência mensal, formato que
/// `DataBase::parse` de reg-core aceita). Caso contrário, devolve o input
/// aparado — a validação dura fica em `DataBase::parse` (reg-core).
/// Para campos A10 com dia (DtContr etc.) use um transform próprio de data
/// ISO — NÃO reutilize este transform: ele nunca emite "AAAA-MM-DD".
fn date_to_data_base(input: &str) -> String {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() == 3 {
        let (d, m, y) = (parts[0], parts[1], parts[2]);
        let is_digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
        if (d.len() == 1 || d.len() == 2) && (m.len() == 1 || m.len() == 2) {
            let year = if y.len() == 2 {
                format!("20{y}")
            } else {
                y.to_string()
            };
            if is_digits(d) && is_digits(m) && (year.len() == 4 && is_digits(&year)) {
                return format!("{year}-{:0>2}", m);
            }
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn transforms_sao_deterministicos() {
        assert_eq!(Transform::Trim.apply("  x  "), "x");
        assert_eq!(Transform::ToLower.apply("CAMPO"), "campo");
        assert_eq!(Transform::StripAccents.apply("Obrigatório"), "Obrigatorio");
        assert_eq!(Transform::StripAccents.apply("Código"), "Codigo");
    }

    #[test]
    fn date_to_data_base_converte_e_preserva_invalido() {
        // Contrato do transform (doc): "dd/mm/aaaa" → AAAA-MM — o DIA é
        // descartado; a saída é aceitável por DataBase::parse (AAAA-MM).
        assert_eq!(Transform::DateToDataBase.apply("30/06/2026"), "2026-06");
        assert_eq!(Transform::DateToDataBase.apply("01/07/26"), "2026-07");
        // A saída FECHA o contrato com DataBase::parse (nada de "AAAA-MM-DD"):
        assert_eq!(
            reg_core::period::DataBase::parse(&Transform::DateToDataBase.apply("30/06/2026")),
            Ok(reg_core::period::DataBase::new(2026, 6).unwrap())
        );
        // Não é data: devolve aparado — DataBase::parse decide depois.
        assert_eq!(Transform::DateToDataBase.apply("junho"), "junho");
    }

    #[test]
    fn monetary_input_delega_ao_reg_core() {
        let m = MonetaryInput {
            raw: "1.234,56".into(),
            unit: MonetaryUnit::Reais,
        };
        assert_eq!(m.to_centavos().unwrap().as_i128(), 123_456);
        let bad = MonetaryInput {
            raw: "abc".into(),
            unit: MonetaryUnit::Reais,
        };
        assert!(bad.to_centavos().is_err());
    }

    #[test]
    fn field_path_leaf() {
        assert_eq!(FieldPath("Doc3040/titulo/saldo".into()).leaf(), "saldo");
        assert_eq!(FieldPath("saldo".into()).leaf(), "saldo");
    }
}
