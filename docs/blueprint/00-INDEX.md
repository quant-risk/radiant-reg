# Blueprint v3.0 — Reconstrução Rust do Radiant Norma / Radiant Reg

> **Data Original:** 16/11/2026 · **Atualização Soberana (Plano v3.0 + IA/System One):** 19/11/2026 · **Versão:** 3.0 · **Dono:** Henrique Costa.
> **Governança Atual (19/11/2026):** O produto passa a ser governado pelas **Etapas 0–4 orientadas exclusivamente a Gates Técnicos Verificáveis** ([`25-ROADMAP-EXECUCAO.md`](25-ROADMAP-EXECUCAO.md) §0.1), **sem datas artificiais de calendário**, priorizando **3040 completo + WASM + CLI/Dossiê RC 18 + STA → `reg-server`/Next.js → Multi-CADOC/Radar/IA/Harness**, com a arquitetura de **IA Embarcada + `System One` (`21-ENGINE-IA.md` v3.0 & `26-SYSTEM-ONE-DECISION-MODEL.md`)** plugada na borda sem invadir o determinismo do núcleo.
> **Contexto em uma linha:** a tese v1 (jul/2026) era ~80% correta sobre o mercado e ~100% falsificada sobre a própria execução — 11 dias de commit burst, 4 meses de congelamento, zero evidência externa; este blueprint reancora o produto em uma reconstrução Rust que converte a classe de bug encontrada na auditoria em erro de compilação ou gate de build, e em um posicionamento que nenhum player BR ocupa hoje.

---

## 1. Síntese executiva — os 10 achados decisivos

1. **O motor v1 só valida de verdade o 3040.** A auditoria do backend (research/01, evidência `arquivo:linha`) provou: as 170 regras do 3050 e 11 do 2070 estão implementadas, testadas e **órfãs** (nunca chamadas); **568+ críticas de catálogo são inalcançáveis** em runtime (chaves de seed `2061-DLO` vs códigos de 4 dígitos na API); o **L3 dentro de `ValidateFull` é no-op** (códigos XD consultados mas nunca registrados); há **dois motores L1-L4 paralelos** com semânticas diferentes e colisão do código XD02. Em Rust, cada um desses bugs vira **erro de compilação ou gate de build**: enum `CadocCode` exaustivo, dispatch sem wildcard (clippy bloqueia `_ =>`), catalog completeness gate com 3 estados declarados (ADR-0010), pipeline única L1-L4 (ADR-0011), xmllint como autoridade com erro explícito `L1AuthorityUnavailable` (ADR-0012). A classe de bug "regra ausente = skip silencioso" morre na linguagem, não em code review.
2. **O repo v1 nasceu e morreu em 11 dias.** 263 commits entre 03 e 13/jul/2026 (57k LoC, 1.178 testes green) — e congelado há 4 meses. **53 versões num único dia** (v3.34.0→v3.34.53, 06–07/jul); sprints de **2027** marcadas "✅ fechada" em doc de 08/jul/2026; 83 sprints fechadas no papel; 4 números de cobertura de 3040 circulando simultaneamente no mesmo README. O M1 (piloto pagante, 30/09/2026) **expirou sem chance**: 0 credencial Sisbacen, 0 smoke BACEN, 0 deploy, 0 cliente, 0 receita. A doutrina v2 é **done exige evidência externa** — credencial, protocolo CRD/STA, contrato, dinheiro ou métrica machine-generated; marcos N1–N6 apontam desde o dia 1 para o artefato que os prova, e os kill criteria K1–K5 são avaliados por pessoas nas datas, nunca pelo changelog.
3. **A janela regulatória é real, verificada nas páginas do BCB em 16/11/2026.** DRSAC com leiaute novo na **data-base dez/2026** (IN 694; obrigados **S1–S4 inclusive IPs**, ~455 entidades; semestral jun/dez; **1ª remessa vence ~fev/2027**); IN 733 **onda 3 vigente agora** (nov/2026, NR3 "Limite sujeito à provisão"); **GRSAC novo em 1º/1/2027** (Res 586 + IN 772); **LCR/LCRS** 90% em 1º/1/2027 → 100% em 1º/7/2027 (CMN 5.296 + BCB 560); **FRTB padronizado em 2027** (Res 470/2025 + 592/2026). Quatro dores simultâneas em 2027 para o mesmo comprador — a única urgência que procurement não adia, porque tem prazo de lei.
4. **O espaço de mercado está vago.** cadoc.ai **pivou** de gerador de CADOC para inteligência regulatória (6.700+ normas com vigência, NormaChat, MCP "Norma Skills") — abandonou exatamente o posicionamento que a tese v1 atacava. Ninguém no BR cobre as quatro capacidades juntas: **gerar** (legacy sem IA), **validar** (BCValidador é commodity gratuito), **remeter** (bureau manual) e **aprender** com cada uso (ninguém anuncia). O gap global confirma o timing: **89% das IFs ainda não embarcaram IA em produção** de regulatory reporting (Agentic Gap, Regnology set/2026, 276 praticantes/22 países; 49% dos Tier 1 em piloto, ~8% em produção). O BCB já é player de IA na supervisão (ChatBCB, R$3,6 mi/ano); o lado emissor do relato ainda roda em planilha e num validador Java de 2010.
5. **Universo medido, não estimado.** 1.722 entidades autorizadas (API Olinda, 16/11/2026): **154 bancos, 194 IPs, 138 SCDs, 736 cooperativas**; ~**1.100 remetentes SCR** endereçáveis (gasto total de reporting ~R$600 mi/ano; **mercado de ferramenta** bottom-up ≈ **R$30–100 mi/ano** — 1.100 remetentes × ARPU R$25–90k do ladder §7); 455 instituições S1–S4 são a população obrigada ao DRSAC (S5 dispensado — não vender DRSAC para cooperativa singular, que é ICP de 3040/4111). A consolidação (ABFintechs 724→588; 679 IFs se adequando ao capital mínimo até 31/12/2026) não encolhe demanda — encolhe amadores: sobra demanda por ferramenta profissional em quem opera enxuto.
6. **O pricing antigo deixava 2–8x na mesa.** Ladder novo: **Free R$0 (wedge PLG) / Starter R$2.490 / Pro R$6.900 / Scale R$16.900 / Enterprise R$29.900+**, com add-ons por entidade/volume/DRSAC. WTP verificado: SCD R$60–120k/ano, IP R$80–200k/ano, banco S3–S4 R$250k–1 mi/ano — e a consultoria manual que o produto substitui custa R$3–15k/mês, acima do pricing antigo inteiro. Os **4 gates de upgrade** (entidades, volume, DRSAC, white-label/benchmark) são a máquina de **NRR ≥115%** — cohort *modelado* de 100 Starter projeta **116%** no ano 2 (não há clientes ainda; a conta completa está em 23-PRODUTO §4). Desconto nunca é dinheiro — é "crédito de dados" para o benchmark, protegendo o data moat.
7. **A stack Rust é madura em 16 das 17 dimensões — e o único risco real é conhecido e cercado.** axum 0.8 + sqlx 0.9 + **Postgres 16 único em produção** + quick-xml/xsd-parser (codegen) + **xmllint/libxml2 como autoridade final** + rmcp para MCP; **17 crates** em cargo workspace com bordas hexagonais; frontend permanece Next.js/TS. **Validação XSD 1.0 pura-Rust não existe madura** (crate `xmlschema` é 0.0.x) — o único risco real do stack, mitigado em 3 camadas (codegen + inline + xmllint; Go também não tem validador nativo, o risco é de engenharia, não de impossibilidade). Build até launch: ~R$42–50k de infra+IA+ferramentas — auditoria SOC 2 e pentest da F5 são **linhas condicionais a receita/procurement**, fora desse orçamento — com 1 dev + agentes.
8. **IA com verificação objetiva, nunca autopilot.** Determinismo no core (engine compila e executa regras), IA na fronteira (extração norma→regra, copiloto, RAG) e **verificação objetiva antes de qualquer output de LLM entrar no produto** — o XSD do BCB + engine Rust como juiz final é o que separa *rejection sampling* de alucinação; concorrentes que só têm "LLM lê norma e responde" não têm essa linha de defesa. TCO: **US$505/mês a 10 IFs → US$2.800/mês a 100 IFs** (<15% da receita do cenário base); extração de regra a $0,30–3/candidato substitui ~8h manuais por documento; copiloto a $0,03–0,12/validação com prompt caching. GPU on-prem só por gatilho contratual, nunca por precaução.
9. **O moat não é o motor — é a taxa de aprendizado por evento.** O motor será copiado (qualquer concorrente com LLM + XSD reconstrói um validador em meses); o histórico de eventos de dezenas de IFs não. O harness auto-melhorável converte cada discrepância, correção, mudança de leiaute e disputa em eval versionado com gate de regressão (provas: Stripe Radar −80% em card testing em 2 anos; SWE-bench FAIL_TO_PASS/PASS_TO_PASS; Duco; OpenAI Cookbook). **10 mecanismos M1–M10**; Rule Coverage Index por matriz regra×edge; **Accuracy SLA vendável** (FN rate held-out <0,5% por 2 trimestres — o único pricing que o Honesty Engine permite oferecer honestamente).
10. **A crítica adversarial foi aplicada, não só registrada.** 3 críticos independentes (arquitetura/coerência, segurança/conformidade, dados/pipelines) produziram **39 achados**, incorporados pelo editor e re-verificados por revisor independente em loop, com gate de cobertura determinístico. Exemplos que mudaram a arquitetura: RLS via `set_config('app.tenant_id', …, true)` centralizada num único padrão `with_tenant` (impossível esquecer); catalog gate em 3 estados que **quebra o build** com regra sem estado; paridade definida **por classe de teste** (stubs descartados, asserts de skip-silencioso reescritos, goldens de L4 e regras órfãs sintéticos) em vez de "100% green" global; xmllint endurecido com erro explícito de indisponibilidade (fim da degradação silenciosa para root-tag check em deploy); **ADR-0015** (secrets em Secrets Manager/Vault, `zeroize`, fallback A3→A1 nunca automático); **ADR-0016** (retenção com purge, CMK por tenant, DPA+ZDR para frontier sem residência BR); **ADR-0017** (tenant_id só de claim JWT RS256; header `X-IF-ID` banido com teste de cross-tenant).

