//! Hash-chain tamper-evident — port de `backend/internal/auditlog/log.go:62-234`
//! (v1 Go) **com o bug da fórmula duplicada corrigido por design**.
//!
//! ## O bug do v1 e a correção
//! O `Log` do v1 calculava o `entry_hash` DENTRO da transação
//! (log.go:118-121) e **recalculava** a mesma fórmula FORA, para retornar a
//! `Entry` (log.go:141-144). As duas cópias eram idênticas no dia em que o
//! código foi escrito — mas nada impedia que uma divergisse da outra: o que
//! é RETORNADO deixaria de ser o que é GRAVADO, e o `Verify` passaria a
//! falhar (ou pior, a falhar só para entradas com determinada forma de
//! metadado). Aqui a fórmula existe em **UM PONTO SÓ** —
//! [`compute_entry_hash`] — e o `append` devolve o MESMO hash que gravou,
//! computado uma única vez dentro da transação. O teste
//! `retornado_eh_exatamente_o_gravado` pina essa propriedade; os testes
//! `cadeia_encadeada_verifica` e os de tamper provam que o `Verify`
//! reconstrói o hash só com o que está gravado.
//!
//! ## Fórmula (ordem fixa, byte-a-byte = log.go:119)
//! ```text
//! entry_hash = sha256( prev_hash + payload_hash_hex + metadata_json
//!                      + actor + action + target + if_id + timestamp )
//! payload_hash = sha256(payload) em hex minúsculo
//! ```
//! - `prev_hash` do primeiro registro (gênese) = 64×'0'
//!   ([`GENESIS_PREV`], log.go:115).
//! - O `timestamp` é RFC3339Nano UTC **GRAVADO** e **HASHADO como o MESMO
//!   valor** ([`agora_rfc3339_nano`], log.go:84-87). Divergência honesta do
//!   v1: lá o `Verify` re-parseava e re-formatava o `created_at`
//!   (log.go:216); aqui o `Verify` usa a string GRAVADA verbatim —
//!   byte-identidade garantida por construção, sem depender de roundtrip de
//!   parse/format.
//! - `metadata` é serializada em JCS ([`crate::jcs`]) — determinística e
//!   zero-float; o v1 usava `json.Marshal` de Go (ordem de mapa não
//!   especificada na spec, estável na prática, não na letra).
//!
//! ## Concorrência
//! `append` roda em `BEGIN IMMEDIATE` ([`TransactionBehavior::Immediate`])
//! dentro de `Mutex<Connection>` — o SELECT do `prev_hash` e o INSERT são
//! atômicos, então duas conexões nunca carimbam o mesmo `prev_hash`
//! (a razão do BEGIN IMMEDIATE documentada em log.go:53-56 e :98-104).
//! `busy_timeout` de 30 s espelha a margem do v1 (log.go:62-73).
//!
//! ## Schema portável (Fase 1 SQLite WAL → Fase 2 Postgres)
//! Colunas de domínio usam apenas tipos SQL padrão (`TEXT`, `INTEGER`) —
//! nenhuma sintaxe exclusiva de SQLite (`AUTOINCREMENT`, `WITHOUT ROWID`,
//! `PRAGMA` em DDL). Mapeamento declarado para a Fase 2
//! (docs/blueprint/20-ARQUITETURA-RUST.md §8):
//! ```text
//! SQLite (Fase 1)                          → Postgres (Fase 2)
//! id INTEGER PRIMARY KEY                   → id BIGSERIAL PRIMARY KEY
//!   (alias de rowid no SQLite; sintaxe       (up-grade de tipo explícito na
//!    válida também no Postgres)               migração; no SQLite é o que dá
//!                                             auto-incremento sem AUTOINCREMENT)
//! if_id/target TEXT NULL                   → TEXT NULL (NULL = admin/sistema)
//! payload_hash/prev_hash/entry_hash TEXT   → TEXT — mantém-se TEXT: o valor é
//!                                             HASHADO COMO GRAVADO; CHAR(64)
//!                                             não é necessário e CHAR pad
//!                                             quebraria o hash
//! metadata TEXT (JCS)                      → TEXT — manter TEXT (JSONB
//!                                             reordena/tipifica na leitura e
//!                                             quebraria a recomputação)
//! created_at TEXT (RFC3339Nano UTC)        → TEXT — manter TEXT pelo mesmo
//!                                             motivo: TIMESTAMPTZ alteraria
//!                                             bytes na leitura e quebraria o
//!                                             hash
//! ```
//! `PRAGMA journal_mode=WAL` é configuração de RUNTIME da conexão (não DDL) —
//! exceção da fusão do wedge V1 (§3.1/§8); Postgres tem MVCC nativo e a
//! pragma simplesmente não existe lá.

