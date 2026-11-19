# 26 — SYSTEM ONE (Decision Model): Arquitetura Cognitiva Intermediária (`System 0 → System 1 → System 2`)

> **Versão:** 3.0 · **Data-base:** 19/11/2026 · **Status:** Especificação Arquitetural (Plano v3.0 — sem modificação de código até aprovação das etapas)  
> **Documentos irmãos:** [`21-ENGINE-IA.md`](21-ENGINE-IA.md) (Engine IA, Open-Weights 2026, Dual Cloud/Local & Serving) · [`20-ARQUITETURA-RUST.md`](20-ARQUITETURA-RUST.md) (`System 0`, workspace & zero-float) · [`22-HARNESS-V4.md`](22-HARNESS-V4.md) (`M1–M10` & Honesty Engine) · [`24-UX-VISAO.md`](24-UX-VISAO.md) (Wizard & HITL 1-clique) · [`25-ROADMAP-EXECUCAO.md`](25-ROADMAP-EXECUCAO.md) (Plano v3.0 Etapas 0–4)

---

## 0. Por Que o System One Existe (Diagnóstico da Lacuna)

O blueprint de 16/11/2026 ([`21-ENGINE-IA.md`](21-ENGINE-IA.md)) estabeleceu corretamente o **determinismo no núcleo** (`System 0`) e o uso de **LLMs generativos na fronteira** (`System 2`), mas deixou um vazio estrutural entre as duas camadas:

1. **Salto caro e lento:** toda ambiguidade de mapeamento de planilhas (`ingest/mapping.rs`), triagem de dúvidas simples, detecção de anomalias numéricas pós-geração e filtragem do Radar normativo caía diretamente em heurística fixa manual ou em chamadas autoregressivas lentas e caras de LLM (`System 2`).
2. **Alvo ausente para `M8` (Destilação):** [`22-HARNESS-V4.md`](22-HARNESS-V4.md) previu destilação por gatilhos quantitativos (`M8`), mas não especificava **qual camada de modelos leves locais** absorve esse aprendizado contínuo sem jamais alterar pesos para "memorizar" regras legais.
3. **Conflito de compilação no workspace Rust:** [`rust/Cargo.toml`](../../rust/Cargo.toml) proíbe `unsafe_code` e nega `float_arithmetic`, `float_cmp`, `cast_precision_loss`, `unwrap_used`, `expect_used` e `wildcard_enum_match_arm` — invariante vigiada em CI pelo teste estrutural `floats_banidos_do_workspace` em [`layers.rs`](../../rust/crates/reg-cli/tests/layers.rs). Redes neurais e embeddings operam em `f32`; o domínio contábil-regulatório **jamais** pode admitir floats.
4. **Risco de atraso ao 3040 e confusão probabilística:** modelos locais pesados ou runtimes C++/GPU (`ort`, `mistral.rs`) acoplados ao núcleo quebrariam builds determinísticos (`--offline=true`, `wasm32`) e atrasariam a entrega do motor 3040.

O **System One** resolve esse conjunto de restrições atuando como uma camada intermediária de **Decision Models rápidos, locais por padrão, estatisticamente calibrados, zero-float na fronteira e estritamente isolados do poder legal do `System 0`**.

---

## 1. Arquitetura em 3 Camadas: `System 0 → System 1 → System 2`

```text
┌───────────────────────────────────────────────────────────────────────────────┐
│ SYSTEM 0 — MOTOR DETERMINÍSTICO RUST (Autoridade Legal, Contábil & Cripto)   │
│ Crates atuais: `reg-core`, `reg-cadoc`, `reg-ledger`, `reg-cli`               │
│ Novos (Etapas 1–2): `reg-validate-core` (`no_fs`, `wasm32`), `reg-sta`        │
│ • Domínio exato: `Centavos(i128)`, `TaxaDecimal`, `CnpjIdentifier` (IN 729),  │
│   `Ipoc`, `DataBase`, `Vigencia`                                              │
│ • `gen3040` / `gen2030`, regras L1–L4, `BacenValidatorAdapter` (`.jar` BCB)   │
│ • `reg-ledger`: `MappingLock` 4-olhos, JCS RFC 8785, Ed25519 `DualApproval`   │
│ INVARIANTES: `unsafe_code="forbid"`, `float_arithmetic="deny"`, zero pesos ML │
└────────────────────────────────▲───────────────────────────────┬──────────────┘
                                 │ (1) Só `Proposal<T>` /        │ (2) Veredictos,
                                 │     `AdvisorySignal` (jamais  │     schemas e
                                 │     altera veredito ou XML)   │     agregados exatos
                                 │                               ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│ SYSTEM 1 — DECISION MODELS IN-PROCESS (Local-First · Zero Egress · $0/token)  │
│ • `reg-s1-tabular` (Rust puro `i64` ponto fixo, zero float, `wasm32`-ready)   │
│ • `reg-ai-runtime` (folha isolada `ort` ONNX INT8 — único `#[allow(float_*)]`) │
│ Submodelos:                                                                   │
│  - S1.1 Column Auto-Mapper (léxico + parsers `reg-core` + Emb + GBDT `i64`)   │
│  - S1.2 Defense-in-Depth Scrubber & Injection Gate (Allowlist + Aho-Corasick  │
│         + `debertinha` + `moBERTo` NER + canários)                            │
│  - S1.3 Portfolio Sanity & Anomaly Scorer (GBDT `i64` + `TreeSHAP` em bps)    │
│  - S1.4 Triage & Escalation Router (`DeB3RTa` + sinais BM25/RRF do RAG)       │
│ FRONTEIRA: zero `f32`/`f64` → só sai `ScoreBps(u16)`, `i32`, `Vec<i16>` Q15   │
└───────────────┬───────────────────────────────────────────────┬───────────────┘
                │ `RestrictedPiiOrUnmasked` / `AirGapped` /     │ `PublicRegulatory` ou
                │ texto livre de usuário/ERP / falha no S1.2    │ `SchemaOnlyTokenized`
                │ (FAIL-CLOSED: nunca sai para nuvem externa)   │ (só Modo B/C + ZDR+SCC)
                ▼                                               ▼
