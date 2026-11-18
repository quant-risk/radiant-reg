//! Testes de integração do gen3040 (CMN 4.966 vigente, Cli/Op individualizado).
//! Referências: docs/blueprint/research/01-backend-audit.md §1.8 (bugs do v1),
//! docs/blueprint/20-ARQUITETURA-RUST.md §6; parser de referência:
//! backend/internal/audit/rules/3040.go (ParseDoc3040, attrs case-sensitive).

#![allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
#![allow(clippy::expect_used)] // testes

use reg_cadoc::gen3040::{
    gerar, CadocCode, CliInput, ContInstFinRes4966, Doc3040Input, EstagioAloc, GeneratedDoc,
    OpInput, Venc4966,
};
use reg_core::cnpj::CnpjIdentifier;
use reg_core::error::RegError;
use reg_core::money::{Centavos, MonetaryUnit};
use reg_core::period::DataBase;

#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod fixtures {
    use super::*;

    pub fn db() -> DataBase {
        DataBase::new(2026, 8).unwrap()
    }

    pub fn centavos(raw: &str) -> Centavos {
        Centavos::parse(raw, MonetaryUnit::Reais).unwrap()
    }

    pub fn op(ipoc: &str) -> OpInput {
        OpInput {
            ipoc_raw: ipoc.to_string(),
            contrt: "CT-001".into(),
            natu_op: "01".into(),
            mod_: "0213".into(),
            origem_rec: "0100".into(),
            indx: "01".into(),
            var_camb: "BRL".into(),
            cep: "01001000".into(),
            dt_contr: "2026-01-10".into(),
            dt_venc_op: "2027-01-10".into(),
            class_op: "A".into(),
            prov_consttd: centavos("0"),
            vlr_contr: Some(centavos("1500,00")),
            caract_especial: None,
            venc: Venc4966 {
                v110: Some(centavos("100")),
                v120: Some(centavos("200")),
                v130: Some(centavos("300,50")),
                v140: Some(centavos("0")),
                v150: Some(centavos("500")),
                v160: Some(centavos("0")),
                v165: Some(centavos("0")),
            },
            cont_inst_fin: None,
            dias_atraso: 0,
        }
    }

    pub fn cli(cd: &str) -> CliInput {
        CliInput {
            cd: cd.to_string(),
            tp: "J".into(),
            autorzc: "S".into(),
            porte_cli: "1".into(),
            ini_relact_cli: "2020-01-01".into(),
            nome_cli: None,
            ops: vec![op(&format!("20260812ABC{cd}0001"))],
        }
    }

    pub fn input() -> Doc3040Input {
        Doc3040Input {
            data_base: db(),
            cnpj: CnpjIdentifier::new("12345678", db()).unwrap(),
            remessa: 1,
            parte: 1,
            tp_arq: "F".into(),
            nome_resp: "Fulano de Tal".into(),
            email_resp: "fulano@banco.com.br".into(),
            tel_resp: "1133334444".into(),
            clientes: vec![cli("CLI00001")],
        }
    }
}

/// Encontra o primeiro elemento com `name` e devolve (attrs, dentro_do_elemento).
#[allow(clippy::unwrap_used)]
fn first_start(xml: &[u8], name: &[u8]) -> Vec<(String, String)> {
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).unwrap() {
            quick_xml::events::Event::Start(s) | quick_xml::events::Event::Empty(s) => {
                if s.name().as_ref() == std::str::from_utf8(name).unwrap() {
                    let out = s
                        .attributes()
                        .map(|a| {
                            let a = a.unwrap();
                            (
                                a.key.as_ref().to_string(),
                                quick_xml::escape::unescape(a.value.as_ref())
                                    .unwrap()
                                    .into_owned(),
                            )
                        })
                        .collect();
                    return out;
                }
            }
            quick_xml::events::Event::Eof => panic!("elemento {name:?} não encontrado"),
            quick_xml::events::Event::Text(_)
            | quick_xml::events::Event::CData(_)
            | quick_xml::events::Event::Comment(_)
            | quick_xml::events::Event::Decl(_)
            | quick_xml::events::Event::PI(_)
            | quick_xml::events::Event::DocType(_)
            | quick_xml::events::Event::End(_)
            | quick_xml::events::Event::GeneralRef(_) => {}
        }
        buf.clear();
    }
}

