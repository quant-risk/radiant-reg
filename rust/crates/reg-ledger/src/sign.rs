//! Assinatura Ed25519 (ed25519-dalek 2) — dossiê RC Conjunta 18/2025 e
//! dual approval 4-olhos (25-ROADMAP-EXECUCAO.md §0 S2-S3/S5: "mapping.lock
//! com 4-olhos", "Dossiê A4/JSON assinado Ed25519").
//!
//! Pipeline do dossiê: [`DossierPayload`] → JSON JCS ([`crate::jcs`]) →
//! bytes → Ed25519 → hex. Assina-se EXATAMENTE o JSON canônico — qualquer
//! diferença de espaçamento/ordem de chave entre produtores invalida a
//! assinatura, por design: o canônico é o contrato de bytes.
//!
//! Segredo: [`SeedAssinatura`] encapsula a seed privada de 32 bytes com
//! `zeroize` no drop (ADR-0015 — material de assinatura nunca fica em
//! memória após o uso; nunca em `Debug`/`Display`). A seed NUNCA expõe
//! `Clone`/`Copy`; ed25519-dalek 2.2 roda com a feature `zeroize` (default)
//! para o material interno do `SigningKey`.
//!
//! [`DualApproval`]: duas assinaturas Ed25519 sobre os MESMOS bytes, de
//! DUAS chaves distintas — a mesma chave assinando duas vezes é rejeitada
//! com erro tipado ([`SignError::ChavesIdenticas`]), pinnado em teste.
//!
//! Limite de confiança (honesto): a assinatura prova INTEGRIDADE e vínculo
//! com a chave pública gravada no artefato; quem controla a chave é questão
//! de custódia (Secrets Manager/Vault na Fase 2 — ADR-0015), não desta
//! primitiva.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::Zeroize;

use crate::jcs;

#[derive(Debug, Error)]
pub enum SignError {
    /// Dual approval exige 2 keypairs DISTINTOS — mesma chave 2× é
    /// rejeitada (4-olhos de verdade).
    #[error("dual approval exige chaves distintas — a mesma chave foi usada 2×")]
    ChavesIdenticas,
    /// Assinatura (ou bytes) não confere — recomputação do JCS divergiu
    /// ou a assinatura não é válida (verify_strict).
    #[error("assinatura inválida sobre os bytes canônicos informados")]
    AssinaturaInvalida,
    /// Chave pública não cabe/parseia como ponto Ed25519.
    #[error("chave pública ed25519 inválida: {0}")]
    ChavePublicaInvalida(String),
    /// Seed em hex inválido (caracteres não-hex).
    #[error("seed privada em hex inválido: {0}")]
    SeedHexInvalida(#[from] hex::FromHexError),
    /// Seed com tamanho errado (esperado 32 bytes).
    #[error("seed privada deve ter 32 bytes (recebidos {0})")]
    SeedTamanhoInvalido(usize),
    /// Payload não-canônico (ex.: float banido pelo JCS).
    #[error("payload do dossiê: {0}")]
    Jcs(#[from] jcs::JcsError),
    /// Payload não serializável em JSON (não deve ocorrer com tipos do
    /// crate, mas o erro é tipado para o chamador).
    #[error("payload serde: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Seed privada Ed25519 — zeroize no drop, `Debug` redigido, sem
/// `Clone`/`Copy` (ADR-0015). A única forma de sair dela é a chave
/// pública e a assinatura.
pub struct SeedAssinatura([u8; 32]);

impl SeedAssinatura {
    /// Seed a partir de 32 bytes crus (ex.: material de KMS).
    pub const fn de_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Seed a partir de hex (64 caracteres). Espaços em branco nas pontas
    /// são tolerados.
    pub fn de_hex(s: &str) -> Result<Self, SignError> {
        let bytes = hex::decode(s.trim())?;
        if bytes.len() != 32 {
            return Err(SignError::SeedTamanhoInvalido(bytes.len()));
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes);
        Ok(Self(seed))
    }

    /// Chave pública correspondente em hex — a identidade auditável.
    pub fn chave_publica_hex(&self) -> String {
        hex::encode(SigningKey::from_bytes(&self.0).verifying_key())
    }
}

impl Drop for SeedAssinatura {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl std::fmt::Debug for SeedAssinatura {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // A seed NUNCA aparece em logs/traces (ADR-0015: scrubbing).
        write!(f, "SeedAssinatura(***redigida***)")
    }
}

/// Payload do dossiê RC Conjunta 18/2025 — o que é assinado. Campos em
/// texto/inteiros (JCS-safe); hashes em hex minúsculo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DossierPayload {
    /// Código do documento (ex. "3040", "2030").
    pub doc_cod: String,
    /// CNPJ da IF remetente (14 dígitos, sem máscara).
    pub if_cnpj: String,
    /// Data-base do documento em AAAA-MM.
    pub data_base: String,
    /// SHA-256 hex do mapping.toml custodiado (trava 4 de
    /// [`crate::mapping_locks`]) — amarra o dossiê ao mapping.lock.
    pub mapping_sha256: String,
    /// Artefatos referenciados com seu hash (ZIP validado, XML, etc).
    pub artefatos: Vec<ArtefatoRef>,
}

/// Referência de artefato dentro do dossiê.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtefatoRef {
    /// Nome canônico do artefato (ex. "DOC3040_VALIDADO.zip").
    pub nome: String,
    /// SHA-256 hex do conteúdo (no 3040: sobre o `*_VALIDADO.zip` — ADR-0018).
    pub sha256: String,
}

/// Dossiê assinado — payload + assinatura + chave pública, tudo hex/JCS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DossierAssinado {
    /// Payload original (re-canonicalizado na verificação).
    pub payload: DossierPayload,
    /// SHA-256 hex do JSON canônico assinado (identificação do artefato).
    pub payload_sha256: String,
    /// Chave pública do signatário em hex.
    pub chave_publica_hex: String,
    /// Assinatura Ed25519 (r||s) em hex (128 caracteres).
    pub assinatura_hex: String,
}

