# 21 — ENGINE DE IA

> Seção do blueprint de reconstrução do Radiant Norma em Rust. Define como a IA entra no produto sem contaminar o determinismo que o vende: extração norma→regra, copiloto de validação, RAG normativo, residência de dados/LGPD, MCP, corpus sintético, fine-tuning, TCO e riscos.
> **Data:** 2026-11-16 · **Fontes:** research/07-engine-ia.md (principal), research/04-rust-stack.md (R11 IA/MCP, R12 workspace), research/09-harness.md (challenger/solver/judge) · Preços verificados em fontes públicas em nov/2026.

---

## 0. Veredito em uma página

| Função | Pick primário | Fallback / secundário | Custo |
|---|---|---|---|
| Extração norma→regra (draft) | **Gemini 3 Pro** ($2/$12, ctx 1M, visão nativa) | Claude Opus 4.5 ($5/$25) p/ casos duros | ~$0.30–3/candidato |
| Crítica contra o § fonte | **Claude Sonnet 4.5** ($3/$15) | Opus 4.5 | incluído no candidato |
| Copiloto de validação | **Claude Sonnet 4.5** + prompt caching | **GPT-5.1** ($1.25/$10) contratual | ~$0.03–0.12/validação |
| Insights não-críticos (opt-in) | **Gemini 3 Flash** ($0.50/$3) | GPT-5.1-mini | <$0.02/insight |
| RAG embeddings | **Qwen3-Embedding** on-prem / **Gemini Embedding 2** ($0.20/M) cloud | BGE-M3 | $0–20/mês |
| Reranking | **BGE-Reranker-v2-M3** self-hosted | Cohere Rerank (~$2/1k buscas) | $0 (self) |
| Vetor store + BM25 | **pgvector (HNSW) + Tantivy** (Rust-native, RRF) | Qdrant se filtragem pesada | $0 infra extra |
| Parsing | **calamine** (XLSX, determinístico) + **Docling** (PDF) | Mistral OCR ($2/1k págs) só escaneados | $0–50/mês |
| LLM on-prem (PII/air-gap) | **GPT-OSS-120B** em 1×H100 OCI SP ($10/h) | Qwen3-30B-A3B em 1×L40S | $2.6–7.3k/mês GPU |
| Serving | **vLLM** (default) / **SGLang** (copiloto, RadixAttention) | llama.cpp só dev | — |
| Agentes/conectores | **MCP spec 2026-07-28 + rmcp** (SDK oficial Rust) | — | dev time |
| Fine-tuning | **Não** para conhecimento versionado; LoRA só estilo | Distillation se copiloto for on-prem | $5–500/run |

**Princípio arquitetural em uma frase:** determinismo no engine (Rust compila e executa regras), IA na fronteira (extração, crítica, explicação, geração de teste), e verificação objetiva antes de qualquer output de LLM entrar no produto.

---

## 1. Filosofia: determinismo no engine + IA na fronteira + verificação objetiva

Este é o padrão consolidado do RegTech de 2026 — **Norm AI** converte regulação em "programas executáveis + agentes" (a unidade de entrega é código + suite de testes por regra, auditável — não um resumo do normativo) e a **Regnology** (relatório "The Agentic Gap") mostra que camada agêntica só funciona sobre dados e regras normalizadas. Ambos chegam à mesma conclusão: agência de IA é a borda, o motor determinístico é o núcleo.

**A vantagem estrutural do Radiant já existe:** a literatura de rejection sampling é clara — gerar N candidatos e filtrar funciona bem **quando o verificador é objetivo** (execução de código, validação de schema) e degrada para alucinação quando o único filtro é um LLM-judge. O Radiant já possui o verificador objetivo: **XSD do leiaute BACEN + engine Rust** que compila e executa a regra contra golden fixtures. Isso é o que separa *rejection sampling* de *alucinação* — concorrentes que só têm "LLM lê norma e responde" não têm essa linha de defesa.

