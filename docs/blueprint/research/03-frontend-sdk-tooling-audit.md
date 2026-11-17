# Auditoria 03 — Frontend, SDKs, Tooling e Pipeline de Dados

> **Produto:** Radiant Norma (Fortvna Risk Solutions) · **Versão auditada:** v3.36.4
> **Escopo:** `frontend/`, `sdk/{py,python,go}`, `tools/{xsdgen,norma-audit}`, `.github/`, `_catalogos/`, `scripts/`, `docker-compose.yml`, `backend/internal/db/migrations/`, `docs/openapi/`
> **Data:** 2026-11-16 · **Método:** leitura de código-fonte com evidência de arquivo:linha, contagens via `wc`/python, cruzamento OpenAPI ↔ handlers Go ↔ SDKs

---

## 0. Sumário executivo

1. **O frontend é real e bem construído (10.759 LOC TS/TSX)** — 11 rotas de página, 6 delas alimentadas por endpoints reais do backend Go, wizard de geração funcional de 6 passos (1 bug conhecido: passo de validação chama `/v1/validate` same-origin sem rewrite → 404 no browser), auth em 3 camadas (middleware Edge + server components + proxy com cookie httpOnly), design system fiel ao REDESIGN.md. Mas **não há um único teste** no frontend e há dependências placebo (`react-query`, `axios`, `zod`, `react-hook-form`, `zustand`, `recharts` declaradas, ~zero uso).
2. **Os SDKs existem em DUPLA e conflitante** — 2 gerações no mesmo repo: geração antiga (`sdk/python` + `sdk/go`, service-based, `api.radiantnorma.com`) e geração nova ("auto-generated from OpenAPI v3.36.2": `sdk/py` + `sdk/go/radiant`, flat, `api.radiantrisk.com/v1`). Os dois pacotes Python se chamam `radiant-norma` no PyPI (colisão), os dois módulos Go se chamam `github.com/fortvna/radiant-norma-go` (colisão). **Bug crítico:** a SDK nova manda `{"documents": [...]}` no `/v1/crossdoc/validate` porque copiou a spec OpenAPI — o backend Go exige `{"cadocs": {map}}`; a SDK antiga está certa.
3. **`_catalogos/` é de fato o ativo mais valioso do produto, mas é ad hoc** — `criticas.json` com **1.099 regras** reais de 6 CADOCs e `leiautes.json` com **4.244 linhas de leiaute** de 8 CADOCs, porém **schema JSON heterogêneo por CADOC** (chaves acentuadas diferentes: `descrição` vs `descrição da crítica` vs `descrição da regra`), 11 códigos duplicados, README desatualizado e pipeline de extração (`extract.py`) com paths hardcoded para `~/Downloads/cadocs`. É portável para Rust **depois de normalizar o schema**.
4. **OpenAPI duplicada e desalinhada** — `backend/docs/api/openapi.yaml` (3.0.3, v2.0.0, 15 paths, prefixo `/v1/*`, morta) convive com `docs/openapi/v1.yaml` (3.1.0, v3.36.2, 46 paths). A v1.yaml cobre ~46 de ~70 rotas reais e está **errada no corpo do crossdoc**. O SDK "gerado" herdou o erro.
5. **CI real só para o backend Go** — 1 workflow (`test.yml`, 174 linhas) com vet/gofmt/race/coverage-gates/drift-check de regras. Zero CI para frontend, SDKs, ou release.
6. **Conectores/ingest:** só o **Manual (wizard web) e o parse File (CSV/XLSX→CanonicalDocument via `/v1/generate/file/parse`) funcionam**; File-adapter-Fetch, API, DB e MCP são stubs honestamente rotulados `ErrNotImplemented` (a própria UI `/console/ingest` admite).

---

## 1. Frontend — mapa completo

### 1.1 Estatísticas

| Métrica | Valor |
|---|---|
| LOC TypeScript/TSX (`src/`) | **10.759** (79 arquivos) |
| Rotas de página (App Router) | 11 |
| Route handlers (`app/api/**`, `app/v1-api/**`) | 9 |
| Componentes (`components/`) | 33 (9 domain, 5 landing, 4 layout, 10 ui, 5 misc) |
| Testes | **0** (nenhum `*.test.*`, nenhum e2e, nenhum playwright/vitest/jest no package.json) |
| Dependências runtime | 13 (Next 15, React 19, @tanstack/react-query 5, axios, jose, lucide, recharts, zod, zustand, react-hook-form, date-fns, clsx, tailwind-merge) |

### 1.2 Mapa de rotas (páginas)