/// Assina o dossiê: JCS → bytes → Ed25519 → hex (determinístico — Ed25519
/// não usa RNG na assinatura; mesma seed+payload = mesma assinatura).
pub fn assinar_dossie(
    payload: &DossierPayload,
    seed: &SeedAssinatura,
) -> Result<DossierAssinado, SignError> {
    let canonico = jcs::canonicalizar(&serde_json::to_value(payload)?)?;
    let sk = SigningKey::from_bytes(&seed.0);
    let assinatura = sk.sign(canonico.as_bytes());
    Ok(DossierAssinado {
        payload: payload.clone(),
        payload_sha256: sha256_hex(canonico.as_bytes()),
        chave_publica_hex: hex::encode(sk.verifying_key()),
        assinatura_hex: hex::encode(assinatura.to_bytes()),
    })
}

/// Verifica o dossiê: re-canonicaliza o payload e confere a assinatura com
/// `verify_strict` (rejeita assinaturas maleáveis além da verificação base).
/// Falha-certa: qualquer byte divergente invalida.
pub fn verificar_dossie(dossie: &DossierAssinado) -> Result<(), SignError> {
    let canonico = jcs::canonicalizar(&serde_json::to_value(&dossie.payload)?)?;
    let vk = chave_publica_de_hex(&dossie.chave_publica_hex)?;
    let assinatura = assinatura_de_hex(&dossie.assinatura_hex)?;
    vk.verify_strict(canonico.as_bytes(), &assinatura)
        .map_err(|_| SignError::AssinaturaInvalida)
}

/// Duas assinaturas independentes (4-olhos) sobre os MESMOS bytes, de duas
/// chaves distintas. Serializável — vai junto da evidência no ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualApproval {
    /// Assinatura da 1ª chave (hex r||s).
    pub primeira: String,
    /// Assinatura da 2ª chave (hex r||s).
    pub segunda: String,
    /// Chave pública da 1ª assinatura (hex).
    pub chave_primeira: String,
    /// Chave pública da 2ª assinatura (hex).
    pub chave_segunda: String,
}

impl DualApproval {
    /// Produz a dupla assinatura. **Mesma chave 2× = [`SignError::ChavesIdenticas`]**
    /// — a comparação é pela chave PÚBLICA (identidade), não pelos bytes da seed.
    pub fn nova(bytes: &[u8], primeira: &SeedAssinatura, segunda: &SeedAssinatura) -> Result<Self, SignError> {
        let ka = SigningKey::from_bytes(&primeira.0);
        let kb = SigningKey::from_bytes(&segunda.0);
        if ka.verifying_key().as_bytes() == kb.verifying_key().as_bytes() {
            return Err(SignError::ChavesIdenticas);
        }
        Ok(Self {
            primeira: hex::encode(ka.sign(bytes).to_bytes()),
            segunda: hex::encode(kb.sign(bytes).to_bytes()),
            chave_primeira: hex::encode(ka.verifying_key()),
            chave_segunda: hex::encode(kb.verifying_key()),
        })
    }