Regras do jogo (não negociáveis):

1. **Nenhum output de LLM entra no produto sem verificação objetiva** — `jsonschema` (crate Rust, Draft 2020-12) valida o JSON candidato; o engine compila e roda a regra contra XSD + fixtures; o humano aprova o que é crítico. JSON válido ≠ semântica correta: a validação de valores é sempre necessária.
2. **IA onde o juízo é difuso** (interpretar texto de norma, explicar violação, gerar caso de teste, criticar candidato); **determinismo onde a resposta existe** (executar regra, validar XSD, transmitir ao STA).
3. **Provenance obrigatória** — toda explicação, toda regra e todo insight cita a fonte (regra do catálogo + § do normativo). Sem citação, não sai.
4. **Regras são dados, não pesos** — conhecimento regulatório vive no catálogo versionado (Postgres) e no RAG, nunca em fine-tune (ver §8).
5. **Agente livre em reporting regulatório = risco de auditoria** — workflow graph com nós de verificação, nunca loop aberto (ver §6).

---

## 2. Pipeline EXTRAÇÃO NORMA→REGRA (o gargalo atual é manual)

Hoje o catálogo de **~1.099 regras** foi montado à mão: ~8h de trabalho por resolução grande, lendo norma + planilha de leiaute e transcrevendo para regras. Este pipeline é o maior alavancador de custo do produto: IA pré-extrai ~80% com confiança verificada, humano confirma em ~20min/doc.

```
 Normativo (DOU / Carta Circular, PDF)          Leiaute (XLSX do BCB)
         │                                              │
         ▼                                              ▼
   Docling (layout+tabelas,                     calamine (Rust, célula-a-célula,
   markdown hierárquico com §)                  determinístico — NUNCA OCR)
         │                                              │
         └──────────────────┬───────────────────────────┘
                            ▼
 ┌─ DRAFT ─────────────────────────────────────────────────────────┐
 │ Gemini 3 Pro (visão nativa, ctx 1M, $2/$12) lê resolução +      │
 │ planilha JUNTOS; rejection sampling n=3, temp 0.2; output       │
 │ JSON schema-constrained (id, classe, ator, condição, evento,    │
 │ limite, fonte §, vigência)                                      │
 └───────────────────────────┬─────────────────────────────────────┘
                             ▼
 ┌─ CRITIC ────────────────────────────────────────────────────────┐
 │ Claude Sonnet 4.5 critica candidato vs § fonte: diff §-a-§,     │
 │ aloca/rejeita evidência, score de confiança por campo           │
 └───────────────────────────┬─────────────────────────────────────┘
                             ▼
 ┌─ TEST (verificação objetiva) ───────────────────────────────────┐
 │ Engine Rust compila a regra e roda contra golden fixtures       │
 │ (CADOCs reais + XSD do leiaute) — passa ou reprova, sem juízo   │
 └───────────────────────────┬─────────────────────────────────────┘
                             ▼
 ┌─ HUMAN ─────────────────────────────────────────────────────────┐
 │ Fila de aprovação em UI de diff (§ da norma ↔ JSON da regra);   │
 │ aprova → catálogo versionado (Postgres)                         │
 └───────────────────────────┬─────────────────────────────────────┘
                             ▼
   RADAR: hash por § detecta mudança → re-dispara o pipeline
   só nos § alterados (delta), com regras afetadas sinalizadas
```

**Custo:** ~$0.30–3 por candidato (draft + critic + test) — ordem de grandeza abaixo das ~8h manuais que substitui. A revisão humana deixa de ser transcrição e passa a ser confirmação (~20min/doc).

**Parsing — regra de ouro:**

| Entrada | Ferramenta | Por quê |
|---|---|---|
| XLSX/XLS de leiaute | **calamine 0.38** (Rust) | Determinístico, célula-a-célula, headers mesclados resolvidos por regra. **Nunca OCR** — OCR erra merged cells (o melhor do setor, Reducto, faz 90.2%); o LLM só interpreta o JSON já estruturado |
| PDF nativo (cartas, resoluções) | **Docling** | Melhor recuperação de layout/tabelas gratuita (94–96% estrutural); markdown com § e tabelas preservados |
| PDF escaneado (raro) | **Mistral OCR** ($2/1k págs) | Só quando não há camada de texto |

