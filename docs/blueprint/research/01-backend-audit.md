# 01 — Auditoria Técnica do Backend Go (Radiant Norma)

**Data:** 2026-11-16 · **Escopo:** `/backend` completo (Go 1.25) · **Objetivo:** spec de verdade para o port Rust
**Método:** leitura exaustiva dos 35 packages `internal/`, 13 entrypoints `cmd/`, migrações SQL; build + suíte de testes + race detector executados durante a auditoria; todas as afirmações têm evidência `arquivo:linha`.

---

## Sumário Executivo

O backend é **melhor do que o número de sprints sugere em infraestrutura** (audit log tamper-evident, RLS, STA WS real, worker com retry/dead-letter, schema registry versionado) e **pior do que a vitrine sugere no motor de validação**. A descoberta central, verificada por tracing de chamadas:

> **Só o CADOC 3040 tem validação L2 semântica real em produção.** As ~340 regras implementadas para 3050, 2070, 2061, 2160, 2170, 3044 e 2060 existem em Go, compilam, têm testes — mas **nunca são despachadas** em nenhum endpoint (§3.2). A camada L3 dentro de `ValidateFull` também é no-op (§3.4). O banco de críticas usa chaves (`2061-DLO`) que a API de 4 dígitos nunca consulta (§3.5).

Números-chave: **56.959 LoC de produção + 32.731 LoC de teste (1.178 funções de teste, 109 arquivos)**, build limpo, **suíte inteira passa** (`go test ./...` exit 0), race detector limpo nos packages críticos. Regras L2 são **código Go compilado** (não data-driven), mas o *loop de execução* é data-driven (tabela `criticas` no DB). Erros são tratados com disciplina incomum (sanitização `SafeError` em todos os handlers, recover de panic por camada). Duplicação significativa: registry com 9 quadruplets copiados, 10 generators com helpers idênticos, dois motores L1–L4 concorrentes com semânticas diferentes.

---

## 1. MAPA — LOC, dependências, entrypoints

### 1.1 Inventário

| Métrica | Valor | Fonte |
|---|---|---|
| Arquivos Go | 287 | `find backend -name "*.go"` |
| LoC produção (sem `_test.go`) | **56.959** | `wc -l` |
| LoC teste | **32.731** (109 arquivos, 1.178 funcs) | `wc -l` |
| Entrypoints `cmd/` | 13 (12 + `_verify`) | §1.3 |
| Packages `internal/` | 35 | `ls internal/` |
| Dependências externas diretas | 11 (chi, pgx/v5, go-redis, excelize, modernc sqlite, jwt, aws-sdk secrets, sync, miniredis) | `go.mod:3-15` |
| CGo / unsafe / reflection | **0 / 0 / 1 uso trivial** | grep |
| `go:embed` | **1** (migrations) | `internal/db/migrate.go:22` |

O número "29.5k LoC" que circulava está desatualizado — o backend real tem ~57k LoC de produção (o package `audit/rules` sozinho tem 24.7k).

### 1.2 LOC por package (produção + teste, ordenado)

| Package | LoC | Papel real | Cobertura de teste |
|---|---:|---|---:|
| `internal/audit` (total) | 27.542 | Norma Audit — L1–L4 | 25,1% |
| ├─ `audit/rules` | **24.696** | Regras L2 hardcode em Go | 61,6% |
| └─ `audit/l4` | 1.121 | Diff histórico | **0% (sem testes)** |
| `internal/api` | 14.380 | Handlers HTTP chi + wiring | 58,0% |
| `internal/generator` | 7.817 | 10 geradores + wizard + validação | 71–100% |
| `internal/sta` | 4.962 | Cliente STA BACEN (WS real) | 73,6% |
| `internal/drsac` | 2.705 | Parser+regras DRSAC 2030 | 43,5% |
| `internal/radar` | 2.677 | Monitor normativo + autopr | 78,4% |
| `internal/crossdoc` | 2.264 | Motor L3 cross-doc | 83,2% |
| `internal/ingest` | 2.106 | Adaptadores de fonte | 25,6% |
| `internal/auth` | 1.421 | JWT verifier/keyring | 71,8% |
| `internal/schema` | 1.399 | Schema Registry | 64,0% |
| `internal/billing` | 1.307 | Stripe | 10,8% |
| `internal/insights` | 1.155 | KPIs + LLM (MiniMax/OpenAI) | 8,3% |
| `internal/secrets` | 1.005 | AWS SM/env/memory | 58,3% |
| `internal/senhaws` | 961 | Rotação senha STA | 95,6% |
| `internal/realtime` | 959 | Hub SSE | 78,9% |
| `internal/db` | 926 | Dual driver + migrations + tenant tx | 62,7% |
| `internal/soc2` | 898 | Controles SOC2 | 0,8% |
| `internal/synth` | 790 | Gerador de dados sintéticos | 11,3% |
| `internal/webhook` | 783 | Dispatcher HMAC | 10,3% |
| `internal/worker` | 779 | Fila envios STA | 86,1% |
| `internal/auditlog` | 674 | Hash chain tamper-evident | 92,5% |
| `internal/ruleprefs` | 672 | Toggle regras por IF | 63,3% |
| `internal/docdli` | 657 | Parser 2062 | 92,2% |
| `internal/bacen` | 651 | Parsers tipados p/ cross-doc | **0% (sem testes)** |
| `internal/branding` | 620 | White label | 92,6% |
| `internal/tenant` | 579 | Lifecycle tenant | 90,2% |
| `internal/observability` | 521 | Sentry + OTel + metrics | 0% |
| `internal/pilot` | 494 | Onboarding | 3,3% |
| `internal/canonical` | 489 | Documento canônico | 72,2% |
| `internal/marketplace` | 410 | Marketplace de regras | 14,6% |
| `internal/doc4111` | 357 | Parser 4111 | 55,4% |
| `internal/multiregion` | 333 | Replicação BR-SP1/SP2 | 29,1% |
| `internal/version` | 68 | Build info | — |
| `cmd/` (13 mains) | 3.598 | Entrypoints | parcial |
| `radiant/` (raiz) | ~230 | SDK Go client | — |

