//! Generator do Cadoc 3040 no formato CMN 4.966 **vigente** — clientes/operações
//! individualizados (`<Cli><Op>`), não o `<Agreg>` legado (formato revogado).
//!
//! Referências:
//! - `docs/blueprint/20-ARQUITETURA-RUST.md` §6 (CanonicalDocument, defaults
//!   proibidos) e §4.5 (zero-float);
//! - `docs/blueprint/research/01-backend-audit.md` §1.8 — os 5 bugs do
//!   gen3040 v1 são corrigidos POR DESIGN aqui:
//!     1. formato Agreg revogado → emitimos Cli/Op individualizado (CMN 4.966);
//!     2. data-base em `2006-01-02` no v1 → `DataBase` de reg-core, sempre AAAA-MM;
//!     3. CNPJ numérico fixado → `CnpjIdentifier` com janela temporal IN 729/2026;
//!     4. fio desligado nº 5 (UF="SP", email fake) → campo ausente é
//!        `RegError::MissingField { field }`, NUNCA default silencioso;
//!     5. Venc v130/v140 nunca emitidos/auditados → `Venc4966` modela o
//!        conjunto COMPLETO v110..v165 (Anexo 1).
//! - Campos/atributos conforme a aba `Doc3040` de `3040/SCR3040_Leiaute.xls`
//!   (leiaute oficial re-baixado pós-IN 733/735 pode alterar campos — validação
//!   final via BacenValidatorAdapter, ADR-0018).
//!
//! PENDÊNCIAS DECLARADAS (nunca inventadas — §6 do blueprint):
//! - Campos do leiaute N11,7 (`TaxEft`, `PercIndx`, `TJE`) não são monetários
//!   (N19,2) e não têm tipo dedicado em reg-core (`TaxaDecimal` não exposta);
//!   não modelados aqui — pendência do ingest (CanonicalInput não os carrega).
//! - Elementos `Gar` (garantias), `Inf` (informações adicionais), `Sicor`,
//!   `Perda`, `ConIpocs` declarados no leiaute mas ausentes do CanonicalInput
//!   atual — não emitidos; a exceção `RuleNotImplemented` do reg-core é o
//!   estado correto quando uma regra exigir esses blocos.
//! - Mapa dias-de-atraso → Estágio (0-5): domínios oficiais dos Anexos 41/42
//!   aguardam confirmação pós re-download do leiaute (ADR-0018) — o mapeamento
//!   `estagio_por_dias_atraso` é declarado e testado, sem skip silencioso.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use reg_core::cnpj::CnpjIdentifier;
use reg_core::error::RegError;
use reg_core::ipoc::Ipoc;
use reg_core::money::Centavos;
use reg_core::period::DataBase;
use sha2::{Digest, Sha256};
use std::io::Cursor;

/// Código do cadoc emitido por este módulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadocCode {
    /// Doc 3040 — Dados individualizados de risco de crédito.
    C3040,
}

impl CadocCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            CadocCode::C3040 => "3040",
        }
    }
}

/// Documento gerado — sem ZIP aqui (ZIP é do BacenValidatorAdapter, ADR-0018).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedDoc {
    pub xml: Vec<u8>,
    pub sha256: String,
    pub data_base: DataBase,
    pub cadoc: CadocCode,
}

/// Fluxo de vencimentos COMPLETO do Anexo 1 (CMN 4.966) — o v1 emitia só
/// v110/v120/v150/v160/v165 e nunca auditava v130/v140 (bug §1.8).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Venc4966 {
    pub v110: Option<Centavos>,
    pub v120: Option<Centavos>,
    pub v130: Option<Centavos>,
    pub v140: Option<Centavos>,
    pub v150: Option<Centavos>,
    pub v160: Option<Centavos>,
    pub v165: Option<Centavos>,
}

