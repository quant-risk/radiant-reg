//! secrets — provedor de segredos (senhas Sisbacen) para os clientes STA/senhaws.
//!
//! Port do mecanismo de env vars do v1 (`backend/internal/secrets/env.go`):
//! a senha NUNCA é parâmetro de CLI (apareceria no `ps` e no histórico do
//! shell) — vem de `RADIANT_SECRET_BACEN_SENHA_<USUARIO>` com o usuário
//! normalizado EXATAMENTE como o `EnvManager.envName` do v1 (env.go:47-62):
//! uppercase ASCII, e todo caractere não-alfanumérico (exceto `_`) vira `_` —
//! `/` e `.` de `12345/0001.fulano` viram `_`.
//!
//! Convenção do v1 (backend/SPRINT_28_RESULTS.md:26):
//! `RADIANT_SECRET_BACEN_SENHA_123450001_FULANO`.
//!
//! O valor retornado é `Zeroizing<String>` — a cópia do segredo é zerada no
//! `Drop` (higiene de memória; o v1 guardava `String` crua na struct).
//!
//! `ProvedorEnv` lê do ambiente do processo. Para testes/injeção, o crate
//! oferece `ProvedorMemoria` (cfg(test)) — as mutações de ambiente
//! (`std::env::set_var`) são `unsafe` na edição 2024 e o workspace proíbe
//! `unsafe_code`, então os testes NUNCA tocam o ambiente real; a normalização
//! é função pura testável.

use std::fmt::Debug;

use reg_core::RegError;
use zeroize::Zeroizing;

/// Prefixo da env var de senha Sisbacen (v1: `bacen/senha/<usuario>` com
/// prefixo `RADIANT_SECRET_` — env.go:14-18).
pub const PREFIXO_ENV_SENHA_BACEN: &str = "RADIANT_SECRET_BACEN_SENHA_";

/// Fonte de segredos. Implementações devem NUNCA logar o valor retornado
/// (F13.8 do v1) e devem retornar o segredo em contêiner zeroizado.
pub trait ProvedorSegredos: Debug {
    /// Resolve a senha Sisbacen do usuário informado.
    fn obter_senha_bacen(&self, usuario: &str) -> Result<Zeroizing<String>, RegError>;
}

/// Normaliza um nome de segredo para nome de env var — port fiel de
/// `EnvManager.envName` (backend/internal/secrets/env.go:47-62):
/// `A-Z`, `0-9` e `_` preservados; `a-z` → maiúscula; todo o resto
/// (`/`, `.`, `-`, …) → `_`.
pub(crate) fn normalizar_para_env(nome: &str) -> String {
    let mut saida = String::with_capacity(PREFIXO_ENV_SENHA_BACEN.len() + nome.len());
    saida.push_str(PREFIXO_ENV_SENHA_BACEN);
    for c in nome.chars() {
        match c {
            'A'..='Z' | '0'..='9' | '_' => saida.push(c),
            'a'..='z' => saida.push(c.to_ascii_uppercase()),
            _ => saida.push('_'),
        }
    }
    saida
}

/// Implementação de produção: lê a senha do ambiente do processo.
#[derive(Debug, Default)]
pub struct ProvedorEnv;

impl ProvedorSegredos for ProvedorEnv {
    fn obter_senha_bacen(&self, usuario: &str) -> Result<Zeroizing<String>, RegError> {
        let nome_var = normalizar_para_env(usuario);
        match std::env::var(&nome_var) {
            Ok(valor) if !valor.is_empty() => Ok(Zeroizing::new(valor)),
            // Valor vazio = não encontrado, como o v1 (env.go:70-72).
            Ok(_) => Err(RegError::MissingField { field: nome_var }),
            Err(std::env::VarError::NotPresent) => Err(RegError::MissingField { field: nome_var }),
            Err(std::env::VarError::NotUnicode(_)) => Err(RegError::InvalidValue {
                field: nome_var,
                reason: "valor da env var não é UTF-8 válido".to_string(),
            }),
        }
    }
}