### 1.3 Entrypoints `cmd/`

| Binário | Linhas | Função |
|---|---:|---|
| `api` | 409 | **Servidor principal** — wiring de todos os serviços (`cmd/api/main.go:100-220`) |
| `worker` | 148 | Processador de fila envios STA (valida → submete → retry/dead-letter) |
| `radar` | 123 | Scanner normativo standalone |
| `senhaws-rotate` | 1.350 | Rotação de senha STA (com testes) |
| `sta-submit` | 644 | CLI de submissão manual |
| `secret-migrate` | 610 | Migração de secrets |
| `seed` / `seed-demo` / `seed-sprint8c` | 345/507/371 | Carga de críticas/leiautes/demo |
| `jwt-mint`, `synth-gen`, `_verify` | 113/220/39 | Utilitários dev |
| `generator-server` | — | **Binário Mach-O compilado** commitado no repo (não é código) |

### 1.4 Grafo de dependências (produção, sem testes)

```
api ──> {audit, auditlog, auth, branding, canonical, crossdoc(+rules), generator(11 sub),
         ingest, insights, loggerutil, marketplace, pilot, radar, realtime, ruleprefs,
         schema, soc2, sta, version, webhook, worker}
audit ──> {audit/l4, audit/rules, bacen, docdli, drsac, loggerutil, observability}
audit/rules ──> {bacen, crossdoc, crossdoc/rules?, doc4111, drsac}      (XD rules)
generator/{gen*} ──> {canonical, schema, generator}
crossdoc ──> {bacen, doc4111, drsac, loggerutil}    (DocSet/Engine)
crossdoc/rules ──> {bacen, crossdoc, doc4111, drsac}
worker ──> {audit, auditlog, sta, loggerutil}
synth ──> {audit, insights}          radar ──> {loggerutil, version}
realtime ──> {auditlog, observability}   ruleprefs/db/testutil ──> db
```

**Conclusão:** o grafo é **acíclico** (DAG limpo). Os pacotes de domínio de baixo nível (`bacen`, `canonical`, `doc4111`, `docdli`, `drsac`) não importam nada interno; `audit` e `crossdoc` consomem-nos. O único ciclo potencial (`generator` ↔ `gen*`) foi deliberadamente evitado via `RegisterDefaults` com glue em `main.go` (`internal/generator/interface.go:163-204`, `cmd/api/main.go:126-140`). Nota: `audit/rules` importa `crossdoc` (para `DocSet`) enquanto `crossdoc` não importa `audit` — direção única, OK.

---

## 2. MOTOR DE VALIDAÇÃO — como funciona de verdade

Existem **dois motores L1–L4 paralelos com semânticas diferentes**:

| | Motor A — `internal/audit` (Norma Audit) | Motor B — `internal/generator/validation` |
|---|---|---|
| Onde roda | `POST /v1/validate`, `ValidateFull`, worker | `POST /v1/generate/batch` |
| L1 | Parse + XSD próprio (`xsd_validator.go`) | Apenas "XML bem-formado" (`validation/validate.go:219-229`) |
| L2 | Críticas do BACEN (DB) + registry Go | ~6 checks de campo obrigatório por CADOC |
| L3 | XD01–XD08 (ver §3.4: **no-op**) | crossdoc Engine real |
| L4 | Diff histórico vs envio anterior | — |

### 2.1 O fluxo real de `/v1/validate`

`api/server.go:709-780` (`validate`) → `audit.Service.Validate` (`audit/service.go:458-605`):

1. **L1**: `validateL1Parse` (`service.go:611-649`) — para JSON, `json.Unmarshal`; para XML, tenta `ValidateXSD` e cai para check de root tag (`expectedRootTag`, `service.go:651-675`). O XSD validator (`audit/xsd_validator.go:18-22`) só conhece **3 CADOCs** (3050, 3045, 3040) e carrega os schemas de **paths relativos ao disco** (`"../../3050/3050_Schema_TXB_V4.xsd"`) — em binário deployado esses paths não existem e o código **silenciosamente degrada para root-tag check** (`xsd_validator.go:36-40`). Não é uma lib XSD completa: é um subset escrito à mão (~480 LoC) que parseia `element/simpleType/complexType` do próprio XSD.
2. **L2**: `LoadCriticas` lê a tabela `criticas` do DB por `cadoc_code` (`service.go:172-212`) e itera cada linha chamando `applyRegra` (`service.go:690-760`). **O catálogo é data; a execução é código.**
3. Severidade: prioriza `rule.Severity()` do registry; senão `Gravidade` do DB; default "A" (`service.go:571-584`).

### 2.2 O dispatch de regras — o achado central

`applyRegra` (`audit/service.go:690-760`) tem **exatamente 3 rotas**:

