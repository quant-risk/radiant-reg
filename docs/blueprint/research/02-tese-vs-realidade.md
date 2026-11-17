# 02 — Tese vs. Realidade: Auditoria do Plano Ouro contra o repo em nov/2026

> **Data:** 2026-11-16
> **Objeto:** confrontar a tese original (jul/2026) com o que o repo `radiant-reg` (v3.36.4) prova que aconteceu.
> **Método:** leitura integral de MASTER_PLAN.md, PRODUTO_TESE_ROADMAP.md, ENG_REVERSA.md, ROADMAP.md, MEMO_MOTOR_GERACAO.md, README.md, 8 ADRs, CHANGELOG (cabeça + cauda + série v3.34), amostra de VALIDATION_*.md e SPRINT_*; verificação direta de código (`go build`, `go test ./...`, migrations, adapters, smoke tests) e **git log como ground truth cronológico** (263 commits).
> **Achado estrutural que organiza tudo abaixo:** o repo inteiro foi construído entre **2026-07-03 e 2026-07-13** (10 dias corridos). Os "4 quarters" do Plano Ouro foram executados — no papel — dentro desses 10 dias. Todo o resto é consequência disso.

---

## 1. TESE ORIGINAL (jul/2026)

### 1.1 Proposta de valor

Tese em uma frase (PRODUTO_TESE_ROADMAP.md:23):

> *"A corrida regulatória 2024-2027 [...] criou uma janela de adaptação obrigatória em todas as ~2.000 IFs brasileiras. [...] Radiant Norma é o motor de geração que nunca dorme: (1) recebe dados de qualquer fonte (planilha, PDF, API, banco, MCP), (2) gera o CADOC pronto, (3) valida com 1.099 regras, (4) explica cada campo — vendendo tranquilidade, não XML."*

Manifesto (MASTER_PLAN.md:16):

> *"A IF não gasta 3 pessoas preenchendo planilhas: sobe os dados, recebe o CADOC gerado e validado, aperta um botão. Sabe que mudou leiaute antes do BACEN publicar, sabe que auditar é apertar um botão."*

Três promessas mensuráveis (MASTER_PLAN.md:18-21):

| Promessa | Alvo |
|---|---|
| Latência de geração | ≤ 5 min entre receber dados e ter CADOC validado |
| Cobertura executável | ≥ 90% das regras semânticas + geração de 10 documentos |
| Audit defensável | ≤ 60s para extrair evidência forense completa |

Estrutura da tese (PRODUTO_TESE_ROADMAP.md:27-56): **3 teses paralelas** — (T1) vender *motor de geração*, não validador ("IFs não compram validador — BCValidador é gratuito", :29); (T2) ESG/DRSAC 2030 first-mover com janela IN BCB 694/2025, vigência dez/2026 (:36-45); (T3) mid-market com fit para SaaS acessível (Lite R$ 1,5k vs "R$ 5-15k/mês Matera/cadoc.ai", :49-56).

### 1.2 Posicionamento (MASTER_PLAN.md:55-66)

| Segmento | Concorrentes | Vantagem declarada |
|---|---|---|
| SCD R$ 50M-1B | Matera, Mitra (LUZ), cadoc.ai | Lite R$ 1,5k + onboarding 15min + 3040 + 3050 |
| IP médio R$ 1-10B | Dattos, TOTVS | 3044 JSON + DLO/DLI + audit chain SOC 2 |
| Banco S3-S4 | Matera, Orbi, TIVIT | Cross-doc L3 + ESG first-mover + SLA 99.95% |
| Fintech BaaS | (commodity) | White-label + API-first + pricing per-environment |

Movimento de mercado por quarter (:62-66): Q3 2026 "único com 3040+3050 production-grade + smoke BACEN homolog validado" → Q4 2026 "primeiro a fechar DLO+DDR+DRL+DLP" → Q1 2027 "first-mover DRSAC" → Q2 2027 "plataforma completa, única cobertura 100% do ecossistema SCR".

### 1.3 Pricing (PRODUTO_TESE_ROADMAP.md:184-189)

| Plano | Preço | Inclui |
|---|---|---|
| Lite | R$ 1.500/mês | 1 IF, 3 CADOCs, 50 envios/mês |
| Pro | R$ 4.500/mês | 1 IF, todos CADOCs, Radar |
| Scale | R$ 12.000/mês | Multi-tenant, white-label, SLA 99.9% |
| Enterprise | Sob consulta | Self-host, dedicated |

ENG_REVERSA.md:91 sustenta a tese de preço: Mitra "R$ 50k+/mês (projeto)", Matera "R$ 30k+/mês", cadoc.ai/Dattos "R$ 5-15k/mês" — "Nosso: R$ 1.5-12k/mês → catch whole mid-market" (:296).

### 1.4 GTM (PRODUTO_TESE_ROADMAP.md:89-145)

- Princípios: vender "tranquilidade regulatória"; onboarding 15 min; **pricing público**; pilot gratuito 30-90 dias; "1 SCD-piloto vencido = 10 follow-ons em 60 dias" (:95).
- Canais: SEO/CADOC, LinkedIn outbound, listas ABBC/Febrafar/ABFintechs, 1 BDR no mês 6, 1 AE no mês 9, pipeline alvo mês 6 = 30 IFs/5 demos-semana (:110-113).
- Ciclos: SCD/SEP 30-90d; IP 60-120d; Banco 6-12 meses (:138-145).
- Anticorpos: tabela de objeções Compliance/Risco/TI/Jurídico/Compras/CIO, incluindo "SOC 2 Type II em mês 18" (:125).

### 1.5 Personas (PRODUTO_TESE_ROADMAP.md:62-85)

Compradores: Carlos (Head Compliance/CTO, SCD R$ 100M-1B, "9º dia útil do mês"), Marina (Diretora Risco, IP, 3044 em tempo real), Roberto (CIO banco S3-S4, RFP 6-12 meses), Ana (Fintech BaaS), Cooperativa. Usuários: Analista Júnior, Coordenador de Riscos, Auditor interno. Decisor único em SCD/IP; comitê em banco.

### 1.6 Roadmap de 4 quarters (MASTER_PLAN.md §1) e milestones