**Taxonomia de regras em 5 classes** (alinhada a LegalRuleML/deôntica derrotável — a extração aponta a classe, o engine sabe como executar cada uma):

| Classe | Exemplo CADOC | Execução |
|---|---|---|
| 1. Estrutural | campo existe, tamanho, tipo, obrigatoriedade | XSD/leiaute — determinístico |
| 2. Deôntica | "obrigatório se X", limite, proibição | motor de regras Rust compilada |
| 3. Cross-doc | consistência entre CADOCs / quadro↔quadro | motor + join de dados |
| 4. Temporal | vigência, base de cálculo por data, backporting | motor + timeline versionada |
| 5. Interpretativa | "preencher quando aplicável", julgamento | IA sugere, humano decide |

O catálogo manual atual é majoritariamente classes 1–2; **as classes 3–4 são o diferencial competitivo** e o maior ganho da extração automática — é trabalho que não escala à mão.

---

## 3. COPILOTO DE VALIDAÇÃO

Explicação campo-a-campo das violações dentro do fluxo de validação (opt-in por tenant; o veredito continua sendo 100% do engine).

| Papel | Modelo | Nota |
|---|---|---|
| Default de explicação | **Claude Sonnet 4.5** ($3/$15, cache read $0.30) | **Prompt caching ligado por default**: o contexto de regras (~25k tokens) é estável e reusado — cache hit ~80% derruba o custo efetivo ~10× |
| Alternativa contratual | **GPT-5.1** ($1.25/$10, batch 50% off) | Router mantém as duas vivas; troca em 1 dia se preço/qualidade mudarem |
| Insights não-críticos | **Gemini 3 Flash** ($0.50/$3) | Agregações, tendências, resumos — nunca veredito |

**Custo:** ~$0.03–0.12 por validação com copiloto (premissas: LLM em ~10% dos eventos, ~1.500 tokens de saída, cache hit 80%).

**Sanitização PII obrigatória quando há dado de cliente:** pipeline de sanitização on-prem (NER por regex + modelo leve: CPF/CNPJ/nome/agência/conta) roda **antes** de qualquer chamada frontier. A IA frontier vê regra + estrutura + valores mascarados — não precisa de PII para explicar uma violação. Dado público (normativo BACEN) segue direto; dado de cliente sanitizado; dado que não pode sair → rota on-prem (ver §5).

**Explainability campo-a-campo com provenance obrigatória:** cada explicação cita (a) a regra do catálogo com versão, (b) o § do normativo que a originou, (c) o valor do campo e a condição violada. Sem os três, a explicação não é renderizada. Isso transforma o copiloto de "chat sobre erro" em **evidência auditável** — o deliverable que compliance de banco aceita.

---

## 4. RAG NORMATIVO

Consulta recorrente de norma/regra durante validação (3 consultas por validação em média). Long-context como banco de dados está descartado: custo de contexto armazenado ($0.50–4.50/Mtok/h no Gemini) e degradação em multi-shot needle tornam RAG mais barato e mais preciso.

```
 Query ("regra de preenchimento do campo 109 do CADOC 4010")
   ├── BM25 — Tantivy (Rust-native): ids exatos, números de campo, siglas
   ├── Dense — Qwen3-Embedding (on-prem) ou Gemini Embedding 2 (cloud)
   ├── Fusão RRF (reciprocal rank fusion) → top 50
   └── Rerank BGE-Reranker-v2-M3 (self-hosted) → top 8 → LLM/engine
```