/// Teste 1: roundtrip com o parser das regras — attrs case-sensitive
/// (CNPJ DtBase TotalCli maiúsculos, v110..v165 minúsculos), Cli/Op aceitos.
#[test]
fn roundtrip_attrs_case_sensitive() {
    let doc = gerar(&fixtures::input()).unwrap();
    let xml = doc.xml.clone();

    let root = first_start(&xml, b"Doc3040");
    let get = |attrs: &[(String, String)], k: &str| -> String {
        attrs.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone()).unwrap_or_default()
    };
    assert_eq!(get(&root, "DtBase"), "2026-08");
    assert_eq!(get(&root, "CNPJ"), "12345678");
    assert_eq!(get(&root, "Remessa"), "1");
    assert_eq!(get(&root, "Parte"), "1");
    assert_eq!(get(&root, "TpArq"), "F");
    assert_eq!(get(&root, "TotalCli"), "1");

    let cli = first_start(&xml, b"Cli");
    assert_eq!(get(&cli, "Cd"), "CLI00001");
    assert_eq!(get(&cli, "Tp"), "J");

    let op = first_start(&xml, b"Op");
    assert_eq!(get(&op, "IPOC"), "20260812ABCCLI000010001");
    assert_eq!(get(&op, "Mod"), "0213");
    assert_eq!(get(&op, "NatuOp"), "01");
    assert_eq!(get(&op, "VlrContr"), "150000");

    let venc = first_start(&xml, b"Venc");
    assert_eq!(get(&venc, "v110"), "10000"); // 100,00 reais = 10000 centavos (N19,2)
    // Teste 2: v130/v140 completos — o v1 nunca emitia/auditava (bug §1.8).
    assert_eq!(get(&venc, "v130"), "30050");
    assert_eq!(get(&venc, "v140"), "0");
    assert_eq!(get(&venc, "v165"), "0");
}

/// Teste 3: declaração XML presente; data-base em YYYY-MM (não 2006-01-02).
#[test]
fn declaracao_xml_e_data_base_yyyy_mm() {
    let doc = gerar(&fixtures::input()).unwrap();
    let xml = String::from_utf8(doc.xml).unwrap();
    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"), "{xml}");
    assert!(xml.contains("DtBase=\"2026-08\""), "{xml}");
    assert!(!xml.contains("2026-08-"), "DtBase nunca tem dia");
}

/// Teste 4: escaping de atributos (& < " ').
#[test]
fn escaping_de_atributos() {
    let mut i = fixtures::input();
    i.nome_resp = "Ana <&> \"A\" 'B'".into();
    let doc = gerar(&i).unwrap();
    let xml = String::from_utf8(doc.xml.clone()).unwrap();
    assert!(
        xml.contains("NomeResp=\"Ana &lt;&amp;&gt; &quot;A&quot; &apos;B&apos;\""),
        "{xml}"
    );
    // E o parser de referência recupera o valor original:
    let attrs = first_start(&doc.xml, b"Doc3040");
    let nome = attrs.iter().find(|(k, _)| k == "NomeResp").unwrap().1.clone();
    assert_eq!(nome, "Ana <&> \"A\" 'B'");
}

/// Teste 5: MissingField quando IPOC ausente / valor ausente — nunca default.
#[test]
fn missing_field_nunca_default() {
    let mut i = fixtures::input();
    i.clientes[0].ops[0].ipoc_raw = "   ".into();
    assert_eq!(
        gerar(&i).unwrap_err(),
        RegError::MissingField { field: "Doc3040/Cli/Op/@IPOC".to_string() }
    );

    let mut i = fixtures::input();
    i.clientes[0].ops[0].vlr_contr = None;
    assert_eq!(
        gerar(&i).unwrap_err(),
        RegError::MissingField { field: "Doc3040/Cli/Op/@VlrContr".to_string() }
    );

    // Fio desligado nº 5: TpArq/EmailResp ausentes NUNCA ganham default.
    let mut i = fixtures::input();
    i.tp_arq = String::new();
    assert!(matches!(gerar(&i), Err(RegError::MissingField { field }) if field == "Doc3040/@TpArq"));
    let mut i = fixtures::input();
    i.email_resp = String::new();
    assert!(matches!(gerar(&i), Err(RegError::MissingField { field }) if field.contains("EmailResp")));
}

