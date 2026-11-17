# 07 — Blueprint do Engine de IA (Radiant Norma)

**Data:** 2026-11-16 · **Escopo:** arquitetura de IA para extração regulamento→regra, copiloto de validação, RAG normativo, agentes MCP e harness sintético · **Status:** pesquisa de mercado nov/2026, preços verificados em fontes públicas (links ao final).

---

## 0. Veredito em uma página

| Função | Pick primário | Fallback / secundário | Custo de entrada |
|---|---|---|---|
| Extração norma→regra (draft) | **Gemini 3 Pro** ($2/$12, ctx 1M, visão nativa p/ PDF+tabela) | Claude Opus 4.5 ($5/$25) para casos duros | ~$15–40 por resolução grande |
| Extração em volume / triagem | **Gemini 3 Flash** ($0.50/$3) | DeepSeek V3.2 (~$0.55/$1.75) | centavos por doc |
| Copiloto de validação campo-a-campo | **Claude Sonnet 4.5** ($3/$15, cache 10%) | GPT-5.1 ($1.25/$10) | ~$0.03–0.12/validação |
| Explainability + AI Insights (opt-in) | Gemini 3 Flash → Sonnet quando precisa de profundidade | GPT-5.1-mini | < $0.02/insight |
| RAG embeddings | **Qwen3-Embedding** (open, on-prem LGPD) / **Gemini Embedding 2** ($0.20/M) | BGE-M3 | ~$0–20/mês |
| Reranking | **BGE-Reranker-v2-M3 / Qwen3-Reranker** (self-host) | Cohere Rerank (~$2/1k buscas) | $0 (self) |
| Vetor store + BM25 | **pgvector + Tantivy** (Rust, já alinhado ao motor Go→Rust) | Qdrant se filtragem pesada | $0 infra extra |
| Parsing PDF/XLSX | **Docling** (open, tabelas) + **calamine** p/ XLSX determinístico | Mistral OCR ($2/1k págs) p/ scanneados | $0–50/mês |
| LLM on-prem (LGPD/PII) | **GPT-OSS-120B** (MXFP4, 1×H100 ou Mac 128GB) | Qwen3-30B-A3B (MoE, throughput alto) | $3.5–7.3k/mês GPU |
| Serving engine | **vLLM** (default) / **SGLang** (prefixos compartilhados) | llama.cpp/Ollama só p/ dev | — |
| Agentes/conectores | **MCP spec 2026-07-28 + rmcp 2.x** (SDK oficial Rust) | — | dev time |
| Harness sintético | challenger→solver→judge com **verificador determinístico (XSD) como juiz final**; LLM judge só p/ semântica | — | $1–9/1k exemplos |
| Fine-tuning | **Não fine-tunar para regras versionadas.** LoRA só p/ estilo/formato de explainability | QLoRA (Unsloth) se um dia precisar | $5–500/run |

**Princípio arquitetural:** determinismo no engine (Go/Rust compila regras), IA na fronteira (extração, explicação, geração de teste) — todo LLM-output entra no produto só após verificador determinístico ou humano aprovar. Isso é exatamente o padrão de Norm AI ("agentic law": norma vira agente com programa executável + testes) e de Regnology (dados padronizados + agêntica sobre engine determinístico).

---

## 1. Extração de regulamento → regra executável

### 1.1 Como o mercado faz (nov/2026)

- **Norm AI** (norm.ai) — o padrão de ouro. Converte CFR/títulos estaduais em *programas executáveis + agentes de IA* ("agentic law"). Parceria com Microsoft (fev/2026) embute a inteligência regulatória no Word/Outlook. Serve Fortune 100. O insight replicável: a unidade de entrega não é "resumo do normativo", é **código + suite de testes por regra**, auditável.
- **Regnology** — engine determinístico + camada agêntica (RGI) sobre dados padronizados; relatório "The Agentic Gap" (2026) mapeia prontidão de IA em regulatory reporting. Conclusão do setor: agência só funciona sobre dados/regras normalizadas — validar tese do Radiant Norma.
- **"Kaplan"**: não existe player relevante com esse nome em regulation-as-code (verificado). Referências reais do domínio: OECD Rules-as-Code (white paper 2020), Catala (DSL jurídico, França), Blawx, LegalRuleML (OASIS), Theory and Principle/Factor (Map Engine).
- **Academia:** extração de obrigações deonticas via LLM → LegalRuleML / lógica deôntica derrotable (CEUR Vol-4174; ACM ICAIL 2025 "Lost in EU Regulation"); PROLEG neuro-simbólico (HKU); Catala como alvo de compilação. 2025–2026 foi o ano em que LLM + formalismo deôntico virou pipeline publicável — i.e., a técnica está madura o suficiente para produção com verificação.

### 1.2 O que funciona vs hype

