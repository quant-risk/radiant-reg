# Blueprint v2.0 — Arquitetura Rust do Motor (Norma v2)

> **Documento central do blueprint de reconstrução.** Sintetiza as 9 pesquisas em
> `research/` (01-backend-audit · 02-tese-vs-realidade · 03-frontend · 04-rust-stack ·
> 05-concorrentes · 06-normativos · 07-engine-ia · 08-produto-pricing · 09-harness).
> Data: 16/11/2026 · Autor: orquestração ZCode sob direção de Henrique Costa.

---

## §0 — Por que Rust, dito honestamente

O ADR-0001 (jul/2026) rejeitou Rust com dois argumentos: *hiring* (10× mais difícil no BR)
e *GC irrelevante* (pausas de ~1ms vs 50ms de rede). Ambos colapsaram contra a realidade:

1. **O argumento de hiring pressupunha um time de 3-5 devs que nunca existiu.** O v1 inteiro
   (57k LoC) foi produzido por **1 pessoa + agentes IA**. Em 2026, o borrow checker é um
   problema de agente, não de dev escasso — e revisar Rust gerado por IA é mais fácil que
   revisar Go gerado por IA, porque o compilador prova metade das propriedades que um
   revisor humano teria de caçar.
2. **Performance nunca foi o argumento certo.** O ganho real de Rust neste domínio é
   **modelagem do regulatório em tipos**: um CADOC inválido deve ser *não-representável em
   compile-time*, e uma regra ausente deve ser *um erro de build*, não um `return nil`
   silencioso. É exatamente a classe de bug que a auditoria do backend encontrou no v1:

> **Achado central da auditoria (research/01):** o motor v1 só valida de verdade o 3040.
> As 170 regras do 3050 e 11 do 2070 estão implementadas, testadas e **órfãs** (nunca
> chamadas). 568+ críticas de catálogo são inalcançáveis em runtime (chaves de seed
> `2061-DLO` vs códigos de 4 dígitos na API). O L3 dentro de `ValidateFull` é no-op
> (códigos XD consultados mas nunca registrados). Há dois motores L1-L4 paralelos e uma
> colisão de código XD02 com semântica diferente. **Em Rust, cada um desses bugs é um erro
> de compilação ou um gate de build.**

Rust não é comprado por throughput (Go dava conta); é comprado por **correção reprodutível**,
que é a moeda de um produto regulatório vendido com SLA de precisão ("Zero Autuação™").

**O que permanece verdade a favor de Go e por que ainda assim migramos:** continuidade do
ecossistema Fortvna e velocidade de iteração (~30-50% mais rápida no 1º ano). Aceitamos o
custo porque o produto v2 muda de natureza: de "validador com regras Go hardcode" para
"motor de regras-as-data com verificação compilada", onde a estrutura de crates e o
sistema de tipos são o produto. Frontend **permanece TypeScript** (Next.js 15) — "tudo em
Rust" significa todo o **backend/motor**, não reinventar a web.

---

## §1 — A doutrina de design (8 princípios não-negociáveis)

1. **Compile-time correctness.** CADOC inválido não-representável; dispatch exaustivo;
   catálogo completo verificado em build. A classe de bug "regra ausente = skip silencioso"
   morre na linguagem.
2. **Regras como dados, motor como compilador.** A regra semântica é um artefato versionado
   (`RuleSpec` JSON) com **provenance obrigatória** (norma, §, data-base, vigência
   `[from, to)`). Código nativo Rust existe só onde a DSL de dados não alcança — e cada
   código nativo é referenciado pelo catálogo, sob pena de falha de build.
3. **Uma pipeline só.** L1→L4 é um único motor, não dois (v1 tinha `audit` e
   `generator/validation` paralelos). Toda validação — síncrona no wizard, batch no worker,
   via SDK, via MCP — passa pelo mesmo `ValidationPipeline`.
4. **Zero globals mutáveis.** O v1 forçava L3 serial (`parsedDRL/parsedDLP/...` globais).
   Em v2, `&DocSet` atravessa a pipeline como parâmetro e o L3 paraleliza com rayon.
5. **Silence is a bug.** Não existe caminho que "retorna nil" quando não sabe o que fazer.
   Erros são tipados (`thiserror`), verdicts são enums exaustivos, e "não implementado"
   é um estado **declarado** (`RuleStatus::Stub`), nunca um vazio silencioso.
6. **Provenance end-to-end.** Todo número, toda verdict, todo insight de IA carrega de onde
   veio. A UI exibe citação clicável (princípio UX "Trust-first"); o audit log é hash-chain.
7. **O harness é cidadão de 1ª classe.** Cada evento de produção (discrepancia, correção,
   mudança de leiaute) entra como ativo de avaliação (`research/09`) — **só depois do
   gate de anonimização** (LGPD; §13). O build não passa se regressar precisão/recall
   das regras tocadas.
8. **Paridade antes de migrar — por classe de teste.** Os 1.178 testes Go do v1 são a
   *spec executável*, mas v2 **diverge deliberadamente** do v1 em pontos declarados:
   skip silencioso vira erro tipado; os 153 stubs severity-I não são portados; regras
   órfãs no v1 (3050/2070/2030) passam a rodar; L4 (0% de cobertura no v1) nasce com
   corpus sintético. "100% green" é, portanto, definido por **política de transformação
   por classe de teste**, não como número global: (i) testes de stub são **descartados**
   (codificam um comportamento que v2 classifica como bug); (ii) asserts que codificam
   skip-silencioso são **reescritos** para o contrato novo; (iii) goldens de L4 e das
   regras órfãs são **gerados sinteticamente** e revisados contra o catálogo. Mecanismo:
   um **runner Go** serializa entrada→verdicts em fixtures JSON versionados; a suite
   Rust consome os fixtures. F2 não termina enquanto essa política não estiver 100% green.

---

## §2 — Visão macro do sistema

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Norma Console (Next.js 15 · TS)                      │
│  Wizard 2.0 · Calendário Normativo · Copiloto com citação · Health público  │
└───────────────┬─────────────────────────────────────────────────────────────┘
                │ HTTPS · OpenAPI 3.1 (utoipa) → tipos TS gerados · SSE
┌───────────────▼─────────────────────────────────────────────────────────────┐
│                        norma-api (axum · Rust)                              │
│   auth JWT+CSRF · rate-limit · RLS (SET LOCAL app.tenant_id) · utoipa      │
│                                                                             │
│  ┌──────────────────────────── VALIDATION PIPELINE (única) ───────────────┐ │
│  │ L1 Estrutura    L2 Semântica         L3 Cross-doc        L4 Histórico   │ │
│  │ tipos gerados   rules-as-data        &DocSet · rayon     diff determin.│ │
│  │ + xmllint       (RuleSpec+native)    (sem globals)       (golden-prev) │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│         ▲                      ▲                        ▲                    │
│  ┌──────┴──────┐      ┌────────┴───────┐       ┌───────┴────────┐          │
│  │ norma-      │      │ norma-         │       │ norma-         │          │
│  │ generator   │      │ connectors     │       │ ia             │          │
│  │12 generators│      │ Manual/File/   │       │ copiloto+RAG+  │          │
│  │ 13 CADOCs   │      │ API/DB/MCP     │       │ extração regras│          │
│  │ canonical→  │      │ (todos reais)  │       │ (router+PII)   │          │
│  │ XML/XSD     │      │                │       │                │          │
│  └─────────────┘      └────────────────┘       └────────────────┘          │
│                                                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  ┌────────────────┐  │
│  │ norma-sta   │  │ norma-radar  │  │ norma-harness │  │ norma-audit    │  │
│  │ WS+senhaws  │  │ diff leiaute │  │ flywheel M1-10│  │ hash-chain     │  │
│  │ PSTA300     │  │ auto-PR      │  │ gates de CI   │  │ tamper-evident │  │
│  └─────────────┘  └──────────────┘  └───────────────┘  └────────────────┘  │
└───────────────┬─────────────────────────────────────────────────────────────┘
                │ sqlx · apalis (jobs · storage Postgres) · fred (Redis)
