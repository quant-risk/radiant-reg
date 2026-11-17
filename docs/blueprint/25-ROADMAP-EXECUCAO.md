# 25 — Roadmap de Execução da Reconstrução Rust (FASE 1: wedge "Radiant Reg" · FASE 2: plataforma Norma v2)

> **Data:** 16/nov/2026 · **Dono:** Henrique Costa · **Recurso:** 1 dev + agentes IA (ZCode) + orçamento de infra/IA.
> **Objeto:** plano de execução da reconstrução do Radiant Norma em Rust, conforme `20-ARQUITETURA-RUST.md` (pipeline única, rules-as-data com gate de completude) — **atualizado pela fusão de 16/11/2026: FASE 1 = wedge V1 "Radiant Reg" com 4 crates; FASE 2 = plataforma multi-tenant (§0 abaixo)**.
> **Fontes vinculadas:** `research/04` §R16 (custo da migração: 14–23 sem de port + 6–10 sem de estabilização para 1 dev + IA) · `research/01` (25% porta 1:1 / 50% spec comportamental / 15% redesenha / 10% morre; os 6 fios desligados) · `research/02` (milestones falsificáveis; done por evidência externa; sprints-fantasma) · `research/09` (harness v4 em 6 fases; Honesty Engine) · `research/23` (launch Q1/2027, janela DRSAC dez/2026–ago/2027, pricing de 5 tiers, kill criteria K1–K5) · `research/07` (rollout IA em 3 estágios; TCO $505/mês @10 IFs → $2.800/mês @100 IFs).
> **Lição que governa este documento:** o v1 queimou 4 quarters "no papel" em 11 dias (263 commits, 2026-07-03→13) e está congelado há 4 meses com **zero** evidência externa: 0 credencial Sisbacen, 0 smoke BACEN, 0 deploy, 0 cliente, 0 receita. Este roadmap não repete isso — **nenhuma fase fecha por declaração; só por evidência externa ou métrica machine-generated.**

---

## 0. A fusão aprovada (16/11/2026): FASE 1 = wedge V1 "Radiant Reg" (S1–S6) · FASE 2 = plataforma multi-tenant

**Decisão do dono (Henrique Costa, 16/11/2026), fundindo o segundo blueprint (Gemini):** a execução passa a ter **duas fases explícitas** — o wedge entra em receita antes da plataforma.

### FASE 1 — wedge V1 "Radiant Reg" (S1–S6, até 31/12/2026)

On-prem, 4 crates (`reg-core`, `reg-cadoc`, `reg-ledger`, `reg-cli` — 20-ARQUITETURA §3.1), SQLite WAL portável, régua de preço em `23-PRODUTO-PRICING-GTM.md` §2.0. Sequência:

| Sprint | Conteúdo | Gate de saída |
|---|---|---|
| **S1 — saneamento (F0′)** | rotacionar/revogar **`backend/dev-private.pem`** (chave commitada que casa com `frontend/.env.pubkey`); `git rm` dos **6 binários Mach-O ~110 MB**; regenerar os **3 PDFs-404 de 6.245 bytes** + `Carta_Circular_3588_STA.pdf` de 725 B + **5 shells Angular de 2.871 B**; corrigir o Dockerfile; **re-download oficial 2030/3040** pós-IN 694/2025 + IN 729/2026 **com SHA-256**; **ponte STA-h em Go para smoke**; bootstrap `rust/` com os 4 crates; **pedido formal PSTA300 protocolado** | saneamento completo em `main`; protocolo PSTA300 registrado |
| **S2–S3 — reg-core/reg-cadoc** | `CnpjAlfa` temporal (IN BCB 729/2026), ingestão anti-f64 (Centavos/TaxaDecimal — 20-ARQUITETURA §4.5), `mapping.lock` com **4-olhos**, dedup blake3 P=256, `BacenValidatorAdapter` B02 (ADR-0018) | tipos compilam; mapping.lock assinado |
| **S4 — 2030 DRSAC Rust** | gen2030 em Rust + cruzamento 2030↔3040 | XML canônico contra o leiaute oficial |
| **S5 — Dossiê RC 18 + STA Rust** | Dossiê A4/JSON assinado Ed25519 (RC Conjunta 18/2025) + STA client Rust (Basic preemptivo/senhaws/PSTA300 — 20-ARQUITETURA §10) | dossiê assinado valida; smoke STA-h |
| **S6 — build air-gapped + go-live** | pacote on-prem instalável nos pilotos | **2–4 contratos até 31/12/2026 (marco N0)** |

### Nota de decisão sobre o baseline (opção A aprovada)

O run E2E de **41,76%** fica **congelado como baseline documentado** (`e2e-audit/20260715-170152-3a51cba`, branch `remediation/gates-1-14`) — não se "conserta o número" no legado; o runner é **reconstruído em S6** como corpus do harness (22-HARNESS §5.6). **Força-push/filter-repo dos remotes fica para a migração do repo da FASE 2** — a rotação de chave é imediata (S1).

### Nota de sequenciamento

A **plataforma multi-tenant (ex-F0–F5 de 17 crates) passa a ser FASE 2, pós-primeiro contrato**, com workspace consolidado em **~8–10 crates de domínio** (+ wiring) — consolidar o que não tem requisito real de cliente; o desenho de 17 crates fica como teto de referência (20-ARQUITETURA §3.1). **Os marcos N1–N6 continuam válidos para a FASE 2** (§4); **N0 (primeiro contrato pago) precede N1** e é o gate de saída da FASE 1.

---

## 1. Estratégia: clean rewrite com golden tests — decidido

**Decisão: clean rewrite (repo novo, workspace Cargo do zero), não strangler.**

Justificativa verificada contra a realidade de 16/nov/2026:

1. **Strangler pressupõe tráfego vivo para redirecionar incrementalmente — e não existe.** Zero clientes em produção, zero deploy, repo congelado desde 13/jul/2026. Não há borda Go "na frente" recebendo requests para ir fatiando por trás. O padrão strangler (research/04 §R16, "portar o core primeiro mantendo a API Go na frente via gRPC/HTTP") era a recomendação *para o caso de migração com sistema rodando*. Nosso caso é de sistema morto: não há nada para estrangular.
2. **O repo congelado vira *spec executável*, não base de port mecânico.** A auditoria (research/01 §6) classificou o destino do Go:
   - ~25% aproveita 1:1 — migrations SQL 001–026, schema do banco, RLS, catálogos JSON, protocolo STA WS, algoritmo do hash-chain, contrato de API.
   - ~50% porta como referência comportamental — as ~358 regras L2 reais, 10 generators, worker, radar, schema registry: o Go vira *spec*, validado pelos 1.178 testes convertidos em golden tests.
   - ~15% redesenha — motor L1-L4 unificado (fim dos 2 motores paralelos), dispatch exaustivo, remoção de globals, chi→axum.
   - ~10% morre — XSD validator caseiro com paths relativos quebrados, motor `generator/validation` duplicado, XD01–XD08 nunca registrados, 153 stubs severity-I, dual-driver SQLite, seeds de sprint.
   Um port mecânico transportaria os **6 fios desligados** que a auditoria mandou não replicar: (1) dispatch silencioso de regras (`return nil` para o desconhecido); (2) chaves de catálogo com sufixo (`2061-DLO`) inalcançáveis pela API de 4 dígitos — 568+ críticas mortas em runtime; (3) XSD com paths relativos ao disco que degrada silenciosamente para root-tag check em deploy; (4) parse repetido por regra (DLI parseia o documento por chamada); (5) defaults silenciosos no generator (`UF="SP"`, email fake); (6) dois sources of truth de L1-L4 com colisão de código XD02.
3. **O seguro que o strangler daria vem dos golden tests.** O risco clássico do clean rewrite é regressão comportamental invisível. Aqui existem dois antídotos que o v1 deixa pronto: os 1.178 testes Go viram corpus de paridade (F0 extrai, F2 exige 100% green) e o **BCValidador oficial do BCB é o oráculo externo** disponível para todo artefato gerado. Paridade por evidência supera convivência de dois motores — que é justamente o defeito estrutural do v1.
4. **A ponte DRSAC não é strangler — é aplicação paralela.** O Go v1 serve o piloto dez/2026–fev/2027 como produto legado congelado com um patch cirúrgico (§3), enquanto o Rust nasce ao lado. Nenhuma feature nova entra no Go. O v1 não "migra": ele se aposenta com dignidade quando o último cliente sair. *(Revisto pela fusão 16/11/2026: o piloto DRSAC roda sobre o gen2030 em Rust — S4 da FASE 1, §0; o Go só sobrevive como ponte STA-h para smoke, S1 — ver nota no topo do §3.)*

