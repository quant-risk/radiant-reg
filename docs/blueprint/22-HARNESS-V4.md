# 22 — HARNESS V4: o sistema auto-melhorável (o harness é o produto)

- **Data:** 2026-11-16
- **Status:** Blueprint de reconstrução — consolidado a partir de `research/09-harness.md` (spec principal), `research/07-engine-ia.md` (corpus sintético, judge) e `research/01-backend-audit.md` (paridade golden tests, classe de bug "dispatch silencioso")
- **Arquitetura-alvo:** motor Rust (crates `harness-core`, `harness-gate`, `harness-miner`, `harness-synth`, `harness-cluster`) + Postgres + 1 pessoa operando com agentes IA (ZCode) + CI GitHub Actions
- **Domínio:** CADOCs SCR/COSIF do BACEN (2030, 3040, 3044, 4010, 3026...), STA, catálogo de 1.099 regras de validação

---

## 1. A tese do harness

Para um SaaS regulatório, **o motor não é o moat** — ele sempre será reescrito quando o BACEN mudar o leiaute, e qualquer concorrente com LLM + XSD reconstrói um validador em meses. A vantagem defensável é a **taxa de aprendizado por evento**: cada validação executada, cada correção de cliente, cada mudança regulatória e cada disputa de regra deve converter-se em ativo de avaliação com custo marginal zero. **O Harness é o produto.** O motor é commodity; o pipeline que transforma evento em regressão bloqueável é o ativo que compõe.

### 1.1 As provas

| Prova | O que demonstra | Tradução Radiant |
|---|---|---|
| **Stripe Radar** (flywheel do ML, blog oficial) | Detecção de ataques gera rótulos (bloqueado/permitido) que viram treino do modelo seguinte; resultado público: **-80% em card testing em 2 anos**. O segredo não é o modelo — é que todo evento ambíguo de produção produz um rótulo | Toda discrepância com o BCValidador e toda correção de cliente produzem um `eval_case` automaticamente; publicamos a curva de FN rate caindo mês a mês |
| **SWE-bench** (paper + repo) | Rigor de benchmark: toda instância tem duas dimensões de veredito — `FAIL_TO_PASS` (o fix faz o teste falho passar) e `PASS_TO_PASS` (o fix não quebra o que já passava) + ambiente reproduzível | Todo `eval_case` carrega `expected_before_fail`; todo run verifica amostra de casos vizinhos; mesmo input + mesma versão de regras + mesma engine = mesmo veredito |
| **Duco** (reconciliação, self-learning matching) | A correção do operador ensina o sistema e reduz "false breaks" — o rótulo humano mais barato e confiável que existe, custando o clique que o usuário já daria | O UX approve/correct faz snapshot automático antes/depois: 0 cliques extras viram `eval_case` com provenance |
| **OpenAI Cookbook** (evaluation flywheel) | Loop formal **Analyze → Measure → Improve**: open coding de ~50 traces → categorias → taxa de falha por categoria prioriza o fix; split gold 20/40/40; medir **TPR/TNR** do grader (não accuracy, que engana em sets desbalanceados); quality gates entre cada volta | A triagem semanal segue exatamente esse ciclo; o judge é calibrado por TPR/TNR contra amostra humana; nada melhora sem passar no gate da volta anterior |

**Padrão unificador** — todas as fontes convergem:

```
evento ambíguo de produção → rótulo barato (0–1 clique)
    → ativo de avaliação (eval_case com provenance) → gate de regressão
    → melhoria do motor → mais eventos → mais rótulos → ...
```

O flywheel não é um modelo — é um **pipeline de conversão evento→ativo com provenance e gate**.

### 1.2 Princípios de design (P1–P7)

- **P1 — Toda discrepância é um presente.** O BCValidador oficial é o oráculo disponível em cada submissão; divergência com o motor Radiant é, por definição, um caso de teste que não tínhamos.
- **P2 — Zero esforço marginal.** Capturar custa 0 cliques (automático) ou 1 clique (approve/correct). Curar custa minutos/semana. Avaliar custa zero (cron + CI).
- **P3 — O harness mede a si mesmo.** Mutation Detection Rate, judge TPR/TNR, leak rate e cobertura são scorecards como qualquer outro. Harness sem métricas sobre si mesmo vira green-wash.
- **P4 — Provenance obrigatória.** Nenhum caso entra no corpus sem origem (`production_discrepancy | user_correction | radar_diff | synthetic | audit | canary | golden`). Caso sem provenance é lixo com aparência de dado.
- **P5 — Gates, não dashboards.** Dashboards informam; gates bloqueiam. Regressão impede merge e impede release.
- **P6 — O corpus cresce por cobertura, não por volume.** A unidade de progresso é a matriz regra×edge, não o número de casos.
- **P7 — Separação treino/teste sagrada.** Held-out selado responde só em nightly/pré-release. O que vazou para o motor deixa de medir.