/// ContInstFinRes4966 — contabilização de instrumentos financeiros
/// (Resolução CMN 4.966/2021, vigente a partir da data-base jan/2025).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContInstFinRes4966 {
    pub clas_at_fin: Option<String>,
    pub est_inst_fin: Option<String>,
    pub qtd_inst: Option<u64>,
    pub vlr_cont_br: Option<Centavos>,
    pub vlr_perda_acum: Option<Centavos>,
    pub vlr_justo: Option<Centavos>,
    /// Rendas do mês (vigência jan/2026) — campo DECLARADO no leiaute.
    pub rend_mes: Option<Centavos>,
    pub pd_est1: Option<String>,
    pub cart_prov_min: Option<String>,
    pub trat_risc: Option<String>,
    /// Motivos de alocação em Estágios (vigência jan/2026).
    pub estagios: Vec<EstagioAloc>,
}

/// Estágio de alocação do instrumento financeiro (Arts. 37 a 39, CMN 4.966).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstagioAloc {
    pub motivo: Option<String>,
    /// Data-base da alocação no Estágio — formato AAAA-MM.
    pub dt_alocacao: DataBase,
}

/// Operação individualizada (`<Op>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpInput {
    /// IPOC cru — vazio = `MissingField` (nunca default). Validado por
    /// `Ipoc::new` de reg-core (regra I14: 15..=67, `[0-9A-Z-]`).
    pub ipoc_raw: String,
    pub contrt: String,
    pub natu_op: String,
    pub mod_: String,
    pub origem_rec: String,
    pub indx: String,
    pub var_camb: String,
    pub cep: String,
    pub dt_contr: String,
    pub dt_venc_op: String,
    pub class_op: String,
    /// Provisão constituída — obrigatória (informar zero quando for o caso).
    pub prov_consttd: Centavos,
    /// Valor contratado — `None` é `MissingField` na emissão (a obrigatoriedade
    /// por modalidade, ex. 0201/0204 isentas, é do validator, não do gerador).
    pub vlr_contr: Option<Centavos>,
    pub caract_especial: Option<String>,
    pub venc: Venc4966,
    pub cont_inst_fin: Option<ContInstFinRes4966>,
    /// Dias de atraso da parcela mais atrasada (`DiaAtraso`, N6) — alimenta
    /// o Estágio quando `cont_inst_fin` está presente.
    pub dias_atraso: u32,
}

/// Cliente individualizado (`<Cli>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliInput {
    /// Identificador `[0-9A-Z]{1,14}` (IN 729/2026) — vazio/inválido = erro.
    pub cd: String,
    /// Tipo do cliente (Anexo 11).
    pub tp: String,
    /// Autorização de consulta (Anexo 20).
    pub autorzc: String,
    pub porte_cli: String,
    /// Início do relacionamento — AAAA-MM-DD.
    pub ini_relact_cli: String,
    /// Nome do cliente no exterior — exigido quando `TpIdentExt` informado.
    pub nome_cli: Option<String>,
    pub ops: Vec<OpInput>,
}

/// Entrada do gerador — shape tipado (tipagem forte aqui, crus no ingest).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doc3040Input {
    pub data_base: DataBase,
    pub cnpj: CnpjIdentifier,
    pub remessa: u32,
    pub parte: u32,
    /// "F" para instituições não particionadas — vazio = `MissingField`
    /// (fio desligado nº 5: nunca inferir).
    pub tp_arq: String,
    pub nome_resp: String,
    pub email_resp: String,
    pub tel_resp: String,
    pub clientes: Vec<CliInput>,
}

/// Falha dura quando campo obrigatório vem vazio — NUNCA default silencioso
/// (fio desligado nº 5 do v1: UF="SP", email fake).
fn require(field: &str, raw: &str) -> Result<String, RegError> {
    let s = raw.trim();
    if s.is_empty() {
        return Err(RegError::MissingField { field: field.to_string() });
    }
    Ok(s.to_string())
}

