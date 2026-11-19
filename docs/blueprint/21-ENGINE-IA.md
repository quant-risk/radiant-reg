# 21 — ENGINE DE IA v3.0 (Arquitetura Dual API ZDR + Open-Weights 2026 & Conexão com `System 1`)

> **Versão:** 3.0 · **Data-base:** 19/11/2026 · **Status:** Arquitetura consolidada (Plano v3.0 — sem modificação de código até aprovação das etapas)
> **Documentos irmãos:** [`26-SYSTEM-ONE-DECISION-MODEL.md`](26-SYSTEM-ONE-DECISION-MODEL.md) (`System 1` completo) · [`20-ARQUITETURA-RUST.md`](20-ARQUITETURA-RUST.md) (`System 0` & zero-float) · [`22-HARNESS-V4.md`](22-HARNESS-V4.md) (`M1–M10`) · [`25-ROADMAP-EXECUCAO.md`](25-ROADMAP-EXECUCAO.md) (Plano v3.0 Etapas 0–4)
> **Fontes:** `research/07-engine-ia.md`, `research/04-rust-stack.md`, `research/09-harness.md`, verificação de código do workspace (`rust/` e `backend/internal/{insights,synth}/`), auditoria de licenças/hardware nov/2026 e revisão adversarial (C1–C6).

---

## 0. Veredito Executivo & Diagnóstico do Estado Real (19/11/2026)

### Estado Atual Verificado no Repositório
| Camada Inspecionada | Estado Real Verificado | Diagnóstico Técnico |
|---|---|---|
| **Workspace Rust (`rust/`)** (`reg-core`, `reg-cadoc`, `reg-ledger`, `reg-cli`) | **0% IA implementada.** Protegido por `unsafe_code = "forbid"`, `float_arithmetic = "deny"`, `float_cmp = "deny"`, `cast_precision_loss = "deny"` e `wildcard_enum_match_arm = "deny"` ([`Cargo.toml`](../../rust/Cargo.toml), vigiado por [`layers.rs`](../../rust/crates/reg-cli/tests/layers.rs)). `reg-cli` impõe `--offline=true` default ([`main.rs`](../../rust/crates/reg-cli/src/main.rs)). | Núcleo jurídico-financeiro sólido. A camada de IA deve acoplar **na borda**, nunca dentro de `reg-core/cadoc/ledger`. |
| **Legado Go (`backend/internal/insights/` & `synth/`)** | `llm.go` (`MiniMaxChat`/`OpenAIChat`) usa `sanitizeForPrompt` que só trunca 200 chars e remove `\n\r\t`; `conversation.go` (`ResponseCache.Invalidate`) apaga cache de **todos os tenants**; `synth/weak.go` é stub (`1.0` se `XML != ""`); `judge.go` aceita caso por default em erro JSON. | **Passivo ativo de compliance LC 105 / multi-tenancy.** Aposentar/desativar imediatamente na **Etapa 0** do Plano v3.0. |
| **Docs Jul/2026 (`ON_PREMISE_LLM_SPEC.md`, `LLM_INTEGRATION_GUIDE.md`)** | Descrevem Go 1.25 + Python MCP + ChromaDB + Ollama + `Qwen2.5`/`Mistral-7B`/`Phi-3-mini` + LLM lendo XLSX. | **Depreciados e arquivados** (marcados no topo com redirecionamento para este doc e `26`). |

### Veredito de Stack IA v3.0 (Resumo em Uma Página)

| Camada / Função | Pick Primário (Nov/2026) | Alternativa / Fallback | Governança / Custo |
|---|---|---|---|
| **System 1 (Decision Models)** | `reg-s1-tabular` (GBDT `i64` + `TreeSHAP`) + `ort` INT8 (`moBERTo`, `DeB3RTa`, `debertinha`) | Heurísticas léxicas + parsers `reg-core` (`--offline` nativo) | `$0`/token · zero-float na fronteira · [`26`](26-SYSTEM-ONE-DECISION-MODEL.md) |
| **Extração Norma→Regra** | **Local:** `gpt-oss-120b` / `Qwen3.6-35B-A3B` · **Cloud ZDR:** `Gemini 3.x Pro` (visão) | `Claude Opus 4.5+` (Cloud) · `Qwen3.5-397B-A17B` (Cluster) | Two-Pass `Draft→Critic→Compile→Human` |
| **Crítica & Copiloto Interativo** | **Local:** `Mistral Small 3.2 24B` / `Gemma 4 26B-A4B` / `gpt-oss-20b` · **Cloud ZDR:** `Claude Sonnet 4.5+` | `Qwen3.5-9B` (Mac/Edge) · `GPT-5.1+` | Single-pass sem *thinking* no chat; **3 campos `Provenance`** |
| **Batch / Challenger Sintético (`M4`)** | **Local:** `Qwen3.5-4B` / `Gemma 4 E4B` · **Cloud:** `Gemini 3.x Flash` (Batch 50% off) | `GPT-5 mini` / `Claude Haiku 4.5+` | Somente perfis sintéticos/públicos (zero PII) |
| **Embeddings & Reranking PT-BR** | **`BGE-M3`** (MIT, Dense+Sparse) + **`BGE-Reranker-v2-M3`** (MIT) | **`Qwen3-Embedding` / `Qwen3-Reranker`** (`0.6B/4B/8B`, Apache 2.0) | `$0` self-hosted (`ort` CPU RAM ou GPU) |
| **RAG Store Dual-Mode** | **Single-Node:** `Tantivy` + `sqlite-vec`/HNSW · **Plataforma:** `pgvector` HNSW + `Tantivy` + RRF `i64` | Qdrant apenas se filtragem pesada | Corpus **público** separado de dados de tenant |
| **Parsing de Fontes BCB** | **`calamine`** (XLS/XLSX determinístico, Rust) + **`Docling`** (PDF nativo) | OCR (`Mistral OCR`) apenas em PDF escaneado | **Nunca OCR para XLSX** |
| **Serving Local** | **`mistral.rs`** (in-process Rust/CLI/Mac/Workstation) · **`SGLang`** (Copiloto `RadixAttn`) | **`vLLM`** (`XGrammar` batch) · **`llama.cpp`** (`--jinja` + KV `q8_0`) | Versão engine + SHA-256 do template Jinja pinados em CI |
| **MCP & Agentes** | **`rmcp`** (SDK oficial Rust, spec 2026-07-28) | — | **Read-only default**, zero sampling, HITL p/ remessa STA |
| **Política de Dados** | `TenantAiConfig` (`AirGapped` · `HybridPolicyRouter` · `CloudZdrFirst`) | Fail-closed para `RouteOnPremOnly` | **Zero texto livre com cliente na nuvem estrangeira (LC 105)** |