---

## 2. Os 10 mecanismos do flywheel (M1–M10)

```
                        ┌──────────────────────────────────────────┐
                        │            RADIANT HARNESS V4            │
  produção ──► M1 discrepância ──► eval_case ─┐                    │
  produção ──► M2 correção ───────► eval_case ├─► CORPUS ──► M6 gates CI
  Radar ────► M3 diff leiaute ────► suite nova│   (matriz          │
  corpus ───► M4 sintético c/s/j ─► eval_case │  regra×edge)       │
  telemetria► M7 clustering ──► rule_candidate│     │              │
  corpus ───► M8 distillation ──► solver menor│     ▼              │
  corpus ───► M9 red-team/canary ─► held-out  │  M5 scorecards ────┘
  auditoria► M10 amostra humana ──► judge calibrado / harness health
                        └──────────────────────────────────────────┘
```

### M1 — Discrepância → TestCase

- **Gatilho:** submissão validada pelo BCValidador oficial diverge do motor Radiant.
- **Fluxo (máquina de estados):** `detectada → classificada (false_positive | false_negative | message_diff) → eval_case(candidate) → triaged → active → green`
- **Entidades:** `discrepancy` → `eval_case(provenance='production_discrepancy')`; se o BC parece errado, abre-se também `rule_candidate(origin='dispute')`.
- **Regra de ouro:** o veredito do BC é `expected` **mesmo quando o BC parece errado** — é ele que reprova o arquivo do cliente no fim do dia. A disputa vira `rule_candidate`, não altera o caso.
- **Métrica:** discrepâncias/semana; % virando eval_case em <7 dias; **Time-to-Green** (dias até o caso passar).

### M2 — Correção de usuário → par input/expected (evalpair)

- **Gatilho:** cliente corrige um valor no UX approve/correct antes de reenviar.
- **Fluxo:** `correction registrada (snapshot automático before/after + reason opcional) → bifurca: (a) já coberta por regra → eval_case para o eval set do tenant; (b) sem regra no catálogo → rule_candidate(origin='correction', cluster_signature=field_path+pattern) → mineração (M7)`
- **Entidades:** `correction` → `eval_case(provenance='user_correction')` | `rule_candidate`.
- **Por que funciona:** modelo Duco/Grammarly — o rótulo custa o clique que o cliente já daria de qualquer jeito.
- **Métrica:** correção→eval_case 100% automático; % de correções que revelam rule_candidate.
- **Telemetria de retornos STA (opt-in, válida também para M7):** a ingestão de retornos do STA (`ListDisponiveis`/`Download`) só acontece com **cláusula contratual opt-in por tenant**; antes de virar fixture, **sanitização zero-PII obrigatória** — detector **Aho-Corasick** sobre os campos declarados do leiaute + **canary tokens** provando que nada escapou; **FNR > 0** (qualquer falso-negativo medido pelos canários) dispara alerta **critical** e bloqueia a fixture.

### M3 — Radar detecta mudança → regression suite da nova versão

- **Gatilho:** Radar normativo detecta Circular/comunicado novo ou diff de leiaute/Regras de Validação publicadas pelo BCB.
- **Fluxo:** `radar_event → layout_diff (campo novo/removido, domínio alterado, regra nova no XLS→CSV) → agente gera: 1) propriedades proptest atualizadas; 2) goldens velhos marcados vigencia_fim=hoje; 3) N casos sintéticos cobrindo o diff (M4); 4) suite "regressão vYYYYMM" versionada por vigência → implementação só está DONE quando suite da nova vigência = 100% green E goldens antigos congelados provam reprodução das versões anteriores`
- **Entidades:** `radar_event` → `layout_diff` → `eval_case`/`property_test` com `vigencia`.
- **Chave de design:** suites **versionadas por vigência** (como o BACEN faz com os leiautes); o runner executa suite da vigência corrente + suite "eterna" de invariants.
- **Métrica:** dias entre publicação do BCB e suite green; propriedades geradas por diff.

### M4 — Corpus sintético challenger/solver/judge cresce por cobertura