/// Identificador do cliente: `[0-9A-Z]{1,14}` (IN 729/2026).
fn valida_cd_cliente(cd: &str) -> Result<String, RegError> {
    let s = cd.trim();
    if s.is_empty() {
        return Err(RegError::MissingField { field: "Doc3040/Cli/@Cd".to_string() });
    }
    let len = s.chars().count();
    if !(1..=14).contains(&len) {
        return Err(RegError::InvalidValue {
            field: "Doc3040/Cli/@Cd".to_string(),
            reason: format!("esperado 1..=14 caracteres (IN 729), recebido {len}"),
        });
    }
    if !s.chars().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase()) {
        return Err(RegError::InvalidValue {
            field: "Doc3040/Cli/@Cd".to_string(),
            reason: "caractere fora de [0-9A-Z] (IN 729)".to_string(),
        });
    }
    Ok(s.to_string())
}

/// Estágio de atraso (0-5 por grau de atraso, CMN 4.966 Arts. 37-39).
/// Mapeamento DECLARADO — domínios oficiais dos Anexos 41/42 pendentes de
/// re-download do leiaute (ADR-0018); nenhum estágio é skipado silenciosamente.
#[must_use]
pub const fn estagio_por_dias_atraso(dias: u32) -> u8 {
    if dias == 0 {
        1
    } else if dias <= 15 {
        2
    } else if dias <= 30 {
        3
    } else if dias <= 60 {
        4
    } else {
        5
    }
}

/// Valor monetário no formato BACEN N19,2: inteiro em centavos, 2 casas
/// implícitas, SEM separador de milhar (ex.: R$ 1.234,56 → "123456").
#[must_use]
pub fn n192(c: Centavos) -> String {
    c.as_i128().to_string()
}

/// Ordenação determinística: clientes por `Cd`, operações por IPOC.
fn ordenar_deterministico(clientes: &mut [CliInput]) {
    for cli in &mut *clientes {
        cli.ops.sort_by(|a, b| a.ipoc_raw.cmp(&b.ipoc_raw));
    }
    clientes.sort_by(|a, b| a.cd.cmp(&b.cd));
}

/// Gera o XML do Doc 3040 (formato CMN 4.966 vigente) + sha256.
///
/// # Errors
/// `RegError::MissingField` para qualquer campo obrigatório ausente;
/// `RegError::InvalidValue` para valores fora dos domínios declarados.
pub fn gerar(input: &Doc3040Input) -> Result<GeneratedDoc, RegError> {
    let mut clientes = input.clientes.clone();
    ordenar_deterministico(&mut clientes);

    let cnpj = require("Doc3040/@CNPJ", input.cnpj.as_str())?;
    let tp_arq = require("Doc3040/@TpArq", &input.tp_arq)?;
    let nome_resp = require("Doc3040/@NomeResp", &input.nome_resp)?;
    let email_resp = require("Doc3040/@EmailResp", &input.email_resp)?;
    let tel_resp = require("Doc3040/@TelResp", &input.tel_resp)?;
    let total_cli = clientes.len().to_string();
    let dt_base = input.data_base.to_string(); // AAAA-MM (Display de reg-core)

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(xml_err)?;

    let mut root = BytesStart::new("Doc3040");
    push_attr(&mut root, "DtBase", &dt_base)?;
    push_attr(&mut root, "CNPJ", &cnpj)?;
    push_attr(&mut root, "Remessa", &input.remessa.to_string())?;
    push_attr(&mut root, "Parte", &input.parte.to_string())?;
    push_attr(&mut root, "TpArq", &tp_arq)?;
    push_attr(&mut root, "NomeResp", &nome_resp)?;
    push_attr(&mut root, "EmailResp", &email_resp)?;
    push_attr(&mut root, "TelResp", &tel_resp)?;
    push_attr(&mut root, "TotalCli", &total_cli)?;
    writer.write_event(Event::Start(root)).map_err(xml_err)?;

    for cli in &clientes {
        emit_cli(&mut writer, cli)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("Doc3040")))
        .map_err(xml_err)?;

    let xml = writer.into_inner().into_inner();
    let mut hasher = Sha256::new();
    hasher.update(&xml);
    let sha256 = hex::encode(hasher.finalize());
    Ok(GeneratedDoc {
        xml,
        sha256,
        data_base: input.data_base,
        cadoc: CadocCode::C3040,
    })
}

