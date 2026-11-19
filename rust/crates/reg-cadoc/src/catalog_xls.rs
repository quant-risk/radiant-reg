//! Extrator determinístico e zero-float dos catálogos oficiais BACEN (.xls):
//! `SCR3040_Criticas.xls` e `SCR3040_Leiaute.xls`.
//!
//! Usa apenas `calamine`, `serde`, `serde_json`, `sha2` e `hex` — zero floats
//! (`float_arithmetic`/`float_cmp`/`cast_precision_loss` banidos pelo workspace),
//! zero `unsafe`, zero `unwrap`/`expect`, dispatch exaustivo em `calamine::Data`.

use std::collections::BTreeMap;
use std::fmt;
use std::io::Cursor;
use std::path::Path;

use calamine::{Data, ExcelDateTime, Reader, open_workbook_auto_from_rs};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Abas oficiais de regras em `SCR3040_Criticas.xls` (ordem canônica BCB).
pub const ABAS_REGRAS_3040: &[&str] = &[
    "Básicas",
    "Campos Obrigatórios",
    "Formato",
    "Semântica",
    "Individualizadas",
    "Agregadas",
    "Negócio",
    "Substituição Parcial",
];

/// Aba padrão de histórico em ambos os `.xls` oficiais do BCB.
pub const ABA_HISTORICO: &str = "HistoricoAtualizacoes";

/// Aba principal de campos em `SCR3040_Leiaute.xls`.
pub const ABA_DOC3040: &str = "Doc3040";

/// Aba de tabelas de domínio em `SCR3040_Leiaute.xls`.
pub const ABA_ANEXO: &str = "Anexo";

/// Aba específica do Anexo 26 (matriz de Informações Adicionais).
pub const ABA_ANEXO_26: &str = "Anexo 26 - InfosAdicionais";

/// Erro tipado da leitura e extração dos catálogos `.xls`.
#[derive(Debug)]
pub enum CatalogError {
    /// Falha de I/O (arquivo ausente, ilegível ou falha ao gravar saída).
    Io {
        path: String,
        source: std::io::Error,
    },
    /// Falha ao abrir/parsear o container `.xls` via calamine.
    Xls { path: String, reason: String },
    /// Aba obrigatória ausente na planilha.
    SheetAusente { path: String, sheet: String },
    /// Cabeçalho esperado não encontrado ou coluna obrigatória ausente.
    HeaderInvalido { sheet: String, reason: String },
    /// Valor de célula inválido/inesperado (ex.: `Habilitado?` fora de `s`/`n`).
    ValorInvalido {
        sheet: String,
        linha: usize,
        reason: String,
    },
    /// Erro de serialização JSON.
    Json(serde_json::Error),
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "falha de I/O em {path}: {source}"),
            Self::Xls { path, reason } => write!(f, "falha ao ler planilha {path}: {reason}"),
            Self::SheetAusente { path, sheet } => {
                write!(f, "aba obrigatória {sheet:?} ausente em {path}")
            }
            Self::HeaderInvalido { sheet, reason } => {
                write!(f, "cabeçalho inválido na aba {sheet:?}: {reason}")
            }
            Self::ValorInvalido {
                sheet,
                linha,
                reason,
            } => write!(f, "valor inválido em {sheet:?} linha {linha}: {reason}"),
            Self::Json(err) => write!(f, "falha ao serializar JSON do catálogo: {err}"),
        }
    }
}

impl std::error::Error for CatalogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json(err) => Some(err),
            Self::Xls { .. }
            | Self::SheetAusente { .. }
            | Self::HeaderInvalido { .. }
            | Self::ValorInvalido { .. } => None,
        }
    }
}

/// Estado `Habilitado?` oficial do BCB (`s` ou `n`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Habilitado {
    #[serde(rename = "s")]
    S,
    #[serde(rename = "n")]
    N,
}

impl Habilitado {
    /// Verdadeiro quando `Habilitado=s`.
    pub fn is_ativa(self) -> bool {
        match self {
            Self::S => true,
            Self::N => false,
        }
    }

    fn parse(raw: &str, sheet: &str, linha: usize) -> Result<Self, CatalogError> {
        let norm = raw.trim().to_ascii_lowercase();
        match norm.as_str() {
            "s" => Ok(Self::S),
            "n" => Ok(Self::N),
            _ => Err(CatalogError::ValorInvalido {
                sheet: sheet.to_string(),
                linha,
                reason: format!("Habilitado? inesperado: {raw:?} (esperado 's' ou 'n')"),
            }),
        }
    }
}

/// Uma crítica canônica extraída de `SCR3040_Criticas.xls`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Critica3040 {
    pub codigo: String,
    pub habilitado: Habilitado,
    pub categoria_aba: String,
    pub inicio_vigencia: Option<String>,
    pub fim_vigencia: Option<String>,
    pub alerta_desde: Option<String>,
    pub erro_desde: Option<String>,
    pub titulo: String,
    pub mensagem: String,
    pub nota_nr: Option<String>,
}

/// Contagens agregadas de uma aba/categoria de críticas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContagemCategoria {
    pub total: usize,
    pub habilitado_s: usize,
    pub habilitado_n: usize,
}

/// Contagens exatas do catálogo de críticas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContagensCriticas {
    pub total: usize,
    pub habilitado_s: usize,
    pub habilitado_n: usize,
    pub por_categoria: BTreeMap<String, ContagemCategoria>,
}

/// Registro da aba `HistoricoAtualizacoes`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricoAtualizacao {
    pub data: Option<String>,
    pub atualizacao: String,
    pub detalhamento: String,
    pub observacoes: Option<String>,
}

/// Catálogo completo de críticas extraído de `SCR3040_Criticas.xls`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogoCriticas3040 {
    pub cadoc: String,
    pub fonte: String,
    pub sha256: String,
    pub contagens: ContagensCriticas,
    pub criticas: Vec<Critica3040>,
    pub historico_atualizacoes: Vec<HistoricoAtualizacao>,
}

/// Categoria normalizada da coluna `Obrigatório` do leiaute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObrigatoriedadeTipo {
    Sim,
    Nao,
    Condicional,
    NaoInformado,
}

/// Cartão de campo estruturado extraído da aba `Doc3040` de `SCR3040_Leiaute.xls`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampoLeiaute {
    pub ordem: usize,
    pub secao: String,
    pub tag: String,
    pub tag_pai: Option<String>,
    pub caminho_xml: String,
    pub atributo: Option<String>,
    pub atributos: Vec<String>,
    pub campo: String,
    pub tipo: Option<String>,
    pub obrigatoriedade: Option<String>,
    pub obrigatorio_categoria: ObrigatoriedadeTipo,
    pub dominio_anexos: Vec<String>,
    pub descricao: String,
    pub fidcs: Option<String>,
    pub observacoes: Option<String>,
    pub notas_nr: Vec<String>,
    pub descontinuado: Option<String>,
}

/// Item (ou subitem) de domínio de um Anexo na aba `Anexo`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnexoItem {
    pub codigo: String,
    pub descricao: String,
    pub subcodigo: Option<String>,
    pub subdescricao: Option<String>,
    pub identificador: Option<String>,
    pub notas_nr: Vec<String>,
    pub descontinuado: Option<String>,
}

