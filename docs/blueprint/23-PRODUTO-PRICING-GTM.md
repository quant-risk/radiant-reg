# 23 — PRODUTO, PRICING & GTM: Radiant Norma v2

> **Data:** 16/nov/2026 · **Status:** decisão — substitui o pricing e o GTM da tese v1 (jul/2026).
> **Fontes:** `research/08-produto-pricing.md`, `research/05-concorrentes-mercado.md`, `research/06-normativos-realidade.md`, `research/02-tese-vs-realidade.md` (todas de 16/nov/2026); coerente com `11-TESE-V2.md` (tese) e `20/21/22` (arquitetura/IA/harness).
> **Âmbito:** mercado endereçado com números oficiais · ladder de preços final · autopsy do pricing antigo · projeção MRR 12m em 3 cenários · GTM híbrido e canais · posicionamento de procurement · decisão de marca · jogada DRSAC · playbook de vendas.

---

## 0. Sumário executivo

1. **Mercado:** 1.722 entidades autorizadas (API Olinda BCB, 16/11/2026), das quais ~1.100 são remetentes SCR endereçáveis (SAM ≈ R$600 mi/ano). O DRSAC novo (IN BCB 694/2025) obriga **S1–S4, inclusive IPs** (~455 entidades + IPs) a partir da data-base dez/2026 — janela de conversão com prazo regulatório fixo: 1ª remessa vence ~fev/2027.
2. **Pricing:** 5 tiers — Free Validador R$0 (wedge PLG), Starter R$2.490, Pro R$6.900, Scale R$16.900, Enterprise R$29.900+ — mais 3 add-ons (+CNPJ R$590, +50k ops R$190, DRSAC R$990). Os 4 gates de upgrade (entidades, volume, DRSAC, white-label/benchmark) são a máquina de **NRR ≥115%**. Preço público é arma: os 4 concorrentes diretos são todos quote-gated. **Vige na FASE 2** — a FASE 1 tem régua própria de wedge (§2.0, fusão 16/11/2026).
3. **Pricing antigo (R$1,5k/4,5k/12k) deixava 2–8x na mesa:** abaixo do Matera (R$30k+/ano), da consultoria manual (R$3–15k/mês) e do risco que mitiga (multa BCB até R$250k + 50%, Lei 13.837/2019) — e sinalizava "brinquedo" em procurement.
4. **Projeção MRR m12 (dez/2027):** conservador **R$76k** · base **R$210k** (45 clientes) · agressivo **R$430k** — premissas linha a linha na seção 4.
5. **GTM híbrido:** PLG-assisted para SCD/IP (ciclo 15–45 dias, CAC R$6–12k); sales-led para bancos/BaaS via consultorias (ciclo 6–12 meses, CAC R$80–150k); canais FBM, ABBC, ABFintechs/Zetta e rev-share 20% com consultorias CADOC. **"Parceria BACEN" não existe** — a aprovação de facto é o arquivo passar no CRD.
6. **Marca:** não lançar como "Norma" (colisão com cadoc.ai). 7 candidatos BR-nativos; busca de anterioridade INPI obrigatória antes de Q1/2027.

---

## 1. Mercado endereçado

### 1.1 Universo oficial de entidades autorizadas (BCB, 16/11/2026)

Contagem extraída em tempo real da API pública OData `Instituicoes_em_funcionamento` (Olinda, metadados atualizados 11/nov/2026):

| Segmento | Qtd. (sedes) | Obrigação CADOC relevante |
|---|---:|---|
| Bancos/Caixa (múltiplo, comercial, filiais estrangeiras, múltiplo cooperativo, câmbio) | **154** | contábil (4111/COSIF) + SCR + Produb |
| Cooperativas de crédito (singulares + centrais + confederações) | **736** | contábil + SCR 3040/3050 |
| Instituições de Pagamento (IP) | **194** | DIP 3501, Doc 3044, DRSAC (S1–S4) |
| Sociedades de Crédito Direto (SCD) | **138** | SCR 3040/3050 + 3044 + contábil |
| Financeiras (SCFI) | 80 | SCR |
| Distribuidoras (DTVM) | 101 | contábil |
| Corretoras (CTVM 57 + câmbio 47) | 104 | contábil / câmbio |
| Administradoras de consórcio | 129 | documento próprio |
| Arrendamento (16) + hipotecárias (5) + fomento (16) | 37 | SCR |
| Bancos de investimento (12) + desenvolvimento/BNDES (4) + APE (1) | 17 | contábil + SCR |
| SCM (21) + SEP (11) | 32 | SCR |
| **TOTAL** | **1.722** | |

**Leitura estrutural:** o volume regulatório do Brasil é long tail — 90% das entidades são pequenas e médias que não têm equipe de relatórios. O grosso do dour escravo (planilha + consultoria improvisada) está exatamente no SAM abaixo.

### 1.2 Segmentação prudencial S1–S5 (base do cronograma DRSAC)

Resolução CMN 4.966/2021; distribuição compilada de dados BCB:

| Segmento | Critério (síntese) | Qtd. | % ativos do sistema |
|---|---|---:|---:|
| S1 | ≥10% do PIB / internacionais | **6** | 63,93% |
| S2 | 1–10% do PIB | **10** | 16,17% |
| S3 | 0,1–1% do PIB | **65** | 12,16% |
| S4 | <0,1% do PIB | **374** | 5,02% |
| S5 | pequenas (incl. cooperativas singulares) | **806** | 2,72% |
| **Total segmentadas** | | **1.261** | |

A lista nominal por segmento é pública (BCB) e serve de base de prospecção pronta: 455 instituições em S1–S4 são a população obrigada ao DRSAC (S5 dispensado — nuance crítica: **não vender DRSAC para cooperativa singular S5**, que é ICP de 3040/4111, não de 2030).

### 1.3 SAM — os ~1.100 remetentes SCR endereçáveis