```go
// 1ª: RawRule (B01–B05, XML bruto)                     — só 3040 tem
if rawRule := s.registry.GetRaw(c.Codigo); rawRule != nil { ... }
// 2ª: regra tipada *Doc3040 — só se is3040
if s.registry != nil && is3040 { rule := s.registry.Get(c.Codigo); ... }
// 3ª: DLI estrutural + regras 2062 — só se is2062
if is2062 { ... docdli.Validate(...); rule2062 := s.registry.Get2062(...) }
// Senão: return nil  ← regra silenciosamente ignorada
```

Consequências verificadas (grep sobre todo o código de produção):

| CADOC | Regras implementadas | Despachadas em produção? |
|---|---|---|
| **3040** | 275 no registry (`rules/registry.go:531-913`): 5 raw + 270 tipadas | **SIM** — única exceção |
| **2062** | 18 DLI (`rules/dli.go:654`) | Parcial — **só se houver críticas com cadoc_code="2062" no DB** (o seed não cria; ver §3.5) |
| **3050** | **170** (`rules/3050.go:3381 Builtin3050()`) | **NÃO** — `Builtin3050()` nunca é chamado (grep: só definição + comentários) |
| **2070** | **11** (`rules/2070.go:318 Builtin2070()`) | **NÃO** — idem |
| **2061** | 24 ELIM (`rules/dlo.go:788 BuiltinDLO(r)`) | Registradas no registry do 3040, mas `Get2061` **nunca é chamado** em produção (grep em `registry.go` + testes apenas) |
| **2160/2170** | 11 DRL + 10 NSFR | **NÃO** — idem (`Get2160`/`Get2170` sem chamadas) |
| **3044** | 17 T-rules (`registry.go:882-899`) | **NÃO** — `Get3044` sem chamadas |
| **2060** | 7 DRM (`registry.go:911`) | **NÃO** — `GetDRM` sem chamadas |
| **2030** | Engine próprio do `drsac` (rules.go 801 LoC + validator.go 294 LoC) | **NÃO** — `drsac.ValidateDocument` sem chamadores externos; `drsac.IntegrateWithAudit()` (`drsac.go:169`) é um no-op de wiring que ninguém chama |

Ou seja: **das ~511 regras Go implementadas além do 3040/2062, zero rodam** no endpoint de validação. O 3040 roda de verdade (com os stubs abaixo). Impacto no port Rust: **as regras Go são spec valiosa, mas o wiring atual não é a verdade do produto — o port deve corrigir, não replicar.**

### 2.3 Regras: código, não data — anatomia

Cada regra é um struct vazio implementando interface:

```go
// internal/audit/rules/registry.go:154-159
type Rule interface {
    Code() string; Sheet() string; Severity() string
    Apply(ctx context.Context, doc *Doc3040) error
}
```

- `Builtin3040()` (`registry.go:531-914`) registra 275 regras com comentários de contabilidade por sprint ("Sprint 32 Fase 1: +14 regras... Total Registry: 275", `registry.go:515-530` — inclusive **dois comentários contraditórios** de cobertura: `275/361 = 76.2%` e `177/361 = 49.0%` nas linhas 528-530).
- **9 maps paralelos** no `Registry` (`registry.go:190-201`): `rules`, `rawRules`, `rules3050`, `rules2070`, `rules2061`, `rules2062`, `rules2160`, `rules2170`, `rules3044`, `rulesDRM` — com quadruplets `Register*/Get*/Codes*/All*` copiados 9x (~450 linhas de boilerplate, `registry.go:236-483`). Em Rust isso colapsa num único `HashMap<String, Box<dyn Rule>>` ou enum tipado.
- Interface `RawRule` (`registry.go:167-172`) para regras que operam no XML bruto (B01–B05).
- **153 regras com `Severity() = "I"` e `Apply` que retorna `nil` incondicionalmente** — stubs contábeis para "fechar % do catálogo". Exemplo real: `C33DiasAtrasoObrigatorio.Apply { return nil }` (`rules/3040_fase4.go`); padrão N01–N10 em `rules/3040_sprint36.go` (`N03LimitePorCli` é `return nil` literal). O comentário do código é honesto sobre isso (`registry.go:521`: "23 reais + 28 stubs (severity I retornam nil)"; comentários "DESTRABALHADAS" `registry.go:805-810,861-870` documentam stubs que viraram reais em sprints seguintes — mas note que tanto o stub quanto a versão destravada ficam registrados, com o segundo sobrescrevendo o primeiro no map).
- Regras de verdade fazem parsing de valores com `strconv`/comparações de string (valores BACEN chegam como string em structs tipados — `Doc3040Root`, `Agregado`, `Operacao`, `Cli`, `Parcela`, `rules/registry.go:26-151`).
- Parser 3040 tipado: `rules.ParseDoc3040` (encoding/xml com structs espelhando o leiaute). Parsers dos outros CADOCs moram em `internal/bacen` (11 arquivos, 651 LoC — puro parsing, 0% cobertura de teste direta, exercitados via crossdoc).

### 2.4 L3 cross-doc — duas implementações, uma morta