/// Tabela de domínio extraída da aba `Anexo`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnexoDominio {
    pub numero: u32,
    pub id: String,
    pub titulo: String,
    pub notas_nr: Vec<String>,
    pub vigencia: Option<String>,
    pub descontinuado: Option<String>,
    pub itens: Vec<AnexoItem>,
}

/// Linha da matriz `Anexo 26 - InfosAdicionais`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfoAdicionalAnexo26 {
    pub tp_grupo: String,
    pub tp_grupo_descricao: String,
    pub subtipo: String,
    pub subtipo_descricao: String,
    pub cd: Option<String>,
    pub ident: Option<String>,
    pub valor: Option<String>,
    pub perc: Option<String>,
    pub qtd: Option<String>,
    pub notas_nr: Vec<String>,
    pub descontinuado: Option<String>,
}

/// Contagens agregadas do catálogo de leiaute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContagensLeiaute {
    pub total_campos: usize,
    pub total_elementos: usize,
    pub total_atributos: usize,
    pub total_anexos: usize,
    pub total_anexo26_itens: usize,
    pub total_historico: usize,
}

/// Catálogo completo de leiaute extraído de `SCR3040_Leiaute.xls`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogoLeiaute3040 {
    pub cadoc: String,
    pub fonte: String,
    pub sha256: String,
    pub contagens: ContagensLeiaute,
    pub campos: Vec<CampoLeiaute>,
    pub anexos: Vec<AnexoDominio>,
    pub anexo26_infos_adicionais: Vec<InfoAdicionalAnexo26>,
    pub historico_atualizacoes: Vec<HistoricoAtualizacao>,
}

/// Lê e extrai `SCR3040_Criticas.xls` deterministicamente a partir do caminho em disco.
pub fn extrair_criticas_3040(path: &Path) -> Result<CatalogoCriticas3040, CatalogError> {
    let path_str = path.display().to_string();
    let bytes = std::fs::read(path).map_err(|source| CatalogError::Io {
        path: path_str.clone(),
        source,
    })?;
    let fonte = file_name_or_path(path, &path_str);
    extrair_criticas_3040_bytes(&bytes, &fonte, &path_str)
}

/// Lê e extrai `SCR3040_Criticas.xls` a partir de bytes em memória (mesmo código da via disco).
pub fn extrair_criticas_3040_bytes(
    bytes: &[u8],
    fonte: &str,
    path_label: &str,
) -> Result<CatalogoCriticas3040, CatalogError> {
    let sha256 = hex::encode(Sha256::digest(bytes));
    let mut wb = open_workbook_auto_from_rs(Cursor::new(bytes)).map_err(|e| CatalogError::Xls {
        path: path_label.to_string(),
        reason: e.to_string(),
    })?;

    let mut criticas = Vec::new();
    let mut por_categoria = BTreeMap::new();
    let mut total_s: usize = 0;
    let mut total_n: usize = 0;

    for &sheet_name in ABAS_REGRAS_3040 {
        let rows = read_sheet_cells(&mut wb, path_label, sheet_name)?;
        let aba_criticas = parse_criticas_sheet(sheet_name, &rows)?;
        let mut cat_s: usize = 0;
        let mut cat_n: usize = 0;
        for c in &aba_criticas {
            match c.habilitado {
                Habilitado::S => cat_s += 1,
                Habilitado::N => cat_n += 1,
            }
        }
        total_s += cat_s;
        total_n += cat_n;
        por_categoria.insert(
            sheet_name.to_string(),
            ContagemCategoria {
                total: aba_criticas.len(),
                habilitado_s: cat_s,
                habilitado_n: cat_n,
            },
        );
        criticas.extend(aba_criticas);
    }

    let hist_rows = read_sheet_cells(&mut wb, path_label, ABA_HISTORICO)?;
    let historico_atualizacoes = parse_historico_sheet(ABA_HISTORICO, &hist_rows)?;

    Ok(CatalogoCriticas3040 {
        cadoc: "3040".to_string(),
        fonte: fonte.to_string(),
        sha256,
        contagens: ContagensCriticas {
            total: criticas.len(),
            habilitado_s: total_s,
            habilitado_n: total_n,
            por_categoria,
        },
        criticas,
        historico_atualizacoes,
    })
}

/// Lê e extrai `SCR3040_Leiaute.xls` deterministicamente a partir do caminho em disco.
pub fn extrair_leiaute_3040(path: &Path) -> Result<CatalogoLeiaute3040, CatalogError> {
    let path_str = path.display().to_string();
    let bytes = std::fs::read(path).map_err(|source| CatalogError::Io {
        path: path_str.clone(),
        source,
    })?;
    let fonte = file_name_or_path(path, &path_str);
    extrair_leiaute_3040_bytes(&bytes, &fonte, &path_str)
}

/// Lê e extrai `SCR3040_Leiaute.xls` a partir de bytes em memória.
pub fn extrair_leiaute_3040_bytes(
    bytes: &[u8],
    fonte: &str,
    path_label: &str,
) -> Result<CatalogoLeiaute3040, CatalogError> {
    let sha256 = hex::encode(Sha256::digest(bytes));
    let mut wb = open_workbook_auto_from_rs(Cursor::new(bytes)).map_err(|e| CatalogError::Xls {
        path: path_label.to_string(),
        reason: e.to_string(),
    })?;

    let doc_rows = read_sheet_cells(&mut wb, path_label, ABA_DOC3040)?;
    let campos = parse_doc3040_sheet(&doc_rows)?;

    let anexo_rows = read_sheet_cells(&mut wb, path_label, ABA_ANEXO)?;
    let anexos = parse_anexo_sheet(&anexo_rows)?;

    let anexo26_rows = read_sheet_cells(&mut wb, path_label, ABA_ANEXO_26)?;
    let anexo26_infos_adicionais = parse_anexo26_sheet(&anexo26_rows)?;

    let hist_rows = read_sheet_cells(&mut wb, path_label, ABA_HISTORICO)?;
    let historico_atualizacoes = parse_historico_sheet(ABA_HISTORICO, &hist_rows)?;

    let total_elementos = campos.iter().filter(|c| c.atributo.is_none()).count();
    let total_atributos = campos.iter().filter(|c| c.atributo.is_some()).count();

    Ok(CatalogoLeiaute3040 {
        cadoc: "3040".to_string(),
        fonte: fonte.to_string(),
        sha256,
        contagens: ContagensLeiaute {
            total_campos: campos.len(),
            total_elementos,
            total_atributos,
            total_anexos: anexos.len(),
            total_anexo26_itens: anexo26_infos_adicionais.len(),
            total_historico: historico_atualizacoes.len(),
        },
        campos,
        anexos,
        anexo26_infos_adicionais,
        historico_atualizacoes,
    })
}

/// Serializa qualquer catálogo em JSON determinístico (`pretty` + newline final).
pub fn to_json_deterministico<T: Serialize>(catalogo: &T) -> Result<String, CatalogError> {
    let mut json = serde_json::to_string_pretty(catalogo).map_err(CatalogError::Json)?;
    json.push('\n');
    Ok(json)
}

fn file_name_or_path(path: &Path, fallback: &str) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| fallback.to_string())
}