fn xml_err(e: std::io::Error) -> RegError {
    RegError::InvalidValue { field: "gen3040.xml".to_string(), reason: format!("falha ao serializar XML: {e}") }
}

fn push_attr(start: &mut BytesStart<'static>, name: &str, value: &str) -> Result<(), RegError> {
    // quick-xml 0.42 escapa o VALOR do atributo automaticamente no
    // write_event (`&`, `<`, `"`, `'`) — passar o valor CRU, nunca pré-escapado
    // (duplo escape é bug). O teste de escaping prova o roundtrip.
    start.push_attribute((name, value));
    Ok(())
}

fn start_tag(name: &str) -> BytesStart<'static> {
    BytesStart::new(name.to_string())
}

fn emit_cli(writer: &mut Writer<Cursor<Vec<u8>>>, cli: &CliInput) -> Result<(), RegError> {
    let cd = valida_cd_cliente(&cli.cd)?;
    let mut tag = start_tag("Cli");
    push_attr(&mut tag, "Cd", &cd)?;
    push_attr(&mut tag, "Tp", &require("Doc3040/Cli/@Tp", &cli.tp)?)?;
    push_attr(&mut tag, "Autorzc", &require("Doc3040/Cli/@Autorzc", &cli.autorzc)?)?;
    push_attr(&mut tag, "PorteCli", &require("Doc3040/Cli/@PorteCli", &cli.porte_cli)?)?;
    push_attr(&mut tag, "IniRelactCli", &require("Doc3040/Cli/@IniRelactCli", &cli.ini_relact_cli)?)?;
    if let Some(nome) = &cli.nome_cli {
        push_attr(&mut tag, "NomeCli", nome)?;
    }
    writer.write_event(Event::Start(tag)).map_err(xml_err)?;

    for op in &cli.ops {
        emit_op(writer, op)?;
    }

    writer.write_event(Event::End(BytesEnd::new("Cli".to_string()))).map_err(xml_err)?;
    Ok(())
}

fn emit_op(writer: &mut Writer<Cursor<Vec<u8>>>, op: &OpInput) -> Result<(), RegError> {
    if op.ipoc_raw.trim().is_empty() {
        return Err(RegError::MissingField { field: "Doc3040/Cli/Op/@IPOC".to_string() });
    }
    let ipoc = Ipoc::new(&op.ipoc_raw)?;
    let mut tag = start_tag("Op");
    push_attr(&mut tag, "IPOC", ipoc.as_str())?;
    push_attr(&mut tag, "Contrt", &require("Doc3040/Cli/Op/@Contrt", &op.contrt)?)?;
    push_attr(&mut tag, "NatuOp", &require("Doc3040/Cli/Op/@NatuOp", &op.natu_op)?)?;
    push_attr(&mut tag, "Mod", &require("Doc3040/Cli/Op/@Mod", &op.mod_)?)?;
    push_attr(&mut tag, "OrigemRec", &require("Doc3040/Cli/Op/@OrigemRec", &op.origem_rec)?)?;
    push_attr(&mut tag, "Indx", &require("Doc3040/Cli/Op/@Indx", &op.indx)?)?;
    push_attr(&mut tag, "VarCamb", &require("Doc3040/Cli/Op/@VarCamb", &op.var_camb)?)?;
    push_attr(&mut tag, "CEP", &require("Doc3040/Cli/Op/@CEP", &op.cep)?)?;
    push_attr(&mut tag, "DtContr", &require("Doc3040/Cli/Op/@DtContr", &op.dt_contr)?)?;
    push_attr(&mut tag, "DtVencOp", &require("Doc3040/Cli/Op/@DtVencOp", &op.dt_venc_op)?)?;
    push_attr(&mut tag, "ClassOp", &require("Doc3040/Cli/Op/@ClassOp", &op.class_op)?)?;
    push_attr(&mut tag, "ProvConsttd", &n192(op.prov_consttd))?;
    match op.vlr_contr {
        Some(v) => push_attr(&mut tag, "VlrContr", &n192(v))?,
        None => return Err(RegError::MissingField { field: "Doc3040/Cli/Op/@VlrContr".to_string() }),
    }
    if let Some(ce) = &op.caract_especial {
        push_attr(&mut tag, "CaracEspecial", ce)?;
    }

    // <Venc> completo v110..v165 — o v1 nunca emitia/auditava v130/v140 (bug §1.8).
    writer.write_event(Event::Start(tag)).map_err(xml_err)?;
    emit_venc(writer, &op.venc)?;

    // ContInstFinRes4966 — campos DECLARADOS no leiaute que o input carrega.
    if let Some(cif) = &op.cont_inst_fin {
        emit_cont_inst_fin(writer, cif, op.dias_atraso)?;
    }

    writer.write_event(Event::End(BytesEnd::new("Op".to_string()))).map_err(xml_err)?;
    Ok(())
}