use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::jcs;

/// Hash anterior da entrada gênese: 64×'0' (log.go:115).
pub const GENESIS_PREV: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Comprimento esperado dos hashes hex (SHA-256 = 32 bytes = 64 hex).
pub const TAM_HASH: usize = 64;

/// Margem de espera por lock de escrita — espelha log.go:62-73 (30 s =
/// 2× busy_timeout do v1, para absorver jitter de CI compartilhado).
pub const BUSY_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Error)]
pub enum ChainError {
    /// Erro SQLite (abertura, schema, transação, I/O).
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// Metadado não-canônico (ex.: `JcsFloatProibido` — zero-float vale
    /// também para o que entra no ledger).
    #[error("metadado do ledger: {0}")]
    Jcs(#[from] jcs::JcsError),
    /// Mutex interno envenenado — um `append` anterior entrou em panic
    /// segurando a conexão. O processo deve reiniciar o `Chain`.
    #[error("mutex da conexão envenenado (panic em chamada anterior)")]
    LockEnvenenado,
}

/// Uma entrada do audit log — espelha a `Entry` de log.go:29-40.
/// `if_id`/`target` vazios chegam aqui como `""` (o banco guarda NULL,
/// como o `nullable()` de log.go:236-241; a leitura normaliza de volta).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// `id` auto-incremento (rowid; BIGSERIAL na Fase 2).
    pub id: i64,
    /// Identificador da IF (tenant na Fase 2); vazio = admin/sistema.
    pub if_id: String,
    /// Quem executou a ação.
    pub actor: String,
    /// Ação auditada (ex.: `mapping.lock`).
    pub action: String,
    /// Alvo da ação (vazio quando não aplicável).
    pub target: String,
    /// sha256(payload) hex minúsculo.
    pub payload_hash: String,
    /// Hash da entrada anterior (gênese = [`GENESIS_PREV`]).
    pub prev_hash: String,
    /// Hash encadeado desta entrada — UMA fórmula ([`compute_entry_hash`]).
    pub entry_hash: String,
    /// Metadado em JSON canônico JCS (`""` quando ausente).
    pub metadata: String,
    /// Timestamp RFC3339Nano UTC — EXATAMENTE a string que foi hashada.
    pub created_at: String,
}

/// Requisição de append — análogo aos parâmetros de `Logger.Log`
/// (log.go:62) com `metadata` já como `Value` (o v1 aceitava `any`).
#[derive(Debug, Clone, Copy)]
pub struct AppendRequest<'a> {
    /// Identificador da IF; string vazia grava NULL (log.go:236-241).
    pub if_id: &'a str,
    /// Ator da ação.
    pub actor: &'a str,
    /// Ação auditada.
    pub action: &'a str,
    /// Alvo da ação; string vazia grava NULL.
    pub target: &'a str,
    /// Bytes do payload — só o SHA-256 é gravado.
    pub payload: &'a [u8],
    /// Metadado estruturado; `None` grava `""`. Será JCS-canonicalizado
    /// (float nele = erro [`ChainError::Jcs`]).
    pub metadata: Option<&'a serde_json::Value>,
}