**Comparação honesta das opções (para registro):**

| Critério | Clean rewrite (escolhido) | Strangler | Port 1:1 completo |
|---|---|---|---|
| Pré-condição | — (não exige tráfego vivo) | Tráfego de produção roteável | — |
| Risco principal | Regressão comportamental | Complexidade de roteamento dual | Transportar os 6 fios desligados |
| Seguro disponível | 1.178 goldens + BCValidador | Rollback trivial por borda | Goldens (parciais — os bugs são do próprio código portado) |
| Custo estimado (research/04) | 14–23 sem de port + estabilização | ~30% do custo, mas exige sistema vivo | Mais alto (portar inclusive o que morre) |
| Fit com a realidade de hoje | **Total** | Zero (não há tráfego) | Zero (o que morre não tem por que ser portado) |

**Strangler condicional (o único gatilho de reversão):** se um cliente em produção do v1 aparecer antes do launch Rust — por exemplo, uma IF do piloto DRSAC que queira continuar no v1 após fev/2027 — o v1 entra em modo "legado com SLA mínimo de manutenção": fix de bug crítico apenas, zero feature nova, até a migração do cliente para o Rust. Isso é strangler de facto, mas é plano de contenção, não estratégia. A checagem é mensal: "existe cliente rodando no v1?" Se sim por mais de 2 meses sem plano de migração, formaliza-se um mini-ADR de contenção.

---

## 2. FASE 2 — plataforma multi-tenant: fases F0–F6 (cronograma re-balanceado)

> **Escopo pós-fusão (16/11/2026):** todo o cronograma F0–F6 abaixo compõe a **FASE 2** — pós-primeiro contrato (marco N0, §0). As datas de 2026–2027 abaixo são o plano de referência da plataforma, **não o relógio da FASE 1** (S1–S6, §0); o workspace da FASE 2 consolida em ~8–10 crates de domínio (20-ARQUITETURA §3.1).

### 2.0 O re-balanceamento explícito da F3 (decisão registrada)

O crítico de arquitetura apontou corretamente: a F3 original alocava 6 semanas para o que `research/04` §R16 estima em **8–12 semanas** (API chi→axum 2–3 sem + STA/cert 2–3 sem + jobs 1–2 sem + generators restantes 3–4 sem — e a F3 original carregava tudo isso mais o fix do wizard). O re-balanceamento adota as três alavancas de uma vez, não escolhe uma:

| Alavanca | Decisão concreta |
|---|---|
| **Extensão** | F3 passa de 6 → **7 semanas** (sem 13–19). F4 passa de 6 → **7 semanas** (sem 20–26). Nominal F0–F5: 30 → **32 semanas**. |
| **Corte de escopo** | Saiem da F3: fix do wizard frontend (→ F4; é TypeScript, não bloqueia o motor) e 4 dos 6 generators restantes (2060/2070/2160/2170 → F4, via macro `cadoc_generator!` que já compartilha ~70% da mecânica). F3 core: API + auth + worker + STA(A1) + 2 generators (2061/2062). |
| **Spike fora do caminho crítico** | O spike de **A3/PKCS#11** (`cryptoki` + SoftHSM, 1–2 sem — research/04 §R1) roda em paralelo via agente ZCode em background durante a F3. A F3 fecha com **A1** (PEM, trivial). A3 entra por feature flag na F4, somente se o spike provar o caminho com SoftHSM antes do token real. Nunca no caminho crítico. |

**Compromisso de datas:** o calendário público de marcos (§4) usa o cenário com buffer: 32 semanas nominais + **4 semanas de buffer explícito (sem 33–36)** = **até 36 semanas** para F0–F5. Se o buffer for consumido, corta-se escopo da F4 nesta ordem fixa: (1) Radar em modo leitor-only sem auto-PR; (2) RAG híbrido reduzido a BM25-only (sem pgvector); (3) conectores DB/MCP para pós-launch. O que nunca se corta: paridade (N1), BCValidador (N2), envio real (N3) — são os três marcos que provam que o produto existe.

### 2.1 F0 — Fundação e verdade inicial (sem 1–2 · 16–29/nov/2026)

**Entregáveis:**

- Repo novo com cargo workspace: 16 crates stub com bordas do Apêndice A do doc 20 (`domain` não importa nada interno; `api/worker/cli` importam tudo; ninguém importa `api`).
- CI GitHub Actions desde o commit 1: fmt + clippy `-D warnings` + linter anti-wildcard (bloqueia `_ =>` no dispatch de CADOC) + test + **catalog gate** + drift check de leiautes.
- **Catálogo normalizado:** ETL de `criticas.json` para schema único — dedup dos 11 códigos duplicados, chaves sufixadas (`2061-DLO`, `2060-DRM`, `2070-DDR`) normalizadas para código de 4 dígitos + categoria explícita, vigência por regra, `status` declarado por regra.
- **Golden corpus:** script `xtask` extrai os 1.178 testes Go para fixtures input/expected versionadas com provenance `golden`. Testes dependentes de globals/ordem viram `flaky-spec` e são derivados manualmente.
- **STATUS.md machine-generated:** números de cobertura/regras/testes emitidos por script a partir do build — fim da classe de contradições C3/C4 do research/02 (4 números de cobertura 3040 simultâneos no mesmo README).
- ADRs 0009–0014 assinadas; tracker dos 6 fios desligados aberto como gates de não-regressão arquitetural.

**Riscos da fase:** a extração dos goldens pode revelar que boa parte dos testes Go testa wiring morto (ex.: os testes do `Builtin3050` testam regras que nunca rodam) → cada golden é etiquetado com o status de produção do que testa; isso vira o mapa honesto de priorização da F2. O catálogo pode revelar mais duplicatas além das 11 conhecidas → dedup documentado com diff revisado em PR.

**Alocação semanal:**

| Semana | Foco |
|---|---|
| 1 | Repo + workspace + CI com os 4 gates; ADRs assinadas; ETL do catálogo iniciado; pedido de credencial protocolado; wiring drsac no v1 *(revogado pela fusão 16/11/2026 — §0: o gen2030 em Go saiu do escopo; critério 30/nov é o Bloqueio A.2 em nova redação)* |
| 2 | Golden corpus completo (1.178 → fixtures); STATUS.md machine-generated; tracker dos 6 fios; leiautes nov/2026 baixados e taggeados |

**Done (evidência externa):** `cargo test` green no workspace com os gates rodando em PR real; contagem do catálogo **publicada pelo build** (nunca digitada à mão); N goldens extraídos com N medido por script; ao menos um PR real bloqueado pelo catalog gate (prova de que o gate funciona, não de que existe).

### 2.2 F1 — Domínio, leiautes nov/2026 e storage (sem 3–6 · 30/nov–27/dez/2026)

**Entregáveis:**

- `norma-domain`: enum `CadocCode` exaustivo dos 13 CADOCs vigentes; newtypes `CnpjIdentifier` (por data-base — IN BCB 729/2026, 20-ARQUITETURA §4.3), `Money` (rust_decimal), `DataCompetencia`, `RuleId`; `Vigencia` como cidadão de primeira classe; **arredondamento declarado por leiaute** (`Rounding::{HalfUp, HalfEven, Truncate}`) — nunca banker's implícito do rust_decimal.
- **Codegen XSD→Rust dos leiautes NOV/2026** (IN BCB 733 onda 3 — vigente *agora*) via `xsd-parser` em `xtask`; leiautes XLS via calamine → modelos de campos versionados por vigência.
- `norma-storage`: sqlx 0.9; migrations 001–026 portadas e limpas (sem seeds de sprint, sem o modo ENABLE RLS — herda direto o FORCE); RLS com `set_config('app.tenant_id', …, true)` (SET LOCAL à transação, padrão centralizado `with_tenant` — impossível esquecer em um único ponto auditável).
- **Catalog completeness gate** no build: todo `RuleSpec` passa `jsonschema`; toda `NativeRuleRef` existe; toda regra tem `Implemented | DataOnly | Stub{motivo}` — regra sem estado explícito **quebra o build**. Mata os 153 stubs silenciosos do v1.

