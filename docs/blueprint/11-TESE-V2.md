# 11 — TESE v2.0: Radiant Norma (Blueprint de Reconstrução Rust)

> **Data:** 16/nov/2026 · **Versão:** 2.0 — substitui `PRODUTO_TESE_ROADMAP.md` e o Plano Ouro (jul/2026) como referência de produto.
> **Escopo pós-fusão (16/11/2026):** a execução desta tese agora é **duas fases** — FASE 1 = wedge V1 "Radiant Reg" (4 crates reg-*, on-prem, receita 2026) e FASE 2 = plataforma multi-tenant. Este documento descreve a tese inteira; ver `25-ROADMAP-EXECUCAO.md` §0 e `00-INDEX.md` §2.1.
> **Fontes:** `research/02-tese-vs-realidade.md`, `research/05-concorrentes-mercado.md`, `research/06-normativos-realidade.md`, `research/08-produto-pricing.md` (todas de 16/nov/2026).
> **Contexto do repo v1:** construído em 263 commits em 11 dias (pico de 53 tags num dia) (03–13/jul/2026), congelado há 4 meses, 83 sprints "fechadas" no papel, 0 clientes, 0 receita. A tese de jul/2026 era ~80% correta sobre o mercado e ~100% falsificada sobre a execução. Esta versão mantém o que a evidência confirma, mata o que ela desmente e reancora o plano em normativos verificados na página oficial do BCB em 16/11/2026.

---

## 1. A tese em uma frase (v2.0)

> **A Radiant Norma será o primeiro engine AI-native de CADOC do Brasil — um motor determinístico em Rust que gera, valida e remete documentos regulatórios com corretude verificada em compilação e evidência citada em cada veredito — e cujo moat não é o motor, mas a taxa de aprendizado por evento regulatório, capturada por um harness auto-melhorável (evento → eval → melhoria).**

### Desdobramentos

| # | Desdobramento | Conteúdo |
|---|---|---|
| D1 | **O que vendemos** | Geração + validação + remessa ponta-a-ponta. Não validador (BCValidador é gratuito, Java, docs de 2010/2013 — pesquisa/06 §6.1) e não inteligência normativa (cadoc.ai já ocupou esse espaço e abdicou do nosso). |
| D2 | **Como ganhamos confiança** | Determinismo no engine: `CadocCode` enum exaustivo, dispatch sem skip silencioso, catálogo de críticas verificado em build, provenance em toda verdict. IA apenas na fronteira (extração, explicação, copiloto), sempre com verificação objetiva downstream. |
| D3 | **Por que agora** | Janela dez/2026–2027 com quatro dores simultâneas e verificadas (DRSAC dez/26, GRSAC jan/27, LCR/LCRS jan/27, ondas 3040) + 89% das IFs sem IA em produção de regulatory reporting (Agentic Gap, Regnology set/2026). |
| D4 | **O que é defensível** | Taxa de aprendizado por evento: cada erro de remessa, retorno de CRD e mudança de leiaute vira eval versionado que melhora o motor de todos os clientes. O motor pode ser copiado; o histórico de eventos não. |
| D5 | **Quem paga** | Mid-market SCR (SAM ≈ 1.100 IFs) com preço público e entrada acessível, escalando por entidades, volume, DRSAC e white-label. WTP verificado está 2–8x acima do pricing antigo. |

---

## 2. O que sobreviveu da tese v1 (reafirmado ou corrigido, com evidência)

