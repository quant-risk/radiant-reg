//! dossier — subcomando `dossier build --mapping <mapping.toml> --out <dir>`
//! (RC Conjunta nº 18/2025).
//!
//! O dossiê é o entregável de entrada exigido pela RC Conjunta 18/2025
//! (docs/blueprint/00-INDEX.md §8: "12 dimensões de qualidade; Dossiê A4/JSON
//! assinado Ed25519 como entregável de entrada"; 23-PRODUTO-PRICING-GTM.md
//! §5.5). Dois artefatos irmãos, MESMA proveniência:
//!
//! - **dossiê.json** — [`DossierPayload`] (reg-ledger::sign) canonicalizado
//!   em JCS RFC 8785 (reg-ledger::jcs) e assinado por [`DualApproval`]
//!   (reg-ledger::sign — 2 chaves Ed25519 DISTINTAS, 4-olhos). O arquivo é
//!   gravado em JCS canônico (o canônico é o contrato de bytes) e carrega o
//!   registro de custódia do SHA-256 do mapping.toml no ledger
//!   tamper-evident (reg-ledger::chain). HONESTIDADE DE COBERTURA: a dupla
//!   alçada assina EXATAMENTE o payload JCS; o registro de custódia
//!   acompanha o artefato SEM estar sob as assinaturas — a integridade dele
//!   é garantida pela própria hash-chain do ledger.
//! - **dossiê.html** — A4 editorial auto-contido: CSS inline, serif do
//!   sistema, ZERO fonte externa (sem `@font-face`/`<link>`/`url()`), sem
//!   JS, hairlines de 1px, ZERO sombras, números tabulares, ZERO métrica
//!   sintética (nenhum score/percentual inventado — só contagens, hashes e
//!   timestamps reais). 4 seções: Capa; Dimensões de Qualidade (as 12 da RC
//!   Conjunta 18/2025); Dicionário & Linhagem campo-a-campo (GERADO DO
//!   mapping.toml — cada campo fonte→destino com unidade e transform);
//!   Prova Criptográfica & Dupla Alçada (payload JCS + as 2 assinaturas +
//!   SHA-256 custodiado no ledger).
//!
//! ## Chaves de assinatura
//! As seeds Ed25519 vêm de variáveis de ambiente ([`ENV_SEED_1`],
//! [`ENV_SEED_2`], hex de 64 chars = 32 bytes) — NUNCA de argv (mesma
//! doutrina de `secrets.rs`: valor em argv aparece no `ps` e no histórico do
//! shell). A custódia das chaves (Secrets Manager/Vault) é Fase 2 — ADR-0015;
//! a assinatura prova integridade e vínculo com a chave pública gravada no
//! artefato, não quem controla a chave.
//!
//! ## Proveniência das 12 dimensões (honestidade de fonte)
//! A RC Conjunta nº 18/2025 define 12 dimensões de qualidade da informação
//! (00-INDEX.md §8; 23-PRODUTO-PRICING-GTM.md:273). A lista literal usada
//! aqui foi conferida em DUAS fontes secundárias INDEPENDENTES na sessão de
//! implementação (Cetax, cetax.com.br/resolucao-conjunta-18-banco-central;
//! Divisio, divisio.com.br/rc18-guia.html — mesmas 12, ordem diversa):
//! acurácia, integridade, completude, consistência, tempestividade,
//! rastreabilidade, acessibilidade, comparabilidade, confiabilidade,
//! relevância, clareza, adaptabilidade. O portal do BCB (exibenormativo) é
//! renderizado por JS e não foi legível nesta sessão — o literal oficial
//! deve ser re-verificado no texto da norma antes de qualquer submissão.
//! Nenhuma dimensão foi inventada e nenhuma foi omitida.
//!
//! ## Exit codes (contrato do doc de módulo de main.rs)
//! 0 = dossiê gerado e assinado · 1 = rejeição-transporte (leitura do
//! mapping.toml, criação de --out, escrita dos artefatos) · 2 =
//! validação-cliente (mapping.toml inválido, cadoc sem suporte no wedge V1,
//! seed ausente/inválida, ledger indisponível, erro de assinatura) · 3 não
//! se aplica — não há oráculo BACEN neste subcomando.

use std::path::Path;

use reg_cadoc::gen3040::CadocCode;
use reg_cadoc::ingest::{MappingDocument, Transform, parse_mapping_toml};
use reg_core::error::RegError;
use reg_core::money::MonetaryUnit;
use reg_ledger::chain::{AppendRequest, Chain, ChainError, Entry};
use reg_ledger::jcs;
use reg_ledger::mapping_locks::Trava;
use reg_ledger::sign::{ArtefatoRef, DossierPayload, DualApproval, SeedAssinatura, SignError};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Variável de ambiente da 1ª seed Ed25519 (hex de 64 chars).
pub(crate) const ENV_SEED_1: &str = "RADIANT_SECRET_DOSSIER_SEED_1";

/// Variável de ambiente da 2ª seed Ed25519 (hex de 64 chars).
pub(crate) const ENV_SEED_2: &str = "RADIANT_SECRET_DOSSIER_SEED_2";