fn emit_venc(writer: &mut Writer<Cursor<Vec<u8>>>, venc: &Venc4966) -> Result<(), RegError> {
    let mut tag = start_tag("Venc");
    for (name, val) in [
        ("v110", venc.v110),
        ("v120", venc.v120),
        ("v130", venc.v130),
        ("v140", venc.v140),
        ("v150", venc.v150),
        ("v160", venc.v160),
        ("v165", venc.v165),
    ] {
        if let Some(c) = val {
            push_attr(&mut tag, name, &n192(c))?;
        }
    }
    writer.write_event(Event::Empty(tag)).map_err(xml_err)?;
    Ok(())
}

fn emit_cont_inst_fin(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    cif: &ContInstFinRes4966,
    dias_atraso: u32,
) -> Result<(), RegError> {
    let mut tag = start_tag("ContInstFinRes4966");
    if let Some(v) = &cif.clas_at_fin {
        push_attr(&mut tag, "ClasAtFin", v)?;
    }
    // Estágio do instrumento: derivado dos dias de atraso (0-5) quando não
    // informado explicitamente — derivação DECLARADA, nunca default mudo.
    let est = cif
        .est_inst_fin
        .clone()
        .unwrap_or_else(|| estagio_por_dias_atraso(dias_atraso).to_string());
    push_attr(&mut tag, "EstInstFin", &est)?;
    if let Some(q) = cif.qtd_inst {
        push_attr(&mut tag, "QtdInst", &q.to_string())?;
    }
    if let Some(v) = cif.vlr_cont_br {
        push_attr(&mut tag, "VlrContBr", &n192(v))?;
    }
    if let Some(v) = cif.vlr_perda_acum {
        push_attr(&mut tag, "VlrPerdaAcum", &n192(v))?;
    }
    if let Some(v) = cif.vlr_justo {
        push_attr(&mut tag, "VlrJusto", &n192(v))?;
    }
    if let Some(v) = cif.rend_mes {
        push_attr(&mut tag, "RendMes", &n192(v))?;
    }
    if let Some(v) = &cif.pd_est1 {
        push_attr(&mut tag, "PdEst1", v)?;
    }
    if let Some(v) = &cif.cart_prov_min {
        push_attr(&mut tag, "CartProvMin", v)?;
    }
    if let Some(v) = &cif.trat_risc {
        push_attr(&mut tag, "TratRisc", v)?;
    }
    writer.write_event(Event::Start(tag)).map_err(xml_err)?;

    for est in &cif.estagios {
        let mut e = start_tag("Estagio");
        if let Some(m) = &est.motivo {
            push_attr(&mut e, "Motivo", m)?;
        }
        push_attr(&mut e, "DtAlocacao", &est.dt_alocacao.to_string())?;
        writer.write_event(Event::Empty(e)).map_err(xml_err)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("ContInstFinRes4966".to_string())))
        .map_err(xml_err)?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;

    fn db() -> DataBase {
        DataBase::new(2026, 8).unwrap()
    }

    fn cnpj() -> CnpjIdentifier {
        CnpjIdentifier::new("12345678", db()).unwrap()
    }

    fn cli_min(cd: &str, ipoc: &str) -> CliInput {
        CliInput {
            cd: cd.to_string(),
            tp: "J".into(),
            autorzc: "S".into(),
            porte_cli: "1".into(),
            ini_relact_cli: "2020-01-01".into(),
            nome_cli: None,
            ops: vec![op_min(ipoc)],
        }
    }

    fn op_min(ipoc: &str) -> OpInput {
        OpInput {
            ipoc_raw: ipoc.to_string(),
            contrt: "C1".into(),
            natu_op: "01".into(),
            mod_: "0213".into(),
            origem_rec: "0100".into(),
            indx: "01".into(),
            var_camb: "BRL".into(),
            cep: "01001000".into(),
            dt_contr: "2026-01-10".into(),
            dt_venc_op: "2027-01-10".into(),
            class_op: "A".into(),
            prov_consttd: Centavos::parse("0", reg_core::money::MonetaryUnit::Reais).unwrap(),
            vlr_contr: Some(Centavos::parse("1234,56", reg_core::money::MonetaryUnit::Reais).unwrap()),
            caract_especial: None,
            venc: Venc4966::default(),
            cont_inst_fin: None,
            dias_atraso: 0,
        }
    }

    fn input(clientes: Vec<CliInput>) -> Doc3040Input {
        Doc3040Input {
            data_base: db(),
            cnpj: cnpj(),
            remessa: 1,
            parte: 1,
            tp_arq: "F".into(),
            nome_resp: "Fulano de Tal".into(),
            email_resp: "fulano@banco.com.br".into(),
            tel_resp: "1133334444".into(),
            clientes,
        }
    }

    fn gerar_xml(i: &Doc3040Input) -> String {
        String::from_utf8(gerar(i).unwrap().xml).unwrap()
    }

    #[test]
    fn declaracao_e_data_base_yyyy_mm() {
        let xml = gerar_xml(&input(vec![cli_min("CLI00001", &"A".repeat(20))]));
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"), "{xml}");
        assert!(xml.contains("DtBase=\"2026-08\""), "{xml}");
        assert!(!xml.contains("2026-08-01"), "data-base NUNCA tem dia (bug v1 §1.8)");
    }

    #[test]
    fn attrs_case_sensitive_como_o_parser_das_regras_espera() {
        let xml = gerar_xml(&input(vec![cli_min("CLI00001", &"A".repeat(20))]));
        for attr in ["DtBase", "CNPJ", "Remessa", "Parte", "TpArq", "NomeResp", "EmailResp", "TelResp", "TotalCli"] {
            assert!(xml.contains(&format!("{attr}=\"")), "attr {attr} ausente: {xml}");
        }
        assert!(xml.contains("<Cli Cd=\"CLI00001\" Tp="), "{xml}");
        assert!(xml.contains("IPOC="), "{xml}");
        assert!(xml.contains("NatuOp=\"01\" Mod=\"0213\""), "{xml}");
        assert!(xml.contains("VlrContr=\"123456\""), "{xml}");
    }

    #[test]
    fn venc_completo_incl_v130_v140_que_o_v1_nunca_emitia() {
        let mut cli = cli_min("CLI00001", &"A".repeat(20));
        let cents = |v: i128| Centavos::parse(&v.to_string(), reg_core::money::MonetaryUnit::Centavos).unwrap();
        cli.ops[0].venc = Venc4966 {
            v110: Some(cents(110)),
            v120: Some(cents(120)),
            v130: Some(cents(130)),
            v140: Some(cents(140)),
            v150: Some(cents(150)),
            v160: Some(cents(160)),
            v165: Some(cents(165)),
        };
        let xml = gerar_xml(&input(vec![cli]));
        for (name, v) in [
            ("v110", 110), ("v120", 120), ("v130", 130), ("v140", 140),
            ("v150", 150), ("v160", 160), ("v165", 165),
        ] {
            assert!(xml.contains(&format!("{name}=\"{v}\"")), "{name} ausente: {xml}");
        }
    }

    #[test]
    fn escaping_de_atributos() {
        let mut i = input(vec![cli_min("CLI00001", &"A".repeat(20))]);
        i.nome_resp = "Mal<&>\"Quoted\"'Name'".into();
        let xml = gerar_xml(&i);
        assert!(xml.contains("NomeResp=\"Mal&lt;&amp;&gt;&quot;Quoted&quot;&apos;Name&apos;\""), "{xml}");
        // Parse de volta com quick-xml recupera o valor original:
        let mut reader = quick_xml::Reader::from_reader(xml.as_bytes());
        let mut found = None;
        let mut buf = Vec::new();
        loop {
            let ev = reader.read_event_into(&mut buf).unwrap();
            match ev {
                Event::Start(s) | Event::Empty(s) if s.name().as_ref() == "Doc3040" => {
                    for a in s.attributes() {
                        let a = a.unwrap();
                        if a.key.as_ref() == "NomeResp" {
                            found = Some(quick_xml::escape::unescape(a.value.as_ref()).unwrap().into_owned());
                        }
                    }
                    break;
                }
                Event::Eof => break,
                Event::Start(_) | Event::End(_) | Event::Empty(_) | Event::Text(_)
                | Event::CData(_) | Event::Comment(_) | Event::Decl(_) | Event::PI(_)
                | Event::DocType(_) | Event::GeneralRef(_) => {}
            }
        }
        assert_eq!(found.as_deref(), Some("Mal<&>\"Quoted\"'Name'"));
    }

    #[test]
    fn missing_field_nunca_default() {
        // IPOC ausente (vazio):
        let e = gerar(&input(vec![cli_min("CLI00001", "")])).unwrap_err();
        assert_eq!(e, RegError::MissingField { field: "Doc3040/Cli/Op/@IPOC".to_string() });
        // Valor contratado ausente:
        let mut cli = cli_min("CLI00001", &"A".repeat(20));
        cli.ops[0].vlr_contr = None;
        let e = gerar(&input(vec![cli])).unwrap_err();
        assert_eq!(e, RegError::MissingField { field: "Doc3040/Cli/Op/@VlrContr".to_string() });
        // TpArq ausente — NUNCA vira "F" silenciosamente (fio desligado nº 5):
        let mut i = input(vec![cli_min("CLI00001", &"A".repeat(20))]);
        i.tp_arq = String::new();
        let e = gerar(&i).unwrap_err();
        assert_eq!(e, RegError::MissingField { field: "Doc3040/@TpArq".to_string() });
        // EmailResp ausente — NUNCA email fake:
        let mut i = input(vec![cli_min("CLI00001", &"A".repeat(20))]);
        i.email_resp = String::new();
        assert!(matches!(gerar(&i), Err(RegError::MissingField { field } ) if field.contains("EmailResp")));
        // Cd do cliente fora do domínio IN 729:
        assert!(valida_cd_cliente("CLI-01!").is_err());
    }

    #[test]
    fn determinismo_bytes_identicos() {
        let mut clientes = vec![
            cli_min("CLI00002", &"B".repeat(21)),
            cli_min("CLI00001", &"C".repeat(18)),
            cli_min("CLI00001", &"A".repeat(19)),
        ];
        // Entrada desordenada de propósito.
        clientes.reverse();
        let i = input(clientes);
        let a = gerar(&i).unwrap();
        let b = gerar(&i).unwrap();
        assert_eq!(a.xml, b.xml);
        assert_eq!(a.sha256, b.sha256);
        let xml = String::from_utf8(a.xml).unwrap();
        let pos_cli1 = xml.find("<Cli Cd=\"CLI00001\"").unwrap();
        let pos_cli2 = xml.find("<Cli Cd=\"CLI00002\"").unwrap();
        assert!(pos_cli1 < pos_cli2, "clientes ordenados por Cd");
        let op_a = xml.find(&format!("IPOC=\"{}\"", "A".repeat(19))).unwrap();
        let op_c = xml.find(&format!("IPOC=\"{}\"", "C".repeat(18))).unwrap();
        let op_b = xml.find(&format!("IPOC=\"{}\"", "B".repeat(21))).unwrap();
        // As duas Ops do Cli00001 vêm em ordem de IPOC; CLI00002 depois.
        assert!(pos_cli1 < op_a && op_a < op_c && op_c < pos_cli2 && pos_cli2 < op_b, "{xml}");
    }

    #[test]
    fn cnpj_alfanumerico_aceito_a_partir_de_2026_07() {
        let mut i = input(vec![cli_min("CLI00001", &"A".repeat(20))]);
        i.data_base = DataBase::new(2026, 7).unwrap();
        i.cnpj = CnpjIdentifier::new("12ABC345", i.data_base).unwrap();
        let xml = gerar_xml(&i);
        assert!(xml.contains("CNPJ=\"12ABC345\""), "{xml}");
        // Antes de jul/2026 o formato é numérico — rejeitado:
        let antiga = CnpjIdentifier::new("12ABC345", DataBase::new(2026, 6).unwrap());
        assert!(antiga.is_err());
    }

    #[test]
    fn n192_formato_bacen() {
        let f = |raw: &str| n192(Centavos::parse(raw, reg_core::money::MonetaryUnit::Reais).unwrap());
        assert_eq!(f("1234,56"), "123456");
        assert_eq!(f("0"), "0");
        assert_eq!(f("1000000000"), "100000000000");
        assert_eq!(f("-1,50"), "-150");
    }

    #[test]
    fn cont_inst_fin_res4966_e_estagio() {
        let mut cli = cli_min("CLI00001", &"A".repeat(20));
        cli.ops[0].dias_atraso = 45;
        cli.ops[0].cont_inst_fin = Some(ContInstFinRes4966 {
            clas_at_fin: Some("1".into()),
            est_inst_fin: None, // derivado dos dias de atraso → 4
            qtd_inst: Some(10),
            vlr_cont_br: Some(Centavos::parse("1000", reg_core::money::MonetaryUnit::Reais).unwrap()),
            vlr_perda_acum: None,
            vlr_justo: Some(Centavos::parse("990", reg_core::money::MonetaryUnit::Centavos).unwrap()),
            rend_mes: Some(Centavos::parse("1234", reg_core::money::MonetaryUnit::Centavos).unwrap()),
            pd_est1: None,
            cart_prov_min: None,
            trat_risc: None,
            estagios: vec![EstagioAloc { motivo: Some("01".into()), dt_alocacao: db() }],
        });
        let xml = gerar_xml(&input(vec![cli]));
        assert!(xml.contains("<ContInstFinRes4966 ClasAtFin=\"1\" EstInstFin=\"4\" QtdInst=\"10\""), "{xml}");
        assert!(xml.contains("VlrContBr=\"100000\""), "{xml}");
        assert!(xml.contains("RendMes=\"1234\""), "{xml}");
        assert!(xml.contains("<Estagio Motivo=\"01\" DtAlocacao=\"2026-08\"/>"), "{xml}");
    }

    #[test]
    fn estagio_por_dias_atraso_faixas() {
        assert_eq!(estagio_por_dias_atraso(0), 1);
        assert_eq!(estagio_por_dias_atraso(1), 2);
        assert_eq!(estagio_por_dias_atraso(15), 2);
        assert_eq!(estagio_por_dias_atraso(16), 3);
        assert_eq!(estagio_por_dias_atraso(30), 3);
        assert_eq!(estagio_por_dias_atraso(31), 4);
        assert_eq!(estagio_por_dias_atraso(60), 4);
        assert_eq!(estagio_por_dias_atraso(61), 5);
    }

    #[test]
    fn ipoc_validado_pelo_reg_core() {
        let mut i = input(vec![cli_min("CLI00001", "curto")]);
        assert!(gerar(&i).is_err(), "IPOC com menos de 15 chars rejeitado (regra I14)");
        i.clientes[0].ops[0].ipoc_raw = "ABC".repeat(5); // 15 chars — mínimo válido
        assert!(gerar(&i).is_ok());
    }
}
