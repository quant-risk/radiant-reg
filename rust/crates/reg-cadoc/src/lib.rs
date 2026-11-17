//! reg-cadoc — Camada 1 do Radiant Reg (wedge V1).
//!
//! Ingestão estrita (calamine/CSV), generators 3040 (CMN 4.966) e 2030 (IN 694/2025),
//! BacenValidatorAdapter (ADR-0018 — o .jar oficial é o oráculo de submissão, B02).
//! Depende APENAS de reg-core (teste `tests/layers.rs` no reg-cli prova a direção).
//!
//! Módulos a implementar em S2-S4: `ingest`, `gen3040`, `gen2030`, `validator`,
//! `dedup` (blake3 P=256).

/// Versão do leiaute alvo, marcada explicitamente — data-base é cidadão de 1ª classe.
pub const TARGET_LEIAUTE_3040: &str = "doc3040 (IN BCB 729/733/735 de 2026 — re-download S1 pendente)";
pub const TARGET_LEIAUTE_2030: &str = "documentoDRSAC (IN BCB 694/2025, data-base dez/2026)";