**Funciona (evidenciado em produção/academia):**
1. **LLM extrai *candidato* de regra em JSON estruturado** (schema fixo: id, tipo, ator, condição, evento, limite, fonte §, vigência). Constrained decoding zera parse-failure (vLLM grammar / native structured outputs dos 3 provedores) — mas **JSON válido ≠ semântica correta**: validação de valores continua necessária.
2. **Verificação em 3 camadas:** (a) verificador determinístico — a regra extraída roda contra o XSD/leiaute do CADOC e contra fixtures douradas; (b) LLM-critic compara regra vs. texto-fonte § por § (evaluator–optimizer); (c) humano aprova no catálogo. Rejection sampling só é confiável com verificador objetivo — validação XSD/execução de teste é exatamente isso.
3. **Visão-linguagem para PDFs de cartas circulares:** Gemini 3 Pro / Claude Opus 4.5 leem página da BACEN (tabela de leiaute, rodapé, fórmula) direto como imagem+texto; long-context (1M Gemini) engole resolução inteira + planilha de leiaute juntos — elimina a etapa mais frágil (chunking prévio).
4. **XLSX de leiaute NÃO vai por OCR:** parse determinístico (calamine em Rust / openpyxl) → JSON de célula+header+tipo; LLM só interpreta o JSON já estruturado. Hype detectado: "OCR resolve planilha" — não resolve, e doc-parsers de OCR erram merged cells (Reducto leaderboard: melhor do setor 90.2%).

**Hype / evitar:**
- Fine-tune que "aprende o regulamento" (ver §7).
- 100% autômato sem review humana — catálogo é ativo regulatório; erro silencioso vira multa do cliente. Meta realista: IA pré-extrai ~80% com certeza ≥ golden set, humano confirma ~20min/doc vs ~8h manual hoje (gargalo atual de ~1.099 regras manual).
- Long-context como banco de dados: custo de contexto armazenado Gemini ($0.50–4.50/Mtok/h) e degradação em "multi-shot needle" — RAG continua mais barato para consulta recorrente.

### 1.3 Pipeline recomendado (agentic, com verificação)

```
Normativo (PDF/DOU) ──► Docling (layout+tabelas) ──► Doc markdown + metadados §
Leiaute (XLSX) ──────► calamine (Rust) ────────────► JSON estruturado de campos
        │
        ▼
[DRAFT] Gemini 3 Pro (visão, long-ctx) → N candidatos de regra (JSON schema, temp 0.2, n=3 amostras)
        ▼
[CRITIC] Claude Sonnet 4.5 — diff candidato vs § fonte; rejeita/aloca evidência; score confiança
        ▼
[TEST] Compilar regra → teste executável no engine Rust; rodar contra golden fixtures (CADOCs reais + XSD)
        ▼
[JUDGE determinístico] passou? → fila de revisão humana (UI diff §↔regra) → catálogo versionado
        ▼
[RADAR] monitora mudança no normativo → re-dispara pipeline só nos § alterados (delta)
```

Rejection sampling no DRAFT (3–10 amostras, julgadas pelo CRITIC+TEST) custo ~$0.30–3/candidato — ordem de grandeza abaixo do custo humano que substitui.

### 1.4 Taxonomia de regras (alinhada a LegalRuleML/deôntica)

| Classe | Exemplo CADOC | Execução |
|---|---|---|
| Estrutural | campo existe, tamanho, tipo, obrigatoriedade | XSD/leiaute — determinístico |
| Semântica (deôntica) | "obrigatório se X", limite, proibição | motor regras Rust compilada |
| Cross-doc | consistência entre CADOCs / quadro↔quadro | motor + join de dados |
| Temporal/histórica | vigência, base de cálculo por data, backporting | motor + timeline versionada |
| Interpretativa (soft) | "preencher quando aplicável", julgamento | IA sugere, humano decide |

Hoje o catálogo manual de ~1.099 regras é majoritariamente semântica+estrutural; as classes 3–4 são o diferencial competitivo e o maior ganho da extração automática.

---

## 2. Modelos por função (nov/2026)

### 2.1 Frontier APIs (preços por 1M tokens, verificados nov/2026)

| Modelo | Input | Output | Cache read | Notas | Uso no Radiant |
|---|---|---|---|---|---|
| **Gemini 3 Pro** | $2.00 (≤200k) / $4.00 (>200k) | $12 / $18 | ~10% | ctx 1M, visão+áudio/vídeo nativo, thinking | extração draft com PDF+XLSX juntos |
| **Gemini 3 Flash** | $0.50 | $3.00 | ~10% | intro até 31/dez/26; sobe ~2027 | triagem, insights, batch |
| Gemini 3.1 Pro | $1.00 (promo, era $2) | $6.00 | — | promo | idem 3 Pro |
| **Claude Opus 4.5** | $5.00 | $25.00 | ~$0.50 | melhor raciocínio longo, escreve críticas/regras difíceis | casos duros, juiz semântico |
| **Claude Sonnet 4.5/4.6** | $3.00 | $15.00 | $0.30 | default de qualidade; structured outputs nativos | copiloto de validação |
| Claude Haiku 4.5 | $1.00 | $5.00 | — | rápido | classificação simples |
| **GPT-5.1** | $1.25 | $10.00 | $0.13 | batch 50% off | alternativa copiloto |
| GPT-5.1 mini | ~$0.25 | ~$2.00 | — | barato | triagem |
| Mistral Medium 3.1 | $0.40 | $2.00 | — | multimodal, 131k ctx | option ESG/UE; fallback |
| DeepSeek V3.2 | $0.50–0.60 | $1.60–2.20 | — | open-weight; preço subiu ~3x ago/26 | volume, off-peak barato |