| Camada | Definição | Qtd. IFs | Valor potencial |
|---|---|---:|---|
| **TAM** | Todas as entidades com obrigação de remessa contábil/estatística ao BCB | ~1.400–1.500 | preço médio R$60k/ano ⇒ **~R$0,9–1 bi/ano** |
| **SAM** | Remetentes SCR endereçáveis pelo v2: SCD, SCM, SCFI, SEP, IP, DTVM/CTVM/SCC, Afo/CH, cooperativas (S3–S5) | **~1.100** | preço médio R$55k/ano ⇒ **~R$600 mi/ano** |
| **SOM 12 meses** | Penetração realista pós-launch Q1/2027 | 20–85 clientes | **R$0,9–5,2 mi ARR** (seção 4) |

População obrigada ao DRSAC (recorte do SAM): **S1–S4 = 455 instituições + IPs enquadradas em S1–S4**, datas-bases semestrais (jun/dez). FIDCs (~3.000) são TAM indireto, acessados via administradores DTVM — não ICP do v2 (revisado pela fusão 16/11/2026: a **cunha FIDC** — §5.5 — promove o administrador de FIDC a comprador direto do Dossiê).

### 1.4 Consolidação: o mercado encolhe em número e cresce em valor

| Indicador | 2021 | 2025/2026 |
|---|---:|---:|
| Fintechs de crédito (SCD+SEP) | 74 | **149** (138 SCD + 11 SEP) |
| IPs ativas | — | **194** (pico >200; endurecimento Res. BCB 494/2025) |
| Associadas ABFintechs | — | **724 → 588** (início 2025 → meados 2026) |
| IFs se adequando ao capital mínimo (RC CMN/BCB 14/2025) | — | **679** até 31/12/2026 |
| Fintechs no Brasil (total) | ~1.158 (2020) | ~2.100 (2025) |

**Leitura estratégica:** o BCB endureceu licenciamento e as sobreviventes ficam maiores, mais reguladas e com menos gente (custo de operar IF licenciada ~US$150 mil/mês). Consolidação 724→588 não é encolhimento de demanda — é encolhimento de amadores: sobra demanda por ferramenta profissional que substitua consultoria improvisada em quem opera enxuto. Favorece SaaS com preço público e onboarding imediato sobre projeto de implantação.

---

## 2. Pricing final

### 2.0 Fase 1 — Monetização do wedge (nov/2026–2027) — *adicionada pela fusão (16/11/2026)*

Antes do ladder SaaS (§2.1–2.5, que vige no launch Rust da FASE 2), a receita da FASE 1 é o **wedge on-prem "Radiant Reg"** (25-ROADMAP §0), com esta régua:

| Oferta | Preço | Notas |
|---|---|---|
| **PoC 14 dias** (dados mascarados) | **R$ 0** | trava de 14 dias, **sem chave de produção** — prova leiaute/ambiente, não vira produto |
| **Sprint de implantação** (4 semanas) | **R$ 35.000–60.000** one-off | FIDC (Res. CVM 247/2026 + RC Conjunta 18/2025), SCD/IP 3040, IF 2030/RC 18 |
| **Licença Reg Core/ESG** | **R$ 6.000–12.000/mês** (contrato anual) | ACV R$ 72–144k; **+R$ 450–800/fundo extra** para FIDC |
| **Licença Enterprise** | **R$ 25.000–45.000+/mês** | bancos S3-S4, BaaS, centrais de cooperativas |

**Unit economics do wedge:** margem de contribuição **~85–89%**, payback **1,7–3,5 meses**, **LTV/CAC 12–17×**, CAC **R$ 18–25k** — fechado pela implantação (o sprint é o CAC). **Racional:** TPRM (CMN 4.893/5.274) e a LC 105 empurram a execução in-perimeter para o primeiro contrato — a venda é licença on-prem assinada, não assinatura SaaS; **Stripe/billing SaaS volta na Fase 2**.

### 2.1 Tabela de tiers (vigência: launch Rust da FASE 2 — Q1/Q2 2027)

| | **Free — Validador** | **Starter** | **Pro** | **Scale** | **Enterprise** |
|---|---|---|---|---|---|
| **Preço mensal (anual)** | R$0 | **R$2.490** | **R$6.900** | **R$16.900** | **a partir de R$29.900** |
| Preço mês a mês | R$0 | R$2.990 | R$8.280 | R$19.900 | sob contrato |
| **Entidades remetentes (CNPJ)** | 1 | 1 | até 3 | até 10 | ilimitado/negociado |
| **Operações SCR (3040/3050)/mês** | 1 mil | 50 mil | 200 mil | 1 mi | ilimitado |
| Documentos | 3040 (gera+valida, **envio manual**) | 3040, 3050, 3044, DIPs core, DRSAC "zerado" | + DRSAC 2030 completo, dossiês, **API** | + todos (SARO/DAS/CAS, Produb, consórcios), multi-cadastro consolidado | + segmentações custom, integrações core |
| **BCValidador** (leiaute oficial) | Sim | Sim | Sim | Sim | Sim |
| API / webhooks | — | — | Sim | Sim | Sim |
| White-label p/ tomadores BaaS | — | — | — | **Sim** | Sim |
| Benchmark & Inteligência (data moat) | — | — | básico | **completo** (opt-in de dados) | completo |
| SLA | — | 99% | 99,5% | **99,9% + Zero Autuação™** | custom |
| Usuários | 1 | 5 | 10 | ilimitado | ilimitado + SSO/SCIM |

### 2.2 Add-ons (usage/expansion)

| Add-on | Preço | Quem compra |
|---|---|---|
| Entidade extra (CNPJ) | **R$590/mês** | BaaS, grupos com filiais, cooperativas centrais |
| Volume SCR extra (bloco de 50k ops/mês) | **R$190/mês** | SCD/IP crescendo carteira |
| DRSAC completo no Starter | **R$990/mês** | S1–S4 pequenas na janela dez/26–ago/27 |
| Benchmark Premium (standalone p/ não-clientes) | R$1.490/mês | bancos que só querem o dado |
| Implantação/homologação assistida | R$15–50k one-off | S3–S4 (Enterprise) |