┌───────────────▼─────────────────────────────────────────────────────────────┐
│  PostgreSQL 16 (único driver em prod) · pgvector · RLS por tenant           │
│  Redis 7 (sessão/rate-limit/lock)  · S3 (artefatos) · OTel+Prometheus+Sentry│
└─────────────────────────────────────────────────────────────────────────────┘
        │                                    │
   ┌────▼─────┐                     ┌────────▼─────────┐
   │ BACEN    │                     │ Worker (apalis)  │
   │ STA-h/ws │                     │ delivery·eval-   │
   │ senhaws  │                     │ nightly·radar    │
   │ CRD      │                     │ ·miner·synth     │
   └──────────┘                     └──────────────────┘
```

---

## §3 — Cargo workspace: 17 crates, bordas limpas

Modular monolith hexagonal: crate por bounded context, traits no domínio, DI por
composição explícita no `main` (sem framework de DI). `[workspace.dependencies]` pinado
(padroniza versões — pesquisa research/04 §R12).

```toml
# Cargo.toml (workspace root) — versões de nov/2026 (research/04)
[workspace.dependencies]
tokio = { version = "1.54", features = ["full"] }
axum = "0.8"
tower-http = { version = "0.7", features = ["cors", "fs", "trace", "compression-gzip"] }
sqlx = { version = "0.9", features = ["runtime-tokio", "tls-rustls", "postgres", "migrate", "chrono", "rust_decimal", "json"] }
reqwest = { version = "0.13", default-features = false, features = ["rustls-tls", "json"] }  # STA/LLM client (§10, §12)
rustls = "0.23"             # TLS — STA usa client cert ICP-Brasil (research/04 §R1)
fred = { version = "10.1", features = ["enable-rustls"] }
apalis = { version = "1.2", features = ["retry"] }  # storage Postgres durável — research/04 §R7
apalis-cron = "0.4"         # agendamentos (crate separado do apalis, research/04 §R7)
utoipa = { version = "5", features = ["axum"] }  # OpenAPI 3.1 do código (§9) — única versão não coberta pela research/04; cravar a exata na F1
cryptoki = "0.11"           # A3 via PKCS#11 (ICP-Brasil) — research/04 §R1
quick-xml = "0.42"
xsd-parser = "1.8"          # gera tipos Rust a partir dos XSD do BCB
calamine = "0.38"           # leitura XLS/XLSX de leiautes
rust_xlsxwriter = "0.102"   # export de planilhas (relatórios)
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1.2"            # JSON Schema dos RuleSpec
jsonschema = "0.61"         # valida RuleSpec no gate
rust_decimal = { version = "1.43", features = ["serde-with-str"] }
thiserror = "2"
tracing = "0.1"
tracing-opentelemetry = "0.31"
metrics = "0.25"
sentry = "0.49"
rmcp = "3.10"               # SDK MCP oficial Tier 1 (3.10.0, nov/2026) — server side
rayon = "1.10"
pgvector = { version = "0.4", features = ["sqlx"] }
tantivy = "0.22"            # BM25 nativo p/ RAG híbrido
zen-engine = "0.4"          # GoRules — decision tables do rules-as-data
insta = "1.49"              # golden files dos leiautes
proptest = "1.12"
criterion = "0.8"
```

### 3.1 Os crates (e o que herdam do v1)

| Crate | Responsabilidade | Herda do v1 (audit research/01) |
|---|---|---|
| `norma-domain` | `CadocCode`, `DataCompetencia`, `CnpjIdentifier` (por data-base — §4.3), `Money`, `RuleId`, `TenantId`, `Vigencia` | tipos de leiaute (spec); nada de código Go |
| `norma-layouts` | tipos Rust **gerados dos XSD BCB** + leiautes XLS → modelos; versionamento por vigência | `leiautes.json` (4.244 linhas) + XSDs oficiais dos diretórios por CADOC |
| `norma-rules` | `RuleSpec` (rules-as-data), carregador, decision tables (zen-engine), subset nativo | catálogo `criticas.json` **normalizado** (1.099 regras; dedup dos 11 códigos; schema único) |
| `norma-canonical` | `CanonicalDocument` — modelo intermediário tipado (ADR-007 gold) | interface do generator como spec |
| `norma-validation` | **A** pipeline L1-L4 unificada | algoritmo de hash-chain, semântica das ~358 regras reais como referência comportamental |
| `norma-generator` | 12 generators (trait + codegen dos caminhos comuns) | os 10 generators como spec (elimina a duplicação de helpers copiados) |
| `norma-connectors` | Manual/File/API/DB/MCP — **todos reais** | `FileAdapter` (parse real) como spec; os 4 stubs viram implementação nova |
| `norma-sta` | STA WS client, senhaws rotation, PSTA300 | **port quase 1:1 do WSClient (1.357 LoC sólidos)** — é a parte mais madura do v1 |
| `norma-radar` | fetch BACEN, diff semântico de leiautes, auto-PR, calendário normativo | Radar v2 como spec (diff XLSX + GitHub PR) |
| `norma-ia` | router multi-model, sanitizador PII, RAG, extração norma→regra, copiloto, MCP server | `insights` como spec de UX; pipeline é novo (research/07) |
| `norma-harness` | flywheel M1-M10, gates, miner, synth, cluster, scorecards | os 1.178 testes Go = **corpus seed** |
| `norma-audit` | hash-chain tamper-evident, evidências SOC 2 | algoritmo 1:1 (`auditlog` 92,5% cobertura — o código Go mais confiável) |
| `norma-storage` | sqlx repos, migrations, RLS, pgvector; **adapters hexagonais** dos ports de persistência declarados em `norma-harness`/`norma-ia` (sem ciclo — Apêndice A) | **migrations 001-026 viram artefato de paridade**; baseline nova Postgres-only consolidada (§8) |
| `norma-api` | axum: REST + SSE, auth, rate-limit, OpenAPI | contrato do `server.go` (~70 rotas) via `docs/openapi/v1.yaml` (corrigida) |
| `norma-worker` | apalis: delivery, eval-nightly, radar cron, miner, synth | worker Go como spec (claim atômico + backoff + DLQ já corretos) |
| `norma-cli` | CLI (seed, synth-gen, migrar catálogo, harness-gate) | `cmd/*` como spec |
| `norma-app` | composition root — wiring de api + worker + cli; **único crate que importa `norma-api`** (exceção da regra de bordas) | `cmd/*/main.go` como spec |

**Regra de bordas:** `domain` não importa nada além de std/serde/rust_decimal; `validation`
importa `domain+layouts+rules`; `api/worker/cli` importam tudo; **nenhum crate importa
`norma-api` exceto o composition root `norma-app` (exceção explícita)**. `norma-harness`
e `norma-ia` declaram ports de persistência como traits no próprio crate; `norma-storage`
os implementa (adapters) e a injeção concreta acontece em `norma-app`/`norma-worker` —
sem ciclo `storage↔harness/ia`. Entidades compartilhadas ficam em `norma-domain`.
Círculos = erro de arquitetura (o v1 manteve o DAG limpo com glue em `main` — mantemos a
mesma disciplina, agora verificável por `cargo workspaces`/`cargo-modules`).

**Duas fases de construção (fusão aprovada em 16/11/2026):** o desenho de 17 crates acima
fica como **teto de referência**. A **FASE 1 é o wedge on-prem com 4 crates** —
`reg-core` (clona responsabilidades de `norma-domain`), `reg-cadoc` (clona
`norma-layouts`+`norma-rules`), `reg-ledger` (clona `norma-audit`+`norma-storage`) e
`reg-cli` (clona `norma-cli`) — com **SQLite WAL portável** (schema migrável para
Postgres) e receita antes da plataforma. No **Plano v3.0 (`25-ROADMAP-EXECUCAO.md` §0.1)**,
esse núcleo evolui incrementalmente com **`reg-validate-core`** (`no_fs`, `wasm32`), **`reg-sta`**, **`reg-server`** (`axum` + Postgres 16 RLS), **`reg-s1-tabular`** (`i64` zero-float) e **`reg-ai` / `reg-ai-runtime`** (feature-flagged `--features ai` na borda, sem nunca quebrar `layers.rs` nem acoplar C++/GPU ao core). A **FASE 2** consolida o teto multi-tenant (~8–10 crates de domínio + wiring; desenho de 17 crates fica como referência).

### 3.2 Matriz CADOC → generator → fase (decisão explícita)

`CadocCode` tem 13 variantes, mas o v1 só tinha 10 generators — sem uma matriz, 3042,
3060 e 3044 ficam sem origem no port. Decisão publicada:

| CADOC | Generator | Fase | Nota |
|---|---|---|---|
| 3040 | gen3040 (herda spec) | F2 | foto mensal SCR |
| 3042 | gen3042 — **variante do gen3040** | F3 | correção SCR: mesma família de leiaute, semântica de correção |
| 3044 | gen3044 — **novo** | F3 | XML **por evento** (IN 682/2025): N eventos por remessa em envelope/zip; DocSet endereça por evento (§5.1) |
| 3050 | gen3050 (herda spec) | F2 | estatísticas agregadas |
| 4111 | gen4111 (herda spec) | F3 | saldos COSIF diários |
| 2030 | gen2030 (herda spec) | F3 | DRSAC — piloto dez/2026 via reg-cadoc Rust (S4) + B02 (ADR-0018) |
| 2060 | gen2060 (herda spec) | F3 | DRM |
| 2061/2062 | gen2061/gen2062 (herdam spec) | F3 | DLO/DLI |
| 2070 | gen2070 (herda spec) | F3 | DDR diário |
| 2160/2170 | gen2160/gen2170 (herdam spec) | F3 | LCR/NSFR |
| 3060 | gen3060 — **novo** | F3 | juros de cartão (Res BCB 468/2025 + IN BCB 621/2025) — onda já vigente |

Total: **12 generators** (10 herdados como spec + gen3044 + gen3060; gen3042 especializa
gen3040).

---

## §4 — Modelagem do domínio: a doutrina em código

### 4.1 `CadocCode` — enum exaustivo, nunca string

```rust
// norma-domain/src/cadoc.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u16")]
pub enum CadocCode {
    Scr3040,   // SCR — Risco de Crédito (mensal)
    Corr3042,  // Correção SCR
    Events3044,// Eventos de crédito (XML, por evento)
    Stats3050, // Estatísticas agregadas
    Cosif4111, // Saldos contábeis diários
    Esg2030,   // DRSAC — ESG (semestral; leiaute novo IN BCB 694)
    Mkt2060,   // DRM — Risco de Mercado
    LimOp2061, // DLO — Limites Operacionais
    LimIn2062, // DLI — Limites Individuais
    Cap2070,   // DDR — Requerimento de Capital (diário)
    Liq2160,   // DRL — LCR (diário; leiaute novo IN BCB 755)
    Liq2170,   // DLP — NSFR (mensal)
    Card3060,  // Juros do cartão (mensal — onda 2026)
}
// SEM impl TryFrom genérico caindo em fallback: código desconhecido é
// erro tipado RuleNotFoundError/CadocParseError — nunca "ok vazio".
```

O bug do v1 (`applyRegra` com 3 rotas e `nil` para todo o resto, research/01 §2) fica
**impossível**: o dispatch é `match cadoc` sem wildcard, e adicionar `Card3060` quebra a
compilação em todos os pontos que precisam decidir algo sobre um CADOC.

### 4.2 `Vigencia` — tempo de primeiro-class citizen

O regulatório é versionado por **data-base** (o Schema Registry v1 acertou isto; herdamos).
```rust
pub struct Vigencia { pub from: DataCompetencia, pub to: Option<DataCompetencia> }