| Rota | Arquivo | LOC | Tipo | Backend real? |
|---|---|---|---|---|
| `/` | `src/app/page.tsx` (28) | — | Landing pública (Hero, Features, HowItWorks, CTA, Footer) | estático (mockup visual do produto) |
| `/login` | `src/app/login/page.tsx` (371) | 371 | Login dev/demo; POST `/api/login` → backend `/v1/auth/dev-token`; **404 hardcoded em prod** (IdP externo esperado, não implementado) | SIM (dev) |
| `/console` | `src/app/console/page.tsx` (477) | 477 | Dashboard executivo — 6 fetches server-side paralelos (`Promise.allSettled`): `/v1/radar/alerts`, `/v1/rules`, `/v1/schemas`, `/v1/envios/stats`, `/v1/insights/kpis`, `/v1/audit_log` | SIM (6 endpoints) |
| `/console/generate` | `src/app/console/generate/page.tsx` (380) | 380 | Catálogo dos 10 generators (1 ready: 3040) — dados **hardcoded** no TSX (`CADOCS[]`); `totalGenerated = 0` com TODO (endpoint `/v1/generate/stats` não existe) | NÃO (estático) |
| `/console/generate/wizard` | `wizard/page.tsx` + `wizard-client.tsx` | 1037 | **Wizard real de 6 passos**: upload→mapear→preview→validar→gerar→STA | PARCIAL (1 bug, §1.4) |
| `/console/ingest` | `src/app/console/ingest/page.tsx` (282) | 282 | Fachada dos 5 conectores (1 ready, 4 stub) — lista hardcoded; botão "Configurar" do Manual **sem onClick/href** (morto) | NÃO (estático) |
| `/envios` | `src/app/envios/page.tsx` (388) + `filter-bar.tsx` (198) | 586 | Lista de envios STA com filtros URL-driven, export CSV/JSON, stats | SIM |
| `/radar` | `src/app/radar/page.tsx` (240) | 240 | Alertas regulatórios BACEN, resolve via client | SIM |
| `/auditoria` | `src/app/auditoria/page.tsx` (293) + `filter-bar.tsx` (156) | 449 | Audit log tamper-evident, exibe `chain_valid` do backend | SIM |
| `/insights` | `src/app/insights/page.tsx` (377) | 377 | KPIs, heatmap (recharts não usado — heatmap é CSS), top-failing rules, recommendations | SIM (4 endpoints) |
| `/regras` | `src/app/regras/page.tsx` + `regras-client.tsx` (488) | ~560 | Catálogo de regras: fetch `/v1/rules` com **fallback que parseia `docs/rules-3040-catalog.md` do disco com regex**; drill-down em modal; toggle persiste | SIM |

### 1.3 Route handlers (BFF do Next)

| Handler | Função | Proxy target |
|---|---|---|
| `POST /api/login` | Dev login → `POST /v1/auth/dev-token` no backend; seta cookie `rn_jwt` httpOnly; **404 em NODE_ENV=production** | backend |
| `POST /api/generate/wizard` | Multipart upload → `POST /v1/generate/file/parse` | backend |
| `POST /api/generate/xml` | CanonicalDocument → `POST /v1/generate/{cadoc}` | backend |
| `POST /api/sta/submit` | → `POST /v1/sta/submit` | backend |
| `POST /api/rules/[code]/toggle`, `GET /api/rules/disabled` | Toggle de regras | backend |
| `POST /api/radar/alerts/[id]/resolve` | Resolve alerta | backend |
| `GET/POST /v1-api/proxy/[...path]` | Proxy genérico que injeta `Authorization: Bearer` do cookie | backend |
| `GET /v1-api/events/stream` | SSE do backend (`/v1/events/stream`) | backend |

### 1.4 Autenticação (3 camadas reais, qualidade alta)

1. **Edge middleware** (`src/middleware.ts`, 151 LOC): protege tudo exceto `/login`, `/api/login|logout`, `/_next`; checa presença/shape do JWT (3 segmentos); **bloqueia cookies `dev:` em produção**; gera **CSP com nonce por request** (`script-src 'self' 'nonce-…'`), `connect-src` para `RADIANT_API_URL` com ws/wss.
2. **Server components** (`src/lib/session.ts` + `auth-server.ts`): `getServerSession()` — dev cookie parse ou `verifyJwtServer` via `jose` com chave pública (`frontend/.env.pubkey`); fail-closed para `dev:` em prod (mesmo gate do backend).
3. **Client**: JS nunca lê o cookie httpOnly (comentário F27.4 em `lib/api.ts`); mutações client passam pelo proxy que injeta o header server-side.

**Bug real encontrado no wizard** (`src/app/console/generate/wizard/wizard-client.tsx:554`): o passo 4 (Validar) faz `fetch('/v1/validate', …)` — request **same-origin** ao próprio Next.js. Não existe route handler `/v1/*` no frontend e `next.config.js` **não define rewrites**. O middleware até deixa passar (`PROTECTED_API = /^\/(v1-api|v1)\//`), mas o Next responde 404 → **o passo "Validar" do wizard nunca funciona** end-to-end no browser (geração e parse funcionam pois usam `/api/*`). Em contraste, os outros passos usam handlers `/api/*` corretos.

**Menor:** o estado do wizard persiste só o `step` na URL (comentário promete "refresh não perde estado", mas `parsedDoc` vive em `useState` — refresh no passo 3+ renderiza nada); o XML passa entre passos via `(window as any).__wizardXml` (hack global).

### 1.5 Estado global e fetching

