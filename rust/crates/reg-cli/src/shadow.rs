//! shadow-diff — gate de desligamento do legado (v1 Go): `shadow-diff
//! --legacy <xml> --rust <xml>` compara os dois documentos, classifica cada
//! divergência e emite o **SMER** (% de equivalência semântica). Exit 0
//! acontece SOMENTE com SMER 100% — o gate que autoriza desligar o legado.
//!
//! ## SUBSET DECLARADO da canonicalização (o que é semanticamente igual)
//! Ambos os XMLs são parseados com quick-xml (mesmo idioma do `validate` e
//! do `sta`) e normalizados em dois níveis:
//!
//! 1. **Nível SEMÂNTICO** (decide o diff): declaração XML (`<?xml …?>`),
//!    comentários, `PI` e `DocType` descartados; atributos ordenados por
//!    nome; texto com runs de whitespace ASCII (espaço, `\t`, `\r`, `\n`,
//!    `\f`) colapsados em UM espaço e cortados nas pontas; `<a/>` ≡
//!    `<a></a>`; CDATA ≡ texto.
//! 2. **Nível de FORMATAÇÃO** (só alimenta a categoria COSMETIC): preserva a
//!    ordem ORIGINAL dos atributos, o texto bruto (whitespace intacto), os
//!    comentários filhos diretos, a declaração XML e a forma (`<a/>` vs
//!    `<a></a>`). Uma linha semanticamente igual cujas formas brutas
//!    divergem é classificada COSMETIC.
//!
//! Demais regras do subset (limites honestos):
//! - **Whitespace puramente inter-elemento** (pretty-print) é formatação em
//!   AMBOS os níveis — não gera linha.
//! - **NBSP (U+00A0) é DADO**, não whitespace: não é whitespace ASCII.
//! - **CDATA** entra como conteúdo CRU (sem resolução de entidades —
//!   semântica XML de CDATA); **Text** e **GeneralRef** têm entidades
//!   resolvidas (`quick_xml::escape::unescape`).
//! - **Namespaces**: o nome QUALIFICADO (prefixo incluso) é preservado — não
//!   há resolução de URI de namespace. Documentos com prefixos diferentes
//!   mas mesmo URI são considerados diferentes (o CADOC 3040 não usa
//!   namespace; declaração honesta do subset).
//! - **Encoding**: bytes lidos como UTF-8 (subset — decl com encoding
//!   diverso não reescreve a decodificação; bytes não-UTF-8 = erro tipado).
//! - **Atributo repetido** (XML mal-formado que o quick-xml aceita): a
//!   última ocorrência vence no nível semântico.
//! - **Mixed content** (texto e elementos filhos no mesmo nó): os textos
//!   são concatenados SEM preservar a ordem relativa texto↔elemento.
//! - **Alinhamento por caminho com índice de ocorrência** (`nome[i]`, i =
//!   índice entre irmãos homônimos, raiz = `[0]`): inserção/remoção no
//!   MEIO de uma lista de irmãos homônimos desloca os índices seguintes —
//!   o diff aponta drift/bug/gap nos índices seguintes em vez de um único
//!   elemento inserido. LIMITAÇÃO DECLARADA: revisão humana decide; o gate
//!   SMER 100% exige que ela aconteça de qualquer forma.
//!
//! ## Categorias do diff (1 linha = 1 caminho de elemento)
//! - **COSMETIC** — semanticamente igual; difere só em formatação (ordem de
//!   atributos, comentários, declaração, whitespace bruto, `<a/>` vs
//!   `<a></a>`). NÃO reduz o SMER.
//! - **DATA_DRIFT** — mesmo caminho, valor semântico diferente.
//! - **LEGACY_BUG** — presente no legado, ausente no Rust. NÃO reduz o SMER:
//!   por definição do gate, é defeito do v1 a documentar (as linhas ficam
//!   no relatório para auditoria), não falha de fidelidade do Rust.
//! - **RADIANT_GAP** — presente no Rust, ausente no legado.
//!
//! ## SMER e gate
//! `smer = (linhas_da_união − DATA_DRIFT − RADIANT_GAP) / linhas_da_união`,
//! com 2 casas calculadas em ARITMÉTICA INTEIRA (`smer_bps`, pontos-básicos
//! 0..=10_000 — doutrina zero-float, 20-ARQUITETURA-RUST.md §4.5). Por
//! construção: SMER 100% ⟺ zero DATA_DRIFT e zero RADIANT_GAP. União vazia
//! (ambos os documentos sem elemento) = SMER 100% (vacuo).
//!
//! ## Exit codes (contrato do doc de módulo de main.rs)
//! 0 = SMER 100% (gate passa) · 1 = rejeição-transporte (leitura dos
//! arquivos, tratada no main) · 2 = validação-cliente (XML mal-formado ou
//! fora do subset declarado) · 3 = gate REPROVADO (SMER < 100%) — a
//! autoridade formal deste subcomando é o PRÓPRIO GATE (não há oráculo
//! BACEN aqui).

use std::collections::BTreeMap;

use quick_xml::events::Event;
use reg_core::error::RegError;
use serde::Serialize;