/// Relatório do `verify` — análogo ao `(bool, int, error)` de log.go:180,
/// com o id da primeira quebra em vez de erro texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyReport {
    /// Entradas validadas: total quando `ok`; as validadas ANTES da quebra
    /// caso contrário (mesma semântica do `count` de log.go:231).
    pub entries: usize,
    /// `true` = cadeia íntegra de ponta a ponta.
    pub ok: bool,
    /// `id` da primeira entrada quebrada (chain ou entry hash), se houver.
    pub first_broken_id: Option<i64>,
}

/// Logger tamper-evident. Cada [`Chain`] tem sua conexão (SQLite WAL);
/// compartilhe por referência `&Chain` — o `append` é seguro entre threads.
pub struct Chain {
    conn: Mutex<Connection>,
}

/// **A** fórmula do entry hash — o ÚNICO ponto onde ela existe no crate
/// (correção do bug do v1: log.go:118-121 e :141-144 tinham duas cópias).
///
/// Ordem EXATA da concatenação (log.go:119):
/// `prev + payload_hash_hex + metadata_json + actor + action + target + if_id + timestamp`.
// allow justificado: a fórmula do ledger é posicional POR SPEC — os 8
// parâmetros espelham log.go:119 um a um; reestruturar em struct esconde
// a ordem que é o próprio contrato auditável.
#[allow(clippy::too_many_arguments)]
pub fn compute_entry_hash(
    prev_hash: &str,
    payload_hash_hex: &str,
    metadata_json: &str,
    actor: &str,
    action: &str,
    target: &str,
    if_id: &str,
    timestamp: &str,
) -> String {
    let concat = format!(
        "{prev_hash}{payload_hash_hex}{metadata_json}{actor}{action}{target}{if_id}{timestamp}"
    );
    sha256_hex(concat.as_bytes())
}

/// Timestamp atual em RFC3339Nano UTC — o MESMO valor é gravado e hashado
/// (log.go:84-87). Formato do Go `time.RFC3339Nano`: segundos fracionários
/// em até 9 dígitos com zeros à direita cortados (`…T00:00:01.1Z`), nenhum
/// dígito quando a fração é zero.
pub fn agora_rfc3339_nano() -> String {
    let nanos = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => i128::try_from(d.as_nanos()).unwrap_or(i128::MAX),
        Err(e) => -(i128::try_from(e.duration().as_nanos()).unwrap_or(i128::MAX)),
    };
    formatar_rfc3339_nano(nanos)
}

/// Formata nanos desde a época Unix (pode ser negativo) em RFC3339Nano UTC.
/// Algoritmo civil-from-days (Howard Hinnant), pinado em testes.
fn formatar_rfc3339_nano(nanos: i128) -> String {
    const NANOS_POR_SEG: i128 = 1_000_000_000;
    const SEG_POR_DIA: i128 = 86_400;
    let secs = nanos.div_euclid(NANOS_POR_SEG);
    let sub_nanos = nanos.rem_euclid(NANOS_POR_SEG) as u32; // [0, 1e9) cabe em u32
    let dias = secs.div_euclid(SEG_POR_DIA);
    let segundo_do_dia = secs.rem_euclid(SEG_POR_DIA);

    // dias cabe em i64 por construção: |dias| ≤ i128::MAX/(1e9×86400) ≈ 2e6.
    let (ano, mes, dia) = civil_from_days(i64::try_from(dias).unwrap_or(0));
    let hora = segundo_do_dia / 3_600;
    let minuto = (segundo_do_dia % 3_600) / 60;
    let segundo = segundo_do_dia % 60;

    let mut out = format!("{ano:04}-{mes:02}-{dia:02}T{hora:02}:{minuto:02}:{segundo:02}");
    if sub_nanos != 0 {
        let mut frac = format!("{sub_nanos:09}");
        while frac.ends_with('0') {
            frac.pop();
        }
        out.push('.');
        out.push_str(&frac);
    }
    out.push('Z');
    out
}

