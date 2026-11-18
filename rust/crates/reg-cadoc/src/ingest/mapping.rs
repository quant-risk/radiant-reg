//! Parse de `mapping.toml` — o contrato de ingestão do cadoc.
//!
//! TRAVAS do blueprint (docs/blueprint/20-ARQUITETURA-RUST.md §3.2 e ADR dos
//! "4-olhos" no reg-ledger):
//!
//! 1. **Contrato de unidade por campo (esta fase, implementada aqui):** todo
//!    campo cujo leaf está em [`MONETARY_FIELDS`] DEVE declarar `unit` no
//!    TOML. Falta de unidade = erro no parse, nunca inferência silenciosa.
//!    (`FieldPath::leaf` + `MonetaryUnit` tipado — a trava é o tipo, não
//!    convenção.)
//! 2. **Lock de 4-olhos (S5, reg-ledger::mapping_locks):** o hash do
//!    mapping.toml aprovado por 2 operadores é gravado no ledger e a ingestão
//!    desta crate recebe o mapping já verificado — aqui apenas o parse e a
//!    validação estrutural; o lock em si vive na Camada 2.
//! 3. **Provenance (S5):** cada registro canônico carrega o id/hash do
//!    mapping usado — entra no dossiê RC 18 via reg-ledger.
//!
//! Este módulo não conhece planilha: conhece o CONTRATO. O consumo acontece
//! em `xlsx.rs`.

use reg_core::error::RegError;
use serde::{Deserialize, Serialize};

use super::{ColumnMapping, FieldPath, MonetaryInput};

/// Campos monetários do domínio SCR — leaf do `FieldPath`. A 1ª trava:
/// declarar uma coluna cujo leaf está aqui sem `unit` é erro de parse.
pub const MONETARY_FIELDS: &[&str] = &[
    "saldo", "valor", "limite", "parcela", "encargo", "principal", "garantia",
];

/// Cabeçalho do mapping.toml.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingMeta {
    /// Cadoc alvo — "3040", "2030", ...
    pub cadoc: String,
    /// Competência AAAA-MM (parseado para `DataBase` no doc pronto).
    pub data_base: String,
    /// CNPJ do IF informante.
    pub if_cnpj: String,
    /// Header da coluna obrigatória do participante.
    pub participante: String,
    /// Header da coluna obrigatória da operação.
    pub operacao: String,
}

/// Coluna crua do TOML (antes da validação das travas).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawColumn {
    source_header: String,
    target_field: String,
    unit: Option<reg_core::money::MonetaryUnit>,
    #[serde(default)]
    transform: Vec<super::Transform>,
}

/// mapping.toml cru, exatamente como o TOML declara.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawMapping {
    meta: MappingMeta,
    #[serde(default)]
    columns: Vec<RawColumn>,
}

/// mapping.toml validado — as travas estruturais já garantidas pelo tipo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingDocument {
    pub meta: MappingMeta,
    /// Data-base parseada (strito AAAA-MM) — erro aqui, não na ingestão.
    pub data_base: reg_core::period::DataBase,
    pub if_cnpj_raw: String,
    /// Header da coluna do participante.
    pub participante_header: String,
    /// Header da coluna da operação.
    pub operacao_header: String,
    /// Colunas validadas (trava 1 aplicada).
    pub columns: Vec<ColumnMapping>,
}