fn read_sheet_cells<R: std::io::Read + std::io::Seek>(
    wb: &mut calamine::Sheets<R>,
    path_label: &str,
    sheet_name: &str,
) -> Result<Vec<Vec<Data>>, CatalogError> {
    let range = wb.worksheet_range(sheet_name).map_err(|e| match e {
        calamine::Error::Msg(_) => CatalogError::SheetAusente {
            path: path_label.to_string(),
            sheet: sheet_name.to_string(),
        },
        other @ (calamine::Error::Io(_)
        | calamine::Error::Ods(_)
        | calamine::Error::Xls(_)
        | calamine::Error::Xlsb(_)
        | calamine::Error::Xlsx(_)
        | calamine::Error::Vba(_)
        | calamine::Error::De(_)) => CatalogError::Xls {
            path: path_label.to_string(),
            reason: format!("aba {sheet_name:?}: {other}"),
        },
    })?;
    Ok(range.rows().map(|r| r.to_vec()).collect())
}

// =========================================================================
// Críticas parser
// =========================================================================

struct CriticasColunas {
    habilitado: usize,
    codigo: usize,
    titulo: usize,
    mensagem: usize,
    alerta: Option<usize>,
    erro: Option<usize>,
    vigente_ate: Option<usize>,
}

fn parse_criticas_sheet(
    sheet_name: &str,
    rows: &[Vec<Data>],
) -> Result<Vec<Critica3040>, CatalogError> {
    let (header_idx, cols) = find_criticas_header(sheet_name, rows)?;
    let mut out = Vec::new();

    for (offset, row) in rows[header_idx + 1..].iter().enumerate() {
        let linha = header_idx + 2 + offset; // 1-based Excel row
        if row.iter().all(cell_is_empty) {
            continue;
        }
        let hab_raw = cell_text(row, cols.habilitado);
        let codigo_raw = cell_text(row, cols.codigo);
        if hab_raw.trim().is_empty() && codigo_raw.trim().is_empty() {
            continue;
        }

        let habilitado = Habilitado::parse(&hab_raw, sheet_name, linha)?;
        let codigo = codigo_raw.trim().to_string();
        if codigo.is_empty() {
            return Err(CatalogError::ValorInvalido {
                sheet: sheet_name.to_string(),
                linha,
                reason: "coluna Código vazia em linha não vazia".to_string(),
            });
        }

        let titulo_raw = cell_text(row, cols.titulo);
        let mensagem_raw = cell_text(row, cols.mensagem);
        let alerta_desde = cols
            .alerta
            .and_then(|idx| row.get(idx))
            .and_then(parse_data_vigencia_cell);
        let erro_desde = cols
            .erro
            .and_then(|idx| row.get(idx))
            .and_then(parse_data_vigencia_cell);
        let inicio_vigencia = menor_data(alerta_desde.as_deref(), erro_desde.as_deref());

        let vig_raw = cols
            .vigente_ate
            .map(|idx| cell_text(row, idx))
            .unwrap_or_default();

        let mut nr_markers = Vec::new();
        collect_nr_markers(&vig_raw, &mut nr_markers);
        collect_nr_markers(&titulo_raw, &mut nr_markers);
        collect_nr_markers(&mensagem_raw, &mut nr_markers);

        let fim_vigencia = parse_vigente_ate(&vig_raw);
        let nota_nr = if nr_markers.is_empty() {
            None
        } else {
            Some(nr_markers.join("; "))
        };

        out.push(Critica3040 {
            codigo,
            habilitado,
            categoria_aba: sheet_name.to_string(),
            inicio_vigencia,
            fim_vigencia,
            alerta_desde,
            erro_desde,
            titulo: titulo_raw.trim().to_string(),
            mensagem: mensagem_raw.trim().to_string(),
            nota_nr,
        });
    }
    Ok(out)
}

fn find_criticas_header(
    sheet_name: &str,
    rows: &[Vec<Data>],
) -> Result<(usize, CriticasColunas), CatalogError> {
    for (idx, row) in rows.iter().take(15).enumerate() {
        let keys: Vec<String> = row
            .iter()
            .map(|c| normalize_key(&cell_to_string(c)))
            .collect();
        let habilitado = keys.iter().position(|k| k == "habilitado");
        let codigo = keys.iter().position(|k| k == "codigo");
        if let (Some(hab_idx), Some(cod_idx)) = (habilitado, codigo) {
            let titulo = keys.iter().position(|k| k == "regra").ok_or_else(|| {
                CatalogError::HeaderInvalido {
                    sheet: sheet_name.to_string(),
                    reason: "coluna 'Regra' ausente no cabeçalho".to_string(),
                }
            })?;
            let mensagem = keys.iter().position(|k| k == "descricao").ok_or_else(|| {
                CatalogError::HeaderInvalido {
                    sheet: sheet_name.to_string(),
                    reason: "coluna 'Descrição' ausente no cabeçalho".to_string(),
                }
            })?;
            let alerta = keys.iter().position(|k| k == "alerta");
            let erro = keys.iter().position(|k| k == "erro");
            let vigente_ate = keys.iter().position(|k| k == "vigente ate");
            return Ok((
                idx,
                CriticasColunas {
                    habilitado: hab_idx,
                    codigo: cod_idx,
                    titulo,
                    mensagem,
                    alerta,
                    erro,
                    vigente_ate,
                },
            ));
        }
    }
    Err(CatalogError::HeaderInvalido {
        sheet: sheet_name.to_string(),
        reason: "cabeçalho com colunas 'Habilitado?' e 'Código' não encontrado".to_string(),
    })
}

// =========================================================================
// HistoricoAtualizacoes parser
// =========================================================================

fn parse_historico_sheet(
    sheet_name: &str,
    rows: &[Vec<Data>],
) -> Result<Vec<HistoricoAtualizacao>, CatalogError> {
    let mut header_idx = None;
    let mut data_col = 0;
    let mut atual_col = 1;
    let mut det_col = 2;
    let mut obs_col = 3;

    for (idx, row) in rows.iter().take(10).enumerate() {
        let keys: Vec<String> = row
            .iter()
            .map(|c| normalize_key(&cell_to_string(c)))
            .collect();
        if let (Some(d), Some(a)) = (
            keys.iter().position(|k| k == "data"),
            keys.iter().position(|k| k == "atualizacao"),
        ) {
            header_idx = Some(idx);
            data_col = d;
            atual_col = a;
            det_col = keys.iter().position(|k| k == "detalhamento").unwrap_or(2);
            obs_col = keys.iter().position(|k| k == "observacoes").unwrap_or(3);
            break;
        }
    }

    let h_idx = header_idx.ok_or_else(|| CatalogError::HeaderInvalido {
        sheet: sheet_name.to_string(),
        reason: "cabeçalho com colunas 'Data' e 'Atualização' não encontrado".to_string(),
    })?;

    let mut out = Vec::new();
    for row in &rows[h_idx + 1..] {
        if row.iter().all(cell_is_empty) {
            continue;
        }
        let data = row.get(data_col).and_then(parse_data_historico_cell);
        let atualizacao = cell_text(row, atual_col).trim().to_string();
        let detalhamento = cell_text(row, det_col).trim().to_string();
        let obs_raw = cell_text(row, obs_col).trim().to_string();
        let observacoes = if obs_raw.is_empty() {
            None
        } else {
            Some(obs_raw)
        };
        if data.is_none()
            && atualizacao.is_empty()
            && detalhamento.is_empty()
            && observacoes.is_none()
        {
            continue;
        }
        out.push(HistoricoAtualizacao {
            data,
            atualizacao,
            detalhamento,
            observacoes,
        });
    }
    Ok(out)
}