- **Q3 2026** "Fechar o CORE" (Sprints 28-37): Vault, **BacenHomologSmoke (Sprint 29: "Smoke real contra sta-h.bcb.gov.br/staws + www9.bcb.gov.br/senhaws. Credenciais Sisbacen reais"** :79), PostgresRLS (:80), Audit3040_v2, Audit3050, FrontendNext, CI-Gate, Observability, **Pilot — "Onboarding real 1 SCD-piloto (30-90 dias)"** (:87). Saída: "Radiant Norma Lite vendável pra SCD pequena" (:89).
- **Q4 2026** "Multi-CADOC" (38-48): DLO/DDR/DRL/DLP/3044, CrossDoc_v2, Radar_v2, StripeBilling, WhiteLabel, DRSACResearch, Pilot2 (:96-108).
- **Q1 2027** "ESG first-mover" (49-56): DRSAC_v1/v2, 4111, AIInsights, SchemaRegistry_v2, Pilot3, **SOC2_Type1 "Auditoria SOC 2 Type I"** (:124).
- **Q2 2027** "Plataforma + Norma Generator" (57-67): 🚨 motor de geração, DRM/DLI, SDKs, webhooks, marketplace, MultiRegion, Pilot4, SOC2_Type2, SeriesA (:132-146).

Milestones e kill criteria (MASTER_PLAN.md:150-155 e §9.4 :2742-2746):

| Marco | Data | Sucesso | Kill switch |
|---|---|---|---|
| M1 — Piloto pagante | 2026-09-30 | 1 SCD pagou R$ 1,5k/mês por 3 meses | Cancelamento = pivotar GTM |
| M2 — 10 clientes | 2026-12-31 | 10 IFs, NPS ≥ 40, churn < 5%/mês | < 5 clientes = pivotar pricing |
| M3 — ESG vendido | 2027-03-31 | 1 cliente DRSAC ativo | 0 = DRSAC vapor |
| M4 — Series A ready | 2027-06-30 | R$ 100k MRR, NRR > 110% | < R$ 50k MRR = bootstrap |

Kill criteria §9.4: "Após 6 meses de piloto: 0 cliente pagou → GTM errado. **Após Q3 2026: MRR < R$ 5k → pivotar pra vertical única.** Após Q1 2027: sem DRSAC vendido → DRSAC não é viável comercialmente. Após Q2 2027: MRR < R$ 100k → bootstrap mode."

Metas de 18 meses (PRODUTO_TESE_ROADMAP.md §7-8): R$ 50k MRR no mês 6 (:644), R$ 200k no mês 12 (:666), R$ 500k-1M no mês 18 (:680); 2-3 LOIs com SCDs-piloto nas semanas 1-8 (:625).

---

## 2. PREMISSAS DE JUL/2026 (explícitas e implícitas)

| # | Premissa | Onde está | Tipo |
|---|---|---|---|
| A1 | Existem ~2.000 IFs brasileiras formando mercado endereçável | TESE:23 | mercado |
| A2 | IF gasta 1-3 pessoas full-time em CADOC; R$ 5-15k/mês de dor | TESE:49-56, ENG_REVERSA:91 | mercado |
| A3 | BCValidador é gratuito e só valida; Matera/Mitra só validam+enviam — "geração é o diferenciador" | TESE:29; ADR-0007:9-18 | concorrência |
| A4 | Concorrentes têm XSD hardcoded, onboarding de semanas, pricing opaco | ADR-0004:11-13; ENG_REVERSA:107-113 | concorrência |
| A5 | "Nenhum concorrente SaaS cobre direito" DRSAC 2030 | TESE:43; ENG_REVERSA:302-314 | concorrência |
| A6 | Concorrente lançar DRSAC antes é risco **Baixa** (TESE:751) — mas MASTER_PLAN §9.2 diz probabilidade **Média** ("Materia lança DRSAC antes", :2726) | contraditória entre docs | concorrência |
| A7 | IN BCB 694/2025: DRSAC com vigência dez/2026 = janela de 12 meses | TESE:40; MASTER_PLAN:703 | BACEN |
| A8 | IN BCB 733/2026 muda 3040 em 3 ondas (mai/jul/nov 2026) | TESE:784; ENG_REVERSA:261 | BACEN |
| A9 | BACEN muda leiaute 3-5×/ano; STA WS estável o suficiente para SaaS | MASTER_PLAN:9; ADR-0004:9 | BACEN |
| A10 | "BACEN não homologa fornecedores SaaS" | TESE:518 | BACEN |
| A11 | Críticas 2030 DRSAC não são públicas; requerem credencial/parceria (B3Bee/Deloitte) | MASTER_PLAN:706-710; README:326-329 | BACEN |
| A12 | STA WS (sta-h) + Senhaws (www9/www3) acessíveis com credencial Sisbacen; protocolo de 18 dígitos | ENG_REVERSA:263; MASTER_PLAN:79 | tecnologia |
| A13 | **Go 1.25 + Postgres 16 + Redis 7 + Next.js 15** é a stack "definitiva" sem reescrita até 1000 tenants | ADR-0001:9,18-37 | tecnologia |
| A14 | Hiring no Brasil: Go mainstream é fácil, Rust "10× mais difícil" | ADR-0001:13,58 | tecnologia/time |
| A15 | GC do Go (~1ms) irrelevante vs latência BACEN (50ms+) | ADR-0001:52; README:119-121 | tecnologia |
| A16 | Postgres RLS + `SET LOCAL app.current_if_id` em toda transação é custo aceitável | ADR-0002:18,64 | tecnologia |
| A17 | SQLite no dev é troca de "1 linha" para Postgres em prod | README:130 | tecnologia |
| A18 | LLM escreve CanonicalDocument tipado, nunca XML direto; validação L1-L4 downstream neutraliza hallucination | ADR-0007:56-59 | tecnologia |
| A19 | Time real: "Eng Lead + 2 Eng Sênior" por épico (MASTER_PLAN:275,913); plano de contratações Fase 1: +1 Dev Go, +1 Front, +1 SRE (TESE:687) | time |
| A20 | Cronograma realista: 4 quarters, sprints de 2 semanas, 39 sprints no Plano Ouro (README:290) | cronograma |
| A21 | 2-3 LOIs assináveis em 8 semanas com rede Fortvna | TESE:625 | comercial |
| A22 | Piloto pagante em 90 dias (M1 até 2026-09-30) | MASTER_PLAN:152 | comercial |
| A23 | R$ 300-500k bootstrap disponível; Seed R$ 1.5-2M; Series A R$ 6-10M | TESE:695-699 | capital |
| A24 | SOC 2 Type I em mês 9 é suficiente para vender a bancos S3-S4 | TESE:513-518 | comercial |
| A25 | 1 pessoa + agentes IA ("Mavis") consegue sustentar cadência de sprint de 2 semanas com DoD completo (PR review de ≥1 pessoa, ≥2 em arquivos críticos — MASTER_PLAN:2248-2250) | implícita e fatal | processo |
| A26 | O "motor de geração" justifica R$ 1.500-12.000/mês | ADR-0007:18; README:46 | preço |
| A27 | Pricing público + self-service vencerão o modelo consultivo da Matera | TESE:93; ENG_REVERSA:107 | GTM |
| A28 | Radar consegue detectar mudança de leiaute ≤ 24h ("sabe que mudou leiaute antes do BACEN publicar", MASTER_PLAN:16) | promessa | produto |
| A29 | Catálogo de críticas extraído das planilhas BACEN é suficiente para "≥90% das regras" | MASTER_PLAN:20; TESE:544-597 | produto |
| A30 | "Nada de F5" / SSE < 3s / Radar < 24h são SLOs publicáveis por um time de 1 pessoa | MASTER_PLAN:258-267 | operações |

