//! reg-ledger — Camada 2 do Radiant Reg (wedge V1).
//!
//! Ledger SQLite WAL com schema **portável** para Postgres na Fase 2 (nunca
//! sintaxe exclusiva de SQLite em colunas/índices de domínio), hash-chain
//! tamper-evident (algoritmo portado de `backend/internal/auditlog/log.go`
//! com o bug da fórmula duplicada corrigido), canonicalização JCS RFC 8785
//! + assinatura Ed25519 (dossiê RC 18/2025), mapping_locks 4-olhos.
//!
//! Módulos a implementar em S5: `chain`, `jcs`, `sign`, `mapping_locks`.