- **Gatilho:** célula vazia na **matriz regra×edge** (classes: `boundary` limite ±1, `negative` deve rejeitar, `mutation` golden com 1 campo mutado, `real_world` padrões de produção anonimizados, `combo` interação de 2+ regras).
- **Fluxo:** `célula vazia (regra R × edge E) → CHALLENGER (LLM barato) gera payload mínimo que ativa R em E, com perturbações estilo OpenAI Cookbook (ruído, fora-de-faixa, zeros, NULL, Unicode) → SOLVER (motor Rust determinístico) executa → JUDGE em 2 estágios: (1) determinístico — engine + XSD dão o ground truth do veredito; (2) LLM com rubrica atômica por item (contra verbosity bias) valida se o caso é legal, minimalista e cobre a célula → PASS: eval_case(provenance='synthetic', cell=R×E) | FAIL: challenger re-tenta com feedback do judge (máx 3x)`
- **Entidades:** `eval_case(synthetic)`; a matriz é derivada de `rule` × enum de edges.
- **Regra do 07-engine-ia:** rejection sampling só é confiável com verificador objetivo — aqui o XSD + engine Rust são o juiz final; LLM só julga semântica do caso (~10% de amostragem).
- **Métrica:** **Rule Coverage Index** (RCI) = % de células com ≥1 caso; challenger acceptance rate; bugs novos gerados por célula vazia.

### M5 — Scorecards versionados (append-only)

- **Gatilho:** todo `eval_run` concluído.
- **Fluxo:** `run finished → scorecard append-only chaveado por (doc_cod, tenant_id, ruleset_version, engine_version, run_id) → alimenta gate M6 e o deliverable mensal do cliente (Seção 7)`
- **Conteúdo:** precision/recall por regra (contra BCValidador + correções confirmadas); `f2p_pass_rate` e `p2p_pass_rate` (linguagem SWE-bench); `false_negative_rate` (o pior erro regulatório: passar arquivo que o BACEN reprova); latency p50/p95; cost_tokens; `held_out_score`.
- **Métrica:** nenhum scorecard é sobrescrito — a série histórica É o produto de auditoria.

### M6 — Gates de CI com 3 tiers

- **Gatilho:** PR aberto (`--tier pr`), cron noturno (`--tier nightly`), pré-release (`--tier release`).
- **Fluxo:** `PR → fast suite (~500 cases, <90s) + mutation incremental (cargo-mutants só nos arquivos diffados) + property suite (proptest, 30s) → compara contra baseline = último release green → 3 blocadores: F2P regression | P2P break | precision/recall caiu >1pp em regra tocada pelo diff → nightly: corpus completo + held-out selado + canários → release: held-out + canários + scorecard assinado`
- **Entidades:** `eval_run(trigger='ci')` + `eval_result`; thresholds por regra na config do `harness-gate`.
- **Regra:** thresholds **só podem apertar**; afrouxar exige PR explícito com justificativa — o gate nunca é ajustado no calor do momento.
- **Mutation-score gate do wedge V1 (fusão 16/11/2026):** além do cargo-mutants de código, o wedge roda mutations **de dados** no gate de CI: cada mutação altera **1 campo/regra por vez** e **exige** erro com o `rule_code` **exato** esperado — mutante que dispara erro genérico (ou nenhum) conta como sobrevivente. **Mutation Score = 100%** e **Stub Ratio = 0%** são hard gate (runner `reg-cli mutate` · `tests/mutations.rs`).

### M7 — Discovery de regras desconhecidas por clustering

- **Gatilho:** volume crescente de `correction`/`discrepancy` sem `rule_id` mapeado.
- **Fluxo:** `telemetria → embeddings de (doc_cod, field_path, before→after, mensagem) → HDBSCAN/UMAP + BERTopic (padrão IEEE de descoberta de modos de falha) → cluster com ≥8 observações e ≥2 tenants sem regra mapeada → rule_candidate(origin='error_cluster', hypothesis=LLM sobre 10 exemplos) → triagem semanal → vira draft de regra no catálogo + N casos sintéticos (M4)`
- **Entidades:** `rule_candidate` (UNIQUE em `cluster_signature` para dedup) → `rule`.
- **Por que é o mecanismo mais valioso:** é o único que gera **receita de catálogo** — "o cliente corrigiu X três vezes sem regra associada" significa que existe uma regra implícita no mercado que o catálogo não tem.
- **Métrica:** regras descobertas/mês; **shadow-rule rate** (% da produção com erro sem regra mapeada).
- **Restrição de fonte (idem M2):** telemetria de retornos STA (`ListDisponiveis`/`Download`) entra no clustering apenas sob **opt-in por tenant** e depois da **sanitização zero-PII** (Aho-Corasick + canary tokens; **FNR > 0 = alerta critical**) — cluster só sobre dado higienizado.

### M8 — Distillation por gatilho (não por vaidade)