pub struct Leiaute<C: Cadoc> { pub cadoc: C, pub version: LeiauteVersion, pub vigencia: Vigencia, pub xsd: XsdRef }

// Toda consulta de regra/leiaute recebe a data-base da competência:
// rules.resolve(cadoc, rule_id, competencia) → escolhe a versão vigente,
// mantém a anterior para re-validação histórica (L4 e replay de auditoria).
```

O modelo acima só é suficiente se **cada versão de regra carregar a própria vigência** —
senão `resolve()` não responde "qual spec valia na data-base X" (replay de auditoria e L4).
Decisão (correção à spec de DDL do research/09, que não tinha isso — a vigência vivia só
na tabela-pai `rule`, com `UNIQUE` por `cod`): `rule_version` ganha `effective_from` com
`UNIQUE (rule_id, effective_from)` + **exclusion constraint** por `daterange` (vigências
sem sobreposição entre versões) e `spec` **imutável** por versão; a vigência da
tabela-pai deixa de ser a única fonte. `ruleset_version` é uma **tag imutável** do repo de
catálogo, registrada com `corpus_hash` no `eval_run`; e `resolve(cadoc, rule_id,
competencia)` opera sobre o **snapshot compilado** da versão vigente — replay histórico
usa o snapshot da época.

### 4.3 Novtypes que matam bugs de campo

```rust
pub struct Money(Decimal);          // rust_decimal; arredondamento POR LEIAUTE
pub struct DataCompetencia { year: u16, month: u8 } // nunca "string competência"
pub struct RuleId(Arc<str>);        // ex: "B01", "XD02", "3050-TXB-014"
pub enum Rounding { HalfUp, HalfEven, Truncate }    // default por leiaute explícito

// Identificador de CNPJ — TEMPORAL, parametrizado por data-base (IN BCB 729/2026,
// CNPJ alfanumérico; RFB atribui CNPJ alfanumérico a novas inscrições desde jul/2026):
//   competência < 2026-07 ⇒ numérico:     [0-9]{8} | [0-9]{14}
//   competência ≥ 2026-07 ⇒ alfanumérico: [0-9A-Z]{8} | [0-9A-Z]{14}, com dígito
//   verificador alfa. O formato aceito por consulta é sempre o da data-base consultada.
pub struct CnpjIdentifier { /* 8 ou 14 posições, validado por competência */ }
```
A pesquisa (research/04 §R6) alertou: `rust_decimal` usa banker's rounding por padrão —
cada leiaute declara sua estratégia, e o generator aplica via `Money.round(strategy)`.
Arredondamento implícito é bug regulatório.

**O `Cnpj8(u32)` do desenho anterior morre aqui.** O identificador de remetente virou
`CnpjIdentifier`, **parametrizado por data-base** conforme a IN BCB 729/2026: competências
anteriores a 2026-07 aceitam apenas a forma numérica (`[0-9]{8|14}`); a partir de 2026-07,
a forma alfanumérica (`[0-9A-Z]{8|14}`) com dígito verificador alfa. Consequência de
assinatura: **toda consulta de leiaute/regra passa a receber o identificador pela
competência** (`rules.resolve(cadoc, rule_id, competencia, &CnpjIdentifier)`) —
re-validação histórica de competência pré-2026-07 usa a forma numérica.

### 4.4 Provenance como tipo

```rust
pub struct Provenance {
    pub norma: NormRef,          // "IN BCB 733/2026, art. 3º" / "CC 3869, anexo 5"
    pub data_base: DataCompetencia,
    pub fonte: Fonte,            // PlanilhaCríticas | XSD | Manual | FAQ | Descoberta(DiscrepanciaId)
    pub capturado_em: DateTime<Utc>,
}
pub struct Verdict {
    pub rule: RuleId, pub outcome: VerdictOutcome, // Ok | Warn | Critical | Info
    pub campo: Option<CampoPath>,                  // ex: "root/operacao/ipoC"
    pub mensagem: Option<MensagemGerada>,          // humanizada (IA ou template)
    pub provenance: Provenance,                    // OBRIGATÓRIO — UI cita isso
}
```
A UI nunca exibe um número/verdict sem `Provenance` (princípio UX Trust-first, research/24).

### 4.5 Política zero-float no domínio

O gen2030 do v1 calculava concentração em **float64** — a classe de erro que não se
corrige com teste pontual, só com lint e tipo. Política para `norma-domain` e demais
crates de domínio:

- Valores monetários trafegam como **`Centavos(i128)`** (inteiro exato); razões e
  percentuais como **`TaxaDecimal`** (rust_decimal com arredondamento declarado — §4.3).
  Float nunca cruza a fronteira do domínio.
- **`#![forbid(unsafe_code)]`** nos crates de domínio.
- **Gate do workspace:** `deny(clippy::float_arithmetic, clippy::float_cmp,
  clippy::cast_precision_loss)` junto do `-D warnings` no CI (§14) — regressão a float é
  erro de build, não code review.

---

## §5 — O motor de validação unificado (L1-L4)

### 5.1 Uma pipeline, dois modos de entrada