Todas as 3 APIs: batch ~50% off, cached-read ~10%, ZDR disponível em tiers API. **Importante para LGPD:** nenhum dos três tem residência de dados no Brasil (OpenAI: EU/US/CA/JP/KR/SG/IN/AU/UAE; Anthropic/Vertex: US/EU). Google Cloud anunciou servidores Blackwell em São Paulo (out/2026), mas ainda sem residência Claude/Gemini-LatAm documentada → ver §3.5.

### 2.2 Open-weights (on-prem / LGPD)

| Modelo | Params | Roda em | Qualidade (regra PT-BR) | Papel |
|---|---|---|---|---|
| **GPT-OSS-120B** | 117B MoE (5.1B ativos), MXFP4 ~61GB | 1×H100 80GB; Mac 128GB; 2×RTX 5090 (apertado) | próximo de o3-mini/o4-mini; forte em structured outputs | **on-prem default** — copiloto sem PII vazando |
| Qwen3-30B-A3B (3.5) | 30B MoE (3.3B ativos) | 1×L40S/A100; até RTX 5090 | ~350+ tok/s geração real via vLLM | throughput alto, insights |
| Qwen3-32B (dense) | 32B | 1×L40S AWQ / 2×5090 | boa em PT-BR | alternativa densa |
| DeepSeek V3.2 | 671B MoE | 8×A100/H100 (~$10.7k/mês self) | top open reasoning | só via API (self-host inviável p/ nós) |
| Llama 4 Maverick/Scout | 400B/109B MoE | multi-GPU | meta abandonou frontier aberto em 2025; perder liderança p/ Qwen | evitável |
| Mistral Large/3 | — | 2–4×GPU | EU-compliance forte | opção de juridico-EU |

**PT-BR custo/qualidade hoje:** API Gemini 3 Flash > tudo em custo-benefício para extração; GPT-OSS-120B é o melhor *on-prem* (open, Apache-2.0, foi destilado com chain-of-thought treinável — permite fine-tune leve legal); Qwen3-A3B quando o gargalo é throughput batch.

### 2.3 Embeddings & rerankers

| Modelo | Preço | Dim | Multilingue/PT | Papel |
|---|---|---|---|---|
| **Gemini Embedding 2** | $0.20/M ($0.10 batch) | ~3k truncável | top MTEB/MARQ | se RAG ficar em cloud |
| Cohere Embed 5 Pro/Fast | $0.12/$0.08 | — | boa | alternativa |
| Voyage 4 (lite) | $0.02–0.12 | — | top precisão API | alternativa |
| **Qwen3-Embedding (0.6B/4B/8B)** | $0 self-host (GPU pequena) | at 2568 | top open multilingue (MMTEB) | **on-prem default** |
| BGE-M3 | $0 | 1024, dense+sparse+colbert | muito usado, robusto | fallback on-prem |
| OpenAI text-embedding-3-small | $0.02 | 1536 | ok | só se tudo já no OpenAI |

| Reranker | Custo | Nota |
|---|---|---|
| **BGE-Reranker-v2-M3 / Qwen3-Reranker** | $0 self (GPU pequena/CPU) | consenso 2026: melhor valor; precisa bater Cohere p/ justificar pago |
| Cohere Rerank 4/5 | ~$2.00/1k search units (100 docs/un) | melhor gerenciado; usar se não quiser GPU rerank |

---

## 3. Inferência on-prem (nov/2026)

### 3.1 Engines

| Engine | Throughput | Prefix caching | Structured output | Veredito |
|---|---|---|---|---|
| **vLLM** | melhor throughput bruto default (PagedAttention); venceu Qwen-serving em benchmark Jarvis Labs H100 | sim | grammar/XSD (outlines/xgrammar) | **default do stack** |
| **SGLang** | p50 −37% vs vLLM num estudo; até 5× vLLM em geração estruturada com prefixos compartilhados (RadixAttention) | radix, melhor da classe | sim | quando muitos prompts compartilham prefixo (nossos prompts de regra são grandes e estáveis — caso perfeito) |
| TensorRT-LLM | pico NVIDIA H100 | sim | parcial | só se eng dedicado NVIDIA |
| Ollama / llama.cpp | baixo (sem batching contínuo sério) | básico | GGUF parcial | dev notebook, demo, Mac Studio |
| TGI | perdeu relevância vs vLLM/SGLang | sim | parcial | evitar |

Recomendação: **vLLM como base + SGLang avaliado no serviço de copiloto** (prefixo de prompt com catálogo de regras é estável e enorme → radix cache amortiza 30–60k tokens de contexto por chamada). Red Hat (out/2026) confirma os dois como escolhas saneáveis por workload.

### 3.2 Hardware — preços reais nov/2026

