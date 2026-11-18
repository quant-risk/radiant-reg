//! Harness M6 — mutation gate de DADOS do wedge V1
//! (docs/blueprint/22-HARNESS-V4.md §M6; no wedge FASE 1 o gate vive em
//! `reg-cli/tests/mutations.rs` e depois migra para módulo interno do
//! `norma-harness`).
//!
//! Contrato do §M6: cada mutação altera **1 campo/regra por vez** e o subject
//! **deve** retornar erro com o `rule_code` **EXATO** esperado. Mutante que
//! dispara erro genérico (código ≠ esperado) **ou nenhum** erro conta como
//! SOBREVIVENTE. **Mutation Score = 100%** é hard gate.
//!
//! O harness mede a si mesmo (P3 do §2): além dos testes do subject-exemplo,
//! há testes que PROVAM que o harness detecta mutante sobrevivente — um
//! subject fraco (valida tudo, menos uma regra) e um subject que mente com
//! erro genérico ambos são reportados pelo NOME, com score < 100%.

#![allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
#![allow(clippy::expect_used)] // testes

// ===================== HARNESS (genérico, M6) =====================

/// Mutação declarada: altera EXATAMENTE 1 campo/regra por vez sobre o input
/// e declara o `rule_code` exato que o subject deve devolver para o mutante.
#[derive(Clone, Copy)]
pub struct Mutacao {
    /// Nome auditável — é ele que aparece na falha do gate se sobreviver.
    pub nome: &'static str,
    /// Aplica a mutação sobre o input bruto (função pura).
    pub aplicar: fn(&str) -> String,
    /// rule_code EXATO exigido para ESTE mutante (erro genérico não mata).
    pub codigo_esperado: &'static str,
}

/// Subject sob teste: `avaliar(input) -> Result<(), Erro>` + extrator do
/// rule_code do erro. Em produção, o subject real é o motor L1-L4 com os
/// rule_codes do catálogo (ADR-0010); aqui é o parser CSV de demonstração.
pub trait SubjectComCodigo {
    type Erro;
    fn avaliar(&self, input: &str) -> Result<(), Self::Erro>;
    /// rule_code exato de um erro do subject (nunca código genérico).
    fn codigo(&self, erro: &Self::Erro) -> String;
}

/// Por que o mutante sobreviveu — as duas classes do §M6.
#[derive(Debug, PartialEq, Eq)]
pub enum MotivoSobrevivencia {
    /// O subject ACEITOU o mutante (nenhum erro) — regra ausente no subject.
    Aceito,
    /// O subject errou, mas com código ≠ esperado (erro genérico).
    CodigoErrado { esperado: String, obtido: String },
}

/// Mutante sobrevivente, nomeado — o gate falha NOMEANDO, nunca com blob.
#[derive(Debug, PartialEq, Eq)]
pub struct MutanteSobrevivente {
    pub nome: &'static str,
    pub motivo: MotivoSobrevivencia,
}

/// Relatório de uma rodada de mutations. Zero-float: score em % inteira.
#[derive(Debug, PartialEq, Eq)]
pub struct RelatorioMutation {
    /// O subject aceita o GOLDEN (baseline)? Se não, NENHUM julgamento é
    /// válido — subject que rejeita o baseline mata mutantes de graça.
    pub golden_ok: bool,
    pub total: usize,
    pub mortas: usize,
    pub sobreviventes: Vec<MutanteSobrevivente>,
}

impl RelatorioMutation {
    /// Mutation Score em percentual INTEIRO (doutrina zero-float §4.5).
    /// Golden quebrado → 0 (não há score honesto sem baseline verde).
    pub fn score_pct(&self) -> usize {
        if self.total == 0 || !self.golden_ok {
            return 0;
        }
        (self.mortas * 100) / self.total
    }

    /// Hard gate do §M6: `Ok(())` só com baseline verde e score 100%.
    /// Falha NOMEANDO cada sobrevivente e o motivo.
    pub fn exigir_score_100(&self) -> Result<(), Vec<String>> {
        let mut falhas = Vec::new();
        if !self.golden_ok {
            falhas.push(
                "golden rejeitado pelo subject — mutantes não são julgáveis (subject quebra no baseline)"
                    .to_string(),
            );
        }
        for s in &self.sobreviventes {
            falhas.push(format!("mutante sobrevivente: {} ({})", s.nome, descrever(&s.motivo)));
        }
        if falhas.is_empty() {
            Ok(())
        } else {
            Err(falhas)
        }
    }
}