```rust
pub struct ValidationPipeline { l1: L1Xsd, l2: L2Rules, l3: L3CrossDoc, l4: L4Historico }

pub struct DocSet<'a> {          // L3 precisa do conjunto — emprestado, NÃO global
    // múltiplos documentos por CADOC: o 3044 chega com N eventos por remessa
    // (IN 682/2025) e o L4 compara por envio — um doc único por código não
    // representa o domínio.
    pub docs: HashMap<CadocCode, Vec<&'a ParsedDoc>>,
    pub competencia: DataCompetencia,
    pub tenant: TenantCtx,
}

impl DocSet<'_> {
    /// Endereçamento do documento dentro do grupo — contrato das regras L3/L4:
    /// a regra declara se quer o envelope único (`primary`) ou a lista (`events`).
    pub fn primary(&self, cadoc: CadocCode) -> Result<&'a ParsedDoc, ValidationError>;
    pub fn events(&self, cadoc: CadocCode) -> &[&'a ParsedDoc];
}

impl ValidationPipeline {
    pub async fn run(&self, set: &DocSet<'_>, history: &HistoryView<'_>)
        -> Result<ValidationReport, ValidationError>
    {
        let l1 = self.l1.validate(set).await?;               // tipos gerados + xmllint autoridade
        let l2 = self.l2.validate(set, &l1)?;                // rules-as-data + nativas; rayon p/ paralelismo
        let l3 = self.l3.validate(set, &l2)?;                // cross-doc; paraleliza por par de CADOCs
        let l4 = self.l4.validate(set, history, &l2)?;       // diff vs. data-base anterior
        Ok(ValidationReport { l1, l2, l3, l4, provenance: self.current_catalog_provenance() })
    }
}
```

### 5.2 L1 — três camadas de defesa (o risco XSD resolvido honestamente)

A pesquisa (research/04 §R3) concluiu: **não existe validador XSD 1.0 completo puro-Rust**
mature em 2026 (crate `xmlschema` é 0.0.x). Estratégia em 3 camadas, que também corrige o
"XSD caseiro com paths relativos que degrada silenciosamente" do v1:

1. **Prevenção:** tipos gerados de XSD via `xsd-parser` — o gerador de XML produz árvore
   válida por construção (newtypes para enums de domínio, `Option` para minoccurs=0).
2. **Validação estrutural própria inline:** rápida, customizada, cobre cardinalidades e
   faixas (as regras COSIF passam além do XSD de qualquer forma).
3. **Autoridade final:** `xmllint` (libxml2) como **execução one-shot com semáforo de
   concorrência** antes de qualquer envio STA — "pool de subprocessos persistentes" não
   existe: cada `xmllint` é um processo novo; o paralelismo é controlado por semáforo com
   **quota por tenant** (§14). **Endurecimento fixado** (não configurável por request):
   `--nonet` sempre; **nunca `--noent`** (substituição de entidades em XML não-confiável
   abre XXE/billion-laughs); cap de bytes por documento + `--maxmem`; **timeout rígido com
   kill** e erro tipado. Falha do xmllint é **erro explícito** (`L1AuthorityUnavailable`,
   `L1Timeout`) — nunca degrada silenciosamente para "root-tag check" como no v1.
   `libxml2` entra no ciclo de patch do distroless/cc e o fuzz nightly de entidades/
   DOCTYPE previsto no CI (§14) cobre o caminho de parsing. Se o custo do spawn medir
   acima do alvo p95, o caminho B é cache de schema compilado via FFI libxml2
   (`xmlSchemaValidCtxt`) no mesmo processo — decisão medida, não assumida.

```rust
pub struct L1Authority { sem: Semaphore, max_bytes: usize, timeout: Duration }
impl L1Authority {
    pub async fn validate(&self, doc: &[u8], xsd: &XsdRef) -> Result<XsdReport, L1Error> {
        // flags fixadas: --nonet --maxmem=… (NUNCA --noent); kill no timeout;
        // erro de infra aqui é visibility total: cai no report como Critical de SISTEMA,
        // não como "passou" — regra Silence is a bug.
    }
}
```

### 5.3 L2 — rules-as-data com catalog completeness gate

```rust
// norma-rules/src/spec.rs — a regra é DADO versionado, com provenance
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct RuleSpec {
    pub id: RuleId,
    pub cadoc: CadocCode,
    pub kind: RuleKind,             // Estrutural | Deontica | CrossDoc | Temporal | Interpretativa
    pub severity: Severity,         // Critical | Warn | Info
    pub vigencia: Vigencia,         // em que data-bases vale
    pub expression: RuleExpr,       // DSL de dados (decision table zen-engine) OU
    pub native: Option<NativeRuleRef>, // referência a função Rust (último recurso)
    pub mensagem: MessageTemplate,  // humanizada, com placeholders tipados
    pub provenance: Provenance,
    pub status: RuleStatus,         // Implemented | DataOnly | Stub{motivo} — "stub" é declarado
}
```

**Catalog completeness gate — dividido por onde cada verificação é possível.** `build.rs`
não enxerga a expansão de proc-macro, então (b) não pode morar ali. Layout correto:
(a) todo `RuleSpec` do catálogo passou `jsonschema` e (c) **toda regra de cada CADOC tem
implementação OU `status: Stub{motivo}`** — ambos validados no **build.rs sobre o JSON do
catálogo**; (b) toda `NativeRuleRef` existe é verificada **pelo compilador**: o xtask
gera o registry com referências tipadas por caminho (`crate::native::scr3040::b01`) e
handler ausente = **erro de compilação** (alternativa aceita: `linkme`/`inventory` para
coletar as referências + teste dedicado que compara o conjunto registrado com o catálogo).
Isso mata os 153 stubs-I silenciosos do v1 e torna a cobertura 3040/3050/2061 uma
métrica **do build**, não de README (fim do "275? 126? 25? 266?" — os 4 números
simultâneos que o README v1 exibia).

```rust
// dispatch exaustivo — o coração da correção
impl L2Rules {
    pub fn apply(&self, ctx: &RuleContext<'_>) -> Vec<Verdict> {
        match ctx.cadoc {
            CadocCode::Scr3040 => self.apply_3040(ctx),
            CadocCode::Stats3050 => self.apply_3050(ctx),
            // ... todos os braços — SEM `_ => vec![]`
            // (-D clippy::wildcard_enum_match_arm; sem wildcard, a exaustividade
            //  já é garantida pelo compilador)
        }
    }
}
```

**Toggles por tenant não morrem no port.** O v1 tem `ruleprefs`/`disabled_rules` (672 LoC,
RLS herdada) — funcionalidade em uso que não tinha destino no v2. Decisão: camada runtime
**RuleOverride** por tenant, avaliada **antes do dispatch L2** (tabela herdada; cache
chaveado por tenant — §14). Regra desabilitada não é skip silencioso: o report declara o
estado explícito `Skipped { motivo, decidido_por }` para o par (regra, tenant) e a
decisão entra no norma-audit.

### 5.4 L3 — cross-doc como `&DocSet`, sem globals

Reparos das 3 feridas do v1 (research/01): códigos XD nunca registrados; colisão XD02 com
semântica dupla; `&DocSet` substitui os globals `parsedDRL/parsedDLP/...` (que forçavam L3
serial). O L3 roda em rayon por par (3040↔3050, 3040↔2030, 3040↔4111, 2061↔2070, ...) e
cada regra cross-doc tem id namespaceado por par (`XD-3040-3050-01`) — fim da colisão.
As regras recebem `DocRef { cadoc, seq }` (endereçamento dentro do `DocSet` — §5.1),
porque "o doc do CADOC X" não é único: o 3044 tem N eventos por remessa e o L4 compara
por envio.

### 5.5 L4 — histórico determinístico

Compara a competência com a anterior **por tenant**, com regras de tolerância tipadas
(variação % máxima por campo/agregado) e emits `Verdict::Warn` com diff citado. O v1 tinha
L4 com **0% de cobertura** — em v2, o golden corpus da L4 é construído a partir dos
fixtures de geração (mesma entrada em duas competências sintéticas).

---

## §6 — Canonical Document e os generators

O ADR-007 do v1 acertou o desenho: **IA/adapters nunca escrevem XML direto** — tudo passa
pelo `CanonicalDocument` tipado, e o generator só ele conhece o leiaute. Mantemos e
endurecemos:

```rust
pub trait Generator {
    fn cadoc(&self) -> CadocCode;
    fn from_canonical(&self, doc: &CanonicalDocument, vigencia: LeiauteVersion) -> Result<Vec<u8>, GenError>;
}

// Os 10 generators compartilham 70% de mecânica (zip, hash, envelope, escaping,
// ranges de vencimento). No v1 esses helpers foram copiados 10×. Em v2:
// macro comum `cadoc_generator!` + traits de blocos (Envelope, Zipper, Hasher),
// com teste de golden file por CADOC (insta snapshot do XML gerado).
```

Defaults proibidos: o v1 injetava `UF="SP"` e email fake quando faltava campo (research/01
§riscos). Em v2, campo ausente obrigatório = `GenError::MissingField { path, provenance }`.

---

## §7 — Conectores: o 80% que faltava

Os 5 conectores existiam como interface com 4 stubs (research/03). O connector é a ponte
entre o dado bruto da IF e o `CanonicalDocument`:

| Conector | v2 | Notas |
|---|---|---|
| Manual | Wizard 2.0 (frontend) | mantém; validação passa a streaming SSE |
| File | `calamine` real | XLSX/XLS de leiaute → mapeamento de colunas com perfil por CADOC (o wizard já tem o UX para isso — o backend é que era stub) |
| API | REST da IF → Canonical | schema versionado por `schemars`; idempotency-key |
| DB | Postgres read-only da IF | credenciais no Vault; SELECT read-only com timeout e row-cap |
| MCP | agente IA da IF via `rmcp` server | tools allow-listed; validate/search read-only, `generate_cadoc` = escrita com human-in-the-loop; tool calls na hash-chain (§12.3) |

---

## §8 — Storage: Postgres único, RLS por transação

Correções diretas dos achados de auditoria: v1 rodava **SQLite como default** disfarçado
de Postgres e tinha migrations com RLS duplicada. Em v2:

```rust
// norma-storage — padrão RLS imutável, centralizado (o v1 acertou a semântica SET LOCAL;
// a assinatura abaixo é a que de fato compila — o `E: From<sqlx::Error>` é exigido pelos
// `?` sobre begin/execute/commit, erro E0277 sem ele)
pub async fn with_tenant<'a, T, E, F>(pool: &'a PgPool, tenant: TenantId, f: F) -> Result<T, E>
where
    E: From<sqlx::Error>,
    F: for<'c> FnOnce(&'c mut Transaction<'static, Postgres>) -> BoxFuture<'c, Result<T, E>>,
{
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")   // true = LOCAL à transação
        .bind(tenant.0).execute(&mut *tx).await?;
    let out = f(&mut tx).await?;
    tx.commit().await?;
    Ok(out)
}
// `set_config(..., true)` = SET LOCAL: vazamento entre conexões do pool é impossível
// (o padrão recomendado pela pesquisa research/04 §R2 — não usar SET global).
```

**Origem do tenant_id (ADR-0017):** o `tenant_id` vem **exclusivamente de claim
verificada do JWT RS256**, resolvida no middleware axum — nunca de header client-side
(o `X-IF-ID`, bridge de dev do v1, é **banido**).

- **Migrations: baseline nova Postgres-only, não as 26 SQL do v1.** As 001-026 ficam como
  **artefato de paridade**, não como baseline executável: carregam débito SQLite
  (placeholders `?`, workarounds de formato datetime, `BEGIN IMMEDIATE`, skip
  `@postgres-only`) e RLS espalhada em 3 arquivos (012/014/026). A baseline nova é um
  **squash** consolidado com a RLS numa única migração **FORCE** e — ponto crítico — o
  rename do GUC (`app.if_id` → `app.tenant_id`) é **migração explícita** que recria as
  policies (`ALTER`/`CREATE POLICY`): as policies do v1 filtram `app.if_id`; um helper
  que só seta `app.tenant_id` sem recriar as policies produz **deny-all silencioso**
  (zero linhas em toda tabela com policy). Herda-se o schema 1:1, **não as policies sem
  esse ajuste** (§15). Novas migrações: tabelas do harness/RAG (abaixo) e
  `idempotency_keys` (§9).
- **Teste cross-tenant obrigatório (testcontainers, no CI):** para toda tabela com
  policy, consulta sob `with_tenant(A)` não retorna linhas de `B` — inclui o caso de
  forjar o header `X-IF-ID`.
- **DDLs do harness/RAG com as correções da spec research/09:** `eval_case.vigencia_fim`
  (exigida pelo M3 — casos golden velhos são marcados, não deletados) + índice
  `(doc_cod, vigencia, vigencia_fim, status)`; índice `eval_result (case_id, run_id)`
  (hoje só existe a PK `(run_id, case_id)`); `scorecard` com `UNIQUE (doc_cod, tenant_id,
  ruleset_version, engine_version, run_id)` — a chave natural do M5, hoje sem constraint —
  + índice `(tenant_id, created_at)`; `eval_run.corpus_hash` (fingerprint do harness —
  comparar só runs compatíveis); `discrepancy`/`correction` com `payload_hash` (o miner
  deduplica proceduralmente sem coluna de hash) e `rule_candidate` com partial unique
  `WHERE cluster_signature IS NOT NULL` (o `UNIQUE` simples admite múltiplos NULLs) —
  miner idempotente via `ON CONFLICT DO NOTHING`. Pruning com retenção definida
  (ADR-0016): ~5k `eval_result`/dia não vive sem política de corte.
- **Tabelas do harness carregam dado de tenant:** `discrepancy`, `correction`,
  `scorecard` entram na baseline sob RLS (ou role de serviço dedicada); o worker
  **anonimiza antes de `status=active`** (gate obrigatório — §13); o harness-gate de CI
  lê **snapshot exportado (artefato)**, nunca o Postgres de produção; held-out usa
  **chave separada**.
- **pgvector + Tantivy:** dense (pgvector HNSW) + BM25 com fusão RRF (research/07 §4).
  O índice Tantivy é on-disk e precisa ser consultável onde a query chega: **in-process
  no norma-api também** — o worker **constrói** o índice e o publica com **troca atômica
  por geração/segmento** (sem isso, o API não teria caminho de query sobre índice escrito
  por outro processo). O indexer é **idempotente e reconstruível do Postgres**
  (`chunk_id` + `content_hash` como chave de sync), com SLO de staleness publicado.
  `norma_chunks` fica restrito a normativo público — índice on-disk **não tem RLS**; se
  um dia indexar conteúdo de tenant, entra filtro por tenant no segmento.
- **Audit hash-chain segmentada:** a cadeia 1:1 do v1 (`payload_hash` + `prev_hash`) não
  suporta retenção (podar quebra `prev_hash`) nem verificação O(n) aceitável. Em v2:
  cadeia **segmentada com checkpoint periódico assinado e ancorado externamente**
  (object storage com Object Lock), **particionamento mensal**, verificação por segmento
  + continuidade de head; retenção quente/fria conforme SOC 2 (ADR-0016).
- **Artefatos XML/ZIP:** payload > ~1 MB vai a storage de objetos com
  `key = content-hash`; o Postgres guarda hash + metadata (sem TOAST gigante).
  `trait ObjectStore` com adapters S3/MinIO/filesystem — no on-prem, filesystem (sem S3).
  Lifecycle/GC compatível com a imutabilidade da hash-chain (nunca apagar objeto
  referenciado por `audit_log`).
- **Migração de dados do piloto Go (até fev/2027):** se v2 **compartilha o Postgres** com
  o piloto Go, as migrações novas são **aditivas**; senão, **ETL** com a chave de dedup
  dos 11 códigos sufixados (`2061-DLO` → `2061`), critério de vencedor por
  provenance/recency, e **mapa old→new aplicado a `disabled_rules`, `rule_failures` e
  `eval_case.rule_cods`** — o estado v1 quebra sem renormalização. A triagem
  1.099→RuleSpec **não cabe na F0 de 2 semanas**: F0 entrega schema/gate/ETL de chaves;
  a conversão por regra é incremental (F0-F2), com estado declarado por `RuleStatus`.