| Opção | Custo | Roda o quê | Observações |
|---|---|---|---|
| AWS sa-east-1 g6e.xlarge (1×L40S 48GB) | **~$1.86/h** (~$1.360/mês 24×7) | Qwen3-30B-A3B, Qwen3-32B-AWQ | prêmio sa-east-1 ~3×US; g6e.48x ~$30/h |
| **OCI sa-saopaulo-1 — preço uniforme global** | A10 ~$2–4/h; **L40S ~$3.5/h**; **H100 $10/h** | H100: GPT-OSS-120B cheio | melhor GPU LatAm preço/dado; preemptible −50/70% |
| GCP São Paulo (Blackwell, out/26) | ~H100-parity | B200/Blackwell | residência física BR, sem residência contratual IA ainda |
| Hetzner dedicated RTX | desde €78,70/mês (refurb) | 1–2×RTX (RAM p/ offload) | dados fora do BR; hikes de preço jun/26; sem A100/H100 |
| Mac Studio M4 Max 128GB | $3.499 one-time | Qwen3-32B, GPT-OSS-120B (dev/QA) | silencioso, escritório; ~60 tok/s GPT-OSS |
| Mac Studio M5 Ultra 256GB | $9.499 one-time (set/26) | 120B confortável + batch moderado | alternativa datacenter-in-office |
| RTX 5090 32GB | $1.999–2.199 | Qwen3-30B-A3B, 32B-AWQ, embeddings+rerank | 2×5090: GPT-OSS-120B marginal (~60+ tok/s c/ offload) |
| NVIDIA DGX Spark 128GB | ~$4.000 | 120B dev | unified memory, dev |

### 3.3 TCO stack on-prem (rodando GPT-OSS-120B *ou* Qwen3-A3B com bom throughput)

**Cenário produção (recomendado):**
- 1× OCI H100 (São Paulo, $10/h, 24×7) = **$7.300/mês** → serve GPT-OSS-120B (~68GB) via vLLM/SGLang; sobra para embeddings/rerank na mesma GPU.
- Alternativa 60% mais barata: 1× L40S OCI ($3.5/h = $2.555/mês) → Qwen3-30B-A3B FP8 + BGE-M3 + reranker; GPT-OSS-120B não cabe (48GB) — trocar por Qwen3-32B-AWQ se aceitar queda de qualidade no copiloto.
- Dev/staging: Mac Studio 128GB $3.499 one-time (amortiza < $100/mês/24m).

### 3.4 Comparativo rápido de cenários on-prem

| Stack | CAPEX | OPEX/mês | Serviço | Fit |
|---|---|---|---|---|
| OCI H100 + vLLM + GPT-OSS-120B | $0 | $7.300 | copiloto on-prem completo, PII nunca sai | 100+ IFs, compliance-first |
| OCI L40S + Qwen3-30B-A3B | $0 | $2.555 | insights+RAG-gen on-prem | 10 IFs |
| 2×RTX 5090 local (escritório) | ~$5k | ~$150 energia | dev, demo, QA de modelos | dev only |
| Mac Studio cluster ×2 | ~$7–19k | ~$200 energia | staging, batch noturno | dev/staging |

### 3.5 Hybrid cloud (PII fora, frontier dentro) — viável em LGPD?

**Sim, com 3 condições** (base: ANPD Resolução CD/ANPD nº 29/2024 sobre transferência internacional, em vigor desde ago/2025; multas até R$50M; estudo FGV aponta que nenhuma grande plataforma cloud passa 100% em LGPD por padrão):

1. **Redação/pseudonimização antes do envio:** pipeline de sanitização (NER on-prem: CPF/CNPJ/nome/conta) roda no vLLM/regex antes de qualquer API frontier. A IA frontier vê regra + estrutura + dados mascarados — não precisa de PII para explicar validação.
2. **Contrato + ZDR:** API tiers com zero-data-retention (OpenAI/Anthropic/Gemini oferecem em tiers API), DPA com cláusula padrão contratual ANPD, opt-out de treino (default nas APIs).
3. **Classificação de dados por rota:** documento normativo BACEN (público) → frontier sem restrição; dados de clientes IF → só on-prem. Feature flag por cliente (alguns IFs exigirão 100% on-prem — o stack GPT-OSS já cobre).

---

## 4. RAG sobre normativos

### 4.1 Ingestão

- **PDF (cartas circulares, resoluções):** Docling (open-source, melhor layout/table recovery gratuito; ~94–96% acurácia estrutural; 2–5 pág/s CPU) → markdown hierárquico com § e tabelas preservadas. Scanneados raros: Mistral OCR ($2/1k págs, OCR 3). Reducto/Azure DI só se precisarem de >90% em tabelas escaneadas complexas (pago).
- **XLSX leiaute:** calamine (Rust) — determinístico, célula-a-célula, com headers mesclados resolvidos por regra; nunca embedder cru sobre planilha.
- **Chunking:** por § com overlap semântico; tabelas de leiaute viram "cartão de campo" (1 doc estruturado por campo CADOC: nome, tipo, tamanho, domínio, regras associadas) — embedded individualmente; fórmulas viram texto canônico + fonte.

### 4.2 Retrieval

```
Query ("regra preenchimento campo 109 CADOC 4010")
  ├── BM25 (Tantivy, Rust) — ids exatos, números de campo, siglas
  ├── dense (Qwen3-Embedding / Gemini Embedding 2)
  ├── RRF fusion → top 50
  └── rerank BGE-Reranker-v2-M3 → top 8 → LLM
```

