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

/// Campos monetários do domínio SCR (formato N19,2 do leiaute) — leaf do
/// `FieldPath`. A 1ª trava: declarar uma coluna cujo leaf está aqui sem
/// `unit` é erro de parse. Lista derivada da aba `Doc3040` de
/// `3040/SCR3040_Leiaute.xls` (L24 FatAnual, L54 VlrContr, L60 ProvConsttd,
/// L64 VlrProxParcela, L69 Venc/v*, L81-82 garantias, L95 Valor, L105-109
/// agregado, L118-119 ContAquisição, L137-141 ContInstFinRes4966, L161
/// Perda) + os substantivos genéricos do domínio; o teste
/// `monetarios_do_leiaute_doc3040_cobertos_pela_trava` percorre o leiaute
/// REAL e prova a cobertura (drift do leiaute quebra o teste).
pub const MONETARY_FIELDS: &[&str] = &[
    // Substantivos genéricos do domínio (herdados da versão anterior).
    "saldo", "valor", "limite", "parcela", "encargo", "principal", "garantia",
    // Cli (L24) e Op (L54/L60/L64) da aba Doc3040.
    "fat_anual", "vlr_contr", "prov_consttd", "vlr_prox_parcela",
    // Venc — todos os códigos do Anexo 1 (vCOD, L69).
    "v20", "v40", "v60", "v80", "v110", "v120", "v130", "v140", "v150", "v160", "v165",
    "v170", "v175", "v180", "v190", "v199",
    "v205", "v210", "v220", "v230", "v240", "v245", "v250", "v255", "v260", "v270", "v280", "v290",
    "v310", "v320", "v330",
    // Gar (L81/L82) e Inf (L95).
    "vlr_orig", "vlr_data",
    // Agregado (L105/L106/L109).
    "vlr_saldo_tot", "vlr_saldo_vinc", "vlr_bonus_rebate",
    // ContAquisicao (L118/L119) e ContInstFinRes4966 (L137-141).
    "cst_amort", "vlr_mercado", "vlr_cont_br", "vlr_perda_acum", "vlr_justo", "rend_mes",
    // Perda (L161).
    "vlr_perda",
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
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

    /// A trava 1 cobre os leaves monetários REAIS do leiaute: o fixture
    /// oficial `3040/SCR3040_Leiaute.xls` é percorrido e TODO campo com
    /// Formato N19,2 (monetário) tem o atributo (camelCase → snake_case)
    /// em `MONETARY_FIELDS`. Drift do leiaute (re-download S1) quebra o
    /// teste — sinal de que a trava precisa ser atualizada, nunca burlada.
    #[test]
    fn monetarios_do_leiaute_doc3040_cobertos_pela_trava() {
        use crate::ingest::read_sheet_rows;
        use std::path::Path;

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../3040/SCR3040_Leiaute.xls");
        if !path.exists() {
            panic!("fixture real ausente: {}", path.display());
        }
        let rows = read_sheet_rows(&path, "Doc3040").unwrap();
        let header_idx = crate::ingest::detect_header_row(&rows).unwrap();
        let header = &rows[header_idx];
        let col_idx = |nome: &str| {
            header.iter().position(|h| {
                let mut s = h.to_lowercase();
                s = s.trim().to_string();
                s == nome
            })
        };
        let col_formato = col_idx("formato").expect("coluna Formato no header Doc3040");
        let col_attr = header
            .iter()
            .position(|h| h.to_lowercase().trim() == "atributo")
            .expect("coluna Atributo no header Doc3040");

        let snake = |attr: &str| -> String {
            let mut out = String::new();
            for (i, c) in attr.chars().enumerate() {
                if c.is_ascii_uppercase() {
                    if i > 0 {
                        out.push('_');
                    }
                    out.push(c.to_ascii_lowercase());
                } else {
                    out.push(c);
                }
            }
            out
        };

        let mut cobertos = 0usize;
        for row in &rows[header_idx + 1..] {
            let formato = row.get(col_formato).map(String::as_str).unwrap_or("").trim().to_string();
            let attr = row.get(col_attr).map(String::as_str).unwrap_or("").trim().to_string();
            if formato != "N19,2" || attr.is_empty() {
                continue; // Venc (L69) tem Atributo vazio — códigos v* cobertos pela lista estática
            }
            let leaf = snake(&attr);
            assert!(
                MONETARY_FIELDS.contains(&leaf.as_str()),
                "campo N19,2 do leiaute ({attr} → {leaf}) fora da trava 1 — MONETARY_FIELDS incompleta"
            );
            cobertos += 1;
        }
        // Guarda mínima: o leiaute REAL tem campos N19,2 com atributo nomeado.
        assert!(cobertos >= 10, "leiaute inesperado: só {cobertos} N19,2 com atributo");
        // E os códigos de Venc (Anexo 1) estão todos na trava:
        for cod in ["v110", "v130", "v205", "v330"] {
            assert!(MONETARY_FIELDS.contains(&cod), "código de Venc {cod} fora da trava");
        }
    }
}
