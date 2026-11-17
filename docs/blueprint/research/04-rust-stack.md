# 04 — Blueprint Rust: Stack Completa para o Radiant Norma (nov/2026)

> Pesquisa honesta de stack Rust para SaaS regulatório (CADOCs BACEN: gerar → validar → transmitir).
> Contexto atual: Go 1.25, chi, pgx, Redis, Postgres 16 (RLS multi-tenant), Next.js 15, 29k LoC, time de 1 dev + agentes IA.
> Método: ~30 queries em web_search + verificação direta de docs.rs/crates.io (nov/2026). Cada decisão tem justificativa, alternativa e onde dói.

---

## 0. Veredicto executivo (leia primeiro)

**A stack Rust recomendada é sólida e madura para 15 das 16 dimensões do produto — exceto uma: validação XSD 1.0, que é o ponto único de risco real do projeto em Rust.** Não existe (em nov/2026) um validador XSD 1.0 completo, puro-Rust e maduro. A estratégia de mitigação existe (dual-validation: validador estrutural Rust inline + libxml2/xmllint como autoridade final), mas é engenharia extra que o Go também precisaria fazer — Go não tem essa vantagem aqui (Go também não tem validador XSD nativo; usa libxml2/xmllint igual).

**Honestidade sobre a migração:** para este workload (validação XML + geração + HTTP CRUD + Postgres), Go é suficiente tecnicamente. Os ganhos reais de Rust aqui são: (1) modelagem de tipos que torna regras CADOC impossíveis de violar em compile-time (enums exaustivos, newtypes, invalid states unrepresentable), (2) latência de cauda previsível, (3) binário único, (4) correção do validador via fuzzing de primeira classe. Os custos reais: velocidade de dev (~30–50% menor no primeiro ano), hiring no Brasil escasso, e ~4–7 meses de port até paridade com 1 dev + IA.

**Recomendação de arquitetura se migrar:** modular monolith em cargo workspace, regras regulatórias como **dados** (decision tables JSON / DSL declarativo) e não como código Rust, com o validador XML como o único módulo que justifica Rust profundo. Estratégia strangler: portar primeiro o core de validação/geração CADOC, manter bordas Go se necessário.

---

## 1. Stack recomendada (tabela de referência)