/// Teste 6: determinismo — mesma entrada → bytes idênticos.
#[test]
fn determinismo() {
    let mut i = fixtures::input();
    i.clientes = vec![fixtures::cli("CLI00002"), fixtures::cli("CLI00001")];
    let a: GeneratedDoc = gerar(&i).unwrap();
    let b = gerar(&i).unwrap();
    assert_eq!(a.xml, b.xml);
    assert_eq!(a.sha256, b.sha256);
    let xml = String::from_utf8(a.xml).unwrap();
    let c1 = xml.find("<Cli Cd=\"CLI00001\"").unwrap();
    let c2 = xml.find("<Cli Cd=\"CLI00002\"").unwrap();
    assert!(c1 < c2, "ordenação determinística por Cd: {xml}");
}

/// Teste 7: CNPJ alfanumérico aceito a partir da data-base jul/2026 (IN 729).
#[test]
fn cnpj_alfanumerico_janela_in729() {
    let mut i = fixtures::input();
    i.data_base = DataBase::new(2026, 7).unwrap();
    i.cnpj = CnpjIdentifier::new("12ABC345", i.data_base).unwrap();
    let doc = gerar(&i).unwrap();
    let xml = String::from_utf8(doc.xml).unwrap();
    assert!(xml.contains("CNPJ=\"12ABC345\""), "{xml}");

    let antes = CnpjIdentifier::new("12ABC345", DataBase::new(2026, 6).unwrap());
    assert!(antes.is_err(), "alfanumérico antes de jul/2026 rejeitado");
}

/// ContInstFinRes4966 + Estagio (DtAlocacao AAAA-MM) são emitidos.
#[test]
fn cont_inst_fin_res4966_e_estagio() {
    let mut i = fixtures::input();
    i.clientes[0].ops[0].dias_atraso = 45; // estágio 4 (mapeamento declarado)
    i.clientes[0].ops[0].cont_inst_fin = Some(ContInstFinRes4966 {
        clas_at_fin: Some("1".into()),
        est_inst_fin: None,
        qtd_inst: Some(10),
        vlr_cont_br: Some(fixtures::centavos("1000")),
        vlr_perda_acum: None,
        vlr_justo: Some(Centavos::parse("990", MonetaryUnit::Centavos).unwrap()),
        rend_mes: Some(Centavos::parse("1234", MonetaryUnit::Centavos).unwrap()),
        pd_est1: None,
        cart_prov_min: None,
        trat_risc: None,
        estagios: vec![EstagioAloc { motivo: Some("01".into()), dt_alocacao: fixtures::db() }],
    });
    let doc = gerar(&i).unwrap();
    let xml = String::from_utf8(doc.xml).unwrap();
    assert!(xml.contains("<ContInstFinRes4966 ClasAtFin=\"1\" EstInstFin=\"4\" QtdInst=\"10\""), "{xml}");
    assert!(xml.contains("VlrContBr=\"100000\""), "{xml}");
    assert!(xml.contains("RendMes=\"1234\""), "{xml}");
    assert!(xml.contains("<Estagio Motivo=\"01\" DtAlocacao=\"2026-08\"/>"), "{xml}");
}

/// GeneratedDoc carrega cadoc e data_base corretos; sha256 é hex de 64 chars.
#[test]
fn generated_doc_shape() {
    let doc = gerar(&fixtures::input()).unwrap();
    assert_eq!(doc.cadoc, CadocCode::C3040);
    assert_eq!(doc.cadoc.as_str(), "3040");
    assert_eq!(doc.data_base, fixtures::db());
    assert_eq!(doc.sha256.len(), 64);
    assert!(doc.sha256.chars().all(|c| c.is_ascii_hexdigit()));
}