/// As 12 dimensões de qualidade da informação da RC Conjunta nº 18/2025 —
/// nome literal + definição editorial. Proveniência no doc de módulo.
pub(crate) const DIMENSOES_RC18: [(&str, &str); 12] = [
    ("acurácia", "o dado corresponde à realidade que descreve"),
    ("integridade", "o dado não sofre alteração indevida em trânsito ou em repouso"),
    ("completude", "todos os campos exigidos estão presentes, sem lacuna"),
    ("consistência", "o mesmo dado é coerente entre sistemas, períodos e documentos"),
    ("tempestividade", "o dado é disponibilizado dentro do prazo regulatório"),
    ("rastreabilidade", "a linhagem do dado é conhecida da origem ao relato"),
    ("acessibilidade", "o dado é recuperável pelos processos autorizados"),
    ("comparabilidade", "o dado é comparável entre períodos, documentos e instituições"),
    ("confiabilidade", "a fonte e o processo de geração são auditáveis e estáveis"),
    ("relevância", "o dado é pertinente à finalidade do relato"),
    ("clareza", "o dado é legível e sem ambiguidade de interpretação"),
    ("adaptabilidade", "o dado é apto a evoluir com as mudanças normativas"),
];

/// Agregação de erros do dossiê — mesma forma de `LockError` do reg-ledger:
/// compõe erros tipados das camadas, NÃO cria erro paralelo de domínio.
#[derive(Debug, Error)]
pub(crate) enum DossieErro {
    /// Erro de domínio do reg-core (parse do mapping.toml, validações).
    #[error("{0}")]
    Reg(#[from] RegError),
    /// Erro do ledger (custódia da prova).
    #[error("ledger recusou a custódia: {0}")]
    Ledger(#[from] ChainError),
    /// Erro de assinatura (dual approval).
    #[error("assinatura do dossiê: {0}")]
    Assinatura(#[from] SignError),
    /// Erro de canonicalização JCS (payload não-canônico — ex.: float
    /// banido pela doutrina zero-float).
    #[error("canonicalização JCS do dossiê: {0}")]
    Jcs(#[from] reg_ledger::jcs::JcsError),
    /// Erro de serialização serde.
    #[error("serialização do dossiê: {0}")]
    Serde(#[from] serde_json::Error),
    /// Erro de I/O do ledger/ambiente.
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
}

/// Artefatos do dossiê construído — conteúdo dos dois arquivos + os dados
/// que o CLI ecoa para o pipeline.
#[derive(Debug, Clone)]
pub(crate) struct ArtefatoDossie {
    /// Conteúdo de `dossie.json` — JCS canônico, com payload + dupla alçada
    /// + custódia no ledger.
    pub json: String,
    /// Conteúdo de `dossie.html` — A4 editorial auto-contido.
    pub html: String,
    /// SHA-256 hex do payload JCS assinado.
    pub payload_sha256: String,
    /// `id` da entrada de custódia no ledger.
    pub entry_id: i64,
    /// `entry_hash` da entrada de custódia (posição na hash-chain).
    pub entry_hash: String,
}

/// Resolve uma seed do ambiente. NUNCA testável com `std::env::set_var`
/// (é `unsafe` na edição 2024 e o workspace proíbe `unsafe_code`) — a
/// validação de VALOR é feita por [`seed_de_valor`], que é testável.
pub(crate) fn seed_do_ambiente(nome: &str) -> Result<SeedAssinatura, RegError> {
    let valor =
        std::env::var(nome).map_err(|_| RegError::MissingField { field: nome.to_string() })?;
    seed_de_valor(&valor, nome)
}

/// Valida o VALOR de uma seed (hex de 64 chars = 32 bytes) — função pura.
pub(crate) fn seed_de_valor(valor: &str, campo: &str) -> Result<SeedAssinatura, RegError> {
    SeedAssinatura::de_hex(valor).map_err(|e| RegError::InvalidValue {
        field: campo.to_string(),
        reason: format!("seed Ed25519 inválida (esperado hex de 64 chars = 32 bytes): {e}"),
    })
}

/// Constrói o dossiê completo (JSON assinado + HTML) para um mapping.toml.
///
/// Ordem dos passos (cada um exige o anterior): parse do mapping → payload
/// → JCS → dupla alçada (2 chaves distintas) → RE-VERIFICAÇÃO (defesa em
/// profundidade: nunca gravamos um artefato que não passa na própria
/// verificação) → custódia do SHA-256 no ledger → renderização dos dois
/// artefatos.
pub(crate) fn construir(
    mapping_bytes: &[u8],
    ledger: &Chain,
    seed_1: &SeedAssinatura,
    seed_2: &SeedAssinatura,
) -> Result<ArtefatoDossie, DossieErro> {
    let mapping_str = std::str::from_utf8(mapping_bytes).map_err(|e| RegError::InvalidValue {
        field: "mapping.toml".to_string(),
        reason: format!("não é UTF-8 válido: {e}"),
    })?;
    let doc = parse_mapping_toml(mapping_str)?;

    // Wedge V1: o dossiê só existe para o cadoc que o generator produz
    // (mesmo contrato do `validate`).
    if doc.meta.cadoc != CadocCode::C3040.as_str() {
        return Err(RegError::InvalidValue {
            field: "mapping.toml.meta.cadoc".to_string(),
            reason: format!(
                "{} sem suporte no wedge V1 (só {})",
                doc.meta.cadoc,
                CadocCode::C3040.as_str()
            ),
        }
        .into());
    }

    let mapping_sha256 = sha256_hex(mapping_bytes);
    let payload = DossierPayload {
        doc_cod: doc.meta.cadoc.clone(),
        if_cnpj: doc.meta.if_cnpj.clone(),
        data_base: doc.data_base.to_string(),
        mapping_sha256: mapping_sha256.clone(),
        artefatos: vec![ArtefatoRef {
            nome: "mapping.toml".to_string(),
            sha256: mapping_sha256.clone(),
        }],
    };

    // Payload JCS — os EXATOS bytes assinados pelas duas chaves (contrato de
    // bytes do reg-ledger::sign).
    let payload_value = serde_json::to_value(&payload)?;
    let payload_canonico = jcs::canonicalizar(&payload_value)?;
    let aprovacao = DualApproval::nova(payload_canonico.as_bytes(), seed_1, seed_2)?;
    aprovacao.verificar(payload_canonico.as_bytes())?;
    let payload_sha256 = sha256_hex(payload_canonico.as_bytes());

    // Custódia tamper-evident — a mesma semântica da Trava::CustodiaSha256
    // do mapping.lock: o SHA-256 do mapping.toml gravado no ledger.
    let metadata = serde_json::json!({
        "doc_cod": &doc.meta.cadoc,
        "data_base": doc.data_base.to_string(),
        "payload_sha256": &payload_sha256,
        "trava": Trava::CustodiaSha256.to_string(),
        "artefato": "mapping.toml",
    });
    let entry = ledger.append(AppendRequest {
        if_id: &doc.meta.if_cnpj,
        actor: "reg-cli/dossier",
        action: "dossier.build",
        target: "mapping.toml",
        payload: mapping_bytes,
        metadata: Some(&metadata),
    })?;

    let documento = serde_json::json!({
        "payload": payload_value,
        "payload_sha256": &payload_sha256,
        "dual_approval": serde_json::to_value(&aprovacao)?,
        "custodia_ledger": {
            "entry_id": entry.id,
            "actor": &entry.actor,
            "action": &entry.action,
            "payload_hash": &entry.payload_hash,
            "entry_hash": &entry.entry_hash,
            "created_at": &entry.created_at,
        },
    });
    let json = jcs::canonicalizar(&documento)?;
    let html = render_html(&doc, &payload, &aprovacao, &payload_sha256, &payload_canonico, &entry);

    Ok(ArtefatoDossie {
        json,
        html,
        payload_sha256,
        entry_id: entry.id,
        entry_hash: entry.entry_hash,
    })
}

/// Grava os dois artefatos em `out` (separado de [`construir`] para que os
/// testes exerçam a geração sem I/O de diretório).
pub(crate) fn gravar(out: &Path, artefato: &ArtefatoDossie) -> Result<(std::path::PathBuf, std::path::PathBuf), DossieErro> {
    let caminho_json = out.join("dossie.json");
    let caminho_html = out.join("dossie.html");
    std::fs::write(&caminho_json, artefato.json.as_bytes())?;
    std::fs::write(&caminho_html, artefato.html.as_bytes())?;
    Ok((caminho_json, caminho_html))
}

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

// ---------------------------------------------------------------------------
// HTML A4 editorial
// ---------------------------------------------------------------------------

/// CSS inline — serif do sistema, hairline 1px, ZERO sombras, ZERO fonte
/// externa (nenhum `@font-face`/`@import`/`url()`), números tabulares.
const CSS: &str = r#"
:root { --paper: #fdfcfa; --navy: #1b2a44; --hairline: rgba(27, 42, 68, 0.35); }
* { box-sizing: border-box; margin: 0; padding: 0; }
@page { size: A4; margin: 22mm 18mm; }
html { background: var(--paper); }
body {
  background: var(--paper);
  color: var(--navy);
  font-family: Georgia, "Times New Roman", Times, serif;
  font-size: 10.5pt;
  line-height: 1.5;
  font-variant-numeric: tabular-nums lining-nums;
  padding: 28px;
}
section { max-width: 174mm; margin: 0 auto; padding: 10px 0 48px; }
.rotulo {
  border-top: 1px solid var(--navy);
  padding-top: 6px;
  font-size: 8pt;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  margin-bottom: 20px;
}
h1 { font-size: 24pt; font-weight: normal; line-height: 1.2; margin-bottom: 6px; }
h2 { font-size: 14pt; font-weight: normal; border-bottom: 1px solid var(--hairline); padding-bottom: 6px; margin-bottom: 14px; }
h3 { font-size: 9pt; font-weight: normal; letter-spacing: 0.12em; text-transform: uppercase; margin: 18px 0 6px; }
.sub { font-size: 11pt; margin-bottom: 30px; }
.nota { font-size: 8.5pt; margin: 10px 0; }
table { width: 100%; border-collapse: collapse; margin: 12px 0; }
th { text-align: left; font-size: 8pt; font-weight: normal; text-transform: uppercase; letter-spacing: 0.08em; border-bottom: 1px solid var(--navy); padding: 6px 8px; }
td { border-bottom: 1px solid var(--hairline); padding: 6px 8px; vertical-align: top; }
td.chave { width: 34%; }
.mono { font-family: Menlo, Consolas, "DejaVu Sans Mono", monospace; font-size: 8pt; word-break: break-all; }
pre { font-family: Menlo, Consolas, "DejaVu Sans Mono", monospace; font-size: 7.5pt; line-height: 1.4; white-space: pre-wrap; word-break: break-all; border-left: 1px solid var(--navy); padding: 8px 12px; margin: 10px 0; }
ol.dimensoes { list-style: none; margin: 12px 0; }
ol.dimensoes li { border-bottom: 1px solid var(--hairline); padding: 9px 0; display: flex; }
ol.dimensoes .num { min-width: 34px; }
ol.dimensoes .nome { min-width: 140px; }
@media print {
  body { padding: 0; }
  section { break-before: page; padding: 0; max-width: none; }
  section#capa { break-before: auto; }
}
"#;

/// Escapa dados dinâmicos para HTML — TODO valor vindo do mapping.toml/ledger
/// passa por aqui antes de entrar na página.
fn escapar_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Rótulo da unidade contratada — exatamente os nomes do contrato serde
/// (`snake_case`) do mapping.toml.
fn rotulo_unidade(u: MonetaryUnit) -> &'static str {
    match u {
        MonetaryUnit::Reais => "reais",
        MonetaryUnit::Centavos => "centavos",
        MonetaryUnit::MilReais => "mil_reais",
    }
}

/// Rótulo do transform — exatamente os nomes do contrato serde
/// (`snake_case`) do mapping.toml.
fn rotulo_transform(t: Transform) -> &'static str {
    match t {
        Transform::Trim => "trim",
        Transform::ToLower => "to_lower",
        Transform::StripAccents => "strip_accents",
        Transform::DateToDataBase => "date_to_data_base",
    }
}

/// Par chave/valor da tabela da capa e da prova.
fn linha_dados(chave: &str, valor: &str, mono: bool) -> String {
    if mono {
        format!(
            "<tr><td class=\"chave\">{}</td><td class=\"mono\">{}</td></tr>\n",
            escapar_html(chave),
            escapar_html(valor)
        )
    } else {
        format!(
            "<tr><td class=\"chave\">{}</td><td>{}</td></tr>\n",
            escapar_html(chave),
            escapar_html(valor)
        )
    }
}

/// Renderiza o dossiê A4 completo — 4 seções, dados 100% reais.
fn render_html(
    doc: &MappingDocument,
    payload: &DossierPayload,
    aprovacao: &DualApproval,
    payload_sha256: &str,
    payload_canonico: &str,
    entry: &Entry,
) -> String {
    let mut h = String::with_capacity(20 * 1024);
    h.push_str("<!DOCTYPE html>\n<html lang=\"pt-BR\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>");
    h.push_str(&escapar_html(&format!(
        "Dossiê de Qualidade de Dados — Doc {} — {}",
        doc.meta.cadoc, doc.data_base
    )));
    h.push_str("</title>\n<style>");
    h.push_str(CSS);
    h.push_str("</style>\n</head>\n<body>\n");

    // ------------------------------------------------------ Seção I — Capa
    h.push_str("<section id=\"capa\">\n<p class=\"rotulo\">Seção I — Capa</p>\n");
    h.push_str("<h1>Dossiê de Qualidade de Dados</h1>\n");
    h.push_str(
        "<p class=\"sub\">Emissão do Radiant Reg (wedge V1) conforme a Resolução Conjunta nº \
         18/2025 — política de qualidade das informações prestadas.</p>\n",
    );
    h.push_str("<table>\n");
    h.push_str(&linha_dados(
        "Documento",
        "3040 — Dados individualizados de risco de crédito (CMN 4.966)",
        false,
    ));
    h.push_str(&linha_dados("Data-base", &doc.data_base.to_string(), false));
    h.push_str(&linha_dados("IF remetente (CNPJ)", &doc.meta.if_cnpj, false));
    h.push_str(&linha_dados(
        "SHA-256 do mapping.toml custodiado",
        &payload.mapping_sha256,
        true,
    ));
    h.push_str(&linha_dados("Emissão", &entry.created_at, false));
    h.push_str("</table>\n");
    h.push_str(
        "<p class=\"nota\">Artefatos irmãos: dossie.json (assinado por dupla alçada Ed25519) e \
         ledger.sqlite (custódia tamper-evident). As 12 dimensões de qualidade da RC Conjunta \
         nº 18/2025 são declaradas na Seção II; a linhagem campo-a-campo que as evidencia, na \
         Seção III; a prova criptográfica, na Seção IV.</p>\n",
    );
    h.push_str("</section>\n");

    // ---------------------------------- Seção II — Dimensões de Qualidade
    h.push_str("<section id=\"dimensoes\">\n<p class=\"rotulo\">Seção II — Dimensões de Qualidade</p>\n");
    h.push_str("<h2>Dimensões de Qualidade da Informação</h2>\n");
    h.push_str(
        "<p class=\"nota\">As 12 dimensões definidas pela Resolução Conjunta nº 18/2025. Este \
         dossiê as declara como compromisso da política de qualidade; a Seção III evidencia a \
         rastreabilidade campo-a-campo e a Seção IV, a integridade do processo — sem pontuação \
         sintética: só o que é real e verificável.</p>\n",
    );
    h.push_str("<ol class=\"dimensoes\">\n");
    for (i, (nome, definicao)) in DIMENSOES_RC18.iter().enumerate() {
        h.push_str(&format!(
            "<li><span class=\"num\">{:02}</span><span class=\"nome\"><strong>{}</strong></span><span> — {}</span></li>\n",
            i + 1,
            escapar_html(nome),
            escapar_html(definicao),
        ));
    }
    h.push_str("</ol>\n</section>\n");

    // ------------------------------- Seção III — Dicionário & Linhagem
    h.push_str("<section id=\"dicionario\">\n<p class=\"rotulo\">Seção III — Dicionário &amp; Linhagem</p>\n");
    h.push_str("<h2>Dicionário &amp; Linhagem Campo-a-Campo</h2>\n");
    h.push_str(&format!(
        "<p class=\"nota\">Gerado diretamente do mapping.toml custodiado (SHA-256 na capa e na \
         Seção IV). Cada linha é a linhagem completa de um campo: coluna de origem → campo \
         canônico de destino, com a unidade contratada e o pipeline de transform. {} campo(s) \
         mapeado(s); coluna do participante: <em>{}</em>; coluna da operação: <em>{}</em>.</p>\n",
        doc.columns.len(),
        escapar_html(&doc.meta.participante),
        escapar_html(&doc.meta.operacao),
    ));
    h.push_str(
        "<table>\n<thead><tr><th>#</th><th>Coluna de origem</th><th>Campo de destino</th><th>Unidade</th><th>Transform</th></tr></thead>\n<tbody>\n",
    );
    for (i, col) in doc.columns.iter().enumerate() {
        let unidade = match col.unit {
            Some(u) => rotulo_unidade(u).to_string(),
            None => "—".to_string(),
        };
        let transforms = if col.transform.is_empty() {
            "—".to_string()
        } else {
            col.transform
                .iter()
                .map(|t| rotulo_transform(*t))
                .collect::<Vec<&str>>()
                .join(" → ")
        };
        h.push_str(&format!(
            "<tr><td>{:02}</td><td>{}</td><td class=\"mono\">{}</td><td>{}</td><td class=\"mono\">{}</td></tr>\n",
            i + 1,
            escapar_html(&col.source_header),
            escapar_html(&col.target_field.0),
            escapar_html(&unidade),
            escapar_html(&transforms),
        ));
    }
    h.push_str("</tbody>\n</table>\n</section>\n");

    // -------------------------------- Seção IV — Prova Criptográfica
    h.push_str("<section id=\"prova\">\n<p class=\"rotulo\">Seção IV — Prova Criptográfica</p>\n");
    h.push_str("<h2>Prova Criptográfica &amp; Dupla Alçada</h2>\n");
    h.push_str(
        "<p class=\"nota\">O payload JSON abaixo é o EXATO objeto assinado — serialização JCS \
         RFC 8785 (subset), cujo SHA-256 identifica o artefato. As duas assinaturas Ed25519 \
         são independentes (4-olhos) sobre os MESMOS bytes canônicos e foram re-verificadas \
         com <em>verify_strict</em> antes da gravação. O SHA-256 do mapping.toml está \
         custodiado no ledger tamper-evident (hash-chain).</p>\n",
    );
    h.push_str("<h3>Payload canônico (JCS)</h3>\n<pre>");
    h.push_str(&escapar_html(payload_canonico));
    h.push_str("</pre>\n");

    h.push_str("<h3>Identificação do payload assinado</h3>\n<table>\n");
    h.push_str(&linha_dados("SHA-256 do payload JCS (assinado)", payload_sha256, true));
    h.push_str(&linha_dados("SHA-256 do mapping.toml", &payload.mapping_sha256, true));
    h.push_str(&linha_dados("Documento", &payload.doc_cod, false));
    h.push_str(&linha_dados("Data-base", &payload.data_base, false));
    h.push_str("</table>\n");

    h.push_str("<h3>Dupla alçada (Ed25519 — 4-olhos)</h3>\n<table>\n");
    h.push_str(&linha_dados("Chave pública — assinatura 1", &aprovacao.chave_primeira, true));
    h.push_str(&linha_dados("Assinatura 1 (r‖s, hex)", &aprovacao.primeira, true));
    h.push_str(&linha_dados("Chave pública — assinatura 2", &aprovacao.chave_segunda, true));
    h.push_str(&linha_dados("Assinatura 2 (r‖s, hex)", &aprovacao.segunda, true));
    h.push_str("</table>\n");

    h.push_str("<h3>Custódia no ledger tamper-evident</h3>\n<table>\n");
    h.push_str(&linha_dados("Entrada #", &entry.id.to_string(), false));
    h.push_str(&linha_dados("Ator / ação", &format!("{} / {}", entry.actor, entry.action), false));
    h.push_str(&linha_dados("payload_hash (SHA-256 do mapping.toml)", &entry.payload_hash, true));
    h.push_str(&linha_dados("entry_hash (posição na hash-chain)", &entry.entry_hash, true));
    h.push_str(&linha_dados("Carimbo de tempo (RFC 3339, hashado junto)", &entry.created_at, false));
    h.push_str("</table>\n");
    h.push_str(
        "<p class=\"nota\">A custódia usa a mesma semântica da trava \
         <em>custodia_sha256</em> do mapping.lock (reg-ledger). A integridade do registro de \
         custódia é garantida pela hash-chain do ledger; a custódia das chaves de assinatura \
         (Secrets Manager/Vault) é compromisso da Fase 2 — ADR-0015.</p>\n",
    );
    h.push_str("</section>\n");

    h.push_str("</body>\n</html>\n");
    h
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;
    use reg_ledger::sign::{assinar_dossie, verificar_dossie};
    use tempfile::TempDir;

    // Seeds FIXAS do RFC 8032 §7 (mesmas dos testes de reg-ledger::sign) —
    // determinismo é doutrina; a auditoria não depende de RNG.
    const SEED_A: &str =
        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    const SEED_B: &str =
        "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb";

    const MAPPING_TESTE: &str = r#"
[meta]
cadoc = "3040"
data_base = "2026-11"
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

[[columns]]
source_header = "Documento"
target_field = "Doc3040/titulo/documento"
transform = ["trim", "to_lower"]
"#;

    fn construir_teste() -> (TempDir, ArtefatoDossie) {
        let tmp = tempfile::TempDir::new().unwrap();
        let ledger = Chain::open(tmp.path().join("ledger.sqlite")).unwrap();
        let seed_1 = SeedAssinatura::de_hex(SEED_A).unwrap();
        let seed_2 = SeedAssinatura::de_hex(SEED_B).unwrap();
        let artefato = construir(MAPPING_TESTE.as_bytes(), &ledger, &seed_1, &seed_2).unwrap();
        (tmp, artefato)
    }

    /// JSON assinado verifica: payload re-canonicalizado confere com
    /// payload_sha256 e as DUAS assinaturas passam em `verificar()`.
    #[test]
    fn json_assinado_verifica() {
        let (_tmp, artefato) = construir_teste();
        let doc: serde_json::Value = serde_json::from_str(&artefato.json).unwrap();
        let payload: DossierPayload = serde_json::from_value(doc["payload"].clone()).unwrap();
        let aprovacao: DualApproval = serde_json::from_value(doc["dual_approval"].clone()).unwrap();

        let bytes = jcs::canonicalizar_para_bytes(&serde_json::to_value(&payload).unwrap()).unwrap();
        aprovacao.verificar(&bytes).unwrap();
        assert_eq!(artefato.payload_sha256, sha256_hex(&bytes));
        assert_eq!(doc["payload_sha256"], artefato.payload_sha256);
        // As duas chaves são DISTINTAS (4-olhos) e ambas carregam assinatura hex de 128.
        assert_ne!(
            aprovacao.chave_primeira, aprovacao.chave_segunda,
            "dupla alçada exige chaves distintas"
        );
        assert_eq!(aprovacao.primeira.len(), 128);
        assert_eq!(aprovacao.segunda.len(), 128);
    }

    /// Caminho single-key do reg-ledger::sign: o mesmo payload assinado por
    /// UMA chave verifica (`verificar_dossie` ok); o payload ADULTERADO sob a
    /// MESMA assinatura falha.
    #[test]
    fn verificar_dossie_single_key_ok_e_adulterado_falha() {
        let (_tmp, artefato) = construir_teste();
        let doc: serde_json::Value = serde_json::from_str(&artefato.json).unwrap();
        let payload: DossierPayload = serde_json::from_value(doc["payload"].clone()).unwrap();
        let seed = SeedAssinatura::de_hex(SEED_A).unwrap();

        let mut dossie = assinar_dossie(&payload, &seed).unwrap();
        verificar_dossie(&dossie).unwrap();

        dossie.payload.if_cnpj = "99999999000100".into();
        assert!(matches!(
            verificar_dossie(&dossie),
            Err(SignError::AssinaturaInvalida) | Err(SignError::ChavePublicaInvalida(_))
        ));
    }

    /// Payload adulterado = verify falha: a dupla alçada do ARTEFATO não
    /// passa sobre os bytes de um payload alterado, e o SHA-256 diverge.
    #[test]
    fn payload_adulterado_falha_verificacao() {
        let (_tmp, artefato) = construir_teste();
        let mut doc: serde_json::Value = serde_json::from_str(&artefato.json).unwrap();
        doc["payload"]["if_cnpj"] = serde_json::json!("99999999000100");
        let payload: DossierPayload = serde_json::from_value(doc["payload"].clone()).unwrap();
        let aprovacao: DualApproval = serde_json::from_value(doc["dual_approval"].clone()).unwrap();

        let bytes = jcs::canonicalizar_para_bytes(&serde_json::to_value(&payload).unwrap()).unwrap();
        assert!(aprovacao.verificar(&bytes).is_err());
        assert_ne!(artefato.payload_sha256, sha256_hex(&bytes));
    }

    /// O dossie.json é JCS canônico: re-canonicalizar o parse reproduz o
    /// arquivo byte a byte (idempotência).
    #[test]
    fn json_eh_jcs_canonico() {
        let (_tmp, artefato) = construir_teste();
        let doc: serde_json::Value = serde_json::from_str(&artefato.json).unwrap();
        let canon = jcs::canonicalizar(&doc).unwrap();
        assert_eq!(canon, artefato.json);
    }

    /// Custódia no ledger: a entrada existe, o payload_hash é o SHA-256 do
    /// mapping.toml, a hash-chain verifica ponta a ponta e o entry_hash do
    /// artefato é o gravado.
    #[test]
    fn custodia_no_ledger_eh_verificavel() {
        let (tmp, artefato) = construir_teste();
        let ledger = Chain::open(tmp.path().join("ledger.sqlite")).unwrap();
        let report = ledger.verify().unwrap();
        assert!(report.ok, "hash-chain deve verificar após o append");
        assert_eq!(report.entries, 1);

        let doc: serde_json::Value = serde_json::from_str(&artefato.json).unwrap();
        assert_eq!(
            doc["custodia_ledger"]["payload_hash"],
            sha256_hex(MAPPING_TESTE.as_bytes())
        );
        assert_eq!(
            doc["custodia_ledger"]["entry_hash"],
            serde_json::json!(artefato.entry_hash)
        );
    }

    /// O HTML contém as 4 seções e as 12 dimensões (nomes literais).
    #[test]
    fn html_contem_as_4_secoes_e_as_12_dimensoes() {
        let (_tmp, artefato) = construir_teste();
        for marcador in [
            "Seção I — Capa",
            "Seção II — Dimensões de Qualidade",
            "Seção III",
            "Seção IV",
            "Dossiê de Qualidade de Dados",
            "Dimensões de Qualidade da Informação",
            "Linhagem Campo-a-Campo",
            "Prova Criptográfica",
        ] {
            assert!(artefato.html.contains(marcador), "falta {marcador}");
        }
        for (nome, _) in DIMENSOES_RC18 {
            assert!(artefato.html.contains(nome), "falta a dimensão {nome}");
        }
        assert_eq!(artefato.html.matches("<section").count(), 4);
    }

    /// O estilo editorial declarado está no CSS: paper/navy, serif do
    /// sistema, tabular-nums, hairline 1px, ZERO sombras e ZERO fonte
    /// externa/JS.
    #[test]
    fn html_estilo_editorial_declarado() {
        let (_tmp, artefato) = construir_teste();
        for css in [
            "--paper: #fdfcfa",
            "--navy: #1b2a44",
            "Georgia",
            "tabular-nums",
            "@page { size: A4",
            "1px solid",
        ] {
            assert!(artefato.html.contains(css), "falta o CSS {css}");
        }
        for banido in ["shadow", "@font-face", "@import", "url(", "<script", "http://", "https://"] {
            assert!(
                !artefato.html.contains(banido),
                "CSS auto-contido não pode ter {banido}"
            );
        }
    }

    /// O dicionário/linhagem é GERADO DO mapping.toml: cada campo com coluna
    /// de origem, destino, unidade e transform.
    #[test]
    fn html_dicionario_gerado_do_mapping() {
        let (_tmp, artefato) = construir_teste();
        for dado in [
            "Nome do Participante",
            "Saldo",
            "Doc3040/titulo/saldo",
            "reais",
            "trim",
            "Vencimento",
            "Doc3040/titulo/vencimento",
            "date_to_data_base",
            "Documento",
            "Doc3040/titulo/documento",
            "to_lower",
            "3 campo(s) mapeado(s)",
        ] {
            assert!(artefato.html.contains(dado), "falta {dado} no dicionário");
        }
    }

    /// A prova carrega payload JCS, SHA-256, as 2 assinaturas/chaves e a
    /// custódia do ledger.
    #[test]
    fn html_prova_carrega_payload_assinaturas_e_custodia() {
        let (_tmp, artefato) = construir_teste();
        let doc: serde_json::Value = serde_json::from_str(&artefato.json).unwrap();
        let aprovacao: DualApproval = serde_json::from_value(doc["dual_approval"].clone()).unwrap();
        let provas: Vec<&str> = vec![
            artefato.payload_sha256.as_str(),
            aprovacao.chave_primeira.as_str(),
            aprovacao.primeira.as_str(),
            aprovacao.chave_segunda.as_str(),
            aprovacao.segunda.as_str(),
            artefato.entry_hash.as_str(),
            "JCS RFC 8785",
            "4-olhos",
            "verify_strict",
            "hash-chain",
        ];
        for dado in provas {
            assert!(artefato.html.contains(dado), "falta {dado} na prova");
        }
    }

    /// Dados dinâmicos são escapados — header HTML-injetável do mapping não
    /// chega cru na página.
    #[test]
    fn html_escapa_dados_do_mapping() {
        let mapping = MAPPING_TESTE.replace("Saldo", "<b>Saldo</b>");
        let tmp = tempfile::TempDir::new().unwrap();
        let ledger = Chain::open(tmp.path().join("ledger.sqlite")).unwrap();
        let seed_1 = SeedAssinatura::de_hex(SEED_A).unwrap();
        let seed_2 = SeedAssinatura::de_hex(SEED_B).unwrap();
        let artefato = construir(mapping.as_bytes(), &ledger, &seed_1, &seed_2).unwrap();
        assert!(artefato.html.contains("&lt;b&gt;Saldo&lt;/b&gt;"));
        assert!(!artefato.html.contains("<b>Saldo</b>"));
    }

    /// A MESMA seed 2× é rejeitada (4-olhos de verdade) — erro tipado
    /// `ChavesIdenticas` atravessa o `DossieErro`.
    #[test]
    fn mesma_seed_eh_rejeitada() {
        let tmp = tempfile::TempDir::new().unwrap();
        let ledger = Chain::open(tmp.path().join("ledger.sqlite")).unwrap();
        let seed = SeedAssinatura::de_hex(SEED_A).unwrap();
        let err = construir(MAPPING_TESTE.as_bytes(), &ledger, &seed, &seed).unwrap_err();
        match &err {
            DossieErro::Assinatura(SignError::ChavesIdenticas) => {}
            // Dispatch exaustivo — sem wildcard silencioso (doutrina do
            // workspace, válida também nos testes).
            DossieErro::Assinatura(e) => panic!("esperado ChavesIdenticas, obtida {e:?}"),
            DossieErro::Reg(e) => panic!("esperado ChavesIdenticas, obtido Reg: {e}"),
            DossieErro::Ledger(e) => panic!("esperado ChavesIdenticas, obtido Ledger: {e}"),
            DossieErro::Jcs(e) => panic!("esperado ChavesIdenticas, obtido Jcs: {e}"),
            DossieErro::Serde(e) => panic!("esperado ChavesIdenticas, obtido Serde: {e}"),
            DossieErro::Io(e) => panic!("esperado ChavesIdenticas, obtido Io: {e}"),
        }
    }

    /// Cadoc sem suporte no wedge V1 (só 3040) — erro tipado.
    #[test]
    fn cadoc_sem_suporte_erra() {
        let mapping = MAPPING_TESTE.replace("cadoc = \"3040\"", "cadoc = \"2030\"");
        let tmp = tempfile::TempDir::new().unwrap();
        let ledger = Chain::open(tmp.path().join("ledger.sqlite")).unwrap();
        let seed_1 = SeedAssinatura::de_hex(SEED_A).unwrap();
        let seed_2 = SeedAssinatura::de_hex(SEED_B).unwrap();
        let err = construir(mapping.as_bytes(), &ledger, &seed_1, &seed_2).unwrap_err();
        assert!(err.to_string().contains("sem suporte no wedge V1"), "{err}");
    }

    /// Trava 1 do mapping (unidade por campo monetário) atravessa o dossiê —
    /// `RegError::MissingField` propagado, nunca inferência silenciosa.
    #[test]
    fn mapping_sem_unidade_erra() {
        let mapping = MAPPING_TESTE.replace("unit = \"reais\"\n", "");
        let tmp = tempfile::TempDir::new().unwrap();
        let ledger = Chain::open(tmp.path().join("ledger.sqlite")).unwrap();
        let seed_1 = SeedAssinatura::de_hex(SEED_A).unwrap();
        let seed_2 = SeedAssinatura::de_hex(SEED_B).unwrap();
        let err = construir(mapping.as_bytes(), &ledger, &seed_1, &seed_2).unwrap_err();
        match &err {
            DossieErro::Reg(RegError::MissingField { field }) => {
                assert!(field.contains("unit"), "{field}");
            }
            // Dispatch exaustivo — o wildcard silencioso é bug banido por
            // lint (a mesma doutrina do workspace vale nos testes).
            DossieErro::Reg(RegError::InvalidValue { field, reason }) => {
                panic!("esperado MissingField, obtido InvalidValue em {field}: {reason}")
            }
            DossieErro::Reg(RegError::RuleNotImplemented { rule, state }) => {
                panic!("esperado MissingField, obtido RuleNotImplemented {rule}/{state}")
            }
            DossieErro::Ledger(e) => panic!("esperado MissingField, obtido Ledger: {e}"),
            DossieErro::Assinatura(e) => panic!("esperado MissingField, obtido Assinatura: {e}"),
            DossieErro::Jcs(e) => panic!("esperado MissingField, obtido Jcs: {e}"),
            DossieErro::Serde(e) => panic!("esperado MissingField, obtido Serde: {e}"),
            DossieErro::Io(e) => panic!("esperado MissingField, obtido Io: {e}"),
        }
    }

    /// `seed_de_valor`: hex válido → seed com chave pública estável; inválido
    /// → `RegError::InvalidValue`. (`seed_do_ambiente` não é testável: set_var
    /// é `unsafe` na edição 2024 e o workspace proíbe `unsafe_code`.)
    #[test]
    fn seed_de_valor_erro_tipado() {
        let seed = seed_de_valor(SEED_A, "teste").unwrap();
        assert_eq!(seed.chave_publica_hex().len(), 64);
        let err = seed_de_valor("zz", "teste").unwrap_err();
        assert!(err.to_string().contains("inválida"), "{err}");
        let err = seed_de_valor("aabb", "teste").unwrap_err();
        assert!(err.to_string().contains("64 chars"), "{err}");
    }

    /// `gravar` escreve os dois artefatos nos caminhos esperados.
    #[test]
    fn gravar_escreve_dossie_json_e_html() {
        let tmp = tempfile::TempDir::new().unwrap();
        let (_guard, artefato) = construir_teste();
        let (caminho_json, caminho_html) = gravar(tmp.path(), &artefato).unwrap();
        assert!(caminho_json.ends_with("dossie.json"));
        assert!(caminho_html.ends_with("dossie.html"));
        assert_eq!(
            std::fs::read_to_string(&caminho_json).unwrap(),
            artefato.json
        );
        assert_eq!(
            std::fs::read_to_string(&caminho_html).unwrap(),
            artefato.html
        );
    }
}