**Princípio arquitetural em uma frase:** **System 0** determina a verdade legal e contábil em Rust (`forbid(unsafe)`, zero-float); **System 1** classifica, protege, sana e roteia localmente em milissegundos; **System 2** raciocina e explica na borda (Local Open-Weights ou Cloud ZDR), sempre sob verificação objetiva (`jsonschema` + `reg-core` + compilação/XSD + humano).

---

## 1. Arquitetura em 3 Camadas & Papel Exato por Superfície do Produto

A arquitetura cognitiva unificada divide o produto em **System 0 (Motor Determinístico) → System 1 (Decision Models) → System 2 (LLMs Local/Cloud)** (detalhes de tipos e algoritmos `S1.1..S1.4` em [`26-SYSTEM-ONE-DECISION-MODEL.md`](26-SYSTEM-ONE-DECISION-MODEL.md)).

### Fronteira de Crates no Workspace Rust (Sem Invadir `layers.rs`)
1. **`reg-core`, `reg-cadoc`, `reg-validate-core` (`no_fs`, `wasm32`), `reg-ledger`, `reg-sta`:** **zero dependências de IA/ML**, zero floats, 100% determinísticos.
2. **`reg-s1-tabular`:** árvores GBDT (`LightGBM`/`CatBoost`), calibradores isotônicos, priors cold-start e `TreeSHAP` compilados para **ponto fixo `i64` puro** — zero float, compatível com `--no-default-features` e compilável para `wasm32-unknown-unknown` no navegador.
3. **`reg-ai` (camada de borda):** implementa `TenantAiConfig`, `DataPolicy`, `LlmTransport`, `EmbeddingPort`, `RerankPort`, `PiiAllowlistMasker`, `Tantivy`/`pgvector` RRF inteiro e `rmcp` — importa apenas `reg-core` e `reg-s1-tabular`.
4. **`reg-ai-runtime` (folha isolada atrás de `--features ai-onnx` / `--features ai-local-llm` ou sidecar UDS):** único ponto com `#[allow(clippy::float_arithmetic)]` para tensores `f32` de `ort` (ONNX Runtime INT8) e opcional `mistral.rs`; exporta para fora apenas `ScoreBps(u16)`, `i32` e `Vec<i16>` (`Q15`).

### Papel Permitido vs. Proibições Absolutas por Superfície

| Superfície do Produto | O Que a IA (`S1` + `S2`) Faz | O Que É Expressamente Proibido (Exclusivo `System 0`) |
|---|---|---|
| **1. Ingestão & Geração (`calamine`, `ingest/mapping.rs`, `gen3040`)** | `S1.1` pré-preenche `mapping.toml.draft` (`Proposal<MappingDraft>`) com `ScoreBps` calibrado e sugestão de `MonetaryUnit`/`Transform`. Faixa cinzenta escala ao `S2` enviando **somente** headers higienizados + estatísticas sintéticas de formato. | **Nunca** contorna as 4 travas de [`mapping_locks.rs`](../../rust/crates/reg-ledger/src/mapping_locks.rs) (`UnidadePorCampo`, `ConservacaoZero`, `QuatroOlhos`, `CustodiaSha256`), **nunca** gera XML bruto e **nunca** imputa no chute `@EstInstFin` (Anexo 41), `@ClassOp`, `@Mod` ou `@NatuOp`. |
| **2. Validação & Copiloto (`reg-validate-core`, WASM, SSE)** | `S1.4` resolve dúvidas simples via lookup `RuleSpec` (`Tier 0`, `<5 ms`, `$0`). Dúvidas contextuais usam RAG híbrido + `S2` gerando explicações e cards `FixSuggestion` (1-clique humano) com **3 campos de `Provenance`** + Output Guardrail. | **Nunca** calcula saldo, PDD ou DV; **nunca** altera/suprime veredito `E/A/I` de L1–L4; do **WASM no browser** só envia `{cadoc, catalog_version, rule_code, field_path, error_class}` — zero XML ou dados de tomador sobem para IA cloud. |
| **3. Pré-Envio & STA (`BacenValidatorAdapter`, `reg-sta`, `senhaws`)** | `S1.3` audita estatisticamente os agregados já gerados para detectar erros silenciosos que passam no `.jar` (escala `1000×` Reais vs MilReais, queda abrupta de PDD, salto anômalo de atraso), emitindo `SanityReviewAdvisory` ou `SanityHoldGate` com justificativa gravada no `reg-ledger`. | **Zero IA** no `.jar` BCB (ADR-0018), no SHA-256 de `*_VALIDADO.zip`, no transporte HTTP/Basic do STA ou no `senhaws`. Submissão ao STA é sempre confirmação humana explícita (HITL). |
| **4. Radar Normativo (`norma-radar` & Norma→Regra)** | Crawler allowlisted (`*.bcb.gov.br`, `*.in.gov.br`, `*.cvm.gov.br`) com SHA-256 + magic bytes + `calamine` diff (XLS) + `Docling` (PDF). `S1` filtra boilerplate; `S2` (Two-Pass `Draft→Critic`) propõe `RuleSpec` + fixtures testados no motor Rust antes do PR humano. | **Nunca** lê planilhas XLS com OCR/LLM; **nunca** promove regra em produção sem compilação + golden fixtures + aprovação humana; trata todo conteúdo externo sob *spotlighting*. |
| **5. Governança RC 18/2025 & Harness (`M1–M10`)** | `S1.2` higieniza retornos STA/CRD (`Canary FNR == 0%`) antes de `M1/M2/M7`; `M4` Challenger gera perfis sintéticos executados pelo `Solver` Rust + `Judge`; `M7` agrupa falhas via embeddings (`>=8 obs`, `>=2 tenants`); `M8` recalibra apenas submodelos `S1` sob gates `M6`. `S1AuditRecord` zero-float entra no `reg-ledger`. | **Nunca** usa fine-tuning para armazenar leis/regras (regras são dados versionados com `Vigencia`), **nunca** publica score placebo no `dossie.html`, e **nunca** compartilha caches ou índices vetoriais com dados entre tenants. |