┌───────────────────────────────────┐       ┌───────────────────────────────────┐
│ SYSTEM 2-LOCAL (Open-Weights 2026)│       │ SYSTEM 2-CLOUD (Frontier ZDR+DPA) │
│ • `mistral.rs` / `llama.cpp` /    │       │ • `Gemini 3.x Pro / Flash`        │
│   `SGLang` (`RadixAttn`) / `vLLM` │       │ • `Claude Sonnet / Opus 4.5+`     │
│ • `Qwen3.5/3.6`, `Gemma 4`,       │       │ • `GPT-5.1+`                      │
│   `Mistral Small 3.2`, `gpt-oss`  │       │ • REGRA LC 105: ZERO texto livre  │
│ • Emb/Rerank: `BGE-M3` / `Qwen3`  │       │   ou dados reais de clientes      │
└───────────────────────────────┬───┘       └───────────────────┬───────────────┘
                                └───────────────┬───────────────┘
                                                ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│ PORTÃO DE VERIFICAÇÃO OBJETIVA & AUDITORIA (`reg-ai` + `reg-ledger`)          │
│ • `jsonschema` Draft 2020-12 + parse estrito `reg-core` (`Centavos`/`TaxaDec`)│
│ • Provenance obrigatória (`rule_code + version`, `§` normativo, condição)     │
│ • Output Guardrail: proíbe afirmar "conforme" se houver `E`/crítico no report │
│ • `RuleSpec` candidato só entra após compilar + passar em fixtures/XSD        │
└───────────────────────────────────────────────────────────────────────────────┘
```

### Fronteiras e Garantias Comparadas

| Dimensão | **System 0** (`reg-core/cadoc/ledger`) | **System 1** (`reg-s1-tabular` / `reg-ai-runtime`) | **System 2** (`LlmTransport` Local/Cloud) |
|---|---|---|---|
| **Latência p95** | `<1–50 ms` (stream XML) / seg (`.jar`) | **`<2 ms` (`i64`) · `5–120 ms` (ONNX CPU)** | `300 ms–10 s+` (GPU local / API) |
| **Custo marginal** | `R$ 0` (CPU local) | **`R$ 0` por inferência** (CPU / RAM / iGPU) | Tokens GPU ou API (`$/Mtok`) |
| **Tipo de saída** | `Verdict`, `GeneratedDoc`, `ValidatedZip` | `Proposal<T>`, `ScoreBps(u16)`, `EgressDecision`, `TreeShapDriver` | JSON estruturado (`RuleDraft`, `FixSuggestion`) / SSE |
| **Acesso a dado cru (LC 105)** | Integral (in-perimeter) | Integral (in-perimeter, zero egress) | Local VPC: permitido pós-mask · Cloud externa: **ZERO dado/texto livre** |
| **Autoridade sobre veredito legal** | **100% (Autoridade única)** | **0%** (só sugere rascunho ou emite *advisory*/*hold gate* aditivo) | **0%** (jamais escreve XML de prod ou rebaixa erro `E/A/I`) |
| **Conformidade zero-float** | 100% `forbid(unsafe)` + `deny(float_*)` | **Zero float na fronteira** (`i64` puro + `ScoreBps(u16)` + `Q15`) | JSON com todos os números decimais/CNPJs como `string` |
| **Modo offline / `--no-ai`** | Sempre funciona | Compatível `--offline=true`; desligável via `--no-ai` | Opcional / desativado com `--no-ai` |

---

## 2. Isolamento no Workspace Rust & Lints Zero-Float

Para cumprir [`layers.rs`](../../rust/crates/reg-cli/tests/layers.rs) e preservar [`rust/Cargo.toml`](../../rust/Cargo.toml) intacto:

1. **Núcleo Sagrado Intocado:** `reg-core`, `reg-cadoc`, `reg-validate-core` (`no_fs`, `wasm32`), `reg-ledger` e `reg-sta` **nunca** importam `reg-ai`, `reg-ai-runtime`, `ort` ou `mistral.rs`. Permanecem 100% `unsafe_code = "forbid"` e zero-float.
2. **Padrão A — Ponto Fixo Inteiro (`reg-s1-tabular`, 100% zero-float & `wasm32`-ready):**
   - Árvores GBDT (`LightGBM` / `CatBoost`), regressões logísticas e calibradores isotônicos são treinados offline e compilados para estruturas Rust puras em **`i64`**.
   - Thresholds de decisão são escalados em `10^4` ou `Centavos(i128)`; pesos de folha em micro-logits `i64`; função logística por tabela de consulta (*LUT*) inteira retornando `ScoreBps(u16)` (`0..=10_000`).
   - **Aritmética matemática sem float para `S1.3` / `S1.4`:**
     - `log2_ratio`: calculado via `u128::ilog2()` (parte inteira) + LUT fixa de 256 entradas na mantissa em `i64` escalado `10_000`.
     - **Divergência Jensen-Shannon:** computada sobre probabilidades empíricas discretizadas em basis points (`10_000` soma exata) + LUT inteira `log2_bps`.
     - **Similaridade de cosseno:** vetores normalizados vindos de `reg-ai-runtime` são quantizados na borda para `Vec<i16>` (`Q15`, `-32767..=32767`); o produto escalar em `reg-s1-tabular`/`reg-ai` acumula em `i64` com *bit-shift* `(acc >> 20)`, produzindo `ScoreBps(u16)` bit-a-bit determinístico entre x86, ARM e WASM.
3. **Padrão B — Folha Feature-Flagged (`reg-ai-runtime` ou Sidecar UDS):**
   - Única crate (ou binário sidecar separado via Unix Domain Socket / HTTP loopback) onde `#[allow(clippy::float_arithmetic)]` é permitido estritamente ao redor dos tensores `f32` de `ort` (ONNX Runtime INT8) e opcional `mistral.rs`.
   - Isolada atrás das feature flags `--features ai-onnx` e `--features ai-local-llm`. Compilar o sistema com `--no-default-features` gera o binário determinístico sem nenhuma dependência C++, CUDA ou Metal — eliminando qualquer conflito com pacotes Linux (`debian-slim` / `ubuntu-latest`) ou `windows-latest`.