// =========================================================================
// Leiaute Doc3040 parser
// =========================================================================

fn parse_doc3040_sheet(rows: &[Vec<Data>]) -> Result<Vec<CampoLeiaute>, CatalogError> {
    let mut campos: Vec<CampoLeiaute> = Vec::new();
    let mut secao_principal = String::new();
    let mut subsecao: Option<String> = None;
    let mut tag_atual = String::new();
    let mut secao_nrs: Vec<String> = Vec::new();
    let mut subsecao_nrs: Vec<String> = Vec::new();
    let mut secao_descont: Option<String> = None;
    let mut subsecao_descont: Option<String> = None;

    for row in rows {
        let campo = cell_text(row, 0).trim().to_string();
        let formato = cell_text(row, 1).trim().to_string();
        let obrig = cell_text(row, 2).trim().to_string();
        let elem = cell_text(row, 3).trim().to_string();
        let attr = cell_text(row, 4).trim().to_string();
        let desc = cell_text(row, 5).trim().to_string();
        let fidcs = cell_text(row, 6).trim().to_string();
        let obs = cell_text(row, 7).trim().to_string();

        if campo.is_empty()
            && formato.is_empty()
            && obrig.is_empty()
            && elem.is_empty()
            && attr.is_empty()
            && desc.is_empty()
            && fidcs.is_empty()
            && obs.is_empty()
        {
            continue;
        }

        // Cabeçalho de tabela repetido por seção:
        if normalize_key(&campo) == "campo" && normalize_key(&formato) == "formato" {
            continue;
        }

        // Linha de título de seção / subseção / rodapé de cardinalidade (só coluna 0 preenchida):
        if !campo.is_empty()
            && formato.is_empty()
            && obrig.is_empty()
            && elem.is_empty()
            && attr.is_empty()
            && desc.is_empty()
            && fidcs.is_empty()
            && obs.is_empty()
        {
            if is_doc3040_title_or_footer(&campo) {
                continue;
            }
            if is_subsection_heading(&campo) {
                let mut nrs = Vec::new();
                collect_nr_markers(&campo, &mut nrs);
                subsecao_nrs = nrs;
                subsecao_descont = extract_descontinuado(&campo);
                subsecao = Some(campo);
            } else {
                let mut nrs = Vec::new();
                collect_nr_markers(&campo, &mut nrs);
                secao_nrs = nrs;
                secao_descont = extract_descontinuado(&campo);
                secao_principal = campo;
                subsecao = None;
                subsecao_nrs.clear();
                subsecao_descont = None;
            }
            continue;
        }

        // Linha de continuação (apenas FIDCs / Observações preenchidos):
        if campo.is_empty()
            && formato.is_empty()
            && obrig.is_empty()
            && elem.is_empty()
            && attr.is_empty()
            && desc.is_empty()
        {
            if let Some(prev) = campos.last_mut() {
                if !fidcs.is_empty() {
                    append_line(&mut prev.fidcs, &fidcs);
                    collect_nr_markers(&fidcs, &mut prev.notas_nr);
                }
                if !obs.is_empty() {
                    append_line(&mut prev.observacoes, &obs);
                    collect_nr_markers(&obs, &mut prev.notas_nr);
                }
            }
            continue;
        }

        if !elem.is_empty() {
            tag_atual = elem.clone();
        }
        if tag_atual.is_empty() {
            return Err(CatalogError::HeaderInvalido {
                sheet: ABA_DOC3040.to_string(),
                reason: format!("campo {campo:?} sem elemento XML ativo"),
            });
        }

        let tag_pai = resolver_tag_pai(&tag_atual, &secao_principal).map(str::to_string);
        let atributo = if attr.is_empty() {
            None
        } else {
            Some(attr.clone())
        };
        let atributos = if attr.is_empty() {
            Vec::new()
        } else {
            parse_atributos_list(&attr)
        };
        let caminho_xml = build_caminho_xml(&tag_atual, tag_pai.as_deref(), &atributos);

        let mut dominio_anexos = Vec::new();
        collect_anexo_refs(&desc, &mut dominio_anexos);
        collect_anexo_refs(&obs, &mut dominio_anexos);
        collect_anexo_refs(&obrig, &mut dominio_anexos);
        collect_anexo_refs(&attr, &mut dominio_anexos);

        let mut notas_nr = secao_nrs.clone();
        for nr in &subsecao_nrs {
            push_unique(&mut notas_nr, nr);
        }
        collect_nr_markers(&campo, &mut notas_nr);
        collect_nr_markers(&desc, &mut notas_nr);
        collect_nr_markers(&obs, &mut notas_nr);
        collect_nr_markers(&fidcs, &mut notas_nr);
        collect_nr_markers(&attr, &mut notas_nr);

        let descontinuado = extract_descontinuado(&desc)
            .or_else(|| extract_descontinuado(&obs))
            .or_else(|| subsecao_descont.clone())
            .or_else(|| secao_descont.clone());

        let secao = match &subsecao {
            Some(sub) if !secao_principal.is_empty() => format!("{secao_principal} / {sub}"),
            Some(sub) => sub.clone(),
            None => secao_principal.clone(),
        };

        let obrigatorio_categoria = classify_obrigatoriedade(&obrig);
        let ordem = campos.len() + 1;
        campos.push(CampoLeiaute {
            ordem,
            secao,
            tag: tag_atual.clone(),
            tag_pai,
            caminho_xml,
            atributo,
            atributos,
            campo,
            tipo: non_empty_opt(formato),
            obrigatoriedade: non_empty_opt(obrig),
            obrigatorio_categoria,
            dominio_anexos,
            descricao: desc,
            fidcs: non_empty_opt(fidcs),
            observacoes: non_empty_opt(obs),
            notas_nr,
            descontinuado,
        });
    }

    if campos.is_empty() {
        return Err(CatalogError::HeaderInvalido {
            sheet: ABA_DOC3040.to_string(),
            reason: "nenhum campo extraído da aba Doc3040".to_string(),
        });
    }
    Ok(campos)
}

fn is_doc3040_title_or_footer(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with("Documento 3040 -") || trimmed.starts_with("Cada ")
}

fn is_subsection_heading(text: &str) -> bool {
    let cleaned = strip_leading_nr_prefix(text.trim());
    let mut chars = cleaned.chars();
    match (chars.next(), chars.next(), chars.next()) {
        (Some(c), Some('.'), Some(' ')) => c.is_ascii_lowercase(),
        _ => false,
    }
}

fn strip_leading_nr_prefix(s: &str) -> &str {
    let trimmed = s.trim_start();
    if let Some(rest) = trimmed.strip_prefix("(NR")
        && let Some(end) = rest.find(')')
    {
        return rest[end + 1..].trim_start();
    }
    trimmed
}