**North Star:** *remessas ponta-a-ponta aceitas por mês* — CADOC gerado pelo motor, validado, remetido via STA e aceito no CRD de uma IF real. É a única métrica que só cresce se os cinco pilares funcionarem juntos, e não pode ser falsificada por código.

| Métrica de suporte | Meta |
|---|---|
| T2U (time-to-update) | ≤10 dias úteis norma→leiaute, medido em cada onda |
| Learning rate | Evals novos/mês derivados de eventos reais; accuracy do suite em tendência mensal |
| Entidades remetentes ativas | CNPJs com remessa no mês (base do usage pricing e da NRR) |
| MRR m12 (dez/2027) | conservador ~R$76k · base ~R$210k (45 clientes) |
| Unit economics | churn ≤0,8%/mês · NRR ≥115% · GM ≥88% · payback ≤12 meses |

---

## 2. As 3 decisões estruturais e os 5 pilares de produto

### 2.1 Decisões estruturais (resolvidas, com justificativa)

| Decisão | Escolha | Justificativa em 1 linha |
|---|---|---|
| Estratégia de migração | **Clean rewrite** (repo novo, workspace Cargo do zero), não strangler | Strangler pressupõe tráfego vivo para redirecionar — e não existe (0 deploy, 0 cliente); o Go v1 vira *spec executável* (1.178 testes → goldens) e fonte de dados, não base de port mecânico que transportaria os 6 fios desligados. |
| Ponte DRSAC | **Plano A**: motor Go v1 serve o piloto dez/2026–fev/2027 com wiring mínimo (`POST /v1/validate/2030`) + validação externa obrigatória (BCValidador + CRD) | O `gen2030` existe, mas `drsac.ValidateDocument` é órfão — sem wiring, o "validador DRSAC do piloto" seria a mesma classe de milestone-virando-código que a auditoria condenou; plano B automático se credencial/IF/beta não aparecerem até 30/nov–05/dez/2026. |
| Launch | **Escalonado**: L1 mar/2027 (Free + Starter, após F3) · L2 jun–jul/2027 (Pro/Scale/Enterprise) | O launch Q1/2027 completo não cabe em 32–36 semanas honestas; o K2 (30/jun) avalia o L1 que já estará no ar, não uma promessa. |
| **Fusão (16/11/2026)** | **FASE 1 = wedge V1 "Radiant Reg"** on-prem, 4 crates, com **receita em 2026** (PoC mascarada → Sprint R$35–60k → Licença R$6–12k/mês; **2–4 contratos até 31/12/2026; marco N0**) · **FASE 2 = plataforma multi-tenant** pós-primeiro contrato (~8–10 crates de domínio) · baseline E2E congelado **41,76%** | Receita primeiro, plataforma depois — TPRM (CMN 4.893/5.274) e LC 105 empurram execução in-perimeter para o 1º contrato; detalhe em 25-ROADMAP §0 e 23-PRODUTO §2.0. |

### 2.2 Os 5 pilares de produto (tese v2 §6)

| Pilar | Promessa | Onde está detalhado |
|---|---|---|
| **(a) Corretude compilada (Rust)** | "O sistema não submete o que não consegue provar": enum exaustivo, dispatch fechado, catálogo verificado em build, provenance em toda verdict | 20-ARQUITETURA §1–§4 |
| **(b) Time-to-update como métrica vendida** | SLA público norma→leiaute ≤10 dias úteis; o diff entre ondas (NR→NR2→NR3) é produto, não documento | 20-ARQUITETURA (Radar) · 23-PRODUTO §8 |
| **(c) Copiloto com evidência citada** | IA na fronteira apenas; todo output verificado objetivamente; toda sugestão cita norma/cláusula; humano aprova | 21-ENGINE-IA §1–§3 · 24-UX §7 |
| **(d) Harness que melhora com uso** | Flywheel evento→eval→melhoria; Accuracy SLA publicado e monitorado; benchmark de segmento como tier premium | 22-HARNESS-V4 (inteiro) |
| **(e) Operação ponta-a-ponta até o CRD/STA** | Gerar → validar → remeter (STA REST + senhaws) → ler retornos do CRD → re-ingestir como eval; credenciais PSTA300 são requisito de dia 1 | 20-ARQUITETURA (norma-sta) · 25-ROADMAP §3 |