| Pilar da tese v1 (jul/2026) | Veredicto v2 | Evidência (16/nov/2026) |
|---|---|---|
| **Motor de geração como produto** — "IFs não compram validador" (T1) | **REAFIRMADO** | BCValidador é commodity gratuito (ZIP Java ~2,7 MB, doc técnico 2010/2013). Nenhum player BR anuncia geração AI-native com remessa após o pivô do cadoc.ai; Matera não posiciona IA; Dattos não remete CADOC; LUZ/MITRA é consultivo. O diferencial declarado em jul/2026 ficou mais forte em nov/2026. |
| **Janela DRSAC dez/2026 → fev/2027** (T2) | **REAFIRMADO, com correções técnicas** | IN BCB 694/2025 verificada: leiaute novo vale na **data-base dez/2026**; obrigados: **S1–S4, inclusive IPs** (≈455 entidades + IPs enquadradas; S5 dispensado — FAQ oficial); **semestral** (jun/dez), prazo 10º dia útil do 2º mês subsequente → **primeira remessa vence ~fev/2027**. Correções: não existe cronograma de adesão S1–S5; o 2030 **não tem planilha de críticas pública** (só XSD + validador + FAQ) — o parser de "críticas 2030" da v1 era inviável; transição de códigos antigos corre até jun/2027. |
| **Mid-market** (T3) | **REAFIRMADO, refinado** | 1.722 entidades autorizadas (API Olinda BCB, 16/11/2026). SAM ≈ 1.100 remetentes SCR: 138 SCDs, 194 IPs, 80 SCFIs, 21 SCMs, 11 SEPs, 101 DTVMs, 57 CTVMs, 47 SCCs, 736 cooperativas, 374 no segmento S4. Consolidação (ABFintechs de 724→588 associadas; autorizações endurecidas) deixa sobreviventes maiores e mais regulados — favorece ferramenta profissional sobre planilha. |
| **Cross-doc L3 como moat** | **REAFIRMADO** | XD01–XD12 implementados e testados no v1 (código real). Res BCB 154/2022 (BCBS 239 BR) torna rastreabilidade exigência, não diferencial. O próprio BCB ganhou IA na supervisão (ChatBCB; R$3,6 mi/ano em IA — pesquisa/05 §E.1): divergência intra-IF será flagrada pelo regulador antes de virar multa. O valor de "2040 OK + 3050 OK mas consolidado diverge" só cresce. |
| **Humano no loop** ("copiloto, não autopilot") | **REAFIRMADO, elevado a princípio** | Agentic Gap (Regnology, set/2026, 276 praticantes/22 países): **89% das IFs ainda não embarcaram IA em produção**; 49% dos Tier 1 têm piloto, ~8% em produção. Suade+PwC e Regnology convergem para "auditable execution fabric". Em compliance, vence quem entrega IA confiável com evidência — autopilot é incomprável. |
| **Pricing acessível** | **CORRIGIDO** (princípio sobrevive, números morrem) | Preço público contra o gated dos 4 concorrentes sobrevive. Os números morreram: Lite R$1,5k/mês está **2–8x abaixo do WTP** e abaixo do custo de consultoria manual (R$3–15k/mês); banda de WTP: SCD R$60–120k/ano, IP R$80–200k/ano, banco S3-S4 R$250k–1 mi/ano. **FASE 1 (wedge)** usa a régua própria **Sprint R$35–60k / Licença R$6–12k/mês** (23-PRODUTO §2.0); o ladder abaixo (pesquisa/08 §6.1): **R$2.490 / R$6.900 / R$16.900 / R$29.900+** + usage e add-ons — **vale a partir da FASE 2**. |

---

## 3. O que MORREU da tese v1