### Contratos de Tipos na Fronteira (`reg-s1-tabular` / `reg-ai`)

```rust
/// Probabilidade calibrada em basis points: 0..=10_000 (10_000 = 100.00%).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ScoreBps(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceBand {
    Verde,     // >= tau_high_bps (ex: >= 9200 bps) E margem >= tau_margin_bps
    Cinzenta,  // tau_low_bps..tau_high_bps (ex: 5000..9199 bps) ou colisão top-2
    Vermelha,  // < tau_low_bps ou conflito OOD / Conformal
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EgressDecision {
    RouteOnPremOnly,   // Política AirGapped, texto livre c/ risco LC 105, falha ONNX ou canary > 0
    ClearedSchemaZdr,  // Exclusivo p/ PublicRegulatory ou SchemaOnlyTokenized sob ZDR + SCC ANPD
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeShapDriver {
    pub feature: String,
    pub impact_bps: i32,          // contribuição Shapley em basis points (+/-)
    pub observed: String,         // grandeza exata formatada (zero float JSON)
    pub baseline_median: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal<T> {
    pub payload: T,
    pub score_bps: ScoreBps,
    pub margin_top2_bps: u16,
    pub band: ConfidenceBand,
    pub drivers: Vec<TreeShapDriver>,
    pub model_sha256: String,
    pub calibrator_sha256: String,
    pub policy_sha256: String,
}
```

---

## 3. Especificação dos 4 Submodelos (`S1.1` a `S1.4`)

### `S1.1` — Column Auto-Mapper (`ingest` → `Proposal<MappingDraft>` → `MappingLock<Rascunho>`)

**Problema real no repo:** [`ingest/mapping.rs`](../../rust/crates/reg-cadoc/src/ingest/mapping.rs) exige hoje um `mapping.toml` manual sob travas estritas (`MONETARY_FIELDS`), e o passo *Mapear* do Wizard ([`24-UX-VISAO.md`](24-UX-VISAO.md)) exige confiança calibrada por campo e zero defaults silenciosos.