**Stack:** **pgvector com índice HNSW** (zero infra nova — já teremos Postgres; crate `pgvector` com feature `sqlx`) + **Tantivy para BM25** — é Rust, casa com o motor e evita um serviço extra tipo Qdrant. LanceDB é plano B se um dia quisermos tudo embedded; Qdrant só se filtragem por payload virar complexa.

**Por que dense-only falha aqui:** "campo 109", "CADOC 3050", "doc 3044" são identificadores exatos — tokens raros que embeddings não ranqueiam bem (o dense acha textos *parecidos*, não o documento *citado*). BM25 resolve exatamente isso; o consenso de 2026 é que híbrido é obrigatório em domínio com siglas e numeradores. A fusão RRF combina os dois rankings sem tuning de pesos.

**Ingestão e chunking:** por § com overlap semântico; tabelas de leiaute viram **"cartões de campo"** (1 doc estruturado por campo CADOC: nome, tipo, tamanho, domínio, regras associadas), embedados individualmente; fórmulas viram texto canônico + fonte.

**Embeddings:**

| Opção | Custo | Quando |
|---|---|---|
| **Qwen3-Embedding (0.6B/4B)** self-host | $0 (GPU pequena/CPU) | on-prem default; top open multilingue |
| **Gemini Embedding 2** | $0.20/M ($0.10 batch) | se o RAG ficar em cloud |

**Eval:** RAGAS (faithfulness, context precision/recall) rodando em CI (DeepEval/pytest + promptfoo para regressão de prompt) contra **golden set de 200 perguntas** curtadas por compliance no bootstrap do catálogo. Retrieval sem eval é fé.

**Atualização incremental via Radar:** ingestão grava **hash por §**; BACEN publica → diff de § → re-embed só do delta (segundos, **<$1 por documento alterado**) → regras afetadas recebem flag "normativo mudou — regra desatualizada?" → dispara o pipeline da §2 sobre o delta. O RAG envelhece em dias, não em trimestres.

---

## 5. RESIDÊNCIA DE DADOS / LGPD

**Fato de mercado (nov/2026): nenhum provedor frontier tem residência de dados no Brasil.** OpenAI: EU/US/CA/JP/KR/SG/IN/AU/UAE. Anthropic e Vertex: US/EU. O Google Cloud anunciou servidores Blackwell em São Paulo (out/2026), mas isso é residência física de infra — não residência contratual de dados de API de IA. Base legal: ANPD Resolução CD/ANPD nº 29/2024 (transferência internacional, em vigor desde ago/2025; multas até R$50M).

**Padrão: hybrid cloud**, com três condições (as três, sempre):

1. **Sanitizador PII antes do envio** — NER on-prem (CPF/CNPJ/nome/conta) no caminho de toda chamada frontier com dado de cliente (§3).
2. **Contrato + ZDR + DPA** — tiers de API com zero-data-retention (OpenAI/Anthropic/Gemini oferecem), DPA com cláusula padrão ANPD, opt-out de treino (default nas APIs).
3. **Router por classificação de dados** — normativo BACEN (público) → frontier sem restrição; dado de cliente sanitizado → frontier com ZDR; dado que não pode sair → rota on-prem. Feature flag por cliente (alguns IFs exigirão 100% on-prem).

**Opção air-gapped (para o cliente que exige never-leave):**

| Componente | Escolha | Por quê |
|---|---|---|
| Modelo | **GPT-OSS-120B** (MoE 117B, 5.1B ativos, MXFP4 ~61GB, Apache-2.0) | Melhor open-weight para PT-BR + structured outputs; cabe em 1×H100 80GB |
| Hardware | **1×H100 OCI São Paulo — $10/h** (preço uniforme global, preemptible −50/70%) | Melhor GPU LatAm por preço; AWS sa-east-1 cobra prêmio ~3× vs região US |
| Serving | **vLLM** (default; PagedAttention, grammar/XSD constrained decoding) | Melhor throughput bruto |
| Copiloto on-prem | **SGLang** avaliado no serviço de copiloto — **RadixAttention** com prefixo compartilhado de regras | Nossos prompts de copiloto compartilham 30–60k tokens de contexto estável (catálogo) → radix cache amortiza; até 5× vLLM em geração estruturada com prefixo comum |
| Alternativa menor | Qwen3-30B-A3B FP8 em 1×L40S ($3.5/h = ~$2.555/mês) | Quando o orçamento manda; queda de qualidade aceitável no copiloto |