fn resolver_tag_pai(tag: &str, secao_principal: &str) -> Option<&'static str> {
    match tag {
        "xml" | "Doc3040" => None,
        "Cli" | "Agreg" => Some("Doc3040"),
        "Op" => Some("Cli"),
        "Gar" | "Inf" | "Sicor" | "ContInstFin" | "ConIpocs" | "ContInstFinRes4966" => Some("Op"),
        "ipocCon" => Some("ConIpocs"),
        "Estagio" | "Perda" => Some("ContInstFinRes4966"),
        "Venc" => {
            if secao_principal.contains("Agreg")
                || secao_principal.to_ascii_lowercase().contains("agrega")
            {
                Some("Agreg")
            } else {
                Some("Op")
            }
        }
        _ => None,
    }
}

fn build_caminho_xml(tag: &str, tag_pai: Option<&str>, atributos: &[String]) -> String {
    let base = match tag {
        "xml" => "xml".to_string(),
        "Doc3040" => "Doc3040".to_string(),
        "Cli" => "Doc3040/Cli".to_string(),
        "Op" => "Doc3040/Cli/Op".to_string(),
        "Agreg" => "Doc3040/Agreg".to_string(),
        "Gar" | "Inf" | "Sicor" | "ContInstFin" | "ConIpocs" | "ContInstFinRes4966" => {
            format!("Doc3040/Cli/Op/{tag}")
        }
        "ipocCon" => "Doc3040/Cli/Op/ConIpocs/ipocCon".to_string(),
        "Estagio" | "Perda" => format!("Doc3040/Cli/Op/ContInstFinRes4966/{tag}"),
        "Venc" => match tag_pai {
            Some("Agreg") => "Doc3040/Agreg/Venc".to_string(),
            _ => "Doc3040/Cli/Op/Venc".to_string(),
        },
        other => match tag_pai {
            Some(p) => format!("{p}/{other}"),
            None => other.to_string(),
        },
    };
    match atributos.len() {
        0 => base,
        1 => format!("{base}/@{}", atributos[0]),
        _ => format!("{base}/@[{}]", atributos.join(",")),
    }
}

fn parse_atributos_list(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('*') {
            continue;
        }
        // Ex.: "Tp (concatenado)" -> "Tp"
        let before_paren = trimmed.split('(').next().unwrap_or(trimmed).trim();
        for part in before_paren.split(',') {
            let tok = part.trim().trim_end_matches('*').trim();
            if !tok.is_empty() {
                push_unique(&mut out, tok);
            }
        }
    }
    out
}

fn classify_obrigatoriedade(raw: &str) -> ObrigatoriedadeTipo {
    let t = raw.trim();
    if t.is_empty() {
        return ObrigatoriedadeTipo::NaoInformado;
    }
    let norm = normalize_key(t);
    if norm == "sim" {
        ObrigatoriedadeTipo::Sim
    } else if norm == "nao" {
        ObrigatoriedadeTipo::Nao
    } else {
        ObrigatoriedadeTipo::Condicional
    }
}

// =========================================================================
// Anexo & Anexo 26 parsers
// =========================================================================

fn parse_anexo_sheet(rows: &[Vec<Data>]) -> Result<Vec<AnexoDominio>, CatalogError> {
    let mut anexos: Vec<AnexoDominio> = Vec::new();
    let mut current: Option<AnexoDominio> = None;
    let mut in_table = false;
    let mut has_sub_cols = false;
    let mut last_dom = String::new();
    let mut last_desc = String::new();

    for row in rows {
        let c0 = cell_text(row, 0).trim().to_string();
        let c1 = cell_text(row, 1).trim().to_string();
        let c2 = cell_text(row, 2).trim().to_string();
        let c3 = cell_text(row, 3).trim().to_string();
        let c4 = cell_text(row, 4).trim().to_string();

        if c0.is_empty() && c1.is_empty() && c2.is_empty() && c3.is_empty() && c4.is_empty() {
            if let Some(anexo) = current.take() {
                anexos.push(anexo);
            }
            in_table = false;
            has_sub_cols = false;
            last_dom.clear();
            last_desc.clear();
            continue;
        }

        if let Some((numero, titulo_limpo)) = parse_anexo_heading(&c0) {
            if let Some(anexo) = current.take() {
                anexos.push(anexo);
            }
            let mut notas_nr = Vec::new();
            collect_nr_markers(&c0, &mut notas_nr);
            let vigencia = extract_vigencia(&c0);
            let descontinuado = extract_descontinuado(&c0);
            current = Some(AnexoDominio {
                numero,
                id: format!("Anexo {numero}"),
                titulo: titulo_limpo,
                notas_nr,
                vigencia,
                descontinuado,
                itens: Vec::new(),
            });
            in_table = false;
            has_sub_cols = false;
            last_dom.clear();
            last_desc.clear();
            continue;
        }

        if let Some(ref mut anexo) = current {
            if !in_table && normalize_key(&c0) == "dominio" {
                in_table = true;
                let k2 = normalize_key(&c2);
                has_sub_cols = k2 == "sub" || k2 == "subdominio";
                continue;
            }
            if !in_table {
                continue;
            }

            let (dom, dom_desc, subcod, subdesc) = if has_sub_cols {
                if !c0.is_empty() {
                    last_dom = c0.clone();
                    last_desc = c1.clone();
                } else if !c1.is_empty() {
                    last_desc = c1.clone();
                }
                (
                    last_dom.clone(),
                    last_desc.clone(),
                    non_empty_opt(c2),
                    non_empty_opt(c3),
                )
            } else {
                (c0, c1, None, None)
            };

            let mut notas_nr = Vec::new();
            collect_nr_markers(&dom_desc, &mut notas_nr);
            if let Some(ref sd) = subdesc {
                collect_nr_markers(sd, &mut notas_nr);
            }
            let descontinuado = extract_descontinuado(&dom_desc)
                .or_else(|| subdesc.as_deref().and_then(extract_descontinuado));

            anexo.itens.push(AnexoItem {
                codigo: dom,
                descricao: dom_desc,
                subcodigo: subcod,
                subdescricao: subdesc,
                identificador: non_empty_opt(c4),
                notas_nr,
                descontinuado,
            });
        }
    }

    if let Some(anexo) = current.take() {
        anexos.push(anexo);
    }
    if anexos.is_empty() {
        return Err(CatalogError::HeaderInvalido {
            sheet: ABA_ANEXO.to_string(),
            reason: "nenhum Anexo encontrado na aba Anexo".to_string(),
        });
    }
    Ok(anexos)
}

fn parse_anexo_heading(raw: &str) -> Option<(u32, String)> {
    let trimmed = raw.trim();
    let idx = trimmed.find("Anexo ")?;
    let after = &trimmed[idx + "Anexo ".len()..];
    let colon_pos = after.find(':')?;
    let num_str = after[..colon_pos].trim();
    if num_str.is_empty() || !num_str.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let num: u32 = num_str.parse().ok()?;
    let title = after[colon_pos + 1..].trim().to_string();
    Some((num, title))
}