#### Pipeline em Cascata com *Early Exit* (`<15 ms` fast-path · `80–120 ms` batch CPU completo p/ 100 colunas)
1. **Estágio Determinístico Léxico (`0.2 ms`):** normalização idêntica a [`xlsx.rs`](../../rust/crates/reg-cadoc/src/ingest/xlsx.rs) (`StripAccents + ToLower + trim`), distância Jaro-Winkler + Jaccard de n-gramas contra o dicionário oficial extraído de [`3040/SCR3040_Leiaute.xls`](../../3040/SCR3040_Leiaute.xls) e sinônimos comuns de ERPs/FIDCs brasileiros (`num_ccb`, `sacado_cnpj`, `pdd_atual`, `saldo_devedor_mil`).
2. **Estágio de Perfil pelos Parsers do `reg-core` (`1 ms` sobre amostra `N = 200` linhas via `calamine`):**
   - `Ipoc::new` (`15..=67`, `[0-9A-Z-]`) → separa `ipoc_raw` de `contrt` genérico;
   - `CnpjIdentifier::new` (`8 | 14`, numérico ou alfanumérico IN 729/2026) → distingue `if_cnpj`, `det_cli`, `cli.cd`;
   - `DataBase::parse` e padrões ISO/BR → distingue `data_base`, `dt_contr`, `dt_venc_op`, `ini_relact_cli`;
   - `Centavos::parse` + quantis inteiros de magnitude e casas decimais (`p10/p50/p99`) → distingue `Reais` vs `Centavos` vs `MilReais` vs `TaxaDecimal` (`N11,7`);
   - Cardinalidade e aderência aos Anexos BCB (ex.: Anexo 41 `{1,2,3}` para `@EstInstFin`, `{AA..H}` para `@ClassOp`, códigos 4 dígitos de `mod_` e `natu_op`).
3. **Bi-Encoder Semântico com *Early Exit* (`ort` INT8):** colunas resolvidas com certeza nos Estágios 1+2 não executam o modelo neural. Apenas o subconjunto ambíguo residual (tipicamente 5–25 headers curtos `<= 16` tokens) roda em **1 único forward batch ONNX INT8** (`BGE-M3` ou `Qwen3-Embedding-0.6B`), comparado via produto escalar `Q15` (`i64`) contra os ~200 cartões de campo pré-computados.
4. **Stacker Calibrado (`reg-s1-tabular` LightGBM `i64` + Calibração Isotônica por CADOC):** ranqueia pares `(coluna_origem, target_field)` e sugere `MonetaryUnit` + `Vec<Transform>`.

#### Política de Decisão (`S1.1`)
| Faixa | Condição | Ação | Trava Não-Negociável |
|---|---|---|---|
| **Verde** | `score_bps >= 9200` **E** `margin_top2_bps >= 1500` | Pré-preenche `MappingLock<Rascunho>` (`mapping.toml.draft`) com badge calibrada `S1` | **Nunca aplica em silêncio:** exige confirmação humana (1 clique) e passa obrigatoriamente pelas 4 travas de [`mapping_locks.rs`](../../rust/crates/reg-ledger/src/mapping_locks.rs) (`UnidadePorCampo`, `ConservacaoZero`, `QuatroOlhos`, `CustodiaSha256`). |
| **Cinzenta** | `5000..9199 bps` ou colisão (`DtContr` vs `DtVencOp`, `Reais` vs `MilReais`) | Escala ao `S2-Fast` enviando **somente** nomes de colunas que passaram em higienização léxica (zero dígitos ≥ 5 / zero CNPJ / zero nome próprio) + estatísticas sintéticas de formato | Sugestão marcada `"S2-assistido"`, exige revisão e aprovação humana. Header com suspeita de dado identificável vai apenas para `S2` local. |
| **Vermelha** | `< 5000 bps` ou conflito OOD/unidade | **Abstenção obrigatória (`NeedsHumanMapping`)** | Bloqueia avanço até mapeamento humano explícito. |

---

### `S1.2` — Defense-in-Depth Scrubber, Anti-Injection & Egress Gate (`Fail-Closed`)

**Correção Crítica Pós-Auditoria (LC 105/2001 art. 10 & Res. CD/ANPD nº 29/2024):**  
Modelos neurais NER (mesmo com `Recall >= 99.5%`) possuem taxa residual de falso-negativo probabilístico (`<= 0.5%` FNR). Sob o sigilo bancário da **LC 105/2001**, **é inaceitável depender apenas de um classificador probabilístico para liberar texto livre de cliente a uma API cloud estrangeira**, e canários sintéticos não provam recall zero-erro em nomes ou CNPJs inéditos.

Portanto, o `S1.2` impõe **barreira estrutural determinística no egress externo + rede neural como defesa em profundidade interna**:

