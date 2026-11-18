//! mapping.lock — as 4 travas do 4-olhos como **máquina de estados tipada**.
//!
//! Requisito do blueprint (25-ROADMAP-EXECUCAO.md §0, S2-S3: "mapping.lock
//! com **4-olhos** — tipos compilam; mapping.lock assinado"): um mapping só
//! é custodiado/assinar-doce depois de passar por 4 travas em ORDEM, cada
//! uma EXIGINDO seu evidencial.
//!
//! ## A máquina
//! ```text
//! Rascunho ──trava1──► UnidadesContratadas ──trava2──► DryRunLimpo
//! ──trava3──► AmostraAssinada ──trava4──► MappingFechado
//! ```
//! Os estados são **tipos fantasma**: transição fora de ordem não compila
//! (não existe método `assinar_amostra` num `MappingLock<Rascunho>`) — a
//! negação mais forte é a do compilador. As transições AINDA validam o
//! evidencial em runtime e devolvem [`LockError::EvidenciaFaltante`]
//! **nomeando a trava faltante** ([`Trava`]) quando ele não serve.
//!
//! ## As 4 travas e seus evidenciais
//! | # | Trava | Evidencial exigido |
//! |---|---|---|
//! | 1 | [`Trava::UnidadePorCampo`] | [`ContratoUnidades`] — unidade monetária declarada POR CAMPO (nunca default silencioso — fio desligado nº 5 do v1) |
//! | 2 | [`Trava::ConservacaoZero`] | [`ConservationCheck`] com `depois − antes == Centavos::ZERO` (reg-core::Centavos, aritmética checked — doutrina zero-float §4.5) |
//! | 3 | [`Trava::QuatroOlhos`] | [`crate::sign::DualApproval`] sobre a amostra, de 2 chaves distintas, verificada com verify_strict |
//! | 4 | [`Trava::CustodiaSha256`] | bytes do mapping.toml custodiado → SHA-256 gravado no ledger via [`crate::chain::Chain::append`] (tamper-evident; ancoragem externa na Fase 2 — ADR-0016) |
//!
//! Cada trava aceita registra [`TravaAplicada`] com o SHA-256 do evidencial
//! (JCS-canonicalizado) e o timestamp — o histórico do `MappingFechado` é a
//! trilha completa do 4-olhos.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use reg_core::{Centavos, MonetaryUnit};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::chain::{AppendRequest, Chain, ChainError};
use crate::sign::{DualApproval, SignError};

// ---------------------------------------------------------------------------
// Estados (tipos fantasma)
// ---------------------------------------------------------------------------

/// Estado inicial — mapping rascunho, nenhuma trava aplicada.
#[derive(Debug)]
pub struct Rascunho;

/// Trava 1 aplicada: unidade monetária contratada por campo.
#[derive(Debug)]
pub struct UnidadesContratadas;

/// Trava 2 aplicada: dry-run com conservação contábil ZERO.
#[derive(Debug)]
pub struct DryRunLimpo;

/// Trava 3 aplicada: amostra assinada em dual approval.
#[derive(Debug)]
pub struct AmostraAssinada;

// O estado final (Fechado) é a struct CONCRETA `MappingFechado` — definida
// junto da transição da trava 4, carregando hash + id da custódia sem Option.

/// As 4 travas — o nome EXPLICITO que o erro de negação carrega.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Trava {
    /// Unidade monetária por campo (trava 1).
    UnidadePorCampo,
    /// Conservação contábil com delta ZERO (trava 2).
    ConservacaoZero,
    /// Dual approval 4-olhos sobre a amostra (trava 3).
    QuatroOlhos,
    /// Custódia do mapping.toml via SHA-256 no ledger (trava 4).
    CustodiaSha256,
}

impl std::fmt::Display for Trava {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nome = match self {
            Trava::UnidadePorCampo => "unidade_por_campo",
            Trava::ConservacaoZero => "conservacao_zero",
            Trava::QuatroOlhos => "quatro_olhos",
            Trava::CustodiaSha256 => "custodia_sha256",
        };
        write!(f, "{nome}")
    }
}