/// Parse e valida um mapping.toml. A trava 1 (unidade por campo) é
/// enforceada aqui: coluna monetária sem `unit` → `RegError::MissingField`.
pub fn parse_mapping_toml(raw_toml: &str) -> Result<MappingDocument, RegError> {
    const FIELD: &str = "mapping.toml";
    let raw: RawMapping = toml::from_str(raw_toml)
        .map_err(|e| RegError::InvalidValue { field: FIELD.into(), reason: format!("TOML inválido: {e}") })?;

    if raw.columns.is_empty() {
        return Err(RegError::InvalidValue { field: FIELD.into(), reason: "nenhuma coluna mapeada".into() });
    }

    let mut columns = Vec::with_capacity(raw.columns.len());
    for col in raw.columns {
        let path = FieldPath(col.target_field);
        // TRAVA 1 — contrato de unidade por campo monetário.
        if MONETARY_FIELDS.contains(&path.leaf()) && col.unit.is_none() {
            return Err(RegError::MissingField {
                field: format!("mapping[{}].unit", col.source_header),
            });
        }
        if col.source_header.trim().is_empty() {
            return Err(RegError::InvalidValue {
                field: FIELD.into(),
                reason: format!("header vazio para {}", path.0),
            });
        }
        columns.push(ColumnMapping {
            source_header: col.source_header,
            target_field: path,
            unit: col.unit,
            transform: col.transform,
        });
    }

    let meta = raw.meta;
    let data_base = reg_core::period::DataBase::parse(meta.data_base.trim())
        .map_err(|e| RegError::InvalidValue { field: format!("{FIELD}.meta.data_base"), reason: e.to_string() })?;

    Ok(MappingDocument {
        data_base,
        if_cnpj_raw: meta.if_cnpj.clone(),
        participante_header: meta.participante.clone(),
        operacao_header: meta.operacao.clone(),
        columns,
        meta,
    })
}

/// Converte uma célula textual em `MonetaryInput` via o pipeline de transforms
/// declarado. Usado por `xlsx.rs`; exposto aqui porque a trava de unidade é
/// responsabilidade do mapping.
pub(crate) fn monetary_input(col: &ColumnMapping, cell: &str) -> Result<MonetaryInput, RegError> {
    let unit = col.unit.ok_or_else(|| RegError::MissingField {
        field: format!("mapping[{}].unit", col.source_header),
    })?;
    let mut raw = cell.to_string();
    for t in &col.transform {
        raw = t.apply(&raw);
    }
    Ok(MonetaryInput { raw, unit })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use reg_core::money::MonetaryUnit;

    const VALID: &str = r#"
        [meta]
        cadoc = "3040"
        data_base = "2026-06"
        if_cnpj = "12345678"
        participante = "Nome do Participante"
        operacao = "Operacao"

        [[columns]]
        source_header = "Saldo"
        target_field = "Doc3040/titulo/saldo"
        unit = "reais"
        transform = ["trim"]

        [[columns]]
        source_header = "Vencimento"
        target_field = "Doc3040/titulo/vencimento"
        transform = ["date_to_data_base"]
    "#;

    #[test]
    fn parse_valido() {
        let doc = parse_mapping_toml(VALID).unwrap();
        assert_eq!(doc.data_base, reg_core::period::DataBase::new(2026, 6).unwrap());
        assert_eq!(doc.columns.len(), 2);
        assert_eq!(doc.columns[0].unit, Some(MonetaryUnit::Reais));
    }

    #[test]
    fn trava_1_campo_monetario_sem_unidade_erra() {
        let broken = VALID.replace("unit = \"reais\"\n", "");
        let err = parse_mapping_toml(&broken).unwrap_err();
        assert!(matches!(err, RegError::MissingField { ref field } if field.contains("unit")));
    }

    #[test]
    fn toml_vazio_erra() {
        assert!(parse_mapping_toml("").is_err());
    }

    #[test]
    fn data_base_invalida_erra_no_parse() {
        let bad = VALID.replace("2026-06", "2026-13");
        assert!(parse_mapping_toml(&bad).is_err());
    }

    #[test]
    fn monetary_input_aplica_transforms() {
        let doc = parse_mapping_toml(VALID).unwrap();
        let mi = monetary_input(&doc.columns[0], " 1.234,56 ").unwrap();
        assert_eq!(mi.raw, "1.234,56");
        assert_eq!(mi.unit, MonetaryUnit::Reais);
    }
}