1. **Estágio 0 — Allowlist Estrutural Determinística de Schema (`<0.2 ms` — Barreira Primária de Egress Cloud):**
   - Classifica como `ClearedSchemaZdr` **exclusivamente**:
     * `PublicRegulatory`: normativos BCB, DOU, XSDs, leiautes públicos, textos de regras e perfis sintéticos `M4`.
     * `SchemaOnlyTokenized`: tuplas fixas e fechadas de catálogo (`{cadoc, catalog_version, rule_code, field_path, error_class}`) — exatamente o que o navegador WASM envia ao `reg-server` após validar localmente — mais campos estruturados de `Doc3040Input` reduzidos por allowlist estrita de caminho (`IPOC → token HMAC efêmero`, `CPF/CNPJ → tipo_pessoa + validade_dv`, `DtContr → delta_dias`, saldos → ordem de grandeza).
   - **Todo texto livre originado de usuário ou ERP** (perguntas abertas no chat citando tomadores/operações, abas de observação de planilha, mensagens cruas de retorno CRD, campos textuais fora da allowlist) é marcado **determinísticamente** como `RestrictedPiiOrUnmasked` e roteado **exclusivamente On-Prem / VPC Brasileira (`RouteOnPremOnly`)** nos Modos B/C.
2. **Estágio 1 — Autômato Dinâmico `Aho-Corasick` + Parsers `reg-core` (`<2 ms`):**
   - Indexa em memória todas as entidades conhecidas da remessa corrente (`Cli.cd`, `nome_cli`, `contrt`, `ipoc_raw`, `nome_resp`) + detectores estruturais CPF Mod-11, CNPJ numérico & alfanumérico IN 729/2026, IPOC, PIX, e-mail e telefone para mascarar com pseudônimos HMAC antes de logs, cache ou processamento interno.
3. **Estágio 2 — Guardrail Neural Local (`ort` INT8 — Defesa em Profundidade):**
   - **`sagui-nlp/debertinha-ptbr-xsmall`** (~40M params, **MIT**, `5–15 ms` CPU em turnos curtos de chat/MCP): detecção de *prompt injection* e classificação de risco residual em prompts externos/MCP.
   - **`Tropic-AI/moBERTo`** (~150M params, ModernBERT-PT, **Apache 2.0**, 8K ctx, `50–120 ms` CPU ou `<15 ms` GPU/ANE): NER PT-BR (`PESSOA`, `ORG_TOMADOR`, `DOC_FISCAL`, `CONTRATO_CREDITO`, `CONTA_BANCARIA`, `ENDERECO`, `CONTATO`, limiar *Recall-First* `tau_pii_bps = 2500`) usado como **rede secundária obrigatória** para higienizar logs `tracing`/Sentry, histórico de conversa e fila de curadoria **`M1/M2/M7`**.
4. **Estágio 3 — Canários Contínuos & Fail-Closed:**
   - Injeta tokens canários sintéticos nos pipelines de batch (`M1/M2/M7`); se **`canary_fnr > 0`**, houver erro no runtime ONNX ou timeout, dispara **`PiiSanitizerTrip`**, invalida o lote de telemetria, aborta o envio e trava em `RouteOnPremOnly`.

---

### `S1.3` — Portfolio Sanity & Anomaly Scorer (`<20 ms`, Zero-LLM, Aditivo Pré-STA)

**Por que o System 0 não pega isso sozinho:** um XML 3040 pode ser 100% válido nas regras L1–L4 e aprovado pelo `.jar` do BCB, mas carregar um desastre econômico-operacional silencioso:
- Escala trocada por `1.000×` (`MilReais` importado como `Reais`);
- Queda abrupta de `ProvConsttd` (PDD) numa carteira estável ou incompatibilidade entre estágios CMN 4.966 (`est_inst_fin ∈ {1,2,3}`);
- Explosão atípica nos vencidos (`v205..v290`) ou baixas em prejuízo (`v310..v330`);
- Descolamento repentino de mix de modalidades (`mod_`) ou concentração HHI Top-10 tomadores.

#### Execução em Ponto Fixo `i64` (`reg-core` + `reg-s1-tabular`, zero float)
- Agrega `Doc3040Input` em `Centavos(i128)` e `TaxaDecimal`:
  1. **Volume & Escala:** `log2_ratio` (`u128::ilog2` + LUT 256 entradas) do saldo total (`sum(v20..v330)`), `vlr_contr` e `fat_anual` p50 vs mediana e MAD (*Median Absolute Deviation*) das janelas `t-1, t-3, t-6, t-12` do próprio tenant.
  2. **Matriz de Provisão (CMN 4.966):** `prov_consttd / saldo` (bps) por `class_op` e estágio (`1/2/3`), comparando `dias_atraso > 90` vs `% Estágio 3`.
  3. **Perfil de Atraso & Baixa:** razão vencidos/prejuízo vs carteira ativa e variação p50/p99 de atraso.
  4. **Concentração & Mix:** divergência **Jensen-Shannon em `i64`** (`10_000` bps + LUT `log2_bps`) do mix `mod_` + delta de HHI Top-10 e taxa de campos condicionais preenchidos.
- **Modelos:**
  - *Regime Maduro (`>= 3` competências L4):* `LightGBM`/`CatBoost` compilado em árvore `i64` + Robust MAD Z-Score.
  - *Regime Cold-Start (`0..2` competências):* Priors hierárquicos Bayesianos/Ridge em `i64` treinados em corpus sintético `M4` + coortes anonimizadas (`k >= 5` IFs de mesmo porte/segmento: SCD/IP/FIDC/Banco) — substituindo `TabPFN` em produção por restrição não-comercial dos pesos.