- **Store:** pgvector (default, zero infra nova, já teremos Postgres; HNSW + filtering) + **Tantivy** para BM25 — é Rust, casa com o motor Go→Rust e evita Qdrant como serviço extra. LanceDB (embedded, Rust, disk-based) é o plano B se quisermos tudo embedded sem Postgres. Qdrant só se a filtragem por metadado virar complexa (payload filters pesados).
- Números de campo/siglas ("campo 109", "CADOC 3050") são o ponto fraco de dense-only — BM25 resolve; híbrido obrigatório (consenso Encore/vdf.ai 2026).

### 4.3 Eval e atualização

- **Eval:** RAGAS (faithfulness, context precision/recall) dentro de DeepEval (pytest/CI) + promptfoo para regressão de prompt. Golden set: 200 perguntas curtadas por compliance (criar no bootstrap do catálogo).
- **Incremental (Radar→index):** hash por § na ingestão; BACEN publica → diff § → re-embed só do delta (~segundos); re-registrar regras afetadas no catálogo com flag "normativo mudou, regra desatualizada?" → dispara pipeline da §1.3 sobre o delta. Custo típico de atualização: <$1 por documento alterado.

---

## 5. Agentes / MCP

- **Maturidade (nov/2026):** protocolo estável, spec **2026-07-28** com **SDK oficial Rust (`rmcp`)** — v2.x, já com CVE corrigido e 100% conformance em SDKs concorrentes; adoção em editores (Zed) e cloud (Agent Platform Google, Azure Foundry). MCP é o padrão de fato para conectar agentes a dados — usar como camada de conectores é aposta segura.
- **Conectores de dados das IFs:** expor API do Radiant Norma como MCP *server* (tools: `query_rule`, `validate_cadoc`, `explain_violation`) e consumir core banking das IFs via MCP *client* com **tools read-only, scoped, allow-listed**. Padrão de agente para "preparar um CADOC": planner determinístico (pipeline fixo por CADOC, não agente livre) + LLM só nos nós de decisão/extração. Agentic-loop aberto em reporting regulatório = risco de auditoria; workflow graph com verificação é o padrão defendido pelo setor (Regnology "Agentic Gap").
- **Segurança (prompt injection em dados regulatórios):** Unit42 documentou vetores de injeção via MCP sampling; Microsoft publicou guia oficial de mitigação. Baseline: (1) tool allow-list + least privilege; (2) human-in-the-loop para toda ação de escrita/envio ao BACEN; (3) conteúdo recuperado (normativos, dados da IF) entra como *dados citados*, nunca como instrução (delimitação + instrução no system); (4) logs de toda tool call (auditoria); (5) guardrail de output antes do envio. Injeção via normativo publicado é cenário real (BACEN PDF é vetado por pipeline oficial, mas conectores de terceiros podem trazer texto malicioso).

---

## 6. Harness sintético (challenger / solver / judge)

- **Estado da arte:** convergência generate→verify→filter→distill (RLHF Book cap. 12; Apple "environment-free synthetic data for API agents"; Self-Verified Distillation arXiv 2605.26132). Achar conclusão dura da literatura: **rejection sampling funciona bem quando o verificador é objetivo (execução de código, validação de schema) e degrada quando só existe LLM-judge** — no nosso caso temos XSD + engine Rust = verificador objetivo. Essa é a vantagem estrutural do Radiant Norma.
- **Pipeline recomendado:**
  1. **Challenger** (LLM, Gemini 3 Flash barato + personas de IF: banco múltiplo, fintech, cooperativa): gera *perfis de dados* que devem violar/satisfazer regras específicas do catálogo.
  2. **Solver** (determinístico, Rust): monta o CADOC real (XML/JSON) a partir do perfil — sem LLM aqui, grounding no XSD é estrutural.
  3. **Judge 1 (determinístico):** engine valida contra XSD + regras → rótulo ground-truth automático (violou quais regras, por quê).
  4. **Judge 2 (LLM, só semântica):** Claude Sonnet revisa se a narrativa do caso é realista (amostragem ~10%).
  5. **Distill:** corpus rotulado alimenta (a) golden set de extração, (b) eval de RAG, (c) futuro LoRA de estilo.
- **Custo medido:** destilação com teacher frontier: **$0.36–8.74 por 1k exemplos** (arXiv 2410.18588); com Gemini 3 Flash gerando perfis (~2k tokens/exemplo) cai para **~$0.05–0.20/1k exemplos** + juízes. 100k CADOCs sintéticos ≈ **$20–80** de API. O custo real é eng (harness) e storage, não tokens.
- **Honesto sobre self-improvement:** loops de "melhora com o uso" funcionam quando há verificador objetivo e o delta é dado de treino rotulado (rejection sampling + distillation); não funcionam como "o modelo se auto-avalia e melhora sozinho" (degradação por drift, verifier-gaming). Implementar: métrica fixa por versão de catálogo + replay de erros reais das IFs (opt-in) como corpus de regressão, não auto-reescrita de prompt/modelo.

---