fn descrever(motivo: &MotivoSobrevivencia) -> String {
    match motivo {
        MotivoSobrevivencia::Aceito => "subject aceitou o mutante (nenhum erro)".to_string(),
        MotivoSobrevivencia::CodigoErrado { esperado, obtido } => {
            format!("erro genérico: esperado código {esperado:?}, obtido {obtido:?}")
        }
    }
}

/// Roda cada mutação INDIVIDUALMENTE sobre o golden e exige erro com o
/// rule_code exato. Cada mutante é julgado sozinho (1 campo/regra por vez) —
/// mutações nunca se acumulam no mesmo input.
pub fn rodar_mutacoes<S: SubjectComCodigo>(
    golden: &str,
    subject: &S,
    mutacoes: &[Mutacao],
) -> RelatorioMutation {
    let golden_ok = subject.avaliar(golden).is_ok();
    let mut mortas = 0;
    let mut sobreviventes = Vec::new();
    for m in mutacoes {
        let mutante = (m.aplicar)(golden);
        match subject.avaliar(&mutante) {
            Err(e) => {
                let obtido = subject.codigo(&e);
                if obtido == m.codigo_esperado {
                    mortas += 1;
                } else {
                    sobreviventes.push(MutanteSobrevivente {
                        nome: m.nome,
                        motivo: MotivoSobrevivencia::CodigoErrado {
                            esperado: m.codigo_esperado.to_string(),
                            obtido,
                        },
                    });
                }
            }
            Ok(()) => sobreviventes.push(MutanteSobrevivente {
                nome: m.nome,
                motivo: MotivoSobrevivencia::Aceito,
            }),
        }
    }
    RelatorioMutation {
        golden_ok,
        total: mutacoes.len(),
        mortas,
        sobreviventes,
    }
}

// ===================== SUBJECT-EXEMPLO: parser CSV =====================

/// SUBJECT-EXEMPLO: parser CSV estrito de um leiaute simplificado de
/// movimentos (didático para demonstrar o mecanismo M6 — o subject de
/// produção será o validador L1-L4 com rule_codes do catálogo).
///
/// Formato: `;`-separado, UTF-8, sem quoting (leiaute didático), linhas:
/// ```text
/// cnpj;data_base;modalidade;saldo
/// 12345678000195;2026-11;0201;1500
/// ```
/// `saldo` é inteiro de centavos, não-negativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserCsv;

const CABECALHO: &str = "cnpj;data_base;modalidade;saldo";

#[derive(Debug, PartialEq, Eq)]
pub enum ErroCsv {
    Cabecalho,
    Colunas { linha: usize },
    Cnpj { linha: usize },
    DataBase { linha: usize },
    Valor { linha: usize },
    SaldoNegativo { linha: usize },
}

impl SubjectComCodigo for ParserCsv {
    type Erro = ErroCsv;

    fn avaliar(&self, input: &str) -> Result<(), ErroCsv> {
        let mut linhas = input.lines();
        // regra 1: cabeçalho EXATO (CSV-CABECALHO)
        match linhas.next() {
            Some(h) if h == CABECALHO => {}
            Some(_) => return Err(ErroCsv::Cabecalho),
            None => return Err(ErroCsv::Cabecalho),
        }
        for (i, linha) in linhas.enumerate() {
            let n = i + 2; // 1-based; cabeçalho é a linha 1
            let campos: Vec<&str> = linha.split(';').collect();
            // regra 2: cardinalidade de colunas (CSV-COLUNAS)
            if campos.len() != 4 {
                return Err(ErroCsv::Colunas { linha: n });
            }
            // regra 3: CNPJ = 14 dígitos (CSV-CNPJ)
            let cnpj = campos[0];
            if cnpj.len() != 14 || !cnpj.bytes().all(|b| b.is_ascii_digit()) {
                return Err(ErroCsv::Cnpj { linha: n });
            }
            // regra 4: data_base AAAA-MM (CSV-DATA-BASE) — por bytes, nunca
            // slice de string (fronteira de char multibyte não entra em pânico)
            let db = campos[1].as_bytes();
            let db_ok = db.len() == 7
                && db[4] == b'-'
                && db[..4].iter().all(|b| b.is_ascii_digit())
                && db[5..].iter().all(|b| b.is_ascii_digit());
            if !db_ok {
                return Err(ErroCsv::DataBase { linha: n });
            }
            // regra 5: saldo é inteiro de centavos (CSV-VALOR)
            let negativo = campos[3].starts_with('-');
            let digitos = campos[3].strip_prefix('-').unwrap_or(campos[3]);
            if digitos.is_empty() || !digitos.bytes().all(|b| b.is_ascii_digit()) {
                return Err(ErroCsv::Valor { linha: n });
            }
            // regra 6: saldo não-negativo (CSV-SALDO-NEGATIVO)
            if negativo {
                return Err(ErroCsv::SaldoNegativo { linha: n });
            }
        }
        Ok(())
    }