- **Explicabilidade `TreeSHAP` em `i32` bps (`<2 ms`):** computa no próprio avaliador Rust os Top-3 drivers em português e grandezas reais (ex.: `[+4.820 bps] prov_consttd/carteira caiu -91% vs mediana 6m`).
- **Faixas:** `< 7000 bps` (`SanityClear`) · `7000..8999 bps` (`SanityReviewAdvisory`) · `>= 9000 bps` (`SanityHoldGate`, exige confirmação humana + justificativa gravada no `reg-ledger` antes do envio STA — **nunca** altera exit code ou erros `E/A/I` do System 0).

---

### `S1.4` — Triage, Radar Filter & Escalation Router (4 Tiers)

Combina sinais determinísticos do System 0 (`cadoc`, `rule_code`, `severidade`, `RuleStatus`), telemetria do RAG (`bm25_exact_rule_hit`, `rrf_top1_bps`, `rrf_margin_bps`), saída `S1.2` (`EgressDecision`) e classificador NLU PT-BR **`higopires/DeB3RTa`** (`70M/426M`, **MIT**) / `debertinha` (**MIT**) via `ort` INT8:

| Rota | Destino | Gatilho Objetivo | Custo & Latência |
|---|---|---|---|
| **Tier 0** | **Lookup Determinístico `RuleSpec` + Cartão de Campo** | `lookup_regra` ou explicação simples onde `bm25_exact_rule_hit == true && rrf_top1_bps >= 8500 && RuleStatus::Implemented` | **`$0` · `<5 ms`** (zero LLM, zero alucinação) |
| **Tier 1** | **System 1 Direto** | `S1.1` verde (`>= 9200 bps`); `S1.3` sanity; deduplicação `M2/M7` via produto escalar `Q15` (`cos_sim_bps >= 8800`); filtro de boilerplate do Radar | **`$0` · `<20 ms`** |
| **Tier 2** | **System 2-Fast** | Explicação contextual de crítica única, `S1.1` faixa cinzenta limpa, resumo de diff Radar (`M3`), rascunho `rule_candidate` (`M7`) | Local (`Mistral Small 3.2`/`Qwen3.5-9B`/`Gemma 4 26B-A4B`/`gpt-oss-20b`) ou Cloud ZDR Flash *(só pós-Allowlist `S1.2`)* |
| **Tier 3** | **System 2-Heavy** | Regras `L3 CrossDoc` (`3040↔3050↔2030↔4111`), baixa margem RAG (`rrf_margin_bps < 800`), pipeline `Norma→Regra` (`Draft+Critic`), ou **escalonamento após falha de verificação objetiva no Tier 2** | Local H100/Mac 128GB (`gpt-oss-120b`/`Gemma 4 31B`/`Qwen3.6-35B-A3B`) ou Cloud ZDR Sonnet/Pro *(só `PublicRegulatory`/`SchemaOnly`)* |

---

## 4. Calibração, Conformal OOD & Learning Loop `M2/M8`

```text
  [Wizard / reg-cli]                       [Retornos STA / CRD / .jar (M1)]
     │ • Aceite/correção S1.1 (M2)            │ • Aceite vs Rejeição BCB
     │ • Confirmação/falso alarme S1.3        ▼
     │ • Regret de roteamento S1.4     ──► [S1.2 GUARDRAIL: Allowlist + Aho-Corasick + moBERTo + Canários]
     ▼                                        │ (Se Canary FNR > 0 ou timeout ONNX → ABORTA LOTE!)
  ┌───────────────────────────────────────────▼──────────────────────────────────────┐
  │ SEPARAÇÃO ESTRITA DE DESTINO (Quarentena de Fronteira)                           │
  │                                                                                  │
  │  Destino A: TREINO / CALIBRAÇÃO `System 1`   Destino B: `M7` RULE DISCOVERY      │
  │  (APENAS modelos S1.1..S1.4 & calibradores)  (NUNCA altera pesos para leis!)     │
  │  • S1.1 pares estruturais (zero dado cliente)• `>= 8 obs` & `>= 2 tenants` →     │
  │  • S1.3 rótulos (bug real vs evento válido)    `rule_candidate` → S2 Draft →     │
  │  • S1.4 regret labels (Tier 0/1/2 bastou?)     Compilação Rust + Humano          │
  └─────────────────────┬────────────────────────────────────────────────────────────┘
                        ▼
  [Gatilho M8: >= 200 rótulos novos OU drift ECE > 0.025 OU cadência semanal]
                        │ (Split temporal estrito + agrupado por tenant + held-out selado)
                        ▼
  [Champion-Challenger & Gate M6] ──(falha em 1 gate)──► Rejeita candidato (mantém SHA-256 atual)
                        │ (passa em todos os hard gates)
                        ▼
  Promove `(model_sha256, calibrator_sha256, policy_sha256)` no Scorecard `M5` append-only
```