---

## 2. Pipeline Extração Norma→Regra (Assíncrono / Two-Pass)

Converte o gargalo manual (~8h por resolução grande) em fluxo auditável assistido (~20 min de confirmação humana sobre diff compilado e testado):

```text
 Normativo (DOU / IN / CC — PDF)                 Leiaute Oficial (XLS/XLSX BCB)
         │                                                      │
         ▼                                                      ▼
   Docling (layout + tabelas, §)                       calamine (Rust determinístico — NUNCA OCR)
         └──────────────────────────┬───────────────────────────┘
                                    ▼
 ┌─ PASS 1 (REASONING / DRAFT) ────────────────────────────────────────────────────┐
 │ Local (`gpt-oss-120b` / `Qwen3.6-35B-A3B`) ou Cloud ZDR (`Gemini 3.x Pro`)       │
 │ Analisa § + JSON `calamine` juntos no canal de raciocínio isolado (n=3, temp 0.2)│
 └──────────────────────────────────┬──────────────────────────────────────────────┘
                                    ▼
 ┌─ PASS 2 (CRITIC & SERIALIZAÇÃO PORTÁTIL) ───────────────────────────────────────┐
 │ Critica §-a-§ (Local ou `Claude Sonnet 4.5+`) → serializa c/ `ThinkingPolicy::   │
 │ Disabled` + `json_schema` portátil (decimais/datas/CNPJs sempre como `string`)   │
 └──────────────────────────────────┬──────────────────────────────────────────────┘
                                    ▼
 ┌─ TEST (VERIFICAÇÃO OBJETIVA EM `System 0`) ─────────────────────────────────────┐
 │ Validação `jsonschema` Draft 2020-12 + parse `reg-core` + compilação Rust       │
 │ contra XSD/golden fixtures e `.jar` BCB — passa ou falha sem juízo subjetivo    │
 └──────────────────────────────────┬──────────────────────────────────────────────┘
                                    ▼
 ┌─ HUMAN PR & RADAR DELTA ────────────────────────────────────────────────────────┐
 │ UI de diff (§ da norma ↔ `RuleSpec` + testes) → catálogo versionado c/ `Vigencia`│
 └─────────────────────────────────────────────────────────────────────────────────┘
```

### Taxonomia de 5 Classes (LegalRuleML)
1. **Estrutural:** obrigatoriedade, tipo, tamanho (XSD / leiaute determinístico).
2. **Deôntica:** proibições, limites, obrigações condicionais (motor Rust compilado).
3. **Cross-Doc (L3):** consistência `3040 ↔ 3050 ↔ 2030 ↔ 4111` sobre `&DocSet`.
4. **Temporal:** vigência retroativa/futura, arredondamento por leiaute.
5. **Interpretativa:** enquadramento aberto ("quando aplicável") — IA sugere evidência, humano decide.

---

## 3. Copiloto de Validação, Output Guardrail & Protocolo Zero-PII Pós-WASM

- **Tier 0 Antes do LLM (`S1.4`):** dúvidas diretas sobre crítica ou campo (`bm25_exact_rule_hit && rrf_top1_bps >= 8500 && RuleStatus::Implemented`) são respondidas em **`<5 ms` e `$0`** a partir do próprio catálogo `RuleSpec` e dos cartões de campo, com zero risco de alucinação.
- **Copiloto SSE (`Tier 2/3`):** quando há necessidade de síntese contextual ou comparação entre regras, o `LlmTransport` responde em streaming SSE utilizando modelos **sem latência de thinking ativo no chat** (`Mistral Small 3.2 24B`, `Qwen3.5` instruct, `gpt-oss-20b` `reasoning_effort="low"`, ou `Claude Sonnet 4.5+` ZDR com Prompt Caching de 80–90% no prefixo público de regras).
- **Protocolo Zero-PII Pós-WASM (Etapa 3):** no navegador, o XML é validado 100% localmente via `reg-validate-core` (`wasm32`). Para explicar uma violação, o frontend envia ao `reg-server` **apenas** a tupla esquemática `{cadoc, catalog_version, rule_code, field_path, error_class}` — nenhum dado de tomador ou operação precisa sair do dispositivo do cliente.
- **Provenance Obrigatória + Output Guardrail:** nenhuma resposta de Copiloto é renderizada sem citar: **(a)** `rule_code` + versão existente no catálogo real; **(b)** artigo/`§` normativo correspondente; **(c)** condição violada. O guardrail pós-LLM **bloqueia qualquer afirmação de conformidade/aprovação** caso exista severidade `E`/crítica no `ValidationReport` real.