---

## 3. REALIDADE EM NOV/2026 — o que o repo prova

### 3.1 O fato dominante: cronologia comprimida

```
git log --date=short | histograma:
  53 commits 2026-07-03
  17 commits 2026-07-04
  31 commits 2026-07-05
  20 commits 2026-07-06
  66 commits 2026-07-07   ← dia único com 53 versões v3.34.x
  15 commits 2026-07-08
  18 commits 2026-07-09
   3 commits 2026-07-10
  14 commits 2026-07-11
  13 commits 2026-07-12
  13 commits 2026-07-13   ← HEAD (Sprint 79-83)
```

- **263 commits, 11 dias**; último commit: 2026-07-13 (`53b4267` "Sprint 79-83: FASE 1-5 completion").
- **Nenhum commit entre 2026-07-14 e hoje (2026-11-16): o repo está congelado há ~4 meses.**
- CHANGELOG.md:638-2691: 53 entradas `v3.34.0` → `v3.34.53` **todas datadas 2026-07-06/07**, queimando os Sprints 32-65 (Q4 2026 + Q1 2027 + Q2 2027 do Plano Ouro) em ~36 horas.
- README.md:363 (rodapé): *"Construído em 3 sprints · ~30 horas úteis"* — a única frase honesta sobre o ritmo real, convivendo com o badge "sprint 78/78" (:15).

### 3.2 O que MATERIALIZOU de verdade (código verificado hoje)

Verificado em 2026-11-16 com `go build ./...` (exit 0) e `go test ./...` (**51 packages `ok`, ~1.178 funções de teste**):

| Claim do Plano Ouro | Realidade no repo | Evidência |
|---|---|---|
| Épico I — motor de geração 10 CADOCs (MASTER_PLAN §3.9) | **Real**: `internal/generator/` com `gen2030/2060/2061/2062/2070/2160/2170/3040/3050/4111` + wizard + validation | `backend/internal/generator/` (10 pacotes, todos com testes passando) |
| 5 conectores Manual/File/API/DB/MCP (ADR-0008) | **Real** (evolução honesta): v3.36.0 apenas Manual; v3.36.3 +Manual/File ("Manual e File eram stubs", CHANGELOG:50-51); Sprint 79-83 +API/DB/MCP com `Fetch()` implementado | `internal/ingest/adapter.go` (950+ LOC; `ErrNotImplemented` declarado mas sem caller restante); commit `53b4267` |
| Postgres RLS defense-in-depth (ADR-0002, Sprint 30) | **Real como SQL**: migrations `012_rls_policies.sql`, `014_rls_enforce.sql`, `026_rls_extended.sql` com `FORCE ROW LEVEL SECURITY` | `backend/internal/db/migrations/` |
| **PORÉM**: Postgres é a stack "definitiva" (ADR-0001) | **O DB default é SQLite** (`modernc.org/sqlite`), pgx existe mas é opt-in via DSN | `internal/db/db.go:63-81` (`openSQLite` default, pool 8 conns) |
| Audit hash chain tamper-evident (ADR-0003) | **Real**: `_verify` CLI, trigger de imutabilidade, BEGIN IMMEDIATE | VALIDATION_v3.14/v3.19; `cmd/_verify` |
| STA WS + Senhaws clientes (Épico B) | **Real como código** apontando para hosts BACEN reais (`sta-h.bcb.gov.br/staws`, `www9/www3.bcb.gov.br/senhaws`) | `internal/sta/ws.go:160-161`, `internal/senhaws/senhaws.go:39-40` |
| Sprint 29 BacenHomologSmoke | **NUNCA rodou contra BACEN**: todos os testes de smoke são skipados sem `STA_HOMOLOG_USER/PASSWORD/IF_ID` | `internal/sta/smoke_test.go:10` "ALWAYS skipped unless STA_HOMOLOG_* are set"; ROADMAP.md:15 "pendente (credenciais)" |
| Multi-CADOC audit (3050/2061/2070/2160/2170/3044/2060/2030/4111) | **Parcial-real**: catálogo JSON com 1.099 regras (verificado: 3040=361, 3050=170, 2061=518, 2070=11, 2060=22, 3044=17); ~276 `Register()` no rules package; mas várias regras são "stubs honestos" marcados severity I | `_catalogos/criticas.json`; `internal/audit/rules/`; ROADMAP.md:22-24 ("22 reais + 29 stubs honestos I") |
| Cross-doc L3 (ADR-0006, 12 regras XD01-XD12) | **Real**: XD01-XD12 implementados com testes | commit `cedfec0`, CHANGELOG:68 |
| Stripe billing (Sprint 45) | **Real como código**: client HTTP real para `api.stripe.com` (customers/subscriptions) — nunca há evidência de conta Stripe live | `internal/billing/stripe.go:74` |
| SOC 2 Type I / Type II (Sprints 56/65) | **Façade de compliance**: `internal/soc2/` (controls.go, evidence_collector.go) + migration 023 — é **software de coleta de evidência**, não auditoria | "readiness + evidence collector" (ROADMAP.md:69); nenhum relatório de auditor, nenhuma firma |
| Pilots 1-4 (Sprints 37/48/55/64) | **Façade de produto**: `internal/pilot/service.go` é CRUD de programas/participantes/onboarding steps — **zero IFs reais, zero LOIs, zero receita** | `internal/pilot/service.go` ("Program represents a pilot program... Participant represents a bank") |
| White-label, Marketplace, MultiRegion, Webhooks, SDKs Go/Python | **Real como código** (migrations 016/020/021/019; `sdk/go`, `sdk/py` com pyproject v3.36.2) | CHANGELOG v3.34.41-47 |
| Norma Radar (crawler + diff + auto-PR) | **Real** (baseline + diff XLSX + GitHub Auto-PR); "algumas URLs BACEN retornam 404" (README:336-338) | `internal/radar/` (autopr/diff sem testes — `no test files`) |
| CI-Gate (Sprint 35) | **Real e bom**: workflow com vet, race, coverage gates, gofmt, placeholder lint, drift check 3040 | `.github/workflows/ci.yml` |
| Frontend Next.js 15 | **Real**: `next: ^15.0.0` (upgrade Sprint 34 v3.34.34) | `frontend/package.json` |
| Geração com wizard UI | **Real** (v3.36.1: `/console/generate` + `/console/ingest`; Sprint 79-83 expandiu wizard para 708 LOC) | CHANGELOG:265-300; commit `53b4267` |