- **SQLite:** na plataforma (FASE 2), restrito a testes unitários de engine (sem I/O de
  rede), nunca em prod. **Exceção da fusão:** o wedge V1 da FASE 1 roda **SQLite WAL
  portável** em produção on-prem (§3.1), com schema migrável para Postgres — o Postgres
  único volta a ser regra quando a plataforma multi-tenant existir.

---

## §9 — API (axum): contrato como produto

- `utoipa` gera OpenAPI 3.1 **do código** (anotações nos handlers) — fim do drift
  OpenAPI↔routes que o v1 tinha (46 paths spec vs ~70 rotas reais, corpo do crossdoc errado).
- Frontend Next.js consome tipos TS gerados (`openapi-typescript`) — o `api-types.d.ts`
  que existia no v1 mas nunca foi gerado.
- SSE para streaming de validação do wizard; **o middleware autentica e fixa o tenant no
  handshake, e a filtragem por tenant é server-side antes do emit** (o v1 entregava tudo
  e filtrava por `kind` no cliente); teste de isolamento com dois tenants no CI. SSE com
  timeout explícito (gotcha hyper: SSE bloqueia graceful shutdown — middleware de
  shutdown-aware stream).
- Erros: envelope único `{"error": {"code", "message", "provenance?", "trace_id"}}` —
  o `SafeError` do v1 (sanitização consistente) é mantido como padrão.
- **Idempotency-Key obrigatória em POST de geração/envio, com implementação real:**
  migration `idempotency_keys (tenant_id, key, request_hash, response_ref, expires_at)`
  com `UNIQUE (tenant_id, key)`; consumo **atômico** via `INSERT … ON CONFLICT` na API e
  **no caminho de envio do worker** (sem isso, o retry do worker duplica envio STA);
  job de TTL para expiração. Escopo definido: geração e envio.
- **Webhooks:** HMAC-SHA256 sobre `timestamp + body` com **janela de 5 min**
  (anti-replay), comparação **constant-time**, secret **por tenant com rotação**, payload
  **minimizado** (sem dado de titular); **deliveries registradas no norma-audit**. O
  worker de delivery tem **egress filtrado** — bloqueia RFC1918/link-local/metadata
  endpoint e re-checa DNS após resolução (SSRF: a URL é da IF).

---

## §10 — STA client: port quase 1:1 (o tesouro do v1)

O WSClient Go (1.357 LoC) é a parte mais madura do v1: fluxo oficial BACEN, verificação
X-Content-Hash, chunked range upload, Basic auth, retry. Vira **port fiel** em Rust com
`reqwest` + `rustls`:

- **Autenticação STA-WS (correção da fusão — a credencial do WS não é certificado):**
  HTTP Basic **preemptivo** (RFC 7617) com `UUUUUDDDD.operador:senha`, senha gerida em
  `/senhaws` (manual STA WS v1.5, p. 4/6). O operador é cadastrado no Sisbacen com
  autorização **PSTA300** como **"usuário especial"** (Res. BCB 366/2024, art. 7º, IV) —
  **conta de serviço, sem dependência de MFA interativo no runtime do daemon**. A conta
  humana máster usa **Autran com 2FA obrigatório** (Comunicados BCB 45.206 e 45.339/2026:
  homolog desde 11/05/2026, produção desde 10/06/2026, com exceção explícita para
  usuários especiais) para administrar credenciais em STRA1300.