---

## 3. Mapa de leitura

| Você é... | Ordem | Tempo | Por quê |
|---|---|---|---|
| **Investidor / executivo** | 00-INDEX → 11-TESE-V2 → 23-PRODUTO-PRICING-GTM | ~70 min | Tese em uma frase + o que sobreviveu/morreu da v1 (§2–3 da 11); mercado medido, ladder de preços, GTM e kill criteria K1–K5 (23). É o contrato comercial do blueprint. |
| **Arquiteto** | 00-INDEX → 20-ARQUITETURA-RUST → 21-ENGINE-IA → **26-SYSTEM-ONE-DECISION-MODEL** → 22-HARNESS-V4; contraponto: research/01 + research/04 | ~3h | Doutrina de design em 8 princípios, zero-float, pipeline única (20); conexão dual Cloud ZDR / Local Open-Weights 2026 (21); Decision Models `S1.1..S1.4`, ponto fixo `i64`, calibração e LC 105 (26); harness como subsistema de build (22). |
| **Operador / quem executa** | 00-INDEX → **25-ROADMAP-EXECUCAO (§0.1 Plano v3.0)** → 22-HARNESS-V4 → 20 (§fases) | ~1h30 | Etapas 0–4 orientadas a gates objetivos sem datas artificiais (25 §0.1), saneamento S7, 3040 + WASM, Dossiê RC 18, STA, `reg-server` e gates M6/M10. |
| **Designer / PM de produto** | 00-INDEX → 24-UX-VISAO | ~40 min | Cinco princípios (Trust-first, HITL 1-clique, calendário como homepage, copiloto citado, status machine-generated); o que KEEP do design system e os 9 P0s que precedem qualquer tela nova. |
| **Diligência / leitor cético** | research/02 (tese vs realidade) → research/05 (concorrentes) → research/06 (normativos) | ~1h30 | As provas do que a v1 prometeu e não entregou; o mapa competitivo com fonte URL a URL; a verificação ao vivo dos normativos no BCB. |

Tempo por documento (leitura densa): 00-INDEX ~12 min · 11-TESE ~20 min · 20-ARQUITETURA ~60 min · 21-ENGINE-IA ~25 min · 22-HARNESS ~25 min · 23-PRODUTO ~30 min · 24-UX ~30 min · 25-ROADMAP ~30 min · cada research 15–40 min (total ~3–4h).

---

## 4. As 10 ADRs novas (0009–0018)

| ADR | Decisão (1 linha) |
|---|---|
| **ADR-0009** | Backend v2 em Rust (axum + sqlx + Postgres único), substituindo ADR-0001 para o motor; frontend permanece Next.js/TS — justificativa é correção compilada do domínio + operação 1-pessoa+agentes, não hiring. |
| **ADR-0010** | Rules-as-data com catalog completeness gate: regra semântica é artefato versionado com provenance e vigência; código nativo é exceção referenciada; regra sem estado declarado quebra o build. |
| **ADR-0011** | Uma única pipeline de validação L1-L4 (fim dos 2 motores paralelos do v1); `&DocSet` emprestado como parâmetro, zero globals; L3 paralelizado com rayon. |
| **ADR-0012** | XSD em 3 camadas: codegen `xsd-parser` + validação estrutural inline + xmllint/libxml2 como autoridade final com erro explícito `L1AuthorityUnavailable` — nunca degrada para root-tag check. |
| **ADR-0013** | Harness como subsistema de build/CI: gates bloqueiam regressão de precisão; status do produto é machine-generated (script → JSON → UI), nunca escrito à mão. |
| **ADR-0014** | IA atrás do engine em 3 camadas (`System 0 → System 1 → System 2`): determinismo e zero-float no core (`reg-core/cadoc/validate-core/ledger/sta`), `reg-s1-tabular` (`i64`), `DataPolicy` dual (`AirGapped`/`HybridPolicyRouter`/`CloudZdrFirst`), barreira determinística LC 105 (zero texto livre com cliente na nuvem externa) + `S1.2` defesa em profundidade + `Provenance` obrigatória. |
| **ADR-0015** | Secrets e material de assinatura: senhaws/chaves A1-A3 em Secrets Manager/Vault (nunca env/arquivo/tracing), `zeroize` em memória, fallback A3→A1 bloqueado por config por IF — nunca automático. |
| **ADR-0016** | Retenção e transferência internacional: prazos por tabela/bucket com purge automatizado, CMK por tenant no S3, scrubbing Sentry/OTel, DPA+ZDR para frontier, fluxo export/erase (art. 18) testado no CI. |
| **ADR-0017** | Identidade de tenant: `tenant_id` exclusivamente de claim verificada do JWT RS256 no middleware; header `X-IF-ID` (bridge de dev do v1) banido; teste de integração prova que forjar o header não cruza tenants. |
| **ADR-0018** | Oráculo oficial de submissão (B02): o Validador oficial do BCB (`.jar` SCR3040) é autoridade final de submissão para 3040/3042 (e 2030 quando houver validador); nenhum ZIP sai para o STA sem chancela do `.jar`; hash SHA-256 exclusivamente sobre os bytes do `*_VALIDADO.zip`; adapter headless (JRE ≥ 1.8) com `--max-part-mb` conservador 512–1024 MiB. |

---

## 5. Fases F0–F6 (re-balanceadas: 32 semanas nominais + buffer 4 = até 36)

> **Escopo pós-fusão (16/11/2026):** as fases abaixo compõem a **FASE 2** — plataforma multi-tenant, pós-primeiro contrato (marco N0). A **FASE 1** = wedge V1 "Radiant Reg" (S1–S6, até 31/12/2026) está em `25-ROADMAP-EXECUCAO.md` §0.