fn parse_anexo26_sheet(rows: &[Vec<Data>]) -> Result<Vec<InfoAdicionalAnexo26>, CatalogError> {
    let header_idx = rows
        .iter()
        .take(10)
        .position(|row| normalize_key(&cell_text(row, 0)) == "<tp>")
        .ok_or_else(|| CatalogError::HeaderInvalido {
            sheet: ABA_ANEXO_26.to_string(),
            reason: "cabeçalho '<Tp>' não encontrado na aba Anexo 26".to_string(),
        })?;

    let mut out = Vec::new();
    let mut cur_grupo = String::new();
    let mut cur_grupo_desc = String::new();

    for row in &rows[header_idx + 1..] {
        if row.iter().all(cell_is_empty) {
            continue;
        }
        let c0 = cell_text(row, 0).trim().to_string();
        let c1 = cell_text(row, 1).trim().to_string();
        let c2 = cell_text(row, 2).trim().to_string();
        let c3 = cell_text(row, 3).trim().to_string();
        let c4 = cell_text(row, 4).trim().to_string();
        let c5 = cell_text(row, 5).trim().to_string();
        let c6 = cell_text(row, 6).trim().to_string();
        let c7 = cell_text(row, 7).trim().to_string();
        let c8 = cell_text(row, 8).trim().to_string();

        if !c0.is_empty() {
            cur_grupo = c0;
            cur_grupo_desc = c1;
        } else if !c1.is_empty() {
            cur_grupo_desc = c1;
        }

        if c2.is_empty() && c3.is_empty() {
            continue;
        }

        let mut notas_nr = Vec::new();
        collect_nr_markers(&cur_grupo_desc, &mut notas_nr);
        collect_nr_markers(&c3, &mut notas_nr);
        collect_nr_markers(&c4, &mut notas_nr);
        collect_nr_markers(&c5, &mut notas_nr);
        collect_nr_markers(&c6, &mut notas_nr);
        collect_nr_markers(&c7, &mut notas_nr);
        collect_nr_markers(&c8, &mut notas_nr);

        let descontinuado =
            extract_descontinuado(&c3).or_else(|| extract_descontinuado(&cur_grupo_desc));

        out.push(InfoAdicionalAnexo26 {
            tp_grupo: cur_grupo.clone(),
            tp_grupo_descricao: cur_grupo_desc.clone(),
            subtipo: c2,
            subtipo_descricao: c3,
            cd: non_empty_opt(c4),
            ident: non_empty_opt(c5),
            valor: non_empty_opt(c6),
            perc: non_empty_opt(c7),
            qtd: non_empty_opt(c8),
            notas_nr,
            descontinuado,
        });
    }
    Ok(out)
}

// =========================================================================
// Zero-float cell conversion & date / marker helpers
// =========================================================================

fn cell_is_empty(cell: &Data) -> bool {
    match cell {
        Data::Empty => true,
        Data::String(s) => s.trim().is_empty(),
        Data::Bool(_)
        | Data::Error(_)
        | Data::Int(_)
        | Data::Float(_)
        | Data::DateTime(_)
        | Data::DateTimeIso(_)
        | Data::DurationIso(_) => false,
    }
}

fn cell_text(row: &[Data], col: usize) -> String {
    row.get(col).map(cell_to_string).unwrap_or_default()
}

/// Conversão exaustiva e zero-float de `calamine::Data` para `String`.
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Bool(b) => b.to_string(),
        Data::Error(e) => format!("#ERRO:{e:?}"),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f64_to_string_sem_float(*f),
        Data::DateTime(dt) => excel_datetime_to_iso(dt),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
    }
}

/// Renderiza `f64` sem operações, comparações ou casts com float:
/// decodifica os bits IEEE-754 em `u64` para detectar inteiros exatos.
fn f64_to_string_sem_float(f: f64) -> String {
    let bits = f.to_bits();
    let sign = (bits >> 63) != 0;
    let exp_bits = ((bits >> 52) & 0x7ff) as u32;
    let frac = bits & 0x000f_ffff_ffff_ffff;

    if exp_bits == 0x7ff {
        return f.to_string();
    }
    if exp_bits == 0 && frac == 0 {
        return "0".to_string();
    }
    if let Some(unbiased) = exp_bits.checked_sub(1023)
        && unbiased <= 52
    {
        let mant = (1u64 << 52) | frac;
        let shift = 52 - unbiased;
        let mask = (1u64 << shift) - 1;
        if (mant & mask) == 0 {
            let int_val = mant >> shift;
            return if sign {
                format!("-{int_val}")
            } else {
                int_val.to_string()
            };
        }
    }
    f.to_string()
}

fn excel_datetime_to_iso(dt: &ExcelDateTime) -> String {
    let (year, month, day, hour, min, sec, milli) = dt.to_ymd_hms_milli();
    if hour == 0 && min == 0 && sec == 0 && milli == 0 {
        format!("{year:04}-{month:02}-{day:02}")
    } else {
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}.{milli:03}")
    }
}

/// Converte serial inteiro Excel (sistema 1900, >= 1900-03-01) em `AAAA-MM-DD`
/// usando apenas aritmética inteira (algoritmo civil de Howard Hinnant).
fn excel_serial_days_to_iso(serial: u32) -> Option<String> {
    if !(61..=2_958_465).contains(&serial) {
        return None;
    }
    let days = i64::from(serial) - 25_569;
    let (year, month, day) = unix_days_to_ymd(days)?;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

fn unix_days_to_ymd(days: i64) -> Option<(i64, u32, u32)> {
    let z = days.checked_add(719_468)?;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = u64::try_from(z - era * 146_097).ok()?;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = i64::try_from(yoe).ok()? + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    Some((year, u32::try_from(m).ok()?, u32::try_from(d).ok()?))
}

fn parse_data_vigencia_cell(cell: &Data) -> Option<String> {
    match cell {
        Data::Empty => None,
        Data::DateTime(dt) => Some(excel_datetime_to_iso(dt)),
        Data::Int(i) => u32::try_from(*i).ok().and_then(excel_serial_days_to_iso),
        Data::Float(f) => {
            let s = f64_to_string_sem_float(*f);
            s.parse::<u32>().ok().and_then(excel_serial_days_to_iso)
        }
        Data::String(s) => parse_vigencia_str(s),
        Data::DateTimeIso(s) | Data::DurationIso(s) => parse_vigencia_str(s),
        Data::Bool(_) | Data::Error(_) => None,
    }
}

fn parse_data_historico_cell(cell: &Data) -> Option<String> {
    match cell {
        Data::Empty => None,
        Data::DateTime(dt) => Some(excel_datetime_to_iso(dt)),
        Data::Int(i) => u32::try_from(*i).ok().and_then(excel_serial_days_to_iso),
        Data::Float(f) => {
            let s = f64_to_string_sem_float(*f);
            s.parse::<u32>()
                .ok()
                .and_then(excel_serial_days_to_iso)
                .or(Some(s))
        }
        Data::String(s) | Data::DateTimeIso(s) | Data::DurationIso(s) => {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        }
        Data::Bool(_) | Data::Error(_) => None,
    }
}

fn parse_vigencia_str(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() || t == "-" || t.eq_ignore_ascii_case("n/a") {
        return None;
    }
    if is_iso_date(t) {
        return Some(t.to_string());
    }
    if let Some((mmm, yy)) = t.split_once('/') {
        if let Some(mm) = pt_month_num(mmm.trim()) {
            let y_str = yy.trim();
            if y_str.len() == 2 && y_str.bytes().all(|b| b.is_ascii_digit()) {
                return Some(format!("20{y_str}-{mm:02}-01"));
            }
            if y_str.len() == 4 && y_str.bytes().all(|b| b.is_ascii_digit()) {
                return Some(format!("{y_str}-{mm:02}-01"));
            }
        }
        if let Some(mm_yy) = normalize_mm_yyyy(mmm.trim(), yy.trim()) {
            return Some(mm_yy);
        }
    }
    if let Ok(serial) = t.parse::<u32>()
        && let Some(iso) = excel_serial_days_to_iso(serial)
    {
        return Some(iso);
    }
    Some(t.to_string())
}

fn parse_vigente_ate(raw: &str) -> Option<String> {
    let cleaned = strip_nr_markers(raw);
    let t = cleaned.trim();
    if t.is_empty() || t == "-" || t.eq_ignore_ascii_case("n/a") {
        return None;
    }
    if let Some((mm, yyyy)) = t.split_once('/')
        && let Some(iso) = normalize_mm_yyyy(mm.trim(), yyyy.trim())
    {
        return Some(iso);
    }
    Some(t.to_string())
}

fn normalize_mm_yyyy(mm: &str, yyyy: &str) -> Option<String> {
    if mm.len() != 2
        || yyyy.len() != 4
        || !mm.bytes().all(|b| b.is_ascii_digit())
        || !yyyy.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let m: u8 = mm.parse().ok()?;
    if !(1..=12).contains(&m) {
        return None;
    }
    Some(format!("{yyyy}-{mm}"))
}

fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..10].iter().all(u8::is_ascii_digit)
}