- **Estado de servidor**: fetch em server components via `apiFetch()` (`lib/api-fetch.ts`, 49 LOC) que lê o cookie via `next/headers` e injeta `Authorization` + bridge `X-IF-ID` para dev. `Promise.allSettled` com tolerância a falha parcial (páginas renderizam mesmo com endpoint down).
- **Estado client**: `useState` local + `localStorage` (perfis de mapeamento do wizard em `radiant:mapping_profiles`) + URL (filtros de `/envios`, `/auditoria`, step do wizard). **Não há zustand em uso.**
- **Realtime**: `lib/use-event-stream.ts` (227 LOC) — hook SSE próprio com reconexão exponencial (1s→30s), heartbeat timeout 60s, filtros por `kind`, status state; usado por 4 componentes `*-live-refresh.tsx` (dashboard, radar, envios, auditoria). Real e bem feito.
- **Placebo de dependências**: `react-query-provider.tsx` monta `QueryClientProvider`, mas **não existe nenhum `useQuery/useMutation` no codebase** (grep confirma só um comentário). `axios` (`lib/api.ts`) não é importado por nenhum arquivo. `zod`, `react-hook-form`, `zustand`, `recharts` declarados no package.json sem uso real (regras/heatmap usam forms nativos e CSS). ≈ 6 dependências mortas.

### 1.6 Design system — REDESIGN.md vs real

O `REDESIGN.md` (305 linhas) descreve "Institutional Premium Noir". Verificação ponto a ponto:

| Promessa (REDESIGN.md) | Real no código | Verdict |
|---|---|---|
| Tokens CSS variables (surface/ink/border) light+dark | `globals.css` implementa exatamente (com paleta v3.36 Midnight Blue→Platinum já atualizada no CSS, embora o §2 do doc ainda mostre violet) | REAL |
| Escala tipográfica `text-2xs`…`display-lg` | `tailwind.config.ts` replica a tabela | REAL |
| Sombras utilitárias + glow (accent/critical/success/warning) | implementadas | REAL |
| Motion 180–240ms `cubic-bezier(0.16,1,0.3,1)` | tokens `duration-*` + `transitionTimingFunction.out-expo` + 12 keyframes | REAL |
| Tipografia Inter/Fraunces/JetBrains Mono | **NÃO** — `layout.tsx` usa fallbacks de sistema (`Georgia`, `SF Mono`) porque `next/font/google` não funciona no ambiente (comentário Sprint 34). O look "editorial serif" degrada para Georgia | PARCIAL (gap documentado) |
| `Button` com `asChild` polimórfico | `ui/button.tsx` (141 LOC) implementa | REAL |
| Componentes Card/Badge/EmptyState/Skeleton/Kbd/Tooltip/Divider | 10 primitivas em `components/ui/` | REAL |
| Command palette (⌘K) | `layout/command-palette.tsx` (395 LOC) — busca regras/alertas/schemas/páginas reais, navegação por teclado, `aria` | REAL |

Acessibilidade: 58 ocorrências de `aria-*`/`role`, `focus-visible` ring global, `prefers-reduced-motion` respeitado em `globals.css`, modal do wizard com `use-focus-trap`. Sem auditoria automatizada (axe/vitest-axe ausente).

i18n: **não existe** — strings PT-BR hardcoded (adequado para o mercado-alvo; `date-fns/locale` ptBR e `Intl.NumberFormat('pt-BR')` para números).

Qualidade TS: `typedRoutes: true`, `reactStrictMode`, script `type-check`, eslint-config-next. Comentários são densos e referenciam sprints/validações (rastreabilidade excepcional). Código server/client corretamente separado (`'use client'` só em islands).

---

## 2. SDKs — 4 pacotes, 2 gerações, conflitos

### 2.1 Inventário

| | `sdk/py` (geração NOVA) | `sdk/python` (geração ANTIGA) | `sdk/go/radiant` (NOVA) | `sdk/go` (ANTIGA) |
|---|---|---|---|---|
| LOC client | 536 (`client.py`) | 224 (`client.py`) | 507 (`client.go`) | 246 (`client.go`) |
| Models | pydantic v2, 441 LOC | dataclasses, 98 LOC | structs + `go.sum` | structs, 96 LOC |
| Testes | 495 LOC (`responses`) | 155 LOC | 186 LOC (httptest) | 187 LOC |
| Versão | 3.36.2 | 1.0.0 | — | 1.0.0 (VALIDATION.md Sprint 60) |
| Base URL default | `https://api.radiantrisk.com/v1` | `https://api.radiantnorma.com` | `https://api.radiantrisk.com/v1` | `https://api.radiantnorma.com` |
| Autor/URL | "Radiant Risk" / quant-risk | "Radiant Financial Technology" / fortvna | fortvna | fortvna |
| Design | flat: 40 métodos (`c.healthz()`, `c.generate_cadoc(...)`) | serviços: `c.cadocs.validate()`, `c.radar.scan()` | flat, header: "Auto-generated types from OpenAPI v3.36.2" | serviços espelhando `sdk/python` |
| Package/module name | **`radiant-norma`** | **`radiant-norma`** (colisão PyPI!) | **`github.com/fortvna/radiant-norma-go`** | **`github.com/fortvna/radiant-norma-go`** (colisão de module path, dois `package radiant`) |

Ambos os pares são SDKs completos e independentes para a MESMA API — não é `py → python` rename, é duplicação de manutenção (READMEs, VALIDATION.md e testes duplicados).

### 2.2 Divergências contra o backend real (evidência)