### Contratos de Calibração e Auditoria (RC Conjunta 18/2025)
1. **Regra de Ouro do Loop:** Nenhuma correção de usuário (`M2`) nem retorno CRD (`M1`) altera pesos ou lógica das regras regulatórias. Regras legais são dados versionados (`RuleSpec`) aprovados por humanos e provados no motor Rust. O loop `M2/M8` calibra **apenas** os submodelos estatísticos do System 1 (`S1.1..S1.4`).
2. **Métricas Obrigatórias no Scorecard `M5` e Hard Gates `M6`:**
   - **`ECE <= 0.025`** (Expected Calibration Error em 10 bins);
   - **Brier Score** decomposto em *Reliability* + *Resolution* (impede calibração trivial na média global);
   - **Risk-Coverage (`Error@Coverage >= 9200`):** taxa de erro real na faixa verde **`<= 0.5%`**;
   - **Proteção OOD (*Conformal*):** em leiautes inéditos do BCB (*covariate shift*), reduz automaticamente `ScoreBps` para a faixa Cinzenta/Vermelha em vez de emitir confiança superestimada;
   - **`S1.2 Canary FNR == 0.00%`** e **`0` violações de roteamento `RouteOnPremOnly`**;
   - **`S1.3 PR-AUC >= champion`** com `False Alarm Rate <= 5%` em `SanityHoldGate`.
3. **Separação Priors Globais vs Adaptador por Tenant:** priors globais do `S1.1` usam apenas features léxicas e hit-rates estruturais (nunca valores de clientes); preferências de colunas de uma IF específica vivem isoladas no adaptador por `tenant_id` (zero vazamento entre clientes).
4. **Invariante `--no-ai` em CI:** teste de propriedade garante que `gen3040` e `reg-validate-core` produzem **bit-a-bit o mesmo XML, mesmo SHA-256 e mesmos vereditos `E/A/I`** com System 1 ativo ou com `--no-ai`.

---

## 5. Auditoria sob RC Conjunta 18/2025 (`S1AuditRecord` & 12 Dimensões)

Toda inferência de System 1 que influencia o fluxo de fechamento gera um **`S1AuditRecord` 100% livre de floats**, serializável via [`jcs.rs`](../../rust/crates/reg-ledger/src/jcs.rs) e referenciável pela assinatura `DualApproval` Ed25519 do [`reg-ledger`](../../rust/crates/reg-ledger/src/lib.rs) no Dossiê RC 18/2025:

```json
{
  "schema_version": "s1_audit_v1",
  "submodel": "s1_3_anomaly_scorer",
  "cadoc": "3040",
  "data_base": "2026-11",
  "model_sha256": "7b9c...e41a",
  "calibrator_sha256": "18af...09c2",
  "policy_sha256": "c401...88b0",
  "thresholds_bps": { "advisory": 7000, "hold_gate": 9000 },
  "score_bps": 9340,
  "decision": "SanityHoldGate",
  "ece_last_eval_bps": 180,
  "treeshap_top3_bps": [
    { "feature": "prov_consttd_ratio_vs_6m_mad", "impact_bps": 4820, "observed": "0.0042", "baseline_median": "0.0470" },
    { "feature": "fat_anual_p50_scale_log_jump", "impact_bps": 3110, "observed": "1000x", "baseline_median": "1x" },
    { "feature": "v310_prejuizo_share_delta",    "impact_bps": 1410, "observed": "0.0810", "baseline_median": "0.0190" }
  ],
  "human_resolution": { "status": "rejected_mapping_fixed", "actor_role": "compliance_officer", "ledger_entry_id": 4821 }
}
```

### Alinhamento às 12 Dimensões Literais da RC Conjunta 18/2025 (`DIMENSOES_RC18`)