    /// Re-verifica as DUAS assinaturas sobre os bytes informados com
    /// `verify_strict`. Falha-certa para qualquer divergência.
    pub fn verificar(&self, bytes: &[u8]) -> Result<(), SignError> {
        let k1 = chave_publica_de_hex(&self.chave_primeira)?;
        let k2 = chave_publica_de_hex(&self.chave_segunda)?;
        let s1 = assinatura_de_hex(&self.primeira)?;
        let s2 = assinatura_de_hex(&self.segunda)?;
        k1.verify_strict(bytes, &s1).map_err(|_| SignError::AssinaturaInvalida)?;
        k2.verify_strict(bytes, &s2).map_err(|_| SignError::AssinaturaInvalida)?;
        Ok(())
    }
}

fn chave_publica_de_hex(hex_str: &str) -> Result<VerifyingKey, SignError> {
    let bytes: [u8; 32] = hex::decode(hex_str)
        .map_err(|_| SignError::ChavePublicaInvalida("hex inválido".into()))?
        .try_into()
        .map_err(|_| SignError::ChavePublicaInvalida(format!("esperados 32 bytes em {}", hex_str.len())))?;
    VerifyingKey::from_bytes(&bytes).map_err(|e| SignError::ChavePublicaInvalida(e.to_string()))
}