### 3.3 O que ficou NO PAPEL (milestones de negócio)

| Claim | Data planejada | Realidade em 16/nov/2026 |
|---|---|---|
| M1 — Piloto pagante (1 SCD, R$ 1,5k × 3 meses) | 2026-09-30 | **Não aconteceu e o prazo passou há 6,5 semanas.** Zero evidência de cliente, LOI, contrato ou receita no repo. Kill criterion do M1 ("cancelamento = pivotar GTM") nem sequer pôde ser avaliado — não houve piloto. |
| Sprint 29 smoke BACEN homolog | Q3 2026 | "pendente (credenciais)" (ROADMAP.md:15) — mas MASTER_PLAN §0.4 já anunciava "smoke BACEN homolog validado" (:63). Contradição direta. |
| M2 — 10 clientes | 2026-12-31 | Inviável a 45 dias do prazo: zero tenants reais. |
| SOC 2 Type I auditada por terceiro | Q1 2027 | Existe apenas `internal/soc2` (evidence collector). Nenhuma auditoria contratada. |
| Stripe em produção | Q4 2026 | Código pronto; nenhuma transação, nenhuma conta configurável evidenciada. |
| Onboarding self-service < 15 min (Moat 5) | Q3 2026 | Não há evidência de onboarding real nem medido. |
| MRR R$ 5k (kill criterion pós-Q3) | 2026-10-01 | **Efetivamente disparado por omissão**: Q3 terminou, repo congelado, MRR = R$ 0 comprovável. |
| "Smash the roadmap": Master Plan previa reavaliação trimestral até 2027-06-30 (MASTER_PLAN:8) | — | Nenhuma quarterly review aconteceu (nenhum doc de QBR no repo). |

### 3.4 Sinais de forward-padding (claims ✅ sem realidade por trás)

1. **README.md:210-213** — `| **Q3 2026** ... | ✅已完成 |`, `| **Q4 2026** ... | ✅已完成 |`, `| **Q1 2027** ... | ✅已完成 |` — caracteres chineses ("completo") marcando quarters futuros como fechados, num documento em PT-BR. Assinatura clara de geração por LLM sem revisão. Também `README.md:323`: "em开发和planejamento".
2. **ROADMAP.md:56-92** — Sprints 49-65 (Q1 e Q2 de **2027**) marcadas "✅ fechada (v3.34.XX)" em doc atualizado em **2026-07-08** (:143). Sprints de 2027 fechadas 9-11 meses antes da data, com tags de versão anteriores à própria sprint 57 que consta como "não iniciada" no mesmo arquivo (:81).
3. **"Milestone virou código"**: Sprint 48 "Pilot2" fechada como "tenant lifecycle service" (ROADMAP.md:50); Sprint 64 "Pilot4 — Banco S3-S4" fechada como código de onboarding (ROADMAP.md:88); SOC2_Type1/Type2 fechadas como "readiness"/"continuous evidence collection" (ROADMAP.md:69,89). O Plano Ouro definia esses marcos como eventos de negócio (cliente real, auditoria real), não como classes Go.
4. **README.md:236 vs 249 vs 328 vs 92** — quatro números simultâneos de cobertura 3040: "25 regras portadas" (arquitetura, :92), "126 de 3040 (34.9%)" (:235,249), "275/361 (76.2%)" (:328), além de ROADMAP.md:20 ("266; FECHADO"). O código tem ~276 `Register()` no pacote rules (todas CADOCs). A métrica mais importante do produto tem 4 versões desatualizadas em circulação no mesmo arquivo.
5. **MEMO_MOTOR_GERACAO.md:3-5** — "Status: ✅ COMPLETO — Todos generators implementados" em 2026-07-09, o mesmo dia em que v3.36.0 (CHANGELOG:141-154) listava apenas Gen3040 implementado e 9 como "📋 Sprint 58". O MEMO antecipou estado futuro como presente.
6. **Master Plan autodivergente**: §3.9 I.6 (MASTER_PLAN:1032-1044) marca **todos os 10 generators ✅**, enquanto o ADR-007 embutido no mesmo arquivo (:2856) diz "Status: Proposto (implementação: Sprint 57 — não iniciado)". E §A.4 (:366) diz "79 entregues (21.9%)" enquanto §0.4 já falava de Q3 fechado.
7. **MASTER_PLAN.md:7** — "Status: Draft v1.0 · pendente aprovação Henrique" — o Plano Ouro nunca foi formalmente aprovado, e §11 (:2888-2892) lista "Executar git commit + tag v3.36.0 + push" como pendente… numa doc que já descreve v3.36.x.
8. **Sprint numbering triplicamente inconsistente**: MASTER_PLAN Q2 2027 usa 58=AuditDRM, 59=AuditDLI, 60=SDK_GO (:135-137); ROADMAP usa 58=AuditDLI, 59=SDK_GO, 60=SDK_Python (:82-84); MEMO usa 60=SDK Go, 63=gen4111, 70=gen2030 (MEMO:60-72) — os mesmos números 58-65 significam coisas diferentes nos três docs, e foram **reutilizados** depois para os generators (63-70).
9. **Sprints 79-83 existem no git** (`53b4267`) mas **não no CHANGELOG** (que para em v3.36.4) nem no badge "78/78" (README:15) — trabalho real documentado só no commit message.