    fn codigo(&self, erro: &ErroCsv) -> String {
        match erro {
            ErroCsv::Cabecalho => "CSV-CABECALHO".to_string(),
            ErroCsv::Colunas { .. } => "CSV-COLUNAS".to_string(),
            ErroCsv::Cnpj { .. } => "CSV-CNPJ".to_string(),
            ErroCsv::DataBase { .. } => "CSV-DATA-BASE".to_string(),
            ErroCsv::Valor { .. } => "CSV-VALOR".to_string(),
            ErroCsv::SaldoNegativo { .. } => "CSV-SALDO-NEGATIVO".to_string(),
        }
    }
}

/// Subject FRACO deliberado: valida tudo, MENOS a regra de saldo não-negativo
/// (regra ausente). Serve para provar que o harness DETETA a mutante que ele
/// deixa passar — é o cenário real de regressão de cobertura de regras.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserCsvSemRegraSaldoNegativo;

impl SubjectComCodigo for ParserCsvSemRegraSaldoNegativo {
    type Erro = ErroCsv;

    fn avaliar(&self, input: &str) -> Result<(), ErroCsv> {
        let p = ParserCsv;
        // reimplementa sem a regra 6 (a regra do meio é "poda" didática):
        match p.avaliar(input) {
            Ok(()) => Ok(()),
            Err(ErroCsv::SaldoNegativo { .. }) => Ok(()), // regra ausente!
            Err(e) => Err(e),
        }
    }

    fn codigo(&self, erro: &ErroCsv) -> String {
        ParserCsv.codigo(erro)
    }
}

/// Subject que MENTE: devolve sempre o mesmo erro genérico (código
/// CSV-CABECALHO) para tudo — inclusive para o golden. O §M6 manda: erro
/// genérico (ou nenhum) conta como sobrevivente; e subject que rejeita o
/// golden não julga nada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubjectGenerico;

impl SubjectComCodigo for SubjectGenerico {
    type Erro = ErroCsv;

    fn avaliar(&self, _input: &str) -> Result<(), ErroCsv> {
        Err(ErroCsv::Cabecalho)
    }

    fn codigo(&self, erro: &ErroCsv) -> String {
        ParserCsv.codigo(erro)
    }
}

// ===================== MUTAÇÕES (1 campo/regra por vez) =====================

const GOLDEN: &str = "cnpj;data_base;modalidade;saldo\n12345678000195;2026-11;0201;1500\n";

const MUTACOES: &[Mutacao] = &[
    Mutacao {
        nome: "cabecalho_campo_renomeado",
        aplicar: |s| s.replace("modalidade", "modality"),
        codigo_esperado: "CSV-CABECALHO",
    },
    Mutacao {
        nome: "coluna_removida_da_linha_de_dados",
        aplicar: |s| s.replace(";0201;1500", ";1500"),
        codigo_esperado: "CSV-COLUNAS",
    },
    Mutacao {
        nome: "cnpj_vazio",
        aplicar: |s| s.replace("12345678000195", ""),
        codigo_esperado: "CSV-CNPJ",
    },
    Mutacao {
        nome: "cnpj_um_digito_a_menos",
        aplicar: |s| s.replace("12345678000195", "1234567800019"),
        codigo_esperado: "CSV-CNPJ",
    },
    Mutacao {
        nome: "data_base_formato_invertido",
        aplicar: |s| s.replace("2026-11", "11/2026"),
        codigo_esperado: "CSV-DATA-BASE",
    },
    Mutacao {
        nome: "saldo_nao_numerico",
        aplicar: |s| s.replace(";1500", ";1x50"),
        codigo_esperado: "CSV-VALOR",
    },
    Mutacao {
        nome: "saldo_negativo",
        aplicar: |s| s.replace(";1500", ";-1500"),
        codigo_esperado: "CSV-SALDO-NEGATIVO",
    },
];

// ===================== TESTES DO GATE =====================

#[test]
fn golden_passa_no_subject_exemplo_sem_mutacao() {
    // Pré-condição do harness: o baseline (sem mutação) é aceito — sem isso,
    // todo "mutante morto" seria morte de graça.
    assert!(ParserCsv.avaliar(GOLDEN).is_ok());
}