- **Certificados A1/A3 (ICP-Brasil) atendem outros canais Sisbacen — não são aplicáveis
  ao STA-WS.** O material permanece no desenho para os canais que exigem assinatura
  (PEM A1 trivial; A3 via `cryptoki`/PKCS#11 — research/04 §R1, com feature flag);
  **fallback A3→A1 nunca é automático** (degradaria o não-repúdio): liberado/bloqueado
  por configuração por IF (ADR-0015). O spike A3/PKCS#11 fica **fora do caminho
  crítico** — sem vínculo com o envio STA (§16).
- **Secrets (ADR-0015):** senhaws e chaves A1/A3 em Secrets Manager/Vault — nunca em env,
  arquivo ou `tracing`; material de assinatura em memória com `zeroize`.
- Senhaws rotation: port do cliente com rotação agendada + lock distribuído no Redis; a
  rotação roda **sob norma-audit** (hash-chain) e a **expiração de certificado tem
  monitor no Calendário Normativo** (alerta 30/7/1 dias — cert vencido é evento de
  operação, não surpresa).
- Cada envio grava: hash pré-envio, payload, resposta, timestamps — alimenta `norma-audit`
  e cria o rastro que o Radar usa para **correlacionar retornos de erro do BCB → nova
  regra candidata** (flywheel M1/M7).

---

## §11 — Radar regulatório: o relógio do produto

O Radar deixa de ser feature e vira **fonte de verdade temporal** do sistema:

1. Crawler diário (leiautedocumentoscrd, INs, CCs, FAQ) → hash por § (igual ao RAG).
2. **Diff semântico de leiautes** (calamine diff XLS+XSD): campo novo/renomeado/tipo mudou
   → classificação breaking/minor/new + impacto por CADOC.
3. Saídas automáticas: (a) issue de leiaute novo para `norma-layouts` codegen;
   (b) suite de regressão da nova versão (flywheel M3); (c) atualização do calendário
   normativo (a homepage do Console); (d) re-index delta do RAG (<$1/doc).
4. Auto-PR no repo de catálogo com o diff — humano aprova (ponte human-in-the-loop).

O calendário verificado (research/06) que o Radar nasce rastreando: IN 733 onda 3
(nov/2026 — **agora**), leiaute DRSAC dez/2026, GRSAC 1º/1/2027 (Res 586 + IN 772),
LCR/LCRS 90%→100% em 2027 (Res CMN 5.296 + Res BCB 560), FRTB SENS 2027 (Res 470/2025).

---

## §12 — IA no motor (pontos de integração; detalhes em `21-ENGINE-IA.md` v3.0 e `26-SYSTEM-ONE-DECISION-MODEL.md`)

A fronteira não-determinística opera em **3 camadas cognitivas (`System 0 → System 1 → System 2`)** com isolamento estrito de crates e lints. Regra estrutural: **IA nunca produz XML, nunca decide verdict e nunca contamina `reg-core/cadoc/validate-core/ledger/sta` com floats (`float_arithmetic = "deny"`) ou pesos ML**. Pontos:

1. **`System 1` (`reg-s1-tabular` zero-float `i64` + `reg-ai-runtime` `ort` INT8 atrás de `--features ai`):** implementa `S1.1` Column Auto-Mapper (`Proposal<MappingDraft>` sempre exigindo confirmação humana + 4 travas de `mapping_locks`), `S1.2` PII/LC 105 & Injection Gate (Allowlist estrutural determinística + Aho-Corasick + `debertinha`/`moBERTo` + canários `FNR == 0%`), `S1.3` Portfolio Sanity & Anomaly Scorer (`TreeSHAP` em `i32` bps, `SanityHoldGate` aditivo) e `S1.4` Router 4-tier (`Tier 0` determinístico `<5 ms` antes de qualquer LLM). Exporta na fronteira apenas `ScoreBps(u16)`, `i32` e `Vec<i16>` (`Q15`), além de `S1AuditRecord` zero-float para `reg-ledger::jcs`.
2. **Copiloto de validação & Conexão Dual (`DataPolicy::{AirGapped, HybridPolicyRouter, CloudZdrFirst}`):** alternável por `TenantAiConfig` sem mudar código de domínio. **Regra LC 105 strict (Resolução C4):** apenas `PublicRegulatory` e `SchemaOnlyTokenized` (ex.: `{cadoc, catalog_version, rule_code, field_path, error_class}` pós-WASM browser) podem usar API Cloud ZDR+DPA+SCC (Res. ANPD 29/2024); qualquer texto livre envolvendo cliente/operação vai **fail-closed** para Local/VPC Brasileira (open-weights 2026 Apache 2.0/MIT via `mistral.rs`/`llama.cpp`/`SGLang`/`vLLM`). **Guardrail de output + 3 campos de `Provenance` obrigatórios** + spot-lighting. Legado Go `backend/internal/insights/` é aposentado na Etapa 0.
3. **Extração norma→regra (`Norma→Regra` Two-Pass):** `Docling` (PDF) + `calamine` (XLSX, nunca OCR) → Pass 1 reasoning → Pass 2 critic & JSON portátil (`ThinkingPolicy::Disabled`, decimais/datas/CNPJs como `string`) → **compilação no engine + golden fixtures** → aprovação humana (`RuleSpec`).
4. **MCP server (`rmcp`, spec 2026-07-28) hardened:** JWT/OAuth2 + **binding credencial↔tenant**; **MCP sampling desabilitado**; **hash SHA-256 dos manifestos/descrições de tools pinado**; apenas tools *read-only* expostas por padrão (`validate_cadoc`, `query_rule`, `explain_violation`, `check_dossier`); geração/submissão STA exige HITL fora da banda da IA e **toda tool call entra na hash-chain** do ledger.
5. **RAG Dual-Store:** On-prem single-node (`Tantivy` BM25 + `BGE-M3` `ort` + `sqlite-vec`/HNSW) e Plataforma (`Tantivy` + `pgvector` HNSW + RRF inteiro + `BGE-Reranker-v2-M3`); golden set de 200 perguntas em CI; **corpus público** isolado de dados de tenant.

---

## §13 — Harness no motor (pontos de integração; detalhes em 22-HARNESS-V4.md)

- **Corpus seed:** os 1.178 testes Go do v1 → `eval_case` na F0, via **fixtures JSON do
  runner Go e a política de paridade do §1.8** (não é dump 1:1: stubs descartados,
  asserts de skip-silencioso reescritos, goldens de L4 sintéticos).
- **Gate de anonimização antes de tudo (LGPD):** nenhum evento de produção entra no
  corpus sem anonimização — pseudonimização determinística + tombstone que propaga
  exclusão (arts. 16/18); dado não-anonimizado é **vedado em fixture versionado no
  repo**; descobertas (`rule_candidate`) de um tenant não vazam no scorecard de outro
  (RLS nas tabelas do harness — §8). O flywheel transforma dado de titular em ativo de
  avaliação — sem esse gate, a exclusão LGPD fica impossível.
- **`harness-gate` no CI:** 3 tiers (PR/nightly/release); regressão de precision/recall
  das regras tocadas por um diff **bloqueia merge** (flywheel M6). O gate lê **snapshot
  exportado (artefato)** — nunca o Postgres de produção — e held-out usa **chave
  separada**; inclui eval adversarial de PII/injection (§12).
- **Catalog completeness gate × harness:** cada regra do catálogo 1.099 tem estado
  (`Implemented | DataOnly | Stub{motivo}`) — o agregado é o **Rule Coverage Index**
  publicado no health do produto, calculado pela máquina.
- **Miner:** corrections/discrepancies → clustering → `rule_candidate` com hipótese LLM
  (a única fonte de regras que "ninguém sabia que existiam").

---

## §14 — Observability, deploy, operação

| Dimensão | Escolha | Nota |
|---|---|---|
| Traces/metrics | `tracing` + OTel + Prometheus (`metrics`) + Sentry | setup com mais atrito que Go (research/04 §R9) — compensado por macro próprio `#[instrument_verdict]` |
| Deploy | distroless/**cc** (convive com xmllint), 20-40MB | Fargate 0.5vCPU/1GB → ~50-75% menos RAM que Go |
| Binário | 1 api + 1 worker + 1 cli (mesma workspace) | Empacotamento nativo CI: `ubuntu-latest` (Docker `debian-slim` + JRE ≥ 1.8) + `windows-latest` *(Plano v3.0 — sem `musl` cross de macOS; IA isolada atrás de `--features ai`)* |
| CI | GitHub Actions: fmt+clippy(-D warnings, -D clippy::wildcard_enum_match_arm, -D clippy::float_arithmetic, -D clippy::float_cmp, -D clippy::cast_precision_loss — §4.5) + test + coverage gate por crate + **harness-gate** + catalog gate + fuzz nightly do parser XML (cargo-fuzz, incl. entidades/DOCTYPE) | |
| SLOs | publicados **só após** produção real (lição v1: SLOs sem prod = ficção) | p95 validação < 500ms é alvo, não claim |
| Segurança | JWT RS256+CSRF (tenant só de claim verificada — ADR-0017), rate-limit Redis por tenant/rota, RLS, hash-chain audit segmentada, secrets (ADR-0015), LGPD data-map; IdP (OIDC + MFA p/ roles de envio STA) decidido na F3 com `/v1/auth/dev-token` desligado em prod (teste de CI que falha se o endpoint existir) | pentest do pentest-agente antes do launch |
| Quotas multi-tenant | admission control por tenant **antes de L1**: req/min por rota, jobs apalis concorrentes por tenant, bytes por documento; fila justa; caps idênticos em **todos** os conectores (DB row-cap, File size-cap, API byte-cap); cache de `disabled_rules` por tenant | o semáforo do xmllint e o rayon são globais — sem quota por tenant, DoS multi-tenant é barato |
| Retenção/LGPD (ADR-0016) | tabela de retenção por tabela/bucket com purge automatizado; CMK por tenant no S3; scrubbing nos SDKs Sentry/OTel; DPA+ZDR no data-map (frontier sem residência BR — research/07 §3.5); fluxo export/erase (art. 18) testado no CI | |

---

## §15 — O que herda, o que redesenha, o que morre (veredito do port)

**Herda 1:1 (~25%):** schema do banco (como **baseline nova consolidada** — §8), catálogo
de críticas (normalizado), leiautes/XSDs oficiais, contrato de API (v1.yaml corrigido),
algoritmo do hash-chain (**com segmentação/checkpoint** — §8), protocolo STA.
**Exceções explícitas:** as policies RLS **não** herdam sem o rename do GUC
(`app.if_id` → `app.tenant_id` — migração explícita que recria as policies, §8); as
migrations 001-026 entram como **artefato de paridade**, não como baseline (débito
SQLite: placeholders `?`, workarounds de datetime, `BEGIN IMMEDIATE`, `@postgres-only`).

**Porta como spec (~50%):** ~358 regras L2 reais (Go vira referência comportamental
validada pelos goldens), 10 generators, STA client, worker, radar, schema registry.

**Redesenha (~15%):** motor L1-L4 unificado (um registry por CADOC, dispatch exaustivo),
fim dos 2 motores paralelos e da colisão XD02, sem globals, conectores reais, OpenAPI
gerada do código, `rules-as-data` com gate de completude.

**Mora (~10%):** XSD validator caseiro (substituído por xmllint+codegen), engine
duplicado do generator/validation, XD mortos, 153 stubs silenciosos, seeds de sprint,
dual-driver SQLite, norm-audit/xsdgen de spike, `_gen_pdfs.py`, uma das duas SDKs por
linguagem (v1 tinha 2 gerações conflitantes colidindo em nomes de pacote).

**Não herdar os 6 fios desligados** (a lista que a auditoria pediu para não replicar):
1. dispatch silencioso de regras · 2. chaves de catálogo com sufixo (`2061-DLO`) ·
3. XSD paths relativos que degradam · 4. parse repetido por regra · 5. defaults
silenciosos (UF="SP", email fake) · 6. dois sources of truth de L1-L4.

---

## §16 — Sequência de construção (resumo; detalhe em 25-ROADMAP-EXECUCAO.md)

| Fase | Semanas | Conteúdo | Gate de saída (evidência externa) |
|---|---|---|---|
| F0 | 1-2 | repo novo, workspace, esquema do catálogo (dedup de chaves + ETL de estado v1 — §8), **política de paridade + runner Go de fixtures** (§1.8), golden corpus (1.178 testes Go → fixtures), CI, STATUS.md machine-generated | `cargo test` green + catalog gate rodando |
| F1 | 3-6 | `norma-domain` + codegen XSD→Rust dos leiautes **nov/2026** (IN 733 onda 3) + `norma-storage` RLS | tipos gerados compilam para os 13 CADOCs vigentes |
| F2 | 7-12 | pipeline L1-L4 unificada + generators 3040/3050 com paridade | **paridade por política do §1.8 (fixtures do runner Go 100% green) + XML Rust passa no BCValidador oficial** |
| F3 | 13-20 | generators restantes (gen3044 por evento, gen3060, gen3042 — §3.2) + API axum **com escopo cortado às rotas do wizard** (restante em F4) + worker + STA port (Basic preemptivo/senhaws/PSTA300 — §10; **spike A3 fora do caminho crítico**) + wizard fix + **ADR de IdP** (OIDC + MFA p/ roles de envio STA; lifecycle de sessão/refresh/revogação; `/v1/auth/dev-token` desligado em prod com teste de CI que falha se o endpoint existir) | **primeiro envio STA-h homolog real** |
| F4 | 21-28 | Radar + IA estágio 1 (router/sanitizador/copiloto/RAG) + harness f1-3 + conectores File/API + rotas API restantes + integração A3 | harness-gate bloqueando regressão; copiloto com citação em prod interna |
| F5 | 29-34 | launch comercial: pricing novo, SDKs regenerados, MCP, conectores DB/MCP, on-prem, SOC 2 readiness real | 1 contrato pagante assinado |
| F6 | contínuo | flywheel composto → Accuracy SLA | FN rate mensurável caindo; health scorecard público |

**Leiautes a gerar na F1 (lista da fusão, 16/11/2026):** além do codegen nov/2026
(IN 733 onda 3), a F1 gera e congela com provenance — **IN BCB 729/2026** (CNPJ
alfanumérico nos leiautes, abr/2026; novas inscrições RFB alfanuméricas desde jul/2026),
**IN BCB 735/2026** (data-base mai/2026 — Desenrola 2.0 no Anexo 37, domínio 13),
**IN BCB 733/2026** nas 3 datas-base mai/jul/nov-2026 (NR3 = Anexo 8, domínio 25
"Limite sujeito à provisão") e **IN 694/2025** (2030 DRSAC, dez/2026).

**Re-balanceamento honesto:** F0-F5 fecha em ~34 semanas (antes: 30). A F3 anterior
(6 semanas) empilhava o que a research/04 §R16 estima em 8-12 semanas (API 2-3 + jobs
1-2 + STA 2-3 + generators 3-4); com +2 semanas em F3 e o spike A3 fora do caminho
crítico, o pior caso (F0-F5 ~36 semanas) fica declarado em vez de implicitamente
estourado.

**Ponte DRSAC dez/2026 (nova redação pós-fusão 16/11/2026):** o piloto da janela
dez/2026-fev/2027 roda o **reg-cadoc 2030 em Rust (S4 da FASE 1)** contra o **leiaute
oficial IN 694/2025 re-baixado**, com chancela do **BacenValidatorAdapter (B02 —
ADR-0018)** e validação externa obrigatória (BCValidador oficial + CRD). O wiring do
gen2030 em Go foi **revogado** (retrabalho — gen2030 era fictício); o Go só sobrevive
como ponte STA-h para smoke. Critérios de bloqueio e switch para plano B em
`25-ROADMAP-EXECUCAO.md` §3.

---

## §17 — ADRs novas que este blueprint decide

| ADR | Decisão |
|---|---|
| **ADR-0009** | **Backend v2 em Rust** (axum + sqlx + Postgres único), substituindo ADR-0001 para o motor; frontend permanece Next.js/TS. Justificativa: correção compilada do domínio regulatório + operação 1-pessoa+agentes (não hiring). |
| **ADR-0010** | **Rules-as-data com catalog completeness gate**: regra semântica é artefato versionado com provenance e vigência; código nativo é exceção referenciada. |
| **ADR-0011** | **Uma pipeline de validação L1-L4** (fim dos motores paralelos); `&DocSet` emprestado, zero globals; L3 paralelizado. |
| **ADR-0012** | **XSD em 3 camadas** (codegen + inline + xmllint autoridade com erro explícito de indisponibilidade). **Nota da fusão:** xmllint é autoridade **estrutural**; o oráculo de **submissão** é o validador oficial (**ADR-0018**). O codegen `xsd-parser` aplica-se somente a CADOCs com XSD oficial disponível (3050 V4, 3045 etc.); para o **3040 o modelo vem do leiaute-XLS + validador oficial**, pois **não existe XSD oficial de 3040**. |
| **ADR-0013** | **Harness como subsistema de build/CI** (gates bloqueiam regressão de precisão; status do produto é machine-generated). |
| **ADR-0014** | **IA atrás do engine (`System 0 → System 1 → System 2`)**: determinismo/zero-float no core; `reg-s1-tabular` (`i64`) + `reg-ai` na borda + `reg-ai-runtime` (`--features ai`); barreira estrutural determinística LC 105 (zero texto livre com cliente na nuvem externa), defesa em profundidade `S1.2` fail-closed → on-prem (`AirGapped` / `HybridPolicyRouter` / `CloudZdrFirst`) e `Provenance` obrigatória (§12, `21-ENGINE-IA.md` v3.0 e `26-SYSTEM-ONE-DECISION-MODEL.md`). |
| **ADR-0015** | **Secrets e material de assinatura**: senhaws/chaves A1-A3 em Secrets Manager/Vault (nunca env/arquivo/tracing), `zeroize` em memória, rotação sob norma-audit, monitor de expiração no Calendário Normativo; fallback A3→A1 bloqueado por config por IF — nunca automático. |
| **ADR-0016** | **Retenção e transferência internacional**: prazos por tabela/bucket com purge automatizado, CMK por tenant no S3, scrubbing Sentry/OTel, DPA+ZDR para frontier, fluxo export/erase (art. 18) testado no CI; hash-chain segmentada com ancoragem externa. |
| **ADR-0017** | **Identidade de tenant**: `tenant_id` exclusivamente de claim verificada do JWT RS256 no middleware axum; header `X-IF-ID` (bridge de dev do v1) banido; teste de integração prova que forjar o header não cruza tenants. |
| **ADR-0018** | **Oráculo oficial de submissão (B02)**: o Validador oficial do BCB (`.jar` SCR3040) é a autoridade final de submissão para 3040/3042 (e 2030 quando houver validador oficial) — **nenhum ZIP sai para o STA sem chancela do `.jar`** (B02: *"Arquivo .ZIP deve ser gerado pelo aplicativo validador"*); o hash SHA-256 do artefato é calculado **exclusivamente sobre os bytes do `*_VALIDADO.zip`** (saída do validador, nunca sobre o ZIP pré-validação); integração via **adapter headless** do validador (JRE ≥ 1.8) com `--max-part-mb` conservador em **512–1024 MiB**. |

---

## Apêndice A — Crates e dependências (ordem de build)

```
norma-domain
 ├─ norma-layouts          (domain)
 ├─ norma-canonical        (domain, layouts)
 ├─ norma-rules            (domain)
 ├─ norma-validation       (domain, layouts, rules, canonical)
 ├─ norma-generator        (domain, layouts, canonical)
 ├─ norma-connectors       (domain, canonical, layouts)
 ├─ norma-sta              (domain)
 ├─ norma-radar            (domain, layouts)
 ├─ norma-ia               (domain, rules, validation)          ← declara ports de persistência (traits)
 ├─ norma-harness          (domain, rules, validation)          ← declara ports de persistência (traits)
 ├─ norma-audit            (domain)
 ├─ norma-storage          (domain, rules)                      ← adapters dos ports de harness/ia; sem ciclo
 ├─ norma-worker           (storage, sta, radar, harness, ia, validation)
 ├─ norma-api              (storage, validation, generator, connectors, ia, sta, audit, harness)
 ├─ norma-cli              (tudo)
 └─ norma-app              (api + worker + cli wiring)
```

O ciclo `norma-storage → harness/ia` do desenho anterior (storage dependia de
harness/ia, que precisam persistir os próprios artefatos) é quebrado hexagonal: os ports
são traits nos próprios crates de domínio, `norma-storage` os implementa como adapters,
e a injeção concreta acontece no composition root (`norma-app`/`norma-worker`).

## Apêndice B — Métricas do motor (o que o harness publica sobre o motor)

| Métrica | Fonte | Publicação |
|---|---|---|
| Rule Coverage Index (por CADOC, por data-base) | catalog completeness gate | `/console/health` + API |
| Precision/Recall por regra (vs oráculo BCValidador + auditoria amostral) | harness M5/M10 | scorecard mensal do tenant |
| FN rate (o erro que reprova o cliente) | harness held-out | Accuracy SLA |
| Time-to-Update (norma publicada → leiaute em sandbox) | radar | SLA vendido ("no dia da norma") |
| Mutation Detection Rate / judge TPR-TNR / canary 100% | Honesty Engine | health do próprio harness |