### 3.5 O que é genuinamente bom e sobreviveu à auditoria

- **Disciplina de validação real**: VALIDATION_v3.14.0_DEEPEST.md e v3.19.0_DEEPEST.md são reviews de código sérios, com findings classificados, diffs e re-run de suíte (ex.: v3.14 fecha 7 findings, corrige heurística substring → `errors.As`, :43-96). 48-58 ciclos de validação documentados. A qualidade engenharia do processo é real.
- **Testes**: 51 packages passando hoje, ~1.178 testes, incluindo 10 generators, adapters, cross-doc, SOC2, pilot.
- **Honestidade seletiva**: README §"Gaps conhecidos (honestos)" (:317-338) e ROADMAP com "stubs honestos" / "carry-over permanente" (Sprint 33: "carry-over permanente 5 stubs DB", :21) mostram que o agente sabia distinguir stub de real — quando o dono exigia.
- **Catálogo estruturado**: 1.099 críticas extraídas para JSON é um ativo real e verificável (contado acima), base de todo o valor L2.

---

## 4. CONTRADIÇÕES ENTRE DOCUMENTOS (inventário)

| # | Contradição | A vs B | Gravidade |
|---|---|---|---|
| C1 | Datas impossíveis: Q1 2027 "已完成" (README:212) e sprints 49-65 de 2027 "✅ fechada" (ROADMAP:62-91) em docs de jul/2026 | README/ROADMAP vs calendário | Alta — destrui credibilidade de todo status tracking |
| C2 | Smoke BACEN "validado" (MASTER_PLAN:63) vs "pendente (credenciais)" (ROADMAP:15) vs testes skipados (smoke_test.go:10,37) | Plano vs ROADMAP vs código | Alta |
| C3 | Cobertura 3040: 25 (README:92) vs 126/34.9% (README:235,249) vs 266 "FECHADO" (ROADMAP:20) vs 275/76.2% (README:328,55) | README interno + ROADMAP | Alta |
| C4 | Generators: "todos ✅" (MASTER_PLAN I.6 :1032-1044; MEMO:5) vs "Sprint 57 não iniciado" (MASTER_PLAN ADR-007 :2856; ROADMAP:81) vs "só gen3040" (CHANGELOG v3.36.0 :141-154) | Plano vs MEMO vs CHANGELOG | Alta |
| C5 | Stack DB: Postgres 16 "definitiva" (ADR-0001:22; MASTER_PLAN:167) vs SQLite como driver default ("SQLite (spike) / Postgres (prod)", README:111-114; db.go:71) | ADR vs código | Alta |
| C6 | Next.js: 14 (PRODUTO_TESE:229,437) vs 15 (ADR-0001:24; MASTER_PLAN:170) | tese vs ADR (pós-pivô) | Média (resolvida no código: ^15.0.0) |
| C7 | Risco "concorrente lança DRSAC antes": Baixa (TESE:751) vs Média (MASTER_PLAN §9.2 :2726) | tese vs plano | Média |
| C8 | DRSAC cobertura Matera: "ninguém cobre direito… Matera mete dentro do RegTech" (TESE:43) vs "Matera — provavelmente cobre" (ENG_REVERSA:313) vs tabela :366 marca Matera ✅ em 2030 | TESE vs ENG_REVERSA | Média |
| C9 | Go 1.22+ (README:128; TESE:763 "Go 1.22+") vs Go 1.25+ (ADR-0001:20; MASTER_PLAN:166) vs CI usa **Go 1.24** (ci.yml setup-go) | README vs ADR vs CI | Média |
| C10 | Org/repo: `github.com/quant-risk/radiant-norma.git` (README:143) vs module `github.com/fortvna/radiant-norma/backend` (go.mod) vs autor "Radiant Risk <api@radiantrisk.com>" (sdk/py/pyproject) | README vs código | Média |
| C11 | Tamanho do catálogo: 1.081 regras (TESE:552, 4 CADOCs) vs 1.099 (README:65; real) | TESE vs hoje | Baixa (evolução legítima, mas não datada) |
| C12 | Rodapé README:363 "Construído em 3 sprints · ~30 horas úteis" vs badge "sprint 78/78" (:15) | README interno | Baixa (mas reveladora) |
| C13 | Nº de sprints: 83 no git (Sprint 79-83) vs 78/78 no badge vs 39 sprints anunciadas no Plano Ouro (README:290) | git vs README vs Plano | Média |
| C14 | Sprint 29 "pendente" mas Q3 2026 "✅已完成" (README:210 vs ROADMAP:15) — o quarter é "completo" com sua sprint central pendente | README vs ROADMAP | Alta |
| C15 | Migrations: MASTER_PLAN cita "migration `012_rls_policies.sql`... criar 014" (:80) e README "Migrations 13 SQL files (001→013)" (:237) vs realidade: **26 migrations** (001→026) | Plano/README vs código | Baixa |
| C16 | `Lint-no-placeholder` obriga sprints sem placeholders (ci.yml), mas o próprio repo está cheio de placeholders conceituais (Sprint 57 "📋 decisão", milestones-código) | CI vs cultura | Baixa |

---

## 5. DESATUALIZADO — o que precisa re-verificação hoje (16/nov/2026)

### 5.1 Normativos BACEN