| Camada | Crate | Versão (nov/2026) | Papel |
|---|---|---|---|
| Runtime | `tokio` | 1.54.1 | Runtime async único do workspace |
| HTTP server | `axum` | 0.8.9 | API REST + SSE |
| HTTP middleware | `tower-http` | 0.7.1 | CORS, Trace, Compression, Timeout, Limit |
| Rate limit | `governor` + tower layer custom | 0.10+ | Rate limit *keyed* por tenant/API key |
| TLS | `rustls` | 0.23.46 | TLS server/client (sem OpenSSL) |
| Cert A1 | `rustls-pemfile` / `rustls` native | — | PKCS#12/PEM carregado direto |
| Cert A3 | `cryptoki` (PKCS#11) | 0.11+ | Assinatura via token/HSM ICP-Brasil |
| Erros (domínio/libs) | `thiserror` | 2.0.21 | Enums de erro tipados |
| Erros (bins/app) | `anyhow` | 1.0.104 | Propagação com contexto |
| DB | `sqlx` | 0.9.0 | Raw SQL compile-time checked + migrations |
| Vetores | `pgvector` (feature `sqlx`) | 0.4.2 | pgvector via sqlx/diesel |
| XML parse/write | `quick-xml` | 0.42.0 | Streaming + serde (de)serialização XML |
| XML DOM (leitura) | `roxmltree` | 0.21.2 | Navegação DOM imutável |
| XSD → código | `xsd-parser` | 1.8.0 | Gerar structs Rust a partir dos XSDs BACEN |
| XSD validação autoritativa | `libxml2` (via `xmllint` subprocess pool ou FFI `libxml`) | 2.14+ | **Único validador XSD 1.0 completo** |
| XSD validação inline | `xmlschema` (pure Rust) | 0.0.x ⚠️ | Rápida, pré-validação; **não é autoridade** |
| Excel leitura | `calamine` | 0.38.0 | XLSX/XLSB/XLS + ODS, streaming |
| Excel escrita | `rust_xlsxwriter` | 0.102.0 | Escrita + `constant_memory` |
| JSON | `serde` / `serde_json` | 1.0.x | Serialização padrão de facto |
| JSON Schema | `schemars` | 1.2.3 | Gerar JSON Schema dos tipos |
| JSON Schema validação | `jsonschema` | 0.61.2 | Draft 2020-12, alta perf |
| Money | `rust_decimal` | 1.43.0 | Decimal 96-bit, arredondamento configurável |
| Jobs/filas | `apalis` + `apalis-cron` | 1.2.0 / 0.4.x | Filas duráveis (Postgres) + cron |
| Retry/circuit breaker | `tower-resilience` | 0.4+ | Retry+backoff, circuit breaker, bulkhead |
| Redis | `fred` | 10.1.0 | Client async-first, cluster, pubsub |
| Tracing | `tracing` + `tracing-subscriber` | 0.1.44 | Structured logging/spans |
| OTel | `tracing-opentelemetry` + `opentelemetry` | 0.3x / 0.3x | Export OTLP |
| Métricas | `metrics` + `metrics-exporter-prometheus` | 0.25+ | Prometheus/OpenMetrics |
| Erros produção | `sentry` + `sentry-tracing` + `sentry-tower` | 0.49.6 | Crash/error reporting |
| Testes snapshot | `insta` | 1.49.0 | Golden files de XML/JSON gerados |
| Testes property | `proptest` | 1.12.0 | Property-based com shrinking |
| Testes integração | `testcontainers` + `testcontainers-modules` | 0.28.0 | Postgres/Redis reais em teste |
| Fuzzing | `cargo-fuzz` + `arbitrary` | — | **Fuzz do validador XML (obrigatório)** |
| Bench | `criterion` | 0.8.2 | Bench do pipeline de validação |
| HTTP client | `reqwest` | 0.13.x (rustls default) | Client para BACEN STA / LLM |
| MCP | `rmcp` (official) | 3.10.0 | SDK MCP Tier 1 do projeto MCP |
| Rules engine (dados) | `zen-engine` (GoRules) | 0.4x+ | Decision tables JSON — regras como dado |
| Deploy | `distroless/cc` (default) ou musl+scratch | — | Imagens 10–40 MB |

---

## 2. Decisões R1–R16

### R1 — HTTP server: **axum 0.8.9**

**Justificativa:** mantido pelo time Tokio (primeira parte), integração nativa com `tower`/`tower-http` (todo middleware que você precisa já existe), API sem macros com extractors, padrão de facto do ecossistema em 2025/2026 para novos projetos de produção. Série 0.8.x estável desde jan/2025 com patches contínuos (0.8.9 em set/2026). Middlewares recomendados: `CorsLayer` (strict em produção), `TraceLayer` com request-id, `TimeoutLayer`, `CompressionLayer`; rate limit global via `tower::limit`, **per-tenant** via `governor` em layer customizada (o `RateLimit` nativo do tower é global — para SaaS multi-tenant você precisa de quota keyed).

**SSE:** `axum::response::sse` com `mpsc` + `ReceiverStream`. **Gotcha crítico:** `with_graceful_shutdown` não encerra enquanto conexões SSE abertas existirem ([hyper#2787](https://github.com/hyperium/hyper/issues/2787)) — precisa fechar os senders dos canais no sinal de shutdown ou aplicar timeout layer dedicado ([axum#3196](https://github.com/tokio-rs/axum/discussions/3196)).

**Erros:** `thiserror` 2.0 nos crates de domínio (enums tipados por caso: `ValidationError { campo, regra, valor }`), `anyhow` 1.0 apenas nos binários/colagem. Eyre dispensável — só agrega valor com handlers customizados de report (color-eyre em CLI).

**TLS/ICP-Brasil:** `rustls` 0.23. **A1** (PKCS#12 arquivo): trivial, carrega chave e cert direto. **A3** (token/smartcard/HSM): o motor do token é PKCS#11 — usar `cryptoki` (wrapper idiomático do ecossistema rustls) ou `rustls-pkcs11`, implementando um `rustls::crypto::signers::SigningKey` custom que delega ao slot do token. Isso funciona, mas é a parte menos "pronta" do stack — budget 1–2 semanas de spike com SoftHSM antes do token real. Nota honesta: em Go você usaria `crypto/tls` + go-pkcs11 com a mesma quantidade de trabalho custom. O BACEN STA usa SSL client cert — este caminho é obrigatório, não opcional.

**Alternativa:** `actix-web` 4 ( TecEmpower leader, maturíssimo, próprio `Actor`-ish; desvantagem: ecossistema tower parcialmente duplicado). `poise` **não é** framework web — é framework de bots Discord sobre Serenity; se a intenção era "poem", ele é bom mas menor que axum.

**Fontes:** [Announcing axum 0.8](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) · [axum middleware docs](https://docs.rs/axum/latest/axum/middleware/index.html) · [cryptoki](https://docs.rs/cryptoki) · [rustls-pkcs11](https://lib.rs/crates/rustls-pkcs11) · [hyper#2787](https://github.com/hyperium/hyper/issues/2787)

---

### R2 — DB: **sqlx 0.9.0** (+ `pgvector` 0.4.2, migrations do próprio sqlx)

**Justificativa:** o modelo do produto é "SQL com RLS fazendo enforcement de segurança" — você **quer** escrever SQL explícito, não esconder atrás de ORM. sqlx dá queries compile-time verificadas (`query!` macro contra o Postgres real via `DATABASE_URL` em build), async-first, migrations versionadas (`sqlx migrate`), e pool com suporte direto ao padrão RLS.

**Padrão RLS multi-tenant (consolidado na indústria):** cada request abre transação e executa `SET LOCAL app.tenant_id = $1` **dentro** da transação (reset automático no commit — evita o leak entre conexões do pool documentado no [SO#80003232](https://stackoverflow.com/questions/80003232/postgres-row-level-security-leaks-across-tenants-when-using-a-pooled-connection)). Em sqlx: wrapper `with_tenant(tx, tenant_id, |tx| async { ... })` que centraliza o `SET LOCAL` — impossível esquecer em um único ponto auditável. Prepared statements: sqlx usa por padrão com cache por conexão; `SET LOCAL` não invalida prepared statements (é GUC de transação), então o padrão é seguro com pooling.

**pgvector:** crate oficial `pgvector` com feature `sqlx` — tipos `Vector`/`HalfVec` direto em queries. Para o RAG de normas/FAQ regulatória.

**Alternativas:** `diesel` 2.4 (maior type-safety de query, porém async só via `diesel-async` e o RLS com `SET LOCAL` fica menos ergonômico; benchmark da comunidade mostra Diesel ORM mode ~igual ou ligeiramente mais rápido que sqlx, mas o custo de aprendizado não se paga aqui); `sea-orm` 2.0 (ORM ActiveRecord async sobre sqlx — útil se o time quiser "ORM feel", boilerplate alto); `tokio-postgres` direto (menos bateria: sem migrations/pool de alto nível — sqlx usa ele como base conceitual mesmo). Escolher diesel/sea-orm só se o produto virar CRUD-heavy sem SQL analítico.

**Fontes:** [sqlx CHANGELOG](https://github.com/launchbadge/sqlx/blob/main/CHANGELOG.md) · [pgvector Rust docs](https://docs.rs/pgvector) · [Diesel vs sqlx oficial](https://diesel.rs/compare/compare_diesel) · [AWS: RLS multi-tenant](https://aws.amazon.com/blogs/database/multi-tenant-data-isolation-with-postgresql-row-level-security) · [Crunchy Data RLS](https://www.crunchydata.com/blog/row-level-security-for-tenants-in-postgres) · [RLS leak com pooling](https://stackoverflow.com/questions/80003232/postgres-row-level-security-leaks-across-tenants-when-using-a-pooled-connection)

---

### R3 — XML/XSD: ⚠️ **o risco nº 1 do projeto em Rust (decisão honesta e específica)**

**Estado do ecossistema em nov/2026 (verificado):**

| Opção | O que é | Maturidade | Servem para XSD 1.0 completo? |
|---|---|---|---|
| `quick-xml` 0.42 | Streaming parser/writer + serde | Muito madura, ~10x mais rápida que serde-xml-rs | **Não** — parse/write, não valida schema |
| `roxmltree` 0.21 | DOM imutável + XPath-lite | Madura | **Não** |
| `xsd-parser` 1.8.0 (Bergmann89) | **Gera código Rust a partir de XSD** (com quick-xml + serde) | Ativa em 2025/2026, releases mensais | Parcial: gera tipos, não valida runtime completo (UPA, co-constraints limitados) |
| `xmlschema` (Rust, sebastienrousseau) | Validador XSD puro-Rust, zero unsafe | **0.0.x — experimental**; faltam `xs:redefine`, substitution groups, mixed content, UPA completo | **Não como autoridade** |
| `libxml` crate (FFI libxml2) | Binding ao `xmlSchemaValidate` | Baixa atividade; docs avisam não-testado em multithread | **Sim** (libxml2 é completo), com caveat de threading |
| `xmllint` subprocess (libxml2) | CLI do libxml2 | Referência de mercado há 20 anos | **Sim** — ouro |
| Xerces-J sidecar | Java | Referência máxima XSD 1.0 | Sim, com custo operacional |

**Recomendação (arquitetura de validação em 3 camadas):**

1. **Geração de XML CADOC:** structs Rust geradas dos leiautes XSD do BCB via `xsd-parser` (os XSDs oficiais dos CADOCs 4010/4016/4030 etc. são publicados na [página de leiautes do BCB](https://www.bcb.gov.br/estabilidadefinanceira/leiautedocumentoscrd)) + `quick-xml` com serde para escaping/namespaces corretos. Newtypes de domínio (ex.: `Cnpj`, `Agencia`, `ValorDecimal(n)`) garantem formato na origem. Isso mata 90% dos erros de geração em compile-time.
2. **Validação inline (rápida, no hot path):** validador estrutural próprio orientado às regras do leiaute (campos obrigatórios, domínios, máscaras, somas COSIF) — isso você vai escrever de qualquer forma porque o diferencial do produto são as regras regulatórias que XSD não cobre. Roda na API, dá erro amigável por campo.
3. **Validação XSD 1.0 autoritativa (antes da transmissão STA):** `xmllint --schema` (libxml2) via **pool de subprocessos** gerenciado (ou FFI `libxml` se throughput exigir, com cuidado de threading). É o mesmo libxml2 que Go/Python/Node usam em produção há anos. Container distroless inclui `libxml2` como binário/so. Fallback extremo se um dia o leiaute usar constructos que libxml2 recuse: sidecar Xerces-J.

**Por que subprocess pool e não FFI por padrão:** isolamento contra OOM/loops (ex.: o advisory de runtime quadrático em parser XML, classe RUSTSEC-2026-0194), crash não derruba a API, e o custo de ~5–20ms por chamada é irrelevante no fluxo de transmissão (que acontece 1x por documento, não 1k por segundo). Fuzzing (`cargo-fuzz`) cobre a camada 2.

**Honestidade:** esta é a única área onde Go e Rust **empatam** — nenhum dos dois tem validador XSD nativo; ambos acabam no libxml2. Rust até tem a vantagem de geração de tipos (xsd-parser + newtypes) acima do que Go oferece (go-xsd é fraco). O risco não é "impossível", é "engenharia própria": estime 3–5 semanas do MVP de validação em camadas + fuzzing.

**Alternativa rejeitada:** WASM validator (xmlschema JS compilado) — camada extra de complexidade sem ganho sobre subprocess. Implementar subset XSD próprio — não; XSD 1.0 tem armadilhas (UPA, substitution groups) e o produto **é** compliance: validador caseiro que diverge do BACEN é passivo jurídico.

**Fontes:** [xsd-parser](https://github.com/Bergmann89/xsd-parser) · [xsd-parser v1.6 announcement](https://users.rust-lang.org/t/xsd-parser-v1-6-0-release-rust-code-generator-for-xml-schemas/143487) · [xmlschema (Rust)](https://crates.io/crates/xmlschema) + [docs.rs/xmlschema](https://docs.rs/xmlschema) · [libxml::schemas](https://docs.rs/libxml/latest/libxml/schemas/index.html) · [quick-xml](https://github.com/tafia/quick-xml) · [BCB leiautes](https://www.bcb.gov.br/estabilidadefinanceira/leiautedocumentoscrd) · [gap histórico XML em Rust](https://www.reddit.com/r/rust/comments/7kjzow/a_surprising_gap_xml_libraries)

---

### R4 — Excel/XLSX: **calamine 0.38 (leitura) + rust_xlsxwriter 0.102 (escrita)**

**Justificativa:** são os dois padrões do ecossistema (mesmo autor do quick-xml no caso do calamine). Leitura: suporta xlsx/xlsb/xls/ods, com `UnzipXMLSizeLimit` para capar memória em arquivos hostis (importante: planilha de cliente é entrada não-confiável). Escrita: API madura, fórmulas suportadas, e **`constant_memory`** para exportações grandes (flush sequencial por linha; limitação: escrever em ordem — aceitável para exports).

**Honestidade:** para este produto, Excel é input de carga de dados (importação de saldos) e output de relatórios. Ambos os crates cobrem. Se aparecer "editar planilha existente preservando tudo", nenhum dos dois serve bem (aí sim avaliar `umya-spreadsheet`, mais lento mas completa).

**Fontes:** [calamine](https://github.com/tafia/calamine) · [rust_xlsxwriter docs (constant_memory)](https://docs.rs/rust_xlsxwriter) · [benchmark calamine vs umya (2026)](https://umaranis.com/2026/05/04/reading-excel-files-in-rust-calamine-vs-umya-spreadsheet)

---

### R5 — JSON/Schema: **serde + serde_json + schemars 1.2 + jsonschema 0.61**

**Justificativa:** serde é o padrão de facto inquestionável (derive, performance, zero-copy parcial). `schemars` 1.x (estabilizado da 1.0 em diante) gera JSON Schema dos seus tipos de request/response — útil para o contrato da API e para gerar instruções para agentes IA. `jsonschema` (Stranger6667) valida Draft 2020-12 com performance de ponta (ganhos ~70% em evaluate recentes), compilando schemas para reuso.

**Uso no produto:** validar payloads de integração de clientes (JSON → CADOC), e validar output estruturado de LLM antes de entrar no pipeline (complementa o schema-typing do rmcp).

**Fontes:** [jsonschema docs](https://docs.rs/jsonschema) · [changelog jsonschema-rs](https://github.com/Stranger6667/jsonschema-rs/blob/master/CHANGELOG.md) · [schemars](https://docs.rs/schemars)

---

### R6 — Money/decimal: **rust_decimal 1.43**

**Justificativa:** decimal 96-bit fixo (até 28 dígitos fracionários), mapeia direto para `NUMERIC` do Postgres via sqlx/diesel, `serde` correto como string. Arredondamento: `RoundingStrategy` explícito — **o default do `round_dp` é banker's rounding (MidpointNearestEven)**; para CADOC, cada leiaute define casas decimais e a regra (em contabilidade COSIF o esperado costuma ser arredondamento comercial half-up — **confirmar por documento no manual do leiaute**, e codificar a estratégia por CADOC, não global).

**Pegadinha honesta:** rust_decimal não é precisão arbitrária (96-bit). Para totais agregados de balanço isso é irrelevante (valores até ~10^28), mas algumas somas intermediárias devem usar `Decimal` com `dp` máximo antes do arredondamento final do layout. Alternativa `bigdecimal` (precisão arbitrária) só se algum leiaute exigir mais de 28 dígitos — improvável. `fastnum` existe como alternativa mais rápida, mas menos ecossistema.

**Fontes:** [RoundingStrategy](https://docs.rs/rust_decimal/latest/rust_decimal/enum.RoundingStrategy.html) · [rust_decimal docs](https://docs.rs/rust_decimal) · [float gui sobre round_dp default](https://floating-point-gui.de/languages/rust)

---

### R7 — Concorrência/jobs: **apalis 1.2 + apalis-cron, storage Postgres; `tower-resilience` para retry/circuit breaker**

**Justificativa:** apalis v1.x (rede designed em 2025 na linha 0.7) é o framework de jobs dominante em Rust: workers com middleware layers, retry, dead-letter, storage **Postgres** (reusa o mesmo PG — uma infra a menos, e jobs transacionais com os dados), Redis/SQLite opcionais. `apalis-cron` para agendamentos (fechamento de período, lembretes de transmissão). Padrão tower: `tower-resilience` traz circuit breaker + retry com backoff exponencial/jitter + bulkhead + coalesce (singleflight) — bom para chamadas ao STA do BACEN e APIs de LLM. `backon`/`backoff` são alternativas menores para retry simples fora de tower.

**Honestidade:** nada equivalente a Sidekiq 100% compatível (protocolo) existe em Rust maduro — não precisa: o produto é verde-field neste lado. Se precisasse interoperar com workers Go/Redis existentes, aí usar `rusty-sidekiq` (compat de protocolo, nicho).

**Fontes:** [apalis.dev](https://apalis.dev) · [apalis v0.7 announcement](https://www.reddit.com/r/rust/comments/1jjebum/introducing_apalis_v07) · [apalis-cron](https://github.com/apalis-dev/apalis-cron) · [tower-resilience](https://docs.rs/tower-resilience) · [tower::retry::backoff](https://tower-rs.github.io/tower/tower/retry/backoff/index.html)

---

### R8 — Redis: **fred 10.1**

**Justificativa:** client async-first nativo (tokio), suporta Redis Cluster, pubsub, pipelines, 100% das commands, typed interface e builder de pool; releases ativos (10.1.0 out/2026). Para o uso do produto (cache de sessão, rate limit distribuído, locks de job), fred é o client mais completo. `redis-rs` 1.8 (client oficial, multi-runtime) é sólido e mais simples — escolha ele se o uso for só get/set/expire.

**Pegadinha honesta:** há relatos da comunidade de latência esporádica com pooling do fred em configurações específicas ([fred#261](https://github.com/aembke/fred.rs/discussions/261)) — para rate limiting distribuído de poucos comandos, redis-rs com `deadpool-redis` é igualmente adequado. Decisão default: fred; reavaliar se o p99 de Redis virar ruído.

**Fontes:** [fred docs](https://docs.rs/fred) · [redis-rs oficial](https://redis.io/docs/latest/develop/clients/rust) · [fred#261](https://github.com/aembke/fred.rs/discussions/261) · [dev.to: Meet Fred](https://dev.to/rosemediahouse/meet-fred-the-most-awesome-redis-client-for-rust-bl8)

---

### R9 — Observability: **tracing + tracing-opentelemetry + opentelemetry; metrics + metrics-exporter-prometheus; sentry 0.49**

**Justificativa:** `tracing` 0.1.44 é padrão absoluto (structured spans, compatível com todo o ecossistema tower/axum). A pilha OTel Rust (traces + metrics API **estáveis**) funciona em produção em 2025/2026 — padrão: `tracing-subscriber` com `EnvFilter` + camada `tracing-opentelemetry` + exporter OTLP para o collector. Frasseza honesta: mais atrito de setup/versões que Go/Java (a comunidade reclama de plug-and-play), e `tracing-opentelemetry` passou por transições de API — **pinar versões** no `[workspace.dependencies]`. Métricas: `metrics` (façade) + `metrics-exporter-prometheus` — ou, se Prometheus for o único backend para sempre, o crate `prometheus` direto (decisão: façade, porque OTLP metrics é caminho provável). Sentry: SDK oficial ativo (0.49.x), com `sentry-tracing` (spans→Sentry) e `sentry-tower` (transactions HTTP) — integração direta na stack axum.

**Fontes:** [OneUptime: Rust OTel (jan/2026)](https://oneuptime.com/blog/post/2026-01-07-rust-opentelemetry-instrumentation/view) · [Shuttle: OTel pipelines (set/2025)](https://www.shuttle.dev/blog/2025/09/23/monitor-data-pipelines-in-rust) · [Sentry Rust tracing docs](https://docs.sentry.io/platforms/rust/tracing) · [metrics-exporter-prometheus](https://crates.io/crates/metrics-exporter-prometheus) · [fricção da comunidade](https://www.reddit.com/r/rust/comments/1r4o7o9/struggling_to_set_up_opentelemetry_in_rust_any)

---

### R10 — Testing: **cargo test + insta + proptest + testcontainers + cargo-fuzz + criterion**

**Justificativa para um VALIDADOR:**

- **insta 1.49 (golden/snapshot):** XML CADOC gerado = golden files por layout e versão de leiaute; `cargo-insta` para revisar diffs quando o leiaute muda (mudanças regulatórias viram diff review, não reescrita de asserts).
- **proptest 1.12:** gerar documentos "quase válidos" (mutações em DOM válido) e assegurar: validador nunca panica, sempre reporta violações com path correto, arredondamentos mantêm invariantes de soma.
- **cargo-fuzz (libFuzzer):** **obrigatório no validador XML** — parsers são o alvo clássico de fuzzing; fuzz targets: `parse(bytes)`, `validate(bytes, schema)`, com corpus de CADOCs reais como seeds; detecta panics, OOM, timeouts e bugs de complexidade algorítmica (a classe do advisory RUSTSEC-2026-0194). Rodar em CI com budget fixo (ex.: 10 min/PR + job noturno longo).
- **testcontainers 0.28 + modules:** Postgres real com RLS policies aplicadas (testar que tenant A não lê tenant B é teste de integração, não unitário), Redis real, e o pool de xmllint real.
- **criterion 0.8:** bench do pipeline parse→validar→gerar; guard de regressão em PR críticos.

**Fontes:** [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html) · [structure-aware fuzzing + arbitrary](https://rust-fuzz.github.io/book/cargo-fuzz/structure-aware-fuzzing.html) · [appsec.guide cargo-fuzz](https://appsec.guide/docs/fuzzing/rust/cargo-fuzz) · [insta.rs](https://insta.rs) · [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs) · [Luca Palmieri: property testing](https://www.lpalmieri.com/posts/an-introduction-to-property-based-testing-in-rust)

---

### R11 — LLM/IA: **reqwest 0.13 + API externa; rmcp 3.10 para MCP; embeddings via serviço (TEI/vLLM), não Rust-native no MVP**

**Justificativa honesta:**

- **Cliente HTTP:** `reqwest` 0.13 (default rustls) — o client mais usado do Rust (~306M downloads na linha 0.12). Para LLM: chamar APIs (OpenAI/Anthropic/vLLM OpenAI-compatible) via reqwest + serde. Não existe necessidade de SDK dedicado.
- **MCP:** o SDK oficial Rust **`rmcp`** foi promovido a **Tier 1** no sistema de conformance do MCP (3.10.0, nov/2026) — expor o validador CADOC como tools MCP para agentes (o "harness" do próprio time) é first-class aqui. Isso é um **ponto forte real do Rust** no projeto: o produto IA-native com tools determinísticas.
- **Embeddings local:** `ort` (ONNX Runtime) é o consenso para inferência de embeddings quando local (é o que o Text Embeddings Inference da HF usa); `candle` (pure Rust) é mais fácil de shipar mas mais lento (caso Manticore: até 14x mais lento para encoders MiniLM/BGE). `burn` atrás dos dois em ecossistema de inferência prática.
- **vLLM como serviço externo vs Rust-native:** **vLLM (ou TEI para embeddings) como serviço** é a resposta pragmática — maturidade de serving (PagedAttention, batch contínuo) é insuperável; mistral.rs é interessante mas sem provas de throughput em escala. Rust-native só se: deploy edge/air-gapped ou custo de GPU ociosa — não é o caso de um SaaS regulatório.
- **RAG stack em Rust:** se RAG em-process for necessário: `swiftide` (pipeline de indexing/RAG streaming, integra Qdrant/pgvector, ativo em 2025) + pgvector. Alternativa: manter RAG no lado Node/Next.js onde o time já vive.

**Fontes:** [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk) · [MCP Tier 1](https://www.digitalapplied.com/blog/mcp-sdk-conformance-tiers-what-tier-1-means) · [ort (pyke)](https://github.com/pykeio/ort) · [Manticore: candle vs ONNX](https://www.reddit.com/r/rust/comments/1eol2nd/hugging_face_embedding_models_in_rust) · [swiftide](https://swiftide.rs) · [mistral.rs](https://github.com/ericlbuehler/mistral.rs) · [faun: mistral.rs vs vLLM](https://faun.dev/toolbox/mistralrs-vs-vllm)

---

### R12 — Workspace: **modular monolith em cargo workspace; hexagonal com traits; DI por composição explícita (sem framework)**

**Estrutura recomendada:**

```
radiant-rs/
  Cargo.toml            # [workspace.dependencies] — pin de versões único
  crates/
    domain/             # entidades, newtypes (Cnpj, Valor), enums CADOC, traits de port
    cadoc-layouts/      # tipos gerados por xsd-parser + builders por leiaute (4010, 4016...)
    rules/              # avaliador de decision tables (zen-engine) + regras COSIF
    validation/         # validador estrutural + cliente do pool xmllint
    storage-pg/         # adapters sqlx (RLS: with_tenant), repositories
    cache/              # adapter fred
    sta-client/         # transmissão BACEN (reqwest + rustls + cert A1/A3)
    llm/                # rmcp tools + clients LLM
    worker/             # apalis workers + cron
    app-api/            # axum: rotas, middlewares, main.rs (composição/DI)
  xtask/                # codegen (xsd-parser), golden file refresh
```

**Padrões (fonte: [matklad — Large Rust Workspaces](https://matklad.github.io/2021/08/22/large-rust-workspaces.html)):** `[workspace.dependencies]` para versão única; dep graph acíclico e em camadas (`domain` não depende de nada interno; adapters dependem de domain, nunca o contrário — ports são traits no domain, adapters implementam); um crate = um bounded context = boundary futuro de serviço.

**Hexagonal em Rust:** traits no `domain` (`trait CadocRepo`, `trait StaTransport`, `trait Validator`), structs genéricas sobre `R: CadocRepo` (não `dyn` no hot path; `Arc<dyn Trait>` só na borda da API). **DI sem framework:** a "injection" é literalmente construir os adapters em `main.rs` e passar para os construtores — idioma padrão Rust; frameworks de DI (shaku etc.) são rejeitados pela comunidade por boilerplate sem ganho. O generics monomorfizado dá dispatch estático de graça — DI Go não tem isso.

**Erros unificados:** cada crate define seus erros `thiserror`; o crate `app-api` converte para `ApiError` com `IntoResponse` (mapping central HTTP status/código de erro por variante — exaustivo por design, o compilador força tratar erro novo).

**Fontes:** [matklad](https://matklad.github.io/2021/08/22/large-rust-workspaces.html) · [40tude: monolith→distributed](https://www.40tude.fr/docs/06_programmation/rust/026_monolith_to_distributed/monolith_to_distributed.html) · [Tweag: workspace publishing (jul/2025)](https://tweag.io/blog/2025-07-10-cargo-package-workspace) · [Reddit: structuring monorepo 2025](https://www.reddit.com/r/rust/comments/1lra6h4/structuring_a_rust_mono_repo)

---

### R13 — Deploy: **distroless/cc como default; musl/scratch quando couber; Fargate/Lambda ambos fortes**

**Justificativa:** binário Rust estático ~10–30 MB. Recomendação pragmática: **distroless/cc** (tem CA certs, tzdata, glibc — o `xmllint`/libxml2 da camada de validação pode convivir como binário extra na mesma imagem) — evita as pegadinhas de musl (DNS, performance malloc — usar `mimalloc`/`jemalloc` se musl). Multi-stage build, `strip` + LTO thin no `[profile.release]`. Imagens finais típicas: distroless 20–40 MB (com xmllint), scratch+musl 5–15 MB.

**Lambda/Fargate:** cold start Rust entre os melhores (~8–20ms init nos benchmarks maxday lambda-perf; empate técnico com Go). **Atenção:** desde ago/2025 a AWS **cobra o INIT phase** — cold start virou custo direto; ambos os runtimes minimizam. Custos 10M invocações: Go e Rust se alternam por workload (Rust ganha em CPU-bound; Go pode ser levemente mais barato em memória mínima). Arm64/Graviton: Rust mostra 30% de redução de custo nos estudos de 2025. Para o produto (Fargate com PG vizinho), o ganho de memória do Rust (~1/2–1/4 da RAM do Go em serviço igual) reduz direto o custo de task.

**Cross-compile:** `cross` (containers) ou build em runner Linux nativo; `cargo zigbuild` para alvos musl aarch64 é o atalho 2025/2026 mais pragmático.

**Fontes:** [kerkour: rust docker alpine vs scratch](https://kerkour.com/rust-docker) · [maxday lambda-perf](https://maxday.github.io/lambda-perf) · [custo Lambda Go vs Rust 2025](https://medium.com/@monikasinghal713/go-vs-rust-on-aws-our-cost-benchmark-revealed-a-surprising-winner-6fae2a753dc0) · [TechRadar Arm64/2025](https://www.techradar.com/pro/arm64-dominates-aws-lambda-in-2025-rust-4-5x-faster-than-x86-costs-30-less-across-all-workloads) · [alpine vs distroless security](https://safeguard.sh/resources/blog/alpine-vs-distroless-which-base-image-is-more-secure)

---

### R14 — Performance honesta: **Go é suficiente para este workload; Rust ganha no que não é throughput**

**Dados (2025/2026):**
- Web service benchmarks realistas: Rust 1.5–2x o throughput de Go no mesmo hardware (ex.: ~2k RPS vs ~1k RPS em benchmark controlado); Anton Putra 2025 e Dmytro Misik mostram o padrão consistente.
- **Mas** o mesmo microservice benchmarked em M1 (Stackademic 2025/2026) conclui: "Go entrega performance próxima de Rust com overhead cognitivo menor".
- GC pauses do Go: o argumento Discord (2020) era p99 de segundos em heap gigante com GC forçado — irrelevante para este perfil (heap pequeno, objetos curtos). Go moderno (1.25, GC com dica de memória, goroutines baratas) tem p99 estável para CRUD+DB.
- **Este workload é I/O-bound** (Postgres, Redis, chamadas STA): o gargalo é DB e rede, não CPU. O tempo de CPU do parse de XML CADOC (~100KB–10MB) é <10ms em ambos. **Nenhum cliente vai perceber a diferença de throughput.**

**Onde Rust realmente ganha aqui (os argumentos honestos pró-migração):**
1. **Modelagem do domínio regulatório:** enums exaustivos + newtypes tornam "CADOC inválido" não-representável em compile-time (ex.: `enum Codigo4010 { ... }` com 500 variantes do leiaute — adicionar variante nova quebra todos os matchs não-exaustivos). Go (interfaces + strings) não tem isso.
2. **Tail latency previsível** (sem GC) para os jobs de fechamento e validação em lote.
3. **Memória:** ~50–75% menos RAM por instância → Fargate mais barato.
4. **Fuzzing + zero unsafe** no validador: argumento de segurança/compliance vendável ao cliente financeiro.
5. **Binário único + imagem mínima** → supply chain auditável (SBT/DevSecOps de cliente bancário).

**Onde Rust NÃO compensa (a verdade):** velocidade de iteração em features CRUD/admin (Go vence 30–50% no 1º ano), hiring (R15), prototipagem de regras (por isso regras como dados, R15), e o próprio XSD (empate técnico, R3). Se o produto fosse "SaaS CRUD comum", a migração seria destruição de valor. O Radiant Norma é um **compilador de dados regulatórios** — o centro do produto é transformação + validação com regras exaustivas, que é exatamente o sweet spot do type system de Rust.

**Fontes:** [Dmytro Misik: Go vs Rust web perf](https://medium.com/@dmytro.misik/go-vs-rust-web-service-performance-7fb10bbf9a9f) · [Stackademic M1 3-lang](https://blog.stackademic.com/go-vs-rust-vs-python-i-benchmarked-the-exact-same-microservice-in-all-three-2026-edition-on-m1-1218491e0f49) · [JetBrains: Rust vs Go (jun/2025)](https://blog.jetbrains.com/rust/2025/06/12/rust-vs-go) · [Discord: Go→Rust](https://discord.com/blog/why-discord-is-switching-from-go-to-rust) · [benchmarks vermel.app](https://programming-language-benchmarks.vercel.app/go-vs-rust)

---

### R15 — Hiring Rust Brasil 2026: **escasso e caro; mitigação = regras como dados, não como código**

**Realidade (dados de 2025/2026):** ~107 empresas no BR usando Rust (TheirStack); vagas locais específicas de Rust são raras (Glassdoor BR: ~56; a maioria das vagas "com Rust" em LinkedIn usa Rust como diferencial). O mercado remoto global puxa os bons devs (média US$ 85k/ano para BR em empresa gringa via plataformas; web3 até US$ 158–238k). Salários BR: CLT R$ 5.5k–40k, PJ até R$ 55k/mês. **Conclusão: manter um time de 3+ devs Rust CLT no BR é caro e lento; manter 1 dev sênior (o dono) + IA é viável.**

**Estratégia de mitigação (o design do produto compensa o hiring):**
1. **Regras como dados:** toda regra CADOC em decision tables JSON (GoRules **Zen Engine** — engine escrita em Rust, bindings multi-linguagem, grava/grava no editor visual Decision Grid). Dev pleno (Go/JS/Python) edita regras sem saber Rust.
2. **Layouts como dados:** XSD → codegen (`xsd-parser` via xtask); leiaute novo do BACEN = rodar codegen + preencher decision table, não escrever código.
3. **MCP tools + agentes IA:** o `rmcp` Tier 1 permite que agentes (Claude/etc.) operem o validador — o "time" de operação regulatória é IA sobre tools Rust.
4. **Frontier de skills:** só o núcleo (validation/, sta-client/, workspace glue) exige Rust idiomático; estimado <15k LoC. O resto é data.

**Nota sobre a variável "1 dev + IA":** o argumento de hiring muda de figura — não é mais "precisamos contratar devs Rust", é "o dev precisa revisar Rust gerado por IA". Isso é factível com 1 pessoa sênior, mas define o teto de throughput do time: a revisão humana é o gargalo (ver R16).

**Fontes:** [rustlang.com.br salários 2026](https://rustlang.com.br/blog/salario-rust-brasil-2026) · [vaganagringa.dev Rust](https://vaganagringa.dev/salarios/rust) · [GoRules Zen Engine](https://github.com/gorules/zen) · [TheirStack: empresas Rust BR](https://theirstack.com) · [TabNews discussão vagas Rust](https://www.tabnews.com.br)

---

### R16 — Custo da migração Go→Rust (29k LoC, 1 dev + agentes IA): **4–7 meses até paridade + 2–3 meses de estabilização; alternativa strangler reduz risco**

**Estimativa honesta (bottom-up):**

| Módulo | LoC Go (~) | Esforço Rust (1 dev + IA) | Observação |
|---|---|---|---|
| HTTP API/rotas/middleware (chi→axum) | 6k | 2–3 sem | IA portou bem; padrões distintos de DI |
| Acesso a dados (pgx→sqlx, RLS) | 5k | 2–3 sem | Reescrever queries com macros; testes de RLS reescrevem |
| Gerador XML CADOC (layouts) | 6k | 3–4 sem | **Melhor não portar: codegen via xsd-parser** (o Go atual era manual) |
| Regras de validação | 7k | **4–8 sem** | **Reescrever como decision tables JSON** (R15) — não como código |
| Jobs/cron/redis | 3k | 1–2 sem | apalis + fred diretos |
| Transmissão STA + cert ICP | 2k | 2–3 sem | rustls+PKCS#11 spike incluído |
| **Total port direto** | 29k | **~14–23 semanas** | ~3.5–6 meses calendário |
| Estabilização (fuzzing, golden files, gray release) | — | +6–10 semanas | sobreposição parcial com port |

**Fatores que IA muda e não muda:** agentes IA aceleram 2–3x o boilerplate (rotas, DTOs, adapters) e **não** aceleram o que exige design de ownership/lifetimes e decisões de modelagem de domínio — e pior: código Rust "que compila" gerado por IA costuma falhar em review de clippy/idiomaticidade e em edge cases de validação. Com 1 dev revisando tudo, o gargalo é a revisão humana, não a geração.

**Alternativa que muda o jogo — strangler:** portar o **core de validação/geração CADOC** primeiro como biblioteca/serviço Rust (~10k LoC equivalentes, 6–10 semanas) mantendo a API Go na frente via chamada gRPC/HTTP; migrar bordas só se/quando fizer sentido. Isso captura ~80% do valor (type-safety do domínio, validador fuzzado, memória) com ~30% do custo e rollback fácil. **Esta é a recomendação se a decisão ainda estiver aberta** (a tese "Rust como melhor solução do mercado" se sustenta no core, não nas bordas).

**Fontes:** [gaultier: lessons from successful Rust rewrite](https://gaultier.github.io/blog/lessons_learned_from_a_successful_rust_rewrite.html) · [Meteroid: 5 lessons SaaS Rust](https://dev.to/meteroid/5-lessons-learned-building-our-saas-with-rust-1doj) · [rustify.rs decision framework 2026](https://rustify.rs/articles/should-you-rewrite-it-in-rust-2026) · [crítica: rust rewrite waste](https://www.linkedin.com/posts/guykorland_unpopular-opinion-the-great-rust-rewrite-activity-7457100487876734976-dh6D)

---

## 3. Riscos consolidados

| # | Risco | Severidade | Mitigação |
|---|---|---|---|
| 1 | XSD 1.0 sem validador Rust maduro | **Alta** | Camada 3 (R3): xmllint autoritativo + validação estrutural própria + fuzzing |
| 2 | Leiautes BACEN mudam (COSIF 2025+) → codegen quebrando | Média | xtask de codegen + golden files insta + decision tables |
| 3 | rustls+PKCS#11 com token A3 real (drivers vendor) | Média | Spike de 1–2 sem com SoftHSM; plano B: serviço de assinatura isolado |
| 4 | Hiring Rust BR para escalar time | Média-alta | Regras/layout como dados; time core ≤3 |
| 5 | Velocidade de feature no 1º ano | Média | Strangler; aceitar -30% de velocidade temporário |
| 6 | OTel Rust: atrito de versões | Baixa | `[workspace.dependencies]` pinado; e2e de tracing no CI |
| 7 | fred pooling latency (raro) | Baixa | Monitorar p99 Redis; fallback redis-rs |
| 8 | Subprocess xmllint sob carga | Baixa | Pool pré-aquecido + timeout + FFI upgrade path |

---

## 4. Fontes (índice)

**Oficiais:** [tokio.rs axum 0.8](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) · [docs.rs axum](https://docs.rs/axum) · [sqlx CHANGELOG](https://github.com/launchbadge/sqlx/blob/main/CHANGELOG.md) · [diesel.rs compare](https://diesel.rs/compare/compare_diesel) · [SeaORM](https://www.sea-ql.org/SeaORM) · [pgvector-rs](https://docs.rs/pgvector) · [rustls](https://docs.rs/rustls) · [cryptoki](https://docs.rs/cryptoki) · [quick-xml](https://github.com/tafia/quick-xml) · [xsd-parser](https://github.com/Bergmann89/xsd-parser) · [xmlschema](https://crates.io/crates/xmlschema) · [calamine](https://github.com/tafia/calamine) · [rust_xlsxwriter](https://docs.rs/rust_xlsxwriter) · [schemars](https://docs.rs/schemars) · [jsonschema](https://docs.rs/jsonschema) · [rust_decimal](https://docs.rs/rust_decimal) · [apalis](https://apalis.dev) · [tower-resilience](https://docs.rs/tower-resilience) · [fred](https://docs.rs/fred) · [redis-rs](https://redis.io/docs/latest/develop/clients/rust) · [tracing](https://docs.rs/tracing) · [Sentry Rust](https://docs.sentry.io/platforms/rust) · [insta](https://insta.rs) · [proptest](https://docs.rs/proptest) · [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs) · [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html) · [criterion](https://docs.rs/criterion) · [reqwest](https://docs.rs/reqwest) · [rmcp oficial](https://github.com/modelcontextprotocol/rust-sdk) · [ort](https://github.com/pykeio/ort) · [candle](https://github.com/huggingface/candle) · [mistral.rs](https://github.com/ericlbuehler/mistral.rs) · [swiftide](https://swiftide.rs) · [Zen Engine](https://github.com/gorules/zen) · [matklad workspaces](https://matklad.github.io/2021/08/22/large-rust-workspaces.html) · [Rust releases](https://blog.rust-lang.org/releases/latest)

**BCB:** [Leiautes CADOC/STA](https://www.bcb.gov.br/estabilidadefinanceira/leiautedocumentoscrd) · [COSIF](https://www.bcb.gov.br/estabilidadefinanceira/cosif) · [Questor: arquivos BACEN XML](https://docs.questor.com.br/pt-br/Produtos/Gest%C3%A3oCont%C3%A1bilCloud/Contabilidade/arquivos-bacen)

**Benchmarks/engenharia:** [maxday lambda-perf](https://maxday.github.io/lambda-perf) · [Dmytro Misik Go vs Rust](https://medium.com/@dmytro.misik/go-vs-rust-web-service-performance-7fb10bbf9a9f) · [Stackademic M1](https://blog.stackademic.com/go-vs-rust-vs-python-i-benchmarked-the-exact-same-microservice-in-all-three-2026-edition-on-m1-1218491e0f49) · [JetBrains Rust vs Go](https://blog.jetbrains.com/rust/2025/06/12/rust-vs-go) · [Discord Go→Rust](https://discord.com/blog/why-discord-is-switching-from-go-to-rust) · [kerkour docker](https://kerkour.com/rust-docker) · [TechRadar Arm64](https://www.techradar.com/pro/arm64-dominates-aws-lambda-in-2025-rust-4-5x-faster-than-x86-costs-30-less-across-all-workloads)

**Hiring/mercado BR:** [rustlang.com.br 2026](https://rustlang.com.br/blog/salario-rust-brasil-2026) · [vaganagringa.dev](https://vaganagringa.dev/salarios/rust) · [TabNews/reddit r/brdev relatos]

**Rewrites:** [gaultier rewrite lessons](https://gaultier.github.io/blog/lessons_learned_from_a_successful_rust_rewrite.html) · [Meteroid SaaS lessons](https://dev.to/meteroid/5-lessons-learned-building-our-saas-with-rust-1doj) · [rustify.rs framework](https://rustify.rs/articles/should-you-rewrite-it-in-rust-2026)

---

*Relatório gerado em 16/nov/2026. Versões verificadas em docs.rs na data. Queries de busca: ~30.*