---

## 4. RAG Normativo Dual-Store (Single-Node On-Prem ↔ Plataforma `reg-server`)

```text
 Query ("regra preenchimento TaxEft / NR3 no 3040")
   ├── BM25 — Tantivy (Rust-native): códigos exatos (`B12`, `C71`), campos, INs/CCs
   ├── Denso/Esparso — `BGE-M3` (MIT) ou `Qwen3-Embedding` (Apache 2.0)
   ├── Fusão RRF em aritmética inteira (`i64`) → top 50
   └── Rerank — `BGE-Reranker-v2-M3` (MIT) / `Qwen3-Reranker` → top 8 + hash por §
```

- **Dois backings atrás do mesmo port `VectorStorePort`:**
  - **On-Prem / Single-Node / Edge:** `Tantivy` em disco + **`sqlite-vec` / índice mmap HNSW** + `BGE-M3` rodando na CPU/RAM via `ort` INT8 (~600 MB RAM).
  - **Plataforma `reg-server` (Etapa 3+):** `Tantivy` + **`pgvector` HNSW** (`sqlx` no Postgres 16). **Restrição estrita:** na v1, o RAG indexa **exclusivamente o corpus normativo público e cartões de campo** (zero mistura com dados de clientes; qualquer futuro índice privado de tenant exige partição física dedicada por `tenant_id` sob RLS `with_tenant` + fluxo de exclusão LGPD art. 18 testado).
- **Eval Contínuo em CI:** RAGAS / DeepEval sobre golden set de 200 perguntas regulatórias curadas (falha de faithfulness / context recall bloqueia merge).
- **Radar Delta Index:** o Radar mantém `sha256` por `§`; mudanças alteram e re-embeddam apenas os parágrafos modificados.

---

## 5. Conexão Dual Transparente (`DataPolicy`), LC 105 & Transferência Internacional

**Fato regulatório verificado:** nenhum provedor Cloud Frontier possui residência contratual de inferência API no Brasil até nov/2026 (infraestrutura física de data center em SP não equivale a residência contratual de API). Sob a **Res. CD/ANPD nº 29/2024** (multas até R$ 50M) e a **LC 105/2001 (art. 10 — sigilo das operações ativas, ilícito penal)**, a governança de dados é **aplicada em código no crate `reg-ai`**:

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DataPolicy {
    /// Modo A: Zero egress externo. Respeita `--offline=true` do `reg-cli`.
    /// Só permite `reg-s1-tabular` + `ort` + endpoint local (`mistral.rs`/`llama-server`/`SGLang`/`vLLM`).
    AirGapped,

    /// Modo B (Default SaaS Pro/Scale): Separação estrutural por classificação.
    /// • `PublicRegulatory` e `SchemaOnlyTokenized` → Cloud Frontier se `zdr_attested && anpd_scc_signed`.
    /// • QUALQUER texto livre c/ contexto de cliente/operação (`RestrictedPiiOrUnmasked`) →
    ///   obrigatoriamente On-Prem / VPC Brasileira (fail-closed: se local estiver off, retorna erro).
    HybridPolicyRouter,

    /// Modo C (Starter/Pro opt-in contratual): Cloud ZDR+SCC+DPA apenas para
    /// `PublicRegulatory` e `SchemaOnlyTokenized` (ex: pós-WASM `{cadoc, rule_code, field_path}`).
    /// `System 1` roda 100% local e qualquer texto livre com risco LC 105 é barrado do egress externo.
    CloudZdrFirst,
}

pub struct TenantAiConfig {
    pub tenant_id: TenantId,            // exclusivo de JWT RS256 verificado (ADR-0017; X-IF-ID banido)
    pub data_policy: DataPolicy,
    pub allow_telemetry_m1_m2: bool,    // opt-in contratual explícito
    pub s2_fast_endpoint: ModelTarget,
    pub s2_heavy_endpoint: ModelTarget,
    pub embedding_backend: EmbedTarget,
    pub vector_store: VectorBackend,
}

pub enum SensitivityClass {
    PublicRegulatory,        // DOU, INs/CCs BACEN, XSDs, leiautes públicos
    SchemaOnlyTokenized,     // apenas {cadoc, rule_code, field_path} ou Allowlist + HMAC efêmero
    RestrictedPiiOrUnmasked, // registros <Cli>/<Op>, texto livre de chat/ERP, retorno CRD bruto
}

pub enum ThinkingPolicy {
    Disabled,                  // obrigatório em JSON schema estrito no chat / Pass 2
    Budgeted(ReasoningEffort), // Low | Medium | High (Two-Pass assíncrono p/ Norma→Regra L3)
}

#[async_trait]
pub trait LlmTransport: Send + Sync {
    fn meta(&self) -> ProviderMeta; // { is_local, zdr_attested, anpd_scc_signed, max_ctx, jinja_sha256 }
    async fn complete_json(&self, req: StructuredReq) -> Result<RawModelOutput, AiError>;
    async fn stream_chat(&self, req: ChatReq) -> Result<SseTokenStream, AiError>;
}

#[async_trait]
pub trait EmbeddingPort: Send + Sync {
    /// Retorna embeddings normalizados quantizados em Q15 (`i16`) — zero float fora do runtime.
    async fn embed_q15(&self, texts: &[&str]) -> Result<Vec<Vec<i16>>, AiError>;
}