1. **Crossdoc — BUG CRÍTICO na geração nova.** Backend: `backend/internal/crossdoc/engine.go:24` — `ValidationRequest.Cadocs map[string]string json:"cadocs"`; handler exige `{"cadocs":…}` senão 400 ("cadocs object required"). A spec `docs/openapi/v1.yaml:683-721` documenta **errado** `{"documents": [CrossDocInput]}` — e `sdk/py` (`client.py:250-262`) + `sdk/go/radiant` implementam a spec → **`crossdoc_validate()` quebra sempre contra o backend real**. A SDK antiga `sdk/python` (`client.py:111-124`) manda `{"cadocs": {...}}` e está **correta**.
2. **Radar scan** — `sdk/python` faz `POST /v1/radar/scan?cadoc=X` (query param). O handler `server.go:982` **ignora o corpo/param** e exige `AdminAuth.IsAdmin` → qualquer JWT de IF comum toma 401. Ambas as SDKs expõem uma chamada que na prática é admin-only e per-cadoc inexistente.
3. **`/v1/rules/{cadoc}` response shape** — `sdk/python` espera `{"rules": [...]}` (correto, `server.go:663-668` retorna map com chave `rules`); consistente.
4. Nenhuma SDK tem **retry/backoff**, **streaming** (SSE `/v1/insights/ask/stream` e `/v1/events/stream` não expostos), nem idempotency-key. `sta_submit`/`sta_situacao`/`sta_disponiveis` retornam `Dict[str, Any]` cru (sem model).
5. Endpoints cobertos pela SDK nova mas **ausentes da SDK antiga**: generate (fields/adapters/batch/history), sta (3), webhooks (5), marketplace (4), l4/compare, envios, audit_log, insights (3). A antiga tem `insights.ask` (que a nova **não tem**) e `schemas/{cadoc}/changelog`.

### 2.3 Cobertura de endpoints — gap matrix (resumo)

Backend expõe ~**70 handlers** em `/v1` (server.go:195-395). OpenAPI v1.yaml documenta 46 paths. SDK nova cobre ~40 operações. Não cobertos por SDKs nem pela spec: wizard (4), `sta/range/*` (3), `insights/ask`, `insights/ask/stream`, `insights/history` (GET+DELETE), `tenant/branding` (3) + `admin/tenant/{id}/branding`, `/events/stream`, `pilot/*` (8), `admin/soc2/*` (3), `admin/schemas`, `schemas/{cadoc}/versions/{versionId}`, `auth/dev-token`, `generate/{cadoc}/sources`, `rules/disabled`, `rules/{code}/toggle`, `insights/recommendations(+ack)`. Detalhe da matriz na §8.

---

## 3. Tools

### 3.1 `tools/xsdgen` (210 LOC, Go)

Gera XSD do 3040 a partir de `leiautes.json` (Sprint 2/T.1): mapeia tipos BACEN `N*` → `xs:decimal`, `A*` → `xs:string`; detecta grupos "Para cada cliente (Cli)" e tenta montar `complexType`. **Qualidade baixa/primitiva** (auto-declarada no output: "NÃO É OFICIAL — BACEN só publica XSD de 3045 e 3026"): o XSD gerado (`_catalogos/3040_generated.xsd`, 151 `xs:element`) declara atributos **soltos no top-level** do schema (não vinculados aos elementos), `complexType` `Cli` recursivo mal-formado (`Cli` contém `Cli*` dentro de si e o elemento também é emitido solto), quase tudo `xs:string`. **Na prática não valida XML real de 3040.** O backend usa este XSD para o L1 do 3040 (`backend/internal/audit/xsd_validator.go:20-25`) com um "practical subset" próprio — ou seja, a validação XSD de 3040 é muito mais fraca do que o marketing sugere.

### 3.2 `tools/norma-audit` (198 LOC, Go)

CLI Sprint 2/T.2: roda B01 (XML parseia), B04 (declaração de encoding), B05 (não-vazio) sobre um XML de exemplo; **B02/B03 são SKIP hardcoded**; carrega `criticas.json` só para exibir metadados. Artefato histórico de prova de conceito — superseded pelo `internal/audit/rules` do backend (51 arquivos, centenas de regras portadas).

---

## 4. CI (.github)

Existe **um único workflow**: `.github/workflows/test.yml` (174 linhas) — backend Go only:

- `go vet`, `gofmt -l` check, build de **todos** `cmd/*/` via glob, `go test -race`, coverage com **gates reais** (auditlog ≥85%, radar ≥70%, audit/rules ≥70%), **placeholder lint** de SPRINT_*.md (`scripts/lint-no-placeholder.sh`), **drift check**: número de regras 3040 no registry Go vs claim no CHANGELOG.md (parse em python3 inline).
- Hardening documentado (Validação 54): `permissions: contents: read`, timeout 15min, `concurrency.cancel-in-progress`, cache de modules.
- Roda em **`macos-latest`** (incomum e caro; justificado por dev em macOS + SQLite).
- **Não cobre**: frontend (lint/type-check/build), SDKs (pytest/go test dos 4 pacotes — os testes existem e nunca rodam em CI), xsdgen/norma-audit, OpenAPI lint/drift, release/publish.

---

