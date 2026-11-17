# 09 — Radiant Harness v4: O Harness que Aprende

> **Tese:** Para um SaaS regulatório, o motor não é a vantagem defensível — ele sempre será reescrito quando o BACEN mudar o leiaute. A vantagem defensível é a **taxa de aprendizado por evento**: cada validação executada, cada correção de cliente, cada mudança regulatória e cada disputa de regra deve converter-se em ativo de avaliação com custo marginal zero. O Harness é o produto.

- **Data:** 2026-11-16
- **Status:** Spec para implementação (substitui conceitos anteriores de harness)
- **Domínio:** CADOCs SCR/COSIF do BACEN (docs 2030, 3040, 3044, 4010, 3026...), mensageria via STA, leiautes e Regras de Validação oficiais publicadas pelo BCB
- **Stack alvo:** Rust + Postgres + agentes IA (ZCode workflows/subagentes) + 1 pessoa

---

## 1. Estado da arte — o que os melhores times fazem

### 1.1 Eval-Driven Development (EDD)

| Prática | Fonte | O que copiar |
|---|---|---|
| Escrever o eval **antes** do código; evals como "especificação viva" | [LangChain — What Is Eval-Driven Development?](https://www.langchain.com/blog/what-is-eval-driven-development) | Todo issue de regra nasce com `eval_case` anexado; regra sem caso não sai de draft |
| Produção → dataset: conectar logs reais aos evals | [Braintrust — Eval feedback loops](https://www.braintrust.dev/blog/eval-feedback-loops) | Pipeline de mineração de traces para o corpus |
| Falha de produção → regression test | [LangSmith — Regression Testing](https://www.langchain.com/blog/regression-testing), [Braintrust vs LangSmith](https://www.braintrust.dev/articles/langsmith-vs-braintrust) | Mecanismo M1/M2 desta spec |
| Flywheel de 3 fases: **Analyze → Measure → Improve**; open coding (~50 traces rotulados livremente) → axial coding (categorias) → taxa de falha por categoria prioriza o fix; split gold 20% train / 40% validation / 40% test; medir **TPR/TNR** do grader (não accuracy, que engana em sets desbalanceados) | [OpenAI Cookbook — Building resilient prompts using an evaluation flywheel](https://developers.openai.com/cookbook/examples/evaluation/building_resilient_prompts_using_an_evaluation_flywheel) | Seção 7 (Honesty Engine) e calibração de judge |
| Expansão sintética por tuplas de dimensões (Channel × Intent × Persona) + perturbações (ruído, gíria, irrelevância) | idem OpenAI Cookbook | Mecanismo M4 (matriz regra×edge) |
| Evals de agentes em 9 passos; "regra dos 100 itens" (≥100 casos por capability antes de confiar) | [Anthropic — Demystifying evals for AI agents](https://www.anthropic.com/research/demystifying-evals-for-ai-agents), [Anthropic — Building strong safety evaluations](https://www.anthropic.com/research/building-strong-safety-evaluations) | Threshold de maturidade por regra (Seção 6.6) |
| Gate de CI que bloqueia merge por regressão de métrica | [Openlayer — CI/CD Evaluation Gates](https://www.openlayer.com/blog/cicd-eval-gates-block-merges-model-failure), [Langfuse — Experiments in CI/CD](https://langfuse.com/docs/evaluation/experiments/experiments-ci-cd), [promptfoo — CI/CD](https://www.promptfoo.dev/docs/integrations/ci-cd) | Mecanismo M6 |

**Síntese:** o padrão premium consolidado é **trace de produção → mineração de falhas → dataset com provenance → grader automático → gate em CI → monitoramento reinicia o loop**. Nenhuma plataforma vende "o harness"; todas vendem o loop.

### 1.2 Harnesses estilo SWE-bench

O harness do SWE-bench ([repo oficial](https://github.com/swe-bench/SWE-bench), [paper](https://arxiv.org/abs/2310.06770)) é a referência de rigor e nos dá duas ideias estruturais:

1. **Toda instância tem duas dimensões de veredito:** `FAIL_TO_PASS` (testes que falhavam e devem passar com o fix) e `PASS_TO_PASS` (testes que já passavam e não podem quebrar). Tradução Radiant: cada `eval_case` carrega `expected_before: fail` e `expected_after: pass`, e o runner sempre verifica também uma amostra de casos vizinhos para garantir que o fix não quebrou nada.
2. **Ambiente reproduzível:** o mesmo input + mesma versão de regras + mesma versão de engine deve produzir sempre o mesmo veredito (determinismo). Instâncias mal-postas foram expurgadas no SWE-bench Verified por revisão humana — mesmo critério nossa triagem deve aplicar.

Aviso relevante: um score SWE-bench conflate modelo + harness + leakage; times sérios fazem "harness fingerprinting" ([Harness Engineering — How to Read a SWE-bench Score](https://medium.com/@level-up/harness-engineering)). O Honesty Engine (Seção 7) existe por causa disso.

### 1.3 LLM-as-judge: vieses e calibração

Vieses documentados e mitigações ([Openlayer](https://www.openlayer.com/blog/llm-as-judge-evaluation-guide), [Deepchecks](https://deepchecks.com/llm-judge-calibration-automated-issues), [FutureAGI](https://futureagi.com/blog/llm-as-judge-best-practices-2026), [arXiv 2609.12439 — Calibrated Ties](https://arxiv.org/html/2609.12439v1)):

| Viés | Efeito | Mitigação no Radiant |
|---|---|---|
| Position bias | Juiz favorece A ou B fixo em comparação par-a-par | Comparar sempre em ambas ordens; empate real só se concordar nas duas |
| Verbosity bias | Saída mais longua recebe nota maior | Rubricas por item atômico (1 veredito por regra × campo), não nota holística |
| Self-preference / family bias | Juiz prefere saída do seu próprio modelo/família | Judge de família diferente do solver; ensemble de 2 judges baratos em vez de 1 caro |
| Miscalibration | Juiz só funciona em distribuição parecida com treino | Calibrar contra amostra humana auditada; medir TPR/TNR a cada versão do judge |

Regra de ouro do OpenAI Cookbook: **trate debiasing como intervenção de medição, não como ajuste de prompt** — e rode o judge só onde o determinismo (Rust) não basta.

### 1.4 Property-based, mutation testing, golden files

- **proptest** ([book](https://proptest-rs.github.io/proptest), [repo](https://github.com/proptest-rs/proptest)): propriedades que devem valer para **qualquer** input, com shrinking automático do contraexemplo mínimo. Ideal para leiautes de posição fixa: "qualquer arquivo gerado satisfaz checksum, tamanhos de campo e ordenação". Casos que falham viram casos golden congelados.
- **cargo-mutants** ([manual](https://mutants.rs/), [ThoughtWorks Radar](https://www.thoughtworks.com/en-us/radar/tools/cargo-mutants), [integração nextest](https://nexte.st/docs/integrations/cargo-mutants)): injeta mutações no motor e mede quantas os testes **matam**. É o único teste honesto do próprio harness — "Mutation Detection Rate" é métrica de primeira classe na v4. Rodar incremental em PR, completo na main.
- **Golden files** ([goldie](https://lib.rs/crates/goldie), [Introduction to golden testing](https://ro-che.info/articles/2017-12-04-golden-tests)): output integral do CADOC congelado; atualização de golden só via diff revisado em PR (`GOLDIE_UPDATE` + `git diff` obrigatório). Nenhum golden é atualizado pelo mesmo PR que muda o motor.

### 1.5 Como o RegTech captura feedback de produção

| Empresa | O que faz | Insight para o Radiant |
|---|---|---|
| **Norm AI** ([site](https://www.norm.ai), [What is a Regulatory AI Agent](https://www.norm.ai/resources/what-is-a-regulatory-ai-agent), [US$48M raise](https://www.norm.ai/resources/norm-ai-secures-48-million-to-transform-regulations-into-compliance-ai-agents)) | Converte regulação em "Regulatory AI Agents" — programas executáveis e auditáveis que fazem determinações de compliance | Regra como **artefato executável versionado** (não documento), cada uma com seu próprio ciclo de avaliação |
| **Regnology** ([Reporting Hub](https://www.regnology.net/en/solutions/for-the-regulated/regnology-reporting-hub), [RGI](https://www.regnology.net/en/solutions/regnology-platform/rgi)) | Push de atualizações de regras para clientes; automação ingestion→submission; "governed automation" | A entrega de regra nova é um **release com suite de regressão embutida** (nosso M3) |
| **Duco** ([MAS solution](https://du.co/solutions/mas), [CIOs](https://du.co/cios)) | Reconciliação com *self-learning matching*: correções do operador ensinam o match e reduzem "false breaks" ao longo do tempo | Correção humana é o **sinal de treino mais barato e confiável** que existe (nosso M2) |
| **Compliance.ai** ([site](https://www.compliance.ai)) | Regulatory change management | O diff regulatório é um evento de primeira classe, não um email |

---

## 2. Data flywheel: padrões comprovados

### 2.1 Stripe Radar — o flywheel canônico

[The ML flywheel (Stripe blog)](https://stripe.com/blog/the-ml-flywheel-how-we-continually-improve-our-models-to-reduce-card-testing): detecção de ataques gera **novos rótulos** (pagamentos bloqueados/permitidos) que viram features e treino do modelo seguinte; loops subsequentes são avaliados contra labels que só existem graças ao loop anterior. Resultado público: **redução de 80%** em ataques de card testing em dois anos. O segredo não é o modelo — é que **todo evento ambíguo de produção produz um rótulo**.

### 2.2 GitHub Copilot — telemetria accept/reject

Aceitar ou rejeitar uma sugestão é um clique que vira sinal de treino ([Telemetry Loops for In-Product AI](https://resources.remap.com/libraries/ai-transformation-saas/telemetry-loops-for-in-product-ai), [Fiddler — Copilot observability](https://www.fiddler.ai/blog/github-copilot-otel)). Lição: **o rótulo tem que custar 1 clique**. No Radiant, "aceitar a correção sugerida" é o clique equivalente.

### 2.3 OpenAI flywheel — Analyze/Measure/Improve com quality gates

[OpenAI Cookbook](https://developers.openai.com/cookbook/examples/evaluation/building_resilient_prompts_using_an_evaluation_flywheel) formaliza o loop e o [Databricks/Zepto](https://www.databricks.com/blog/evaluation-first-ai-agents-how-zepto-scales-customer-support-databricks-and-mlflow) mostra o padrão "evaluation-first" em produção real: **quality gates entre cada volta do loop** — nada melhora sem passar no gate da volta anterior.

### 2.4 Discovery por clusterização

[Monitoramento de falhas via BERTopic+UMAP (IEEE)](https://www.computer.org/publications/tech-news/community-voices/llm-safety): embeddings de eventos de produção clusterizados revelam **modos de falha que ninguém conhecia** — exatamente o mecanismo M7 (descobrir regras que o catálogo não tem).

### 2.5 O padrão unificador

```
evento ambíguo de produção → rótulo barato (1 clique / 1 diff)
    → ativo de avaliação (eval case) → gate de regressão
    → melhoria do motor → mais eventos → mais rótulos → ...
```

Todas as fontes convergem: **o flywheel não é um modelo, é um pipeline de conversão evento→ativo com provenance e gate.**

---

## 3. Princípios de design da v4

- **P1 — Toda discrepância é um presente.** O BCValidador oficial do BACEN é o oráculo de verdade disponível em cada submissão; qualquer divergência entre ele e o motor Radiant é, por definição, um caso de teste que não tínhamos.
- **P2 — Zero esforço marginal.** Capturar tem que custar 0 cliques (automático) ou 1 clique (approve/correct). Curar custa minutos por semana (triagem agendada). Avaliar custa zero (cron + CI).
- **P3 — O harness mede a si mesmo.** Mutation Detection Rate, judge TPR/TNR, held-out leak rate e cobertura de regras são scorecards como qualquer outro. Um harness sem métricas sobre si mesmo vira green-wash.
- **P4 — Provenance obrigatória.** Nenhum caso entra no corpus sem origem registrada (`production_discrepancy | user_correction | radar_diff | synthetic | audit | canary`). Casos sem provenance são lixo com aparência de dado.
- **P5 — Gates, não dashboards.** Dashboards informam; gates bloqueiam. Regressão de precisão/recall por regra impede merge e impede release.
- **P6 — O corpus cresce por cobertura, não por volume.** A unidade de progresso é a matriz regra×edge (Seção 6.6), não o número de casos.
- **P7 — Separação treino/teste sagrada.** Casos sintéticos/candidatos alimentam o desenvolvimento; held-out selado responde só em eval-nightly e pré-release. O que vazou para o motor deixa de medir.

---

## 4. Os 10 mecanismos do flywheel

Visão geral:

```
                        ┌──────────────────────────────────────────┐
                        │            RADIANT HARNESS v4            │
                        │                                          │
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

---

### M1 — Discrepância → TestCase

**Gatilho:** submissão validada pelo BCValidador oficial diverge do motor Radiant.

**Fluxo:**

```
submissão tenant ──► motor Radiant (erros OURS)
       │
       └──► BCValidador BACEN (erros BC)
                    │
             diff(OURS, BC) ──► discrepancy
                    │
      ┌─────────────┼──────────────────┐
      ▼             ▼                  ▼
false_positive  false_negative   message_diff
(nós apontamos,  (BC aponta,      (mesma regra,
 BC não)         nós não)         texto diferente)
      │             │                  │
      └── worker propõe eval_case ─────┘
             (input + expected = veredito do BC, provenance=M1)
                    │
             triagem semanal (agente classifica + humano aprova)
```

**Regra de ouro:** o veredito do BC é `expected` **mesmo quando o BC parece errado** — nesses casos abre-se `rule_candidate` (disputa de regra), mas o caso entra no corpus como "comportamento do oráculo", porque é ele que reprova o arquivo do cliente no fim do dia.

**Entidade:** `discrepancy` → `eval_case(provenance='production_discrepancy')`.

**Métrica:** discrepâncias/semana; % virando eval_case em <7 dias; **Time-to-Green** (dias até o caso passar).

---

### M2 — Correção de usuário → par input/expected

**Gatilho:** cliente corrige um valor no UX approve/correct antes de reenviar.

**Fluxo:**

```
validação aponta erro ──► cliente edita campo ──► correction
        (0 cliques: snapshot automático de antes/depois + diff)
                              │
              ┌───────────────┴─────────────────┐
              ▼                                 ▼
   correção já coberta por regra      correção sem regra no catálogo
              │                                 │
   eval_case(input, expected)          rule_candidate(origin='correction',
   com rule_ids da regra + tenant      cluster_signature=field_path+pattern)
              │                                 │
        entra no eval set                worker de mineração agrupa
        do tenant + candidato global     (M7) → proposta de regra nova
```

**Por que funciona:** é o modelo Duco/Grammarly — o rótulo custa o clique que o cliente já daria de qualquer jeito. O snapshot é automático: o UX approve/correct registra `before/after/reason opcional` sem nenhum passo extra.

**Entidade:** `correction` → `eval_case(provenance='user_correction')`.

**Métrica:** taxa de correção→eval_case (alvo: 100% automático); % de correções que revelam rule_candidate.

---

### M3 — Radar detecta mudança → suite de regressão automática da nova versão

**Gatilho:** Radar (monitor normativo) detecta nova Circular/comunicado ou diff de leiaute/Regras de Validação publicadas pelo BCB ([página oficial de leiautes SCR](https://www.bcb.gov.br/estabilidadefinanceira/leiautedocumentoscrd), [doc 3040 + XLS de regras](https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040)).

**Fluxo:**

```
Radar ──► diff(novo_leiaute, leiaute_atual)
              │
    ┌─────────┼──────────────┬───────────────────┐
    ▼         ▼              ▼                   ▼
campo novo/  campo removido  domínio/código    regra de validação
tamanho/mudança              alterado          nova/alterada (XLS→CSV)
    │         │              │                   │
    └─────────┴──────┬───────┴───────────────────┘
                     ▼
   agente gera, para cada diff:
     1. propriedades proptest atualizadas (layout invariants)
     2. casos golden velhos marcados vigencia_fim=hoje
     3. N casos sintéticos cobrindo o diff (M4)
     4. suite "regressão vYYYYMM" versionada por vigência
                     │
                     ▼
   implementação da mudança SÓ está "done" quando
   suite da nova vigência = 100% green + goldens antigos
   congelados provam que versões anteriores continuam reproduzíveis
```

**Chave de design:** suites são **versionadas por vigência** (como o BACEN faz com os leiautes). Um caso pertence a uma vigência; o runner executa sempre a suite da vigência corrente + suite "eterna" (propriedades invariantes).

**Entidade:** `radar_event` → `layout_diff` → `eval_case`/`property_test` com `vigencia`.

**Métrica:** dias entre publicação do BCB e suite green; nº de propriedades geradas por diff.

---

### M4 — Corpus sintético challenger/solver/judge cresce por cobertura

**Modelo:** em vez de "gerar dados sintéticos" genérico, o corpus cresce guiado pelos **buracos da matriz regra×edge**. O agente *challenger* recebe o buraco e tem a missão adversarial de gerar um caso que quebre o motor naquela célula; o motor (*solver*) roda; o *judge* valida se o caso é legal, minimalista e cobre a célula pretendida.

**Fluxo:**

```
matriz regra×edge ──► célula vazia (regra R × edge "boundary+2")
        │
        ▼
   CHALLENGER (LLM): gera payload mínimo que ativa R naquele edge
        │             (usa gerador determinístico + perturbações
        │              estilo OpenAI Cookbook: ruído, fora-de-faixa,
        ▼              codificação estranha, zeros, NULL, Unicode)
   SOLVER (motor Rust): executa
        │
        ▼
   JUDGE (LLM + rubrica): caso é válido? minimal? cobre a célula?
        │                  (rubrica atômica por item — combate verbosity bias;
        ▼                   ordem embaralhada em comparações)
   PASS ──► eval_case(provenance='synthetic', cell=R×edge)
   FAIL ──► challenger re-tenta com feedback do judge (máx 3x)
```

**Matriz de cobertura (regra×edge):**

```
              │ boundary │ negative │ mutation │ real_world │ combo
  V2030-0001  │    3     │    2     │    1     │     5      │  1
  V2030-0002  │    0  ◄── buraco: challenger ataca aqui
  V3040-0017  │    4     │    3     │    2     │     8      │  2
  ...
```

Classes de edge: `boundary` (limites exatos ±1), `negative` (deve rejeitar), `mutation` (caso golden com 1 campo mutado), `real_world` (padrões estatísticos de produção anonimizados), `combo` (interação entre 2+ regras).

**Métrica:** Rule Coverage Index = % de células da matriz com ≥1 caso; challenger acceptance rate; novos bugs/caixa gerada (célula vazia que virou falha real).

---

### M5 — Scorecards por CADOC / tenant / rule version

**Design:** scorecard é **append-only** (nunca sobrescrito), chaveado por `(doc_cod, tenant_id, ruleset_version, engine_version, run_id)`.

Conteúdo por scorecard:
- `precision / recall` por regra (contra BCValidador + correções confirmadas)
- `f2p_pass_rate`, `p2p_pass_rate` (linguagem SWE-bench)
- `false_negative_rate` (o pior erro em regulatório: passar arquivo que o BACEN reprova)
- `latency p50/p95`, `cost_tokens`
- `held_out_score` (suite selada, só em nightly/release)

**Uso:** o scorecard é o deliverable mensal do produto ("Relatório de Confiabilidade" por tenant — ver Seção 8) e a base do gate M6.

---

### M6 — Gates de CI que bloqueiam regressão

**Implementação:** `harness-gate` (CLI Rust) roda no PR e no pré-release.

```
PR aberto ──► harness-gate run --tier pr
                │
   ┌────────────┼──────────────────┐
   ▼            ▼                  ▼
 fast suite   mutation incremental  property suite
 (~500 cases, (cargo-mutants só nos   (proptest, 30s)
  <90s)        arquivos diffados)
                │
        compara contra baseline = último release green
                │
   ┌────────────┼──────────────────────┐
   ▼            ▼                      ▼
 F2P regress   P2P break            precision/recall
 (caso que     (caso green          por regra caiu >1pp
 passava,      quebrou)             em regra tocada pelo diff
 parou de      (BLOQUEIA)           (BLOQUEIA)
 passar,
 BLOQUEIA)
                │
nightly ──► tier nightly: full corpus + held-out selado + canários
release ──► tier release: held-out + canários + scorecard assinado
```

**Regra:** thresholds são por regra e **só podem apertar**; afrouxar exige PR explícito com justificativa (o gate nunca é "ajustado" no calor do momento).

**Fontes:** [Openlayer CI/CD gates](https://www.openlayer.com/blog/cicd-eval-gates-block-merges-model-failure), [Langfuse experiments CI/CD](https://langfuse.com/docs/evaluation/experiments/experiments-ci-cd), [promptfoo CI/CD](https://www.promptfoo.dev/docs/integrations/ci-cd), [cargo-mutants CI](https://mutants.rs/ci.html).

---

### M7 — Discovery de regras desconhecidas por clusterização

**Gatilho:** volume crescente de `correction` / `discrepancy` sem `rule_id` mapeado.

**Fluxo:**

```
telemetria (corrections + discrepancies + validações)
        │
        ▼
embeddings de (doc_cod, field_path, before→after, mensagem)
        │
        ▼
clusterização (HDBSCAN/UMAP + BERTopic sobre mensagens)
   [padrão IEEE/BERTopic para descoberta de modos de falha]
        │
        ▼
cluster com ≥8 observações e ≥2 tenants, sem regra mapeada
        │
        ▼
rule_candidate(origin='error_cluster', hypothesis=<gerada por LLM
  a partir de 10 exemplos do cluster>, n_observations, tenants[])
        │
        ▼
triagem semanal ──► vira draft de regra no catálogo
                    └─► e N casos sintéticos (M4) para a regra nova
```

**Por que é o mecanismo mais valioso:** é o único que gera **receita de catálogo** (regra que você nem sabia que existia) e não só proteção de regressão. É o "cliente corrigiu X três vezes sem regra associada → existe uma regra implícita no mercado que o catálogo não tem".

**Métrica:** regras descobertas/mês; % da produção coberta por regra explícita ("regra sombra" = cluster sem regra).

---

### M8 — Distillation / fine-tune quando o dataset amadurece

**Gatilhos por regra ou por solver inteiro** (heurísticas de mercado: poucas centenas de exemplos bastam para tarefas estreitas; ~500–1.000+ exemplos de qualidade é o ponto onde fine-tune/destilação passa a valer; ver [Distill — quantos exemplos](https://distill.pub) e [Google "Distilling Step-by-Step", arXiv 2305.02301](https://arxiv.org/abs/2305.02301)):

| Condição | Ação |
|---|---|
| Regra com ≥200 casos golden e texto de mensagem ainda gerado por LLM | Destilar gerador de mensagem da regra (prompt → modelo pequeno ou template) |
| Solver LLM-auxiliado com ≥1.000 pares (input→veredito do oráculo) | Fine-tune do assistente de classificação de erros ambíguos |
| Judge com TPR/TNR estável ≥95% por 8 semanas e ≥500 decisões auditadas | Congelar judge como "silver standard" para gerar dados de destilação |
| Custo mensal de LLM em validações > US$200 com padrão repetitivo | Destilar componente quente |

**Guardrail:** todo modelo destilado entra como `engine_version` candidata e só sobe via scorecard A/B no tier release (held-out decide), nunca direto.

---

### M9 — Red-team contínuo do próprio motor (adversarial + canary)

- **Canary issues:** N casos com veredito conhecido injetados **aleatoriamente** em cada run (sela contra vazamento e detecta runner quebrado — se um canário passa/fail errado, o run é inválido).Inspiração: [canary strings em benchmarks](https://www.wired.com/story/ai-benchmarks-contamination/) e [contaminação/Sealed Suite](https://www.testmuai.com/blog/benchmark-contamination).
- **Adversarial evals agendados:** semanalmente o challenger (M4) ataca as 10 regras com pior recall histórico + gerador de payloads malignos (campos deslocados, encoding traiçoeiro, valores válidos-que-parecem-inválidos) — red team via [promptfoo red team](https://www.promptfoo.dev/docs/red-team) como referência de taxonomia.
- **Mutation testing periódico:** cargo-mutants completo na main semanalmente; Mutation Detection Rate < 70% em qualquer crate do motor = issue automática.
- **Eval-awareness:** casos canário têm formato distinto do corpus (para detectar se algum componente está "reconhecendo o eval" — ver [Eval Awareness](https://harness-guide.com)).

---

### M10 — Auditoria humana amostral + calibração do judge

- Semanalmente, 30 casos sorteados estratificados (por doc_cod × veredito × provenance) vão para revisão humana (o dono, 30–45 min com UI de 1 clique).
- Os rótulos humanos alimentam: (a) calibração do judge → **judge agreement TPR/TNR** publicado no scorecard do próprio judge; (b) disputes → triagem de regra; (c) o split 20/40/40 do gold standard ([OpenAI Cookbook](https://developers.openai.com/cookbook/examples/evaluation/building_resilient_prompts_using_an_evaluation_flywheel)).
- Judge é **versionado como código** (prompt + modelo + rubric no git); trocar judge = PR com comparação de TPR/TNR contra a amostra auditada acumulada.

---

## 5. Arquitetura — Rust + Postgres + agentes

### 5.1 Componentes

```
┌─────────────────────────────────────────────────────────────────┐
│                        CRATES RUST                              │
│                                                                 │
│  radiant-engine      motor CADOC (gera/valida)                  │
│  radiant-rules       catálogo de regras compilado (versionado)  │
│                                                                 │
│  harness-core        runner determinístico de evals             │
│   ├─ executa case → verdict, sem I/O de rede                    │
│   ├─ FAIL_TO_PASS / PASS_TO_PASS / canário detection            │
│   └─ sai com scorecard JSON assinado                            │
│                                                                 │
│  harness-gate        CLI de CI (tier pr|nightly|release)        │
│  harness-miner       worker: discrepancies/corrections → cases  │
│  harness-synth       geração challenger/solver/judge            │
│  harness-cluster     embeddings + HDBSCAN/BERTopic (M7)         │
└─────────────────────────────────────────────────────────────────┘
        │                    │                      │
   Postgres            cron eval-nightly        agentes ZCode
   (corpus+runs)       (worker async Rust)      (triagem semanal,
                                                proposta de regras,
                                                red-team criativo)
```

### 5.2 Schema Postgres (núcleo)

```sql
-- Catálogo de regras (regra como artefato executável versionado — padrão Norm AI)
CREATE TABLE rule (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  cod             TEXT UNIQUE NOT NULL,          -- 'V2030-0012'
  doc_cod         TEXT NOT NULL,                 -- '2030','3040','4010',...
  descricao       TEXT NOT NULL,
  fonte           TEXT,                          -- 'Circular 3.995, art. 12, II'
  categoria       TEXT NOT NULL CHECK (categoria IN
                    ('leiaute','negocio','consistencia','mensagem')),
  vigencia_inicio DATE NOT NULL,
  vigencia_fim    DATE,
  status          TEXT NOT NULL DEFAULT 'draft'
                    CHECK (status IN ('draft','active','deprecated','disputed')),
  created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE rule_version (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  rule_id     UUID REFERENCES rule(id),
  semver      TEXT NOT NULL,
  spec        JSONB NOT NULL,        -- AST/predicate executável da regra
  changed_by  TEXT,                  -- 'radar_event:812' | 'rule_candidate:34'
  created_at  TIMESTAMPTZ DEFAULT now(),
  UNIQUE (rule_id, semver)
);

-- A unidade atômica do harness
CREATE TABLE eval_case (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  doc_cod       TEXT NOT NULL,
  vigencia      DATE NOT NULL,
  input         JSONB NOT NULL,      -- payload (sintético ou anonimizado)
  expected      JSONB NOT NULL,      -- {should_pass, expected_errors:[
                                     --  {rule_cod, field, message_regex}]}
  provenance    TEXT NOT NULL CHECK (provenance IN
                  ('production_discrepancy','user_correction','radar_diff',
                   'synthetic','audit','canary','golden')),
  source_ref    UUID,                -- discrepancy/correction/radar_event origem
  rule_cods     TEXT[] DEFAULT '{}',
  edge_class    TEXT CHECK (edge_class IN
                  ('boundary','negative','mutation','real_world','combo')),
  weight        NUMERIC DEFAULT 1.0,
  is_held_out   BOOLEAN DEFAULT false,
  expected_before_fail BOOLEAN NOT NULL DEFAULT true,  -- F2P semântica
  status        TEXT NOT NULL DEFAULT 'candidate'
                  CHECK (status IN ('candidate','triaged','active','retired')),
  created_by    TEXT, created_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX ON eval_case (doc_cod, vigencia, status);
CREATE INDEX ON eval_case USING gin (rule_cods);

CREATE TABLE eval_run (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  trigger         TEXT NOT NULL CHECK (trigger IN
                    ('ci','nightly','pre_release','manual')),
  engine_version  TEXT NOT NULL,
  ruleset_version TEXT NOT NULL,
  judge_version   TEXT,
  commit_sha      TEXT,
  tier            TEXT NOT NULL,     -- pr|nightly|release
  started_at      TIMESTAMPTZ, finished_at TIMESTAMPTZ,
  summary         JSONB              -- f2p,p2p,precision,recall,canary_ok,...
);

CREATE TABLE eval_result (
  run_id     UUID REFERENCES eval_run(id),
  case_id    UUID REFERENCES eval_case(id),
  verdict    TEXT CHECK (verdict IN ('pass','fail','error','canary_fail')),
  actual     JSONB, latency_ms INT, tokens INT,
  PRIMARY KEY (run_id, case_id)
);

CREATE TABLE discrepancy (           -- BCValidador vs motor (M1)
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id     TEXT, submission_id UUID, doc_cod TEXT,
  bc_errors     JSONB, our_errors JSONB,
  delta_class   TEXT CHECK (delta_class IN
                  ('false_positive','false_negative','message_diff')),
  resolved_eval_case_id UUID REFERENCES eval_case(id),
  created_at    TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE correction (            -- correção do usuário (M2)
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id     TEXT, submission_id UUID, doc_cod TEXT,
  field_path    TEXT, before TEXT, after TEXT, reason TEXT,
  auto_eval_case_id UUID REFERENCES eval_case(id),
  created_at    TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE rule_candidate (        -- discovery (M7) e disputas (M1/M2)
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  origin           TEXT CHECK (origin IN
                     ('error_cluster','discrepancy','correction','dispute')),
  cluster_signature TEXT,            -- assinatura p/ dedup
  n_observations   INT, tenants TEXT[],
  hypothesis       TEXT, examples JSONB,
  status           TEXT DEFAULT 'open'
                     CHECK (status IN ('open','triaged','promoted','rejected')),
  promoted_rule_id UUID REFERENCES rule(id),
  created_at       TIMESTAMPTZ DEFAULT now(),
  UNIQUE (cluster_signature)
);

CREATE TABLE scorecard (             -- append-only (M5)
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  run_id          UUID REFERENCES eval_run(id),
  doc_cod TEXT, tenant_id TEXT,
  ruleset_version TEXT, engine_version TEXT,
  metrics         JSONB NOT NULL,   -- precision, recall, fn_rate, p95...
  created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE canary_seed (           -- M9
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  payload JSONB, expected_verdict TEXT,
  active BOOLEAN DEFAULT true, inserted_in_run UUID NULL
);

CREATE TABLE judge_calibration (     -- M10
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  judge_version TEXT, case_id UUID, judge_verdict TEXT,
  human_verdict TEXT, run_batch TEXT, created_at TIMESTAMPTZ DEFAULT now()
);
```

### 5.3 Fluxo eval-nightly

```
03:00 cron ──► harness-gate run --tier nightly
   ├─ full corpus ativo (determinístico, Rust, sem LLM)
   ├─ held-out selado (descriptografado em memória, resultado agregado)
   ├─ canários embaralhados no corpus (validade do run)
   ├─ harness-synth: challenger ataca 20 piores células da matriz
   ├─ judge LLM só nos ~30% de casos não determinísticos
   ├─ scorecards gerados (append-only)
   └─ relatório 1 página no canal do projeto + diffs de métrica vs D-1
```

### 5.4 Pipeline de mineração (worker contínuo)

```
evento (discrepancy|correction|radar_event)
   └─► dedup (hash do payload normalizado)
       └─► propose eval_case (status=candidate)
           └─► fila de triagem semanal (agente ZCode classifica:
               válido? minimal? regra(s) certa(s)? held-out?)
               └─► humano aprova batch em 1 tela (approve/bulk)
                   └─► status=active → próximo run já o executa
```

### 5.5 Dashboards (1 página cada)

1. **Flywheel:** casos novos/semana por provenance; Time-to-Green mediano; células da matriz preenchidas.
2. **Qualidade:** precision/recall por regra (top 10 piores); FN rate por doc_cod; disparidades vs BCValidador abertas.
3. **Honestidade:** mutation detection rate; judge TPR/TNR; canary pass rate; held-out leak rate.
4. **Regras sombra:** clusters abertos com n_observations ≥8 (candidatas a regra nova).

### 5.6 Custo computacional estimado (nov/2026, 1 pessoa, ~10 tenants)

| Item | Volume | Custo/mês |
|---|---|---|
| eval-nightly determinístico (Rust, 3–5k cases) | 30 runs | ~0 (VM própria, <10 min/run) |
| judge LLM (~30% dos cases não determinísticos, modelo classe Haiku/GLM ~US$1/M in + US$5/M out, ~2.5k tok/case) | ~1k calls/noite | ~US$75–100 |
| embeddings + clusterização (local, modelo pequeno) | contínuo | ~US$5 |
| challenger sintético (LLM médio, 200 casos novos/dia) | diário | ~US$30–60 |
| cargo-mutants semanal completo | 4x | ~0 (CPU local, horas) |
| destilação/fine-tune eventual (gatilho M8) | 1–2x/ano | ~US$100–300 por evento |
| **Total recorrente** | | **~US$110–170/mês** |

Conclusão de custo: o flywheel é barato porque 70%+ do corpus é determinístico (Rust) e o LLM só aparece onde ambiguidade existe.

---

## 6. Honesty Engine — como o harness não virar green-wash

### 6.1 Os 5 modos de falha do harness

| Modo | Como acontece | Contramedida |
|---|---|---|
| **Overfit ao corpus** | fix passa nos casos que motivaram o fix e quebra vizinhos | PASS_TO_PASS amostral em todo run; corpus cresce via challenger (adversário não conhece o fix) |
| **Vazamento held-out** | casos held-out vazam para desenvolvimento/prompts | held-out criptografado, aberto só no runner; **held-out leak rate** monitorado; casos canário com formato distinto |
| **Judge virou alvo** | sistema otimiza para o judge, não para a verdade | judge de família diferente do solver; judge calibrado vs amostra humana (TPR/TNR); rubricas atômicas; ensemble |
| **Oráculo capturado** | confiança cega no BCValidador mesmo quando ele muda/m erra | disputas registradas (`rule_candidate`); histórico de veredito do oráculo por versão; casos "oráculo disse X" versionados por data |
| **Métrica inflada por seleção** | corpus só com casos fáceis que o motor já passa | matriz regra×edge obriga células negativas/adversárias; auditoria amostral estratificada mede a taxa real em produção |

### 6.2 Harness Health Scorecard (o harness avaliado como produto)

| Métrica | Definição | Alvo | Alarme |
|---|---|---|---|
| Mutation Detection Rate | % de mutantes mortos (cargo-mutants) | ≥75% | <70% |
| Judge TPR / TNR | vs amostra humana auditada | ≥95%/95% | <90% |
| Canary pass rate | canários com veredito correto no run | 100% | <100% (run inválido) |
| Held-out leak rate | casos held-out citados em prompts/repo | 0 | >0 |
| Rule Coverage Index | % células regra×edge com ≥1 caso | ≥80% ativas | <60% |
| Shadow-rule rate | % produção com erro sem regra mapeada | <5% | >10% |
| Audit agreement | humano concorda com veredito do motor na amostra | ≥97% | <95% |
| Time-to-Green p50 | discrepância→caso green | ≤7 dias | >14 dias |

**Auditoria humana amostral** (M10) fecha o ciclo: é a única fonte não sintética de verdade sobre o próprio harness. Sem ela, todas as métricas acima são autorreferentes.

**Fingerprint do harness:** cada `eval_run` registra `engine_version + ruleset_version + judge_version + corpus_hash`; score comparado só entre fingerprints compatíveis — leitura direta da lição de "harness fingerprinting" do SWE-bench.

---

## 7. Mercado — "melhora com o uso" como feature de pricing

| Produto | Como comunica self-improvement | O que a Radiant replica |
|---|---|---|
| **Stripe Radar** ([flywheel post](https://stripe.com/blog/the-ml-flywheel-how-we-continually-improve-our-models-to-reduce-card-testing)) | Métrica pública do flywheel: "80% de redução em card testing" — vende o **resultado composto**, não a tecnologia | Publicar (para tenants) a curva de FN rate caindo mês a mês com uso |
| **Datadog Watchdog** ([2018](https://www.datadoghq.com/blog/watchdog), [product page](https://www.datadoghq.com/product/platform/watchdog)) | "Zero configuration" — valor desde o primeiro dia, melhoria invisível e contínua | Onboarding sem configuração de regras: o tenant começa no corpus global e ganha scorecard próprio |
| **Duco** ([CIOs](https://du.co/cios)) | "Eliminates noise and false breaks" — melhora porque o operador corrige; self-learning matching | Correção do operador → menos falso-positivo para o mesmo tenant (personalização por tenant é feature vendável) |
| **Grammarly** | Sugestões que se personalizam com o uso contínuo | Scorecard mensal "Radiant aprendeu N regras novas com o mercado este mês; sua taxa de reprovação caiu X%" |
| **Regnology RGI** ([platform](https://www.regnology.net/en/solutions/regnology-platform/rgi)) | "Governed automation" — melhora regulada e auditável | Scorecard assinado + changelog de regras = compliance story, não só qualidade |

**Mecânica de pricing sugerida:**

```
Base (por tenant): mensageria + validação + scorecard mensal
Usage:             por submissão/validação (o volume alimenta o flywheel —
                   quem mais usa, mais o sistema aprende, e paga proporcionalmente)
Premium:           "Accuracy SLA" — FN rate < X% contratual, respaldado
                   pelo held-out + auditoria (só a Radiant com Honesty Engine
                   consegue oferecer isso com honestidade)
Network effect:    regra descoberta em um tenant (M2/M7) protege todos os
                   tenants no próximo release — pitch: "seu problema de hoje
                   não volta nem nos seus concorrentes... nem para você"
```

O pitch central: **"Cada submissão que você faz deixa o Radiant — e todo o mercado que o usa — menos propenso a errar a próxima."** É o mesmo enquadramento do Stripe Radar: a rede inteira aprende com cada ataque.

---

## 8. Roadmap de implementação

| Fase | Entregas | Critério de pronto |
|---|---|---|
| **F0 (sem 1–2)** | schema Postgres; `harness-core` runner + F2P/P2P; goldens iniciais; proptest de leiaute | `harness-gate --tier pr` rodando em PR |
| **F1 (sem 3–4)** | M1 discrepância→case; M2 correção→case (snapshot automático no UX); worker de mineração; triagem semanal com agente | ≥1 case novo/semana com provenance |
| **F2 (sem 5–6)** | M3 Radar→suite versionada por vigência; M6 gates completos (precision/recall por regra); scorecards | Radar detecta diff → suite gerada sem código manual |
| **F3 (sem 7–9)** | M4 challenger/solver/judge + matriz regra×edge; canários; held-out selado | RCI ≥60%; canários 100% |
| **F4 (sem 10–12)** | M7 clusterização; M10 auditoria amostral + judge calibration; dashboards; cargo-mutants em CI | Harness Health Scorecard no ar; RCI ≥80% |
| **F5 (quando gatilho M8)** | destilação de componentes quentes; Accuracy SLA como pricing | Primeiro A/B de engine_version destilada aprovado em held-out |

---

## 9. Fontes

**Eval-driven development e plataformas**
- https://www.langchain.com/blog/what-is-eval-driven-development
- https://www.braintrust.dev/blog/eval-feedback-loops
- https://www.langchain.com/blog/regression-testing
- https://www.braintrust.dev/articles/langsmith-vs-braintrust
- https://developers.openai.com/cookbook/examples/evaluation/building_resilient_prompts_using_an_evaluation_flywheel
- https://developers.openai.com/api/docs/guides/evaluation-best-practices
- https://www.anthropic.com/research/demystifying-evals-for-ai-agents
- https://www.anthropic.com/research/building-strong-safety-evaluations
- https://langfuse.com/docs/evaluation/experiments/experiments-ci-cd
- https://www.openlayer.com/blog/cicd-eval-gates-block-merges-model-failure
- https://www.promptfoo.dev/docs/integrations/ci-cd
- https://www.promptfoo.dev/docs/red-team

**Benchmarks e honestidade de harness**
- https://github.com/swe-bench/SWE-bench
- https://arxiv.org/abs/2310.06770
- https://www.thoughtworks.com/en-us/radar/tools/cargo-mutants
- https://mutants.rs/ci.html
- https://arxiv.org/html/2502.17521 (survey static→dynamic evaluation)
- https://arxiv.org/html/2609.12439v1 (calibração de LLM-as-judge)

**LLM-as-judge**
- https://www.openlayer.com/blog/llm-as-judge-evaluation-guide
- https://deepchecks.com/llm-judge-calibration-automated-issues
- https://futureagi.com/blog/llm-as-judge-best-practices-2026

**Testing em Rust**
- https://proptest-rs.github.io/proptest
- https://github.com/proptest-rs/proptest
- https://nexte.st/docs/integrations/cargo-mutants
- https://lib.rs/crates/goldie
- https://ro-che.info/articles/2017-12-04-golden-tests

**Data flywheel**
- https://stripe.com/blog/the-ml-flywheel-how-we-continually-improve-our-models-to-reduce-card-testing
- https://stripe.dev/blog/scaling-radar-fighting-fraud-with-agent-assisted-machine-learning
- https://www.fiddler.ai/blog/github-copilot-otel
- https://www.databricks.com/blog/evaluation-first-ai-agents-how-zepto-scales-customer-support-databricks-and-mlflow
- https://www.nvidia.com/en-us/glossary/data-flywheel

**Discovery por clusterização**
- https://www.computer.org/publications/tech-news/community-voices/llm-safety (BERTopic/UMAP)
- https://www.confident-ai.com/knowledge-base/guides/ai-production-issue-detection-framework

**RegTech**
- https://www.norm.ai / https://www.norm.ai/resources/what-is-a-regulatory-ai-agent
- https://www.regnology.net/en/solutions/for-the-regulated/regnology-reporting-hub
- https://www.regnology.net/en/solutions/regnology-platform/rgi
- https://du.co/solutions/mas / https://du.co/cios
- https://www.compliance.ai

**Domínio BACEN**
- https://www.bcb.gov.br/estabilidadefinanceira/leiautedocumentoscrd (leiautes CADOC/SCR)
- https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040 (doc 3040 + Regras de Validação XLS)

**Destilação**
- https://arxiv.org/abs/2305.02301 (Distilling Step-by-Step)
- https://cloud.google.com/discover/human-in-the-loop