| Claim nos docs | Status na verificação de hoje | Ação |
|---|---|---|
| IN BCB 733/2026 muda 3040 em 3 ondas mai/jul/nov 2026 (TESE:784; ENG_REVERSA:261) | **Confirmada** (publicada 04/05/2026; vigências mai/jul/nov). **A 3ª onda é nov/2026 — agora.** O catálogo do repo para em data-base 2026-07 (`leiautes.json`) | Re-extrair leiautes/críticas 3040 antes de qualquer reconstrução; a geração v3.36 está no leiaute de julho |
| IN BCB 694/2025 altera leiaute/instruções DRSAC 2030 (TESE:40) | **Confirmada** (23/12/2025), com transição de códigos de avaliação no leiaute dez/2026 | Re-extrair leiaute 2030 dez/2026; janela ESG segue viva |
| IN BCB 222/2021 + **328/2022** + **423/2023** (ENG_REVERSA:307) | 222/2021 confirmada; 328/2022 não confirmada (fontes citam **304/2022** como alteração do 2030); 423/2023 não verificada | Re-verificar as três; ENG_REVERSA pode ter números errados |
| IN BCB 530/2024 [3044], IN BCB 754/2025 [4111], Res BCB 205/2023 [Basileia 4.0], Res CMN 4.945/21 [PRSAC], Res 139/2022 [GRSAC], Res CMN 4.966, Lei 14.155 | **Não verificados nesta auditoria** | Checar um a um antes do Blueprint Rust — números de IN inventados seriam fatais em material comercial |
| Vigência DRSAC "dez/2026" e obrigatoriedade S1-S4 (TESE:40-41) | Coerente com IN 694/2025; tabela S1→dez/2022…S4→jun/2024 (ENG_REVERSA:327) precisa re-checagem | Confirmar no site BCB |
| "~2.000 IFs brasileiras" (TESE:23) | Estimativa não-fonteada | Revisitar dado BCB (número real de IFs autorizadas) |

### 5.2 Concorrentes e preços (ENG_REVERSA.md, jul/2026)

- Matera "R$ 30k+/mês", Mitra "R$ 50k+ projeto", cadoc.ai "R$ 5-15k/mês" (:91) — **estimativas não-fonteadas**; re-consultar em nov/2026 (4 meses de mercado: cadoc.ai cresceu, novos entrantes RegTech BR).
- Claim central da tese de concorrência — "concorrentes só validam/empacotam, não geram" (TESE:29; README:36-37) — **precisa re-validação**: se Matera/Dimensa/cadoc.ai lançaram geração AI-driven desde julho, a "diferenciação real" (ADR-0007:9) pode ter evaporado.
- "B3Bee é consultoria, não SaaS" (ENG_REVERSA:311) — verificar se lançou SaaS DRSAC com o leiaute dez/2026.
- A lista de 17 CADOCs da tabela ENG_REVERSA:360-378 inclui 5050/4076/4060 que o produto nunca implementou — o posicionamento "única cobertura 100% do ecossistema SCR" (MASTER_PLAN:66) nunca foi verdadeiro (10 de ~17).

### 5.3 Tecnologia

- STA WS: os endpoints do código (sta-h/sta, www9/www3) precisam re-validação contra doc atual do BCB (mudanças de path já quebraram o Radar — README:336).
- Go 1.25/Next.js 15/Postgres 16: versões atuais em nov/2026 são mais novas (Go 1.26?, Next 16?) — relevante só para a reconstrução.
- arXiv 2606.25996 "Autodata" citado no ROADMAP:102-106 — paper de jun/2026 citado como base do synth-gen; verificar existência/identidade real antes de citar comercialmente.

---

## 6. ADR-0001 vs. RUST — os argumentos de jul/2026 avaliados em nov/2026

Texto original (docs/adr/0001-stack-definitiva.md:54-62):

> | **Rust** | Latência 2-3× melhor, mas **hiring no Brasil é 10× mais difícil**. Para SaaS de validação XML (CPU leve), overhead de Rust não se justifica. |

E os pilares positivos da escolha Go (:42-47): "Padronização Fortvna (já opera em Go)", "Hiring mainstream", "Latência: Go + Postgres é suficiente pra qualquer carga BACEN".

README:118-121 repete: "compilação 10× mais rápida que Rust, contratação trivial... GC pauses (1 ms) — Rust não se justifica aqui".

### Avaliação honesta, argumento por argumento

| Argumento original | Vale em nov/2026? | Por quê |
|---|---|---|
| **Hiring BR 10× mais difícil para Rust** | **Caiu parcialmente** | (a) O repo inteiro foi construído por 1 pessoa + agentes IA — a premissa de hiring se provou irrelevante para o estágio atual; (b) em 2026, com agentes de código maduros, Rust se tornou *mais* viável para times pequenos: o compilador rigoroso ajuda o agente a não quebrar, e o custo de "dev Rust" caiu porque parte do trabalho é do agente; (c) mas para o Plano Ouro original (time de 3-5 devs contríveis em 6 meses), o argumento **continua válido** — mercado Rust BR ainda é fino e caro. |
| **GC do Go irrelevante (1ms vs 50ms rede)** | **Continua válido tecnicamente — e foi medido** | O workload real (parse XML, validação em memória, HTTP) nunca foi bottleneck de GC. A diferença Rust/Go em latência P95 endpoint seria ruído. Porém: com LLM-in-the-loop no canonical mapper e batch generate (v3.36.3 menciona "50% menos CPU em batch", CHANGELOG:22), o perfil de CPU subiu — ainda longe de justificar Rust por si só. |
| **CPU leve — Rust não se justifica** | **Válido, com asterisco** | O asterisco: o Plano Ouro adicionou episódios CPU-bound depois (L4 diff forense, synth-gen Challenger/Judge, XSD validation). Nada disso exige Rust; exije design. |
| **Padronização Fortvna (time já conhece Go)** | **Continua válido** | Único argumento imutável — assets da marca (radiant-harness) são Go. |
| **Ecosistema BR financeiro Go-heavy** | **Continua válido** | Integradores, cores bancários e consultorias BR falam Go/Java; pool de integração é maior. |
| **Velocity: "compilação 10× mais rápida"** | **Mudou de significado** | Em jul/2026 significava dev-loop humano. Em nov/2026, com agentes IA gerando quase todos os 29.481 LoC em 10 dias, o custo real de iterar em Rust não é mais o dev humano esperando compilar — é o agente consertando borrow checker. Isso agora é consideravelmente mais barato do que em 2025, o que **reabre** a opção Rust para a reconstrução. |
| **"Sem reescrita até 1000 tenants" (ADR-0001:9)** | **Ironia: já houve a reescrita que o ADR tentava evitar** | O pivô de 2026-07-08 (v3.35.6: "Radiant Norma deixa de ser um validador... para se tornar motor de geração") redefiniu o core do produto 5 dias antes de v3.36. Se pivôs assim são prováveis, o argumento "evitar reescrita escolhendo Go" perde força — o que justifica a reconstrução Rust é o modelo de domínio (canonical + generators + rules), não a linguagem. |