1. **`ValidateFull` L3** (`audit/service.go:403-445`): itera códigos `XD01..XD08` via `s.registry.Get(code)`. **Nenhuma regra XD01–XD08 está registrada** — as estruturas existem (`rules/crossdoc_liquidity.go:27-...` define `XD01`–`XD05`, `rules/crossdoc_dli.go` define XD06–XD08), mas não há nenhum `Register(XD...)` no registry (grep: só `X01–X10` do sprint 38 em `registry.go:851-860`). Resultado: **L3 do ValidateFull sempre reporta `LayerPassed`**. Além disso, se rodasse, teria um bug de perf: `rules.ParseDoc3040` é chamado **dentro do loop** de 8 códigos (`service.go:424-427`) e o erro de parse é ignorado (`doc3040, _ = ...`).
2. **crossdoc.Engine** (`crossdoc/engine.go:94-191`): o motor real — regras XD02/XD03/XD06/XD07/XD08 (+ registry completo em `crossdoc/rules/registry.go:61-65`), DocSet de XMLs brutos, execução **paralela com recover de panic** (`engine.go:128-184`), skip por docs ausentes. Exposto em `/v1/crossdoc/validate` (`server.go:250`) e usado pelo batch do generator. **Colisão de código: `XD02` existe nos dois motores** com semântica diferente (no audit: DtBase DRL/DLP/3044, `crossdoc_liquidity.go:68`; no crossdoc: total de operações 3040↔3050, `crossdoc/rules/xd_rules.go:25`).

Nota de concorrência: o comentário em `service.go:302-304` admite o motivo de L3 rodar **serialmente depois** de L1/L2/L4 — as regras XD do pacote `rules` leem **globals de pacote** (`rules/drl.go:517 var parsedDRL`, `dlp.go:456 var parsedDLP`, `crossdoc_liquidity.go:19 var parsed3044`, `dli.go:648 var parsedDLI`), setadas por `SetDRL/SetDLP/Set3044` (`service.go:407-414`). Estado global mutável + goroutines = data race por design; o código contorna, mas é o maior anti-padrão a não portar para Rust.

### 2.5 L4 histórico

`audit/l4/engine.go` — `Compare(ctx, envioID)` compara snapshot do envio atual vs anterior da mesma IF/CADOC: regras que falharam antes e passam agora ("FixedRules"), mudanças de campo (`extractFieldChanges`, `engine.go:242`), alertas. 1.121 LoC **sem nenhum teste** (`ls internal/audit/l4/` — 4 arquivos, 0 `_test.go`; coverage 0.0%). Há também `diff_expanded.go` (459 LoC) e `dlo_diff.go` que parecem extensões sem integração visível além do engine.

### 2.6 Schema Registry

`schema/registry.go` — versionamento **por data-base** em `schema_versions` (UNIQUE cadoc+effective_from, migration 001): `GetEffective(cadoc, dataBase)` (`registry.go:55-90`) pega a versão mais recente `effective_from <= dataBase`; fields como `fields_json`; `Insert` **auto-computa changelog por diff** com a versão anterior (`registry.go:97-120`, `changelog.go`); cache in-memory 5min **com singleflight anti-stampede** (`registry.go:251-319`, comentário honesto: "Em produção: trocar para Redis"). `ListCadocs` é dinâmico (UNION de schema_versions e criticas, `registry.go:215-238`) — substituiu lista hardcoded (W4). Porta trivialmente para Rust.

### 2.7 Generator (Sprint 57)

- Interface `CADOCGenerator` (`generator/interface.go:36-60`): `CadocCode()`, `Generate(ctx, *canonical.CanonicalDocument, dataBase) (*GeneratedDoc, error)`, `RequiredFields() []schema.Field`, `SupportedVersions()`, `EstimateComplexity()`. `GeneratedDoc` carrega XML + SHA256 + `FieldMap` de auditoria + metadata (`interface.go:65-98`).
- `Registry` com `RegisterDefaults` e glue em `cmd/api/main.go:126-140` — padrão explícito para evitar import cycle (comentário didático em `interface.go:163-204`). Em Rust: `HashMap<&str, Box<dyn CadocGenerator>>`, sem drama.
- 10 implementações (`gen3040` 485 LoC … `gen2070` 243 LoC, 4.942 LoC total). A de 3040 é real: agrupa operações por chave de agregado (`groupByKey`, `gen3040/generator.go:245-266`), soma faixas de vencimento, totalizador, FieldMap. **Porém preenche defaults silenciosos** (`op.UF = "SP"`, `gen3040/generator.go:275-277`; `emailResp` default `"resp@if.com.br"`, `generator.go:223`) — decisão de produto que merece revisão no port.
- **Duplicação**: `strVal`, `intVal`, `decimalToFloat`, `strFloat` copiados nos **10 generators** (grep `func strVal|func intVal|...` = 10 arquivos).
- Canonical: `canonical.CanonicalDocument` (`canonical/document.go:32-70`) com header, participantes, operações, garantias, `Money` (shopspring/decimal), `Extra map[string]any` (escape hatch tipagem-frouxa usado como canal de campo por CADOC — em Rust viraria `HashMap<String, serde_json::Value>` ou enum de payloads por CADOC).
- Segundo motor de validação do generator (`generator/validation/validate.go`) é **raso e duplicado** (L1 = bem-formado apenas, comentário admite: "Full XSD schema validation would require loading the XSD from the Schema Registry... first-pass", `validate.go:222-223`); L2/L3 por CADOC com structs anônimos repetidos (`check3040Required`…`check2170Required`, quase idênticos, `validate.go:258-381`).

### 2.8 STA client