#[derive(Debug, Error)]
pub enum LockError {
    /// Negação de transição: o evidencial não serve (ausente, vazio,
    /// inconsistente) — o erro NOMEIA a trava que não foi satisfeita.
    #[error("trava {trava} faltante: {motivo}")]
    EvidenciaFaltante {
        /// Qual das 4 travas foi negada.
        trava: Trava,
        /// Por quê — auditável, com os valores envolvidos.
        motivo: String,
    },
    /// Erro do ledger (custódia da trava 4).
    #[error("ledger recusou a custódia do mapping.lock: {0}")]
    Chain(#[from] ChainError),
    /// Erro de assinatura (dual approval da trava 3).
    #[error("assinatura da trava 4-olhos: {0}")]
    Assinatura(#[from] SignError),
    /// Erro do domínio monetário (ex.: overflow na diferença).
    #[error("domínio monetário: {0}")]
    Reg(#[from] reg_core::RegError),
    /// Evidencial não-canônico (float banido também nos evidenciais).
    #[error("evidencial do mapping.lock: {0}")]
    Jcs(#[from] crate::jcs::JcsError),
    #[error("evidencial serde: {0}")]
    Serde(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Evidenciais
// ---------------------------------------------------------------------------

/// Evidencial da trava 1: unidade monetária declarada por campo.
/// Um mapping sem unidade por campo é o fio desligado nº 5 do v1
/// (defaults silenciosos) — aqui é impossível construir.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContratoUnidades {
    unidades: BTreeMap<String, MonetaryUnit>,
}

impl ContratoUnidades {
    /// Contrato com as unidades por campo (ex.: `"saldo_total": Reais`).
    pub fn novo(unidades: BTreeMap<String, MonetaryUnit>) -> Self {
        Self { unidades }
    }

    /// Contrato propositalmente vazio — serve para TESTAR a negação da
    /// trava 1 (nunca para produção).
    pub fn vazio() -> Self {
        Self { unidades: BTreeMap::new() }
    }

    /// Quantidade de campos contratados.
    pub fn len(&self) -> usize {
        self.unidades.len()
    }

    /// `true` se nenhum campo foi contratado.
    pub fn is_empty(&self) -> bool {
        self.unidades.is_empty()
    }

    /// Nome dos campos contratados (ordem de &str — uso de diagnóstico).
    pub fn campos(&self) -> impl Iterator<Item = &str> + '_ {
        self.unidades.keys().map(String::as_str)
    }
}

/// Evidencial da trava 2: conservação contábil do dry-run. A transição
/// exige `depois − antes == Centavos::ZERO` — toda diferença é erro, nunca
/// arredondamento implícito.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConservationCheck {
    /// Somatório antes do dry-run (Centavos exatos).
    pub antes: Centavos,
    /// Somatório depois do dry-run (Centavos exatos).
    pub depois: Centavos,
}

/// Registro de uma trava aceita — trilha auditável do histórico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TravaAplicada {
    /// Qual trava foi aplicada.
    pub trava: Trava,
    /// SHA-256 hex do evidencial JCS-canonicalizado.
    pub evidencia_sha256: String,
    /// Timestamp RFC3339Nano da aplicação (mesma fonte do ledger).
    pub quando: String,
}

/// mapping.lock em estado `E`. As transições consomem o valor anterior e
/// devolvem o próximo estado — a ordem é garantida pelo sistema de tipos.
#[derive(Debug)]
pub struct MappingLock<Estado> {
    mapping_cod: String,
    historico: Vec<TravaAplicada>,
    _estado: PhantomData<Estado>,
}

impl<Estado> MappingLock<Estado> {
    /// Código do mapping sob lock.
    pub fn mapping_cod(&self) -> &str {
        &self.mapping_cod
    }

    /// Trilhas das travas já aplicadas (em ordem de aplicação).
    pub fn historico(&self) -> &[TravaAplicada] {
        &self.historico
    }

    // Auxiliar compartilhado: registra a trava aplicada no histórico.
    fn registrar(trava: Trava, evidencia_sha256: String) -> TravaAplicada {
        TravaAplicada {
            trava,
            evidencia_sha256,
            quando: crate::chain::agora_rfc3339_nano(),
        }
    }
}

// ---------------------------------------------------------------------------
// Transições — cada uma exige SEU evidencial, com negação tipada
// ---------------------------------------------------------------------------

impl MappingLock<Rascunho> {
    /// Cria um mapping lock em rascunho.
    pub fn novo(mapping_cod: impl Into<String>) -> Self {
        Self {
            mapping_cod: mapping_cod.into(),
            historico: Vec::new(),
            _estado: PhantomData,
        }
    }

    /// **Trava 1 — unidade por campo**: exige [`ContratoUnidades`] com pelo
    /// menos um campo nomeado (não-blank) por unidade.
    pub fn contratar_unidades(
        self,
        contrato: &ContratoUnidades,
    ) -> Result<MappingLock<UnidadesContratadas>, LockError> {
        if contrato.unidades.is_empty() {
            return Err(LockError::EvidenciaFaltante {
                trava: Trava::UnidadePorCampo,
                motivo: "contrato de unidades vazio — declare MonetaryUnit para cada campo monetário do mapping".into(),
            });
        }
        if contrato.unidades.keys().any(|k| k.trim().is_empty()) {
            return Err(LockError::EvidenciaFaltante {
                trava: Trava::UnidadePorCampo,
                motivo: "campo sem nome no contrato de unidades (key em branco)".into(),
            });
        }
        let evidencia = evidencia_sha256(&contrato.unidades)?;
        let aplicada = Self::registrar(Trava::UnidadePorCampo, evidencia);
        Ok(self.proximo(aplicada))
    }
}

impl MappingLock<UnidadesContratadas> {
    /// **Trava 2 — conservação ZERO**: o dry-run só fecha com
    /// `depois − antes == Centavos::ZERO`. Qualquer delta é negado
    /// nomeando a trava, com o valor do delta no motivo.
    pub fn fechar_dry_run(self, check: ConservationCheck) -> Result<MappingLock<DryRunLimpo>, LockError> {
        let delta = check.depois.checked_sub(check.antes)?;
        if delta != Centavos::ZERO {
            return Err(LockError::EvidenciaFaltante {
                trava: Trava::ConservacaoZero,
                motivo: format!(
                    "conservação violada: delta = {} centavos (antes = {}, depois = {}) — exigido Centavos::ZERO",
                    delta.as_i128(),
                    check.antes.as_i128(),
                    check.depois.as_i128()
                ),
            });
        }
        let evidencia = evidencia_sha256(&check)?;
        let aplicada = Self::registrar(Trava::ConservacaoZero, evidencia);
        Ok(self.proximo(aplicada))
    }
}

impl MappingLock<DryRunLimpo> {
    /// **Trava 3 — 4-olhos sobre a amostra**: exige [`DualApproval`] que
    /// verifique (verify_strict) sobre EXATAMENTE os bytes da amostra
    /// informados. Chaves idênticas já são negadas dentro de `DualApproval::nova`.
    pub fn assinar_amostra(
        self,
        aprovacao: &DualApproval,
        amostra: &[u8],
    ) -> Result<MappingLock<AmostraAssinada>, LockError> {
        if let Err(erro) = aprovacao.verificar(amostra) {
            return Err(LockError::EvidenciaFaltante {
                trava: Trava::QuatroOlhos,
                motivo: format!("dual approval não verifica sobre os bytes da amostra: {erro}"),
            });
        }
        let evidencia = evidencia_sha256(&aprovacao)?;
        let aplicada = Self::registrar(Trava::QuatroOlhos, evidencia);
        Ok(self.proximo(aplicada))
    }
}

impl MappingLock<AmostraAssinada> {
    /// **Trava 4 — custódia**: SHA-256 do mapping.toml custodiado gravado
    /// no ledger via `chain.append` (payload = os bytes do toml; metadata
    /// = hash + histórico de travas). Em caso de erro do ledger, nada é
    /// aplicado e o erro sobe tipado — o lock NÃO fecha parcialmente.
    /// O estado final [`MappingFechado`] é concreto: carrega o hash e o id
    /// da custódia SEM `Option`/`expect` — a invariante vive no tipo.
    pub fn custodiar_e_fechar(
        self,
        mapping_toml: &[u8],
        chain: &Chain,
    ) -> Result<MappingFechado, LockError> {
        if mapping_toml.is_empty() {
            return Err(LockError::EvidenciaFaltante {
                trava: Trava::CustodiaSha256,
                motivo: "mapping.toml custodiado está vazio".into(),
            });
        }
        let mapping_sha256 = sha256_hex(mapping_toml);
        let travas: Vec<serde_json::Value> = self
            .historico
            .iter()
            .map(|t| serde_json::Value::String(t.trava.to_string()))
            .collect();
        let metadata = serde_json::json!({
            "mapping_cod": self.mapping_cod,
            "mapping_sha256": mapping_sha256,
            "travas": travas,
        });
        let entry = chain.append(AppendRequest {
            // Tenant da custódia: wiring do CLI (S5/S6) informa; o payload
            // e o metadata carregam a identidade do mapping.
            if_id: "",
            actor: "reg-ledger/mapping_locks",
            action: "mapping.lock",
            target: &self.mapping_cod,
            payload: mapping_toml,
            metadata: Some(&metadata),
        })?;
        let mut historico = self.historico;
        historico.push(Self::registrar(Trava::CustodiaSha256, mapping_sha256.clone()));
        Ok(MappingFechado {
            mapping_cod: self.mapping_cod,
            historico,
            mapping_sha256,
            custodia_entry_id: entry.id,
        })
    }
}

/// Estado final da máquina — mapping.lock FECHADO. Estrutura concreta
/// (não fantasma): o hash custodiado e a entrada do ledger são
/// infalivelmente presentes.
#[derive(Debug)]
pub struct MappingFechado {
    mapping_cod: String,
    historico: Vec<TravaAplicada>,
    mapping_sha256: String,
    custodia_entry_id: i64,
}

impl MappingFechado {
    /// Código do mapping custodiado.
    pub fn mapping_cod(&self) -> &str {
        &self.mapping_cod
    }

    /// Trilha completa das 4 travas, em ordem de aplicação.
    pub fn historico(&self) -> &[TravaAplicada] {
        &self.historico
    }

    /// SHA-256 hex do mapping.toml custodiado (evidencial da trava 4).
    pub fn mapping_sha256(&self) -> &str {
        &self.mapping_sha256
    }

    /// `id` da entrada `mapping.lock` no ledger.
    pub fn custodia_entry_id(&self) -> i64 {
        self.custodia_entry_id
    }
}

impl<Estado> MappingLock<Estado> {
    /// Consome `self` e devolve a máquina no PRÓXIMO estado com a trilha
    /// acumulada — o tipo de destino é inferido da transição chamadora.
    fn proximo<ProximoEstado>(self, aplicada: TravaAplicada) -> MappingLock<ProximoEstado> {
        let mut historico = self.historico;
        historico.push(aplicada);
        MappingLock {
            mapping_cod: self.mapping_cod,
            historico,
            _estado: PhantomData,
        }
    }
}

/// SHA-256 hex do evidencial JCS-canonicalizado (trilha auditável).
/// Evidenciais do crate são JCS-safe por construção; um evidencial
/// estrangeiro com float é negado aqui com erro tipado.
fn evidencia_sha256<E: Serialize>(evidencial: &E) -> Result<String, LockError> {
    let canon = crate::jcs::canonicalizar(&serde_json::to_value(evidencial)?)?;
    Ok(sha256_hex(canon.as_bytes()))
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    hex::encode(digest)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;
    use crate::sign::SeedAssinatura;

    const SEED_A: &str =
        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    const SEED_B: &str =
        "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb";

    fn contrato_cheio() -> ContratoUnidades {
        let mut unidades = BTreeMap::new();
        unidades.insert("saldo_total".into(), MonetaryUnit::Reais);
        unidades.insert("limite".into(), MonetaryUnit::Centavos);
        ContratoUnidades::novo(unidades)
    }

    fn conservacao_zero() -> ConservationCheck {
        ConservationCheck {
            antes: Centavos::parse("12345,67", reg_core::MonetaryUnit::Reais).unwrap(),
            depois: Centavos::parse("12345,67", reg_core::MonetaryUnit::Reais).unwrap(),
        }
    }

    fn dual_aprovada(amostra: &[u8]) -> DualApproval {
        DualApproval::nova(amostra, &SeedAssinatura::de_hex(SEED_A).unwrap(), &SeedAssinatura::de_hex(SEED_B).unwrap())
            .unwrap()
    }

    /// NEGAÇÃO TRAVA 1: contrato vazio → erro nomeando unidade_por_campo.
    #[test]
    fn trava1_sem_unidades_eh_negada() {
        let err = MappingLock::novo("map-3040")
            .contratar_unidades(&ContratoUnidades::vazio())
            .unwrap_err();
        assert!(
            matches!(err, LockError::EvidenciaFaltante { trava: Trava::UnidadePorCampo, .. }),
            "{err}"
        );
        assert!(err.to_string().contains("unidade_por_campo"), "{err}");
    }

    /// NEGAÇÃO TRAVA 1b: campo em branco no contrato é negado.
    #[test]
    fn trava1_campo_em_branco_eh_negado() {
        let mut unidades = BTreeMap::new();
        unidades.insert("   ".into(), MonetaryUnit::Reais);
        let err = MappingLock::novo("m")
            .contratar_unidades(&ContratoUnidades::novo(unidades))
            .unwrap_err();
        assert!(
            matches!(err, LockError::EvidenciaFaltante { trava: Trava::UnidadePorCampo, .. }),
            "{err}"
        );
    }

    /// TRAVA 1 aceita com contrato válido e registra a trilha.
    #[test]
    fn trava1_aceita_contrato_valido() {
        let lock = MappingLock::novo("map-3040").contratar_unidades(&contrato_cheio()).unwrap();
        assert_eq!(lock.mapping_cod(), "map-3040");
        assert_eq!(lock.historico().len(), 1);
        assert_eq!(lock.historico()[0].trava, Trava::UnidadePorCampo);
        assert_eq!(lock.historico()[0].evidencia_sha256.len(), 64);
    }

    /// NEGAÇÃO TRAVA 2: deltas positivos e negativo negados com o valor no
    /// motivo — conservação exige EXATAMENTE Centavos::ZERO.
    #[test]
    fn trava2_delta_diferente_de_zero_eh_negada() {
        // Construtor canônico de Centavos (o campo da tupla é privado no
        // reg-core — parse é a única porta, como deve ser).
        let centavos = |v: i128| Centavos::parse(&v.to_string(), MonetaryUnit::Centavos).unwrap();
        for (antes, depois) in [(100, 101), (100, 99), (0, 1)] {
            let err = MappingLock::novo("m")
                .contratar_unidades(&contrato_cheio())
                .unwrap()
                .fechar_dry_run(ConservationCheck {
                    antes: centavos(antes),
                    depois: centavos(depois),
                })
                .unwrap_err();
            assert!(
                matches!(err, LockError::EvidenciaFaltante { trava: Trava::ConservacaoZero, .. }),
                "{err}"
            );
            assert!(err.to_string().contains("conservacao_zero"), "{err}");
            // O valor do delta está no motivo, com sinal (checked_sub).
            let delta_esperado = depois - antes;
            assert!(
                err.to_string()
                    .contains(&format!("delta = {delta_esperado} centavos")),
                "{err}"
            );
        }
    }

    /// TRAVA 2 aceita com delta ZERO (Centavos exatos).
    #[test]
    fn trava2_aceita_delta_zero() {
        let lock = MappingLock::novo("m")
            .contratar_unidades(&contrato_cheio())
            .unwrap()
            .fechar_dry_run(conservacao_zero())
            .unwrap();
        assert_eq!(lock.historico().len(), 2);
        assert_eq!(lock.historico()[1].trava, Trava::ConservacaoZero);
    }

    /// NEGAÇÃO TRAVA 3: dual approval sobre bytes DIFERENTES da amostra →
    /// erro nomeando quatro_olhos.
    #[test]
    fn trava3_dual_approval_bytes_diferentes_eh_negada() {
        let amostra = b"amostra oficial do mapping";
        let aprov = dual_aprovada(amostra);
        let err = MappingLock::novo("m")
            .contratar_unidades(&contrato_cheio())
            .unwrap()
            .fechar_dry_run(conservacao_zero())
            .unwrap()
            .assinar_amostra(&aprov, b"amostra adulterada")
            .unwrap_err();
        assert!(
            matches!(err, LockError::EvidenciaFaltante { trava: Trava::QuatroOlhos, .. }),
            "{err}"
        );
        assert!(err.to_string().contains("quatro_olhos"), "{err}");
    }

    /// TRAVA 3 aceita dual approval sobre os bytes corretos.
    #[test]
    fn trava3_aceita_dual_approval_correta() {
        let amostra = b"amostra oficial";
        let aprov = dual_aprovada(amostra);
        let lock = MappingLock::novo("m")
            .contratar_unidades(&contrato_cheio())
            .unwrap()
            .fechar_dry_run(conservacao_zero())
            .unwrap()
            .assinar_amostra(&aprov, amostra)
            .unwrap();
        assert_eq!(lock.historico().len(), 3);
        assert_eq!(lock.historico()[2].trava, Trava::QuatroOlhos);
    }

    /// NEGAÇÃO TRAVA 3b: a mesma chave 2× já é negada na construção do
    /// evidencial (erro tipado de sign, propagateado).
    #[test]
    fn trava3_mesma_chave_eh_negada_na_construcao() {
        let amostra = b"amostra";
        let err = DualApproval::nova(
            amostra,
            &SeedAssinatura::de_hex(SEED_A).unwrap(),
            &SeedAssinatura::de_hex(SEED_A).unwrap(),
        )
        .unwrap_err();
        assert!(matches!(err, crate::sign::SignError::ChavesIdenticas));
    }

    /// NEGAÇÃO TRAVA 4: mapping.toml vazio → erro nomeando custodia_sha256.
    #[test]
    fn trava4_toml_vazio_eh_negado() {
        let dir = tempfile::tempdir().unwrap();
        let chain = Chain::open(dir.path().join("ledger.sqlite3")).unwrap();
        let amostra = b"amostra";
        let err = MappingLock::novo("m")
            .contratar_unidades(&contrato_cheio())
            .unwrap()
            .fechar_dry_run(conservacao_zero())
            .unwrap()
            .assinar_amostra(&dual_aprovada(amostra), amostra)
            .unwrap()
            .custodiar_e_fechar(b"", &chain)
            .unwrap_err();
        assert!(
            matches!(err, LockError::EvidenciaFaltante { trava: Trava::CustodiaSha256, .. }),
            "{err}"
        );
        assert!(err.to_string().contains("custodia_sha256"), "{err}");
        // O ledger não ganhou lixo: zero entradas.
        assert_eq!(chain.verify().unwrap().entries, 0);
    }

    /// CAMINHO FELIZ completo: 4 travas em ordem, mapping.lock gravado no
    /// ledger com payload = toml bytes, e a cadeia permanece íntegra.
    #[test]
    fn caminho_feliz_4_travas_com_ledger() {
        let dir = tempfile::tempdir().unwrap();
        let chain = Chain::open(dir.path().join("ledger.sqlite3")).unwrap();
        let amostra = b"amostra oficial do mapping 3040";
        let toml = b"[unidades]\nsaldo_total = \"reais\"\nlimite = \"centavos\"\n";

        let fechado = MappingLock::novo("map-3040")
            .contratar_unidades(&contrato_cheio())
            .unwrap()
            .fechar_dry_run(conservacao_zero())
            .unwrap()
            .assinar_amostra(&dual_aprovada(amostra), amostra)
            .unwrap()
            .custodiar_e_fechar(toml, &chain)
            .unwrap();

        // O SHA custodiado é o SHA-256 dos bytes do toml (recomputado
        // independente no teste).
        let esperado = sha256_hex(toml);
        assert_eq!(fechado.mapping_sha256(), esperado);
        assert_eq!(fechado.historico().len(), 4);
        assert_eq!(fechado.historico()[3].trava, Trava::CustodiaSha256);

        // A entrada está no ledger: a cadeia permanece íntegra e o id de
        // custódia é a primeira entrada.
        let rep = chain.verify().unwrap();
        assert!(rep.ok);
        assert_eq!(rep.entries, 1);
        assert_eq!(fechado.custodia_entry_id(), 1);
    }

    /// Prova de ordem por TIPO (comentário-compilável): as chamadas abaixo,
    /// trocadas de ordem, NÃO COMPILAM — não existe `fechar_dry_run` num
    /// `MappingLock<Rascunho>`, nem `custodiar_e_fechar` num
    /// `MappingLock<DryRunLimpo>`. A negação mais forte é a do compilador;
    /// os testes acima cobrem a negação em runtime do evidencial.
    #[test]
    fn ordem_eh_garantida_pelo_sistema_de_tipos() {
        // O próprio encadeamento abaixo SÓ compila na ordem correta:
        let amostra = b"amostra";
        let _lock = MappingLock::<Rascunho>::novo("prova-tipo")
            .contratar_unidades(&contrato_cheio())
            .unwrap()
            .fechar_dry_run(conservacao_zero())
            .unwrap()
            .assinar_amostra(&dual_aprovada(amostra), amostra)
            .unwrap();
    }
}