| Fase | Semanas | Datas | Entregável central | Gate de saída (evidência externa) |
|---|---|---|---|---|
| **F0** fundação e verdade inicial | 1–2 | 16–29/nov/2026 | Workspace 17 crates, CI com 4 gates, catálogo normalizado, golden corpus dos 1.178 testes Go, STATUS.md machine-generated, ADRs assinadas | CI green em PR real; contagens publicadas pelo build; ≥1 PR bloqueado pelo catalog gate (prova de que funciona, não de que existe) |
| **F1** domínio, leiautes nov/2026, storage | 3–6 | 30/nov–27/dez/2026 | `norma-domain` (enum exaustivo, newtypes, Vigencia, arredondamento por leiaute), codegen XSD nov/2026, sqlx + RLS + catalog gate | Tipos compilam para todos os CADOCs vigentes; RLS provado com testcontainers (tenant A não lê tenant B); gate demonstradamente quebra build |
| **F2** pipeline unificada + 3040/3050 | 7–12 | 28/dez/2026–07/fev/2027 | Pipeline única L1-L4, generators 3040/3050, paridade golden, fuzz do parser em CI | **N1** paridade 100% green (contada pelo harness) + **N2** XML Rust aceito no BCValidador oficial |
| **F3** API, worker, STA, primeiro envio | 13–19 | 08/fev–28/mar/2027 | axum + utoipa, worker apalis (claim atômico), STA client port (Basic preemptivo + PSTA300), generators 2061/2062; spike A3 fora do caminho crítico | **N3** envio STA-h homolog real com protocolo capturado; degrada para N3' (CRD manual) se credencial não chegar |
| **F4** Radar, IA estágio 1, harness, conectores | 20–26 | 29/mar–16/mai/2027 | Radar com diff semântico + auto-PR, copiloto com citação, gates M6 nos 3 tiers, conectores File/API | Gate bloqueia um PR real com regressão de precisão (log do gate vermelho anexado); Radar gera PR de diff simulado; copiloto cita fonte clicável |
| **F5** launch comercial | 27–32 | 17/mai–27/jun/2027 | Pricing público, SDKs 1 por linguagem, MCP server rmcp, conectores DB/MCP, SOC 2 readiness real com auditora contratada | **N4** contrato pagante assinado **e dinheiro recebido**; pentest executado com findings triados |
| **Buffer** | 33–36 | 28/jun–25/jul/2027 | Contingência nomeada — cortes da F4 em ordem fixa (Radar leitor-only → RAG BM25-only → conectores DB/MCP); nunca se corta paridade (N1), BCValidador (N2), envio real (N3) | Buffer não consumido = F6 começa antes; buffer consumido sem corte = próxima re-ancoragem puxa mais buffer |
| **F6** flywheel composto | contínuo | desde ~jul/2027 | M1/M2/M7 contínuos, Accuracy SLA, health scorecard semanal (MDR ≥75%, judge TPR/TNR ≥95%, RCI ≥80%), T2U medido por onda | FN rate caindo por trimestre; K1–K5 avaliados nas datas; **N5** ≥10 clientes/R$50k MRR; **N6** Accuracy SLA em contrato |

---

## 6. Marcos N1–N6 e kill criteria K1–K5

### 6.1 Marcos — a cadeia N1→N6 é consequencial: cada elo é pré-condição do seguinte

| Marco | Evidência externa que prova | Alvo |
|---|---|---|
| **N0** | Primeiro contrato pago da FASE 1 (wedge) — contrato assinado + dinheiro recebido; 2–4 contratos até 31/12/2026 | 31/dez/2026 |
| **N1** | Paridade golden 100% green, contada pelo harness (machine-generated) | 07/fev/2027 |
| **N2** | XML gerado pelo Rust aceito no BCValidador oficial (log anexado à issue) | 07–14/fev/2027 (gate F2 admite +1 semana para a validação externa) |
| **N3** | Protocolo de envio STA-h homolog capturado (hash pré-envio, payload, resposta, timestamps) | 28/mar/2027 |
| **N4** | Contrato assinado + dinheiro recebido + retenção no 3º mês | 31/mai/2027 |
| **N5** | ≥10 clientes pagantes, MRR ≥R$50k (piso do K2) | 31/dez/2027 |
| **N6** | FN rate held-out <0,5% por 2 trimestres + 1º cliente com Accuracy SLA em contrato | 31/jan/2028 |

### 6.2 Kill criteria — avaliados por pessoas nas datas, não pelo changelog