### 2.3 Por que o Free é o wedge PLG (e não canibaliza)

1. **O BCValidador oficial é gratuito, Java, ZIP de ~2,7 MB, documentação de 2010/2013** — validar leiaute é commodity; cobrar por isso perde para o próprio BCB. Então damos **mais** de graça que o BCB: validação de 3040 com regras semânticas do catálogo, no browser, sem instalar JRE — e capturamos o e-mail do responsável regulatório no momento exato da dor.
2. **Limites desenhados para o upgrade, não para a dor:** 1 entidade, 1 mil ops, sem remessa (envio manual), sem API. O SCD que cresce bate no teto de volume; o grupo com 2 CNPJs bate no teto de entidades; quem odeia reenviar manualmente sobe para o Starter. Free converte pelo teto, não pelo marketing.
3. **Funil qualificado por uso:** quem validou 3040 três meses seguidos é lead 10x mais quente que download de e-book. Meta de conversão free→pago: 3% em 90 dias (cenário base, seção 4).

### 2.4 Os 4 gates como máquina de NRR ≥115%

| Gate | Gatilho de upgrade | Receita incremental típica | Janela típica |
|---|---|---|---|
| Entidades (CNPJ) | BaaS/grupo adiciona tomador ou filial | +R$590/entidade/mês | 3–9 meses |
| Volume SCR | carteira cresce além do teto | +R$190 por bloco de 50k/mês | 6–12 meses |
| DRSAC completo | S1–S4 no Starter precisa do 2030 full | +R$990/mês ou upgrade Pro (+R$4.410) | dez/26–ago/27 |
| White-label/benchmark | BaaS revende / quer o dado de segmento | upgrade Scale (+R$10k/mês) ou Benchmark R$1.490 | 9–18 meses |

**Matemática do cohort (prova do target):** 100 logos Starter = R$2,988 mi de ARR ano 1. No ano 2: 45% adicionam DRSAC (+R$534,6k), 25% adicionam 1 CNPJ (+R$177k), 15% adicionam 2 blocos de volume (+R$68,4k); churn logo 10%/ano (−R$298,8k). NRR do cohort = 3.469,2/2.988 = **116%**. Isto é consistente com a evidência de mercado: NRR global mediana 101%, enterprise 118%; usage-based é a via comprovada para NRR >120% ("quase impossível com subscrição flat" — Lago/OpenView/Metronome 2025). **Regra de design: desconto nunca é dado por dinheiro — é dado como "crédito de dados"** (opt-in de dados anonimizados para o benchmark), protegendo o data moat e criando goodwill em procurement.

### 2.5 Princípios de pricing

1. **Preço público sempre** — os 4 concorrentes diretos (Matera, Mitra, cadoc.ai, Dattos) são quote-gated; preço na página encurta procurement e sinaliza confiança. Enterprise mantém ancla pública ("a partir de R$29.900") com contrato sob medida.
2. **ARPU BR ~20–35% do equivalente americano** é a faixa operável (RegTech de reporte nos EUA: Duco/Trintech ~US$180k/ano; Workiva mediana US$49k; Vanta ~US$20k). A vantagem do entrante é lucrar nessa faixa (infra Rust de custo trivial) e subir ARPU por usage + módulos — nunca por lista inicial inflada.
3. **Outcome como SLA, não como success fee:** "Zero Autuação™" é garantia contratual com crédito/reembolso. Success fee em compliance desperta suspeita de independência em procurement; SLA de-riska a compra.
4. **Valor que cresce com o cliente:** entidades e volume são métricas de valor (o cliente cresce, paga mais) — não métricas de custo nosso (validação é CPU-bound barata; GM ≥88%).

---

## 3. Por que o pricing antigo (R$1,5k/4,5k/12k) deixava 2–8x na mesa

### 3.1 As referências de valor que o pricing antigo ignorava

| Referência de valor | Quanto custa | Pricing antigo capturava | Deixado na mesa |
|---|---|---|---|
| Matera (RegTech equivalente, 250+ IFs) | **R$30k+/ano** | Lite R$18k/ano | ≥1,7x já no entry |
| Consultoria compliance terceirizada | **R$3–15k/mês** (R$36–180k/ano) | Lite/Pro R$18–54k/ano | **2–3x** |
| WTP observada — SCD pequena | R$60–120k/ano | R$18k | **3,3–6,7x** |
| WTP observada — IP | R$80–200k/ano | R$54k (Pro) | **1,5–3,7x** |
| WTP observada — banco S3-S4 | R$250k–1 mi/ano | R$144k (Scale) | **1,7–7x** |
| Custo do erro — autuação BCB | **R$250k + 50%** por ato (Lei 13.837/2019, art. 11; Res. BCB 131/2021 — processo vinculado; régua antiga Res. 2.194: R$150/dia útil) | R$0 (sem SLA de risco) | o produto mais barato da esteira não transferia o risco de R$375k |

### 3.2 Os três defeitos estruturais do ladder antigo

1. **Entry abaixo do custo da alternativa manual.** R$1,5k/mês contra uma consultoria que custa R$3–15k/mês pelo mesmo trabalho não é desconto — é **âncora de "brinquedo"**. Procurement bancário trata preço muito abaixo do benchmark como risco (fornecedor que não sobrevive, sem SLA, sem time). O R$2.490 do novo entry é metade da consultoria mais barata e menor que um analista júnior — impulsivo para o gestor de SCD, mas sério para o comprador.
2. **Sem usage, sem add-ons, sem NRR.** O ladder antigo tinha preço fixo por tier e nenhuma alavanca de expansão; com compliance obrigatório, o churn logo tende a ser baixo, mas a receita fica plana (NRR ~100%). O novo ladder tem 4 gates mensuráveis (seção 2.4) — o cliente cresce, a conta cresce, sem renegociação.
3. **Vendava "geração" sem transferir risco.** O produto mais barato da esteira deixava a multa de R$375k inteiramente no cliente. O novo empacota **Zero Autuação™** no Scale+ — transforma o risco regulatório no argumento de compra ("pague R$29,9k/mês para que a autuação por erro de leiaute seja problema do fornecedor").