| Elemento morto | Evidência da morte | Substituto v2 |
|---|---|---|
| **Roadmap-as-code** (83 sprints fechadas em 36h) | v3.34.0→v3.34.53 datadas 06–07/jul/2026; sprints 49–65 de **2027** marcadas "✅ fechada" em doc de 08/jul/2026; quarters futuros marcados "已完成"; numeração de sprints reutilizada 3x com significados diferentes | Milestones só fecham com **evidência externa**: protocolo CRD/STA, contrato assinado, relatório de auditor, eval versionado. Status de números (cobertura, regras, testes) emitido por script a partir do código — nunca escrito à mão. |
| **Pricing antigo** (Lite R$1.5k) | 2–8x abaixo do WTP observado; abaixo da consultoria manual que substitui; sinaliza "brinquedo" em procurement | **FASE 1 (wedge):** régua Sprint R$35–60k / Licença R$6–12k/mês (23-PRODUTO §2.0). **FASE 2:** ladder R$2.490/6.900/16.900/29.900+ com 4 alavancas de NRR: entidades (CNPJ), volume SCR, DRSAC completo, white-label/benchmark. Preço público como âncora; contrato enterprise quando procurement exigir. |
| **SOC 2 como sprint** | Sprints 56/65 "fechadas" como `internal/soc2` (evidence collector em Go); nenhuma auditoria contratada, nenhum auditor | Compliance vira propriedade nativa do motor (audit hash-chain tamper-evident, provenance em toda verdict — portadas para Rust) e a auditoria de terceiro é **evento externo comprado** (ISO 27001/SOC 2 Type II quando procurement bancário pedir), não linha de roadmap. |
| **SQLite em produção** | ADR-0001 declarava Postgres 16 "definitiva"; o código usa SQLite como driver default (`db.go:71`) | v2 escolhe **uma** stack de dados: Postgres + sqlx (16 crates, axum), sem driver alternativo em produção. SQLite só como estratégia de teste explícita e documentada. |
| **Marca "Norma"** | Colisão direta com Norma / NormaChat / NormaCast / Norma App do cadoc.ai (uso comercial público anterior; confusão de search e risco de marca) | Entidade legal mantém "Radiant"; marca de produto nova, curta, PT-pronunciável, domínio .com.br livre, checagem INPI **antes de Q1/2027**. Custo de rebrand hoje é baixo (pré-launch). |
| **"10 CADOCs de uma vez"** | O motor v1 só valida 3040 de verdade: ~22 regras reais + stubs honestos; **2030 com zero regras** (críticas não públicas); catálogo 1.099 é real, mas cobertura de leiaute ≠ profundidade de regras; "100% do ecossistema SCR" nunca foi verdadeiro | **Profundidade verificada antes de cobertura**: 3040 + 3050 (críticas públicas, fonte verificável) no launch; 2030 na sequência (XSD + FAQ como fonte, revisão humana obrigatória); demais CADOCs por demanda pagante. |
| **Milestones falsificáveis por código** | M1 (piloto pagante, 30/09/2026) expirou sem cliente; Sprint 29 "smoke BACEN validado" nunca rodou (sem credenciais); Pilot 2/3/4 "fechadas" como CRUDs Go; Stripe pronto sem conta live | Milestone v2 = fato externo verificável. Kill criteria avaliados por pessoas, não pelo changelog (ver §7). |

---

## 4. O vazio de mercado

**O movimento que abriu o vazio:** cadoc.ai pivou de gerador de CADOC para **inteligência regulatória** (6.700+ normas BCB/CMN com vigência, grafo de conexões, NormaChat, MCP Server "Norma Skills"). Abandonou exatamente o posicionamento que a tese v1 atacava — e com isso o espaço **"engine AI-native que gera, valida e REMETE CADOC"** ficou vago no Brasil.

**O gap global que confirma o timing:** Regnology "Agentic Gap" (set/2026, 276 praticantes, 22 países): **89% das instituições ainda não embarcaram IA em produção de regulatory reporting**; 49% dos Tier 1 têm pilotos agentic, só ~8% chegaram a produção. A ponte apontada pelo mercado é tratar IA como "auditable execution fabric" — não compliance officer autônomo. No mundo, Suade se autodeclara "the only AI-native regulatory reporting platform" (abr/2026). **No BR, ninguém ocupa essa frase — e ninguém cobre as quatro capacidades ao mesmo tempo:**

| Capacidade | Quem tem hoje no BR |
|---|---|
| **Gerar** o CADOC (dado → leiaute) | Matera/PDCase/Prognum (legacy, sem IA); ninguém AI-native |
| **Validar** com regras oficiais | BCValidador (gratuito, commodity); casas de software embutem o próprio |
| **Remeter** ponta-a-ponta (STA/CRD) | Bureau manual (RegulaSoft); legacy amarrado ao core |
| **Aprender** com cada uso (flywheel de eventos) | Ninguém anuncia |

**O regulador empurra o mercado para nós:** o BCB virou player de IA (ChatBCB, R$3,6 mi/ano em IA na supervisão, CdE IA desde 2024) e modernizou a ingestão (JSON nos 9010/9011/9030/9031, API Olinda). O lado receptor do relato está machine-readable; o lado emissor ainda roda em planilha e num validador Java de 2010. Quem gera dados sem rastreabilidade será flagrado por uma supervisão com IA.