- **Gatilhos por regra ou solver:** regra com ≥200 goldens e mensagem ainda gerada por LLM → destilar gerador de mensagem; solver LLM-auxiliado com ≥1.000 pares input→veredito do oráculo → fine-tune do classificador de erros ambíguos; judge com TPR/TNR ≥95% estável por 8 semanas e ≥500 decisões auditadas → congelar como "silver standard" gerador de dados; custo LLM >US$200/mês com padrão repetitivo → destilar componente quente.
- **Fluxo:** `gatilho atingido (medido pelo harness, não declarado) → modelo destilado entra como engine_version CANDIDATA → A/B no tier release, held-out decide → sobe ou descarta`
- **Guardrail:** nunca sobe direto; o corpus que a destilação consumiu fica congelado como P2P do novo componente.

### M9 — Red-team contínuo (canary + adversarial + mutation)

- **Canary issues:** N casos com veredito conhecido injetados **aleatoriamente** em cada run; canário errado invalida o run inteiro (sela contra vazamento e detecta runner quebrado). Canários têm **formato distinto do corpus** para detectar eval-awareness (algum componente "reconhecendo o eval").
- **Adversarial semanal:** challenger ataca as 10 regras com pior recall histórico + payloads malignos (campos deslocados, encoding traiçoeiro, válidos-que-parecem-inválidos).
- **cargo-mutants:** incremental em PR, completo na main semanalmente; **Mutation Detection Rate <70% em qualquer crate do motor = issue automática**.

### M10 — Auditoria amostral + calibração do judge

- **Gatilho:** semanal, 30 casos sorteados estratificados por `doc_cod × veredito × provenance` (30–45 min, UI de 1 clique).
- **Fluxo:** `amostra estratificada → veredito humano → (a) judge_calibration → TPR/TNR do judge publicado no scorecard do próprio judge; (b) disputes → rule_candidate; (c) alimenta o split 20/40/40 do gold standard (OpenAI Cookbook)`
- **Judge versionado como código:** prompt + modelo + rubric no git; trocar judge = PR comparando TPR/TNR contra a amostra auditada acumulada. Vieses tratados como intervenção de medição, não ajuste de prompt: position bias (comparar nas duas ordens), verbosity bias (rubrica atômica), self-preference (judge de família diferente do solver; ensemble de 2 judges baratos).

---

## 3. Modelo de dados (Postgres — DDLs resumidos de `research/09`)

```sql
-- Regra como artefato executável versionado (padrão Norm AI)
CREATE TABLE rule (
  id UUID PK, cod TEXT UNIQUE,             -- 'V2030-0012'
  doc_cod TEXT, descricao TEXT, fonte TEXT,-- 'Circular 3.995, art. 12, II'
  categoria TEXT CHECK (IN 'leiaute','negocio','consistencia','mensagem'),
  vigencia_inicio DATE, vigencia_fim DATE,
  status TEXT CHECK (IN 'draft','active','deprecated','disputed')
);
CREATE TABLE rule_version (
  id UUID PK, rule_id FK(rule), semver TEXT,
  spec JSONB NOT NULL,                     -- AST/predicate executável
  changed_by TEXT,                         -- provenance da mudança: 'radar_event:812' | 'rule_candidate:34'
  UNIQUE (rule_id, semver)
);

-- A unidade atômica do harness
CREATE TABLE eval_case (
  id UUID PK, doc_cod TEXT, vigencia DATE,
  input JSONB, expected JSONB,             -- {should_pass, expected_errors:[{rule_cod,field,message_regex}]}
  provenance TEXT NOT NULL CHECK (IN 'production_discrepancy','user_correction',
                                   'radar_diff','synthetic','audit','canary','golden'),
  source_ref UUID,                         -- discrepancy/correction/radar_event de origem
  rule_cods TEXT[], edge_class TEXT CHECK (IN 'boundary','negative','mutation','real_world','combo'),
  weight NUMERIC, is_held_out BOOLEAN,
  expected_before_fail BOOLEAN,            -- semântica FAIL_TO_PASS do SWE-bench
  status TEXT CHECK (IN 'candidate','triaged','active','retired'),
  created_by TEXT, created_at TIMESTAMPTZ
);
-- índices: (doc_cod, vigencia, status) + GIN(rule_cods)

CREATE TABLE eval_run (
  id UUID PK, trigger TEXT CHECK (IN 'ci','nightly','pre_release','manual'),
  tier TEXT,                               -- pr|nightly|release
  engine_version TEXT, ruleset_version TEXT, judge_version TEXT, commit_sha TEXT,
  corpus_hash TEXT, fingerprint TEXT,      -- Honesty Engine (Seção 4.3)
  summary JSONB                            -- f2p, p2p, precision, recall, canary_ok
);
CREATE TABLE eval_result (
  run_id FK(eval_run), case_id FK(eval_case),
  verdict TEXT CHECK (IN 'pass','fail','error','canary_fail'),
  actual JSONB, latency_ms INT, tokens INT, PK (run_id, case_id)
);

CREATE TABLE discrepancy (                 -- M1: BCValidador vs motor
  id UUID PK, tenant_id TEXT, submission_id UUID, doc_cod TEXT,
  bc_errors JSONB, our_errors JSONB,
  delta_class TEXT CHECK (IN 'false_positive','false_negative','message_diff'),
  resolved_eval_case_id FK(eval_case), created_at TIMESTAMPTZ
);
CREATE TABLE correction (                  -- M2: correção do usuário
  id UUID PK, tenant_id TEXT, submission_id UUID, doc_cod TEXT,
  field_path TEXT, before TEXT, after TEXT, reason TEXT,
  auto_eval_case_id FK(eval_case), created_at TIMESTAMPTZ
);
CREATE TABLE rule_candidate (              -- M7 discovery + disputas M1/M2
  id UUID PK, origin TEXT CHECK (IN 'error_cluster','discrepancy','correction','dispute'),
  cluster_signature TEXT UNIQUE,           -- dedup
  n_observations INT, tenants TEXT[], hypothesis TEXT, examples JSONB,
  status TEXT CHECK (IN 'open','triaged','promoted','rejected'),
  promoted_rule_id FK(rule), created_at TIMESTAMPTZ
);
CREATE TABLE scorecard (                   -- M5: append-only, nunca sobrescrito
  id UUID PK, run_id FK(eval_run), doc_cod TEXT, tenant_id TEXT,
  ruleset_version TEXT, engine_version TEXT,
  metrics JSONB NOT NULL,                  -- precision, recall, fn_rate, p95, held_out
  created_at TIMESTAMPTZ
);
CREATE TABLE canary_seed (                 -- M9
  id UUID PK, payload JSONB, expected_verdict TEXT,
  active BOOLEAN, inserted_in_run UUID NULL
);
CREATE TABLE judge_calibration (           -- M10
  id UUID PK, judge_version TEXT, case_id FK(eval_case),
  judge_verdict TEXT, human_verdict TEXT, run_batch TEXT, created_at TIMESTAMPTZ
);
```