Real, não stub: `sta/ws.go` (1.357 LoC) implementa o fluxo oficial BACEN (POST /arquivos → PUT conteudo, posicaoupload, download com verificação `X-Content-Hash`, disponiveis, situacao, chunked range, Basic auth preemptivo — cabeçalho do arquivo documenta as seções do manual). `StubClient` para dev (`stub.go`). `retry.go` com `RetryingClient` (307 LoC). Interface segregation: `ReadClient`, `ChunkedClient` para capability checks em runtime. Seleção via `RADIANT_STA_BACKEND=ws` (`cmd/api/main.go:108-112`). Sólido para portar como spec de integração.

---

## 3. QUALIDADE

### 3.1 Pontos fortes reais

1. **Honestidade documental incomum**: comentários citam sprints, bugs corrigidos e trade-offs (`audit/service.go:166-171` explica bug de NULL Scan; `worker/worker.go:208-212` explica formato datetime SQLite; `cmd/api/main.go:185-195` explica mismatch kid). Isso é ouro para o port: o "porquê" está no código.
2. **Higiene de erro**: `loggerutil.SafeError` (`loggerutil/safe.go:62`) sanitiza DSN/host/password antes de qualquer superfície externa — aplicado consistentemente em api, worker, crossdoc, cmd (`grep SafeError` dezenas de usos). 141 usos de `_ =` (erros ignorados), a maioria em audit-log fire-and-forget (`server.go:765`), com intenção documentada.
3. **Concorrência defensiva**: recover de panic por camada/goroutine (`service.go:307-396`, `crossdoc/engine.go:135-148`), singleflight contra stampede (`schema/registry.go:286`), `BEGIN IMMEDIATE` + WAL + busy_timeout 30s no SQLite (`db/db.go:64-67`), claim atômico de fila no worker (`worker/worker.go:148-171`).
4. **Segurança**: JWT verifier com keyring/kid (`cmd/api/main.go:180-212`), CSRF, rate limit Redis, `DisallowUnknownFields` (`api/validate.go:89-99`), regex de cadoc_code (`api/validate.go:22-34`), RLS Postgres, audit log hash-chain, env gates fail-closed (`main.go:215+`).
5. **Race detector limpo** nos packages audit/crossdoc/rules (executado nesta auditoria); suíte completa verde.

### 3.2 Dead code / código não-wireado (o maior risco de "spec falsa")

| Item | Evidência | Status |
|---|---|---|
| `Builtin3050()` (170 regras) | `rules/3050.go:3381` — único referenciado em comentários | Implementado, testado (61,6% pkg), **nunca wireado** |
| `Builtin2070()` (11 regras) | `rules/2070.go:318` | Idem |
| `Get3050/Get2070/Get2061/Get2160/Get2170/Get3044/GetDRM` | grep produção: 0 chamadas | API morta |
| XD01–XD08 (L3 do ValidateFull) | `rules/crossdoc_liquidity.go`, `crossdoc_dli.go` — nunca registrados | **L3 no-op** (`service.go:415-441`) |
| drsac engine (rules.go 801 + validator 294 LoC) | `drsac.ValidateDocument` sem chamadores; `IntegrateWithAudit()` vazio/não-chamado | Órfão |
| doc4111 `Validate`/`ValidateDocument` | Só o parser é usado (por crossdoc) | Validador órfão |
| `All3050` aloca slice e descarta | `registry.go:256-257` (`out := make...; _ = out // unused`) | Bug inócuo, cheiro |
| Comentários de cobertura contraditórios | `registry.go:528-530` (76,2% vs 49,0%) | Doc podre |
| `generator-server` | Binário Mach-O commitado | Lixo de repo |
| `l4/diff_expanded.go` + `dlo_diff.go` (607 LoC) | Sem testes, integração parcial | Semi-órfão |
| `rules/3026.go` (229 LoC) + `expectedRootTag "3026"` | CADOC 3026 não está nos 10 do produto | Resquício |

### 3.3 Stubs e TODOs

- **33 TODO/FIXME/XXX** em produção (grep) — taxa baixa para 57k LoC.
- **153 regras stub** `Severity()=="I"` que retornam `nil` (contabilizadas no 3040; ~28-45 por sprint de expansão conforme comentários `registry.go:519-526`). O produto conta "275 regras 3040" mas **apenas ~120-130 executam lógica real**; o restante é placeholder honesto.
- STAs: `StubClient` é dev-only por design; `multiregion` (333 LoC) é CRUD de metadados, **não** replicação real; `soc2` é catálogo de controles/evidence (0,8% cobertura); `billing` tem Stripe client HMAC real mas wiring mínimo; `insights` LLM é opt-in via env (`main.go:117-142`).

### 3.4 Duplicação estrutural (custo direto no port)

1. Registry de regras: 9 quadruplets `Register*/Get*/Codes*/All*` (~450 linhas).
2. 10 generators: helpers `strVal/intVal/decimalToFloat/strFloat/strFloat/sha256Hex` duplicados 10x.
3. Dois motores L1–L4 com semântica conflitante (audit vs generator/validation) e dois registries XD com código colidindo (`XD02`).
4. `check*Required` do generator/validation: 6 funções quase idênticas.
5. Parsers do mesmo documento em 2-3 lugares: 3040 tem parser em `rules` (para L2), em `bacen` (para cross-doc) e structs em `gen3040` (para emitir); 2062 tem `docdli.DocumentoDLI` **e** `rules.ParseDocDLI` com parse duplicado por regra (`service.go:745-752` parseia por chamada, sem cache).
6. Migration SQLite vs Postgres: SQL escrito para SQLite com `@postgres-only` skip (migração manual), RLS só em Postgres.