**Síntese:** o pricing antigo precificava a feature (gerar XML); o novo precifica o resultado (remessa aceita no CRD, janela normativa cumprida, risco transferido). Entre R$18k e R$60–120k/ano de WTP verificada, havia 2–8x escondidos — e a banda de preços dos concorrentes (R$3–8k bundle BaaS até R$50k+ Matera/Mitra) confirma que o mercado já paga o novo ladder sem pestanejar.

---

## 4. Projeção MRR 12 meses — 3 cenários com premissas por linha

### 4.1 Premissas comuns

**M0 = mar/2027 (L1: Free + Starter only)**; ladder completo no L2 jun–jul/2027; **beta DRSAC desde dez/2026** (janela: 1ª remessa vence ~fev/2027, 2ª ~ago/2027); churn aplicado mês a mês; preços de lista anual (desconto de lista já embutido nos ARPU). **A receita do wedge 2026 (licenças on-prem — PoC/Sprint/Licença, §2.0) fica fora do MRR SaaS, em linha própria** — o MRR abaixo mede apenas a plataforma (FASE 2).

### 4.2 Premissas por linha (drivers)

| Premissa | Conservador | Base | Agressivo |
|---|---:|---:|---:|
| Novos clientes/mês (bruto) | 2 | 4 | 8 |
| Churn logo mensal | 1,5% | 1,0% | 0,8% |
| Registros no Free Validador até m12 | ~300 | ~600 | ~1.000 |
| Conversão Free→pago (90 dias) | 2% | 3% | 4% |
| Ciclo SCD/IP (PLG-assisted) | 45 dias | 35 dias | 30 dias |
| Parceiros consultoria ativos (rev-share 20%) | 0–1 | 2 | 3 |
| White-label BaaS no ar | 0 | 1 | 2 |
| Enterprise S3–S4 fechado até m12 | 0 | 1 | 2–3 |
| Attach add-on DRSAC (Starter S1–S4) | 10% | 25% | 35% |

### 4.3 Cenários — clientes ativos no m12 (dez/2027)

| Cenário | Clientes m12 | Mix (Starter/Pro/Scale/Ent) | ARPU médio | **MRR m12** | ARR anualizado |
|---|---:|---|---:|---:|---:|
| **Conservador** | 20 | 80% / 14% / 5% / 1% | ~R$3,8k | **~R$76k** | ~R$0,9 mi |
| **Base** | 45 | 75% / 17% / 6% / 2% | ~R$4,7k | **~R$210k** | ~R$2,5 mi |
| **Agressivo** | 85 | 68% / 20% / 9% / 3% | ~R$5,1k | **~R$430k** | ~R$5,2 mi |

### 4.4 Decomposição por linha de receita no m12 (arredondada)

| Linha | Conservador | Base | Agressivo |
|---|---:|---:|---:|
| Starter (nº × R$2.490) | 16 → R$39,8k | 34 → R$84,7k | 58 → R$144,4k |
| Pro (nº × R$6.900) | 3 → R$20,7k | 8 → R$55,2k | 17 → R$117,3k |
| Scale (nº × R$16.900) | 1 → R$16,9k | 3 → R$50,7k | 7 → R$118,3k |
| Enterprise (nº × R$29.900+) | 0 → R$0 | 1 → R$29,9k | 2 → R$59,8k |
| Add-ons (CNPJ/volume/DRSAC) | ~R$1k | ~R$9k | ~R$19k |
| **Total bruto do mix** | **R$78,4k** | **R$229,5k** | **R$458,8k** |
| **MRR m12 declarado** (timing de entrada, churn no meio do ano, descontos anuais) | **R$76k** | **R$210k** | **R$430k** |

*A decomposição é snapshot de mix no m12; os totais declarados refletem a rampa real (clientes que entram tarde no ano pesam fração do MRR no fechamento). A diferença entre cenários é o que importa, não a terceira casa decimal.*

### 4.5 Rampa de MRR por trimestre

M0 = mar/2027 (L1 abre Free+Starter; ladder completo no L2 jun–jul/2027). A receita do wedge 2026 (licenças — §2.0) não entra nesta rampa.

| | M0 (mar/27) | M3 (jun/27) | M6 (set/27) | M9 (dez/27) | M12 (fev/28) |
|---|---:|---:|---:|---:|---:|
| Conservador | 5k | 18k | 38k | 60k | **76k** |
| Base | 9k | 42k | 95k | 160k | **210k** |
| Agressivo | 15k | 75k | 165k | 290k | **430k** |

### 4.6 Alavancas base → agressivo (e por que 45 clientes é defensável)

1. 2–3 parceiros consultoria ativos com rev-share (cada um traz 1–2 fechamentos/mês na janela DRSAC);
2. 1 white-label BaaS no ar (expansion via rede de tomadores, não via vendas);
3. peg-on de DRSAC na janela (add-on R$990 × 25–35% da base Starter S1–S4);
4. 1 banco S3–S4 Enterprise (ciclo 6–12 meses fechado em Q4/2027).

**Defensabilidade de 45 clientes em 12 meses:** SAM de ~1.100 IFs; cadoc.ai, Dattos e Regulatório Mais já venderam nessa banda de preço sem produto completo de remessa; ciclo SCD/IP de 15–45 dias com PLG-assisted; e a janela DRSAC cria urgência com prazo regulatório fixo — a única urgência que procurement não adia. Cross-check com kill criteria da tese v2: K2 (<10 clientes ou MRR <R$50k até 30/jun/2027) e K5 (0 clientes da janela DRSAC até 28/fev/2027) são os disjuntores honestos desta projeção.