/// `civil_from_days` de Howard Hinnant: dias desde 1970-01-01 → (ano, mês, dia).
/// Divisões em Euclides para aceitar dias negativos (pré-1970).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097); // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let ano = yoe + era * 400 + i64::from(m <= 2);
    (ano, m, d)
}

/// NULL para string vazia — espelha `nullable()` de log.go:236-241.
/// O hash SEMPRE usa a string normalizada `""` (log.go:119 usa o parâmetro
/// cru, que é `""`; a leitura normaliza NULL → `""` em log.go:201-204).
fn nullable(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    hex::encode(digest)
}

const SCHEMA_AUDIT_LOG: &str = "\
-- Schema PORTÁVEL: nenhum tipo/sintaxe exclusiva de SQLite nas colunas de
-- domínio. Mapeamento Fase 2 (Postgres) documentado no doc do módulo.
CREATE TABLE IF NOT EXISTS audit_log (
    id           INTEGER PRIMARY KEY,
    if_id        TEXT,
    actor        TEXT NOT NULL,
    action       TEXT NOT NULL,
    target       TEXT,
    payload_hash TEXT NOT NULL,
    prev_hash    TEXT NOT NULL,
    entry_hash   TEXT NOT NULL,
    metadata     TEXT NOT NULL,
    created_at   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_log_if_id ON audit_log (if_id);
";

impl Chain {
    /// Abre (ou cria) o ledger em `path` e aplica o schema portável.
    /// Configura WAL + busy_timeout (runtime de conexão, não DDL — ver doc
    /// do módulo sobre a Fase 2).
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ChainError> {
        let conn = Connection::open(path)?;
        conn.query_row("PRAGMA journal_mode=WAL", [], |_| Ok(()))?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        conn.execute_batch(SCHEMA_AUDIT_LOG)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Registra uma nova entrada (port de `Logger.Log`, log.go:62-157).
    ///
    /// Passos: `payload_hash` → metadado JCS → timestamp único → BEGIN
    /// IMMEDIATE → prev_hash → **fórmula em um ponto só** → INSERT → commit.
    /// A [`Entry`] retornada carrega o MESMO `entry_hash` gravado — não há
    /// recálculo fora da transação (bug do v1 corrigido).
    pub fn append(&self, req: AppendRequest<'_>) -> Result<Entry, ChainError> {
        let payload_hash = sha256_hex(req.payload);
        let metadata = match req.metadata {
            Some(v) => jcs::canonicalizar(v)?,
            None => String::new(),
        };
        // Timestamp ÚNICO para gravar e hashar (log.go:84-87) — gerado antes
        // da transação para que nada dentro dela o re-derive.
        let created_at = agora_rfc3339_nano();

        let mut conn = self.conn.lock().map_err(|_| ChainError::LockEnvenenado)?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        let prev_hash: Option<String> = tx
            .query_row(
                "SELECT entry_hash FROM audit_log ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let prev_hash = prev_hash.unwrap_or_else(|| GENESIS_PREV.to_string());

        let entry_hash = compute_entry_hash(
            &prev_hash,
            &payload_hash,
            &metadata,
            req.actor,
            req.action,
            req.target,
            req.if_id,
            &created_at,
        );

        tx.execute(
            "INSERT INTO audit_log
                (if_id, actor, action, target, payload_hash, prev_hash, entry_hash, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                nullable(req.if_id),
                req.actor,
                req.action,
                nullable(req.target),
                payload_hash,
                prev_hash,
                entry_hash,
                metadata,
                created_at,
            ],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;

        Ok(Entry {
            id,
            if_id: req.if_id.to_string(),
            actor: req.actor.to_string(),
            action: req.action.to_string(),
            target: req.target.to_string(),
            payload_hash,
            prev_hash,
            // O MESMO valor computado e gravado acima — sem recálculo.
            entry_hash,
            metadata,
            created_at,
        })
    }

    /// Valida a cadeia inteira (port de `Logger.Verify`, log.go:180-234):
    /// (1) `prev_hash` de cada entrada == `entry_hash` da anterior;
    /// (2) `entry_hash` recomputável só a partir do que está GRAVADO
    /// (timestamp gravado verbatim — ver doc do módulo).
    /// Para na primeira quebra e reporta o `id` dela.
    pub fn verify(&self) -> Result<VerifyReport, ChainError> {
        let conn = self.conn.lock().map_err(|_| ChainError::LockEnvenenado)?;
        let mut stmt = conn.prepare(
            "SELECT id, if_id, actor, action, target, payload_hash, prev_hash, entry_hash, metadata, created_at
             FROM audit_log ORDER BY id ASC",
        )?;
        let mut rows = stmt.query([])?;

        let mut prev_hash = GENESIS_PREV.to_string();
        let mut validadas: usize = 0;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let if_id: Option<String> = row.get(1)?;
            let actor: String = row.get(2)?;
            let action: String = row.get(3)?;
            let target: Option<String> = row.get(4)?;
            let payload_hash: String = row.get(5)?;
            let prev_gravado: String = row.get(6)?;
            let entry_gravado: String = row.get(7)?;
            let metadata: String = row.get(8)?;
            let created_at: String = row.get(9)?;

            // 1. Chain check (log.go:209-211)
            if prev_gravado != prev_hash {
                return Ok(VerifyReport {
                    entries: validadas,
                    ok: false,
                    first_broken_id: Some(id),
                });
            }

            // 2. Entry hash recomputado sobre o gravado (log.go:214-228),
            //    com a MESMA fórmula única do append.
            let if_id_norm = if_id.unwrap_or_default();
            let target_norm = target.unwrap_or_default();
            let esperado = compute_entry_hash(
                &prev_gravado,
                &payload_hash,
                &metadata,
                &actor,
                &action,
                &target_norm,
                &if_id_norm,
                &created_at,
            );
            if entry_gravado != esperado {
                return Ok(VerifyReport {
                    entries: validadas,
                    ok: false,
                    first_broken_id: Some(id),
                });
            }

            prev_hash = entry_gravado;
            validadas += 1;
        }
        Ok(VerifyReport {
            entries: validadas,
            ok: true,
            first_broken_id: None,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;
    use std::thread;

    fn req(payload: &str) -> AppendRequest<'_> {
        AppendRequest {
            if_id: "IF-001",
            actor: "reg-ledger/testes",
            action: "teste",
            target: "alvo-1",
            payload: payload.as_bytes(),
            metadata: None,
        }
    }

    fn chain_tmp(dir: &tempfile::TempDir, nome: &str) -> Chain {
        Chain::open(dir.path().join(nome)).unwrap()
    }

    #[test]
    fn genesis_prev_tem_64_zeros() {
        assert_eq!(GENESIS_PREV.len(), TAM_HASH);
        assert!(GENESIS_PREV.chars().all(|c| c == '0'));
    }

    /// RFC3339Nano: frações cortam zeros à direita; zero fração = sem ponto.
    #[test]
    fn formato_rfc3339_nano_pina_valores_conhecidos() {
        let fmt = |nanos: i128| formatar_rfc3339_nano(nanos);
        assert_eq!(fmt(0), "1970-01-01T00:00:00Z");
        assert_eq!(fmt(1_000_000_001), "1970-01-01T00:00:01.000000001Z");
        assert_eq!(fmt(1_100_000_000), "1970-01-01T00:00:01.1Z"); // corta zeros
        assert_eq!(fmt(999_999_999), "1970-01-01T00:00:00.999999999Z");
        // Datas reais: 1_700_000_000 s = 2023-11-14T22:13:20Z (epoch externo).
        assert_eq!(
            fmt(1_700_000_000 * 1_000_000_000 + 123_456_789),
            "2023-11-14T22:13:20.123456789Z"
        );
        // 2000-02-29 (ano bissexto, dia 11016 da época) e pré-1970 (negativo).
        assert_eq!(fmt(951_782_400 * 1_000_000_000), "2000-02-29T00:00:00Z");
        assert_eq!(fmt(-86_400 * 1_000_000_000), "1969-12-31T00:00:00Z");
    }

    /// O timestamp de agora é parseável no formato pinado e termina em Z.
    #[test]
    fn agora_rfc3339_nano_forma_valida() {
        let agora = agora_rfc3339_nano();
        let data = &agora[..10];
        let tracos = (data.as_bytes()[4], data.as_bytes()[7]);
        assert_eq!(tracos, (b'-', b'-'));
        assert_eq!(&agora[10..11], "T");
        assert!(agora.ends_with('Z'), "{agora}");
        // Fração (se existir) entre 1 e 9 dígitos SEM zeros à direita.
        if let Some(idx) = agora.find('.') {
            let frac = &agora[idx + 1..agora.len() - 1];
            assert!((1..=9).contains(&frac.len()), "{agora}");
            assert!(!frac.ends_with('0'), "{agora}");
        }
    }

    /// BUG DO V1 CORRIGIDO: o hash retornado é EXATAMENTE o gravado —
    /// fórmula existe num ponto só; nada recalcula fora da transação
    /// (v1: log.go:118-121 vs :141-144).
    #[test]
    fn retornado_eh_exatamente_o_gravado() {
        let dir = tempfile::tempdir().unwrap();
        let chain = chain_tmp(&dir, "a.sqlite3");
        // Chave com espaço à esquerda exercita a ordenação JCS (0x20 ordena
        // antes de letras) sobre o metadado que entra no hash.
        let meta = serde_json::json!({" canal": 1, "origem": "teste", "z": [2, 1] });
        let entry = chain
            .append(AppendRequest {
                if_id: "IF-009",
                actor: "ator",
                action: "acao",
                target: "",
                payload: b"bytes do payload",
                metadata: Some(&meta),
            })
            .unwrap();

        // Lê a linha crua direto do banco (acesso interno do teste).
        let gravado: (String, String, String, String, String, String) = chain
            .conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT payload_hash, prev_hash, entry_hash, metadata, created_at, if_id
                 FROM audit_log WHERE id = ?1",
                params![entry.id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                    ))
                },
            )
            .unwrap();

        assert_eq!(gravado.2, entry.entry_hash, "gravado != retornado");
        assert_eq!(gravado.0, entry.payload_hash);
        assert_eq!(gravado.3, entry.metadata);
        assert_eq!(gravado.4, entry.created_at);
        // if_id vazio é gravado como NULL (port de nullable()); a Entry
        // retornada carrega a string crua — espelho do Go.
        assert_eq!(gravado.5, "IF-009");

        // E a fórmula única reproduz o hash só com os valores gravados:
        let recomputado = compute_entry_hash(
            &entry.prev_hash,
            &entry.payload_hash,
            &entry.metadata,
            &entry.actor,
            &entry.action,
            &entry.target,
            &entry.if_id,
            &entry.created_at,
        );
        assert_eq!(recomputado, entry.entry_hash);
    }

    /// Payload hashado em hex minúsculo (sha256("abc") conhecido).
    #[test]
    fn payload_hash_sha256_conhecido() {
        let chain = chain_tmp(&tempfile::tempdir().unwrap(), "p.sqlite3");
        let entry = chain.append(req("abc")).unwrap();
        assert_eq!(
            entry.payload_hash,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// Gênese: primeira entrada referencia 64×'0' (log.go:115).
    #[test]
    fn primeira_entrada_referencia_genesis() {
        let chain = chain_tmp(&tempfile::tempdir().unwrap(), "g.sqlite3");
        let entry = chain.append(req("primeiro")).unwrap();
        assert_eq!(entry.prev_hash, GENESIS_PREV);
        let rep = chain.verify().unwrap();
        assert!(rep.ok);
        assert_eq!(rep.entries, 1);
        assert_eq!(rep.first_broken_id, None);
    }

    /// Chain cresce encadeada; verify ok com múltiplas entradas.
    #[test]
    fn cadeia_encadeada_verifica() {
        let chain = chain_tmp(&tempfile::tempdir().unwrap(), "c.sqlite3");
        let mut anterior = None;
        for i in 0..5 {
            let e = chain.append(req(&format!("payload-{i}"))).unwrap();
            if let Some(prev) = anterior {
                assert_eq!(e.prev_hash, prev, "entrada {i} não referenciou a anterior");
            }
            anterior = Some(e.entry_hash);
        }
        let rep = chain.verify().unwrap();
        assert!(rep.ok);
        assert_eq!(rep.entries, 5);
    }

    /// Tamper detection (1): alteração de campo hashado → quebra no id certo.
    #[test]
    fn tamper_em_actor_eh_detectado() {
        let chain = chain_tmp(&tempfile::tempdir().unwrap(), "t1.sqlite3");
        let primeira = chain.append(req("a")).unwrap().id;
        chain.append(req("b")).unwrap();
        chain
            .conn
            .lock()
            .unwrap()
            .execute(
                "UPDATE audit_log SET actor = 'invasor' WHERE id = ?1",
                params![primeira],
            )
            .unwrap();
        let rep = chain.verify().unwrap();
        assert!(!rep.ok);
        assert_eq!(rep.first_broken_id, Some(primeira));
        assert_eq!(rep.entries, 0, "nenhuma entrada validada antes da quebra");
    }

    /// Tamper detection (2): payload_hash alterado → entry hash quebra.
    #[test]
    fn tamper_em_payload_hash_eh_detectado() {
        let chain = chain_tmp(&tempfile::tempdir().unwrap(), "t2.sqlite3");
        let primeira = chain.append(req("a")).unwrap().id;
        chain
            .conn
            .lock()
            .unwrap()
            .execute(
                "UPDATE audit_log SET payload_hash = replace(payload_hash, 'b', 'a') WHERE id = ?1",
                params![primeira],
            )
            .unwrap();
        let rep = chain.verify().unwrap();
        assert!(!rep.ok);
        assert_eq!(rep.first_broken_id, Some(primeira));
    }

    /// Tamper detection (3): timestamp alterado quebra a recomputação —
    /// o created_at gravado é parte do hash.
    #[test]
    fn tamper_em_created_at_eh_detectado() {
        let chain = chain_tmp(&tempfile::tempdir().unwrap(), "t3.sqlite3");
        let primeira = chain.append(req("a")).unwrap().id;
        chain
            .conn
            .lock()
            .unwrap()
            .execute(
                "UPDATE audit_log SET created_at = '1970-01-01T00:00:00Z' WHERE id = ?1",
                params![primeira],
            )
            .unwrap();
        let rep = chain.verify().unwrap();
        assert!(!rep.ok);
        assert_eq!(rep.first_broken_id, Some(primeira));
    }

    /// Tamper detection (4): DELETE no meio → quebra de ENCADEAMENTO na
    /// entrada seguinte (prev_hash dela ≠ hash da anterior), no id dela —
    /// igual ao comportamento do Go (log.go:209-211).
    #[test]
    fn delete_no_meio_quebra_encadeamento_no_id_seguinte() {
        let chain = chain_tmp(&tempfile::tempdir().unwrap(), "t4.sqlite3");
        let ids: Vec<i64> = (0..4)
            .map(|i| chain.append(req(&format!("p{i}"))).unwrap().id)
            .collect();
        chain
            .conn
            .lock()
            .unwrap()
            .execute("DELETE FROM audit_log WHERE id = ?1", params![ids[1]])
            .unwrap();
        let rep = chain.verify().unwrap();
        assert!(!rep.ok);
        assert_eq!(
            rep.first_broken_id,
            Some(ids[2]),
            "a quebra é na entrada seguinte ao buraco"
        );
        assert_eq!(rep.entries, 1, "só a gênese foi validada antes da quebra");
    }

    /// Verify de banco vazio: íntegro com 0 entradas.
    #[test]
    fn verify_banco_vazio_eh_ok() {
        let chain = chain_tmp(&tempfile::tempdir().unwrap(), "v.sqlite3");
        let rep = chain.verify().unwrap();
        assert!(rep.ok);
        assert_eq!(rep.entries, 0);
        assert_eq!(rep.first_broken_id, None);
    }

    /// Append concorrente com 2 conexões no MESMO arquivo (o cenário do v1
    /// que motivou BEGIN IMMEDIATE — log.go:53-56): cadeia única, sem
    /// prev_hash duplicado, 50 entradas íntegras.
    #[test]
    fn append_concorrente_2_conexoes_nao_quebra_cadeia() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("concorrente.sqlite3");
        let chain_a = Chain::open(&path).unwrap();
        let chain_b = Chain::open(&path).unwrap();

        let h_a = thread::spawn(move || {
            for i in 0..25 {
                chain_a
                    .append(AppendRequest {
                        if_id: "IF-A",
                        actor: "worker-a",
                        action: "concorrente",
                        target: "",
                        payload: format!("a-{i}").as_bytes(),
                        metadata: None,
                    })
                    .unwrap();
            }
        });
        let h_b = thread::spawn(move || {
            for i in 0..25 {
                chain_b
                    .append(AppendRequest {
                        if_id: "IF-B",
                        actor: "worker-b",
                        action: "concorrente",
                        target: "",
                        payload: format!("b-{i}").as_bytes(),
                        metadata: Some(&serde_json::json!({"i": i})),
                    })
                    .unwrap();
            }
        });
        h_a.join().unwrap();
        h_b.join().unwrap();

        // Terceira conexão vê o estado consolidado.
        let leitor = Chain::open(&path).unwrap();
        let rep = leitor.verify().unwrap();
        assert!(rep.ok, "cadeia quebrou sob concorrência: {rep:?}");
        assert_eq!(rep.entries, 50);
        assert_eq!(rep.first_broken_id, None);

        // ids únicos 1..=50 — prova que não houve corrida de INSERT.
        let conn = leitor.conn.lock().unwrap();
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 50);
        let distintos: i64 = conn
            .query_row("SELECT COUNT(DISTINCT id) FROM audit_log", [], |r| r.get(0))
            .unwrap();
        assert_eq!(distintos, 50);
    }

    /// if_id/target vazios: NULL no banco, `""` no hash — paridade com o Go
    /// (nullable() em log.go:236-241; concat em :119; leitura em :201-204).
    #[test]
    fn campos_vazios_vao_como_null_e_hasham_como_string_vazia() {
        let chain = chain_tmp(&tempfile::tempdir().unwrap(), "n.sqlite3");
        let entry = chain
            .append(AppendRequest {
                if_id: "",
                actor: "sistema",
                action: "bootstrap",
                target: "",
                payload: b"x",
                metadata: None,
            })
            .unwrap();
        let (if_id_db, target_db): (Option<String>, Option<String>) = chain
            .conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT if_id, target FROM audit_log WHERE id = ?1",
                params![entry.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(if_id_db, None);
        assert_eq!(target_db, None);
        // verify recomputa com "" e casa.
        assert!(chain.verify().unwrap().ok);
    }

    /// Metadado com f64 é recusado no append (doutrina zero-float atravessa
    /// o ledger inteiro).
    #[test]
    fn metadata_float_proibido_no_append() {
        let chain = chain_tmp(&tempfile::tempdir().unwrap(), "f.sqlite3");
        let meta = serde_json::json!({"total": 1.5});
        let err = chain
            .append(AppendRequest {
                if_id: "IF-001",
                actor: "a",
                action: "acao",
                target: "",
                payload: b"x",
                metadata: Some(&meta),
            })
            .unwrap_err();
        assert!(matches!(
            err,
            ChainError::Jcs(jcs::JcsError::JcsFloatProibido { .. })
        ));
        assert!(err.to_string().contains("JcsFloatProibido"), "{err}");
        // Nada foi gravado (erro antes do INSERT).
        assert!(chain.verify().unwrap().ok);
        assert_eq!(chain.verify().unwrap().entries, 0);
    }
}
