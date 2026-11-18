//! Leitor calamine de planilhas BACEN (.xls/.xlsx) → rows de `String`.
//!
//! Comportamento espelhado no `_catalogos/extract.py` do v1
//! (docs/blueprint/20-ARQUITETURA-RUST.md §3.2): header pode estar em
//! qualquer uma das primeiras linhas (keyword "cód"/"codigo"/"campo"/"tag")
//! e o código/pivo pode estar em qualquer coluna — aqui usamos o mapping
//! declarado em vez de heurística de coluna.
//!
//! Texto primeiro: toda célula (float, date, bool) é renderizada para
//! `String` determinística; a semântica numérica fica em `Centavos::parse`
//! e a temporal em `DataBase::parse` — nunca no leitor.

use calamine::{Data, open_workbook_auto, Reader};
use reg_core::error::RegError;

use super::mapping::monetary_input;
use super::{CanonicalInput, ColumnMapping, MappingDocument, Transform};

/// Palavras-chave de header — mesma lista de comportamento do extract.py v1,
/// adaptada aos leiautes (Doc3040 usa "Campo"/"Formato"; críticas usam "Cód").
const HEADER_KEYWORDS: &[&str] = &["cód", "codigo", "código", "campo", "tag", "chave"];

/// Número máximo de linhas inspecionadas para achar o header (extract.py: 10).
const MAX_HEADER_SEARCH: usize = 10;

/// Lê uma sheet inteira como rows de `String` (célula vazia = string vazia).
pub fn read_sheet_rows(
    path: &std::path::Path,
    sheet_name: &str,
) -> Result<Vec<Vec<String>>, RegError> {
    const FIELD: &str = "ingest.xlsx";
    let path_str = path.display().to_string();
    let mut wb = open_workbook_auto(path)
        .map_err(|e| RegError::InvalidValue { field: FIELD.into(), reason: format!("abrir {path_str}: {e}") })?;

    let range = wb
        .worksheet_range(sheet_name)
        .map_err(|e| RegError::InvalidValue { field: FIELD.into(), reason: format!("sheet {sheet_name:?} de {path_str}: {e}") })?;

    let mut rows = Vec::with_capacity(range.height());
    for row in range.rows() {
        rows.push(row.iter().map(cell_to_string).collect());
    }
    Ok(rows)
}