---

## 5. GTM híbrido

### 5.1 Motion por persona

| Persona | Motion | Ciclo realista | CAC alvo | Payback |
|---|---|---|---:|---|
| SCD/IP/DTVM pequena | **PLG-assisted**: Free → Starter self-serve com assistência | 15–45 dias | **R$6–12k** | ~2–3 meses (ARPU R$2,5–6,9k) |
| Fintech BaaS / multi-CNPJ | Sales-assisted (1 AE) | 2–4 meses | R$15–30k | ~3–5 meses |
| Banco S3–S4 / cooperativa central | **Sales-led + canal consultoria** | 6–12 meses (procurement, pentest, DD de terceiros) | **R$80–150k** | ~4–6 meses (R$29,9k+/mês) |

Benchmark de controle: mediana global de CAC é US$2,00 de S&M por US$1 de nova ARR e payback de 16 meses — o mix acima fica abaixo disso porque o PLG carrega o entry e o canal de consultoria não tem custo fixo de AE para pequenas. Ciclo enterprise regulado BR: 6–18 meses (homologação técnica + compliance de fornecedor são os alongadores).

### 5.2 Canais (com o que já está provado em cada um)

1. **FBM Educação** — autoridade de conteúdo em CADOCs (curso "Especialista em CADOCs" para contadores/riscos/compliance): a audiência exata do ICP, já organizada. Jogada: aulas guest, co-marketing de INs, certificação conjunta. Meta: 1 parceria ativa até fev/2027.
2. **ABBC** (educação executiva, análise técnica da IN 694) — webinar conjunto jan/2027: "as 4 dores de 2027" (GRSAC, LCR/LCRS, DRSAC, ondas 3040). Meta: 1 webinar + 1 artigo co-branded.
3. **ABFintechs (588 associadas) e Zetta** — eventos, comitês, fila de indicação; ~2.100 fintechs no país. Meta: 1 presença Q2/2027, 3 SQLs por evento.
4. **Consultorias CADOC/compliance (ARR, Merc Group, boutiques tipo B3Bee)** — **rev-share 20% do 1º ano**. Pitch correto para o parceiro: a ferramenta não corta a tarifa deles — elimina as horas mortas de leiaute/planilha e multiplica a tarifa estratégica por hora. Meta: 2–3 parceiros ativos até Q2/2027.
5. **BaaS providers como canal white-label** (tier Scale) — o BaaS empacota o regulatório do tomador; cada tomador é entidade cobrada. Conflito a explorar: Regulatório Mais (Celcoin) não pode ser canal de BaaS concorrentes — nós somos neutros. Meta: 1 white-label no ar até Q4/2027. **Nota da fusão (white-label = FASE 2):** a **RC 16/2025** (BaaS, vigência 28/11/2025, transição de contratos até 31/12/2026) — fim das contas-bolsão e **responsabilidade integral da prestadora** — re-posiciona o tier white-label como oferta da FASE 2; **conferir o literal na página oficial antes de virar requisito contratual**.
6. **SEO/LinkedIn técnico** — keyword "CADOC 3044" está tomada por posts de blog (Dattos, Matera, Sydle, FBM); abrimos keywords **sem dono**: "validador CADOC online", "gerador 3040", "preencher DRSAC 2030", "diff de leiaute", "documento zerado CRD". cadoc.ai provou que conteúdo regulatório gera audiência ICP no BR.

### 5.3 Por que "partner BACEN" não existe (e o que existe no lugar)

- **Não há programa de homologação, certificação ou parceria de software de CADOC no BCB.** O validador oficial é o BCValidador — gratuito, distribuído em ZIP. Não existe marketplace nem selo de fornecedor aprovado. **Investir em "certificação BACEN" é queimar dinheiro** (risco já mapeado na pesquisa: §7.4 de 08 e §5 de 11-TESE-V2).
- **A aprovação de facto é o CRD:** arquivo passa no validador, remessa aceita, histórico limpo. Reputação regulatória se constrói remessa a remessa — é exatamente por isso que o North Star da tese v2 é "remessas aceitas por mês", e por isso o caso de abertura do piloto é o documento zerado DRSAC via CRD (obrigatório para quem não tem exposição, baixa barreira, dor real).
- **O que procurement bancário pede de verdade em due diligence de terceiros:** adesões setoriais (ABBC, ABFintechs, Zetta) como credibilidade de canal + **ISO 27001/SOC 2 Type II + LGPD** na régua do comitê de segurança + audit trail verificável (hash-chain do motor). Isso sim entra no orçamento; "parceria BACEN", não.

### 5.4 Calendário GTM 12 meses (síntese)

1. **Dez/2026:** beta fechado DRSAC com 5–8 IFs S1–S4 (movimento 1 da seção 8).
2. **Q1/2027:** launch Rust — Free Validador, pricing público, marca nova, 2 parceiros consultoria.
3. **Q2/2027:** conteúdo SEO/LinkedIn + webinar ABBC/ABFintechs; 2–3 parceiros ativos; primeiro white-label em negociação.
4. **Q3/2027:** benchmark como tier premium (após ~30 contribuintes); caso "zero autuação" documentado.
5. **Q4/2027:** primeiro Enterprise S3–S4; meta base R$210k MRR; avaliação de K1–K5.

### 5.5 A cunha FIDC — Res. CVM 247/2026 + Res. Conjunta 18/2025 — *adicionada pela fusão (16/11/2026)*