**Provenance obrigatória:** `eval_case.provenance`, `eval_case.source_ref`, `rule_version.changed_by`, `eval_run.fingerprint` — a trilha "de onde veio cada caso e cada mudança de regra" é o que torna o corpus auditável (e vendável, Seção 7). Caso sem provenance não passa da triagem.

---

## 4. Honesty Engine — como o harness não virar green-wash

### 4.1 Os 5 modos de falha e contramedidas

| Modo | Como acontece | Contramedida |
|---|---|---|
| **Overfit ao corpus** | Fix passa nos casos que o motivaram e quebra vizinhos | PASS_TO_PASS amostral em todo run; corpus cresce via challenger, que não conhece o fix |
| **Vazamento held-out** | Casos selados vazam para desenvolvimento/prompts | Held-out criptografado, aberto só no runner; held-out leak rate monitorado (alvo 0); canários com formato distinto do corpus |
| **Judge virar alvo** | Sistema otimiza para o judge, não para a verdade | Judge de família diferente do solver; calibração vs amostra humana (TPR/TNR); rubricas atômicas; ensemble de 2 judges |
| **Oráculo capturado** | Confiança cega no BCValidador mesmo quando ele muda/merra | Disputas registradas (`rule_candidate`); histórico de veredito do oráculo por versão; casos "oráculo disse X" versionados por data |
| **Inflação por seleção** | Corpus só com casos fáceis que o motor já passa | Matriz regra×edge obriga células negativas/adversárias; auditoria amostral estratificada mede a taxa real em produção |

### 4.2 Harness Health Scorecard (o harness avaliado como produto)

| Métrica | Definição | Alvo | Alarme |
|---|---|---|---|
| Mutation Detection Rate | % de mutantes mortos (cargo-mutants) | **≥75%** | <70% |
| Judge TPR / TNR | vs amostra humana auditada | **≥95%/95%** | <90% |
| Canary pass rate | canários com veredito correto no run | **100%** | <100% (run inválido) |
| Held-out leak rate | casos held-out citados em prompts/repo | 0 | >0 |
| Rule Coverage Index | % células regra×edge com ≥1 caso | **≥80%** das regras ativas | <60% |
| Shadow-rule rate | % produção com erro sem regra mapeada | <5% | >10% |
| Audit agreement | humano concorda com o motor na amostra | ≥97% | <95% |
| Time-to-Green p50 | discrepância→caso green | **≤7 dias** | >14 dias |

A auditoria humana (M10) fecha o ciclo: é a única fonte não sintética de verdade sobre o harness. Sem ela, todas as métricas acima são autorreferentes.