**Riscos da fase:** `xsd-parser` pode não suportar algum constructo dos XSD do BCB (choice profundo, substitution groups) → spike na semana 3; se travar, geração manual dos 3 leiautes críticos (3040/3050/2030) com os demais via macro. O leiaute nov/2026 pode ter correção pós-publicação (o Radar ainda não existe) → diff manual contra jul/2026 antes de congelar goldens. Postgres é pré-requisito do sqlx `query!` — o ambiente dev sobe na semana 3 (ação 8 das 72h).

**Alocação semanal:**

| Semana | Foco |
|---|---|
| 3 | `norma-domain` completo (enum + newtypes + Vigencia); spike `xsd-parser` contra os XSD reais do BCB |
| 4 | Codegen `xtask` rodando para os 13 leiautes; fallback manual decidido (ou não) com evidência; `norma-storage` com migrations portadas |
| 5 | RLS com testcontainers (teste cross-tenant); catalog completeness gate implementado e demonstrado quebrando build |
| 6 | Roundtrip XML↔tipos por leiaute; goldens de leiaute (insta) congelados para nov/2026; review de fase e decisão go/no-go do codegen |

**Done (evidência externa):** tipos gerados compilam para todos os CADOCs vigentes (build machine-checked); suite RLS com testcontainers prova que tenant A não lê tenant B (teste de integração real, não unitário); catalog gate demonstradamente quebra o build em PR com regra sem estado; XML de exemplo dos leiautes nov/2026 parseia nos tipos gerados com roundtrip.

### 2.3 F2 — Pipeline unificada + 3040/3050 com paridade (sem 7–12 · 28/dez/2026–07/fev/2027)

**Entregáveis:**

- `norma-validation`: a **única** pipeline L1→L4 do sistema. L1 = tipos gerados + validação estrutural inline + `xmllint` pool como autoridade final com erro explícito `L1AuthorityUnavailable` (nunca degrada para root-tag check como no v1). L2 = rules-as-data com dispatch `match` sem wildcard (clippy bloqueia). L3 = `&DocSet` por parâmetro, sem globals, rayon por par de CADOC, IDs namespaceados (`XD-3040-3050-01` — fim da colisão XD02). L4 = diff determinístico por tenant com tolerâncias tipadas.
- `norma-generator` para **3040 + 3050**: macro `cadoc_generator!` + traits de blocos (Envelope, Zipper, Hasher); defaults proibidos — campo ausente obrigatório é `GenError::MissingField { path, provenance }`.
- **Suite de paridade golden 100% green** (corpus da F0, subset 3040/3050); fuzzing `cargo-fuzz` do parser com budget em CI (10 min/PR + job noturno).

**Riscos da fase (a mais densa do plano):** portar as ~120–130 regras 3040 reais do v1 é arqueologia — strconv sobre strings, 153 stubs a descartar, comentários de cobertura contraditórios → RuleSpec derivado em batches com golden a cada batch; o `Builtin3050` (170 regras órfãs) é spec comportamental, não wiring a replicar. L4 do v1 tem 0% de cobertura → corpus L4 construído sinteticamente (mesma entrada em duas competências). Arredondamento rust_decimal pode divergir do Go em casos de borda — os goldens existem exatamente para isso; divergência real vira decisão documentada por leiaute, não silêncio.

**Alocação semanal:**

| Semana | Foco |
|---|---|
| 7 | `norma-validation` esqueleto: pipeline única L1-L4 com `&DocSet`; L1 com xmllint pool e erro explícito |
| 8 | L2 rules-as-data: carregador + dispatch exaustivo; primeiras ~40 regras 3040 com golden cada |
| 9 | Regras 3040 em batches (~40-50/semana via IA + revisão); generator 3040 com macro comum |
| 10 | Generator 3050 (reaproveitando ~70% da mecânica); L3 cross-doc com rayon; L4 sintético |
| 11 | Paridade: rodar corpus completo; matar diffs; fuzz do parser em CI |
| 12 | Estabilização; BCValidador oficial rodado no XML Rust (N2); review de fase; decisão do corte de escopo se N1 estiver em risco |

**Done (evidência externa):** **N1** — paridade golden 100% green, contada pelo harness (número machine-generated, nunca claim); **N2** — XML gerado pelo Rust **passa no BCValidador oficial do BCB**, com log/print do validador anexado à issue do marco.

### 2.4 F3 — API, worker, STA e primeiro envio real (sem 13–19 · 08/fev–28/mar/2027) — *7 semanas, re-balanceada*

**Entregáveis (core, caminho crítico):**