- **Res. CVM 247/2026** (verificada no PDF oficial: vigente **1/11/2026**): administradores de FIDC passam a responder por **prazo unificado de 9 dias úteis** pós-encerramento do mês; o atraso expõe **multa cominatória diária** — via art. 56-A do Anexo II da Res. CVM 175; **gestores ficam autorizados a consultar o SCR**; e as **Securitizadoras entram no SCR** com vacatio até **31/10/2027**.
- **Res. Conjunta 18/2025** (política de qualidade de dados, **12 dimensões**): conformidade integral em **31/12/2026** — 45 dias a contar de 16/11/2026, data deste blueprint.
- **Entregável de entrada:** o **Dossiê A4/JSON assinado Ed25519** — linhagem campo-a-campo + prova criptográfica — é o entregável que abre a reunião: o administrador de FIDC precisa provar a qualidade dos dados que remete, e o dossiê assinado é a evidência (régua de preço em §2.0).

---

## 6. Posicionamento e mensagens de procurement

### 6.1 As cinco mensagens que vencem comitê

| # | Mensagem | Objeção que mata | Prova anexa |
|---|---|---|---|
| 1 | **"BCValidador dentro"** — todo arquivo é validado com o leiaute oficial (e além: regras semânticas do catálogo) antes de sair do sistema; nenhum arquivo reprovado é remetido | "como sei que não vou levar crítica do BCB?" | scorecard de precisão (seção 9.4) |
| 2 | **"Zero Autuação™"** — autuação por erro de leiaute gerado pelo sistema = crédito/reembolso contratual | "o risco continua meu" | risco de R$375k vira risco do fornecedor; SLA no contrato |
| 3 | **"Onboarding em 15 minutos"** — importa cadastro, valida, remete; sem projeto, sem amarração de core | "implantação da Matera levou 9 meses" | demo com os dados do prospect na 1ª conversa |
| 4 | **"Leiaute no dia da norma"** — SLA público norma→leiaute ≤10 dias úteis, com diff semântico e reteste; a IN 733 mudou o 3040 em 3 ondas (mai/jul/nov/2026) e a IN 694 trocou o leiaute do DRSAC sem amplo aviso de mercado | "quem mantém o leiaute atualizado?" | T2U medido em cada onda (métrica do North Star) |
| 5 | **"Preço público"** — lista na página, add-ons na página, sem cotação misteriosa | "quantos meses até a proposta?" | procurement avança sem kaizen de cotações |

### 6.2 vs Matera (incumbent legacy)

Matera é forte, tem 250+ IFs e receita-alvo R$500 mi — e é exatamente por isso que a nossa resposta não é "somos melhores": é **time-to-update + DX + neutralidade**. Leiautes hardcoded doem neles a cada IN (733, 750, 694, 755) tanto quanto nos clientes; implantação em meses amarra o core; preço gated alonga procurement. Não atacar relacionamento e capacidade de implementação enterprise — atacar a velocidade de mudança normativa, que é a dor que a IN 733 provou em público.

### 6.3 vs cadoc.ai (parceiro candidato, não inimigo)

Eles pivotaram para inteligência regulatória (6.700+ normas, NormaChat, MCP "Norma Skills") e **criaram** o nosso vazio. Não competir por keyword; propor integração: o grafo deles como fonte de mudança normativa, nós como camada de execução (norma→regra→leiaute→remessa). A mensagem de mercado fica: "inteligência normativa existe; execução assinada é outra coisa". Negociar antes de gastar em guerra de SEO.

### 6.4 vs Celcoin/Evertec (o conflito de ecossistema)

Celcoin vende BaaS **e** Regulatório Mais no mesmo grupo: é competidor potencial de todo SCD/IP que seria nosso cliente — e cliente de compliance não se constrói sobre o próprio competidor. Evertec consolidou GRC (Compliasset) na mesma lógica. Nossa resposta: **a camada neutra** — sem BaaS, sem core, sem vertical; single-tenant e on-prem possíveis (Rust single binary); regras e provenance auditáveis por terceiro indicado pela IF. Neutralidade é argumento de procurement, não só de arquitetura. Para BaaS não-conflitantes, o white-label do Scale transforma o mesmo argumento em canal.

---

## 7. Marca: não lançar como "Norma"

### 7.1 A decisão

> **Nota da fusão (16/11/2026):** "Radiant Reg" é o **nome de trabalho do wedge da FASE 1** (25 §0). A busca de marca BR-nativa descrita abaixo aplica-se ao **produto da FASE 2**.

**"Radiant Norma" não vai a mercado como marca de produto.** "Norma" é o nome do produto do cadoc.ai (Norma, NormaChat, NormaCast, Norma App) — uso comercial público anterior, confusão de search imediata (buscar "Norma CADOC" traz o concorrente) e risco de marca em uso. "Radiant" não significa nada em PT-BR e é intraduzível em procurement. Decisão: **"Radiant" permanece como entidade legal/investimento**; o produto lança com marca BR-nativa própria. Custo do rebrand hoje é baixo (pré-launch); depois do primeiro ciclo de vendas, é caro.

### 7.2 Critérios do nome

1. Curto (≤7 letras na forma de uso) e pronunciável em PT-BR;
2. Evoca CADOC, remessa, validação ou norma regulatória **sem colidir** com "Norma"/cadoc.ai e sem copiar marca alheia;
3. Domínio .com.br livre e handles coerentes;
4. Pesquisável: buscou, achou a gente (sem ambiguidade com termos genéricos impossíveis de dominar — ex.: "remessa" sozinho é guerra perdida contra câmbio/remessas internacionais);
5. Passa em procurement bancário: soa sério, sem graça de startup de consumo;
6. Sem colisão com termos regulados ou institucionais (Sisbacen, CRD, cartório etc.).

### 7.3 Candidatos (7 — a validar em pesquisa com 10 ICPs)