## 5. `_catalogos/` — o ativo de dados crítico

### 5.1 Inventário

| Arquivo | Tamanho | Conteúdo |
|---|---|---|
| `criticas.json` | 544 KB | **1.099 regras** de validação, 6 CADOCs |
| `leiautes.json` | 1,1 MB | **4.244 linhas** de leiaute, 8 CADOCs |
| `leiautes_3044.json` | 8,6 KB | Schema JSON estruturado do 3044 (documento + 5 objetos + 17 regras + exemplo mínimo) |
| `3040_generated.xsd` | 30 KB | XSD gerado pelo xsdgen (primitivo, não-oficial) |
| `_debug_criticas_3040.json`, `_debug_*` | — | intermediários de extração (3 arquivos) |
| `drm_criticas_raw.json` | — | raw DRM |
| `extract.py` (332 LOC), `extract_3044.py` (8,3 KB), `extract_drm.py` (2,8 KB) | — | extração de XLS/XLSX → JSON |
| `README.md` | — | documenta **estado antigo** (ver §5.4) |

### 5.2 `criticas.json` — formato e cobertura

```
{_metadata: {descricao, fonte, data_extracao: 2026-07-03, autor: "Mavis · Radiant", uso, total_cadocs: 6, total_criticas: 1099},
 criticas: {"<CADOC>": [<regra>, ...]}}
```

| CADOC | Regras | Fonte |
|---|---|---|
| 3040 (SCR) | 361 | `3040/SCR3040_Criticas.xls` |
| 3050 (TXB) | 170 | `3050/Criticas_TXB_V*.xlsx` |
| 2061-DLO | 518 | `_referencias/Criticas_Processamento_2061_2071.xlsx` |
| 2060-DRM | 22 | PDF `Criticas_Pos_Processamento_2060_V2_Jan25.pdf` |
| 2070-DDR | 11 | `2070-DDR/2070_DDR_Criticas.xlsx` |
| 3044 | 17 | Manual SCR Doc3044 (jul/2025) |
| **Total** | **1.099** | bate com o número "1.099 regras" citado na UI |

### 5.3 Qualidade — os riscos concretos

1. **Schema ad hoc, heterogêneo por CADOC.** Não há um formato único de "regra". Cada bloco é um dump cru da planilha de origem com chaves diferentes:
   - 3040: `codigo`, `habilitado?`, `regra`, **`descrição`** (acentuada)
   - 3050: `codigo`, **`descrição da regra`**, `observações` — e código `2001` repetido dezenas de vezes (célula mesclada)
   - 2061: `codigo`, **`crítica`**, **`descrição da crítica`**, `data-base inicio`
   - 2070: `codigo`, `doc`, `tipo`, **`descrição`**
   - 2060/3044: único bloco com chaves canônicas ASCII (`codigo`, `tipo`, `descricao`, `base_confrontada`, `fonte`)
   Consequência: a análise de campos canônicos mostra `descricao` vazia em **1.079/1.099** registros e `tipo` vazia em 1.066 — o texto real está nas chaves acentuadas alternativas. Qualquer consumidor precisa de 5 parsers diferentes (o backend Go consome via `map[string]any` no `cmd/seed`, mascarando o problema).
2. **11 códigos duplicados** por (cadoc, codigo) — ex. 3040 `41051` ×4, 3050 `2001` ×N.
3. **Sem versionamento nem schema formal** — não existe JSON Schema/Avro para `criticas.json`; `_metadata.data_extracao` é a única marca temporal.
4. **Pipeline não-rodo no repo**: `extract.py:10-11` hardcode `ROOT = Path("/Users/henrique/Downloads/cadocs")` — as planilhas-fonte ESTÃO versionadas no repo (`3040/`, `3050/`, `2030-DRSAC/`, `2062-DLI/`, `2070-DDR/`, `_referencias/`), mas o script lê de fora. Re-extração exige correção de paths (portar para Rust é direto: xlrd/openpyxl → `calamine`).
5. **README desatualizado**: diz "1.081 regras de 4 CADOCs" e "3050: 191" — o arquivo real tem 1.099 de 6 e 3050 com 170.
6. `leiautes.json` é **raw rows** (`{sheet, values[]}` — linhas cruas da planilha, inclusive cabeçalhos), não campos parseados; quem interpreta é o xsdgen/seed. O único leiaute em formato rico é o **3044** (campos nomeados com tipo/tamanho/obrigatoriedade/exemplo — o melhor artefato do diretório, pronto para um schema registry).
7. Consumidores reais no backend: `cmd/seed/main.go` (popula o Postgres) e `internal/audit/xsd_validator.go` (mapa de XSDs: 3050 → XSD **oficial** BACEN `3050/3050_Schema_TXB_V4.xsd`, 3045 → `SCR3045.xsd`, 3040 → XSD gerado, demais → fallback root-tag).

### 5.4 Avaliação para o port

O conteúdo (as regras e os leiautes em si) é o diferencial competitivo real e **porta direto para Rust como dados**. Mas a camada de formato precisa ser reconstruída: definir schema canônico (`{cadoc, codigo, severidade, descricao, base, fonte, vigencia}`), resolver duplicatas, unificar chaves acentuadas, e re-extrair das XLS versionadas com parser determinístico. Hoje é um "Excel em JSON", não um catálogo de regras consultável.