### 4.3 Fingerprint de eval_run

Todo `eval_run` registra `engine_version + ruleset_version + judge_version + corpus_hash` (colunas `fingerprint`/`corpus_hash`). Score só é comparado entre fingerprints compatíveis — leitura direta da lição de "harness fingerprinting" do SWE-bench: um score que conflate modelo + harness + leakage não significa nada. Isto impede os dois crimes mais comuns: comparar run de corpus diferente como se fosse progresso, e medir com judge diferente sem recalibração.

---

## 5. Integração com o motor Rust

### 5.1 Catalog completeness gate — a ponte build ↔ harness

O build do `radiant-rules` (catálogo compilado) carrega um **gate de completeza**: para **cada uma das 1.099 regras** do catálogo BACEN, o código deve declarar **exatamente um** dos três estados:

1. `IMPLEMENTED` — existe `rule_version` executável no motor (código Rust tipado);
2. `DATA_ENTRY` — regra data-driven aplicada via tabela de críticas (sem código próprio);
3. `GAP { risk }` — explicitamente sem cobertura, com risco estimado (severidade da crítica × exposição do doc × tenants afetados).

O build **falha** se alguma regra não tem marcação. Consequências:

- **A classe de bug "dispatch silencioso" morre por construção.** No Go atual (`audit/service.go:758-759`), regra ausente = skip silencioso; ~511 regras implementadas para 3050/2070/2061/2160/2170/3044/2060 nunca são despachadas, e 153 stubs `return nil` contam como cobertura. Em Rust, enum tipado + match exaustivo + o gate tornam isso erro de compilação ou de build — nunca de produção.
- **O GAP vira fila de trabalho priorizada por risco**, ordenada automaticamente pelo harness: gaps de docs com maior volume de submissões de tenants e severidade "A" sobem para o topo; cada gap fechado é, por definição, um lote de casos F2P (o caso que prova o gap virando implementação já nasce no corpus via M4).
- O gate emite o **Rule Coverage Index** que o Honesty Engine monitora: completeza declarada (build) × cobertura real medida (matriz regra×edge) precisam convergir — divergência entre os dois é alarme de honestidade.

### 5.2 Os 1.178 testes Go como seed inicial do corpus

O backend Go tem **32.731 LoC de teste em 1.178 funções verdes** — a maior coleção de vereditos corretos que existe sobre o domínio. Conversão em três passos:

1. **Extração:** cada fixture XML/JSON + expectativa dos testes Go vira `eval_case(provenance='golden', source_ref=<arquivo_test>:<func>)` — provenance aponta para o teste original, mantendo a trilha auditável.
2. **Filtro de honestidade:** casos que exercitam os 153 stubs severity-I (`return nil`) são marcados `weight` reduzido e anotados — eles provam comportamento stub, não semântica; a regra correspondente entra como `GAP` no gate de completeza. Casos de regras não-despachadas (3050 etc.) entram com `expected` derivado do **catálogo BACEN** (a verdade do produto), não do wiring Go quebrado.
3. **Importação:** `harness-miner` ingere o lote, triagem em batch (agente + aprovação humana 1-clique), corpus ativo na semana 4.

### 5.3 Paridade Go↔Rust como primeira suite

O motor Go é a **spec comportamental executável**: cada XML de teste roda nos dois motores e o veredito é diffado. A suite de paridade é o F2P do port:

- **Definição de "portado":** uma regra só está portada quando a paridade dela = 100% nos casos golden — medido pelo `harness-gate`, não declarado à mão.
- Regras com semântica dupla no Go (os dois motores L1–L4, colisão de `XD02`) são resolvidas **antes** do port com decisão documentada no `rule_version.changed_by` — a paridade força a decisão a ser explícita.
- Depois do port, a suite de paridade é aposentada com honras: os casos permanecem no corpus como `golden` P2P permanente do motor Rust.

### 5.4 O que o harness caça por design (herança da auditoria)

| Classe de bug do Go | Como o harness caça no Rust |
|---|---|
| Dispatch silencioso (`return nil` no `applyRegra`) | Gate de completeza + propriedade proptest: "toda crítica do catálogo produz erro OU gap explícito — nunca OK silencioso" |
| Chaves de críticas quebradas (`2061-DLO` vs API de 4 dígitos) | Canary por doc: payload desenhado para disparar ≥1 regra de cada CADOC; doc que nunca dispara regra = alarme |
| XSD degradando silenciosamente (paths relativos) | Canary de leiaute: arquivo que viola XSD **deve** falhar L1; se passar, run inválido |
| Defaults silenciosos no generator (`UF="SP"`) | Caso `negative` na matriz: payload sem UF **deve** ser rejeitado ou avisado |
| L3 cross-doc no-op | Casos `combo` com veredito do oráculo: cross-doc que nunca reports = F2P permanente |