/// Categoria de uma linha do diff — os nomes são o vocabulário do relatório.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum Categoria {
    /// Diferença só de formatação (não reduz o SMER).
    #[serde(rename = "COSMETIC")]
    Cosmetica,
    /// Mesmo caminho, valor semântico diferente.
    #[serde(rename = "DATA_DRIFT")]
    DataDrift,
    /// Presente no legado, ausente no Rust (não reduz o SMER).
    #[serde(rename = "LEGACY_BUG")]
    LegacyBug,
    /// Presente no Rust, ausente no legado.
    #[serde(rename = "RADIANT_GAP")]
    RadiantGap,
}

impl Categoria {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Categoria::Cosmetica => "COSMETIC",
            Categoria::DataDrift => "DATA_DRIFT",
            Categoria::LegacyBug => "LEGACY_BUG",
            Categoria::RadiantGap => "RADIANT_GAP",
        }
    }
}

/// Uma linha classificada do diff — o relatório é auditável linha a linha.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct LinhaDiff {
    /// Categoria classificada.
    pub categoria: Categoria,
    /// Caminho canônico do elemento (`nome[i]/nome[i]/…`).
    pub caminho: String,
    /// Valor do lado legado (`None` quando só existe no Rust).
    pub legado: Option<String>,
    /// Valor do lado Rust (`None` quando só existe no legado).
    pub rust: Option<String>,
}

/// Contagem por categoria — "categorias" do relatório.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub(crate) struct ContagemCategorias {
    /// Linhas COSMETIC (formatação; não reduz o SMER).
    pub cosmetic: u64,
    /// Linhas DATA_DRIFT (reduzem o SMER).
    pub data_drift: u64,
    /// Linhas LEGACY_BUG (não reduzem o SMER).
    pub legacy_bug: u64,
    /// Linhas RADIANT_GAP (reduzem o SMER).
    pub radiant_gap: u64,
}

/// Relatório do `shadow-diff` — categorias + SMER + linhas classificadas.
#[derive(Debug, Clone)]
pub(crate) struct RelatorioShadow {
    /// Contagem por categoria.
    pub categorias: ContagemCategorias,
    /// SMER em pontos-básicos (0..=10_000; 10_000 = 100%).
    pub smer_bps: u64,
    /// Linhas da união legado∪Rust (a base do percentual).
    pub total_linhas: u64,
    /// Linhas semanticamente equivalentes (inclui as COSMETIC).
    pub equivalentes: u64,
    /// Linhas classificadas (todas as não equivalentes + as COSMETIC).
    pub linhas: Vec<LinhaDiff>,
}

impl RelatorioShadow {
    /// O gate de desligamento do legado: SMER 100% ⟺ zero DATA_DRIFT e zero
    /// RADIANT_GAP (equivalência garantida pela fórmula do SMER).
    pub(crate) fn gate_passa(&self) -> bool {
        self.smer_bps == 10_000
    }
}

/// SMER em ponto percentual com 2 casas — formatação INTEIRA do `smer_bps`
/// (zero-float: divisão/resto de inteiros, sem `f64`).
pub(crate) fn smer_pct(bps: u64) -> String {
    format!("{}.{:02}%", bps / 100, bps % 100)
}

/// Relatório estruturado para pipelines (`--json`).
pub(crate) fn relatorio_json(rel: &RelatorioShadow) -> serde_json::Value {
    serde_json::json!({
        "categorias": serde_json::to_value(rel.categorias)
            .unwrap_or(serde_json::Value::Null),
        "smer": smer_pct(rel.smer_bps),
        "smer_bps": rel.smer_bps,
        "total_linhas": rel.total_linhas,
        "equivalentes": rel.equivalentes,
        "gate_passa": rel.gate_passa(),
        "linhas": &rel.linhas,
    })
}

