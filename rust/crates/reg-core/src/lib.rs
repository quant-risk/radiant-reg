//! reg-core — Camada 0 do Radiant Reg (wedge V1).
//!
//! Tipos de domínio regulatório. Nenhuma dependência além de serde/thiserror.
//! Módulos a implementar na S2 (spec: docs/blueprint/20-ARQUITETURA-RUST.md §4):
//! - [`money`]    Centavos(i128) — zero float, aritmética checked
//! - [`cnpj`]     CnpjIdentifier temporal (IN BCB 729/2026)
//! - [`ipoc`]     IPOC (15..=67 chars, sem OrigemRec)
//! - [`period`]   DataBase (AAAA-MM) e Vigencia `[from, to)`
//!
//! Até os módulos serem preenchidos na S2, o crate expõe `RegError` comum.

pub mod error;

pub use error::RegError;