### 3.5 Banco de críticas: chaves quebradas entre seed e API

`cmd/seed/main.go:155-159` grava `criticas.cadoc_code` **igual à chave do JSON**: `2060-DRM`, `2061-DLO`, `2070-DDR`, `3040`, `3044`, `3050` (verificado no `_catalogos/criticas.json`). A API só aceita `^[0-9]{4}$` (`api/validate.go:22`) e `LoadCriticas(ctx, "2061")` não acha nada (`service.go:173-179` — igualdade exata). Consequências:

- 2060/2061/2070: críticas inalcançáveis pelo endpoint (568+ regras de catálogo mortas no runtime, além das Go não-wireadas).
- 2062: sem críticas no seed → **nem L2 estrutural DLI roda via API** (o caminho `is2062` de `applyRegra` só dispara dentro do loop de críticas).
- 3050/3044: críticas encontradas, mas `applyRegra` devolve `nil` para todas (§2.2) → validação L2 sempre passa.
- Em resumo: **a promessa "1.099 regras de catálogo carregadas" é verdadeira no DB e falsa no runtime.**

### 3.6 Cobertura de teste real (medida `go test -cover` nesta auditoria)

Alta (>80%): auditlog 92,5, branding 92,6, docdli 92,2, senhaws 95,6, loggerutil 96,2, tenant 90,2, crossdoc 83,2, worker 86,1, generator 100, gen3050 91,7.
Média (50–80%): api 58, audit/rules 61,6, auth 71,8, canonical 72,2, db 62,7, schema 64, ruleprefs 63,3, sta 73,6, radar 78,4, realtime 78,9, gen* 71-83, doc4111 55,4.
Crítica/baixa: **audit (service) 25,1**, crossdoc/rules 23,4, ingest 25,6, drsac 43,5, **l4 0%, bacen 0%, observability 0%**, billing 10,8, insights 8,3, soc2 0,8, pilot 3,3, webhook 10,3, synth 11,3, wizard 16,1, marketplace 14,6.
O coração do produto (service.go de despacho + l4) é o pior coberto; as folhas (regras, parsers DLI) são bem cobertas.

---

## 4. DATA LAYER

### 4.1 Migrations

26 arquivos SQL (001→026, 1.404 LoC) **embedados** (`//go:embed migrations/*.sql`, `db/migrate.go:22`), aplicados por ordem com tracking `schema_migrations`, idempotentes, `BEGIN IMMEDIATE`. Dual-driver: SQLite (modernc, sem CGo) e Postgres (pgx stdlib) com detecção por prefixo DSN (`db/db.go:38-56`); migration `@postgres-only` é pulada em SQLite (`012_rls_policies.sql` cabeçalho). **Port Rust: as 26 migrações são o artefato mais reaproveitável 1:1** (com o trabalho SQLite→Postgres-only para retirar os workarounds de formato datetime).

### 4.2 Schema do banco (tabelas principais)

`ifs` (tenant, plano, sta_user), `schema_versions` (fields_json, xsd, changelog), `criticas` (UNIQUE cadoc+sheet+codigo, enabled, data_base_inicio/fim), `envios` (status pending→processing→validated/sent/accepted/rejected/error, protocol_sta, attempts, next_retry_at, dead-letter), `audit_log` (hash chain: `payload_hash` + prev hash), `audit_events`, `rule_failures`, `disabled_rules`, `radar_alerts`/`radar_baselines`, `webhooks`, `marketplace_rules`, `pilot*`, `insights_conversations`, `wizard_sessions`, `range_sessions`, `soc2*`, `white_label`, `stripe*`.

### 4.3 RLS

- 012: `ENABLE ROW LEVEL SECURITY` + policies `USING (if_id = current_setting('app.if_id', true))` para envios, audit_log, audit_events, rule_failures, disabled_rules, acknowledged_recommendations — **modo permissivo por decisão documentada** ("fase ENABLE; FORCE requer SET LOCAL em BeginTx", `012_rls_policies.sql:22-30`).
- 014: `FORCE RLS` para as tabelas de 012; 026 estende FORCE para wizard_sessions, range_sessions, webhooks, insights_conversations.
- Aplicação: `db.WithTenantTx` seta `app.if_id` (`ruleprefs/preferences.go` cabeçalho; `db/tenant.go`). Em SQLite é no-op.
- **Port Rust: padrão já Postgres-first — manter, e o port é a oportunidade de dropar o modo ENABLE (012) que existe só por compat de migração.**

### 4.4 Onde moram as regras/catálogos

| Artefato | Onde vive | Como entra em runtime |
|---|---|---|
| Catálogo de críticas (1.099) | `_catalogos/criticas.json` → tabela `criticas` | `cmd/seed` (com chaves quebradas §3.5) |
| Leiates | `_catalogos/leiautes.json` → `schema_versions.fields_json` | `cmd/seed` |
| XSDs | `_catalogos/3040_generated.xsd` + XSD oficial 3050 em path **relativo ao repo** | `audit/xsd_validator.go:18-22` (quebrado em deploy) |
| Regras executáveis | **Código Go compilado** (`audit/rules/*.go`) | `rules.Builtin3040()` wireado no Service (`audit/service.go:130`) |
| Toggles por IF | Tabela `disabled_rules` | `ruleprefs` + interface `RulePrefs` (`audit/service.go:117-119` — quebra de dependência para evitar import cycle) |