> **Tese v2 em uma linha:** o primeiro engine AI-native de CADOC do Brasil, onde o sistema melhora a cada uso, com verificação objetiva.

---

## 5. Posicionamento

### 5.1 Quadro comparativo

| | Matera | cadoc.ai | Celcoin/Regulatório Mais | Evertec/Compliasset | **Radiant Norma v2** |
|---|---|---|---|---|---|
| O que é | Legacy: core bancário + RegTech (250+ IFs, 90M+ contas) | Inteligência regulatória (pivô nov/2026) | Regulatório em bundle BaaS | GRC + agenda regulatória | Engine AI-native de geração+validação+remessa |
| IA | Não posiciona | NormaChat, MCP (normas) | Conteúdo técnico | Alertas | Determinismo no core + IA na fronteira com verificação |
| Preço | R$30k+/ano (gated) | R$5–15k/ano (gated) | R$3–8k/ano (bundle) | gated | Público: wedge R$35–60k Sprint / R$6–12k/mês Licença (23 §2.0) → ladder R$2.490→R$29.900+ na FASE 2 |
| Conflito | Amarra core | — | **Vende BaaS e compete com clientes SCD/IP** | Consolidação de ecossistema | Neutro e auditável |

### 5.2 vs Matera — o incumbent legacy

250+ IFs, receita-alvo R$500 mi, implantação em meses, amarração de core bancário, leiautes hardcoded — cada IN nova (733, 750, 694, 755) dói neles tanto quanto nos clientes. Não posiciona IA e não publicará preço. **Ataque:** time-to-update (leiaute novo em dias, não meses) + DX (onboarding self-serve) + neutralidade (não amarra nada). Não atacar onde eles são fortes: relacionamento, capacidade de implementação, conta enterprise.

### 5.3 vs cadoc.ai — parceiro candidato, não inimigo

O pivô deles **criou** o nosso vazio; o movimento certo não é atacar quem saiu do espaço. Eles detêm o ativo que não vale duplicar: grafo de 6.700+ normas com vigência e conexões. **Integração proposta:** Radiant consome norma como dado (Norma Skills/MCP como fonte de mudança normativa) e executa a parte que eles abdicaram — mapeamento norma→regra→leiaute→remessa. Para eles, somos o desdobramento de produto do grafo; para nós, somos a camada de execução do grafo. Negociar antes de concorrer por keyword.

### 5.4 vs ecossistema Celcoin/Evertec — a camada neutra

Celcoin vende BaaS **e** Regulatório Mais no mesmo grupo: é competidor potencial de todo SCD/IP que seria nosso cliente — e cliente de compliance não se constrói sobre o próprio competidor. Evertec consolidou GRC (Compliasset) na mesma lógica de ecossistema. **Posicionamento Radiant:** a camada neutra auditável — sem vertical própria, sem BaaS, sem core; single-tenant e on-prem possíveis (Rust single binary), regras e provenance auditáveis por terceiro indicado pela IF. Neutridade é argumento de procurement, não só de engenharia.

### 5.5 vs global (Regnology, AxiomSL, Suade)

Não jogam no BR e não mostrarão interesse no SAM de ~R$600 mi/ano. Copiar os padrões que venceram lá — rules-as-data versionadas com vigência e citação normativa (Fenergo/Higson), granular data model (RGD/Regnology), copiloto com evidência (Norm AI, Suade) — não o preço deles (Duco ~US$183k/ano é inalcançável e irrelevante aqui).

---

## 6. Pilares de produto v2

### (a) Corretude compilada (Rust)

O compilador é o primeiro validador. `CadocCode` como enum exaustivo (adicionar um CADOC sem implementar geração = erro de compilação); dispatch sem skip silencioso (todo `match` fecha); catálogo de críticas completo verificado em build (cada crítica tem handler ou marca explícita `Severity::Informational` com justificativa versionada); **provenance em toda verdict** (`FieldMapping{Source, RuleRef, Confidence}` + citação normativa). Arquitetura: 16 crates, axum + sqlx + Postgres; frontend permanece Next.js. Isso é a promessa comercial "o sistema não submete o que não consegue provar".