/// Renderiza uma célula calamine para `String` determinística — dispatch
/// EXAUSTIVO sobre `Data` (wildcard_enum_match_arm é lint denied do workspace).
/// Floats integrais viram inteiros (`30.0` → `"30"`), igual ao read_xls do v1.
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Bool(b) => b.to_string(),
        Data::Error(e) => format!("#ERRO:{e:?}"),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 9.0e15 {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        // Serial numérico do Excel cru — sem conversão silenciosa para data;
        // DateToDataBase/DataBase::parse decidem a semântica depois.
        Data::DateTime(dt) => {
            let f = dt.as_f64();
            if f.fract() == 0.0 && f.abs() < 9.0e15 {
                format!("{}", f as i64)
            } else {
                format!("{f}")
            }
        }
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

/// Detecta a linha de header procurando keywords nas primeiras N linhas,
/// célula a célula (não na linha concatenada — mais preciso que o v1).
/// Retorna `None` se nada for achado — o chamador decide o erro
/// (nunca "assumir linha 0" como o v1 fazia por baixo).
pub fn detect_header_row(rows: &[Vec<String>]) -> Option<usize> {
    for (i, row) in rows.iter().take(MAX_HEADER_SEARCH).enumerate() {
        for cell in row {
            let normalized: String = Transform::StripAccents.apply(cell);
            let lower = normalized.to_lowercase();
            if HEADER_KEYWORDS.iter().any(|k| lower.contains(k)) {
                return Some(i);
            }
        }
    }
    None
}

/// Localiza o índice da coluna cujo header (normalizado trim/lower/acentos)
/// bate com `source_header` do mapping. Erro tipado — nunca coluna 0 por
/// omissão (fio desligado do `detect_code_column` do v1).
fn find_column(header: &[String], mapping: &ColumnMapping) -> Result<usize, RegError> {
    let want = normalize_header(&mapping.source_header);
    header
        .iter()
        .position(|h| normalize_header(h) == want)
        .ok_or_else(|| RegError::MissingField {
            field: format!("coluna {:?} na planilha", mapping.source_header),
        })
}

fn normalize_header(h: &str) -> String {
    let mut s = Transform::StripAccents.apply(h);
    s = Transform::ToLower.apply(&s);
    s.trim().to_string()
}

/// Produz os `CanonicalInput` a partir de uma sheet lida + mapping validado.
/// Linhas vazias (todas as células em branco) são puladas explicitamente —
/// não é skip silencioso: é delimitação de registro, documentada.
pub fn canonical_rows(
    rows: &[Vec<String>],
    mapping: &MappingDocument,
) -> Result<Vec<CanonicalInput>, RegError> {
    const FIELD: &str = "ingest.canonical";
    let header_idx = detect_header_row(rows).ok_or_else(|| RegError::InvalidValue {
        field: FIELD.into(),
        reason: "header não detectado nas primeiras 10 linhas".into(),
    })?;
    let header = &rows[header_idx];

    let data_base = mapping.data_base;
    let if_cnpj = reg_core::cnpj::CnpjIdentifier::new(&mapping.if_cnpj_raw, data_base)
        .map_err(|e| RegError::InvalidValue { field: "mapping.meta.if_cnpj".into(), reason: e.to_string() })?;

    let participante_col = find_column(header, &ColumnMapping {
        source_header: mapping.participante_header.clone(),
        target_field: super::FieldPath("participante".into()),
        unit: None,
        transform: Vec::new(),
    })?;
    let operacao_col = find_column(header, &ColumnMapping {
        source_header: mapping.operacao_header.clone(),
        target_field: super::FieldPath("operacao".into()),
        unit: None,
        transform: Vec::new(),
    })?;

    // Colunas do [[columns]] resolvidas uma vez; monotetárias com unit=None
    // são transform-textuais (ex. vencimento) e não entram no wedge mínimo.
    let mut resolved: Vec<(usize, &ColumnMapping)> = Vec::new();
    for col in &mapping.columns {
        let idx = find_column(header, col)?;
        if col.unit.is_some() {
            resolved.push((idx, col));
        }
    }

    let mut out = Vec::new();
    for row in &rows[header_idx + 1..] {
        let cell = |i: usize| row.get(i).map(String::as_str).unwrap_or("");
        let participante = cell(participante_col).trim().to_string();
        let operacao = cell(operacao_col).trim().to_string();
        if participante.is_empty() && operacao.is_empty() {
            continue; // linha em branco — delimitação de registro, não skip de dado
        }
        if participante.is_empty() || operacao.is_empty() {
            return Err(RegError::MissingField {
                field: format!("linha {}: participante/operacao parcial", out.len() + header_idx + 2),
            });
        }
        // Valida os campos monetários contratados — trava 1 re-checada aqui:
        // célula vazia em campo monetário obrigatório é erro tipado.
        for (idx, col) in &resolved {
            monetary_input(col, cell(*idx))?.to_centavos()?;
        }
        out.push(CanonicalInput { data_base, if_cnpj: if_cnpj.clone(), participante, operacao });
    }
    Ok(out)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::ingest::parse_mapping_toml;
    use std::path::Path;

    /// Fixture REAL do repo: leiaute oficial SCR 3040 (.xls BIFF8 legado).
    fn leiaute_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../3040/SCR3040_Leiaute.xls")
    }

    #[test]
    fn detect_header_em_rows_sinteticas() {
        let rows = vec![
            vec!["CADOC SCR — Leiaute".into(), "".into()],
            vec!["Campo".into(), "Formato".into(), "Obrigatório".into()],
            vec!["CNPJ".into(), "Texto(14)".into(), "Sim".into()],
        ];
        assert_eq!(detect_header_row(&rows), Some(1));
        let none: Vec<Vec<String>> = vec![vec!["x".into()]];
        assert_eq!(detect_header_row(&none), None);
    }

    #[test]
    fn xls_real_abre_e_detecta_header_doc3040() {
        let path = leiaute_path();
        if !path.exists() {
            panic!("fixture real ausente: {}", path.display());
        }
        let rows = read_sheet_rows(&path, "Doc3040").expect("sheet Doc3040 deve abrir");
        assert!(rows.len() > 10, "Doc3040 deve ter linhas suficientes, teve {}", rows.len());
        let h = detect_header_row(&rows).expect("header de Doc3040 deve ser detectado");
        let header_join: String = rows[h].join("|");
        assert!(
            ["campo", "formato"].iter().all(|k| header_join.to_lowercase().contains(k)),
            "header inesperado: {header_join}"
        );
        // 2 células conhecidas: "Campo" e "Formato" presentes no header.
        assert!(rows[h].iter().any(|c| c.trim().eq_ignore_ascii_case("campo")));
        assert!(rows[h].iter().any(|c| normalize_header(c) == "formato"));
    }

    #[test]
    fn sheet_inexistente_erra_tipado() {
        let err = read_sheet_rows(&leiaute_path(), "NaoExiste").unwrap_err();
        assert!(matches!(err, RegError::InvalidValue { .. }));
    }

    #[test]
    fn canonical_rows_end_to_end_sintetico() {
        let mapping_toml = r#"
            [meta]
            cadoc = "3040"
            data_base = "2026-06"
            if_cnpj = "11222333000181"
            participante = "Participante"
            operacao = "Operação"

            [[columns]]
            source_header = "Saldo"
            target_field = "Doc3040/titulo/saldo"
            unit = "reais"
            transform = ["trim"]
        "#;
        let doc = parse_mapping_toml(mapping_toml).unwrap();
        let rows = vec![
            vec!["Layout".into(), "".into()],
            vec!["Campo".into(), "Participante".into(), "Operação".into(), "Saldo".into()],
            vec!["cnpj".into(), "ACME SA".into(), "OP-1".into(), "1.234,56".into()],
            vec!["".into(), "".into(), "".into(), "".into()],
            vec!["cnpj".into(), "BETA LTDA".into(), "OP-2".into(), "42,00".into()],
        ];
        let out = canonical_rows(&rows, &doc).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].participante, "ACME SA");
        assert_eq!(out[0].operacao, "OP-1");
        assert_eq!(out[1].participante, "BETA LTDA");
        assert_eq!(out[0].data_base, reg_core::period::DataBase::new(2026, 6).unwrap());
        assert_eq!(out[0].if_cnpj.as_str(), "11222333000181");

        // Valor não numérico em campo monetário → erro da trava 1.
        let bad_rows = vec![
            rows[1].clone(),
            vec!["cnpj".into(), "ACME".into(), "OP-3".into(), "abc".into()],
        ];
        assert!(canonical_rows(&bad_rows, &doc).is_err());
    }

    #[test]
    fn coluna_ausente_erra_com_nome() {
        let mapping_toml = r#"
            [meta]
            cadoc = "3040"
            data_base = "2026-06"
            if_cnpj = "11222333000181"
            participante = "Campo"
            operacao = "Operação"
            columns = []
        "#;
        // columns vazio no TOML não é permitido — testamos via doc válido.
        let doc = parse_mapping_toml(&mapping_toml.replace(
            "columns = []",
            "[[columns]]\nsource_header = \"ZZZ\"\ntarget_field = \"outro\"",
        )).unwrap();
        let rows = vec![vec!["Campo".into(), "Operação".into()]];
        let err = canonical_rows(&rows, &doc).unwrap_err();
        assert!(matches!(err, RegError::MissingField { ref field } if field.contains("ZZZ")));
    }
}