**Implicação para o port:** no Rust, vale mover as ~511 regras não-3040 para o formato data-driven (o catálogo JSON já existe e é a fonte) OU mantê-las como código Rust tipado com o catálogo servindo de spec de teste. Recomendação no §6.

---

## 5. PORT-RUST — o que é trivial, o que precisa redesenho

### 5.1 Porta trivialmente (tradução quase 1:1)

- **Migrations SQL + schema DB + RLS** (26 SQL) — copiar.
- **Catálogos JSON** (criticas/leiautes) — já são dados.
- **Parsers tipados XML/JSON** (`bacen/`, `doc4111`, `docdli`, `drsac` ~1.700 LoC) → `serde`/`quick-xml` com derive; structs são planas e string-typed.
- **Generators** (4.942 LoC): transformação canônico→XML pura; `xml.NewEncoder` → `quick-xml`/`xmltree`; `shopspring/decimal` → `rust_decimal`.
- **Schema Registry, ruleprefs, tenant, branding, auditlog, webhook, billing, pilot, marketplace, insights (KPIs)**: CRUD SQL direto, sem nada de Go-específico.
- **STA WSClient** (1.357 LoC): HTTP REST puro (reqwest) — sem streaming esotérico; a parte mais sensível é a verificação `X-Content-Hash` e chunked range, ambos lineares.
- **Worker**: loop + claim atômico → `tokio` + `FOR UPDATE SKIP LOCKED` (melhora) ou o mesmo UPDATE...RETURNING.
- **HTTP API** (chi 14.380 LoC): roteamento chi → `axum`; middlewares 1:1; SSE hub (`realtime`) → `tokio::sync::broadcast`.

### 5.2 Go-específico que exige redesign (mas é pequeno)

| Padrão Go | Onde | Redesenho Rust |
|---|---|---|
| **Globals mutáveis de pacote** (`parsedDRL/parsedDLP/parsed3044/parsedDLI` + setters) | `rules/drl.go:517`, `dlp.go:456`, `crossdoc_liquidity.go:19`, `dli.go:648`; orquestrados em `service.go:302-414` | **Eliminar** — passar `&DocSet` por parâmetro (como o motor crossdoc já faz). É o motivo de L3 ser serial em Go; em Rust vira execução paralela sem raça |
| **Goroutines + WaitGroup + recover por camada** | `service.go:302-398`, `crossdoc/engine.go:128-184` | `tokio::spawn` + `catch_unwind` ou (melhor) regras que nunca panicam → `Result`; paralelismo com `futures::join!`/rayon |
| **singleflight** | `schema/registry.go:286` | `moka`/`mini-moka` ou `tokio::sync::Mutex<Option<CacheEntry>>` + `dashmap` |
| **database/sql (dual driver)** | todo o código SQL usa `?` placeholder SQLite-style | `sqlx` Postgres-only; o translate de `?`→`$n` desaparece ao dropar SQLite |
| **embed.FS** | migrations | `include_str!`/`rust-embed` ou `refinery`/`sqlx migrate` |
| **Interfaces dinâmicas (Rule/RawRule/CrossDocRule/CADOCGenerator/Client)** | registry, generator, sta | `trait` + `Box<dyn Trait>` ou enum `Rule` tipado (preferível: dá exhaustiveness checking e mata a classe de bugs do dispatch silencioso) |
| **encoding/xml (na validação XSD caseiro)** | `audit/xsd_validator.go` | Substituir por **XSD real** (`xsd-parser`/`xmlschema` crate ou serviço `xmllint`) — o subset caseiro é o elo mais fraco do L1 |
| **reflection** | 1 uso trivial (grep) | irrelevante |
| **excelize** (Radar v2 XLSX) | `radar/radar_v2.go` | `calamine` (leitura) — excelize é o único dep pesado; se Radar só lê, calamine basta |
| **sync/atomic, hash chain** | auditlog | trivial |

Canais: 13 usos, concentrados em `realtime` (hub SSE) e `worker` — nada de pipelines complexos de channels.

### 5.3 O que seria MELHOR em Rust

1. **O dispatch de regras**: em Go, regra ausente = skip silencioso (`service.go:758-759`) — a classe inteira de bugs do §2.2. Com enum tipado + match exaustivo ou tabela `(cadoc, código) -> handler` construída por macro com verificação de cobertura, o compilador audita a cobertura do catálogo.
2. **Paralelismo de camadas**: L1/L2/L4 + L3 sem globals → paralelo de verdade, sem o workaround serial.
3. **Golden tests de paridade**: 1.178 testes Go existentes podem virar fixtures — rodar cada XML de teste nos dois motores e diffar resultados. É o maior seguro do port.
4. **Erros**: `thiserror`/`anyhow` substituem o padrão `fmt.Errorf("...: %w")` mantendo a disciplina de sanitização (`SafeError` → `sanitize.rs` único, 1:1).
5. **XSD real** em vez do subset caseiro — crates/`libxml2` bindings dão validação de verdade para os 10 CADOCs (hoje só 3050/3045/3040 têm schema).

### 5.4 O que NÃO portar

- `audit/xsd_validator.go` como está (subset caseiro + paths relativos quebrados) — reimplementar com lib real.
- `generator/validation/validate.go` — motor duplicado e raso; unificar no motor principal.
- `rules/registry.go` boilerplate 9x — colapsar.
- Globals XD + XD01–XD08 mortos + `X01–X10` colidindo com XD02–XD12 — unificar num só namespace de cross-doc.
- `cmd/_verify`, `generator-server` (binário), seeds pontuais de sprint, `radiant.db.backup`.
- Stubs severity-I (153) — portar só as ~358 regras com lógica real, usando o catálogo como checklist.
- Modo ENABLE RLS (012) e dual-driver SQLite no caminho de produção.