| Dimensão RC 18/2025 | Evidência Auditável Entregue pelo System 1 |
|---|---|
| **1. Acurácia** | `S1.3` flagra erros de escala (`1000×`) e quedas abruptas de PDD antes da remessa mesmo quando passam no `.jar`; `S1.1` reduz erro humano no `mapping.toml`. |
| **2. Integridade** | `S1AuditRecord` zero-float canonicalizado via JCS RFC 8785, encadeado no `reg-ledger` sob `DualApproval` Ed25519; `S1.2` impede vazamento de LC 105. |
| **3. Completude** | `S1.1` detecta colunas exigidas não mapeadas e campos condicionais ausentes antes de falhar em runtime. |
| **4. Consistência** | `S1.3` checa coerência multivariada entre atraso (`v205..v330` / `DiaAtraso`), `EstInstFin` (`1/2/3`) e `ProvConsttd`. |
| **5. Tempestividade** | `S1.1` acelera onboarding de novas ondas (IN 733/729/735); `S1.4` Tier 0/1 resolve a maior parte das dúvidas em `<5–20 ms` sem fila de LLM. |
| **6. Rastreabilidade** | Fingerprint completo `(model_sha256, calibrator_sha256, policy_sha256)`, `eval_run` que promoveu o modelo (`M5/M6`) e operador que confirmou (`4-olhos`). |
| **7. Acessibilidade** | Executa local/in-process (`reg-s1-tabular` até em `wasm32` no browser; `reg-server` on-prem), compatível com `--offline=true`. |
| **8. Comparabilidade** | `S1.3` compara a safra contra janelas L4 (`t-1..t-12`) do próprio tenant e coortes anonimizadas (`k >= 5`) sob vetor canônico idêntico. |
| **9. Confiabilidade** | Monitoramento contínuo (`ECE <= 0.025`, Brier Score, `Canary FNR == 0%`), degradação Conformal em OOD e zero impacto no `System 0`. |
| **10. Relevância** | `S1.4` prioriza críticas bloqueantes (`E`) e escala ao `System 2` apenas interpretações complexas e regras cross-doc. |
| **11. Clareza** | `TreeSHAP` (`treeshap_top3_bps`) explica exatamente o porquê de cada alerta em grandezas contábeis reais, sem números placebo. |
| **12. Adaptabilidade** | Loop `M2/M8` recalibra o `System 1` continuamente conforme mudam ERPs e ondas do BCB, sem hardcode frágil. |

---

## 6. Plano de Aterrisagem nas Etapas 0–4 do Plano v3.0

- **Etapas 0–2 (Saneamento, Motor 3040, CLI/Dossiê/STA):** **0% runtime ML/ONNX/GPU** para não acoplar C++/GPU às entregas do core nem conflitar com o empacotamento `ubuntu-latest` / `windows-latest`. Apenas `reg-core` define os tipos puros compartilháveis (`RuleId`, `ProvenanceRef`, `CadocCode`) e `reg-ledger` mantém payload JCS extensível para futuros `S1AuditRecord`.
- **Etapa 3 (`reg-server` `axum` + `frontend/` limpo + WASM Streaming):** entra **`reg-s1-tabular` (`i64` zero-float)** — inclusive no browser `wasm32` para pré-sugestão léxica/`reg-core` (`S1.1`) e alerta local de escala (`S1.3`) sem subir XML — e entra **`reg-ai` + `reg-ai-runtime` (`ort` INT8)** atrás de `--features ai` no servidor com `S1.2` (`moBERTo`/`debertinha` + Allowlist estrutural) + `S1.4 Tier 0` + RAG público + Copiloto SSE.
- **Etapa 4 (Multi-CADOC + Radar + Harness `M1–M10`):** ativa o loop completo `M2/M8` com scorecard `M5`, hard gates `M6`, calibração mensal/semanal, priors cold-start `S1.3`, `S2 Local T0–T5` e `rmcp` auditado no `reg-ledger`.

---

## 7. Referência de Primitivas `i64` (`reg-s1-tabular`) & Gates Hard Blockers

### Exemplo de Primitivas Zero-Float na Fronteira

```rust
/// Produto escalar de dois embeddings Q15 L2-normalizados (`i16`) em basis points (0..=10_000).
pub fn cosine_q15_bps(a: &[i16], b: &[i16]) -> ScoreBps {
    let len = a.len().min(b.len());
    let mut acc: i64 = 0;
    for i in 0..len {
        acc = acc.saturating_add(i64::from(a[i]) * i64::from(b[i]));
    }
    if acc <= 0 {
        return ScoreBps(0);
    }
    // 32767^2 ≈ 1_073_676_289 (~2^30 - 65535); escala exata p/ 0..=10_000 em inteiros.
    let scaled = (acc.saturating_mul(10_000)) / 1_073_676_289;
    ScoreBps(u16::try_from(scaled.clamp(0, 10_000)).unwrap_or(10_000))
}
```

### Gates Objetivos do `System One` (`M6` / Compliance)

| Componente | Métrica | Gate Bloqueante de Promoção |
|---|---|---|
| **Isolamento Core** | `layers.rs` + `--no-ai` | `0` floats/`unsafe` nos crates core; XML/SHA-256/veredito idênticos com `--no-ai` |
| **`S1.1` Auto-Mapper** | `Auto-Accept Error@>=9200` + `ECE` | `Error <= 0.5%`, `ECE <= 0.025`, zero bypass de `mapping_locks` |
| **`S1.2` PII / LC 105 Gate** | Canary FNR + Allowlist estrutural | **`Canary FNR == 0.00%`**, `Recall >= 99.5%` no scrubber PT-BR, zero texto livre c/ cliente em cloud |
| **`S1.3` Anomaly Scorer** | `PR-AUC` + `FAR` em `SanityHoldGate` | `PR-AUC >= champion` em perturbações M4 + `10×/1000×`, `False Alarm Rate <= 5%` |
| **`S1.4` Router** | Violações de egresso & fallback | `0` violações `RouteOnPremOnly`, redução líquida de custo/latência s/ perda de resolução |