Mesma GPU serve embeddings (Qwen3-Embedding) + reranker (BGE-Reranker-v2-M3) junto do LLM.

**Crossover:** a infra on-prem só compensa acima de **100–300 IFs** — ou antecipada por 1–2 clientes enterprise air-gapped que paguem o H100 sozinhos (ver TCO §9).

---

## 6. MCP E AGENTES

**SDK:** `rmcp`, SDK oficial Rust do projeto MCP (Tier 1 de conformance; spec **2026-07-28**). O Radiant expõe e consome MCP:

- **MCP server** (crate `norma-ia`): tools `query_rule`, `validate_cadoc`, `explain_violation` — todas **read-only, allow-listed, scoped por tenant**. Agentes externos (do próprio time ou do cliente) operam o validador determinístico via MCP; o produto IA-native com tools Rust é o pitch.
- **MCP client** (conectores das IFs): consumir core banking da IF via tools **read-only, scoped, allow-listed** — nada de escrita em sistema do cliente.

**Segurança contra prompt injection** (vetores via MCP sampling documentados — Unit42; guia oficial Microsoft; conteúdo regulatório de terceiros pode trazer texto malicioso):

1. **Conteúdo recuperado é dado citado, nunca instrução** — normativos e dados da IF entram delimitados como evidência; instrução vive só no system prompt.
2. Tool allow-list + least privilege; logs de toda tool call (auditoria).
3. **Humano-no-loop em todo envio ao BACEN** — nenhum agente transmite; o guardrail de output roda antes de qualquer fila de transmissão.
4. Guardrail de output + validação `jsonschema` antes do consumo de qualquer resposta de modelo.

**Agente "preparar CADOC" = workflow graph, não loop livre:** pipeline fixo por tipo de CADOC (planner determinístico) com nós de verificação entre etapas; LLM só nos nós de decisão/extração (classificar erro ambíguo, sugerir correção). Agentic-loop aberto em reporting regulatório é o anti-padrão que o relatório "Agentic Gap" da Regnology documenta: sem grafo auditável, não há como explicar ao auditor por que o sistema fez X.

---

## 7. CORPUS SINTÉTICO (challenger / solver / judge)

O corpus de avaliação cresce por **cobertura da matriz regra×edge** (boundary / negative / mutation / real_world / combo), não por volume. Pipeline em 4 estágios:

```
 Célula vazia da matriz regra×edge
        ▼
 CHALLENGER (LLM barato — Gemini 3 Flash + personas de IF:
 banco múltiplo, fintech, cooperativa) → gera perfil de dados
 que deve violar/satisfazer regras específicas
        ▼
 SOLVER (Rust puro, ZERO LLM) → monta o CADOC real (XML/JSON)
 a partir do perfil; grounding no XSD é estrutural
        ▼
 JUDGE 1 (determinístico) → engine valida contra XSD + regras
 → rótulo ground-truth automático (violou quais regras, por quê)
        ▼
 JUDGE 2 (LLM, só semântica) → Claude Sonnet revisa realismo
 da narrativa em ~10% de amostra
        ▼
 eval_case rotulado (provenance='synthetic') → golden set de
 extração + eval de RAG + futuro LoRA de estilo
```

**Por que funciona:** rejection sampling + distillation funcionam quando o verificador é objetivo — e o nosso judge 1 é o engine Rust/XSD, não um LLM. **Custo:** ~$0.05–0.20 por 1k exemplos (challenger Flash ~2k tokens/exemplo) + juízes; **100k CADOCs sintéticos ≈ $20–80 de API**. O custo real é engenharia do harness e storage, não tokens.