| # | Quando | Critério de morte | Consequência |
|---|---|---|---|
| K1 | 31/mar/2027 | <5 IFs reais com ≥1 remessa ponta-a-ponta aceita (remessa aceita **via CRD manual conta**, marcada N3' enquanto a credencial STA não chegar) | Tese de SaaS morta → pivotar para componente: engine licenciado a casas de software |
| K2 | 30/jun/2027 | <10 clientes pagantes OU MRR <R$50k | Pricing/GTM errados → reprecificar, trocar canal (consultorias no lugar de PLG) |
| K3 | 30/set/2027 | T2U mediano >10 dias úteis em 2 ondas consecutivas | SLA é promessa falsa → removê-lo do pitch; reposicionar em corretude |
| K4 | 31/dez/2027 | Eval suite flat (accuracy sem melhoria por 2 trimestres) com ≥30 eventos/IF | Flywheel é narrativa → moat volta a ser cobertura de leiaute |
| K5 | 28/fev/2027 | 0 IFs **pagantes** vindas da janela DRSAC — o beta é pago; K5 é o gate de *primeira receita*, N4 exige retenção no 3º mês | Urgência normativa não converte → matar GTM de janela; GTM por varredura de consultorias |

---

## 7. Pricing FASE 2 (ladder completo no L2 jun-jul/2027; L1 mar/2027 abre Free+Starter; a FASE 1 usa a régua do wedge — 23 §2.0)

| | **Free** | **Starter** | **Pro** | **Scale** | **Enterprise** |
|---|---|---|---|---|---|
| **Mensal (anual)** | R$0 | **R$2.490** | **R$6.900** | **R$16.900** | **a partir de R$29.900** |
| Mês a mês | R$0 | R$2.990 | R$8.280 | R$19.900 | sob contrato |
| Entidades (CNPJ) | 1 | 1 | até 3 | até 10 | ilimitado/negociado |
| Ops SCR/mês | 1 mil | 50 mil | 200 mil | 1 mi | ilimitado |
| Documentos | 3040 (envio manual) | 3040, 3050, 3044, DIPs core, DRSAC "zerado" | + DRSAC completo, dossiês, API | + todos (SARO/DAS/CAS, Produb, consórcios), multi-cadastro | + segmentações custom, integrações core |
| API / white-label / benchmark | — / — / — | — / — / — | Sim / — / básico | Sim / **Sim** / **completo** | Sim / Sim / completo |
| SLA | — | 99% | 99,5% | 99,9% + Zero Autuação™ | custom |
| Usuários | 1 | 5 | 10 | ilimitado | ilimitado + SSO/SCIM |

**Add-ons:** entidade extra (CNPJ) **R$590/mês** · volume extra (+50k ops) **R$190/mês** · DRSAC completo no Starter **R$990/mês** (janela dez/26–ago/27, S1–S4 pequenas) · Benchmark Premium standalone **R$1.490/mês** · implantação/homologação assistida R$15–50k one-off.

**Zero Autuação™** (item de SLA do Scale): marca a registrar; compromisso = FN rate contratual respaldado por held-out + auditoria amostral do harness; **não cobre** erro de dado de origem da IF. Redação jurídica pendente antes do L1 — sem ela, o item não entra em contrato.

**Máquina de NRR:** os 4 gates (entidade, volume, DRSAC, white-label/benchmark) projetam cohort de 100 Starter em **116%** de NRR no ano 2. MRR m12 (dez/2027): conservador **R$76k** · base **R$210k** (45 clientes) · agressivo **R$430k**. O Free é wedge deliberado: validar leiaute é commodity (o próprio BCB dá de graça), então damos mais que o BCB — regras semânticas do catálogo no browser, sem JRE — e capturamos o e-mail do responsável no momento da dor; os limites (1 entidade, 1 mil ops, sem remessa, sem API) convertem pelo teto.

---

## 8. Calendário normativo 2026–2028 (verificado no BCB em 16/11/2026)

| Norma | CADOC | Data-base / vigência | Prazo / fase | Impacto |
|---|---|---|---|---|
| IN 733 onda 3 (NR3 "Limite sujeito à provisão") | 3040 | nov/2026 — **vigente agora** | data-base corrente | 3ª onda de leiaute em 6 meses quebrou sistemas legados; primeiro dataset do port Rust |
| IN BCB 729/2026 (CNPJ alfanumérico nos leiautes) | SCR (3040/3050/3044) | abr/2026 | leiautes a partir da data-base mai/2026 | novas inscrições RFB alfanuméricas desde jul/2026; identificador temporal por data-base (`CnpjIdentifier` — 20-ARQUITETURA §4.3) |
| IN 694 (DRSAC novo) | 2030 | dez/2026 | 1ª remessa ~fev/2027 (10º dia útil do 2º mês subsequente) | S1–S4 + IPs (~455 entidades); semestral jun/dez; documento zerado via CRD; críticas semânticas não públicas (só XSD + validador + FAQ) |
| IN BCB 735/2026 (Desenrola 2.0 — MPV 1.355/2026) | 3040 | data-base mai/2026 | onda adicional do 3040 | Anexo 37, domínio 13 |
| RC CMN/BCB 14/2025 (capital social mínimo) | — | — | 31/12/2026 | ~679 IFs se adequando; urgência de compliance além de CADOC |
| Res. CVM 247/2026 (FIDC) | — (SCR via gestores) | vigente 1/11/2026 | 9 dias úteis pós-encerramento do mês | multa cominatória diária (art. 56-A do Anexo II da Res. CVM 175); gestores autorizados a consultar o SCR; Securitizadoras no SCR com vacatio até 31/10/2027 — cunha FIDC (23-PRODUTO §5.5) |
| RC Conjunta 18/2025 (qualidade de dados) | — | — | conformidade integral 31/12/2026 | 12 dimensões de qualidade; Dossiê A4/JSON assinado Ed25519 como entregável de entrada (23-PRODUTO §5.5) |
| RC Conjunta 16/2025 (BaaS) | — | vigência 28/11/2025 | transição de contratos até 31/12/2026 | fim das contas-bolsão; responsabilidade integral da prestadora — re-posiciona white-label da FASE 2 (conferir literal na página oficial antes de requisito contratual) |
| Comunicados BCB 45.206/45.339/2026 (MFA Autran) | — | homolog 11/05/2026 · produção 10/06/2026 | 2FA obrigatório no Autran | exceção explícita para usuários especiais (Res. BCB 366/2024, art. 7º IV) — relevante para PSTA300: conta de serviço sem MFA interativo no runtime (20-ARQUITETURA §10) |
| Res 586 + IN 772 (GRSAC novo) | GRSAC | — | 1º/1/2027 | revoga Res 139/2021 + IN 153/2021 |
| CMN 5.296 + Res BCB 560 (LCR/LCRS) | — | — | 90% em 1º/1/2027 → 100% em 1º/7/2027 | S2 passa a calcular LCR; S3/S4 calculam LCRS |
| IN 755 (DRL) | 2160 | vigente desde jul/2026 | subconsolidado sem penalidade só até 31/12/2026 | S2–S4; upsell a partir de jun/2027 |
| Res 470/2025 + 592/2026 (FRTB padronizado) | — | — | 2027 | carteira de negociação; família de leiautes 2036/2060 |
| Res 512/2025 (compulsório-poupança) | 3040 | 2027→ | dedução +1,5 p.p./ano | pressiona marcação no 3040; showcase permanente do T2U |
| Transição códigos 01–04 (Anexos 08/09) | 2030 | — | jun/2027 (2ª data-base DRSAC) | códigos antigos deixam de ser aceitos em registros novos; retarget das mesmas ~455 IFs |
| IN 750 (DLO) | 2061 | 50% (2026) → 75% (2027) | 100% em jan/2028 | escalada anual |
| CLFPS — pisos de provisionamento | — | — | 0 em 1º/1/2028 | fim das transições; output floor em fase final |
| Contínuo | 3060 (mensal, cartões) · 3044 (diário/eventual) · 5710/5711 (cripto/VASPs) · 2090 (ICAAP/TEBU) | vigentes/ondas | — | demanda contínua de time-to-update |

**Leitura:** 2027 é o ano da dor (GRSAC + LCR/LCRS 90→100% + FRTB + transição 2030 + compulsório) — o GTM vende alívio de quatro dores simultâneas para o mesmo comprador (Head de Compliance/CTO de S2–S4 e fintechs). 2028 muda o argumento: de "mude rápido quando a norma muda" para "mude rápido sempre" — o harness com 2 anos de eventos é a prova.

---

## 9. Os 10 mecanismos do flywheel do harness (M1–M10)

| # | Mecanismo | Em 1 linha |
|---|---|---|
| M1 | **Discrepância → TestCase** | Divergência com o BCValidador oficial vira `eval_case` com provenance; o veredito do BC é o `expected` mesmo quando o BC parece errado — a disputa vira `rule_candidate`, não altera o caso. |
| M2 | **Correção de usuário → evalpair** | O clique approve/correct faz snapshot before/after automático e vira caso de teste — o rótulo mais barato que existe custa o clique que o usuário já daria (modelo Duco/Grammarly). |
| M3 | **Radar → regression suite por vigência** | Diff de leiaute detectado gera propriedades proptest, goldens antigos congelados por vigência e casos sintéticos; implementação só está DONE quando a suite da nova vigência está 100% green. |
| M4 | **Corpus sintético challenger/solver/judge** | Célula vazia na matriz regra×edge → challenger gera payload mínimo → solver Rust executa → judge em 2 estágios (determinístico primeiro, LLM em ~10% de amostragem) — rejection sampling confiável porque o verificador é objetivo. |
| M5 | **Scorecards versionados append-only** | Todo run gera scorecard chaveado por (doc, tenant, ruleset, engine, run); a série histórica é o produto de auditoria — nada é sobrescrito. |
| M6 | **Gates de CI em 3 tiers** | PR (fast ~500 cases <90s + mutation incremental), nightly (corpus completo + held-out selado + canários), release (held-out + canários + scorecard assinado); regressão de F2P/P2P/precision-recall bloqueia merge; thresholds só apertam. |
| M7 | **Discovery por clustering** | Correções sem regra mapeada viram clusters (HDBSCAN/UMAP + BERTopic) → `rule_candidate` com ≥8 observações e ≥2 tenants — o único mecanismo que gera receita de catálogo: "o cliente corrigiu X três vezes sem regra" é uma regra implícita do mercado. |
| M8 | **Distillation por gatilho** | Destila só quando medido: ≥200 goldens por regra, judge estável 8 semanas, custo LLM >US$200/mês — nunca "porque ficou legal"; candidato entra como engine_version candidata por A/B com held-out decidindo. |
| M9 | **Red-team contínuo** | Canários com veredito conhecido injetados aleatoriamente (run com canário errado é inválido) + adversarial semanal nas 10 piores regras + cargo-mutants (Mutation Detection Rate <70% = issue automática). |
| M10 | **Auditoria amostral + calibração do judge** | 30 casos estratificados por doc×veredito×provenance/semana (30–45 min, 1 clique) calibram TPR/TNR do judge; judge de família diferente do solver; é a única fonte não-sintética de verdade sobre o sistema. |

---

## 10. Próximas 72 horas (16–18/nov/2026) — as 10 ações imediatas

1. **Saneamento F0′ do repo legado + criar o repo novo** (FASE 1 — 25-ROADMAP §0): **chave rotacionada/revogada hoje** (`backend/dev-private.pem`, commitada, casa com `frontend/.env.pubkey`); **binários removidos** (`git rm` dos 6 Mach-O ~110 MB); **PDFs regenerados** (3 PDFs-404 de 6.245 bytes, `Carta_Circular_3588_STA.pdf` de 725 B, 5 shells Angular de 2.871 B) **com verificação de magic bytes**; **re-download oficial 2030/3040** (pós-IN 694/2025 + IN 729/2026) **com SHA-256**; em paralelo, cargo workspace do wedge com os 4 crates stub (`reg-core`, `reg-cadoc`, `reg-ledger`, `reg-cli`), `[workspace.dependencies]` pinado e CI completo (fmt, clippy `-D warnings`, linter anti-wildcard, test, catalog gate, drift check) — commit 1 já contém as ADRs assinadas.
2. **Abrir o pedido de credencial Sisbacen / PSTA300** (homologação) — o maior lead-time do projeto e o bloqueio histórico do v1 desde jul/2026; checklist dos formulários, **pedido formal PSTA300 protocolado** (operador como "usuário especial" — Res. BCB 366/2024, art. 7º IV), follow-up semanal recorrente; degradação para N3' se não chegar até 31/jan/2027.
3. **Extrair o golden corpus**: script `xtask` converte os 1.178 testes Go em fixtures input/expected versionadas, etiquetadas por status de produção do código que as originou (wired vs órfão — o mapa honesto da F2); dependentes de globals/ordem viram `flaky-spec`.
4. **Normalizar o catálogo**: ETL de `criticas.json` → schema único, dedup dos 11 códigos duplicados, chaves sufixadas (`2061-DLO`) → código de 4 dígitos + categoria; STATUS.md machine-generated no ar desde o dia 1. **CORREÇÃO 18/11/2026:** o STATUS.md machine-generated NÃO foi criado na FASE 1 — **adiado para a FASE 2** (25-ROADMAP §FASE 2 registra o adiamento); o registro de bloqueios/pendências é manual na §10.1 abaixo.
5. **Baixar e versionar os leiautes nov/2026** (IN 733 onda 3), XSDs oficiais, FAQ DRSAC dez/2026 e planilhas de Regras de Validação — git-tag por data-base; a primeira correção real da base normativa parada em jul/2026.
6. **Wiring mínimo do drsac no v1** (plano A da ponte) — **revogado pela fusão (16/11/2026)**: o gen2030 em Go saiu do escopo (retrabalho); o critério de 30/nov passa a ser a nova redação do Bloqueio A.2 (25-ROADMAP §3.2) e o 2030 do piloto passa a ser o `reg-cadoc` em Rust (S4 da FASE 1 — 25-ROADMAP §0).
7. **Tracker dos 6 fios desligados** como gates de não-regressão arquitetural — um teste/CI por fio (dispatch silencioso, chaves sufixadas, XSD relativo, parse repetido, defaults silenciosos, motores duplos) que falha se o padrão renascer no código novo.
8. **Provisionar infra dev** (Postgres gerenciado, Redis, bucket, Sentry, domínio) com budget cap mensal automático no cloud e migrations 001–026 limpas rodando.
9. **Abrir as issues dos marcos N1–N6** com o campo "artefato externo que prova este marco" preenchido de antemão (protocolo STA, log do BCValidador, contrato, recibo) e kill criteria agendados como reminder.
10. **Prospectar o piloto DRSAC**: contato com 5–8 IFs S1–S4 da lista nominal (FBM/ABBC/consultorias — canais verificados) para validar demanda do beta dez/2026; sem sinalização até 05/dez/2026, o §3.2 do roadmap manda para o plano B automaticamente.

### 10.1 Bloqueios e pendências DECLARADAS da FASE 1 (registro manual — 18/11/2026)

Estado honesto da FASE 1: **S1–S5 técnica; S6 bloqueado em N0** ("Fase 1 técnica completa" do commit d25dc5f não qualifica o gate §0 — nenhuma fase fecha por declaração). Pendências que vivem em comentário de código ganham aqui superfície de documento:

**Bloqueios de evidência N0 (commit d25dc5f):**
- **gen2030** — não implementado (S4 virou gen3040 CMN 4.966, commit 8124fcb); depende do re-download do leiaute 2030 IN 694/2025 (`reg-cadoc/src/lib.rs` `TARGET_LEIAUTE_2030`) — a janela DRSAC dez/2026 depende dele;
- **smoke STA-h real (PSTA300)** — bloqueado na credencial Sisbacen (pedido em andamento; degradação N3' documentada);
- **musl cross (CI Linux)** — não feito: sem pacote air-gapped instalável;
- **re-download oficial 2030/3040 com SHA-256** — NÃO executado: `docs/normativos/MANIFESTO.md` segue 100% `[ ]` e "Registro de capturas" vazio; o gen3040 roda sobre o XLS de jul/2026 enquanto a IN 733 onda 3 é a vigente (00-INDEX §8).

**Pendências declaradas em código (fonte única: os módulos; aqui só o mapa):**
- gen3040 (`rust/crates/reg-cadoc/src/gen3040.rs` — doc "PENDÊNCIAS DECLARADAS"): campos N11,7 `TaxEft`/`PercIndx`/`TJE` não modelados (reg-core não expõe `TaxaDecimal`); blocos `Gar`/`Inf`/`Sicor`/`Perda`/`ConIpocs`/`Agreg` não emitidos; `@EstInstFin` ausente → `RuleNotImplemented` (domínio Anexo 41 = {1,2,3}; derivação dias→estágio é modelagem institucional);
- CNPJ alfanumérico (`rust/crates/reg-core/src/cnpj.rs:52,65`): DV base-36 da IN 729/2026 pendente de confirmação oficial — `dv_pendente()`;
- BacenValidatorAdapter (`rust/crates/reg-cadoc/src/validator.rs`): timeout `kill` mata só o wrapper (o `java` neto pode sobreviver — killpg é `unsafe`, proibido no workspace); mitigação operacional: wrapper com `exec`;
- paridade golden (`rust/crates/reg-cadoc/tests/golden_smoke.rs`): runner executado com registry vazio — 46 casos, 0 regras portadas (skip DECLARADO por regra; paridade executiva = 0);
- mapping.lock: máquina de 4 travas implementada e testada em `reg-ledger`, mas nunca exercitada com `mapping.toml` real (arquivo não existe no repo); dedup blake3 P=256 não implementado (sem dependência).

**Workflows:** definição de workflow commitada só do blueprint-fecho; `fase-s3-corpus.dwf.ts` adicionada ao índice em 18/11/2026; runs S4/S6 (`dwfrun-904ae740`, `dwfrun-aba003aa`) ficam gitignored por política, mas os IDs são citados aqui para rastreabilidade.

### 10.2 Auditoria 19/11/2026 & Plano v3.0 (Etapas 0–4 + Arquitetura IA/System One)

Auditoria direta sobre `feat/rust-v4-core` (`510b56f`), planilhas oficiais do BCB e legado Go/Frontend consolidou o **Plano v3.0 orientado a gates objetivos (sem datas artificiais de calendário)** em [`25-ROADMAP-EXECUCAO.md`](25-ROADMAP-EXECUCAO.md) §0.1 e a especificação de IA em [`21-ENGINE-IA.md`](21-ENGINE-IA.md) v3.0 + [`26-SYSTEM-ONE-DECISION-MODEL.md`](26-SYSTEM-ONE-DECISION-MODEL.md):

- **Catálogo 3040 real:** `3040/SCR3040_Criticas.xls` contém `253` regras (`224 Habilitado=s` e `29 Habilitado=n`), enquanto o port não commitado `rules3040/` (187 regras) trouxe stubs `Ok(())` e divergências (`B12`, `C71..C74`) — exige quarentena e reconstrução limpa por ondas sobre as `224` ativas com `catalog-parity` e `mutations.rs` real (**Etapas 0–1**).
- **Descoberta em `SCR3040_Leiaute.xls`:** abas de atualizações (`r82..r86`) já contêm descontinuação Cosif, IN 733/2026 e IN 735/2026; faltam erratas pós-mai/2026, `.jar` atual, certs raiz BCB (§2.3.1) e preencher `MANIFESTO.md`.
- **Fronteira Web & Empacotamento:** `musl` cross é substituído por builds nativos `ubuntu-latest` (Docker `debian-slim` + JRE ≥ 1.8) + `windows-latest`; DRSAC 2030 sai do caminho crítico pré-3040 e entra pós-3040 sob demanda; validação web nasce zero-upload em WASM streaming (`reg-validate-core`) isolada do `frontend/` v1 até a Etapa 3.
- **IA em 3 camadas (`System 0 → System 1 → System 2`):** `0%` runtime ML/GPU nas Etapas 0–2 (aposentadoria imediata de `backend/internal/insights/` & `synth/` na Etapa 0); entrada controlada do `System 1` (`reg-s1-tabular` `i64` zero-float + `ort` INT8) + `reg-ai` (`DataPolicy`) + RAG público na Etapa 3; loop `M2/M8`, `S2` local (`T0–T5`, Apache 2.0/MIT 2026), Radar Two-Pass e `rmcp` na Etapa 4.

---

## 11. Riscos top-5 (do roadmap §7 — severidade alta primeiro)

| # | Risco | Mitigação | Sinal de alarme |
|---|---|---|---|
| 1 | **XSD 1.0 sem validador Rust maduro** (`xmlschema` 0.0.x; único risco real do stack) | 3 camadas (ADR-0012): codegen + inline + xmllint autoridade; fuzz obrigatório; plano B extremo: sidecar Xerces-J | Spike da semana 3 não converte os XSD oficiais sem trabalho manual pesado |
| 2 | **Credencial Sisbacen não obtida** (em 4 meses, nunca foi) | Pedido aberto nas 72h + follow-up semanal; fallback: BCValidador + envio manual via CRD (aceitável para o zerado) | Follow-up #2 sem resposta; 31/jan/2027 degrada para N3' |
| 3 | **DRSAC: críticas restritas** (não públicas; engine do v1 órfão) | Piloto vende geração + BCValidador + envio assistido do zerado, sem promessa de regras semânticas; mitigação/sinal atrelados aos **4 itens do Bloqueio A.2** (saneamento + re-download oficial + PSTA300 protocolado + reg-cadoc 2030 Rust canônico — 25 §3.2) | IF do piloto pedir cobertura que não existe; qualquer dos 4 itens do Bloqueio A.2 não fechado até 30/nov |
| 4 | **Revisão de Rust por 1 pessoa** é o gargalo | Revisores automáticos em série (clippy → anti-wildcard → catalog gate → harness-gate → cargo-mutants); humano só em decisão de domínio | Fila de revisão >2 dias por PR |
| 5 | **Burnout do único dev** | Buffer de 20% nomeado; semanas de 5 dias; 1 semana off após N3 e após launch; spikes em background | 2 fins de semana consecutivos trabalhando |

Os outros 5 (mudança de leiaute no meio do port, cópia por concorrente, migração do catálogo, frontend legado, judge inflar métricas) e as mitigação completas estão em `25-ROADMAP-EXECUCAO.md` §7.

---

## 12. Inventário do blueprint e nota de método

### 12.1 Arquivos

**Documentos principais (11 — 9 ativos + 2 v1 substituídos + 2 legados jul/2026 depreciados):**

| Arquivo | Conteúdo |
|---|---|
| `docs/blueprint/00-INDEX.md` | Este índice — ponto de entrada (atualizado v3.0 em 19/11/2026). |
| `docs/blueprint/11-TESE-V2.md` | Tese v2: o que sobreviveu/morreu da v1, vazio de mercado, posicionamento, 5 pilares de produto, North Star, kill criteria K1–K5, calendário GTM. |
| `docs/blueprint/20-ARQUITETURA-RUST.md` | Arquitetura: 8 princípios de design, 17 crates (vs wedge + Plano v3.0), pipeline única L1-L4, modelagem do domínio, XSD em 3 camadas, STA, RLS, ADRs 0009–0018, §12 IA zero-float. |
| `docs/blueprint/21-ENGINE-IA.md` | Engine de IA v3.0: conexão dual Cloud ZDR / Local Open-Weights 2026 (`T0–T5` Apache 2.0/MIT), papel por superfície, barreira LC 105, `Thinking` vs JSON, `rmcp` hardened, RAG dual-store, TCO e rollout Etapas 0–4. |
| **`docs/blueprint/26-SYSTEM-ONE-DECISION-MODEL.md`** | **System One (Decision Model) v3.0:** camada cognitiva intermediária `S1.1..S1.4` (`reg-s1-tabular` `i64` zero-float + `reg-ai-runtime` `ort`), calibração `ECE <= 0.025`, Conformal OOD, loop `M2/M8` e auditoria RC 18/2025 (`S1AuditRecord`). |
| `docs/blueprint/22-HARNESS-V4.md` | Harness v4: tese do moat (taxa de aprendizado por evento), 10 mecanismos M1–M10, modelo de dados, Honesty Engine, Accuracy SLA, M8 calibrando `System 1`. |
| `docs/blueprint/23-PRODUTO-PRICING-GTM.md` | Mercado medido (1.722 entidades, S1–S5, SAM ~1.100 remetentes), ladder de 5 tiers + add-ons, autopsy do pricing antigo, projeção MRR em 3 cenários, GTM híbrido, marca, jogada DRSAC em 3 movimentos, playbook de vendas. |
| `docs/blueprint/24-UX-VISAO.md` | UX: 5 princípios (Trust-first, HITL 1-clique, calendário como homepage, copiloto citado, status machine-generated), KEEP do design system "Institutional Premium Noir", 9 P0s, Wizard 2.0 com streaming L1–L4, telas novas, roadmap UX por fase. |
| `docs/blueprint/25-ROADMAP-EXECUCAO.md` | Execução: **§0.1 Plano v3.0 soberano (Etapas 0–4 sem datas artificiais, gates técnicos + IA integrada)** + registro histórico do ciclo 16–18/nov/2026, clean rewrite, orçamento e riscos top-10. |
| `docs/ON_PREMISE_LLM_SPEC.md` & `docs/LLM_INTEGRATION_GUIDE.md` | Specs Go/Ollama jul/2026 — **depreciados/arquivados** em 19/11/2026 (substituídos por `21` e `26`). |
| `MASTER_PLAN.md` (raiz) | Plano Ouro v1 — **substituído** pela 11-TESE-V2; mantido como registro histórico e fonte da auditoria (tese vs realidade). |
| `PRODUTO_TESE_ROADMAP.md` (raiz) | Tese v1 — **substituída**; idem. |

**Pesquisas (9, `docs/blueprint/research/`):**

| Arquivo | Escopo |
|---|---|
| `01-backend-audit.md` | Auditoria exaustiva do Go (35 packages, 57k LoC, 1.178 testes): regras órfãs, críticas inalcançáveis, L3 no-op, motores duplos — evidência `arquivo:linha`. |
| `02-tese-vs-realidade.md` | O Plano Ouro contra o git log: 11 dias de construção, 53 versões/dia, sprints-fantasma, milestones falsificáveis por código. |
| `03-frontend-sdk-tooling-audit.md` | Frontend real (10.8k LoC TS), wizard com passo 404, SDKs em 2 gerações colidindo, catálogo ad hoc, OpenAPI desalinhada, zero testes de frontend. |
| `04-rust-stack.md` | Stack Rust dimensão a dimensão (~30 queries): madura em 15/16, XSD 1.0 como único risco real, custo de migração 14–23 semanas. |
| `05-concorrentes-mercado.md` | Mapa BR (Matera, cadoc.ai, Celcoin, Evertec, BCValidador) + padrão global 2026 (Regnology, Suade, Norm AI, Fenergo) — fonte URL a URL. |
| `06-normativos-realidade.md` | Verificação ao vivo no BCB em 16/11/2026: IN 694, IN 733 onda 3, GRSAC, LCR/LCRS, FRTB, correções da tese v1 (S5 dispensado, semestral, PSTA300). |
| `07-engine-ia.md` | Preços de IA verificados nov/2026 (Gemini 3 Pro/Sonnet 4.5/GPT-5.1/on-prem), pipeline de extração, copiloto, RAG, rollout em 3 estágios, TCO 10/100 IFs. |
| `08-produto-pricing.md` | TAM/SAM/SOM via API Olinda, WTP por segmento, unit economics (churn, NRR, GM, payback), PLG vs sales-led, projeção MRR 12m. |
| `09-harness.md` | Spec do harness v4: estado da arte (Stripe Radar, SWE-bench, Duco, OpenAI Cookbook), DDLs Postgres, Honesty Engine, 6 fases de implementação. |

**Volumes anexos da fusão (5, `docs/blueprint/volumes/`) — pendente de entrada:**

| Volume | Conteúdo |
|---|---|
| `docs/blueprint/volumes/` (Vols. 1–5) | Blueprint Gemini — anexos da fusão de 16/11/2026. **Pendente de entrada: o diretório `docs/blueprint/volumes/` ainda não existe no repo.** |

**Workflow persistente:** `.zcode/workflows/blueprint-fecho.dwf.ts` — fecha o blueprint de forma idempotente (só reescreve o que falta ou está stub <150 linhas): escreve seções faltantes, roda os 3 críticos em loop adversarial com editor + verificador independente, valida cobertura com gate determinístico (`wc`/glob) e consolida este índice com leitura independente.

### 12.2 Nota de método

Topologia da produção deste blueprint, nesta ordem: **(1)** 9 pesquisas independentes com evidência citada (código `arquivo:linha`, APIs públicas, páginas oficiais do BCB renderizadas em 16/11/2026, fontes de mercado URL a URL); **(2)** 3 críticos adversariais independentes sobre a arquitetura (lentes de coerência/exequibilidade, segurança/conformidade, dados/pipelines); **(3)** 39 achados consolidados; **(4)** loop editor→verificador — o editor aplica cada achado no doc 20, o verificador independente reconfirma do zero e devolve os não incorporados, até esvaziar a fila; **(5)** gate de cobertura determinístico (glob + `wc`, mínimo de linhas por documento) e este índice escrito por agente separado, com leitura independente dos 10 documentos. Todas as contagens de entidades e datas normativas foram extraídas na data deste blueprint — **re-executar as consultas no dia de qualquer apresentação a board**.

**Fusão com segundo blueprint adversarial (16/11/2026):** o blueprint mestre foi reconciliado com um segundo blueprint (Gemini) que propunha um wedge V1 on-prem de 4 crates; a fusão foi aprovada pelo dono (Henrique Costa). Claims verificados contra o repo (`backend/dev-private.pem` commitado casando com `frontend/.env.pubkey`; 6 binários Mach-O ~110 MB; PDFs-404; `StubClient`; gen2030/4111/3040) e contra fonte primária (Res. CVM 247/2026 no PDF oficial, Res. Conjunta 18/2025, INs BCB 729/735/2026, Comunicados BCB 45.206/45.339/2026). **Pendente:** literal da RC Conjunta 16/2025 conferido na página oficial antes de virar requisito contratual.

### 12.3 O que este blueprint não é

Não é promessa de cobertura ("100% do ecossistema SCR" morreu com a v1): o launch é 3040 + 3050 com profundidade verificada, 2030 na sequência com revisão humana obrigatória (críticas do 2030 não são públicas), demais CADOCs por demanda pagante. Não é plano de equipe: opera 1 dev + agentes, e o cronograma é governado por lead-times externos (credencial Sisbacen, ciclos de venda), não por linhas de código semanais. Não é tese de moat por código: o motor é replicável; o flywheel de eventos não. Não é SOC 2 como sprint: compliance é propriedade nativa do motor + auditoria de terceiro comprada como evento externo, quando procurement pedir.

---

*Blueprint fechado em 16/11/2026. Re-ancoragem obrigatória nos kill criteria K1–K5 nas datas respectivas e sempre que um marco N fechar.*