### (b) Time-to-update como métrica vendida

O SLA público **norma→leiaute** (alvo: leiaute atualizado + diff semântico + regras re-testadas em sandbox em ≤10 dias úteis da publicação da IN) é a métrica que a compra avalia, porque é a dor que a IN 733 expôs: três ondas de leiaute em 6 meses (mai/jul/nov/2026) quebraram sistemas legados. Schema registry versionado por data-base com changelog e EOL (mecânica da ADR-0004, mantida). O **diff entre ondas** (NR→NR2→NR3) é produto, não documento — ninguém no BR o entrega.

### (c) Copiloto com evidência citada

IA na fronteira apenas: extração de regra candidata de IN/PDF/planilha, explicação de campos, drafts de resposta a crítica. Todo output de IA passa por verificação objetiva (XSD, catálogo de regras, cross-doc) antes de virar verdict; toda sugestão cita norma/cláusula (padrão Norm AI); humano aprova — "compliance officer não confia em mágica" e 89% do mercado confirma que autopilot não passa do piloto. Determinismo no engine + IA verificável na fronteira é a única arquitetura vendável em compliance BR.

### (d) Harness que melhora com uso (Accuracy SLA)

O flywheel evento→eval→melhoria: todo erro de remessa, retorno de CRD, mudança de leiaute e divergência cross-doc vira **eval versionado** no suite; evals rodam no CI a cada mudança de motor/regra; Accuracy SLA publicado e monitorado (taxa de reprovações evitáveis no CRD caindo mês a mês). Benchmark de segmento (padrões de rejeição por tipo de IF) como tier premium com opt-in de dados. **O motor não é o moat; a taxa de aprendizado por evento é** — o motor pode ser replicado em um ano; o histórico de eventos de dezenas de IFs não.

### (e) Operação ponta-a-ponta até o CRD/STA

Gerar → validar (compatível com BCValidador + regras próprias) → remeter (STA REST + senhaws; hosts verificados: `sta.bcb.gov.br/staws` produção, `sta-h.bcb.gov.br/staws` homologação, `www3/www9.bcb.gov.br/senhaws`) → ler retornos do CRD → re-ingestir como eval (fecha o pilar d). Credenciais Sisbacen (transação PSTA300) são **requisito de dia 1**, não sprint 29 — sem elas o produto é demonstração para nós mesmos. Primeiro caso de uso do piloto: documento zerado DRSAC via CRD (baixa barreira, obrigatório, dor real).

---

## 7. North Star metrics + kill criteria v2

### 7.1 North Star

> **Remessas ponta-a-ponta aceitas por mês**: CADOC gerado pelo motor, validado, remetido via STA e **aceito no CRD por uma IF real**. É a única métrica que só cresce se os cinco pilares funcionarem juntos — e não pode ser falsificada por código.

### 7.2 Métricas de suporte

| Métrica | Meta | Base |
|---|---|---|
| T2U (time-to-update) | ≤10 dias úteis norma→leiaute, medido em cada onda | Pilar (b); comparável ao retrabalho que a IN 733 impôs ao mercado |
| Learning rate | Evals novos/mês derivados de eventos reais; accuracy do suite em tendência mensal | Pilar (d) |
| Entidades remetentes ativas | CNPJs com remessa no mês (base do usage pricing e da NRR) | Ladder de preços pesquisa/08 |
| MRR m12 (dez/2027) | Conservador ~R$76k · Base ~R$210k (45 clientes) | Projeção pesquisa/08 §6.3 |
| Unit economics | Churn ≤0,8%/mês · NRR ≥115% · GM ≥88% · payback ≤12 meses | Benchmarks pesquisa/08 §5 |

### 7.3 Kill criteria (honestos — todos exigem evidência externa)