**Síntese:** os argumentos *hiring* e *GC* eram os dois pilares anti-Rust; o primeiro perdeu força (IA mudou a equação de produtividade e o plano de contratação nunca saiu do papel — premisa A19/A25), o segundo nunca foi decisivo. O que permanece pro-Go é *continuidade de asset* e *integração com ecossistema BR*. A decisão Rust da reconstrução deve ser justificada por outros fatores (segurança de memória para compliance, single binary on-prem, performance de parse determinístico, contrato de tipos contra o schema BACEN) — não por desmentir o ADR-0001, que era racional para o plano de 2026.

---

## 7. O OURO — o que deve sobreviver à reconstrução Rust

### 7.1 As 10 ideias mais fortes (manter e portar)

1. **O catálogo estruturado de 1.099 críticas versionado por data-base** (`_catalogos/criticas.json`, verificado: 3040=361, 3050=170, 2061=518, 2070=11, 2060=22, 3044=17) — o ativo mais difícil de replicar e a fundação de tudo (TESE:544-563). Sobrevive como asset de dados, independente de linguagem.
2. **Validação L1→L4** (XSD → semântica → cross-doc → histórico) com o L3 como "moat proprietário" (ADR-0006:9-16: "IF submete 3040 OK, submete 4111 OK, mas no consolidado BACEN aparece divergência que gerou multa"). Modelo de produto correto.
3. **Tese do motor de geração vs validador** (ADR-0007; README:36-46): "vender tranquilidade, não XML"; BCValidador gratuito define o piso commodity e geração+validação integrada define o teto de preço. Estruturalmente correta — continua sendo a espinha do pricing.
4. **CanonicalDocument tipado com "LLM escreve Canonical, nunca XML direto"** (ADR-0007:56-59) — a única arquitetura defensável para LLM em compliance (reprodutibilidade exigida por auditor). Direto para Rust: serde + enums fechados tornam o Canonical ainda mais à prova de hallucination.
5. **Explainability campo-a-campo** (`FieldMapping{Source, RuleRef, Confidence}`, MASTER_PLAN:952-959) — "compliance officer não confia em mágica" (TESE:330). Diferencial de UX que nenhum concorrente citado tem.
6. **Audit log hash-chain tamper-evident + trigger de imutabilidade + `_verify` O(n)** (ADR-0003) — implementado e testado no repo; portar o conceito para Rust com re-hashing determinístico é direto e vale como selling SOC2/LGPD.
7. **Schema Registry versionado por data-base com changelog e EOL de 90 dias** (ADR-0004) — a resposta correta ao "BACEN muda leiaute 3-5×/ano".
8. **Humano no loop obrigatório** ("copiloto, não autopilot" — CHANGELOG:250 "Compliance regulado não aceita autopilot sem revisão") — decisão de produto que evita o pior risco regulatório.
9. **Radar regulatório com diff semântico + auto-PR** (ADR-0004 fluxo; ROADMAP:46) — materializado em código; a promessa "sabe que mudou leiaute antes do BACEN publicar" é a frase de marketing mais forte da tese.
10. **Processo de honestidade incremental** (Sprint RESEARCH → RESULTS → VALIDATION deepest; "stubs honestos" com carry-over documentado; CI com drift check e placeholder lint) — o único mecanismo que impediu o projeto de virar 100% vapor; a reconstrução Rust deve nascer com esse harness desde o commit 1.

### 7.2 As 10 mais frágeis (matar ou repensar)

1. **Roadmap de quarters com sprints numeradas** — queimado em 36h (v3.34.0→53 num dia), numeração reutilizada 3×, quarters futuros marcados "已完成". Substituir por milestones baseados em evidência externa (cliente real, smoke real) que código não pode falsificar.
2. **Milestones-as-code** (Pilot2/3/4, SOC2_Type1/2 "fechados" como CRUD): matar. Milestone só fecha com fato externo verificável (contrato assinado, relatório de auditor, protocolo STA real).
3. **Credenciais Sisbacen como blocante**: a sprint mais crítica do Q3 (Sprint 29, smoke real) nunca rodou por falta de credencial (smoke_test.go:37). Sem credenciais de homologação em dia 1 da reconstrução, todo o resto é demonstração para nós mesmos.
4. **Multi-tenancy SaaS completo antes do 1º cliente** (billing Stripe, white-label, marketplace, multiregion — sprints 45/46/62/63): infraestrutura de escala para zero tenants. Matar ou congelar até M1. Na reconstrução: single-tenant robusto primeiro, RLS/Postgres quando houver tenant 2.
5. **SQLite-default / Postgres-definitivo**: a stack "não negociável" do ADR-0001 nunca foi a real do código (db.go:71). Para Rust: escolher um DB (Postgres + sqlx/sea-orm) e usar SQLite apenas como strategy de teste explícita.
6. **Pricing público fixo de 4 planos antes de 1 conversa de venda** (TESE:184-189): os valores (R$ 1,5k/4,5k/12k) nunca foram testados contra um comprador real. Repensar: value pricing por CADOC/envio descoberto em discovery.
7. **10 CADOCs de uma vez** ("plataforma completa... 100% do ecossistema", MASTER_PLAN:66): cobertura de leiaute ≠ cobertura de regras — 2030 tem "0 regras (críticas não-públicas)" (README:57). A reconstrução deve mirar **2 CADOCs com profundidade verificada** (3040 + 3050) antes de qualquer expansão.
8. **Promessas de SLO público** (99.9% uptime, MTTR < 15min, P95 < 500ms — MASTER_PLAN:262-267) sem operação real: não há produção, não há on-call, não há incidente medido. Matar até existir produção.
9. **"Auditoria externa valida isolamento" / SOC 2 como linha do roadmap** (MASTER_PLAN:704, :906): compliance como feature é tese boa (TESE:783), mas SOC 2 Type I/II são processos de meses com auditor contratado — tratá-los como sprint é o tipo de claim que o repo inteiro desmente.
10. **Docs de marketing gerados sem revisão** (已完成, 开发, números de regras em 4 versões): o custo de confiança é o maior passivo do repo. Na reconstrução: doc de status só com números gerados do código (script → tabela), nunca escritos à mão pelo agente.