| Nome | Racional | Risco a checar |
|---|---|---|
| **Cadência** | evoca CADOC (cad-) e o ritmo semestral/mensal das remessas; sério, musical, memorável | colisão fonética com "Cadê?"; marcas em 42 |
| **Crivo** | o crivo é a peneira que separa o aprovado do reprovado — metáfora exata do validador; 5 letras | marcas em 9/42; "crivo" genérico em jurídico |
| **Remeta** | imperativo de "remeter": o produto é a promessa em uma palavra | descritividade (proteção de marca mais fraca) |
| **Cadex** | CADOC + ex(change/engine); curto, tech, pesquisável | semelhança com marcas "Cadex" estrangeiras (baterias) |
| **Égide** | "sob a égide da norma" — proteção regulatória em 6 letras, tom jurídico | tom abstrato; pesquisa com ICP para checar memorabilidade |
| **Vigia** | vigilância regulatória contínua (o Radar que não dorme) | genérico em segurança do trabalho |
| **Lastro** | solidez financeira BR-nativa; procurement confia na palavra | pouco ligado a CADOC (evocação indireta) |

### 7.4 Disclaimer obrigatório

**Nenhuma busca de anterioridade foi realizada para este documento.** Antes de Q1/2027, executar: (a) busca de anterioridade no INPI nas classes pertinentes (típicos 09 e 42; avaliar 36); (b) disponibilidade de domínios (.com.br e .com) e handles; (c) varredura de colisão no Google/App Stores/redes; (d) registro do pedido de marca antes do primeiro gasto de marketing (taxas INPI por classe; desconto para ME/EPP). Este capítulo é critério de decisão, não parecer jurídico — a escolha final passa por advogado de propriedade intelectual.

---

## 8. Jogada DRSAC — 3 movimentos com metas numéricas

**Contexto verificado:** IN BCB 694/2025 muda leiaute/instruções do 2030 com vigência na **data-base dez/2026**; obrigados: **S1–S4, inclusive IPs** (S5 dispensado); semestral (jun/dez); prazo: 10º dia útil do 2º mês subsequente → **1ª remessa vence ~fev/2027**, 2ª ~ago/2027; sem exposição = "documento zerado" via CRD; **não existe planilha pública de críticas para o 2030** (só XSD + validador Windows + FAQ) — o que torna um validador semântico próprio diferencial real, não commodity. Transição: códigos antigos dos Anexos 08/09 só em registros já existentes até dez/2026; códigos 01–04 deixam de ser aceitos em registros novos em jun/2027.

### Movimento 1 — Beta fechado (dez/2026–fev/2027)

**reg-cadoc 2030 em Rust (S4 da FASE 1 — leiaute oficial IN 694/2025 re-baixado) + chancela do BacenValidatorAdapter (ADR-0018)** serve o piloto; o wiring do gen2030 em Go foi revogado pela fusão (retrabalho). Captação via FBM/consultorias + lista nominal S1–S4.

| Meta | Número |
|---|---|
| IFs no beta fechado | **8** (de 5–8 alvo) |
| Remessas beta aceitas no CRD | **100%** (documento zerado e/ou com exposição) |
| Casos documentados (com anuência) | 3 |
| Contratos convertidos até a 1ª remessa | **2–4 contratos na régua do wedge** (Sprint R$35–60k / Licença R$6–12k/mês — §2.0) |
| Receita do movimento | Sprints (R$35–60k) + licenças (R$6–12k/mês) na régua do wedge — **linha própria, fora do MRR SaaS** (§4.1) |

### Movimento 2 — Conversão da janela (fev–ago/2027)

Launch Rust no ar, pricing público, case "remessa aceita" nas mãos. Retarget da lista S1–S4 para a 2ª data-base (jun/2027, com a transição dos códigos 01–04 como gatilho de dor novo).

| Meta | Número |
|---|---|
| Clientes com DRSAC pagante na 2ª data-base | **15** |
| Attach do add-on DRSAC na base Starter S1–S4 | **≥25%** |
| T2U da onda jun/2027 medido e publicado | ≤10 dias úteis |
| Webinars/eventos com canal (ABBC/ABFintechs/FBM) | 2 |
| Contribuintes para o benchmark ESG (opt-in) | 15 |

### Movimento 3 — Expansão (ago–dez/2027)

DRSAC deixa de ser produto e vira **porta**: quem mandou o 2030 certo pelo Radiant recebe o upsell natural para GRSAC (Res. 586 + IN 772, vigência 1º/1/2027), DRL 2160 (leiaute novo IN 755) e o resto do calendário 2027 — e o benchmark de segmento vira produto premium com os contribuintes acumulados.

| Meta | Número |
|---|---|
| Base DRSAC com ≥1 módulo adicional | **≥40%** |
| Contribuintes do benchmark (opt-in) | **30** |
| Enterprise S3–S4 originado no movimento | 1 |
| MRR atribuível à jogada DRSAC (acumulada) | **R$25–35k** do R$210k base |

**Disjuntores:** K5 (0 IFs pagantes da janela até 28/fev/2027 → matar GTM de janela) e K1 (<5 IFs com remessa aceita até 31/mar/2027 → pivotar para engine licenciado). A jogada DRSAC é a maior aposta única do ano 1 — e a única com prazo que procurement não pode adiar.

---

## 9. Playbook de vendas

### 9.1 Discovery de 30 minutos (estrutura fixa)

| Bloco | Tempo | Perguntas-chave | O que extrair |
|---|---|---|---|
| 1. Contexto | 2 min | papel do contato; CNPJ; segmento S (se souber) | enquadramento ICP |
| 2. Como remete hoje | 8 min | ferramenta/consultoria; quem assina o 3040; fluxo do 3044; datas do ciclo | custo atual mensurável (horas × tarifa) |
| 3. Dor quantificada | 8 min | horas por remessa; última crítica/autuação; retrabalho com IN 733/694; quem apaga incêndio no dia 10 | o número que âncora o ROI |
| 4. Orçamento e processo | 6 min | quem aprova; quanto paga hoje; prazo de decisão; exigências de segurança | mapa do comitê |
| 5. Próximo passo | 6 min | propor validação do arquivo real deles no Free, ao vivo | compromisso de demo com dado real |

