//! Erro comum do domínio. Tipado, sem `anyhow` — a mensagem de erro regulatório
//! é artefato auditável (provenance obrigatória, ADR-0010).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum RegError {
    /// Campo obrigatório do leiaute ausente — nunca default silencioso
    /// (fio desligado nº 5 do v1: UF="SP", email fake).
    #[error("campo obrigatório ausente: {field}")]
    MissingField {
        /// Caminho do campo, ex. "Doc3040/@cnpj".
        field: String,
    },
    /// Valor inválido para o domínio declarado no leiaute.
    #[error("valor inválido em {field}: {reason}")]
    InvalidValue { field: String, reason: String },
    /// Regra do catálogo sem implementação — estado DECLARADO, nunca skip
    /// silencioso (fio desligado nº 1 do v1).
    #[error("regra {rule} sem implementação declarada (estado: {state})")]
    RuleNotImplemented { rule: String, state: String },
}