---

## 6. Periferia operacional

- **`_gen_pdfs.py` (380 LOC)**: gera 3 PDFs de marketing/estratégia (README, ENG_REVERSA, PRODUTO_TESE) via chrome-headless-shell com path hardcoded em `~/Library/Caches/ms-playwright/...` e `ROOT=/Users/henrique/Downloads/cadocs`. Artefato de conveniência pessoal — **não portar**.
- **`scripts/`**: `install-hooks.sh` (39), `pre-commit.sh` (71), `lint-no-placeholder.sh` (72, usado pelo CI — real).
- **`docker-compose.yml` (62)**: postgres:16-alpine com healthcheck + serviços `api`/`worker` sob `profiles: ["prod"]`. Suficiente para dev local.
- **Migrations `backend/internal/db/migrations/001–026`** (26 arquivos SQL, cada um documentado com sprint): 001 schema inicial → 002 xml_content em envios → 003-004 radar (no-unique, baselines) → 005 worker retry/lease → 006 insights/envios enriquecimento → 007-008 disabled_rules (+CHECK) → 009 recommendations ack → 010 FKs tenant → 011 índices envios → 012 RLS policies → 013 CHECKs envios → 014 FORCE RLS → 015 Stripe billing → 016 white label → 017 DRSAC research → 018 insights conversations (LLM) → 019 webhooks → 020 marketplace → 021 multiregion → 022 pilot4 onboarding → 023 SOC2 evidence → 024 STA range sessions → 025 wizard sessions → 026 RLS extended. SQL Postgres puro, portável direto (incl. policies RLS).

---

## 7. OpenAPI — 2 specs, 1 viva

| | `backend/docs/api/openapi.yaml` | `docs/openapi/v1.yaml` |
|---|---|---|
| Versão OpenAPI | 3.0.3 | 3.1.0 |
| `info.version` | 2.0.0 | **3.36.2** (bate com CHANGELOG) |
| Paths | 15 (todos `/v1/*`) | 46 (sem prefixo; server URL contém `/v1`) |
| Servidores | localhost + `api.radiant-norma.fortvna.com.br` (placeholder) | `api.radiantrisk.com/v1` |
| Estado | **Morta** (não acompanha o backend desde ~v2.0) | Viva, "auto-generated from handler signatures (Sprint 77)" |

Problemas da v1.yaml (viva):
1. **Prefixo misto**: 43 paths sem `/v1` mas 4 COM (`/v1/marketplace/{id}/install`, `/v1/marketplace/{id}/rate`, `/v1/marketplace/installed`, `/v1/webhooks/{id}` + deliveries) → resolução dupla `/v1/v1/...`.
2. **~30 rotas reais ausentes** (ver matriz §8) — wizard, STA range, LLM ask/stream/history, branding, pilot, SOC2, admin, events/stream, dev-token, `versions/{versionId}`, `generate/{cadoc}/sources`, rules toggle/disabled, recommendations ack.
3. **Corpo do crossdoc errado** (§2.2) — a fonte do bug da SDK nova.
4. O `frontend/package.json` tem script `generate-api-types` apontando para `../backend/docs/api/openapi.yaml` — mas `src/lib/api-types.d.ts` **nunca foi gerado** (arquivo não existe) e o frontend usa interfaces manuais.

---

## 8. Gap matrix OpenAPI ↔ handlers ↔ SDKs

Legenda: S=v1.yaml documenta · H=handler Go existe · P=`sdk/py`+`sdk/go/radiant` (nova) · A=`sdk/python`+`sdk/go` (antiga) · ✓/✗