fn assinatura_de_hex(hex_str: &str) -> Result<Signature, SignError> {
    let bytes: [u8; 64] = hex::decode(hex_str)
        .map_err(|_| SignError::AssinaturaInvalida)?
        .try_into()
        .map_err(|_| SignError::AssinaturaInvalida)?;
    Ok(Signature::from_bytes(&bytes))
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    hex::encode(digest)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;

    /// Seeds FIXAS e distintas — determinismo é doutrina (reprodução de
    /// auditoria não pode depender de RNG).
    const SEED_A: &str =
        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    const SEED_B: &str =
        "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb";
    // (RFC 8032 §7.2 TEST 1/TEST 3 — vetores públicos de Ed25519.)
    const SEED_A_REPETIDA: &str =
        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";

    fn seed_hex(s: &str) -> SeedAssinatura {
        SeedAssinatura::de_hex(s).unwrap()
    }

    fn payload_simples() -> DossierPayload {
        DossierPayload {
            doc_cod: "3040".into(),
            if_cnpj: "12345678000195".into(),
            data_base: "2026-11".into(),
            mapping_sha256: "ab".repeat(32),
            artefatos: vec![ArtefatoRef {
                nome: "DOC3040_VALIDADO.zip".into(),
                sha256: "cd".repeat(32),
            }],
        }
    }

    /// Assinar → verificar com os mesmos bytes canônicos: ok.
    #[test]
    fn assinar_e_verificar_dossie() {
        let seed = seed_hex(SEED_A);
        let dossie = assinar_dossie(&payload_simples(), &seed).unwrap();
        verificar_dossie(&dossie).unwrap();
        assert_eq!(dossie.chave_publica_hex, seed.chave_publica_hex());
        assert_eq!(dossie.assinatura_hex.len(), 128);
    }

    /// Ed25519 é determinístico: mesma seed+payload → mesma assinatura hex.
    #[test]
    fn assinatura_eh_deterministica() {
        let seed = seed_hex(SEED_A);
        let d1 = assinar_dossie(&payload_simples(), &seed).unwrap();
        let d2 = assinar_dossie(&payload_simples(), &seed).unwrap();
        assert_eq!(d1.assinatura_hex, d2.assinatura_hex);
        assert_eq!(d1.payload_sha256, d2.payload_sha256);
    }

    /// Payload alterado (um dígito do CNPJ) → verificação falha.
    #[test]
    fn payload_adulterado_falha() {
        let seed = seed_hex(SEED_A);
        let mut dossie = assinar_dossie(&payload_simples(), &seed).unwrap();
        dossie.payload.if_cnpj = "12345678000196".into();
        assert!(matches!(
            verificar_dossie(&dossie),
            Err(SignError::AssinaturaInvalida)
        ));
    }

    /// Assinatura alterada → verificação falha (verify_strict).
    #[test]
    fn assinatura_adulterada_falha() {
        let seed = seed_hex(SEED_A);
        let mut dossie = assinar_dossie(&payload_simples(), &seed).unwrap();
        let mut bytes = hex::decode(&dossie.assinatura_hex).unwrap();
        bytes[0] ^= 0x01;
        dossie.assinatura_hex = hex::encode(bytes);
        assert!(matches!(
            verificar_dossie(&dossie),
            Err(SignError::AssinaturaInvalida)
        ));
    }

    /// Chave trocada pela de outro signatário → falha (vínculo payload↔chave).
    #[test]
    fn chave_trocada_falha() {
        let dossie = assinar_dossie(&payload_simples(), &seed_hex(SEED_A)).unwrap();
        let mut outro = dossie.clone();
        outro.chave_publica_hex = seed_hex(SEED_B).chave_publica_hex();
        assert!(matches!(
            verificar_dossie(&outro),
            Err(SignError::AssinaturaInvalida) | Err(SignError::ChavePublicaInvalida(_))
        ));
    }

    /// Roundtrip serde do dossiê assinado continua verificando.
    #[test]
    fn roundtrip_serde_mantem_verificacao() {
        let dossie = assinar_dossie(&payload_simples(), &seed_hex(SEED_A)).unwrap();
        let json = serde_json::to_string(&dossie).unwrap();
        let de: DossierAssinado = serde_json::from_str(&json).unwrap();
        assert_eq!(de, dossie);
        verificar_dossie(&de).unwrap();
    }

    /// Dual approval com 2 chaves distintas: assina e re-verifica.
    #[test]
    fn dual_approval_com_chaves_distintas() {
        let bytes = b"amostra do mapping: doc3040, data-base 2026-11";
        let aprov = DualApproval::nova(bytes, &seed_hex(SEED_A), &seed_hex(SEED_B)).unwrap();
        aprov.verificar(bytes).unwrap();
        // Mesmos bytes: ok. Bytes diferentes: falha em ambas.
        aprov.verificar(b"amostra adulterada").unwrap_err();
    }

    /// 4-OLHOS: a MESMA chave assinando 2× é rejeitada — erro tipado
    /// ChavesIdenticas (requisito da S5, teste de negação).
    #[test]
    fn dual_approval_mesma_chave_eh_rejeitada() {
        let bytes = b"qualquer bytes";
        let err = DualApproval::nova(bytes, &seed_hex(SEED_A), &seed_hex(SEED_A_REPETIDA))
            .unwrap_err();
        assert!(matches!(err, SignError::ChavesIdenticas));
        assert!(err.to_string().contains("distintas"), "{err}");
    }

    /// DualApproval serializável sobrevive a roundtrip e re-verifica.
    #[test]
    fn dual_approval_roundtrip_serde() {
        let bytes = b"evidencia da amostra";
        let aprov = DualApproval::nova(bytes, &seed_hex(SEED_A), &seed_hex(SEED_B)).unwrap();
        let json = serde_json::to_string(&aprov).unwrap();
        let de: DualApproval = serde_json::from_str(&json).unwrap();
        de.verificar(bytes).unwrap();
    }

    /// Seed inválida (hex ímpar / tamanho errado) → erro tipado.
    #[test]
    fn seed_invalida_erro_tipado() {
        assert!(matches!(
            SeedAssinatura::de_hex("zz-nao-hex"),
            Err(SignError::SeedHexInvalida(_))
        ));
        let err = SeedAssinatura::de_hex("aabb").unwrap_err();
        assert!(matches!(err, SignError::SeedTamanhoInvalido(2)), "{err}");
    }

    /// `Debug` da seed NÃO vaza material (ADR-0015: scrubbing em
    /// tracing/logs) — propriedade observável testável sem unsafe.
    #[test]
    fn debug_da_seed_redigido() {
        let seed = seed_hex(SEED_A);
        let debug = format!("{seed:?}");
        assert!(debug.contains("redigida"), "{debug}");
        assert!(!debug.contains(&SEED_A[..16]), "{debug}");
        assert!(!debug.contains(&hex::encode(
            seed_hex(SEED_A).0 // se vazasse os bytes crus, apareceria aqui
        )));
    }

    /// Os bytes assinados são EXATAMENTE o JCS do payload — re-canonicalizar
    /// reproduz o payload_sha256 (contrato de bytes do dossiê RC 18).
    #[test]
    fn bytes_assinados_sao_o_jcs_do_payload() {
        let seed = seed_hex(SEED_A);
        let payload = payload_simples();
        let dossie = assinar_dossie(&payload, &seed).unwrap();
        let canon = crate::jcs::canonicalizar(&serde_json::to_value(&payload).unwrap()).unwrap();
        assert_eq!(dossie.payload_sha256, sha256_hex(canon.as_bytes()));
    }
}
