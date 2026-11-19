//! reg-cadoc — Camada 1 do Radiant Reg (wedge V1).
//!
//! Ingestão estrita (calamine/CSV), generator 3040 (CMN 4.966) — o gen2030
//! (IN 694/2025) é BLOQUEIO DECLARADO (re-download do leiaute 2030 pendente,
//! docs/normativos/MANIFESTO.md) — e BacenValidatorAdapter (ADR-0018 — o
//! .jar oficial é o oráculo de submissão, B02).
//! Depende APENAS de reg-core (teste `tests/layers.rs` no reg-cli prova a direção).
//!
//! Módulos a implementar: `gen2030` (bloqueado no leiaute), `dedup`
//! (blake3 P=256 — sem dependência no workspace; pendência declarada).

/// Versão do leiaute alvo, marcada explicitamente — data-base é cidadão de 1ª classe.
pub const TARGET_LEIAUTE_3040: &str =
    "doc3040 (IN BCB 729/733/735 de 2026 — re-download S1 pendente)";
pub const TARGET_LEIAUTE_2030: &str = "documentoDRSAC (IN BCB 694/2025, data-base dez/2026)";

/// Ingestão estrita: calamine → `CanonicalInput` (docs/blueprint/20-ARQUITETURA-RUST.md §3.2).
pub mod ingest;

/// Golden corpus: loader + runner de paridade (§6 — skip declarado, nunca silêncio).
pub mod golden;

/// Generator 3040 no formato CMN 4.966 VIGENTE (Cli/Op individualizado, §6).
pub mod gen3040;

/// BacenValidatorAdapter — oráculo oficial de submissão B02 (ADR-0018, §17).
pub mod validator;

/// Extrator determinístico zero-float dos catálogos oficiais SCR3040 (.xls → JSON).
pub mod catalog_xls;