| Domínio | Endpoint | Spec | Handler | SDK nova | SDK antiga | Obs |
|---|---|---|---|---|---|---|
| Health | healthz/readyz | ✓ | ✓ | ✓ | ✓ | |
| Schemas | GET schemas, /schema, {cadoc}, versions | ✓ | ✓ | ✓ | parcial (versions) | changelog e versions/{id} fora da SDK nova |
| Rules | GET rules, rules/{cadoc} | ✓ | ✓ | ✓ | ✓ | |
| Rules | disabled + toggle | ✓ | ✓ | ✗ | ✗ | só o frontend usa |
| Validate | POST /validate | ✓ | ✓ | ✓ | ✓ | |
| Generate | POST {cadoc}, fields, adapters, batch, history, file/parse | ✓ (menos sources) | ✓ | ✓ | ✗ | sources ausente da spec |
| Wizard | start/get/advance/xml | ✗ | ✓ | ✗ | ✗ | 4 handlers fora da spec |
| Crossdoc | rules + validate | ✓ **corpo errado** | ✓ (exige `cadocs`) | **✗ quebra** | ✓ | bug §2.2 |
| STA | submit, disponiveis, situacao | ✓ | ✓ | ✓ | submit+disponiveis apenas (antiga) | |
| STA | range/init, {protocolo} PUT/GET | ✗ | ✓ | ✗ | ✗ | chunked upload §5.6 |
| Radar | alerts, {id}, resolve, scan | ✓ | ✓ | ✓ | scan (shape errado, admin-only) | |
| L4 | l4/compare | ✓ | ✓ | ✓ | ✗ | |
| Envios | list + stats | ✓ | ✓ | list ✓ stats ✗ | ✗ | |
| Audit | audit_log | ✓ | ✓ | ✓ | ✗ | |
| Insights | kpis, heatmap, top-failing | ✓ | ✓ | ✓ | ✗ | |
| Insights | recommendations + acknowledge | ✓ | ✓ (2 verbs) | ✗ | ✗ | |
| Insights | ask POST, ask/stream GET, history GET/DEL | ✗ | ✓ | ✗ | ask ✓ (POST /v1/insights/ask) | LLM real no backend |
| Events | events/stream (SSE) | ✗ | ✓ | ✗ | ✗ | só o frontend |
| Branding | tenant/branding ×3 + admin | ✗ | ✓ | ✗ | ✗ | white label |
| Webhooks | CRUD + deliveries + retry | ✓ (prefixo misto) | ✓ | ✓ | ✗ | |
| Marketplace | list/publish/install/rate/installed | ✓ (prefixo misto) | ✓ | ✓ | ✗ | |
| Pilot | programs/participants/steps/esg ×8 | ✗ | ✓ | ✗ | ✗ | |
| SOC2 | readiness/controls/evidence | ✗ | ✓ | ✗ | ✗ | |
| Admin | admin/schemas | ✗ | ✓ | ✗ | ✗ | |
| Auth | dev-token | ✗ | ✓ (gated) | ✗ | ✗ | usado pelo frontend |
| Metrics | metrics, readiness | ✗ | ✓ | ✗ | ✗ | ops |

Cobertura: spec ≈ 46/70 (66%) · SDK nova ≈ 40 ops · SDK antiga ≈ 12 ops.

---

## 9. Superfície de produto real — o que um usuário faz HOJE de ponta a ponta

**Funciona de verdade (browser → Next BFF → backend Go → Postgres):**
1. Landing → login dev (if_id+role → dev-token JWT RS256 → cookie httpOnly).
2. Dashboard com números reais de envios/alertas/regras/audit (+2 placebos: "cobertura de schema" hash-based).
3. **Gerar CADOC 3040 via wizard**: upload CSV/XLSX → parse server-side para CanonicalDocument → mapeamento de colunas (com perfil salvo em localStorage e auto-match) → preview → **[passo Validar quebrado — 404]** → gerar XML (motor 3040 real, sha256) → download XML ou "enviar ao BACEN" via `/v1/sta/submit` (backend STA é **stub** — aceita e registra, não há conexão real ao STA).
4. Radar: ver alertas BACEN detectados (crawler real no worker) e resolvê-los.
5. Envios: histórico filtrável (cadoc/status/período), export CSV/JSON, stats com delta.
6. Auditoria: audit log hash-chained com `chain_valid`, filtros, export.
7. Insights: KPIs comparativos, heatmap de erros por hora/dia, top regras reprovadas, recommendations (heurísticas backend; LLM `/ask` existe na API mas **não tem UI**).
8. Regras: catálogo navegável, drill-down, disable/enable persistido por IF.
9. Tempo real: SSE com auto-reconnect refrescando as 4 páginas operacionais.

**Não funciona (stub/planned, honestamente rotulado na UI):**
- Conectores File-API/REST-API/DB/MCP (`ErrNotImplemented` no backend; UI mostra badge "planejado"; botão "Configurar" do Manual é morto).
- 9 dos 10 generators (3050/3044/2061/2062/2060/2070/2160/2170/4111) — regras catalogadas, motors inexistentes.
- Envio real ao STA do BACEN (`staSubmit` stub; `sta/range` existe mas sem UI/SDK).
- Login de produção (IdP externo — endpoint dev 404 em prod, sem fluxo OIDC implementado).
- Wizard de geração para os outros CADOCs (frontend só oferece 3040/3050 no select; 3050 sem motor).

---

## 10. O que porta direto para Rust vs artefato de conveniência

**Porta direto (valor real, preserve o conteúdo):**
- `_catalogos/criticas.json` + `leiautes.json` + `leiautes_3044.json` — após normalização de schema (§5.3); fontes XLS estão versionadas no repo.
- XSD **oficiais** BACEN: `3050/3050_Schema_TXB_V4.xsd`, `3040/SCR3045.xsd` (+ PDFs manuais como referência normativa).
- `backend/internal/db/migrations/001–026` — SQL Postgres puro, roda em qualquer stack.
- `docs/openapi/v1.yaml` como ponto de partida da spec (corrigir crossdoc, prefixos, e completar ~30 rotas a partir do `server.go:195-395`).
- O formato da API em si (payloads/verbos/erros) — o `server.go` é a fonte da verdade; SDKs novas (sdk/py + go/radiant) são modelos de superfície de API razoáveis após corrigir crossdoc.
- Padrões do frontend: tokens de design (REDESIGN.md + tailwind.config + globals.css), arquitetura middleware/proxy/SSE, e o fluxo do wizard (mapeamento colunas→canônico com perfil por CADOC é a melhor UX do produto).
- `scripts/lint-no-placeholder.sh` (disciplina de docs) e o formato dos SPRINT/VALIDATION docs.