**Honesto sobre self-improvement:** "melhora com o uso" significa delta rotulado por verificador objetivo alimentando eval e futuro destilado — não auto-reescrita de prompt/modelo. Métrica fixa por versão de catálogo + replay de erros reais das IFs (opt-in) como corpus de regressão.

---

## 8. FINE-TUNING

| Abordagem | Custo | Risco | Quando |
|---|---|---|---|
| RAG + engine determinístico | ~$0 incremental | baixo | **Default** — regras versionadas mudam toda semana via Radar; RAG atualiza em segundos |
| LoRA de estilo (tom/formato do texto de explainability) | $50–500/run QLoRA (Unsloth; ~$5 GPU bruta p/ 7–8B) | médio-baixo | Quando o formato do texto ao cliente precisa ser consistente |
| Fine-tune "aprender regras BACEN" | $5k–30k | **alto** — overfit na versão do normativo, rollback difícil, auditoria não confia em peso opaco sem provenance | **Nunca para conhecimento versionado** |

**Regra:** não fine-tunar conhecimento versionado. Regulatório é o pior caso para fine-tune: o conteúdo muda semanalmente e cada output exige provenance citável — peso opaco não cita. **Distillation** entra só se o copiloto migrar on-prem: destilar o *formato de saída* do Sonnet (não o conhecimento) no GPT-OSS-120B, que já é CoT-treinável e aceita LoRA leve.

---

## 9. TCO MENSAL (infra IA) + ROLLOUT

Premissas: copiloto em ~10% dos eventos, ~25k tokens de contexto (cache hit 80%), ~1.500 tokens de saída; extração ~40 normativos/mês; RAG 3 consultas/validação.

### Cenário A — 10 IFs (~3.000 chamadas LLM/mês)

| Item | Cloud híbrido | Custo | On-prem | Custo |
|---|---|---|---|---|
| Copiloto/explain | Gemini 3 Flash (cache) | ~$90 | GPT-OSS-120B @ L40S OCI | $2.555 |
| Casos duros (10%) | Sonnet 4.5 | ~$120 | — | — |
| Extração regras | Gemini 3 Pro + Sonnet critic | ~$200 | idem (frontier) | $200 |
| Embeddings | Qwen3-Emb self | $0 | self | $0 |
| Rerank | BGE self | $0 | self | $0 |
| RAGAS/eval LLM judge | Flash batch | ~$15 | self | $0 |
| Harness sintético | Flash | ~$60 | idem | $60 |
| Parsing | Docling CPU | $20 | idem | $20 |
| **Total** | | **≈ $505/mês** | | **≈ $2.835/mês** |

### Cenário B — 100 IFs (~30.000 chamadas LLM/mês + agêntico)

| Item | Cloud híbrido | Custo | On-prem (híbrido) | Custo |
|---|---|---|---|---|
| Copiloto/explain | Flash + Sonnet nos 10% | ~$1.900 | GPT-OSS-120B @ H100 OCI | $7.300 |
| Extração regras | frontier | ~$400 | frontier | $400 |
| Harness (2M exemplos/mês) | Flash batch | ~$250 | on-prem | ~$0 marginal |
| Eval/observabilidade | batch | ~$150 | self | $0 |
| Embeddings (30M tokens/mês) | Gemini Emb 2 | $6 | self | $0 |
| Parsing | Docling + OCR | ~$25 | idem | $25 |
| MCP runtime | self (Rust) | $80 | idem | $80 |
| **Total** | | **≈ $2.800/mês** | | **≈ $7.800/mês** |

**Leitura:** até ~100 IFs, cloud híbrido custa <$3k/mês; a GPU on-prem só vence quando (a) cliente exige PII never-leave, (b) insights não-opt-in explodem ou (c) batch pesado de sintético. Crossover: **100–300 IFs**.

### Rollout em 3 estágios