### 5.5 Shadow Mode — mecanismo de migração do legado Go (SMER)

A paridade (§5.3) compara os motores sobre corpus fixo; o Shadow Mode compara sobre a
**produção real dos pilotos** enquanto o legado ainda responde:

- **`shadow-diff`:** em cada remessa, o wedge roda em paralelo ao motor legado Go e compara os XMLs com **canonicalização C14N** + **diff linha-a-linha** — nunca diff textual ingênuo (ordem de atributos e whitespace não são drift).
- **Classificação obrigatória de cada divergência:** `COSMETIC` (equivalente sob C14N, só formação) · `DATA_DRIFT` (dado difere — investigar a fonte) · `LEGACY_BUG` (o Go erra e o Radiant acerta — nasce como caso dourado) · `RADIANT_GAP` (o Radiant erra — F2P imediato).
- **Gate de desligamento:** **SMER = 100%** (Shadow Match Equivalence Rate) antes de desligar o motor legado — nenhum `DATA_DRIFT`/`RADIANT_GAP` em aberto na janela de observação.

### 5.6 Corpus dourado complementar — oráculos E2E do legado e retornos STA reais

- **85 oráculos E2E do legado:** o run congelado `e2e-audit/20260715-170152-3a51cba/runtime/benchmark-results.json` (**baseline documentado de 41,76% — NO-GO**) entra como **seed do Loop 3** do corpus dourado, com o runner E2E **reconstruído em S6** (FASE 1 — `25-ROADMAP-EXECUCAO.md` §0). Os 85 casos viram `eval_case(provenance='golden')` com `source_ref` apontando o run original — a paridade não recomeça do zero.
- **Retornos STA reais (anonimizados):** fixtures de retornos aceitos/rejeitados do STA/CRD entram como **casos de integração**, sempre após o gate zero-PII de M2/M7 (opt-in + Aho-Corasick + canary tokens).

---

## 6. Operação com 1 pessoa + agentes

### 6.1 Rotinas

| Cadência | Quem | O quê |
|---|---|---|
| Contínuo | `harness-miner` (worker Rust) | Eventos (discrepancy, correction, radar_event) → dedup por hash do payload normalizado → `eval_case(candidate)` na fila de triagem |
| Noturno 03:00 | cron `eval-nightly` | Corpus completo determinístico + held-out selado + canários embaralhados + challenger nas 20 piores células + judge LLM só nos ~30% não determinísticos + scorecards append-only + relatório de 1 página com diffs vs D-1 |
| Semanal | Agente ZCode (triagem) | Classifica candidatos: válido? minimal? regras certas? held-out? — prepara batch para aprovação |
| Semanal | Humano (30–45 min) | **Aprovação em batch, UI de 1 clique** (approve/bulk): é o único toque humano obrigatório do flywheel |
| Semanal | Agente ZCode (red-team) | Adversarial nas 10 piores regras; auditoria amostral de 30 casos preparada para o humano |
| Por PR / release | GitHub Actions | `harness-gate --tier pr/nightly/release` bloqueando merge por F2P/P2P/precision-recall |

### 6.2 Custo (~nov/2026, 1 pessoa, ~10 tenants)

| Item | Volume | Custo/mês |
|---|---|---|
| eval-nightly determinístico (Rust, 3–5k cases) | 30 runs | ~0 (VM própria, <10 min/run) |
| Judge LLM (~30% dos cases, modelo classe Haiku/GLM) | ~1k calls/noite | ~US$75–100 |
| Embeddings + clusterização (local, modelo pequeno) | contínuo | ~US$5 |
| Challenger sintético (LLM médio, 200 casos/dia) | diário | ~US$30–60 |
| cargo-mutants completo | 4x/mês | ~0 (CPU local) |
| Destilação/fine-tune eventual (gatilho M8) | 1–2x/ano | US$100–300 por evento |
| **Total recorrente** | | **~US$110–170/mês** |

O flywheel é barato porque **70%+ do corpus é determinístico (Rust) e o LLM só aparece onde existe ambiguidade** — o juiz final do corpus sintético é o XSD + engine, custando zero por run.

---

## 7. Como o harness vira receita

**Mecânica de pricing:**

```
Base (por tenant):    mensageria + validação + scorecard mensal
Usage:                por submissão/validação — o volume alimenta o flywheel;
                      quem mais usa, mais o sistema aprende, e paga proporcionalmente
Premium:              "Accuracy SLA" — FN rate < X% contratual
Network effect:       regra descoberta em um tenant (M2/M7) protege todos no próximo release
```