**Reescrever/migrar com cuidado:**
- `extract.py`/`extract_3044.py`/`extract_drm.py` → em Rust com `calamine`, paths relativos ao repo, output com JSON Schema versionado.
- `xsdgen` → em Rust com geração **correta** (atributos aninhados aos elementos, tipos A8/A10/N19,2, minOccurs por planilha) — hoje a saída não valida nada de fato.
- Regras de validação: o backend Go (`internal/audit/rules/`, 51 arquivos) é a implementação de referência das críticas — é isso que deve ser portado regra a regra com testes de paridade, usando criticas.json como checklist.
- CI: replicar os gates (coverage por pacote crítico, drift-check catálogo↔código) no novo pipeline.

**Artefato de conveniência (não portar):**
- `_gen_pdfs.py` + os PDFs gerados na raiz (marketing local, paths hardcoded).
- `tools/norma-audit` (prova de conceito Sprint 2, 3/5 regras).
- `backend/docs/api/openapi.yaml` (spec morta v2.0.0 — substituída pela v1.yaml).
- Uma das duas SDKs por linguagem (eliminar `sdk/python` + `sdk/go` OU `sdk/py` + `sdk/go/radiant` — recomenda-se consolidar na geração "flat" corrigindo o bug de crossdoc e o module/package name).
- Dependências mortas do frontend (react-query, axios, zod, react-hook-form, zustand, recharts).

---

## 11. Riscos principais (prioridade)

1. **SDK nova quebrada por causa da spec** (crossdoc `documents` vs `cadocs`) — qualquer cliente que seguir v1.yaml falha em L3. Corrigir spec + SDK + testar contra backend.
2. **Wizard passo "Validar" 404** (`fetch('/v1/validate')` same-origin) — a jornada hero do produto ("gera, valida e envia") tem o passo central morto no browser. Fix: usar handler BFF `/api/validate` ou rewrite.
3. **Catálogo sem schema formal** — 1.099 regras heterogêneas e duplicadas é o ativo e o maior risco de fidelidade regulatória (não dá para afirmar cobertura sem normalizar).
4. **Colisão de nomes de pacotes** (`radiant-norma` ×2 no PyPI; `github.com/fortvna/radiant-norma-go` ×2) — impede publicação e confunde consumidor.
5. **XSD 3040 gerado não valida** — L1 de 3040 é mais fraca do que o discurso de venda ("1.099 regras, 90% coverage" na UI não reflete a camada estrutural).
6. **Zero testes no frontend e CI só de backend** — regressões de UI/SDK não são detectadas.
7. **Placebos residuais na UI** (coverage hash-based, deadlines hash-based, `totalGenerated=0`, botão morto) — pequenos, mas minam confiança de um comprador regulatório que auditar a tela.

---

## Apêndice A — Contagens de referência

- Frontend: 10.759 LOC TS/TSX; maiores arquivos: `wizard-client.tsx` 1037, `regras-client.tsx` 488, `console/page.tsx` 477, `command-palette.tsx` 395, `envios/page.tsx` 388.
- SDKs: sdk/py 1.675 LOC (client 536 + models 441 + tests 495 + init 105 + exceptions 33) · sdk/python 632 LOC · sdk/go/radiant 880 LOC · sdk/go 716 LOC.
- Tools: xsdgen 210, norma-audit 198.
- CI: 1 workflow, 174 linhas.
- Catálogo: 1.099 críticas · 4.244 leiautes · 8+6 CADOCs · XSD gerado 151 elementos.
- Migrations: 26 SQL.
- Backend rotas /v1: ~70 handlers (server.go:195-395); OpenAPI v1.yaml: 46 paths; backend/docs/api/openapi.yaml: 15 paths (morta).

## Apêndice B — Evidências-chave (arquivo:linha)

- Bug crossdoc SDK nova: `sdk/py/radiant/client.py:250-262` + `docs/openapi/v1.yaml:683-721` vs `backend/internal/crossdoc/engine.go:23-31` e `backend/internal/api/server.go:1091-1096`.
- Bug wizard validate: `frontend/src/app/console/generate/wizard/wizard-client.tsx:554` + ausência de rewrites em `frontend/next.config.js`.
- Coverage placebo: `frontend/src/app/console/page.tsx:100-106` (`stableCoverage`) e `envios/page.tsx:81-88` (`nextDeadline`).
- Conectores stub: `frontend/src/app/console/ingest/page.tsx:8-14` (comentário) e backend `internal/generator/adapters` (ErrNotImplemented citado no próprio handler).
- STA stub: `backend/internal/api/server.go:250` ("STA submission (stub)").
- Catálogo heterogêneo: chaves por CADOC em `_catalogos/criticas.json` (blocos 3040/3050/2061/2070/2060/3044).
- Colisões de nome: `sdk/py/pyproject.toml` vs `sdk/python/pyproject.toml` (ambos `name = "radiant-norma"`); `sdk/go/go.mod` vs `sdk/go/radiant/go.mod` (ambos `module github.com/fortvna/radiant-norma-go`).
- XSD paths do backend: `backend/internal/audit/xsd_validator.go:20-25`.
- Extração com path externo: `_catalogos/extract.py:10-11`; `_gen_pdfs.py:5` e `:28`.