/// Implementação em memória para testes (injeção sem tocar o ambiente do
/// processo — `std::env::set_var` é `unsafe` na edição 2024 e o workspace
/// proíbe `unsafe_code`).
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct ProvedorMemoria {
    senhas: std::collections::BTreeMap<String, String>,
}

#[cfg(test)]
impl ProvedorMemoria {
    #[allow(dead_code)]
    pub(crate) fn inserir(mut self, usuario: &str, senha: &str) -> Self {
        self.senhas.insert(usuario.to_string(), senha.to_string());
        self
    }
}

#[cfg(test)]
impl ProvedorSegredos for ProvedorMemoria {
    fn obter_senha_bacen(&self, usuario: &str) -> Result<Zeroizing<String>, RegError> {
        match self.senhas.get(usuario) {
            Some(senha) if !senha.is_empty() => Ok(Zeroizing::new(senha.clone())),
            _ => Err(RegError::MissingField {
                field: format!("(memória) {usuario}"),
            }),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn normalizacao_idem_v1_env_name() {
        // Casos portados da convenção do v1 (manager_test.go:141 —
        // "bacen/senha/123450001.fulano" → "RADIANT_SECRET_BACEN_SENHA_123450001_FULANO";
        // aqui o prefixo BACEN_SENHA_ já embutido e só o usuário varia).
        assert_eq!(
            normalizar_para_env("123450001.fulano"),
            "RADIANT_SECRET_BACEN_SENHA_123450001_FULANO"
        );
        // `/` e `.` viram `_` (env.go:58 — default arm).
        assert_eq!(
            normalizar_para_env("12345/0001.fulano"),
            "RADIANT_SECRET_BACEN_SENHA_12345_0001_FULANO"
        );
        // lowercase → uppercase; `-` → `_` (env.go:53-58).
        assert_eq!(
            normalizar_para_env("123450001.Fulano-01"),
            "RADIANT_SECRET_BACEN_SENHA_123450001_FULANO_01"
        );
        // `_` é PRESERVADO, não dobrado (env.go:53 — `r == '_'` caixa própria).
        assert_eq!(
            normalizar_para_env("123450001.al_do"),
            "RADIANT_SECRET_BACEN_SENHA_123450001_AL_DO"
        );
    }

    #[test]
    fn provedor_memoria_retorna_valor_zeroizado() {
        let provedor = ProvedorMemoria::default().inserir("12345/0001.fulano", "senha-teste");
        let senha = provedor.obter_senha_bacen("12345/0001.fulano").unwrap();
        assert_eq!(senha.as_str(), "senha-teste");
        // Tipo do retorno é Zeroizing<String> — o Drop zera a cópia (estático).
        let senha: Zeroizing<String> = senha;
        drop(senha);
    }

    #[test]
    fn provedor_memoria_faltante_e_missing_field() {
        let provedor = ProvedorMemoria::default();
        match provedor.obter_senha_bacen("12345/0001.fulano") {
            Err(RegError::MissingField { field }) => {
                assert!(field.contains("12345/0001.fulano"));
            }
            _ => panic!("esperava MissingField"),
        }
    }

    #[test]
    fn provedor_env_faltante_e_missing_field_com_nome_da_var() {
        // Caminho de BUSCA (var ausente) — não mutamos o ambiente.
        // O nome é específico o suficiente para não existir no ambiente de CI.
        let provedor = ProvedorEnv;
        match provedor.obter_senha_bacen("99999/9999.usuario_inexistente_neste_ambiente") {
            Err(RegError::MissingField { field }) => {
                assert_eq!(
                    field,
                    "RADIANT_SECRET_BACEN_SENHA_99999_9999_USUARIO_INEXISTENTE_NESTE_AMBIENTE"
                );
            }
            _ => panic!("esperava MissingField (var ausente)"),
        }
    }
}