/// Relatório legível para o operador.
pub(crate) fn relatorio_humano(rel: &RelatorioShadow) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "SMER: {} ({} bps) — DATA_DRIFT: {} — RADIANT_GAP: {} — gate: {}\n",
        smer_pct(rel.smer_bps),
        rel.smer_bps,
        rel.categorias.data_drift,
        rel.categorias.radiant_gap,
        if rel.gate_passa() { "PASSA" } else { "REPROVA" },
    ));
    out.push_str(&format!(
        "linhas da união: {} | equivalentes: {} | COSMETIC: {} | LEGACY_BUG: {}\n",
        rel.total_linhas, rel.equivalentes, rel.categorias.cosmetic, rel.categorias.legacy_bug,
    ));
    if rel.linhas.is_empty() {
        out.push_str("sem divergências classificadas\n");
    }
    for l in &rel.linhas {
        out.push_str(&format!(
            "[{}] {} — legado: {} | rust: {}\n",
            l.categoria.as_str(),
            l.caminho,
            l.legado.as_deref().unwrap_or("—"),
            l.rust.as_deref().unwrap_or("—"),
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// Árvore normalizada
// ---------------------------------------------------------------------------

/// Nó da árvore normalizada — preserva SEMANTICS (attrs, texto) e as
/// informações de FORMATAÇÃO usadas pela categoria COSMETIC.
#[derive(Debug, Default, Clone)]
struct NoXml {
    /// Nome QUALIFICADO (prefixo incluso — subset declara que não há
    /// resolução de namespace).
    nome: String,
    /// Atributos na ordem ORIGINAL, entidades resolvidas e normalizadas.
    attrs: Vec<(String, String)>,
    /// Texto bruto concatenado (Text + GeneralRef decodificados, CDATA cru),
    /// whitespace intacto; vazio quando puramente inter-elemento.
    texto: String,
    /// Comentários filhos diretos, em ordem de documento.
    comentarios: Vec<String>,
    /// Declaração XML do documento — apenas no nó raiz ("" nos demais).
    decl: String,
    /// `true` quando o elemento veio de `<a/>` (formatação: empty vs paired).
    eh_empty: bool,
    /// Elementos filhos, em ordem de documento.
    filhos: Vec<NoXml>,
}

/// Erro de parse/classificação do subset — `RegError::InvalidValue` nomeando
/// o lado (`shadow-diff.legado` / `shadow-diff.rust`). Nenhum erro paralelo.
fn erro_lado(lado: &str, motivo: String) -> RegError {
    RegError::InvalidValue {
        field: format!("shadow-diff.{lado}"),
        reason: motivo,
    }
}

/// Nome qualificado do elemento/atributo (`QName(pub &str)` no quick-xml
/// 0.42) — UTF-8 é garantido por construção: `QName` já é `&str`; bytes
/// não-UTF-8 estouram no próprio parser (subset declara UTF-8).
fn nome_qualificado(q: quick_xml::name::QName<'_>) -> String {
    q.0.to_string()
}

/// Coleta os atributos na ordem original com valores normalizados
/// (`normalized_value` resolve entidades predefinidas e normaliza whitespace
/// conforme XML 1.0 — mesmo idioma de `sta.rs::coletar_atributos`).
fn coletar_attrs(
    e: &quick_xml::events::BytesStart<'_>,
    lado: &str,
) -> Result<Vec<(String, String)>, RegError> {
    e.attributes()
        .map(|a| {
            let a = a.map_err(|err| erro_lado(lado, format!("atributo XML inválido: {err}")))?;
            let nome = a.key.0.to_string();
            let valor = a
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .map_err(|err| {
                    erro_lado(
                        lado,
                        format!("valor de atributo XML inválido em {nome}: {err}"),
                    )
                })?
                .into_owned();
            Ok((nome, valor))
        })
        .collect()
}

/// Concatena texto no nó corrente. Texto fora de elemento é aceito somente
/// se for whitespace puro (prólogo/epílogo — formatação).
fn apendar_texto(pilha: &mut [NoXml], texto: &str, lado: &str) -> Result<(), RegError> {
    let Some(no) = pilha.last_mut() else {
        if texto.chars().all(|c| c.is_ascii_whitespace()) {
            return Ok(());
        }
        return Err(erro_lado(
            lado,
            "conteúdo textual fora do elemento-raiz".into(),
        ));
    };
    no.texto.push_str(texto);
    Ok(())
}

/// Pós-processamento da árvore: texto PURAMENTE branco em elemento que
/// possui filhos é formatação de pretty-print — descartado em AMBOS os
/// níveis (o texto chega antes/depois dos filhos, então o teste precisa ser
/// feito com a árvore completa).
fn finalizar_arvore(no: &mut NoXml) {
    if !no.filhos.is_empty() && no.texto.chars().all(|c| c.is_ascii_whitespace()) {
        no.texto.clear();
    }
    for f in &mut no.filhos {
        finalizar_arvore(f);
    }
}

/// Parse do XML em árvore normalizada (subset declarado no doc de módulo).
fn parsear_arvore(bytes: &[u8], lado: &str) -> Result<NoXml, RegError> {
    let mut reader = quick_xml::Reader::from_reader(bytes);
    let mut pilha: Vec<NoXml> = Vec::new();
    let mut raiz: Option<NoXml> = None;
    let mut decl: Option<String> = None;
    let mut raiz_fechada = false;
    let mut buf = Vec::new();
    loop {
        buf.clear();
        let evento = reader
            .read_event_into(&mut buf)
            .map_err(|e| erro_lado(lado, format!("XML mal-formado: {e}")))?;
        let fim = matches!(evento, Event::Eof);
        match evento {
            Event::Start(e) => {
                if raiz_fechada {
                    return Err(erro_lado(lado, "conteúdo após o elemento-raiz".into()));
                }
                pilha.push(NoXml {
                    nome: nome_qualificado(e.name()),
                    attrs: coletar_attrs(&e, lado)?,
                    ..NoXml::default()
                });
            }
            Event::End(_) => {
                let mut no = pilha.pop().ok_or_else(|| {
                    erro_lado(lado, "fechamento sem abertura correspondente".into())
                })?;
                if pilha.is_empty() {
                    finalizar_arvore(&mut no);
                    raiz = Some(no);
                    raiz_fechada = true;
                } else if let Some(pai) = pilha.last_mut() {
                    pai.filhos.push(no);
                }
            }
            Event::Empty(e) => {
                if raiz_fechada {
                    return Err(erro_lado(lado, "conteúdo após o elemento-raiz".into()));
                }
                let no = NoXml {
                    nome: nome_qualificado(e.name()),
                    attrs: coletar_attrs(&e, lado)?,
                    eh_empty: true,
                    ..NoXml::default()
                };
                match pilha.last_mut() {
                    Some(pai) => pai.filhos.push(no),
                    None => {
                        raiz = Some(no);
                        raiz_fechada = true;
                    }
                }
            }
            Event::Text(t) => {
                // quick-xml 0.42 NÃO des-escapa texto: entidades chegam cruas.
                let cru: &str = &t;
                let decodificado = quick_xml::escape::unescape(cru)
                    .map_err(|e| erro_lado(lado, format!("entidade XML inválida: {e}")))?;
                apendar_texto(&mut pilha, &decodificado, lado)?;
            }
            Event::GeneralRef(r) => {
                // quick-xml 0.42: BytesRef deref devolve o conteúdo ENTRE
                // `&` e `;` (ex.: `&amp;` → "amp", `&#38;` → "#38") — a
                // referência completa é reconstruída antes da resolução.
                let cru: &str = &r;
                let referencia = format!("&{cru};");
                let decodificado = quick_xml::escape::unescape(&referencia)
                    .map_err(|e| erro_lado(lado, format!("entidade XML inválida: {e}")))?;
                apendar_texto(&mut pilha, &decodificado, lado)?;
            }
            // CDATA é conteúdo CRU — entidades dentro de CDATA são literais
            // (semântica XML); nunca passar por `unescape`.
            Event::CData(c) => {
                let cru: &str = &c;
                apendar_texto(&mut pilha, cru, lado)?;
            }
            Event::Comment(c) => {
                let cru: &str = &c;
                if let Some(no) = pilha.last_mut() {
                    no.comentarios.push(cru.to_string());
                }
                // comentário no prólogo/epílogo (fora de elemento) — descartado
            }
            Event::Decl(d) => {
                let cru: &str = &d;
                if decl.is_none() {
                    decl = Some(cru.to_string());
                }
            }
            // Instruções de processamento e DOCTYPE: declarações de
            // processamento, não dados — descartadas em ambos os níveis.
            Event::PI(_) | Event::DocType(_) => {}
            Event::Eof => {}
        }
        if fim {
            break;
        }
    }
    if !pilha.is_empty() {
        return Err(erro_lado(
            lado,
            "elemento aberto sem fechamento até o EOF".into(),
        ));
    }
    let mut raiz = raiz.ok_or_else(|| erro_lado(lado, "documento sem elemento-raiz".into()))?;
    raiz.decl = decl.unwrap_or_default();
    Ok(raiz)
}

// ---------------------------------------------------------------------------
// Achatamento em linhas (caminho → conteúdo)
// ---------------------------------------------------------------------------

/// Uma linha do documento: o nível SEMÂNTICO (`attrs` ordenado + `texto`
/// colapsado decide o diff) e o nível de FORMATAÇÃO (só alimenta COSMETIC).
#[derive(Debug, Default, Clone)]
struct LinhaAchatada {
    /// Atributos ordenados por nome — o valor semântico.
    attrs: BTreeMap<String, String>,
    /// Texto com whitespace colapsado — o valor semântico.
    texto: String,
    /// Atributos na ordem original — formatação.
    attrs_loose: Vec<(String, String)>,
    /// Texto bruto — formatação.
    texto_loose: String,
    /// Comentários filhos diretos — formatação.
    comentarios: Vec<String>,
    /// Declaração XML (raiz apenas) — formatação.
    decl: String,
    /// `<a/>` vs `<a></a>` — formatação.
    eh_empty: bool,
}

/// Renderiza o valor SEMÂNTICO da linha (o que vai no relatório para
/// DATA_DRIFT / LEGACY_BUG / RADIANT_GAP).
fn render_semantico(l: &LinhaAchatada) -> String {
    let mut partes: Vec<String> = l.attrs.iter().map(|(k, v)| format!("@{k}={v:?}")).collect();
    partes.push(format!("texto={:?}", l.texto));
    partes.join(" ")
}

/// Renderiza o valor de FORMATAÇÃO da linha (o que vai no relatório para
/// COSMETIC).
fn render_formatacao(l: &LinhaAchatada) -> String {
    let mut partes: Vec<String> = l
        .attrs_loose
        .iter()
        .map(|(k, v)| format!("@{k}={v:?}"))
        .collect();
    partes.push(
        if l.eh_empty {
            "forma=empty"
        } else {
            "forma=paired"
        }
        .to_string(),
    );
    for c in &l.comentarios {
        partes.push(format!("comentario={c:?}"));
    }
    if !l.decl.is_empty() {
        partes.push(format!("decl={:?}", l.decl));
    }
    partes.push(format!("texto_bruto={:?}", l.texto_loose));
    partes.join(" ")
}

/// Colapsa runs de whitespace ASCII em um espaço e corta as pontas.
/// NBSP (U+00A0) NÃO é whitespace ASCII — é DADO (subset declarado).
fn colapsar_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pendente = false;
    for c in s.chars() {
        if c.is_ascii_whitespace() {
            pendente = !out.is_empty();
        } else {
            if pendente {
                out.push(' ');
                pendente = false;
            }
            out.push(c);
        }
    }
    out
}

/// Achata a árvore em `caminho[i] → linha`. O índice de ocorrência entre
/// irmãos homônimos é SEMPRE emitido (determinístico e à prova de colisão).
fn achatamento_rec(no: &NoXml, caminho: &str, mapa: &mut BTreeMap<String, LinhaAchatada>) {
    let linha = LinhaAchatada {
        // Atributo repetido: a última ocorrência vence (subset declarado).
        attrs: no.attrs.iter().cloned().collect(),
        texto: colapsar_whitespace(&no.texto),
        attrs_loose: no.attrs.clone(),
        texto_loose: no.texto.clone(),
        comentarios: no.comentarios.clone(),
        decl: no.decl.clone(),
        eh_empty: no.eh_empty,
    };
    mapa.insert(caminho.to_string(), linha);
    let mut contagem: BTreeMap<&str, usize> = BTreeMap::new();
    for f in &no.filhos {
        let indice = contagem.entry(f.nome.as_str()).or_insert(0);
        let proximo = *indice;
        *indice += 1;
        achatamento_rec(f, &format!("{caminho}/{}[{proximo}]", f.nome), mapa);
    }
}

// ---------------------------------------------------------------------------
// Diff classificado
// ---------------------------------------------------------------------------

/// Compara o XML do legado com o XML do Rust e emite o relatório
/// classificado com o SMER (doc de módulo para o subset e as categorias).
pub(crate) fn diff(legado: &[u8], rust: &[u8]) -> Result<RelatorioShadow, RegError> {
    let arvore_legado = parsear_arvore(legado, "legado")?;
    let arvore_rust = parsear_arvore(rust, "rust")?;

    let mut mapa_legado = BTreeMap::new();
    achatamento_rec(
        &arvore_legado,
        &format!("{}[0]", arvore_legado.nome),
        &mut mapa_legado,
    );
    let mut mapa_rust = BTreeMap::new();
    achatamento_rec(
        &arvore_rust,
        &format!("{}[0]", arvore_rust.nome),
        &mut mapa_rust,
    );

    let mut linhas: Vec<LinhaDiff> = Vec::new();
    let mut cont = ContagemCategorias::default();
    let mut equivalentes: u64 = 0;

    // Merge das duas chaves ORDENADAS (BTreeMap) — união determinística.
    let mut it_legado = mapa_legado.keys();
    let mut it_rust = mapa_rust.keys();
    let mut chave_legado = it_legado.next();
    let mut chave_rust = it_rust.next();
    while let Some(kl) = chave_legado {
        let Some(kr) = chave_rust else {
            cont.legacy_bug += 1;
            let linha = &mapa_legado[kl];
            linhas.push(LinhaDiff {
                categoria: Categoria::LegacyBug,
                caminho: kl.clone(),
                legado: Some(render_semantico(linha)),
                rust: None,
            });
            chave_legado = it_legado.next();
            continue;
        };
        match kl.cmp(kr) {
            std::cmp::Ordering::Equal => {
                let la = &mapa_legado[kl];
                let lr = &mapa_rust[kr];
                let mesmo_semantico = la.attrs == lr.attrs && la.texto == lr.texto;
                if mesmo_semantico {
                    equivalentes += 1;
                    if render_formatacao(la) != render_formatacao(lr) {
                        cont.cosmetic += 1;
                        linhas.push(LinhaDiff {
                            categoria: Categoria::Cosmetica,
                            caminho: kl.clone(),
                            legado: Some(render_formatacao(la)),
                            rust: Some(render_formatacao(lr)),
                        });
                    }
                } else {
                    cont.data_drift += 1;
                    linhas.push(LinhaDiff {
                        categoria: Categoria::DataDrift,
                        caminho: kl.clone(),
                        legado: Some(render_semantico(la)),
                        rust: Some(render_semantico(lr)),
                    });
                }
                chave_legado = it_legado.next();
                chave_rust = it_rust.next();
            }
            std::cmp::Ordering::Less => {
                cont.legacy_bug += 1;
                let linha = &mapa_legado[kl];
                linhas.push(LinhaDiff {
                    categoria: Categoria::LegacyBug,
                    caminho: kl.clone(),
                    legado: Some(render_semantico(linha)),
                    rust: None,
                });
                chave_legado = it_legado.next();
            }
            std::cmp::Ordering::Greater => {
                cont.radiant_gap += 1;
                let linha = &mapa_rust[kr];
                linhas.push(LinhaDiff {
                    categoria: Categoria::RadiantGap,
                    caminho: kr.clone(),
                    legado: None,
                    rust: Some(render_semantico(linha)),
                });
                chave_rust = it_rust.next();
            }
        }
    }
    while let Some(kr) = chave_rust {
        cont.radiant_gap += 1;
        let linha = &mapa_rust[kr];
        linhas.push(LinhaDiff {
            categoria: Categoria::RadiantGap,
            caminho: kr.clone(),
            legado: None,
            rust: Some(render_semantico(linha)),
        });
        chave_rust = it_rust.next();
    }

    // Total = união de chaves dos dois mapas: equivalentes (que JÁ inclui as
    // linhas cosmetically-divergentes — semântica igual) + legacy_bug +
    // data_drift + radiant_gap. Somar cont.cosmetic de novo contava a linha
    // cosmetic 2× e inflava o total (o % do relatório mentia quando COSMETIC
    // coexiste com DATA_DRIFT/RADIANT_GAP).
    let total = equivalentes + cont.legacy_bug + cont.data_drift + cont.radiant_gap;
    let equivalentes_semanticos = total - cont.data_drift - cont.radiant_gap;
    // SMER em pontos-básicos — aritmética inteira (zero-float). O
    // intermediário u128 elimina overflow; o quociente ≤ 10_000 cabe em u64.
    let smer_bps: u64 = if total == 0 {
        10_000
    } else {
        u64::try_from(
            (u128::from(10_000u64) * u128::from(equivalentes_semanticos)) / u128::from(total),
        )
        .unwrap_or(u64::MAX)
    };

    Ok(RelatorioShadow {
        categorias: cont,
        smer_bps,
        total_linhas: total,
        equivalentes,
        linhas,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;

    const XML_BASE: &str = r#"<?xml version="1.0" encoding="UTF-8"?><Doc3040 data-base="2026-11"><Cli><Op><Saldo>100</Saldo></Op></Cli></Doc3040>"#;

    #[test]
    fn par_identico_smer_100() {
        let rel = diff(XML_BASE.as_bytes(), XML_BASE.as_bytes()).unwrap();
        assert_eq!(rel.smer_bps, 10_000);
        assert_eq!(smer_pct(rel.smer_bps), "100.00%");
        assert!(rel.gate_passa());
        assert_eq!(rel.total_linhas, 4); // Doc3040[0] + Cli + Op + Saldo
        assert_eq!(rel.equivalentes, 4);
        assert!(rel.linhas.is_empty());
        assert_eq!(rel.categorias, ContagemCategorias::default());
    }

    /// COSMETIC: ordem de atributos trocada + declaração diversa (linha da
    /// raiz) e whitespace de texto (linha do Saldo). SMER continua 100%.
    #[test]
    fn formatacao_divergente_classifica_cosmetic() {
        let legado = r#"<?xml version="1.0" encoding="UTF-8"?><Doc3040 data-base="2026-11" id="X"><Cli><Saldo> 100 </Saldo></Cli></Doc3040>"#;
        let rust = r#"<?xml version="1.0"?><!-- emissão --><Doc3040 id="X" data-base="2026-11"><Cli><Saldo>100</Saldo></Cli></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert_eq!(rel.categorias.cosmetic, 2);
        assert_eq!(rel.categorias.data_drift, 0);
        assert_eq!(rel.smer_bps, 10_000);
        assert!(rel.gate_passa());
        assert_eq!(rel.linhas.len(), 2);
        for l in &rel.linhas {
            assert_eq!(l.categoria, Categoria::Cosmetica);
        }
        // total_linhas = união de chaves (3 caminhos: Doc3040, Cli, Saldo) —
        // NUNCA equivalentes + cont.cosmetic (a linha cosmetic já está em
        // equivalentes; somar de novo infla o total).
        assert_eq!(rel.total_linhas, 3);
        assert_eq!(rel.equivalentes, 3);
    }

    /// COSMETIC coexistindo com DATA_DRIFT: o total é a união de chaves e o
    /// SMER é o honesto — a linha cosmetic não é contada 2× (bug do total
    /// inflado: união 4 → relatório antigo dizia 5 e 80% em vez de 75%).
    #[test]
    fn total_eh_a_uniao_quando_cosmetic_coexiste_com_drift() {
        let legado =
            r#"<Doc3040><Cli><Saldo>100</Saldo><Sufixo>A</Sufixo><Obs>v1</Obs></Cli></Doc3040>"#;
        let rust = r#"<Doc3040><Cli><Saldo>110</Saldo><Sufixo> A </Sufixo><Novo>r4</Novo></Cli></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        // União: Doc3040, Cli, Saldo (drift), Sufixo (cosmetic), Obs (legacy_bug), Novo (radiant_gap).
        assert_eq!(rel.categorias.data_drift, 1);
        assert_eq!(rel.categorias.cosmetic, 1);
        assert_eq!(rel.categorias.legacy_bug, 1);
        assert_eq!(rel.categorias.radiant_gap, 1);
        // equivalentes = Doc3040 + Cli + Sufixo (cosmetic incluído) = 3.
        assert_eq!(rel.equivalentes, 3);
        assert_eq!(
            rel.total_linhas, 6,
            "total = união (não 7 = equivalentes + cosmetic + drift + bug + gap)"
        );
        // Honestos: (6 - 1 drift - 1 gap) / 6 = 4/6 → 6666 bps (66.66%).
        assert_eq!(rel.smer_bps, 6_666);
        assert!(!rel.gate_passa());
    }

    /// DATA_DRIFT: mesmo caminho, valor diferente — SMER < 100% e gate reprova.
    /// União de 3 linhas (raiz, Cli, Saldo), 2 equivalentes → 20_000/3 = 6666 bps.
    #[test]
    fn data_drift_reduz_smer_e_reprova_gate() {
        let legado = r#"<Doc3040><Cli><Saldo>100</Saldo></Cli></Doc3040>"#;
        let rust = r#"<Doc3040><Cli><Saldo>110</Saldo></Cli></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert_eq!(rel.categorias.data_drift, 1);
        assert_eq!(rel.total_linhas, 3);
        assert_eq!(rel.equivalentes, 2);
        assert_eq!(rel.smer_bps, 6_666); // 10_000 × 2 / 3, inteiro
        assert_eq!(smer_pct(rel.smer_bps), "66.66%");
        assert!(!rel.gate_passa());
        let linha = &rel.linhas[0];
        assert_eq!(linha.categoria, Categoria::DataDrift);
        assert!(linha.caminho.contains("Saldo[0]"));
        assert!(linha.legado.as_deref().unwrap().contains("100"));
        assert!(linha.rust.as_deref().unwrap().contains("110"));
    }

    /// LEGACY_BUG: elemento só no legado — NÃO reduz o SMER (defeito do v1,
    /// documentado), mas a linha aparece no relatório.
    #[test]
    fn legacy_bug_nao_reduz_smer() {
        let legado =
            r#"<Doc3040><Cli><Saldo>100</Saldo><Obs>v1 emitia obs sem fonte</Obs></Cli></Doc3040>"#;
        let rust = r#"<Doc3040><Cli><Saldo>100</Saldo></Cli></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert_eq!(rel.categorias.legacy_bug, 1);
        assert_eq!(rel.categorias.data_drift, 0);
        assert_eq!(rel.categorias.radiant_gap, 0);
        assert_eq!(rel.smer_bps, 10_000);
        assert!(rel.gate_passa());
        let linha = &rel.linhas[0];
        assert_eq!(linha.categoria, Categoria::LegacyBug);
        assert!(linha.caminho.contains("Obs[0]"));
        assert!(linha.rust.is_none());
    }

    /// RADIANT_GAP: elemento só no Rust — reduz o SMER e reprova o gate.
    #[test]
    fn radiant_gap_reduz_smer() {
        let legado = r#"<Doc3040><Cli><Saldo>100</Saldo></Cli></Doc3040>"#;
        let rust = r#"<Doc3040><Cli><Saldo>100</Saldo><Novo>campo sem contraparte no legado</Novo></Cli></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert_eq!(rel.categorias.radiant_gap, 1);
        assert!(rel.smer_bps < 10_000);
        assert!(!rel.gate_passa());
        let linha = &rel.linhas[0];
        assert_eq!(linha.categoria, Categoria::RadiantGap);
        assert!(linha.legado.is_none());
        assert!(linha.caminho.contains("Novo[0]"));
    }

    /// Elementos vazios de formas diferentes (`<a/>` vs `<a></a>`) são
    /// semanticamente iguais — COSMETIC na forma.
    #[test]
    fn empty_vs_paired_eh_cosmetic() {
        let legado = r#"<Doc3040 data-base="2026-11"/>"#;
        let rust = r#"<Doc3040 data-base="2026-11"></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert_eq!(rel.categorias.cosmetic, 1);
        assert_eq!(rel.smer_bps, 10_000);
        assert!(rel.gate_passa());
        assert!(
            rel.linhas[0]
                .legado
                .as_deref()
                .unwrap()
                .contains("forma=empty")
        );
    }

    /// Pretty-print (whitespace inter-elemento) NÃO gera linha — subset
    /// declarado: formatação em ambos os níveis.
    #[test]
    fn pretty_print_nao_gera_linha() {
        let legado = r#"<Doc3040>
  <Cli>
    <Saldo>100</Saldo>
  </Cli>
</Doc3040>"#;
        let rust = r#"<Doc3040><Cli><Saldo>100</Saldo></Cli></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert!(rel.linhas.is_empty());
        assert_eq!(rel.smer_bps, 10_000);
    }

    /// Comentário DENTRO do elemento é formatação — COSMETIC; comentário no
    /// prólogo é descartado.
    #[test]
    fn comentario_interno_eh_cosmetic() {
        let legado = r#"<Doc3040><Cli><Saldo>100</Saldo></Cli></Doc3040>"#;
        let rust = r#"<Doc3040><Cli><!-- revisado --><Saldo>100</Saldo></Cli></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert_eq!(rel.categorias.cosmetic, 1);
        assert!(rel.linhas[0].caminho.contains("Cli[0]"));
        assert!(
            rel.linhas[0]
                .rust
                .as_deref()
                .unwrap()
                .contains("comentario=\" revisado \"")
        );
    }

    /// NBSP é DADO (não é whitespace ASCII) — divergência semântica.
    #[test]
    fn nbsp_eh_dado_nao_formatacao() {
        let legado = "<Doc3040><Saldo>100\u{a0}</Saldo></Doc3040>";
        let rust = "<Doc3040><Saldo>100</Saldo></Doc3040>";
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert_eq!(rel.categorias.data_drift, 1);
        assert!(!rel.gate_passa());
    }

    /// Elementos repetidos ganham índice de ocorrência — drift localiza a
    /// ocorrência exata (Cli[1]) sem tocar Cli[0].
    #[test]
    fn elementos_repetidos_localizam_por_indice() {
        let legado = r#"<Doc3040><Cli><Saldo>1</Saldo></Cli><Cli><Saldo>2</Saldo></Cli></Doc3040>"#;
        let rust = r#"<Doc3040><Cli><Saldo>1</Saldo></Cli><Cli><Saldo>3</Saldo></Cli></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert_eq!(rel.categorias.data_drift, 1);
        assert!(rel.linhas[0].caminho.contains("Cli[1]/Saldo[0]"));
    }

    /// CDATA é texto cru (entidades internas são literais — semântica XML).
    #[test]
    fn cdata_eh_texto_cru() {
        let legado = r#"<Doc3040><Nota><![CDATA[a & b]]></Nota></Doc3040>"#;
        let rust = r#"<Doc3040><Nota><![CDATA[a & b]]></Nota></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert!(rel.linhas.is_empty());
        assert_eq!(rel.smer_bps, 10_000);
    }

    /// Entidades resolvidas no nível semântico: `&amp;` (Text, entidade
    /// resolvida) ≡ `&` literal dentro de CDATA (conteúdo cru) — caminhos
    /// distintos, mesmo valor semântico. (`&` CRUO em texto é XML ilegal e
    /// é rejeitado — teste `referencia_nao_fechada_err`.)
    #[test]
    fn entidades_resolvidas() {
        let legado = r#"<Doc3040><Nome>A &amp; B</Nome></Doc3040>"#;
        let rust = r#"<Doc3040><Nome><![CDATA[A & B]]></Nome></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert!(rel.linhas.is_empty());
        assert_eq!(rel.smer_bps, 10_000);
    }

    /// `&` cru em texto (referência não fechada) é XML ilegal — erro tipado.
    #[test]
    fn referencia_nao_fechada_err() {
        let err = diff(b"<Doc3040><Nome>A & B</Nome></Doc3040>", b"<Doc3040/>").unwrap_err();
        assert!(err.to_string().contains("mal-formado"), "{err}");
    }

    /// XML mal-formado é erro tipado nomeando o lado.
    #[test]
    fn xml_malformado_erro_tipado() {
        let err = diff(b"<Doc3040><Cli></Doc3040>", b"<Doc3040/>").unwrap_err();
        match &err {
            RegError::InvalidValue { field, reason } => {
                assert!(field.contains("shadow-diff.legado"), "{field}");
                assert!(reason.contains("mal-formado"), "{reason}");
            }
            RegError::MissingField { field } => {
                panic!("esperado InvalidValue, obtido MissingField: {field}")
            }
            RegError::RuleNotImplemented { rule, state } => {
                panic!("esperado InvalidValue, obtido RuleNotImplemented {rule}/{state}")
            }
        }
    }

    /// Elemento aberto sem fechamento — erro tipado, seja pela pilha do
    /// subset ("fechamento") seja pelo parser pull do quick-xml.
    #[test]
    fn elemento_aberto_sem_fechamento_err() {
        let err = diff(b"<Doc3040>", b"<Doc3040/>").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("fechamento") || msg.contains("mal-formado"),
            "{msg}"
        );
    }

    /// Documento sem elemento-raiz — erro tipado.
    #[test]
    fn documento_sem_raiz_err() {
        let err = diff(b"", b"<Doc3040/>").unwrap_err();
        assert!(err.to_string().contains("raiz"), "{err}");
    }

    /// Bytes não-UTF-8 são recusados com erro tipado nomeando o lado
    /// (subset declara UTF-8; o BOM UTF-16 `[0xFF, 0xFE]` é consumido como
    /// BOM pelo parser, então o vetor inválido vai DENTRO do texto — a
    /// sequência `0xC3 0x28` não decodifica).
    #[test]
    fn bytes_nao_utf8_err() {
        let err = diff(b"<Doc3040><Nome>x\xC3\x28</Nome></Doc3040>", b"<Doc3040/>").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("mal-formado") || msg.contains("UTF") || msg.contains("raiz"),
            "{msg}"
        );
    }

    /// União vazia (ambos sem elemento é impossível — exige raiz —, então o
    /// caso vacuo testável é o reporte com total 0 NÃO existir; o teste pina
    /// a invariante do gate com os mínimos: 1 linha equivalente).
    #[test]
    fn gate_passa_soh_com_zero_drift_e_gap() {
        // drift 0, gap 0, legacy_bug > 0 → gate passa (defeito do v1 não
        // impede o desligamento — é a definição do ask).
        let legado = r#"<A><X>1</X><Y>1</Y></A>"#;
        let rust = r#"<A><X>1</X></A>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert_eq!(rel.categorias.legacy_bug, 1);
        assert!(rel.gate_passa());

        // um único RADIANT_GAP já reprova.
        let legado = r#"<A><X>1</X></A>"#;
        let rust = r#"<A><X>1</X><Y>1</Y></A>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        assert_eq!(rel.categorias.radiant_gap, 1);
        assert!(!rel.gate_passa());
    }

    /// Relatórios JSON e humano são determinísticos e carregam o contrato
    /// ({ categorias, smer, linhas }).
    #[test]
    fn relatorios_json_e_humano() {
        let legado = r#"<Doc3040><Cli><Saldo>100</Saldo></Cli></Doc3040>"#;
        let rust = r#"<Doc3040><Cli><Saldo>110</Saldo></Cli></Doc3040>"#;
        let rel = diff(legado.as_bytes(), rust.as_bytes()).unwrap();
        let j = relatorio_json(&rel);
        assert_eq!(j["smer"], "66.66%");
        assert_eq!(j["smer_bps"], 6_666);
        assert_eq!(j["gate_passa"], false);
        assert_eq!(j["categorias"]["data_drift"], 1);
        assert_eq!(j["linhas"].as_array().unwrap().len(), 1);
        assert_eq!(j["linhas"][0]["categoria"], "DATA_DRIFT");
        let humano = relatorio_humano(&rel);
        assert!(humano.contains("SMER: 66.66%"));
        assert!(humano.contains("gate: REPROVA"));
        assert!(humano.contains("[DATA_DRIFT]"));
    }
}