#[test]
fn subject_exemplo_mata_todas_as_mutacoes_score_100() {
    let rel = rodar_mutacoes(GOLDEN, &ParserCsv, MUTACOES);
    assert!(rel.golden_ok);
    assert_eq!(rel.total, MUTACOES.len());
    assert_eq!(rel.mortas, MUTACOES.len());
    assert!(rel.sobreviventes.is_empty(), "sobreviventes: {rel:?}");
    assert_eq!(rel.score_pct(), 100);
    assert_eq!(rel.exigir_score_100(), Ok(()));
}

#[test]
fn cada_mutacao_altera_exatamente_um_campo_e_eh_individual() {
    // Prova a doutrina "1 campo/regra por vez": cada mutação aplicada ao
    // golden difere dele em exatamente 1 sub-string visível e cada mutante é
    // julgado isoladamente (o harness nunca acumula mutações).
    for m in MUTACOES {
        let mutante = (m.aplicar)(GOLDEN);
        assert_ne!(mutante, GOLDEN, "mutação {} não alterou nada", m.nome);
        // julgamento isolado: só ela, contra o subject
        let rel = rodar_mutacoes(GOLDEN, &ParserCsv, std::slice::from_ref(m));
        assert_eq!(rel.total, 1);
        assert_eq!(rel.mortas, 1, "mutação {} sobreviveu isolada: {rel:?}", m.nome);
    }
}

#[test]
fn harness_detecta_mutante_sobrevivente_e_o_nomeia() {
    // O subject fraco deixa passar EXATAMENTE o saldo negativo — o gate
    // falha NOMEANDO a mutante sobrevivente (§M6: "falha nomeando").
    let rel = rodar_mutacoes(GOLDEN, &ParserCsvSemRegraSaldoNegativo, MUTACOES);
    assert!(rel.golden_ok);
    assert_eq!(rel.sobreviventes.len(), 1, "sobreviventes: {rel:?}");
    assert_eq!(rel.sobreviventes[0].nome, "saldo_negativo");
    assert_eq!(rel.sobreviventes[0].motivo, MotivoSobrevivencia::Aceito);
    assert_eq!(rel.score_pct(), 85); // 6 mortas de 7 → 85% inteiro
    let falhas = rel.exigir_score_100().expect_err("gate deve falhar");
    assert_eq!(falhas.len(), 1);
    assert!(falhas[0].contains("saldo_negativo"), "falhas: {falhas:?}");
    assert!(falhas[0].contains("subject aceitou o mutante"), "falhas: {falhas:?}");
}

#[test]
fn erro_generico_conta_como_sobrevivente_e_golden_quebrado_invalida_julgamento() {
    // Subject que responde com o mesmo erro genérico para tudo:
    // - rejeita o próprio golden → julgamento invalidado (golden_ok = false);
    // - mutantes com código esperado ≠ genérico viram sobreviventes com
    //   motivo CodigoErrado (§M6: "mutante que dispara erro genérico conta
    //   como sobrevivente").
    // Exceção honesta: a mutação de cabeçalho ESPERA o código genérico que o
    // subject devolve — morte por coincidência, o que o golden quebrado já
    // invalida no gate.
    let rel = rodar_mutacoes(GOLDEN, &SubjectGenerico, MUTACOES);
    assert!(!rel.golden_ok);
    assert_eq!(rel.score_pct(), 0); // sem baseline verde, score não é honesto
    let falhas = rel.exigir_score_100().expect_err("gate deve falhar");
    assert!(falhas[0].contains("golden rejeitado"), "falhas: {falhas:?}");
    assert_eq!(rel.sobreviventes.len(), MUTACOES.len() - 1, "rel: {rel:?}");
    for s in &rel.sobreviventes {
        assert_ne!(s.nome, "cabecalho_campo_renomeado");
        match &s.motivo {
            MotivoSobrevivencia::CodigoErrado { esperado, obtido } => {
                assert_eq!(obtido, "CSV-CABECALHO");
                assert_ne!(esperado, obtido);
            }
            MotivoSobrevivencia::Aceito => panic!("não era para aceitar: {s:?}"),
        }
    }
}

#[test]
fn gate_passa_apenas_com_baseline_verde_e_score_100() {
    // Condição positiva do hard gate: relatório verde → Ok(()).
    let rel = rodar_mutacoes(GOLDEN, &ParserCsv, MUTACOES);
    assert_eq!(rel.exigir_score_100(), Ok(()));
    // E o relatório do subject fraco NUNCA passa (defesa anti-green-wash).
    let fraco = rodar_mutacoes(GOLDEN, &ParserCsvSemRegraSaldoNegativo, MUTACOES);
    assert!(fraco.exigir_score_100().is_err());
    let mentiroso = rodar_mutacoes(GOLDEN, &SubjectGenerico, MUTACOES);
    assert!(mentiroso.exigir_score_100().is_err());
}