fn pt_month_num(mmm: &str) -> Option<u8> {
    match mmm.to_ascii_lowercase().as_str() {
        "jan" => Some(1),
        "fev" => Some(2),
        "mar" => Some(3),
        "abr" => Some(4),
        "mai" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "ago" => Some(8),
        "set" => Some(9),
        "out" => Some(10),
        "nov" => Some(11),
        "dez" => Some(12),
        _ => None,
    }
}

fn menor_data(a: Option<&str>, b: Option<&str>) -> Option<String> {
    match (a, b) {
        (Some(da), Some(db)) => Some(if da <= db {
            da.to_string()
        } else {
            db.to_string()
        }),
        (Some(da), None) => Some(da.to_string()),
        (None, Some(db)) => Some(db.to_string()),
        (None, None) => None,
    }
}

/// Coleta marcadores `(NR)`, `(NR1)`, `(NR2)`, ..., incluindo typos sem `)` como `(NR1 Anexo 48`.
fn collect_nr_markers(text: &str, dest: &mut Vec<String>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 3 <= bytes.len() {
        if bytes[i] == b'(' && bytes[i + 1] == b'N' && bytes[i + 2] == b'R' {
            let mut j = i + 3;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            let closed_or_boundary =
                j == bytes.len() || bytes[j] == b')' || bytes[j].is_ascii_whitespace();
            if closed_or_boundary {
                let digits = &text[i + 3..j];
                let marker = format!("(NR{digits})");
                push_unique(dest, &marker);
                i = if j < bytes.len() && bytes[j] == b')' {
                    j + 1
                } else {
                    j
                };
                continue;
            }
        }
        i += 1;
    }
}

fn strip_nr_markers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        if let Some(after_nr) = rest.strip_prefix("(NR") {
            let digit_len = after_nr.bytes().take_while(u8::is_ascii_digit).count();
            let after_digits = &after_nr[digit_len..];
            if after_digits.starts_with(')') {
                i += 3 + digit_len + 1;
                continue;
            }
            if after_digits.is_empty()
                || after_digits.chars().next().is_some_and(char::is_whitespace)
            {
                i += 3 + digit_len;
                continue;
            }
        }
        if let Some(ch) = rest.chars().next() {
            out.push(ch);
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    out
}

/// Coleta referências `"Anexo N"` presentes em textos do leiaute.
fn collect_anexo_refs(text: &str, dest: &mut Vec<String>) {
    let lower = text.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(rel_idx) = lower[search_from..].find("anexo") {
        let abs_idx = search_from + rel_idx + "anexo".len();
        let after = &text[abs_idx..];
        let ws_len = after.bytes().take_while(|b| *b == b' ').count();
        let num_slice = &after[ws_len..];
        let digits_len = num_slice.bytes().take_while(u8::is_ascii_digit).count();
        if digits_len > 0 {
            let num = &num_slice[..digits_len];
            push_unique(dest, &format!("Anexo {num}"));
            search_from = abs_idx + ws_len + digits_len;
        } else {
            search_from = abs_idx;
        }
    }
}

fn extract_descontinuado(text: &str) -> Option<String> {
    extract_pipe_or_phrase(text, "descontinuado")
}

fn extract_vigencia(text: &str) -> Option<String> {
    extract_pipe_or_phrase(text, "vigência").or_else(|| extract_pipe_or_phrase(text, "vigencia"))
}