#[async_trait]
pub trait RerankPort: Send + Sync {
    /// Retorna `(indice_doc, score_bps)` em `u16` (`0..=10_000`).
    async fn rerank(&self, query: &str, docs: &[&str], top_k: usize) -> Result<Vec<(usize, u16)>, AiError>;
}
```

### Contratos de Bloqueio Anti-Vazamento
1. **Regra LC 105 (Resolução C4):** NER probabilístico (`moBERTo`) **não é autorização** para mandar texto livre de tomadores para API nos EUA/UE. Todo payload contendo observação de planilha, retorno bruto de CRD ou pergunta livre citando cliente/contrato é classificado deterministicamente pelo Estágio 0 do `S1.2` como `RestrictedPiiOrUnmasked` e nunca sai do perímetro brasileiro/on-prem.
2. **Allowlist + Generalização Antecipada:** antes de qualquer telemetria `M1/M2/M7` ou processamento VPC interno: `IPOC → token HMAC efêmero`, `CPF/CNPJ → tipo_pessoa + validade_dv`, `DtContr/DtVencOp → delta_dias`, valores financeiros → ordem de grandeza; `Aho-Corasick` + `moBERTo` + canários (`Canary FNR == 0%` hard blocker).
3. **Isolamento de Cache & Logs:** banido qualquer cache global estilo `ResponseCache.Invalidate` do Go v1; toda chave inclui `(tenant_id, ruleset_version)`; scrubbing obrigatório em `tracing` e Sentry.

---

## 6. Catálogo Atualizado de Modelos Open-Weights, Embeddings & Cloud (Nov/2026)

> **Critério Comercial Rígido:** [`rust/Cargo.toml`](../../rust/Cargo.toml) declara `license = "Proprietary"`. Portanto, todo o catálogo default on-prem utiliza exclusivamente modelos **Apache 2.0** ou **MIT**.
> **Explicitamente Bloqueados em Produção:** **`Itau-Unibanco/NorBERTo`** (`CC-BY-NC-SA-4.0`) e **`TabPFN` v2+** (pesos *Non-Commercial* sem licença comercial Prior Labs). Evitam-se também no self-host `Llama 4` (licença customizada e menor eficiência por GB vs `Gemma 4`/`Qwen3.6`), `Qwen3.8-2.4T-A95B`, `GLM-5.3` e `MiniMax-M3` (cláusulas comerciais/MaaS restritivas).

### 6.1 `System 2` Local (Open-Weights 2026) por Hardware & Memória Real

| Perfil Hardware | Modelo Primário | Alternativas | Params (Total / Ativos) | Licença | Memória Real (Pesos + KV Cache) & Observação | Serving Engine |
|---|---|---|---|---|---|---|
| **T0: CPU Air-Gap Mínimo** *(8–16 GB RAM, s/ GPU)* | **`Qwen3.5-4B`** *(ou `0.8B`)* | `Gemma 4 E4B` (~4.5B eff) / `E2B` | `4B` denso | Apache 2.0 | `Q4_K_M`: `4B` ~2.5–2.8 GB (~4–5 GB RAM c/ KV) | `mistral.rs` / `llama-server` CPU |
| **T1a: Mac Apple Silicon 24 GB** *(~16–18 GB úteis GPU)* | **`Qwen3.5-9B`** *(ou `Gemma 4 12B`)* | `gpt-oss-20b` *(MXFP4 ~14 GB, apertado c/ 8K KV)* | `9B` / `12B` | Apache 2.0 | `Q4_K_M`: ~5.5–7 GB total; preserva folga p/ macOS e `BGE-M3` na CPU | `mistral.rs` Metal / `llama.cpp` |
| **T1a+: Mac 32–48 GB RAM** | **`Gemma 4 26B-A4B`** *(32 GB)* | `Qwen3.6-35B-A3B` *(48 GB)* | `25.2B / ~4B` · `35B / 3B` | Apache 2.0 | `Gemma 4 26B-A4B` `Q4_K_M`: ~16–19 GB; `Qwen3.6-35B-A3B` `Q4`: ~23–26 GB | `mistral.rs` Metal / `llama.cpp` |
| **T1b: Mac Studio 128 GB** *(M4 Max / Ultra)* | **`gpt-oss-120b`** (`MXFP4`) | `Qwen3.6-35B-A3B` (`Q8`) · `Gemma 4 31B` | `116.8B / 5.1B` | Apache 2.0 | `gpt-oss-120b` `MXFP4`: ~61 GB pesos (~68 GB total c/ KV) | `mistral.rs` Metal / `llama.cpp` |
| **T2a: GPU 16 GB VRAM** *(RTX 4080S / L4 / T4)* | **`gpt-oss-20b`** (`MXFP4`) | `Qwen3.5-9B` (`FP8`) · `Gemma 4 12B` | `20.9B / 3.6B` | Apache 2.0 | ~13–14 GB VRAM *(Embeddings/`S1` rodam na RAM CPU)* | `vLLM` / `llama-server` CUDA |
| **T2b: GPU 24 GB VRAM** *(RTX 3090 / 4090)* | **`Mistral Small 3.2 24B`** *(JSON sem atrito)* **ou** **`Gemma 4 26B-A4B`** | `Qwen3.5-27B` *(Q4+KV `q8_0`, justo ~20 GB)* | `24B` · `25.2B / 4B` | Apache 2.0 | ~15–19 GB VRAM (`Q4_K_M` + KV `q8_0` 8–16K); **`BGE-M3`/`moBERTo` na RAM CPU** para evitar OOM. *(`Qwen3.6-35B-A3B` Q4 ~23 GB pede GPU ≥32 GB como RTX 5090).* | `llama.cpp` (`-ctk q8_0 --jinja`) / `mistral.rs` |
| **T3: GPU 48 GB VRAM** *(L40S / A40 / A6000)* | **`Qwen3.6-35B-A3B`** (`FP8`) | `Gemma 4 31B` (`FP8`) · `Mistral Small 3.2 24B` | `35B / 3B` *(Gated DeltaNet)* | Apache 2.0 | `FP8`: ~35–40 GB + folga p/ KV 256K e Embed/Rerank na GPU | **`SGLang`** (`RadixAttn`) / `vLLM` |
| **T4: GPU 80 GB VRAM** *(1× H100 / A100 OCI SP)* | **`gpt-oss-120b`** (`MXFP4`) | `Qwen3.6-35B-A3B` (`FP8`) + `Gemma 4 31B` | `116.8B / 5.1B` | Apache 2.0 | ~68 GB c/ KV → co-reside só c/ `BGE-M3` + `BGE-Reranker-v2-M3` (~4–5 GB), **não** `4B/8B` Emb | **`SGLang`** / `vLLM` |
| **T5: Multi-GPU** *(4×–8× H100/H200)* | **`Qwen3.5-397B-A17B`** (`FP8`) | `DeepSeek-V3.2` · `Kimi K2.5`* | `397B/17B` · `671B/37B` | Apache 2.0 (`Qwen`) · MIT (`DS`) | `397B` FP8 em 4×H200 / 8×H100 *(verificar cláusula UI OEM p/ Kimi Modified MIT)* | `SGLang` / `vLLM` TP+EP |

### 6.2 Embeddings, Rerankers, Encoders `System 1` & Cloud API
- **Embeddings & Rerankers:**
  - **`BGE-M3`** (569M, **MIT**, 8.192 ctx, Dense + Sparse + ColBERT) → default edge/on-prem e H100 + `120B` (~600 MB INT8 CPU via `ort` ou ~1.5–2.5 GB VRAM).
  - **`Qwen3-Embedding-0.6B / 4B / 8B`** (**Apache 2.0**, 32.768 ctx, Matryoshka MRL) → `0.6B` em CPU/24GB quando precisar 32K ctx; `4B/8B` apenas em ≥48 GB dedicados.
  - **`BGE-Reranker-v2-M3`** (568M, **MIT**) e **`Qwen3-Reranker-0.6B/4B`** (**Apache 2.0**).
- **Encoders `System 1` (`ort` INT8 em CPU/RAM):**
  - **`Tropic-AI/moBERTo`** (~150M, **Apache 2.0**, 8K ctx): NER PII / guardrail longa janela (`50–120 ms` CPU ou `<15 ms` GPU/ANE).
  - **`higopires/DeB3RTa`** (`70M small` / `426M base`, **MIT**): NLU jurídico-financeiro PT-BR para `S1.4` e Radar.
  - **`sagui-nlp/debertinha-ptbr-xsmall`** (~40M, **MIT**): anti-injection e classificação rápida (`5–15 ms` CPU em prompts curtos).
- **Cloud API Frontier (apenas Modos B/C com ZDR + DPA + Cláusulas-Padrão ANPD Res. 29/2024):**
  - `ReasoningVision`: **`Gemini 3 / 3.x Pro`** (1M ctx, visão nativa PDF + `calamine`) · fallback `Claude Opus 4.5+`.
  - `BalancedCached`: **`Claude Sonnet 4.5+`** (prompt caching no prefixo público de regras) · fallback `GPT-5.1+`.
  - `FastBatch`: **`Gemini 3 / 3.x Flash`** (Batch 50% off em perfis sintéticos `M4`) · fallback `GPT-5 mini` / `Claude Haiku 4.5+`.
  - *(APIs open-weights baratas de terceiros são vetadas para qualquer metadado de cliente por opacidade de jurisdição/DPA.)*

---

## 7. Engenharia de Serving 2026 & Hardening Adversarial (`Thinking`, `rmcp`, Injection)

1. **Disciplina `Thinking` vs `json_schema` (Sem Latência Dobrada no Chat):**
   - Aplicar gramáticas (`XGrammar`/`LLGuidance`/`GBNF`) com modo *thinking* ligado quebra ou gera loops de espaços em modelos híbridos (`Qwen3.6`, `Gemma 4`). Além disso, `Two-Pass` dobra a latência interativa e `--jinja` varia entre versões de engine.
   - **Solução em 2 trilhos no `LlmTransport`:**
     * *Copiloto interativo / chamadas rápidas (`Tier 2`):* usar preferencialmente perfis sem thinking (`Mistral Small 3.2 24B`, `Qwen3.5` instruct, `gpt-oss-20b` com `reasoning_effort="low"`), e **pinar versão exata da engine + hash SHA-256 do template Jinja** no manifesto de deploy (validado por *Cross-Engine Contract Tests* em CI).
     * *Batch assíncrono pesado (`Norma→Regra`, `M3`, `M4`):* usa padrão **Two-Pass** (Pass 1 no canal isolado `reasoning_content` → Pass 2 com `ThinkingPolicy::Disabled` + `json_schema` portátil).
2. **Portable JSON Schema Profile + Dupla Checagem em Rust:** profundidade `<= 4`, `additionalProperties: false`, zero `if/then/else`, **todos os valores decimais/taxas/datas/CNPJs como `string`**. Toda resposta exige `finish_reason == "stop"`, zero tag `<think>`/`<|channel>thought` residual, validação via `jsonschema` Draft 2020-12, parse via `reg-core` (`Centavos`/`TaxaDecimal`/`CnpjIdentifier`), checagem de `Provenance` contra o catálogo e Output Guardrail.
3. **Cache de Prefixo (`RadixAttention`):** as 253+ regras 3040 e cartões de campo formam um prefixo estático público de ~25–60K tokens; em GPU on-prem prioriza-se **`SGLang`** (`RadixAttention`) no Copiloto e **`vLLM`** em batches.
4. **Hardening de `rmcp` (MCP spec 2026-07-28) & Prompt Injection:**
   - **Desabilitar MCP sampling** (vetor primário de exfiltração/injeção); **pinar SHA-256 dos manifestos/descrições de tools** (*anti tool-poisoning*); expor na v1 apenas tools *read-only* (`query_rule`, `validate_cadoc`, `explain_violation`, `check_dossier`) sob JWT tenant-scoped; geração/transmissão STA exige aprovação humana fora da banda da IA; toda tool call e decisão entra na hash-chain do `reg-ledger`.
   - Planilhas, PDFs, retornos RAG e histórico entram sempre sob **spotlighting com delimitador randômico por requisição** + instrução isolada no system prompt + allowlist oficial (`*.bcb.gov.br`, `*.in.gov.br`, `*.cvm.gov.br`) no Radar.

---

## 8. Corpus Sintético (`M4`), Política de Fine-Tuning, TCO & Encaixe no Plano v3.0

- **Regras são Dados, Não Pesos:** fine-tuning de LLM jamais deve memorizar regras do BCB (mudam semanalmente e pesos opacos não citam `§`). A aprendizagem contínua (`M1/M2/M7`) alimenta o RAG, os goldens versionados com `Vigencia` e a calibração dos submodelos **`System 1` (`M8`)**.
- **`M4` Challenger / Solver / Judge:** cobre a matriz `regra × edge` com verificador objetivo (`System 0` Rust + XSD) atuando como `Judge 1` determinístico e `Judge 2` LLM (~10% amostra semântica, família distinta do gerador, calibrado por 30 auditorias humanas semanais no `M10`). Custo sintético: `~$0.05–0.20` por 1k exemplos (`100k CADOCs ≈ $20–80` em Batch).

### Política de Fine-Tuning vs. RAG vs. Destilação `M8`
| Abordagem | Custo | Risco | Veredito |
|---|---|---|---|
| **RAG Híbrido + `System 0` + `System 1`** | `~$0` incremental | Baixo | **Default:** regras mudam semanalmente via Radar; RAG delta atualiza em segundos com provenance. |
| **Destilação `M8` para `System 1` (`i64` / `DeB3RTa`)** | `$0–50/run` CPU/single-GPU | Baixo (gate `M6` obrigatório) | **Ativo na Etapa 4:** reduz latência para `<5–20 ms` e corta 75–90% do tráfego ao System 2. |
| **LoRA de estilo de saída (`System 2` local)** | `$50–500/run` (QLoRA) | Médio-baixo | Apenas para padronizar tom/formato de explicações em `S2` on-prem após M8 estável. |
| **Fine-tune para "aprender regras BACEN"** | `$5k–30k` | **Crítico** (overfit em vigência vencida + sem citação `§`) | **Proibido.** |

### Impacto no TCO & Fim do Degrau Binário "Cloud ou H100 ($7.3k/mês)"

Premissas base de volume (pós-Etapa 3/4): copiloto em ~10–25% das dúvidas não absorvidas pelo `S1.4 Tier 0/1`, ~25K tokens de prefixo público em cache (cache hit ~80%), ~1.500 tokens de saída, ~40 normativos/mês no Radar, 3 consultas RAG/validação.

| Estágio / Perfil de Uso | Composição Técnica | Custo Mensal Estimado |
|---|---|---|
| **Estágio 0 — Etapas 0–2 do Produto (Core 3040 + CLI + STA)** | **0% IA em runtime** (apenas dev-tooling offline opcional) | **$0 de infra IA** |
| **Estágio 1 — On-Prem Compacto (`T1a`/`T2a`/`T2b` + `System 1`)** | `S1` absorve `75–90%` em `<20 ms` CPU RAM + `Mistral Small 3.2 24B` / `Gemma 4 26B-A4B` / `gpt-oss-20b` (`mistral.rs`/`llama.cpp`) | **CAPEX único** (Workstation 24 GB VRAM ou Mac 24–32 GB — elimina degrau de $2.5k–7.3k/mês) |
| **Estágio 2A — Cloud Híbrido ZDR (~10 IFs · ~3k chamadas/mês)** | `S1` local (`$0`) + `BGE-M3`/`BGE-Reranker` self-host (`$0`) + `Flash` cache (`~$90`) + `Sonnet` casos duros (`~$120`) + `Norma→Regra` (`~$200`) + RAGAS/M4/Docling (`~$95`) | **`~$505/mês`** (ou menos c/ corte `Tier 0` do `S1.4`) |
| **Estágio 2B — Escala SaaS (~100 IFs · ~30k chamadas/mês)** | Mix `Flash`+`Sonnet` (`~$1.900`) + Extração (`~$400`) + Batch `M4`/RAGAS (`~$400`) + Parse/`rmcp` (`~$105`) | **`~$2.800/mês`** (`<15%` da receita projetada) |
| **Estágio 3 — Datacenter On-Prem Enterprise (`T3`/`T4`)** | `1× L40S 48 GB` (`Qwen3.6-35B-A3B` FP8, `~$2.555/mês`) ou `1× H100 80 GB` OCI SP (`gpt-oss-120b` + `BGE-M3` + `BGE-Reranker`, `~$7.300/mês`) | **Apenas sob gatilho contratual Enterprise** (repassado em contrato) |

### Encaixe nas Etapas 0–4 do Plano v3.0 (Sem Atrasar o 3040)
- **Etapa 0 (Saneamento):** **0% IA runtime**; aposentar `backend/internal/insights/` & `synth/`; marcar docs de jul/2026 como depreciados; extrair 253 regras de `SCR3040_Criticas.xls` via `calamine` (`224` ativas + `29` `Habilitado=n`).
- **Etapa 1 (Motor 3040 Provado):** **0% IA runtime**; definir apenas tipos puros `RuleId`, `ProvenanceRef`, `CadocCode` em `reg-core`.
- **Etapa 2 (CLI, Dossiê RC 18, `reg-ledger`, `reg-sta`):** **sem `ort`/GPU no binário core** (builds `ubuntu-latest` Docker+JRE & `windows-latest`, `--offline=true`); garantir extensibilidade JCS no `reg-ledger`.
- **Etapa 3 (`reg-server` `axum`, `frontend/` limpo, WASM Streaming):** entram **`reg-s1-tabular` (`i64`)** (no browser WASM apenas `S1.1` léxico + `S1.3` local, protocolo Zero-PII pós-WASM) + **`reg-ai` hexagonal (`--features ai`)** + RAG Público (`Tantivy` + `pgvector` + `BGE-M3` + `BGE-Reranker-v2-M3`) + `S1.2` + `S1.4 Tier 0` + Copiloto SSE com `Provenance` + Output Guardrail.
- **Etapa 4 (Multi-CADOC, Radar, IA Completa & `M1–M10`):** entram **`System 1` calibrado (`M2/M8`)**, **`S2 Local T0–T5`**, **Radar Two-Pass `Norma→Regra`**, **`M4`/`M7`/`M10`** e **`rmcp` hardened** + suíte `AirGapped` isolada de rede.

---

## 9. Matriz Consolidada de Gates Objetivos & Riscos Monitorados

| Risco / Frente | Severidade | Mitigação & Gate Objetivo de Promoção |
|---|---|---|
| **Promo pricing / oscilação API Cloud** | Alta | Interface única `LlmTransport` + modelos Open-Weights Apache 2.0/MIT testados por *Cross-Engine Contract Tests* em CI (`finish_reason == "stop"`, zero `<think>` leak, `100%` parse `jsonschema` + `reg-core`). |
| **Vazamento LC 105 / ANPD Res. 29/2024** | **Crítica** | Estágio 0 Allowlist determinística proíbe texto livre com cliente na nuvem externa; **`S1.2 Canary FNR == 0.00%`**; `0` violações de roteamento `RouteOnPremOnly`. |
| **Thinking Mode quebrando JSON Schema** | Alta | Copiloto usa modelos não-thinking / `reasoning_effort=low` + SHA-256 do template Jinja pinado em CI; Two-Pass isolado só em batch assíncrono. |
| **`rmcp` (tool poisoning / sampling injection)** | Alta | **MCP sampling desabilitado**; SHA-256 dos manifestos pinado; read-only default; HITL fora da banda p/ STA; hash-chain no `reg-ledger`. |
| **Invasão de floats / regressão determinística** | Crítica | [`layers.rs`](../../rust/crates/reg-cli/tests/layers.rs) (`0` floats/`unsafe` em `reg-core/cadoc/validate-core/ledger`); propriedade **`--no-ai`** bit-idêntica em CI. |
| **Alucinação no Copiloto / RAG** | Média-Alta | `S1.4 Tier 0` determinístico `<5 ms`; RAGAS/DeepEval em CI sobre 200 perguntas; Output Guardrail bloqueia `100%` respostas sem 3 campos de `Provenance` ou com falsa conformidade. |
| **Green-wash / OOM Local VRAM** | Média | Limites de VRAM/RAM reais por perfil (`T0–T5`), embeddings em RAM CPU quando GPU ≤24 GB, e gates `M6`/`M10` (`MDR >= 75%`, `Judge TPR/TNR >= 95%`, `ECE <= 0.025`). |

---

## 10. Fontes & Referências

- [`26-SYSTEM-ONE-DECISION-MODEL.md`](26-SYSTEM-ONE-DECISION-MODEL.md) · [`20-ARQUITETURA-RUST.md`](20-ARQUITETURA-RUST.md) · [`22-HARNESS-V4.md`](22-HARNESS-V4.md) · [`25-ROADMAP-EXECUCAO.md`](25-ROADMAP-EXECUCAO.md)
- Norm AI (https://www.norm.ai) · Regnology "The Agentic Gap" (https://www.regnology.net/en/solutions/regnology-platform/rgi) · LegalRuleML OASIS (https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/legalruleml-core-spec-v1.0.html)
- Preços API nov/2026: https://aistudio.google.com/docs/pricing · https://www.anthropic.com/pricing · https://developers.openai.com/api/docs/pricing
- MCP spec 2026-07-28 & `rmcp` (https://modelcontextprotocol.io/docs/2026-07-28/sdk · https://github.com/modelcontextprotocol/rust-sdk) · Unit42 / Microsoft MCP Prompt Injection
- vLLM vs SGLang (Red Hat, out/2026: https://developers.redhat.com/articles/2026/10/14/vllm-vs-sglang-choosing-the-right-inference-engine-for-your-workload) · Preços GPU OCI / AWS
- ANPD Res. CD/ANPD nº 29/2024 · LC 105/2001 · RC Conjunta 18/2025 · RAGAS / Docling
- HuggingFace Model Cards auditados em nov/2026: Qwen3.5 / Qwen3.6-35B-A3B / Qwen3-Embedding / Qwen3-Reranker (Apache 2.0), Gemma 4 E2B/E4B/12B/26B-A4B/31B (Apache 2.0), Mistral Small 3.2 24B (Apache 2.0), `gpt-oss-20b/120b` (Apache 2.0), `BAAI/bge-m3` & `BAAI/bge-reranker-v2-m3` (MIT), `Tropic-AI/moBERTo` (Apache 2.0), `higopires/DeB3RTa` (MIT), `sagui-nlp/debertinha-ptbr-xsmall` (MIT), `DeepSeek-V3.2` (MIT).