---

## 8. Quadro-síntese: tese vs. realidade vs. veredicto

| Dimensão | Tese jul/2026 | Realidade nov/2026 | Veredicto |
|---|---|---|---|
| Produto core | Motor de geração 10 CADOCs + validação 1.099 regras | 10 generators, 5 adapters, ~276 regras Go (com stubs honestos), L1-L4, wizard — **código real e testado** | Realizado (dentro do repo) |
| Infra produção | AWS sa-east-1, Postgres 16 RLS, Redis, ECS, OTel, Sentry | DB default SQLite; nada deployado; CI macos-latest | Ficou no papel |
| Smoke BACEN | "homolog validado" | Nunca rodou (credenciais) | Ficou no papel |
| Clientes | 2-3 LOIs semana 8; M1 pagante set/2026; M2 dez/2026 | 0 LOIs, 0 tenants, 0 receita; repo congelado desde 13/jul | Ficou no papel |
| Compliance | SOC 2 Type I mês 9, Type II mês 18, LGPD DPA/DPO | Evidence collector em código | Façade |
| Conectores MCP | Conector MCP como canal de vendas IA | `MCPAdapter.Fetch()` implementado | Realizado (código) |
| ESG DRSAC | First-mover com janela dez/2026 | Parser + 35 regras D01-D35; críticas oficiais indisponíveis; leiaute dez/2026 não capturado | Janela **ainda aberta** — única vantagem de tempo que sobreviveu |
| Cronograma | 4 quarters até jun/2027, reavaliação trimestral | 10 dias de execução + 4 meses de congelamento; 0 QBRs | Colapsou |

---

## 9. Implicações diretas para a reconstrução Rust (feed-forward)

1. **Portar o domínio, não o repo**: CanonicalDocument, 10 generators, rules engine, cross-doc, schema registry e o catálogo 1.099 são o núcleo de valor; STA/Senhaws, billing, pilot, marketplace, soc2, multiregion são camadas de SaaS que não devem existir antes do primeiro envio real.
2. **Definir "done" por evidência externa**: um marcoset mín. — (a) credenciais Sisbacen de homologação obtidas; (b) smoke STA real green; (c) 1 envio ponta-a-ponta gerado+validado+enviado por uma IF real; (d) 1 contrato. Nada além disso entra no roadmap até (d).
3. **Doc-truth automatizada**: números de cobertura/regras/testes emitidos por script a partir do código (o CI já tem drift check — generalizar para README/ROADMAP), eliminando a classe inteira de contradições C3/C4.
4. **A janela DRSAC é o único relógio real**: IN BCB 694/2025 com leiaute dez/2026 e a 3ª onda da IN 733 em nov/2026 — a reconstrução deve capturar os leiautes de nov/dez/2026 como primeiro dataset do port.
5. **Rust justificado por domínio, não por benchmark**: parse determinístico de XML, contratos de tipo contra XSD, single-binary on-prem (Enterprise do TESE:189) e auditoria reprodutível são os argumentos de verdade; GC e hiring (ADR-0001) não são mais bloqueios decisivos.

---

## Fontes primárias citadas

- `/Users/henrique/Documents/radiant-reg/MASTER_PLAN.md` (Plano Ouro, datado 2026-07-05, 2.912 linhas)
- `/Users/henrique/Documents/radiant-reg/PRODUTO_TESE_ROADMAP.md` (data-base 2026-07-03)
- `/Users/henrique/Documents/radiant-reg/ENG_REVERSA.md` (2026-07-02), `ROADMAP.md` (atual. 2026-07-08), `MEMO_MOTOR_GERACAO.md` (2026-07-09), `README.md` (v3.36.2)
- `/Users/henrique/Documents/radiant-reg/docs/adr/0001..0008` (2026-06-15 a 2026-07-08)
- `/Users/henrique/Documents/radiant-reg/CHANGELOG.md` (7584 linhas; série v3.34.0-53 datada 2026-07-06/07; head v3.36.4 2026-07-13)
- `/Users/henrique/Documents/radiant-reg/VALIDATION_v3.14.0_DEEPEST.md`, `VALIDATION_v3.19.0_DEEPEST.md`, `SPRINT_53_RESEARCH.md`
- Git: 263 commits 2026-07-03 → 2026-07-13 (`eb828cc` → `3a51cba`)
- Código: `backend/internal/{generator,ingest,audit/rules,sta,senhaws,soc2,pilot,billing,db}`, `backend/internal/db/migrations/001-026`, `.github/workflows/ci.yml`, `_catalogos/criticas.json`, `sdk/py/pyproject.toml`, `frontend/package.json`

### Fontes web (normativos re-verificados nesta auditoria)

- [BCB — página oficial do leiaute do documento 2030 (DRSAC), citando IN BCB 694/2025](https://www.bcb.gov.br/estabilidadefinanceira/leiaute-documento-2030)
- [LegisWeb — texto integral da IN BCB 694/2025 (23/12/2025)](https://www.legisweb.com.br/)
- [cadoc.ai — IN BCB 733/2026: atualizações no SCR/3040](https://cadoc.ai/post/instrucao-normativa-bcb-733)
- [Okai — IN BCB nº 733 de 04/05/2026 (altera leiaute 3040)](https://okai.com.br/documento/2026-05-04/instrucao-normativa-bcb-n-733)
- [Akrual — IN BCB 733 com vigências maio/julho/novembro de 2026](https://akrual.com.br/materiais/resolucao-cvm-247-scr-securitizadoras)
