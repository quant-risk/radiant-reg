//! reg-ledger — Camada 2 do Radiant Reg (wedge V1).
//!
//! Ledger SQLite WAL com schema **portável** para Postgres na Fase 2
//! (nunca sintaxe exclusiva de SQLite em colunas/índices de domínio),
//! hash-chain tamper-evident (port de `backend/internal/auditlog/log.go`
//! com o bug da fórmula duplicada corrigido por design), canonicalização
//! JCS RFC 8785 (subset zero-float) + assinatura Ed25519 (dossiê RC
//! Conjunta 18/2025), mapping_locks 4-olhos como máquina de estados tipada.
//!
//! ## Origens (spec executável)
//! - **Hash-chain**: `backend/internal/auditlog/log.go:62-157` (Log com
//!   BEGIN IMMEDIATE e timestamp gravado == hashado) e `:180-234` (Verify
//!   com detecção de tamper). O **bug do v1** — fórmula do entry hash
//!   DUPLICADA dentro (log.go:118-121) e fora (log.go:141-144) da
//!   transação — foi corrigido por design: existe UMA função
//!   ([`chain::compute_entry_hash`]) e a [`chain::Entry`] retornada carrega
//!   o MESMO hash gravado (pinnado em teste).
//! - **Blueprint**: `docs/blueprint/20-ARQUITETURA-RUST.md` §3.1 (wedge V1:
//!   4 crates, SQLite WAL portável), §4.5 (doutrina zero-float), §8
//!   (schema portável e migração Fase 2), §17 ADR-0016 (hash-chain
//!   segmentada com ancoragem externa — Fase 2).
//! - **mapping.lock 4-olhos**: `docs/blueprint/25-ROADMAP-EXECUCAO.md` §0,
//!   S2-S3/S5 ("mapping.lock com 4-olhos", "mapping.lock assinado").
//! - **Dossiê**: RC Conjunta 18/2025 — JSON assinado Ed25519 ([`sign`]).
//!
//! ## Módulos
//! - [`chain`] — hash-chain + persistência SQLite portável, append
//!   concorrente seguro, `verify()` com tamper detection.
//! - [`jcs`] — canonicalização JCS RFC 8785 (subset honesto): chaves por
//!   UTF-16, escapes ECMAScript, inteiros sem ponto, f64 = erro tipado
//!   `JcsFloatProibido`.
//! - [`sign`] — Ed25519: dossiê JCS→bytes→assinatura→hex; `DualApproval`
//!   exige 2 chaves distintas; seed privada com zeroize (ADR-0015).
//! - [`mapping_locks`] — `Rascunho → UnidadesContratadas → DryRunLimpo →
//!   AmostraAssinada → MappingFechado`; cada transição exige seu
//!   evidencial, negação tipada nomeando a trava.
//!
//! ## Exemplo — JCS determinístico
//! ```
//! use reg_ledger::jcs;
//! let v: serde_json::Value = serde_json::from_str(r#"{"b":2,"a":1}"#).unwrap();
//! assert_eq!(jcs::canonicalizar(&v).unwrap(), r#"{"a":1,"b":2}"#);
//! ```
//!
//! Direção das dependências (gate `tests/layers.rs` do reg-cli):
//! reg-core ← reg-ledger ← reg-cli; reg-ledger não conhece reg-cadoc.

/// Hash-chain tamper-evident portada do v1 Go (bug da fórmula duplicada
/// corrigido) + persistência SQLite portável.
pub mod chain;

/// Canonicalização JSON JCS RFC 8785 — subset zero-float honesto.
pub mod jcs;

/// Assinatura Ed25519 — dossiê RC 18/2025 e dual approval 4-olhos.
pub mod sign;

/// mapping.lock — as 4 travas 4-olhos como máquina de estados tipada.
pub mod mapping_locks;

// Re-exports de conveniência — os caminhos completos continuam disponíveis.
pub use chain::{
    AppendRequest, BUSY_TIMEOUT, Chain, ChainError, Entry, GENESIS_PREV, TAM_HASH, VerifyReport,
    agora_rfc3339_nano, compute_entry_hash,
};
pub use jcs::{JcsError, canonicalizar, canonicalizar_para_bytes, numero_inteiro};
pub use mapping_locks::{
    ConservationCheck, ContratoUnidades, LockError, MappingFechado, MappingLock, Trava,
    TravaAplicada,
};
pub use sign::{
    ArtefatoRef, DossierAssinado, DossierPayload, DualApproval, SeedAssinatura, SignError,
    assinar_dossie, verificar_dossie,
};