| Estágio | Gatilho | O que muda |
|---|---|---|
| **1. MVP — 100% API** | hoje | Frontier + **router multi-modelo + sanitizador PII já no MVP** (~1 sprint; desbloqueia os dois mundos sem comprar GPU) |
| **2. L40S on-prem** | 1º cliente exigir residência | Nó OCI São Paulo com Qwen3-30B-A3B + Qwen3-Embedding + BGE-Reranker (~$2.555/mês); copiloto degradado consciente |
| **3. H100 + GPT-OSS-120B** | >~50 IFs ou 2 clientes air-gapped | Copiloto on-prem completo via vLLM/SGLang (~$7.300/mês); frontier mantida só para extração (normativo é público) |

---

## 10. ARQUITETURA COMPLETA DE IA NO MOTOR

```
┌────────────────────────────────────────────────────────────────────────────┐
│                     RADIAN T NORMA — ENGINE DE IA (Rust)                   │
│                                                                            │
│  BACEN: DOU / cartas circulares / XLSX de leiaute                          │
│      │                                                                     │
│      ▼                                                                     │
│  ┌────────────┐   ┌─────────────────────────────────────────────────────┐  │
│  │ RADAR      │──►│ INGESTÃO                                            │  │
│  │ hash por § │   │ Docling (PDF) · calamine (XLSX, Rust) · Mistral OCR │  │
│  │ diff delta │   │ → markdown por § + cartões de campo + JSON leiaute  │  │
│  └────────────┘   └────────┬────────────────────────────────────────────┘  │
│                            ▼                                               │
│  ┌──────────────────────────────────────┐  ┌─────────────────────────────┐ │
│  │ PIPELINE NORMA→REGRA (crate norma-ia)│  │ RAG NORMATIVO               │ │
│  │ DRAFT  Gemini 3 Pro (visão, 1M ctx)  │  │ pgvector HNSW (embeddings)  │ │
│  │ CRITIC Claude Sonnet 4.5 (vs §)      │  │ + Tantivy BM25 (Rust)       │ │
│  │ TEST   engine Rust: XSD + fixtures   │  │ fusão RRF + BGE-Reranker    │ │
│  │ HUMAN  UI de diff → catálogo         │  │ emb: Qwen3 / Gemini Emb 2   │ │
│  └───────────────┬──────────────────────┘  └─────────────────────────────┘ │
│                  ▼                                                         │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ CATÁLOGO (Postgres) — ~1.099+ regras versionadas, provenance §       │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                  ▼                                                         │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ MOTOR RUST — execução determinística (validação, XSD, transmissão)  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│        │                          │                        │               │
│        ▼                          ▼                        ▼               │
│  ┌───────────────┐   ┌────────────────────┐   ┌─────────────────────────┐  │
│  │ COPILOTO      │   │ HARNESS SINTÉTICO  │   │ MCP (rmcp, SDK oficial) │  │
│  │ Sonnet cloud  │   │ challenger (Flash) │   │ server: query_rule /    │  │
│  │ ou GPT-OSS    │   │ solver (Rust/XSD,  │   │ validate_cadoc /        │  │
│  │ 120B on-prem; │   │ zero LLM) →        │   │ explain_violation       │  │
│  │ sanitizador   │   │ judge det.+LLM 10% │   │ read-only allow-listed; │  │
│  │ PII antes;    │   │ corpus rotulado    │   │ HITL p/ envio BACEN;    │  │
│  │ provenance    │   │ ~$0.05–0.2/1k      │   │ conteúdo = dado citado, │  │
│  │ obrigatória   │   │                    │   │ nunca instrução         │  │
│  └───────────────┘   └────────────────────┘   └─────────────────────────┘  │
│                                                                            │
│  ┌──────────────────────── ROUTER MULTI-MODELO ─────────────────────────┐  │
│  │ classificação de dados: público → frontier · sanitizado+ZDR → cloud  │  │
│  │ · PII never-leave → on-prem · feature flag por cliente               │  │
│  └──────────────────────────────────┬───────────────────────────────────┘  │
│                                     ▼                                      │
│   { Gemini 3 Pro/Flash · Claude Sonnet/Opus · GPT-5.1 }                    │
│   { GPT-OSS-120B / Qwen3-A3B — vLLM ou SGLang, H100/L40S OCI SP }          │
│                                                                            │
│  Frontend Next.js ──► API axum (SSE) ──► tudo acima (IA nunca no client)   │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## 11. RISCOS MONITORADOS

| # | Risco | Severidade | Mitigação |
|---|---|---|---|
| 1 | **Promo pricing Gemini vence dez/26–jan/27** (Flash $0.50 e 3.1 Pro $1 são introdutórios; sobe ~2027) | Alta | **Router multi-modelo é obrigatório, não opcional** — trocar modelo em 1 dia; custo reavaliado no review de jan/27 |
| 2 | **rmcp jovem** (SDK Tier 1 mas com CVE recente no histórico) | Média | Fixar versão no `[workspace.dependencies]`; **conformance tests no CI**; acompanhar releases do spec 2026-07-28 |
| 3 | Residência LatAm inexistente nos 3 frontier providers | Média | Sanitização PII obrigatória (§5); monitorar GCP São Paulo (Blackwell out/26) para residência contratual futura |
| 4 | APIs baratas de terceiros (DeepSeek/Qwen) com jurisdição opaca | Média | Só para normativos públicos ou self-host; nunca dado de cliente |
| 5 | Long-context tratado como banco de dados (custo armazenado $0.50–4.50/Mtok/h + degradação multi-shot needle) | Média | RAG híbrido é a única via de consulta recorrente; ctx 1M reservado à extração one-shot |
| 6 | Dependência de revisão humana virar gargalo de novo | Média | SLA de revisão é o produto (audit trail), não custo; meta 20min/doc; medir throughput da fila |
| 7 | Verifier-gaming / judge virar alvo | Média | Judge determinístico (XSD+engine) como juiz final; LLM-judge só semântica em 10%; judge de família diferente do gerador |

---

## 12. Fontes principais

- Norm AI (agentic law; parceria Microsoft fev/2026): https://www.norm.ai
- Regnology RGI + "The Agentic Gap" (2026): https://www.regnology.net/en/solutions/regnology-platform/rgi
- LegalRuleML (OASIS): https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/legalruleml-core-spec-v1.0.html
- Preços Gemini/Anthropic/OpenAI (nov/2026): https://aistudio.google.com/docs/pricing · https://www.anthropic.com/pricing · https://developers.openai.com/api/docs/pricing
- MCP spec 2026-07-28 + SDK Rust (rmcp): https://modelcontextprotocol.io/docs/2026-07-28/sdk · https://github.com/modelcontextprotocol/rust-sdk
- Injeção via MCP (Unit42 / Microsoft): https://unit42.paloaltonetworks.com/model-context-protocol-attack-vectors · https://developer.microsoft.com/blog/protecting-against-indirect-injection-attacks-mcp
- vLLM vs SGLang (Red Hat, out/2026): https://developers.redhat.com/articles/2026/10/14/vllm-vs-sglang-choosing-the-right-inference-engine-for-your-workload
- GPT-OSS-120B hardware: https://openai.com/index/introducing-gpt-oss
- Preços GPU (OCI uniforme / AWS sa-east-1): https://www.oracle.com/cloud/price-list/ · https://instances.vantage.sh/aws/ec2/g6e.xlarge
- LGPD/transferência internacional (ANPD Res. 29/2024): https://www.trade.gov/market-intelligence/brazils-new-rules-international-data-transfers
- Distillation custo ($0.36–8.74/1k): https://arxiv.org/html/2410.18588v1 · RLHF Book cap. 12: https://rlhfbook.com/c/12-synthetic-data
- RAGAS: https://docs.ragas.io · Docling/parsers: https://agentscamp.com/guides/comparisons/best-document-parsers-for-rag-2026