## 7. Fine-tuning vs RAG + parsing determinístico

| Abordagem | Custo | Risco | Quando |
|---|---|---|---|
| RAG + engine determinístico | ~$0 incremental | baixo | **default** — regras versionadas mudam; RAG atualiza em segundos |
| LoRA (estilo/formato de explainability) | $50–500/run QLoRA (Unsloth; ~$5 de GPU bruta p/ 7–8B) | médio-baixo | quando tom/formato do texto ao cliente precisa ser consistente |
| LoRA "aprender regras BACEN" | $5k–30k (distill+treino) | **alto** — overfit em versão do normativo; rollback difícil; auditoria não confia em peso opaco | **nunca para regras versionadas** |

Consenso 2026 (Winder.AI, BigData Boutique): comece RAG; LoRA/QLoRA apenas para comportamento/formato, não conhecimento mutável; full fine-tune raramente justificado. Regras regulatórias são o pior caso para fine-tune (mudam toda semana via Radar, precisam de proveniência auditável por regra). A arquitetura híbrida certa: **determinismo no engine + RAG no conhecimento + LoRA opcional no estilo** + distillation só para destilar o *formato de saída* do copiloto num modelo on-prem menor (GPT-OSS-120B já é CoT-treinável, aceita LoRA leve).

---

## 8. Arquitetura recomendada (ASCII)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          RADIAN T NORMA — ENGINE IA                          │
│                                                                             │
│  BACEN (DOU/cartas/XLSX)                                                    │
│      │                                                                      │
│      ▼                                                                      │
│  ┌──────────────┐   ┌──────────────────────────────────────────────────┐    │
│  │ RADAR        │──►│ INGESTÃO                                          │   │
│  │ diff §/hash  │   │ Docling (PDF) │ calamine (XLSX, Rust)            │   │
│  └──────────────┘   └───────┬──────────────────────────────────────────┘    │
│                             ▼                                               │
│   ┌─────────────────────────────────────────────┐  ┌─────────────────────┐  │
│   │ PIPELINE REGRA→CÓDIGO (agentic + verificação)│  │ RAG NORMATIVO       │  │
│   │ DRAFT  Gemini 3 Pro (visão, 1M ctx)          │  │ pgvector (HNSW)     │  │
│   │ CRITIC Claude Sonnet 4.5 (diff vs §)         │  │ + Tantivy BM25 (RRF)│  │
│   │ TEST   engine Rust (XSD + golden fixtures)   │  │ + BGE-Reranker v2-M3│  │
│   │ HUMAN  fila de aprovação (UI diff)           │  │ emb: Qwen3-Emb 4B   │  │
│   └──────────────┬──────────────────────────────┘  └─────────────────────┘  │
│                  ▼                                                          │
│   ┌──────────────────────────────────────────────────────────────────┐      │
│   │ CATÁLOGO (Postgres) — 1.099+ regras versionadas, proveniência §  │      │
│   └──────────────────────────────────────────────────────────────────┘      │
│                  │                                                          │
│                  ▼                                                          │
│   ┌──────────────────────────────────────────────────────────────────┐      │
│   │ MOTOR (Go→Rust) — execução determinística L1–L4                  │      │
│   └──────────────────────────────────────────────────────────────────┘      │
│         │                        │                       │                  │
│         ▼                        ▼                       ▼                  │
│   COPILOTO LLM            HARNESS SINTÉTICO         AGENTES/MCP (rmcp)      │
│   (Sonnet cloud OU        challenger(Flash)→        tools read-only scoped  │
│    GPT-OSS-120B on-prem)  solver(Rust/XSD)→        HITL p/ envio BACEN     │
│   sanitização PII antes   judge(XSD det.+LLM 10%)   guardrails anti-inject  │
│   da cloud; opt-in        corpus rotulado ~$0.1/1k                          │
└─────────────────────────────────────────────────────────────────────────────┘
        │ router de modelo: dado público → frontier; PII → on-prem
        ▼
   { Gemini 3 Pro/Flash · Claude Sonnet/Opus · GPT-5.1 · GPT-OSS-120B (vLLM/SGLang, H100 SP) }