---

## 6. VEREDICTO — usar o Go como spec

| Estratégia | % do backend | O que inclui |
|---|---:|---|
| **Aproveitar 1:1** | **~25%** | 26 migrations SQL, schema do DB, RLS policies, catálogos JSON, tipos dos leiautes (structs → serde), STA ws_types (protocolo manual BACEN), API contract (rotas/JSON de `server.go:183-356`), audit log hash-chain (algoritmo), cláusulas de negócio documentadas nos comentários |
| **Portar como spec apenas (reimplementar + paridade por testes)** | **~50%** | Regras L2 reais (~358: 3040 completo + 170 de 3050 + 18 DLI + 24 DLO + 11 DRL + 10 DLP + 7 DRM + 17 T-rules + DRSAC) — código Go vira *referência comportamental*, validado pelos 1.178 testes convertidos em golden tests; generators (mesma matemática de agregação); STA WSClient; worker/backoff; radar |
| **Redesenhar (estrutura muda, semântica preservada)** | **~15%** | Motor de validação unificado (1 registry por CADOC com dispatch exaustivo — mata os achados §2.2/§3.2); cross-doc unificado (fim dos 2 motores e da colisão XD02); remoção de globals; HTTP layer chi→axum; realtime/SSE; singleflight→cache idiomático |
| **Jogar fora** | **~10%** | XSD validator caseiro, generator/validation duplicado, XD mortos, stubs severity-I, seeds de sprint, `_verify`, binário commitado, modo ENABLE RLS, dual-driver SQLite em produção |

**Riscos do port mapeados no Go que o Rust DEVE corrigir (não herdar):**
1. Dispatch silencioso de regras não-implementadas (`service.go:758-759`) — virar erro de compilação/config explícita.
2. Chaves de críticas com sufixo (`2061-DLO`) incompatíveis com códigos de API (§3.5) — normalizar no ETL do catálogo.
3. XSD paths relativos ao repo (`xsd_validator.go:18-22`) — embedar schemas.
4. Parse repetido por regra (DLI `service.go:747-752`; doc3040 no loop L3 `service.go:424-427`) — parse 1x, regras N.
5. Defaults silenciosos no generator (UF="SP", email fake — `gen3040/generator.go:223,275-277`) — virar erro ou warning explícito.
6. Two sources of truth para L1–L4 — um motor só.

**Estimativa de esforço relativo (com Go como spec + golden tests):** rules 3040/regras em geral ~40% do esforço; HTTP+infra ~25%; generators ~15%; STA/worker/radar ~15%; limpeza/unificação ~5%. A existência de 1.178 testes verdes e comentários de intenção reduz materialmente o risco — o Go está em condição de "spec executável", algo raro em ports.

---

## Apêndice A — Evidências-chave (índice)

| Achado | Evidência |
|---|---|
| Só Builtin3040 wireado | `audit/service.go:130`; grep `Builtin3050()` → só `rules/3050.go:3381` |
| applyRegra 3 rotas + skip silencioso | `audit/service.go:690-760` (skip em 758-759) |
| XD01–XD08 nunca registrados | `audit/service.go:415-419` consulta; grep `Register(XD` = 0; definições em `rules/crossdoc_liquidity.go:27` |
| L3 serial por causa de globals | comentário `audit/service.go:302-304`; vars em `rules/drl.go:517`, `rules/dlp.go:456` |
| Builtin3050/Builtin2070 órfãos | `rules/3050.go:3381`, `rules/2070.go:318` |
| Getters per-CADOC sem chamadas | grep `Get3050|Get2070|Get2061|Get2160|Get2170|Get3044|GetDRM` — só registry.go e testes |
| Stubs severity-I = 153 | grep `Severity() string { return "I"` em `rules/*.go` |
| Registry 9 maps + quadruplets | `rules/registry.go:190-217`, `236-483` |
| Críticas com chaves sufixadas | `cmd/seed/main.go:155-173`; `_catalogos/criticas.json` (chaves `2061-DLO` etc.) |
| XSD paths relativos | `audit/xsd_validator.go:18-22`, degradação em `36-40` |
| L3 parseia doc3040 no loop, ignora erro | `audit/service.go:419-427` |
| Colisão XD02 (2 motores) | `rules/crossdoc_liquidity.go:68` vs `crossdoc/rules/xd_rules.go:25` |
| Helpers duplicados nos 10 gens | grep `func strVal` → 10 arquivos em `generator/gen*/generator.go` |
| Real STA WS | `sta/ws.go` (1.357 LoC, cabeçalho documenta seções do manual) |
| Migrations embed | `db/migrate.go:22`; 26 SQL em `db/migrations/` |
| RLS ENABLE→FORCE | `012_rls_policies.sql`, `014_rls_enforce.sql`, `026_rls_extended.sql` |
| Suite verde + race limpo | `go test ./... -count=1` exit 0; `go test -race ./internal/audit/... ./internal/crossdoc/...` ok |
| Cobertura por package | output `go test -cover` (§3.6) |
| comentário contraditório cobertura | `rules/registry.go:528-530` |
| slice morto em All3050 | `rules/registry.go:256-257` |