fn extract_pipe_or_phrase(text: &str, keyword_lower: &str) -> Option<String> {
    for seg in text.split('|') {
        let trimmed = seg.trim();
        if trimmed.to_lowercase().contains(keyword_lower) {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn normalize_key(s: &str) -> String {
    let stripped = strip_accents(s);
    let collapsed: String = stripped
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    collapsed.trim_end_matches('?').trim().to_string()
}

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

fn non_empty_opt(s: String) -> Option<String> {
    if s.trim().is_empty() { None } else { Some(s) }
}

fn append_line(slot: &mut Option<String>, addition: &str) {
    match slot {
        Some(existing) if !existing.is_empty() => {
            existing.push('\n');
            existing.push_str(addition);
        }
        Some(existing) => existing.push_str(addition),
        None => *slot = Some(addition.to_string()),
    }
}

fn push_unique(dest: &mut Vec<String>, item: &str) {
    if !dest.iter().any(|existing| existing == item) {
        dest.push(item.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn criticas_xls_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../3040/SCR3040_Criticas.xls")
    }

    fn leiaute_xls_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../3040/SCR3040_Leiaute.xls")
    }

    #[test]
    fn criticas_3040_contagens_exatas_e_regras_chave() -> Result<(), CatalogError> {
        let path = criticas_xls_path();
        assert!(path.exists(), "fixture oficial ausente: {}", path.display());

        let cat = extrair_criticas_3040(&path)?;
        assert_eq!(cat.cadoc, "3040");
        assert_eq!(
            cat.sha256,
            "163b679f018a2215bc062970e250863921c31c8bc3a4ecf4d788a0c14a355b6b"
        );
        assert_eq!(cat.contagens.total, 255);
        assert_eq!(cat.contagens.habilitado_s, 226);
        assert_eq!(cat.contagens.habilitado_n, 29);
        assert_eq!(cat.criticas.len(), 255);

        let expected_cats: &[(&str, usize, usize, usize)] = &[
            ("Básicas", 19, 19, 0),
            ("Campos Obrigatórios", 81, 79, 2),
            ("Formato", 5, 4, 1),
            ("Semântica", 111, 97, 14),
            ("Individualizadas", 15, 9, 6),
            ("Agregadas", 14, 8, 6),
            ("Negócio", 1, 1, 0),
            ("Substituição Parcial", 9, 9, 0),
        ];
        assert_eq!(cat.contagens.por_categoria.len(), expected_cats.len());
        for &(name, tot, s, n) in expected_cats {
            let c = cat.contagens.por_categoria.get(name).ok_or_else(|| {
                CatalogError::HeaderInvalido {
                    sheet: name.to_string(),
                    reason: "categoria ausente".to_string(),
                }
            })?;
            assert_eq!(
                (c.total, c.habilitado_s, c.habilitado_n),
                (tot, s, n),
                "{name}"
            );
        }

        // Presença e fidelidade de B12, C56b, S118, C71
        let find = |code: &str| -> Result<&Critica3040, CatalogError> {
            cat.criticas
                .iter()
                .find(|r| r.codigo == code)
                .ok_or_else(|| CatalogError::ValorInvalido {
                    sheet: "criticas".to_string(),
                    linha: 0,
                    reason: format!("regra {code} ausente"),
                })
        };

        let b12 = find("B12")?;
        assert_eq!(b12.habilitado, Habilitado::S);
        assert_eq!(b12.categoria_aba, "Básicas");
        assert!(b12.mensagem.contains("TpFundo"));
        assert!(b12.mensagem.contains("36"));

        let c56b = find("C56b")?;
        assert_eq!(c56b.habilitado, Habilitado::S);
        assert_eq!(c56b.categoria_aba, "Campos Obrigatórios");
        assert_eq!(c56b.alerta_desde.as_deref(), Some("2026-07-01"));
        assert_eq!(c56b.erro_desde.as_deref(), Some("2026-09-01"));
        assert_eq!(c56b.inicio_vigencia.as_deref(), Some("2026-07-01"));

        let s118 = find("S118")?;
        assert_eq!(s118.habilitado, Habilitado::S);
        assert_eq!(s118.categoria_aba, "Semântica");
        assert_eq!(s118.inicio_vigencia.as_deref(), Some("2026-07-01"));
        assert!(s118.mensagem.contains("Anexo 50"));

        let c71 = find("C71")?;
        assert_eq!(c71.habilitado, Habilitado::S);
        assert_eq!(c71.categoria_aba, "Campos Obrigatórios");
        assert_eq!(c71.inicio_vigencia.as_deref(), Some("2025-01-01"));
        assert!(c71.mensagem.contains("EstInstFin"));

        let c44 = find("C44")?;
        assert_eq!(c44.fim_vigencia.as_deref(), Some("2026-05"));
        assert_eq!(c44.nota_nr.as_deref(), Some("(NR)"));

        let f03 = find("F03")?;
        assert_eq!(f03.erro_desde.as_deref(), Some("2011-01-01"));
        assert_eq!(f03.inicio_vigencia.as_deref(), Some("2011-01-01"));

        assert!(!cat.historico_atualizacoes.is_empty());
        assert!(
            cat.historico_atualizacoes
                .iter()
                .any(|h| h.atualizacao.contains("C56b") && h.atualizacao.contains("S118"))
        );

        // Determinismo: duas extrações consecutivas geram JSON idêntico byte-a-byte.
        let cat2 = extrair_criticas_3040(&path)?;
        assert_eq!(
            to_json_deterministico(&cat)?,
            to_json_deterministico(&cat2)?
        );
        Ok(())
    }

    #[test]
    fn leiaute_3040_campos_anexos_e_historico_reais() -> Result<(), CatalogError> {
        let path = leiaute_xls_path();
        assert!(path.exists(), "fixture oficial ausente: {}", path.display());

        let leiaute = extrair_leiaute_3040(&path)?;
        assert_eq!(leiaute.cadoc, "3040");
        assert_eq!(
            leiaute.sha256,
            "0eb74aa1245fd24bed37036d68c69d2045272e53f5fb1d96af336a376f1b1ba2"
        );
        assert!(!leiaute.campos.is_empty());
        assert!(!leiaute.anexos.is_empty());
        assert!(!leiaute.anexo26_infos_adicionais.is_empty());
        assert!(!leiaute.historico_atualizacoes.is_empty());

        // Campo EstInstFin em ContInstFinRes4966
        let est = leiaute
            .campos
            .iter()
            .find(|c| c.tag == "ContInstFinRes4966" && c.atributo.as_deref() == Some("EstInstFin"))
            .ok_or_else(|| CatalogError::ValorInvalido {
                sheet: ABA_DOC3040.to_string(),
                linha: 0,
                reason: "EstInstFin ausente".to_string(),
            })?;
        assert_eq!(est.tipo.as_deref(), Some("A1"));
        assert_eq!(est.obrigatorio_categoria, ObrigatoriedadeTipo::Nao);
        assert_eq!(est.dominio_anexos, vec!["Anexo 41".to_string()]);
        assert_eq!(
            est.caminho_xml,
            "Doc3040/Cli/Op/ContInstFinRes4966/@EstInstFin"
        );

        // Campo SitGar com nota (NR1) e Anexo 47
        let sit_gar = leiaute
            .campos
            .iter()
            .find(|c| c.tag == "Gar" && c.atributo.as_deref() == Some("SitGar"))
            .ok_or_else(|| CatalogError::ValorInvalido {
                sheet: ABA_DOC3040.to_string(),
                linha: 0,
                reason: "SitGar ausente".to_string(),
            })?;
        assert!(sit_gar.notas_nr.contains(&"(NR1)".to_string()));
        assert_eq!(sit_gar.dominio_anexos, vec!["Anexo 47".to_string()]);

        // Anexo 48 (heading com typo "(NR1 Anexo 48") e Anexo 51 presentes
        assert!(leiaute.anexos.iter().any(|a| a.numero == 48));
        assert!(leiaute.anexos.iter().any(|a| a.numero == 51));
        // Anexo 8 domínio 25 atualizado pela NR3
        let anexo8 = leiaute
            .anexos
            .iter()
            .find(|a| a.numero == 8)
            .ok_or_else(|| CatalogError::ValorInvalido {
                sheet: ABA_ANEXO.to_string(),
                linha: 0,
                reason: "Anexo 8 ausente".to_string(),
            })?;
        assert!(
            anexo8
                .itens
                .iter()
                .any(|i| i.codigo == "25" && i.notas_nr.contains(&"(NR3)".to_string()))
        );

        // Determinismo JSON
        let leiaute2 = extrair_leiaute_3040(&path)?;
        assert_eq!(
            to_json_deterministico(&leiaute)?,
            to_json_deterministico(&leiaute2)?
        );
        Ok(())
    }

    #[test]
    fn helpers_zero_float_e_datas() {
        assert_eq!(f64_to_string_sem_float(0.0), "0");
        assert_eq!(f64_to_string_sem_float(-0.0), "0");
        assert_eq!(f64_to_string_sem_float(20.0), "20");
        assert_eq!(f64_to_string_sem_float(40544.0), "40544");
        assert_eq!(f64_to_string_sem_float(-7.0), "-7");
        assert_eq!(f64_to_string_sem_float(1.5), "1.5");

        assert_eq!(
            excel_serial_days_to_iso(40544).as_deref(),
            Some("2011-01-01")
        );
        assert_eq!(
            excel_serial_days_to_iso(46204).as_deref(),
            Some("2026-07-01")
        );
        assert_eq!(
            excel_serial_days_to_iso(46266).as_deref(),
            Some("2026-09-01")
        );
        assert_eq!(
            parse_vigente_ate("05/2026\n(NR)").as_deref(),
            Some("2026-05")
        );
        assert_eq!(parse_vigencia_str("jan/11").as_deref(), Some("2011-01-01"));
    }
}