| # | Quando | Critério de morte | Consequência |
|---|---|---|---|
| K1 | 31/mar/2027 | <5 IFs reais com ≥1 remessa ponta-a-ponta aceita | Tese de SaaS morta → pivotar para componente: engine licenciado a casas de software |
| K2 | 30/jun/2027 | <10 clientes pagantes OU MRR <R$50k | Pricing/GTM errados → reprecificar, trocar canal (consultorias no lugar de PLG) |
| K3 | 30/set/2027 | T2U mediano >10 dias úteis em 2 ondas normativas consecutivas | O SLA é promessa falsa → removê-lo do pitch; reposicionar em corretude |
| K4 | 31/dez/2027 | Eval suite flat (accuracy sem melhoria por 2 trimestres) com ≥30 eventos/IF | O flywheel é narrativa → moat volta a ser cobertura de leiaute; correr cobertura |
| K5 | 28/fev/2027 | 0 IFs pagantes vindas da janela DRSAC (remessa vencida ~fev/2027) | Urgência normativa não converte → matar GTM de janela; GTM por varredura de consultorias |

**Regra geral:** nenhuma métrica conta sem evidência externa (protocolo CRD/STA, contrato, extrato de cobrança, relatório de eval versionado com hash). Nenhuma milestone fecha com merge.

---

## 8. Calendário GTM 2026–2028 (ancorado em normativos verificados)

> **Normas pós-fusão (16/11/2026):** o calendário abaixo não inclui as 6 normas incorporadas pela fusão — IN 729/2026 (CNPJ alfanumérico), IN BCB 735/2026 (Desenrola 2.0), Res. CVM 247/2026 (FIDC, 9 dias úteis), RC Conjunta 18/2025 (conformidade 31/12/2026), RC Conjunta 16/2025 (BaaS) e Comunicados BCB 45.206/45.339/2026 (MFA Autran). **`00-INDEX.md` §8 é a fonte canônica** da tabela normativa completa.

| Período | Evento regulatório (verificado 16/11/2026) | Movimento de produto/GTM |
|---|---|---|
| **nov/2026 (agora)** | Onda 3 da IN BCB 733 (NR3: "Limite sujeito à provisão", Anexo 8) vigente na data-base nov/2026 | Congelar repo Go como arquivo v1 + fonte de dados; capturar leiaute 3040 nov/2026 como primeiro dataset do port Rust; iniciar obtenção de credenciais PSTA300; pesquisa de marca/INPI |
| **dez/2026** | **Data-base DRSAC dez/2026** sob leiaute IN 694 (primeira remessa vence ~fev/2027); GRSAC e LCR/LCRS a 30 dias | **Piloto DRSAC via FASE 1:** o piloto com **5–8 IFs S1–S4** (beta fechado via FBM/consultorias) roda o **reg-cadoc 2030 em Rust** (S4 do wedge — leiaute oficial IN 694/2025 re-baixado), com chancela do BacenValidatorAdapter (B02 — ADR-0018) e revisão humana obrigatória (críticas semânticas do 2030 não são públicas); o wiring do gen2030 em Go foi **revogado** (retrabalho — gen2030 era fictício); a plataforma Rust da FASE 2 constrói em paralelo. **CORREÇÃO 18/11/2026:** o S4 da FASE 1 entregou o **gen3040** (CMN 4.966 — commit 8124fcb); o **gen2030 é BLOQUEIO DECLARADO** (re-download do leiaute 2030 pendente — 00-INDEX §10.1): o piloto DRSAC só fecha com esse bloqueio resolvido |
| **31/12/2026** | Prazo RC CMN/BCB 14/2025 (capital social mínimo — ~679 IFs se adequando) | Conteúdo/comunidade: urgência de compliance além de CADOC |
| **1º/1/2027** | **GRSAC novo** (Res BCB 586 + IN 772, revogam Res 139/2021 e IN 153/2021) e **LCR/LCRS 90%** (CMN 5.296 + BCB 560; S2 passa a calcular LCR; S3/S4 LCRS) | Webinar ABBC/ABFintechs "as 4 dores de jan/2027"; conteúdo SEO de GRSAC/LCRS (keywords sem dono) |
| **Q1/2027** | — | **Launch Rust**: Free Validador (PLG) + pricing público + marca nova; DRSAC completo no Pro; add-on DRSAC no Starter (R$990) para S1–S4 pequenas na janela |
| **~fev/2027** | **Primeira remessa DRSAC vence** (10º dia útil do 2º mês subsequente à data-base dez/26) | Pico de conversão da janela; caso de uso "documento zerado via CRD" para quem não tem exposição |
| **jun/2027** | **2ª data-base DRSAC** (remessa vence ~ago/2027) + fim da transição: códigos 01–04 do 2030 deixam de ser aceitos em registros novos | Retarget das mesmas ~455 IFs S1–S4 + IPs; início do upsell DRL 2160 (leiaute novo da IN 755) para S2–S4 |
| **1º/7/2027** | **LCR/LCRS 100%**; **FRTB padronizado** vigente em 2027 (Res BCB 470/2025 + 592/2026) | Módulo de leiautes 2027 pronto (família 2036/2060); 2–3 parceiros consultoria ativos com rev-share |
| **Q3/2027** | Compulsório-poupança: dedução +1,5 p.p./ano (Res BCB 512/2025) pressiona marcação no 3040 | Benchmarking como tier premium (após ~30 clientes contribuintes); caso "zero autuação" documentado |
| **Q4/2027** | — | Primeiro Enterprise S3-S4; meta MRR base R$210k (conservador R$76k); avaliar K1–K5 |
| **2028** | **DLO 100% em jan/2028** (IN 750: 50%/2026→75%/2027→100%/jan/2028); pisos de provisionamento (CLFPS) = 0 em 1º/1/2028; output floor em fase final | O motor nativo em rules-as-data absorve ondas sem retrabalho — argumento de defesa/renovação; white-label BaaS como canal de expansion |
| **contínuo** | 3060 mensal (novo, cartões); 3044 diário/eventual; 5710/5711 cripto (VASPs); 2090 ICAAP/TEBU | Demanda contínua de time-to-update = showcase permanente do harness |