1. **Accuracy SLA / "Zero Autuação™".** FN rate contratual — passar arquivo que o BACEN reprova é o pior erro regulatório. O SLA é **respaldado pelo held-out selado + auditoria amostral estratificada**: só um time que tem Honesty Engine (Seção 4) consegue oferecer isso sem mentir. É a materialização comercial da tese: o concorrente pode copiar o motor em meses; não pode copiar o corpus + gates + calibração que respaldam um SLA.
2. **Scorecard mensal como deliverable.** O "Relatório de Confiabilidade" por tenant (scorecard assinado, append-only) é o produto visível do harness: curva de FN rate caindo mês a mês com uso — o enquadramento do Stripe Radar ("80% de redução") aplicado ao cliente: "sua taxa de reprovação caiu X%; o Radiant aprendeu N regras novas com o mercado este mês". Scorecard assinado + changelog de regras é também **compliance story**, não só qualidade (padrão Regnology "governed automation").
3. **Benchmarking tier opt-in.** Tenants que aderirem ganham comparação anonimizada com pares (mesmo porte/tipo de IF) sobre precision/recall por doc — só viável porque o corpus e os scorecards já existem; vira mais um incentivo para o cliente corrigir (cada correção melhora seu próprio scorecard e o benchmark do mercado).
4. **Pitch central:** *"Cada submissão que você faz deixa o Radiant — e todo o mercado que o usa — menos propenso a errar a próxima."* A rede inteira aprende com cada evento; quem não está na rede não tem nem os rótulos.

---

## 8. Roadmap de 6 fases

Cada fase tem critério de pronto **emitido pelo próprio harness** (o harness é quem mede — nada é "declarado pronto à mão").

| Fase | Semana | Entregas | Critério de done (medido pelo harness) |
|---|---|---|---|
| **F0 — Gate de CI** | 1–2 | Schema Postgres; `harness-core` runner com F2P/P2P; goldens iniciais; proptest de leiaute; `harness-gate --tier pr` no GitHub Actions | O gate **bloqueia um PR real**: ao menos 1 merge impedido por regressão F2P ou P2P registrada em `eval_run(trigger='ci')` |
| **F1 — Corpus seed** | 3–4 | Conversão dos 1.178 testes Go em `eval_case(provenance='golden')`; suite de paridade Go↔Rust; filtro de stubs severity-I | `harness-gate` reporta **≥1.000 casos active com provenance='golden'** e paridade rodando no tier pr |
| **F2 — Miner** | 5–6 | M1 discrepância→case; M2 correção→case (snapshot automático no UX); worker contínuo; triagem semanal agente + batch 1-clique | **≥1 caso novo/semana com provenance de produção** por 2 semanas consecutivas; Time-to-Green medido no relatório nightly |
| **F3 — Synth** | 7–8 | M4 challenger/solver/judge + matriz regra×edge; canários; held-out selado; M6 gates completos (precision/recall por regra) | RCI **≥60%** computado pelo gate; canary pass rate **100%** em 7 runs nightly consecutivos |
| **F4 — Cluster discovery** | 9–10 | M7 clusterização; M10 auditoria amostral + judge calibration; dashboards; cargo-mutants em CI; gate de completeza do catálogo no build | **Harness Health Scorecard no ar no nightly** (todas as 8 métricas da Seção 4.2); RCI ≥80%; primeiro `rule_candidate(origin='error_cluster')` promovido a draft de regra |
| **F5 — Accuracy SLA** | 11–12+ | M8 destilação quando o gatilho disparar; SLA contratual; scorecards mensais como deliverable; benchmarking opt-in | **FN rate medido em held-out por 4 semanas consecutivas** abaixo do threshold contratual + primeiro scorecard assinado entregue a tenant piloto |

Dependência crítica entre fases: F2 depende do gate (F0) — sem regressão bloqueável, caso novo não tem valor. F5 depende do Honesty Engine (F4) — SLA sem auditoria amostral é promessa sem lastro.

---

## Fontes internas

- `docs/blueprint/research/09-harness.md` — spec completa (DDLs, 10 mecanismos, Honesty Engine, custo, roadmap)
- `docs/blueprint/research/07-engine-ia.md` — corpus sintético challenger/solver/judge, verificador determinístico como juiz final, custos de LLM
- `docs/blueprint/research/01-backend-audit.md` — 1.178 testes Go, ~511 regras não-despachadas, 153 stubs, classe "dispatch silencioso", plano de paridade

Fontes externas principais (ver lista completa no research/09): Stripe Radar flywheel · SWE-bench (paper/repo) · OpenAI Cookbook evaluation flywheel · Duco self-learning matching · cargo-mutants · proptest · Openlayer/Langfuse/promptfoo CI gates · Norm AI · Regnology RGI.