- `norma-api` em axum: OpenAPI 3.1 via `utoipa` **gerada do código** (fim do drift spec vs rotas do v1: 46 paths vs ~70 rotas); envelope de erro único `{"error": {code, message, provenance?, trace_id}}` (padrão `SafeError` mantido); SSE com shutdown-aware timeout (gotcha hyper#2787); Idempotency-Key obrigatória em POST de geração/envio.
- Auth: JWT com keyring/kid + CSRF + rate-limit per-tenant via `governor` (o rate limit do tower é global — multi-tenant exige quota keyed).
- `norma-worker`: apalis 1.2 com storage Postgres; claim atômico (`FOR UPDATE SKIP LOCKED` — melhora sobre o v1); backoff exponencial + DLQ; cron de fechamento de período.
- **STA client port** (o tesouro do v1): WSClient 1.357 LoC → `reqwest` + `rustls`; fluxo oficial BACEN (POST /arquivos → PUT conteúdo, posicaoupload, verificação `X-Content-Hash`, chunked range, Basic auth preemptivo); cert **A1**; senhaws rotation com lock distribuído Redis.
- Generators **2061 (DLO) e 2062 (DLI)** — os dois seguintes na ordem de valor (limites são dor recorrente de IP/SCD).
- **Fora do caminho crítico:** spike A3/PKCS#11 com SoftHSM (agente em background). **Cortado da F3:** wizard fix e generators 2060/2070/2160/2170 (→ F4).

**Riscos da fase:** sem credencial Sisbacen não há envio — o pedido abre na semana 1 (§8) e o follow-up é semanal; o fluxo de homologação pode exigir documentação não-prevista → janela de slip absorvida pelo buffer; TLS client com cert A1 é a parte "pronta" do stack, mas o ambiente STA-h pode ter idiossincrasias de handshake → smoke com `StubClient` primeiro, ambiente real depois.

**Alocação semanal:**

| Semana | Foco |
|---|---|
| 13 | API axum: rotas essenciais + utoipa + envelope de erro; auth JWT+CSRF |
| 14 | Rate-limit per-tenant; SSE; worker apalis com claim atômico + DLQ |
| 15 | STA port: WSClient (fluxo de arquivo); smoke contra StubClient; spike A3 roda em background |
| 16 | STA port: senhaws rotation + retry/circuit breaker; generators 2061 |
| 17 | Generator 2062 (DLI); integração api↔worker↔sta ponta-a-ponta em staging |
| 18 | Smoke com cert A1 no ambiente STA-h de homologação (se credencial ok); correções de handshake |
| 19 | **N3**: primeiro envio homolog real capturado; generators 2070/2160 se sobrou folga; review de fase + launch L1 (Free + Starter) |

**Done (evidência externa):** **N3** — primeiro envio **STA-h homolog real** com protocolo de transmissão/aceite capturado como artefato (hash pré-envio, payload, resposta, timestamps — que já alimenta `norma-audit` e cria o rastro do flywheel M1). Sem credencial: N3 não fecha; o marco degradado **N3'** (remessa aceita no CRD de uma IF real, envio manual) substitui temporariamente com data renegociada explicitamente.

### 2.5 F4 — Radar, IA estágio 1, harness e conectores (sem 20–26 · 29/mar–16/mai/2027) — *7 semanas*

**Entregáveis:**

- `norma-radar`: crawler diário (leiautedocumentoscrd, INs, CCs, FAQ), hash por §, **diff semântico de leiautes** nov/dez/2026 (calamine diff XLS + XSD), classificação breaking/minor/new + impacto por CADOC, **auto-PR** no repo de catálogo com aprovação humana, calendário normativo como fonte de verdade temporal. Rastreando desde o dia 1: GRSAC 1º/1/2027 (Res 586 + IN 772), LCR/LCRS 90→100% (Res CMN 5.296 + Res BCB 560), FRTB SENS 2027.
- **Engine IA estágio 1** (research/07, estágio 1 do rollout): router multi-modelo (frontier por função; troca de modelo em 1 dia) + **sanitizador PII obrigatório** antes de qualquer chamada frontier (nenhum provider tem residência LatAm — sanitização é obrigatória, não opcional) + copiloto de validação que humaniza críticas campo-a-campo **com citação da Provenance** + RAG híbrido (pgvector HNSW + Tantivy BM25 + fusão RRF + rerank).
- **Harness fases 1–3** (research/09): gates M6 nos 3 tiers de CI (PR/nightly/release — regressão de precision/recall das regras tocadas por um diff **bloqueia merge**); M1 discrepância→case; M2 correção→case com snapshot automático no UX approve/correct; miner M7 semeado com triagem semanal.
- Conectores **File (calamine real, mapeamento de colunas com perfil por CADOC) e API (REST da IF → Canonical, schema versionado por schemars, idempotency-key)**.
- Fix do wizard no frontend contra os tipos TS gerados do OpenAPI; validação streaming via SSE.

**Riscos da fase:** IA é a fronteira não-determinística — regra estrutural do doc 20 §12: **IA nunca produz XML, nunca decide verdict**; ela explica, propõe e descobre; o engine verifica. O Radar pode encontrar URLs BACEN 404 (já aconteceu no v1) → fontes com fallback e alerta explícito. O custo de challenger/synth é controlado por budget diário (US$30–60/mês na faixa do research/07).

**Alocação semanal:**

| Semana | Foco |
|---|---|
| 20 | Conectores File (calamine) e API; wizard fix contra tipos TS gerados |
| 21 | Radar: crawler + hash por §; A3 entra por feature flag se o spike provou com SoftHSM |
| 22 | Radar: diff semântico de leiautes + auto-PR + calendário normativo; generators 2070/2160 |
| 23 | Harness M1/M2: discrepância→case e correção→case com snapshot automático |
| 24 | Harness M6: gates nos 3 tiers; miner M7 semeado; IA: router multi-modelo + sanitizador PII |
| 25 | IA: copiloto com citação de Provenance; RAG (pgvector + Tantivy + RRF) |
| 26 | Integração e demonstração dos 4 dones da fase; review de fase |

**Done (evidência externa):** harness-gate **bloqueia um PR real** com regressão de precision/recall (log do gate vermelho anexado); Radar gera PR automático a partir de um diff simulado do leiaute dez/2026 (commit sintético versionado no repo); copiloto responde com citação clicável em ambiente interno (gravação); conector File importa XLSX real de teste e produz `CanonicalDocument` validado ponta-a-ponta.

### 2.6 F5 — Launch comercial (sem 27–32 · 17/mai–27/jun/2027)

**Entregáveis:**

- Launch com o **pricing novo** (research/23 §2): Free Validador R$0 (wedge PLG), Starter R$2.490, Pro R$6.900, Scale R$16.900, Enterprise R$29.900+; add-ons +CNPJ R$590, +50k ops R$190, DRSAC R$990; desconto nunca é dinheiro — é "crédito de dados" (opt-in para o benchmark, protegendo o data moat).
- SDKs regenerados **1 por linguagem** (o v1 tinha 2 gerações colidindo em nomes de pacote — research/01).
- **MCP server** (`rmcp`): tools `validate_document`, `generate_cadoc`, `explain_rule`, `search_norma` — read-only, allow-listed, log de toda tool call; conteúdo recuperado é dado citado, nunca instrução (mitigação de injection).
- Conectores **DB (Postgres read-only da IF, credenciais em vault, row-cap) e MCP** — completando os 5 conectores do produto.
- **On-prem option** para Enterprise: binário musl + guia de instalação (single binary é argumento de venda do Rust).
- **SOC 2 readiness REAL:** controles implementados, coleta de evidência operando de verdade, pentest interno do agente executado com findings triados, **cronograma assinado com auditora contratada** — não "software de compliance" como no v1 (`internal/soc2` era façade de 0,8% de cobertura; research/02 §3.2).

**Riscos da fase:** venda para banco S3-S4 tem ciclo 6–12 meses — o launch mira SCD/IP (15–45 dias) com PLG-assisted e 2 parceiros consultoria com rev-share; SOC 2 Type I não fecha no launch (processo de meses) — o que se vende no procurement inicial é LGPD + pentest + readiness com data marcada; preço público é arma, mas precisa da página e do contrato prontos antes da primeira demo.

**Done (evidência externa):** **N4** — 1 contrato pagante assinado **e dinheiro recebido**; página de pricing pública no ar; pentest executado com findings triados; ao menos 1 demo gravada com ciclo de compra documentado.

### 2.7 Buffer (sem 33–36 · 28/jun–25/jul/2027)

Não é fase: é contingência nomeada. Só se "gasta" o buffer com regra registrada (cortes da F4 na ordem fixa do §2.0). Buffer não consumido = F6 começa antes e o launch L2 ganha folga. Buffer consumido sem corte de escopo = sinal de planejamento otimista demais → próxima re-ancoragem puxa mais buffer, não menos.

### 2.8 F6 — Flywheel composto (contínuo, a partir de ~jul/2027)

**Entregáveis contínuos:**

- Mineração contínua (M1 discrepância→case, M2 correção→case, M7 clusterização→rule_candidate) alimentando o corpus com provenance; triagem semanal com agente + aprovação humana em batch.
- Gates de precisão por release; **Accuracy SLA vendável** — FN rate held-out contratual (o único pricing que o Honesty Engine permite oferecer honestamente).
- **Health scorecard semanal** (Honesty Engine): Mutation Detection Rate ≥75%, judge TPR/TNR ≥95%, canários 100%, held-out leak 0, Rule Coverage Index ≥80%, Time-to-Green p50 ≤7 dias.
- T2U (norma publicada → leiaute em sandbox) medido em cada onda normativa — é o SLA público "leiaute no dia da norma" (≤10 dias úteis) do research/23.
- Destilação M8 só nos gatilhos (≥200 casos por regra; custo LLM >US$200/mês; judge estável 8 semanas) — nunca "porque ficou legal".

**Mapeamento das fases do harness v4 (research/09 §8) sobre as fases deste roadmap:**

| Fase harness (research/09) | Onde cai neste roadmap | Critério do harness |
|---|---|---|
| H-F0 (schema, runner, goldens, proptest) | F0–F2 deste roadmap (goldens na F0; runner/gates no motor da F2) | `harness-gate --tier pr` rodando em PR |
| H-F1 (M1 discrepância, M2 correção, miner, triagem) | F4 deste roadmap (semanas 23) | ≥1 case novo/semana com provenance |
| H-F2 (M3 Radar→suite por vigência, M6 gates completos, scorecards) | F4 deste roadmap (semanas 22 e 24) | Radar detecta diff → suite gerada sem código manual |
| H-F3 (M4 challenger/solver/judge, matriz regra×edge, canários, held-out) | F4–F6 (semana 25 + contínuo) | RCI ≥60%; canários 100% |
| H-F4 (M7 clusterização, M10 auditoria + calibração, dashboards, cargo-mutants) | F6 (contínuo) | Harness Health Scorecard no ar; RCI ≥80% |
| H-F5 (M8 destilação; Accuracy SLA como pricing) | F6, só nos gatilhos do M8 | Primeiro A/B de engine destilada aprovado em held-out |

**Done (evidência externa, contínua):** FN rate mensurado caindo por trimestre; scorecard público atualizado por máquina; kill criteria K1–K5 (tese v2 §7.3) avaliados **nas datas, por pessoas** — não pelo changelog.

### 2.9 Calendário consolidado

| Fase | Semanas | Datas (2026→2027) | Gate de saída (evidência externa) |
|---|---|---|---|
| F0 fundação | 1–2 | 16/nov–29/nov/2026 | CI + gates rodando; catálogo e goldens contados por script |
| F1 domínio/leiautes | 3–6 | 30/nov–27/dez/2026 | Tipos nov/2026 compilam; RLS provado; catalog gate quebra build |
| F2 pipeline+paridade | 7–12 | 28/dez/2026–07/fev/2027 | **N1** paridade green + **N2** BCValidador oficial |
| F3 API/STA/envio | 13–19 | 08/fev–28/mar/2027 | **N3** envio STA-h homolog real (A1) |
| F4 radar/IA/harness | 20–26 | 29/mar–16/mai/2027 | Gate bloqueia PR; Radar auto-PR; copiloto com citação |
| F5 launch comercial | 27–32 | 17/mai–27/jun/2027 | **N4** contrato pagante + dinheiro |
| Buffer | 33–36 | 28/jun–25/jul/2027 | contingência nomeada (cortes em ordem fixa) |
| F6 flywheel | contínuo | desde ~jul/2027 | scorecard semanal; K1–K5 nas datas |

**Conflito Q1/2027 × 32–36 semanas, resolvido explicitamente:** o launch Q1/2027 do research/23 não cabe como launch completo de 5 tiers com o cronograma honesto de research/04. Resolução: **launch escalonado** — (L1) em **mar/2027**, ao fim da F3, abre o **Free Validador + Starter R$2.490 (3040 com geração, validação e envio manual)** — exatamente o escopo do tier Starter definido no research/23; (L2) em **jun–jul/2027**, ao fim da F5, abre Pro/Scale/Enterprise, DRSAC completo na 2ª data-base (jun/2027), API/webhooks e white-label. A projeção MRR de research/08 (conservador: R$76k no m12) mantém-se plausível porque L1 dá 9–10 meses de venda até dez/2027. O kill criterion K2 (30/jun/2027: <10 clientes ou <R$50k MRR) permanece o disjuntor inalterado — ele é avaliado sobre o L1 que já estará no ar, não sobre uma promessa.

### 2.10 O que explicitamente NÃO entra no v2 (exclusões de escopo)

Herança direta da lição "multi-tenancy completo antes do 1º cliente" (research/02 §7.2, item 4) e da auditoria dos mortos:

| Excluído do v2 (pós-N4 no mínimo) | Por quê |
|---|---|
| Multi-region real (BR-SP1/SP2) | O `multiregion` do v1 era CRUD de metadados, não replicação; prematuro com <10 tenants |
| Marketplace de regras | Ninguém tem regras para vender antes de ter clientes usando as nossas |
| Billing Stripe completo antes do 1º contrato | No v1 existia client Stripe sem conta live; no v2, billing entra quando N4 exigir — não antes |
| White-label | Tier Scale; só faz sentido com base pagante |
| Webhooks + SDKs em 2 linguagens por geração | 1 SDK por linguagem no launch; webhooks na F5 só se um cliente pedir em contrato |
| SLOs públicos (99.9% uptime etc.) | Lição research/02 §7.2 item 8: SLO sem produção é ficção; publicam-se após 3 meses de produção real |
| Documento 3026 e CADOCs fora dos 13 vigentes | Resquício do v1 (`rules/3026.go` era morto); o enum `CadocCode` é fechado |
| Stubs severity-I | Portar só as ~358 regras com lógica real; o catálogo serve de checklist e o gate impede stub silencioso |

---

## 3. Ponte DRSAC: plano A com wiring obrigatório, plano B com disjuntores

**Contexto regulatório verificado (research/23 §8, research/06):** IN BCB 694/2025 muda leiaute/instruções do 2030 com vigência na data-base dez/2026; DRSAC obrigatório para S1–S4 + IPs (~455 entidades + IPs enquadradas); semestral (jun/dez); prazo de remessa: 10º dia útil do 2º mês subsequente → **1ª remessa sob o novo leiaute vence ~fev/2027**; sem exposição = "documento zerado" via CRD; **não existe planilha pública de críticas para o 2030** (só XSD + validador Windows + FAQ) — o que torna um validador semântico próprio diferencial real, mas também significa que o piloto não pode prometer cobertura de regras auditada.

**Nota da fusão (16/11/2026):** a salvaguarda 1 do plano A (**wiring mínimo do gen2030 em Go**) foi **retirada do escopo — retrabalho** duplicar o gen2030 em Go quando o `reg-cadoc` 2030 em Rust é entregável da FASE 1 (S4, §0). O piloto DRSAC passa a rodar sobre o **XML canônico gerado pelo Rust contra o leiaute oficial** + chancela do validador oficial (B02 — ADR-0018); o Go só sobrevive como **ponte STA-h para smoke** (S1). O **kill criterion de 30/11/2026 recebe nova redação** — Bloqueio A.2 abaixo.

### 3.1 Plano A (recomendado): motor Go v1 serve o piloto dez/2026–fev/2027

O `gen2030` existe no v1 — mas a auditoria provou (research/01 §2.2 e §3.2) que **`drsac.ValidateDocument` é órfão**: zero chamadores em produção; `IntegrateWithAudit()` (`drsac.go:169`) é um no-op que ninguém chama; as 35 regras D01–D35 nunca rodaram contra críticas oficiais — que não são públicas. Portanto o plano A **só é honesto com estas duas salvaguardas obrigatórias**:

1. **Wiring mínimo no v1 (1–2 dias, semana 1):** endpoint `POST /v1/validate/2030` expondo `drsac.ValidateDocument`, com resultado gravado no audit log hash-chain. Sem isso, o "validador DRSAC" do piloto seria exatamente a classe de milestone-virando-código que a auditoria condenou (Pilot2/3/4, SOC2-as-code).
2. **Validação externa obrigatória:** todo XML do piloto passa no **BCValidador oficial** (validador Windows do BCB) e no CRD antes de qualquer envio. O oráculo externo é a fonte de verdade do piloto; o engine interno do v1 é assistente de rascunho, nunca autoridade — porque ninguém pode provar que as D01–D35 do v1 correspondem às críticas reais do BCB (que não são públicas).

**Escopo do piloto:** documento zerado 2030 via CRD (baixa barreira, dor real, caso de abertura definido no research/23 — movimento 1), 5–10 IFs S1–S4 beta fechado, captado via FBM/consultorias + lista nominal S1–S4 (pública no site do BCB). Limitação **declarada por escrito** ao piloto: o produto do piloto é geração + validação BCValidador + envio assistido; sem Accuracy SLA, sem regras semânticas auditadas. Vender mais que isso seria forward-padding.

**Meta do movimento 1 (research/23 §8):** 3–5 IFs no beta, ≥1 remessa zerada aceita no CRD até ~fev/2027, 5 contratos convertidos até a 1ª remessa.

### 3.2 Critérios de bloqueio e switch

| Gatilho | Condição | Ação |
|---|---|---|
| **Bloqueio A.1** | Credencial Sisbacen não obtida até **30/nov/2026** | Piloto recai para "gera XML + valida no BCValidador + IF submete via CRD web por conta própria" — o documento zerado aceita isso. Envio via STA WS não é pré-requisito do zerado; o que trava tudo é acesso ao CRD, que depende da mesma credencial. |
| **Bloqueio A.2** | Até **30/nov/2026** não houver: **saneamento completo em `main`** + **re-download oficial do 2030 versionado** (com SHA-256) + **pedido PSTA300 protocolado** + **`reg-cadoc` 2030 em Rust gerando XML canônico contra o leiaute oficial** | **Nova redação (fusão 16/11/2026)** — substitui o wiring do gen2030 em Go, retirado do escopo por retrabalho. Não fechado → congelar investimento no legado e ativar plano B imediatamente. |
| **Bloqueio A.3** | Nenhuma IF sinalizada para o beta até **05/dez/2026** | Sem beta não há piloto — plano B; o relógio comercial passa a mirar a 2ª data-base (jun/2027). |
| **Switch A→B** | Defeito estrutural no `gen2030` do v1 custando >1 semana de correção | Não vale investir em legado morto: plano B. |
| **Plano B (detalhe)** | Piloto deslocado para o Rust pós-F3 (~mar/2027) | Perde a 1ª data-base (K5 dispara: 0 IFs pagantes da janela até 28/fev/2027 → matar GTM de janela); mira a 2ª data-base (jun/2027) usando a **transição dos códigos 01–04 dos Anexos 08/09** (deixam de ser aceitos em registros novos em jun/2027) como gatilho de dor novo. O leiaute dez/2026 continua sendo capturado como dataset da F1 — o trabalho não se perde. |
| **Kill da jogada DRSAC** | 2ª data-base (jun/2027) chegar sem 2030 em produção (Rust **ou** v1) | K1/K5 consolidados: jogada DRSAC morta para o ano 1; GTM volta para varredura de consultorias; o 2030 vira case de estudo, não aposta. |

**Decisão:** plano A com wiring mínimo + validação externa obrigatória. O plano B é automático se qualquer bloqueio disparar — sem negociação no calor do momento (mesma lógica dos thresholds do harness: afrouxar exige decisão explícita documentada).

---

## 4. Marcos N0–N6 — honestos, substituem M1–M4

Todos os marcos seguem a doutrina do research/02 §9.2: **milestone só fecha com fato externo verificável** (credencial, protocolo, XML aceito, contrato, dinheiro). "Fechada como classe Rust" é proibido — a lição dos Pilot2/3/4 e do SOC2-as-code. Cada marco tem issue própria no tracker, criada na F0, com o campo "artefato que prova" preenchido de antemão. **N0 entra pela fusão (16/11/2026) e precede a cadeia N1–N6** — é o gate de saída da FASE 1; **N1–N6 continuam válidos para a FASE 2** (§0).

**Resumo executivo dos marcos:**

| Marco | Evidência externa que prova | Data alvo | Kill |
|---|---|---|---|
| **N0** | Primeiro contrato pago da FASE 1 assinado **+ dinheiro recebido** (PoC mascarada → Sprint → Licença — 23-PRODUTO §2.0) | 31/dez/2026 | Bloqueio A.2 em nova redação (30/nov); 31/dez sem N0 → reprecificar o wedge |
| N1 | Paridade golden 100% green (machine-generated) | 07/fev/2027 | 28/fev sem ≥90% no 3040 → cortar escopo |
| N2 | XML Rust aceito no BCValidador oficial | 14/fev/2027 | 28/fev sem aceite → spike de re-derivação |
| N3 | Protocolo de envio STA-h homolog capturado | 28/mar/2027 | 31/jan sem credencial → N3' via CRD manual |
| N4 | Contrato + dinheiro recebido + 3º mês pago | 31/mai/2027 (retenção 31/ago) | 30/jun sem contrato → reprecificar/trocar canal |
| N5 | ≥10 clientes pagantes, MRR ≥R$50k | 31/dez/2027 | K2 (30/jun) e K2' (31/mar/2028 → bootstrap) |
| N6 | FN rate <0,5% por 2 trimestres + 1º cliente com SLA | 31/jan/2028 | K4 (eval flat) → remover SLA do pitch |

A cadeia N1→N6 é deliberadamente **consequencial**: cada marco é pré-condição do seguinte (paridade → oráculo externo → envio real → cliente → escala → SLA). Pular um elo é a receita exata do colapso do v1 (vendeu "smoke validado" sem credencial; fechou "SOC 2" sem auditor).

### N0 — Primeiro contrato pago (FASE 1) · alvo 31/dez/2026 — *adicionado pela fusão (16/11/2026)*

- **Evidência:** contrato assinado **+ dinheiro recebido** da régua do wedge (PoC mascarada R$0 → Sprint R$35–60k → Licença R$6–12k/mês — 23-PRODUTO §2.0); meta da FASE 1: **2–4 contratos até 31/12/2026** (S6, §0).
- **Relação com N4:** N0 é o contrato do **wedge on-prem** (FASE 1); N4 é o contrato do launch SaaS da plataforma (FASE 2). N0 não substitui N4 — precede N1 e financia a FASE 2.
- **Kill:** 30/nov/2026 sem os 4 itens do Bloqueio A.2 em nova redação (§3.2) → plano B; 31/dez/2026 sem N0 → a FASE 1 não converteu: reprecificar o wedge (23-PRODUTO §2.0) antes de alocar qualquer recurso da FASE 2.

### N1 — Paridade golden green · alvo 07/fev/2027 (fim F2)

- **Evidência:** suite de paridade dos goldens extraídos dos 1.178 testes Go 100% green **no Rust**, contada pelo harness (número machine-generated emitido pelo runner).
- **Kill:** 28/fev/2027 sem ≥90% nos goldens do 3040 → cortar escopo da F2 para 3040-only; 3050 vai para o buffer. Segundo kill: 31/mar/2027 ainda abaixo → re-avaliar a estratégia de derivação das regras (RuleSpec manual em vez de tradução batch).

### N2 — XML Rust passa no BCValidador oficial · alvo 14/fev/2027

- **Evidência:** XML gerado pelo motor Rust aceito pelo validador oficial do BCB (log/print do validador anexado à issue). Este é o marco que prova que o codegen XSD não é teoria.
- **Kill:** 28/fev/2027 sem aceite por divergência estrutural não-trivial → 2 semanas de spike de re-derivação do codegen (flags do `xsd-parser`, newtypes de domínio) antes de qualquer fallback manual dos leiautes críticos.

### N3 — Primeiro envio STA-h homolog real · alvo 28/mar/2027 (fim F3)

- **Evidência:** protocolo de transmissão/aceite do ambiente de homologação BACEN capturado (hash pré-envio, payload, resposta, timestamps).
- **Pré-requisito:** credencial Sisbacen (pedido aberto nas 72h; follow-up semanal).
- **Kill:** 31/jan/2027 sem credencial após 2 follow-ups formais → **N3'** (remessa aceita no CRD de uma IF real, envio manual) substitui com data renegociada; a remessa automática via STA sai do pitch até resolver.
- **Nota histórica:** a Sprint 29 do v1 prometia exatamente este smoke e nunca rodou — "pendente (credenciais)" desde jul/2026. N3 é o marco que fecha essa dívida de 8 meses.

### N4 — 1 cliente pagante · assinatura alvo 31/mai/2027, retenção confirmada 31/ago/2027

- **Evidência:** contrato assinado **+ dinheiro recebido** (recibo/boleto/pagamento Stripe) + confirmação de retenção no 3º mês pagante. "Cliente" não é conta criada, não é tenant seedado, não é usuário beta.
- **Kill:** 0 contratos até 30/jun/2027 (K2 parcial) → reprecificar/trocar canal (consultorias no lugar de PLG — o disjuntor do research/23); cancelamento antes do 3º mês → pivotar GTM (a lição exata do M1: "cancelamento = pivotar GTM").

### N5 — 10 clientes / R$76k MRR · alvo 31/dez/2027

- **Evidência:** ≥10 clientes pagantes ativos com MRR ≥R$50k (piso do K2) e run-rate na trajetória do cenário conservador de research/08 (20 clientes/R$76k no m12 é o conservador; N5 pede o piso honesto: 10 clientes e a trajetória para R$76k).
- **Kill:** K2 já dispara antes (30/jun/2027: <10 clientes pagantes OU MRR <R$50k → pricing/GTM errados); segundo kill: 31/mar/2028 sem 10 clientes → bootstrap mode (lição do M4: "< R$50k MRR = bootstrap").

### N6 — Accuracy SLA vendável · alvo 31/jan/2028

- **Evidência:** FN rate held-out contratável (<0,5% por 2 trimestres consecutivos com ≥30 eventos/IF) + judge TPR/TNR ≥95% contra amostra humana auditada + health scorecard público atualizado por máquina + **primeiro cliente com Accuracy SLA no contrato** (add-on — o pricing que o Honesty Engine do research/09 §7 permite oferecer).
- **Kill:** K4 (eval flat: accuracy sem melhoria por 2 trimestres) → o SLA não é vendável; reposicionar o pitch em corretude + cobertura de leiaute e remover "Accuracy SLA" da página até cumprir.

**Regra de leitura:** os kill criteria **disparam antes** dos alvos correlatos por design (K2 em 30/jun avalia o que N5 só confirma em dez). Kill criterion avaliado e não disparado também é evidência — negativa — e reancora o plano. Nenhum marco se "renegocia" silenciosamente: mudança de data de marco vira entrada no STATUS.md com o artefato que justificou.

---

## 5. Orçamento (infra + IA + ferramentas)

### 5.1 Infraestrutura

| Item | Fase de build (F0–F4) | Launch/prod (F5+) | Notas |
|---|---|---|---|
| Compute (Fargate ARM64: api 0.5vCPU/1GB + worker 0.5vCPU/1GB) | staging ~US$40/mês | ~US$40/mês | binários Rust 20–40MB distroless/cc (convive com xmllint); alternativa dev: VPS Hetzner ~€20/mês |
| Postgres gerenciado | dev/staging ~US$30/mês | ~US$120/mês (small) | RLS é o enforcement — Postgres único em prod; SQLite só em teste unitário de engine |
| Redis | dev ~US$10/mês | ~US$25/mês | sessão/rate-limit/locks de job |
| ALB + NAT + S3 + ECR | ~US$30/mês | ~US$60/mês | S3 para artefatos de envio (hash, payload, resposta) |
| Observability | Sentry free + OTel self-host ~US$0 | ~US$26/mês | pinar versões OTel no `[workspace.dependencies]` (research/04 §R9) |
| CI (GitHub Actions) | ~US$20/mês | ~US$30/mês | inclui minutes de fuzz nightly (cargo-fuzz) e cargo-mutants semanal |
| **Subtotal infra** | **~US$130–170/mês** | **~US$300/mês** | |

### 5.2 IA — TCO por estágio (research/07 §9)

| Estágio | Gatilho | Composição | Custo/mês |
|---|---|---|---|
| **0 — Build (F0–F4)** | agora, zero clientes | agentes de dev (ZCode) + LLM de eval-nightly + challenger sintético reduzido + embeddings local | ~US$150–250 |
| **1 — MVP 100% API** | F4/F5 | router multi-model + sanitizador PII + copiloto (Flash/Sonnet) + RAG (Qwen3-Emb/BGE self-host) + harness ~10 IFs | **~US$505** (cenário A do research/07) |
| **2 — L40S on-prem** | 1º cliente exigir residência de dados | Qwen3-30B-A3B + Qwen3-Embedding + BGE-Reranker (OCI São Paulo) | +US$2.555 (repassado ao Enterprise) |
| **3 — H100 + GPT-OSS-120B** | >50 IFs ou 2 clientes air-gapped | copiloto on-prem completo via vLLM/SGLang | +US$7.300 |

Regra absoluta: estágios 2 e 3 são acionados **por gatilho contratual, nunca por precaução**. O crossover cloud→GPU fica entre 100–300 IFs (research/07 §9). A @100 IFs o total IA é ~US$2.800/mês (cenário B) — ainda <15% da receita do cenário base de research/08.

### 5.3 Ferramentas e one-offs

GitHub (repo privado) + Actions já contados na infra; **marca nova** (busca de anterioridade INPI + registro em 2 classes): ~R$3–5k; domínios .com.br/.com: ~R$400; pentest interno via agente: $0; SoftHSM (spike A3): $0 (software); Mac Studio 128GB para dev de modelos on-prem (opcional, só no estágio 2): US$3.499 one-time — não entra no budget do build.

### 5.4 Total por fase e custo do build até launch

| Fase | Duração | Infra+IA+ferramentas/mês | Total da fase (aprox.) |
|---|---|---|---|
| F0–F1 | 6 sem | ~US$350 | ~US$520 |
| F2 | 6 sem | ~US$450 | ~US$675 |
| F3 | 7 sem | ~US$580 (inclui staging com STA) | ~US$1.015 |
| F4 | 7 sem | ~US$750 (IA challenger/synth ativa) | ~US$1.312 |
| F5 | 6 sem | ~US$1.155 (prod + IA estágio 1 + ferramentas) | ~US$1.732 |
| Buffer | 4 sem | ~US$1.155 | ~US$1.155 |
| **Total build (32–36 sem)** | ~8 meses | — | **~US$6.4–7.4k ≈ R$38–44k** |
| One-offs (marca/INPI/domínios) | — | — | ~R$4–5.5k |
| **Custo total até launch** | — | — | **~R$42–50k** |

**Pós-launch (10 IFs):** infra ~US$300 + IA US$505 + ferramentas US$50 ≈ **US$855/mês (~R$5k)** contra MRR de ~R$24,9k (10 Starter) — margem bruta ≥85%, consistente com a meta de research/08 §5.3. **@100 IFs:** ~US$3.700/mês (~R$21,5k) contra MRR na casa de R$200k+ — ainda ≥89%. A GPU on-prem só entra quando o cliente paga por ela (add-on Enterprise ou implantação assistida). Sensibilidade: se o lead-time da credencial esticar o build além de 36 semanas, o custo cresce ~US$1.155/mês de buffer estendido — o teto de dor é ~R$60k total, ainda dentro do bootstrap de research/02 (premissa A23).

---

## 6. Capacidade realista: o que 1 dev + agentes entrega (e o que não entrega)

**A evidência disponível é dupla, e as duas metades são verdadeiras ao mesmo tempo:**

1. **Código não é mais o gargalo.** 263 commits em 11 dias produziram 57k LoC que compilam, passam 1.178 testes e sobreviveram a uma auditoria dura com evidência `arquivo:linha`. A velocidade de geração de código com agentes é ~10× a de um dev humano sozinho. Na prática: **~25–40 PRs/semana de código revisado** é sustentável; 1.500–2.500 LoC Rust novo/reescrito por semana é meta conservadora honesta; o que a IA não acelera são decisões de ownership/lifetimes e modelagem de domínio (research/04 §R16) — e por isso a revisão humana é o gargalo, não a digitação.
2. **Evidência externa é 1× humano — e é o único relógio real.** Os sprints-fantasma provaram que CÓDIGO ≠ PRODUTO: 4 meses depois do último commit, zero credencial Sisbacen (Sprint 29 skipada desde sempre: `smoke_test.go:10`), zero smoke, zero deploy, zero cliente, zero receita. Credencial é burocracia do BCB (semanas–meses); envio homolog depende de calendário externo; venda tem ciclo de 15–45 dias (SCD/IP) a 6–12 meses (banco S3-S4). **Nenhuma aceleração de IA comprime esses relógios.** Por isso o cronograma é governado por lead-times externos — o pedido de credencial abre na hora zero (§8, ação 2) — e não por Linhas de Código Semanais.

**Regime de trabalho (anti-burnout, porque o projeto morre se o único dev cair):**

- 5 dias/semana; sprints de 1 semana com revisão de fase no fim, não sprints-fantasma de "2 semanas" que viram 36 horas de commit burst.
- 1 slot semanal fixo de 30–45 min de auditoria amostral do harness (M10: 30 casos estratificados) — é o único laço não-sintético de verdade sobre o próprio sistema.
- Revisão humana concentrada em decisões de domínio/ownership; a máquina revisa o resto em série: clippy `-D warnings` → linter anti-wildcard → catalog gate → harness-gate → cargo-mutants. IA revisa IA antes do humano.
- 20% do calendário já reservado como buffer (§2.7); 1 semana off planejada após N3 e outra após o launch.
- Máximo de 2 frentes em paralelo: o caminho crítico + no máximo 1 spike/background (A3 na F3, Radar-crawler na F4).

**Regra de ouro anti-forward-padding (vale para humanos e agentes, sem exceção):**

> **"Done exige evidência externa: credencial, envio real, cliente real, dinheiro real — ou métrica emitida por máquina."**

Derivações operacionais:

- Nenhum ✅ escrito à mão em STATUS.md, ROADMAP.md ou CHANGELOG. O status é **machine-generated** (script → tabela a partir do build e do harness); o CI bloqueia docs com claims não rastreados a um artefato.
- Milestone não pode virar classe Rust. "SOC 2", "Pilot", "Cliente" são eventos do mundo; no código existem apenas os componentes que os *suportam*.
- Todo marco N1–N6 aponta, desde o dia 1, para o artefato externo que o prova (protocolo, log do BCValidador, contrato, recibo) numa issue própria.
- Sinal de sprint-fantasma: **>3 semanas consecutivas sem nenhuma evidência externa nova** → stop, re-planejar, cortar escopo. Não existe "estava quase pronto".
- Proibido antecipar estado futuro como presente (a lição do MEMO_MOTOR_GERACAO: "✅ COMPLETO" escrito no mesmo dia em que 9 de 10 generators eram "📋 Sprint 58").
- Doc de marketing sem revisão humana não existe (a lição dos `已完成`/`开发` do README v1).

**DoD por PR (Definition of Done no dia a dia):**

- [ ] CI completo green: fmt, clippy `-D warnings`, linter anti-wildcard, test, catalog gate, drift check.
- [ ] Toda regra nova/alterada tem golden ou é `Stub{motivo}` declarado (o gate impede o meio-termo).
- [ ] Toca regra validada? → harness-gate roda o tier PR; regressão de precision/recall bloqueia.
- [ ] Mensagem de erro nova? → variante tipada no enum (`thiserror`), nunca string solta.
- [ ] Número novo na UI? → carrega `Provenance` (a UI cita a fonte ou não exibe).
- [ ] `STATUS.md` regenerado pelo script — nada manual.
- [ ] Revisão humana apenas nos pontos de domínio: modelagem, arredondamento por leiaute, decisão de vivência de regra.

---

## 7. Riscos top-10 com mitigação

| # | Risco | Severidade | Mitigação concreta | Sinal de alarme precoce |
|---|---|---|---|---|
| 1 | **XSD 1.0 sem validador Rust maduro** (crate `xmlschema` é 0.0.x; único risco real do stack — research/04 §R3) | Alta | Arquitetura em 3 camadas (ADR-0012): codegen `xsd-parser` + validação estrutural inline + **`xmllint`/libxml2 como autoridade** via pool de subprocessos com erro explícito `L1AuthorityUnavailable`; fuzz `cargo-fuzz` obrigatório; plano B extremo: sidecar Xerces-J. Nota: Go também não tem validador nativo — o risco é de engenharia, não de impossibilidade | Spike da semana 3 (F1) não converte os XSD oficiais sem trabalho manual pesado |
| 2 | **Credencial Sisbacen não obtida** (em 4 meses de projeto, nunca foi) | Alta | Pedido aberto nas 72h com checklist completo + follow-up semanal agendado; fallback desenhado: validação BCValidador + envio manual via CRD (aceitável para documento zerado); sem ela, N3 não fecha e remessa automática sai do pitch | Follow-up #2 sem resposta; 31/jan/2027 é a data de degradação para N3' |
| 3 | **DRSAC: críticas restritas** (não públicas; v1 tem engine órfão) | Média-alta | Piloto vende geração + BCValidador + envio assistido do zerado (sem promessa de regras semânticas); wiring mínimo no v1 obrigatório (§3.1); validador semântico próprio vira diferencial *depois*, alimentado por discrepâncias M1 do harness | IF do piloto pedir cobertura de regras que não existe; wiring não green até 30/nov |
| 4 | **Revisão de código Rust por 1 pessoa** é o gargalo do throughput (research/04 §R15/R16) | Média-alta | Revisores automáticos em série (clippy `-D warnings` + linter anti-wildcard + catalog gate + harness-gate + cargo-mutants); humano só em decisão de domínio; cap de ~40% do tempo do dev em revisão; IA revisa IA antes do humano; PRs pequenos | Fila de revisão >2 dias por PR; clippy warnings acumulando |
| 5 | **Burnout do único dev** | Média-alta | Buffer de 20% nomeado; semanas de 5 dias; 1 semana off após N3 e após launch; kill criteria existem para permitir **parar** sem culpa; spikes delegados a agentes em background; nada de commit burst tipo 66 commits/dia | 2 fins de semana consecutivos trabalhando; slots de auditoria amostral pulados |
| 6 | **BACEN muda leiaute no meio do port** (IN 733 onda 3 é nov/2026 — agora) | Média | Leiautes nov/dez/2026 são o primeiro dataset da F1 (não jul/2026); codegen via xtask = regenerar é barato; goldens versionados por vigência; Radar na F4 com T2U como métrica de produto; suites de regressão por vigência (M3 do harness) | Radar 404 em fonte BACEN sem fallback; diff manual encontrando mudança não coberta |
| 7 | **Concorrente copia** (Matera/cadoc.ai re-entram em geração; novo entrante RegTech com IA) | Média | Moat não é segredo de código: é flywheel (cada submissão melhora a rede — pitch Stripe Radar) + T2U ≤10 dias úteis + pricing público + velocidade de resposta ao leiaute. Monitorar trimestralmente (research/05); integração com cadoc.ai (Norma Skills como fonte normativa) em vez de guerra de keyword | 2 movimentos de concorrente em geração AI-driven na mesma onda normativa |
| 8 | **Migração de catálogo** (11 códigos duplicados, chaves `2061-DLO` quebradas, 1.099 críticas de proveniência incerta) | Média | ETL de normalização na F0 com schema único + testes de dedup; completeness gate impede regressão *para sempre* (regra sem estado quebra o build); paridade contada por regra pelo harness, nunca por número de README | Gate passando com regra sem golden; contagem divergindo entre script e docs |
| 9 | **Frontend legado** (Next.js 15 do v1 com wizard parcialmente quebrado e contratos driftados — 46 paths vs ~70 rotas) | Média | Frontend permanece TS (decisão do doc 20); não portar o legado: wizard reconstruído contra OpenAPI utoipa com tipos TS gerados (`api-types.d.ts` que nunca existiu); fix mínimo só na F4; frontend nunca bloqueia o motor | Wizard consumindo endpoint fora do OpenAPI; SSE sem timeout layer |
| 10 | **Judge IA inflar métricas** (green-wash do próprio harness — "o sistema otimiza para o judge, não para a verdade") | Média | Honesty Engine (research/09 §6): judge de família diferente do solver; TPR/TNR contra amostra humana semanal; canários em todo run (run com canário errado é inválido); held-out selado criptografado; Mutation Detection Rate no scorecard; métrica só vale com fingerprint (engine+ruleset+judge+corpus); auditoria humana amostral semanal é a única fonte não-sintética | Judge agreement caindo; canários passando com corpus suspeito; métricas melhorando sem evento externo novo |

---

## 8. Primeiras 72 horas (16–18/nov/2026) — 10 ações imediatas

1. **Criar o repo novo** (nome da marca nova — decisão pendente do research/23 §7, com busca INPI em paralelo) com cargo workspace das 16 crates stub, `[workspace.dependencies]` pinado, e CI (fmt + clippy `-D warnings` + test + catalog gate + drift check). Commit 1 já contém os ADRs 0009–0014 assinadas.
2. **Abrir o pedido de credencial Sisbacen** (homologação) — o item de maior lead-time do projeto inteiro e o bloqueio histórico do v1 desde jul/2026. Checklist dos formulários, documentação da IF/CNPJ, protocolo registrado, follow-up semanal recorrente no calendário. Se a credencial depender de IF parceira, mover para o topo da conversa com o piloto (ação 10).
3. **Extrair o golden corpus**: script `xtask` converte os 1.178 testes Go em fixtures input/expected versionadas; marcar `flaky-spec` os dependentes de globals/ordem e derivá-los manualmente; etiquetar cada golden com o status de produção do código que o originou (wired vs órfão — o mapa honesto da F2).
4. **Normalizar o catálogo**: ETL `criticas.json` → schema único; dedup dos 11 códigos duplicados; chaves sufixadas (`2061-DLO`) → código de 4 dígitos + categoria; contagem publicada pelo script; `STATUS.md` machine-generated no ar desde o primeiro dia.
5. **Baixar e versionar os leiautes nov/2026** (IN 733 onda 3), XSDs oficiais, FAQ DRSAC dez/2026 e planilhas de Regras de Validação — git-tag por data-base no repo de catálogo. O v1 parou em jul/2026; esta é a primeira correção de verdade da base normativa.
6. **Wiring mínimo do drsac no v1** (plano A da ponte) — **revogado pela fusão (16/11/2026)**: o gen2030 em Go saiu do escopo (retrabalho); o critério de 30/nov passa a ser a nova redação do Bloqueio A.2 (§3.2: saneamento completo em main + re-download oficial 2030 versionado + pedido PSTA300 protocolado + `reg-cadoc` 2030 em Rust gerando XML canônico). Nas 72 horas, a evidência externa equivalente é o **saneamento S1 completo** (§0) e o protocolo PSTA300 registrado; o smoke do 2030 zerado passa a ser contra o **validador oficial (B02) sobre o XML do Rust** (S4).
7. **Criar o tracker dos 6 fios desligados** como gates de não-regressão arquitetural: um gate por fio (dispatch silencioso, chaves sufixadas, XSD relativo, parse repetido, defaults silenciosos, motores duplos) — cada um vira um teste/CI que falha se o padrão renascer no código novo.
8. **Provisionar infra dev**: Postgres gerenciado + Redis + bucket + Sentry + domínio; budget cap mensal configurado no cloud (o orçamento do §5.4 só é honesto se houver teto de gasto automático); ambiente roda as migrations 001–026 limpas.
9. **Abrir as issues dos marcos N1–N6** com o campo "artefato externo que prova este marco" preenchido de antemão (protocolo STA, log do BCValidador, contrato, recibo) — o board inteiro nasce já amarrado à doutrina de evidência; kill criteria com data de avaliação agendada como reminder.
10. **Prospectar o piloto DRSAC**: contato com 3–5 IFs S1–S4 da lista nominal (via FBM/ABBC/consultorias — canais verificados no research/23 §4) para validar demanda do beta dez/2026. O relógio comercial começa antes do código: sem sinalização até 05/dez/2026, o §3.2 manda para o plano B automaticamente.

---

*Documento gerado em 16/nov/2026. Datas de normativos conforme research/06 e research/23 (verificadas em 16/nov/2026). Este roadmap vale até o gate N2 (14/fev/2027); re-ancoragem obrigatória nos kill criteria K1–K5 (tese v2 §7.3) nas datas respectivas — e re-ancoragem voluntária sempre que um marco fechar.*