```

---

## 9. TCO mensal estimado (infra IA)

Premissas: validação com LLM em 10% dos eventos (opt-in/copiloto), ~25k tokens de contexto de regras por chamada (cache hit 80%), ~1.500 tokens de saída; extração = ~40 normativos/mês; RAG = 3 consultas por validação.

### Cenário A — 10 IFs (≈3.000 chamadas LLM/mês)

| Item | Escolha cloud | Custo | Escolha on-prem | Custo |
|---|---|---|---|---|
| Copiloto/explain | Gemini 3 Flash (cache) | **~$90** | GPT-OSS-120B @ L40S OCI | $2.555 |
| Casos duros (10%) | Sonnet 4.5 | ~$120 | — | — |
| Extração regras | Gemini 3 Pro + Sonnet critic | ~$200/mês | idem (frontier) | $200 |
| Embeddings | Qwen3-Embed self | $0 | self | $0 |
| Rerank | BGE self | $0 | self | $0 |
| RAGAS/eval LLM judge | Flash batch | ~$15 | self | $0 |
| Harness sintético | Flash (500k exemplos/mês) | ~$60 | idem | $60 |
| Parsing | Docling CPU | $20 (compute) | idem | $20 |
| Guardrails/logs | self | $0 | self | $0 |
| **Total** | | **≈ $505/mês** | | **≈ $2.835/mês** |

### Cenário B — 100 IFs (≈30.000 chamadas LLM/mês + agêntico)

| Item | Escolha cloud | Custo | Escolha on-prem (híbrido) | Custo |
|---|---|---|---|---|
| Copiloto/explain | Flash + Sonnet nos 10% | **~$1.900** | GPT-OSS-120B @ H100 OCI | $7.300 |
| Extração regras | frontier | ~$400 | frontier | $400 |
| Harness (2M exemplos/mês) | Flash batch | ~$250 | on-prem | ~$0 marginal |
| Eval/observabilidade (RAGAS+judges) | batch | ~$150 | self | $0 |
| Embeddings (30M tokens/mês) | Gemini Emb 2 | $6 | self | $0 |
| Parsing (Docling 3k págs + OCR 500) | | ~$25 | | $25 |
| MCP conectores/agentes runtime | self (Rust) | $80 infra | | $80 |
| **Total** | | **≈ $2.800/mês** | | **≈ $7.800/mês** |

Leitura: **até ~100 IFs, cloud híbrido com Gemini Flash/Sonnet custa menos de $3k/mês** — a infra GPU on-prem só compensa quando (a) cliente exige PII never-leave, (b) volume de insights não-opt-in explode, ou (c) batch pesado de sintético. A linha de cruzamento fica entre 100–300 IFs ou 1 clientes enterprise "air-gapped" que paguem o H100 sozinhos.

Recomendação de rollout: começar 100% API (modelo router + sanitizador PII já no MVP — custa ~1 sprint e desbloqueia os dois mundos), adicionar nó L40S on-prem (Qwen3-A3B + embeddings + rerank) quando o primeiro cliente exigir residência, migrar copiloto para GPT-OSS-120B/H100 quando >50 IFs ou 2 clientes air-gapped.

---

## 10. Riscos e decisões abertas

1. **Promo pricing:** Gemini Flash $0.50 e 3.1 Pro $1 são introdutórios (revisão dez/2026–jan/2027); arquitetura precisa de router multi-modelo (já previsto) para trocar em 1 dia.
2. **Residência de dados LatAm inexistente** nos 3 frontier providers — sanitização é obrigatória, não opcional (ver §3.5). Monitorar Google Cloud SP (Blackwell out/26) para residência contratual futura.
3. **DeepSeek/Qwen via API de terceiros:** evitável para dados de cliente (jurisdição opaca); usar só para normativos públicos ou self-host.
4. **Dependência de catálogo humano:** mesmo com pipeline automático, manter SLA de revisão humana — é o produto (audit trail), não um custo.
5. **rmcp 2.x jovem** (CVE recente) — fixar versão, conformance tests no CI.

---

## 11. Fontes (verificadas nov/2026)

**Extração norma→regra / mercado**
- Norm AI: https://www.norm.ai · Parceria Microsoft: https://www.prnewswire.com/news-releases/norm-ai-and-microsoft-legal--compliance-ai-built-for-the-way-work-actually-happens-302692324.html · https://fintech.global/2026/02/25/how-norm-ai-is-embedding-legal-intelligence-into-microsoft-365
- Regnology: https://www.regnology.net/en/solutions/regnology-platform/rgi · https://www.regnology.net/en/resources/insights/the-agentic-gap-report · https://www.regnology.net/en/resources/insights/chartis-regulatory-risk-reporting-agentic-ai
- OECD Rules as Code (contexto), Blawx, Theory&Principle/Factor — via busca "regulation as code" 2026
- LegalRuleML (OASIS): https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/legalruleml-core-spec-v1.0.html · D2 reporting obligations (EU): interoperable-europe.ec.europa.eu (PDF "A LegalRuleML specialisation")
- Deôntica+LLM: https://ceur-ws.org/Vol-4174/paper7.pdf · https://dl.acm.org/doi/full/10.1145/3769126.3769260 · PROLEG/HKU: https://www.lawtech.hku.hk/event/neuro-symbolic-legal-reasoning-reasoning-by-proleg-rule-and-fact-extraction-by-llm
- Structured outputs: https://mlflow.org/articles/structured-outputs-llm · https://www.gmicloud.ai/en/blog/structured-output-for-llm-inference-json-schema-enforcement-constrained-decoding-and-parse-failure-rates · https://arxiv.org/html/2501.10868v1

**Modelos & preços**
- Gemini: https://aistudio.google.com/docs/pricing · https://www.spheron.network/blog/gemini-3-pro-api-pricing-vs-self-hosted-llms-cost-privacy-2026
- OpenAI: https://developers.openai.com/api/docs/pricing · https://openrouter.ai/openai/gpt-5.1
- Anthropic: https://www.anthropic.com/pricing
- Qwen/DeepSeek: https://benchlm.ai/alibaba/api-pricing · https://deepinfra.com/blog/qwen-api-pricing-2026-guide · https://openrouter.ai/deepseek/deepseek-v3.2 · https://getdeploying.com/llms/deepseek-v3.2
- Mistral: https://docs.mistral.ai/inference/pricing · https://mistral.ai/pricing
- Embeddings: https://milvus.io/blog/choose-embedding-model-rag-2026.md · https://wz-it.com/en/blog/best-embedding-models-german
- Rerankers: https://cohere.com/rerank · https://mixpeek.com/curated-lists/best-rerankers · https://presenc.ai/research/best-open-weight-reranker-models-2026 · https://ianas.fr/en/blog/2026/06/07/reranker-comparatif-cohere-bge-jina-voyage

**Inferência & hardware**
- vLLM/SGLang: https://tensormesh.ai/learn/vllm-vs-sglang · https://www.spheron.network/blog/vllm-vs-sglang-2026 · https://developers.redhat.com/articles/2026/10/14/vllm-vs-sglang-choosing-the-right-inference-engine-for-your-workload · https://jarvislabs.ai/blog/vllm-sglang-trtllm-comparison · https://www.yottalabs.ai/post/vllm-vs-sglang-which-inference-engine-should-you-use-in-2026
- Qwen3-A3B throughput: https://recipes.vllm.ai/Qwen/Qwen3-30B-A3B · https://arxiv.org/html/2512.23029v1 · https://qwen.readthedocs.io/en/latest/getting_started/speed_benchmark.html
- GPT-OSS-120B hardware: https://openai.com/index/introducing-gpt-oss · https://www.yottalabs.ai/post/gpt-oss-120b-hardware-requirements-gpu-memory-2026 · https://www.llamabuilds.ai/models/gpt-oss-120b/requirements
- AWS sa-east-1: https://instances.vantage.sh/aws/ec2/g6e.xlarge · https://aws.amazon.com/ec2/capacityblocks/pricing · https://gpusmith.com/articles/en/aws-gpu-instance-pricing
- OCI: https://www.oracle.com/cloud/price-list/ (H100 $10/h, L40S ~$3.5/h uniformes) · https://gpufinder.dev · https://computeprices.com
- Hetzner: https://www.hetzner.com/dedicated-rootserver/matrix-gpu · https://northflank.com/blog/hetzner-cloud-server-price-increases
- Mac/DGX/5090: AppleInsider (M4 Max 128GB $3.499; M5 Ultra $5.499/$9.499, set/26) · NVIDIA RTX 5090 MSRP $1.999–2.199

**RAG**
- Vetores: https://encore.dev/articles/best-vector-databases · https://vdf.ai/blog/best-vector-database-for-rag · https://aimultiple.com/open-source-vector-databases
- Parsing: https://agentscamp.com/guides/comparisons/best-document-parsers-for-rag-2026 · https://llms.reducto.ai/table-extraction-accuracy-scanned-pdfs · https://mixpeek.com/curated-lists/best-pdf-extraction-tools
- Eval: https://docs.ragas.io/en/stable/concepts/metrics/available_metrics/faithfulness · https://deepeval.com/blog/deepeval-alternatives-compared · https://qaskills.sh/blog/deepeval-vs-ragas-llm-evaluation-2026

**MCP & segurança**
- SDK Rust oficial: https://github.com/modelcontextprotocol/rust-sdk · spec 2026-07-28: https://blog.modelcontextprotocol.io/posts/2026-07-28 · https://modelcontextprotocol.io/docs/2026-07-28/sdk
- Segurança: https://unit42.paloaltonetworks.com/model-context-protocol-attack-vectors · https://developer.microsoft.com/blog/protecting-against-indirect-injection-attacks-mcp · https://socprime.com/blog/mcp-security-risks-and-mitigations · https://www.redhat.com/en/blog/ai-security-defending-against-prompt-injection-and-unsafe-actions

**Sintético & fine-tuning**
- Distillation: https://arxiv.org/html/2410.18588v1 ($0.36–8.74/1k) · https://rlhfbook.com/c/12-synthetic-data · https://arxiv.org/html/2605.26132v1 · https://www.honeyhive.ai/learn/synthetic-data-generation · https://machinelearning.apple.com/research/environment-free · https://tianpan.co/blog/2026/04/09/knowledge-distillation-economics-production-ai
- FT vs RAG: https://winder.ai/rag-vs-fine-tuning-2026-decision-framework · https://bigdataboutique.com/blog/fine-tuning-llms-when-rag-isnt-enough · https://pub.towardsai.net/fine-tuning-llms-in-2026-lora-qlora-unsloth-and-everything-in-between-929eaf94aea2

**LGPD**
- https://www.promptquorum.com/local-llms/local-llm-lgpd-compliance-brazil-2026 (FGV) · https://www.trade.gov/market-intelligence/brazils-new-rules-international-data-transfers (ANPD) · https://aws.amazon.com/compliance/brazil-data-privacy · https://www.activepieces.com/blog/how-to-choose-ai-governance-tools-for-latam-compliance · OpenAI residency: https://developers.openai.com/api/docs/guides/data-residency (sem Brasil) · Vertex/Claude multi-region US/EU: anúncio abr/2026 · GCP Blackwell SP: anúncio set/2026