**Scorecard ICP (0–10):** obrigação 3040 ativa (2) · >2 CNPJs ou >50k ops/mês (2) · já levou crítica/autuação (2) · consultoria ativa hoje (2) · S1–S4 na janela DRSAC (2). **≥7:** AE com proposta em 5 dias · **4–6:** self-serve assistido (Starter) · **<4:** nurturing no Free.

### 9.2 Anticorpos para IF (objeção → resposta → prova)

| Objeção | Resposta | Prova |
|---|---|---|
| "Uso consultoria e funciona" | Consultoria custa R$3–15k/mês e revisa à mão; Starter é metade da consultoria mais barata, com leiaute oficial dentro e histórico auditável | demo comparativa; trial 30 dias |
| "BCValidador é grátis" | É. Ele valida no fim da esteira e não gera, não explica campo, não versiona leiaute por data-base; nosso Free cobre o mesmo em browser, sem JRE | comparativo lado a lado no Free |
| "A Matera já nos atende" | Ótimo fornecedor — pergunta: quantos dias levou a última onda de leiaute? IN 733 foram 3 ondas em 6 meses | T2U publicado; onboarding 15min |
| "E a segurança/LGPD?" | Dados no nosso ambiente com audit hash-chain tamper-evident; single-tenant/on-prem no Enterprise; ISO 27001/SOC 2 na régua de procurement | documentação de arquitetura; DPA |
| "IA erra em compliance" | IA nunca assina veredito: motor determinístico + catálogo versionado + humano no loop; toda sugestão cita norma/cláusula | scorecard de precisão (9.4) |
| "Sem orçamento agora" | Janela DRSAC: multa até R$375k por atraso vs add-on de R$990/mês; o ROI é uma evitação | FAQ oficial + Lei 13.837/2019 |
| "E se vocês caírem no dia 10?" | SLA 99,9% + Zero Autuação™; e o arquivo pode ser remetido direto no STA a qualquer momento — remessa não tem lock-in | cláusula de portabilidade no contrato |
| "Vocês são pequenos" | Por isso o risco é nosso: garantia contratual de autuação; implantação assistida no Enterprise | Zero Autuação™ por escrito |

### 9.3 Estrutura de proposta (1 página + anexos)

1. **Resumo executivo** — escopo por CADOC × data-bases cobertas; persona do dia a dia (analista) e do assinante (compliance/CFO).
2. **Scorecard de precisão do motor** (anexo obrigatório — seção 9.4).
3. **Escopo contratual e SLA** — uptime por tier; Zero Autuação™ no Scale+; critérios de ativação da garantia.
4. **Preço público** — tier + add-ons elegíveis; sem "cotação sob consulta".
5. **Onboarding 15min + piloto 30 dias** — critérios de aceite objetivos: semana 1 import+validação retroativa; semana 2 1ª remessa assistida; semana 3 2ª remessa autônoma; semana 4 scorecard final + proposta de contrato.
6. **Garantias** — portabilidade de dados, saída sem multa, neutralidade (sem BaaS/core), auditabilidade por terceiro indicado pela IF.

### 9.4 Scorecard de precisão do motor (o que vai em toda proposta)

Lição do repo v1 (auditoria 02): números escritos à mão viraram 4 versões contraditórias no mesmo arquivo. O scorecard abaixo é **emitido por script do build** — nunca editado à mão:

| Métrica | Fonte | Apresentação |
|---|---|---|
| Críticas cobertas por CADOC × data-base | catálogo de regras do build | "3040 mai/26: 361 críticas, X implementadas, Y stubs honestos declarados" |
| Accuracy da eval suite | harness (flywheel evento→eval) | tendência mensal, com hash do run |
| T2U mediano da última onda | registro de cada IN (733/694/750) | "IN 733 NR3: N dias úteis" |
| Reprovações evitadas no CRD | retornos dos clientes (anonimizado) | contador acumulado |
| Uptime do SLA | monitoramento | janela dos últimos 90 dias |

Proposta sem scorecard atualizado não sai — o documento de venda é também o contrato de honestidade da tese v2.

### 9.5 Pipeline math do cenário base

45 clientes em 12 meses ≈ 4 fechamentos/mês. Com win rate 25% pós-piloto e 4–6 pilotos ativos por mês: ~40 conversas no topo/mês → ~16 ICPs qualificados → 6 pilotos → 4 fechamentos. Na janela DRSAC (movimento 2, seção 8), o piloto de 30 dias comprimido no prazo regulatório é o conversor — é para isso que o add-on de R$990 existe: baixar a barreira de entrada o suficiente para o prazo mandar.

---

## Fontes e consistência

- **Números de mercado:** `research/08-produto-pricing.md` (Olinda 16/11/2026; WTP; benchmarks SaaS; projeção) e `research/06-normativos-realidade.md` (1.722 entidades; S1–S5; FAQ DRSAC; calendário 2027).
- **Concorrência/canais:** `research/05-concorrentes-mercado.md` (mapa BR nov/2026; cadoc.ai pivô; Celcoin/Evertec; benchmarks globais de pricing).
- **Autopsy do pricing v1 e lições de honestidade:** `research/02-tese-vs-realidade.md` (§7.2 item 6 — "pricing nunca testado contra comprador real"; §3.4 — forward-padding).
- **Decisões de tese/arquitetura:** `11-TESE-V2.md` (North Star; kill criteria K1–K5; calendário GTM 2026–2028; posicionamento §5).
- Contagens de IFs extraídas das APIs públicas do BCB em 16/11/2026 — re-executar as consultas OData no dia do board.

*Documento gerado em 16/nov/2026. Preços em reais por mês, base anual salvo indicação. Este doc fixa decisões: mudanças passam por revisão desta página, não por adendos em outros arquivos.*