### Leitura do calendário

1. **O reg-cadoc em Rust (FASE 1) é o que protege a janela.** A primeira remessa DRSAC sob o leiaute novo vence ~fev/2027; a plataforma da FASE 2 não estará pronta antes disso. O piloto dez/2026 roda o **reg-cadoc 2030 em Rust** (S4 do wedge, contra o leiaute oficial re-baixado) com chancela do BacenValidatorAdapter (ADR-0018); o wiring do gen2030 em Go foi revogado por retrabalho (gen2030 era fictício). Repo v1 não é legado descartável: é arquivo + fonte de dados do port. **Atualização 18/11/2026:** o gen3040 saiu (S4 virou 3040); o gen2030 segue BLOQUEIO DECLARADO (00-INDEX §10.1) e é o caminho crítico da janela.
2. **A janela DRSAC é urgência com prazo fixo** — a única urgência que procurement não adia. O piloto de dez/2026 não precisa de produto completo; precisa de geração+XSD+humano no loop.
3. **2027 é o ano da dor** (GRSAC, LCR/LCRS 90%→100%, FRTB, compulsório +1,5 p.p., transição 2030): o GTM vende alívio de quatro dores simultâneas para o mesmo comprador — Head de Compliance/CTO de S2–S4 e fintechs.
4. **2028 muda o argumento de venda:** de "mude rápido quando a norma muda" para "mude rápido sempre" — DLO 100%, pisos = 0, output floor em fase final; o harness com histórico de 2 anos de eventos é a prova.

---

## Consistência com a arquitetura-alvo

- **Repos:** este repo vira `archive-v1` + fonte de dados (catálogo 1.099, leiautes, git log como registro histórico); reconstrução Rust em repo novo, 16 crates, axum + sqlx + Postgres, frontend Next.js mantido.
- **Motor:** as promessas da tese v2 (§6a) são exatamente os mecanismos decididos na arquitetura — enum exaustivo, dispatch fechado, catálogo verificado em build, provenance universal.
- **IA:** determinismo no engine + IA na fronteira com verificação objetiva (§6c) — sem exceção, sem "modo autopilot".
- **Harness:** flywheel evento→eval→melhoria (§6d) é o componente defensível central; o restante da arquitetura existe para torná-lo confiável.

*Documento gerado em 16/nov/2026. Todas as referências normativas foram verificadas nas páginas oficiais do BCB na data (pesquisa/06); contagens de IFs extraídas da API Olinda na data (pesquisa/08 §1.1) — re-executar no dia do board.*
