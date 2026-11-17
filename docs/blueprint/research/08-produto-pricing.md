# 08 — Produto, Pricing e GTM: Pesquisa de mercado com dados reais

> **Projeto:** Radiant Norma (SaaS regulatório BR — geração/validação/remessa de CADOCs)
> **Data da pesquisa:** 16/nov/2026 · **Fontes:** oficiais BCB (APIs públicas), imprensa econômica, dados de procurement global (Vendr), benchmarks SaaS.
> **Método:** dados quantitativos extraídos direto das APIs abertas do Banco Central (OData "Instituições em Funcionamento", atualizado 11/nov/2026) e PDFs oficiais; preços de mercado via fontes públicas citadas URL a URL. Onde só existe estimativa, está marcado como estimativa.

---

## Sumário do documento

1. [TAM / SAM / SOM](#1-tam--sam--som)
2. [WTP — quanto as IFs pagam hoje](#2-wtp--quanto-as-ifs-pagam-hoje)
3. [Modelos de pricing RegTech global e captura do flywheel](#3-modelos-de-pricing-regtech-global-e-captura-do-flywheel)
4. [PLG vs Sales-led no mercado regulado BR](#4-plg-vs-sales-led-no-mercado-regulado-br)
5. [Unit economics de SaaS B2B regulado (BR + global)](#5-unit-economics-de-saas-b2b-regulado-br--global)
6. [Recomendação: pricing final + projeção MRR 12 meses](#6-recomendação-pricing-final--projeção-mrr-12-meses)
7. [Posicionamento e GTM](#7-posicionamento-e-gtm)
8. [Fontes](#8-fontes)

---

## 1. TAM / SAM / SOM

### 1.1 Universo oficial de entidades autorizadas (BCB, nov/2026)

Contagem extraída diretamente da API pública do BCB — serviço OData `Instituicoes_em_funcionamento` (dataset "Relação de Instituições em Funcionamento no País", metadados atualizados em 11/nov/2026):

- Endpoint: `https://olinda.bcb.gov.br/olinda/servico/Instituicoes_em_funcionamento/versao/v1/odata/`
- Dataset: <https://dadosabertos.bcb.gov.br/dataset/relacao-de-instituicoes-em-funcionamento-no-pais>
- Página oficial: <https://www.bcb.gov.br/estabilidadefinanceira/relacao_instituicoes_funcionamento>

| Tipo de entidade | Qtd. (sedes, nov/2026) | Obrigação CADOC? |
|---|---:|---|
| Banco Múltiplo (incl. BB) | 130 | Sim — contábil + SCR + Produb |
| Banco Comercial (+ filiais estrangeiras) | 18 | Sim |
| Banco de Investimento | 12 | Sim |
| Banco de Desenvolvimento + BNDES | 4 | Sim |
| Banco Múltiplo Cooperativo | 2 | Sim |
| Banco de Câmbio | 4 | Sim (câmbio/SCR) |
| Caixa Econômica Federal | 1 | Sim |
| **Subtotal bancos** | **~171** | |
| Cooperativas de crédito — singulares | 701 | Sim — contábil + SCR |
| Cooperativas — centrais | 25 | Sim (consolidado) |
| Cooperativas — confederações | 4 | Sim |
| SCD (Sociedade de Crédito Direto) | **138** | Sim — SCR (3040/3050) + contábil |
| SEP (Empréstimo entre Pessoas) | **11** | Contábil; SCR conforme operações |
| SCM (Crédito ao Microempreendedor) | 21 | Sim — SCR |
| SCFI (Crédito, Financiamento e Investimento) | 80 | Sim — SCR |
| **Instituições de Pagamento (IP)** | **194** | Sim — DIP 3501, Doc 3044, DRSAC (S1-S4) |
| DTVM (Distribuidora de TVM) | 101 | Sim — contábil |
| CTVM (Corretora de TVM) | 57 | Sim |
| SCC (Corretora de Câmbio) | 47 | Sim (câmbio) |
| Agência de Fomento | 16 | Sim |
| Companhia Hipotecária | 5 | Sim (SCR) |
| Soc. de Arrendamento Mercantil | 16 | Sim (SCR) |
| Administradora de Consórcio | 129 | Sim (documento próprio de consórcios) |
| **TOTAL sedes autorizadas** | **~1.600** | |

Verificação cruzada de cooperativas: 736 no total (701 singulares + 25 centrais + 4 confederações), fonte OData `SedesCooperativas` (mesma URL acima).

### 1.2 Segmentação prudencial S1–S5 (base do cronograma DRSAC)

Segmentação da Resolução CMN 4.553/2017 (exposição como % do PIB, operações internacionais, porte). Distribuição agregada (dados BCB compilados por Dados Bancários, <https://dadosbancarios.com.br/analise-segmentos>):

| Segmento | Critério (síntese) | Qtd. | % ativos do sistema |
|---|---|---:|---:|
| S1 | ≥10% do PIB / internacionais | **6** | 63,93% |
| S2 | 1–10% do PIB | **10** | 16,17% |
| S3 | 0,1–1% do PIB | **65** | 12,16% |
| S4 | <0,1% do PIB | **374** | 5,02% |
| S5 | pequenas (incl. cooperativas singulares) | **806** | 2,72% |
| **Total segmentadas** | | **1.261** | |

Listas nominais por segmento (BCB): <https://www.bcb.gov.br/estabilidadefinanceira/regprudencialsegmentacao>

### 1.3 Quem tem obrigação CADOC (3040 / 3050 / DRSAC)

**Doc 3040 — SCR operações de crédito (individual, mensal).** Obrigatório para entidades que realizam operações de crédito e arrendamento mercantil: bancos, cooperativas de crédito, SCD, SCM, SCFI, companhias hipotecárias, arrendadoras e IPs com operação de crédito. Fontes oficiais:

- <https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040>
- Instruções de preenchimento: <https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/SCR_InstrucoesDePreenchimento_Doc3040.pdf>

**Doc 3050 — Estatísticas agregadas de crédito e arrendamento (mensal).** Mesma base de obrigados do 3040; o próprio BCB publica FAQ específico e tabela de equivalência 3040↔3050:

- <https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3050> · FAQ: <https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3050_faq>
- Equivalência de modalidades: <https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/Equival%C3%AAncia_entre_modalidades_dos_documentos_3040_e_3050.pdf>

**Doc 2030 — DRSAC (Risco Social, Ambiental e Climático).** **Gatilho de mercado para a janela dez/2026–jun/2027**, extraído do FAQ oficial (PDF baixado em 16/nov/2026):

- "Todas as instituições autorizadas a funcionar pelo Banco Central do Brasil pertencentes aos segmentos **S1 ao S4, inclusive Instituições de Pagamento (IP)**, estão obrigadas a enviar o DRSAC. As instituições do segmento S5 estão dispensadas."
- Documento **semestral** (datas-base junho e dezembro); remessa **até o 10º dia útil do 2º mês subsequente** à data-base; instituições sem exposição devem registrar "documento zerado" no CRD.
- PDF oficial: <https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/drsac2030/Perguntas-Respostas-DRSAC.pdf>
- Leiaute: <https://www.bcb.gov.br/estabilidadefinanceira/leiaute-documento-2030>
- IN BCB 694 (dez/2025) alterou leiaute/instruções: <https://www.legisweb.com.br/legislacao/?id=488596> e <https://abbc.org.br/instrucao-normativa-bcb-n-694-instrucoes-drsac>

**População DRSAC = S1-S4 ≈ 455 instituições** (6+10+65+374) **+ IPs enquadradas em S1-S4** — primeira remessa sob o novo leiaute: data-base dez/2026 (vence ~fev/2027) e jun/2027 (vence ~ago/2027). É a janela exata citada no planejamento do produto.

**Demais documentos relevantes (catálogo CADOC/CRD):** DIP 3501 das IPs, Doc 3044 (contas remuneradas de IPs, obrigatório desde 1º/nov/2025 — <https://credisoft.com.br/cadoc-3044>), SARO/DAS/CAS (4010/4020/4030), Produb (800/803/809), documento de consórcios. Catálogo: <https://www.bcb.gov.br/estabilidadefinanceira/catalogodocumentos> e controle de remessa CRD: <https://www3.bcb.gov.br/crd>

**FIDCs:** não remetem CADOC diretamente (são supervisionados pela CVM; reportam via administradores/informe CVM-Anbima). ~**3.000 FIDCs em funcionamento** (CVM/Anbima, jan/2025 — <https://www.anbima.com.br> / acordo de cooperação noticiado em jan/2025). São TAM indireto: acessados via seus administradores (DTVMs) e gestoras.

### 1.4 Crescimento e consolidação 2021–2026

| Indicador | 2021 | 2025/2026 | Fonte |
|---|---:|---:|---|
| Fintechs de crédito (SCD+SEP) | 74 | **149** (138 SCD + 11 SEP) | 74 em 2021: BCB citado em estudo (<https://www.bcb.gov.br/conteudo/relatorioinflacao/estudosespeciais/ee089_fintechs_de_credito_e_bancos_digitais.pdf>); 2026: OData BCB (§1.1) |
| Total sociedades de crédito (SCD+SEP+SCM+SCFI) | — | **250** | OData BCB (§1.1) |
| IPs ativas | — | **194** | OData BCB (§1.1) |
| Fintechs no Brasil (todas) | ~1.158 (2020) | ~2.100 (meados 2025); projeção ~3.200 (2028) | Valor International via ABFintechs/ASBANC |
| Associadas ABFintechs | 724 (início 2025) | **588** (meados 2026) — **consolidação por exigência do BC** | Valor Econômico, citado em cobertura de mercado |
| Autorização prévia p/ IP (Res. BCB 494/2025) | — | Regime novo; parcela relevante de IPs deve ser "desenquadrada" até 2028 | <https://azify.com/blog/licencas-banco-central-fintechs> + cobertura BCB |

**Leitura estratégica:** o mercado **cresce em valor e encolhe em número** — o BCB está endurecendo licenciamento (Res. 494/2025) e as sobreviventes ficam maiores e mais reguladas (mais documentos, mais risco). Isso favorece ferramenta profissional sobre planilha/consultoria improvisada.

### 1.5 TAM / SAM / SOM (numéricos)

| Camada | Definição | Qtd. IFs | Valor potencial |
|---|---|---:|---|
| **TAM** | Todas as entidades com obrigação de remessa contábil/estatística ao BCB | ~1.400–1.500 | preço médio R$60k/ano ⇒ **~R$0,9–1 bi/ano** |
| **SAM** | IFs endereçáveis pelo produto v1 (SCD, SCM, SCFI, SEP, IP, DTVM/CTVM/SCC, Afo/CH, cooperativas, S3-S4) | ~1.100 | preço médio R$55k/ano ⇒ **~R$600 mi/ano** |
| **SOM 12 meses** (pós-launch Q1/2027) | Penetração realista com PLG + canal | 20–85 clientes | **R$0,9–5,2 mi ARR** (ver §6.3) |

*Obs.: TAM em valor usa a banda de preços observada (§2); é estimativa top-down, não dado oficial.*

---

## 2. WTP — quanto as IFs pagam hoje

### 2.1 Preços observados dos concorrentes diretos

| Fornecedor | Preço | Evidência/URL |
|---|---|---|
| Matera (RegTech/core) | **R$30k+/ano**; contratos enterprise maiores | 250+ IFs atendidas no BR, receita-alvo R$500 mi (Brazil Journal, jun/2025 — <https://braziljournal.com/brands/matera-muda-marca-para-consolidar-sua-evolucao-e-lideranca-em-tecnologia-para-o-mercado-financeiro>); captação de R$500 mi jul/2024; aquisição Fáciltech nov/2025 (<https://finsidersbrasil.com.br/noticias-sobre-fintechs/matera-compra-faciltech-na-maior-aquisicao-de-sua-historia>). Sem preço público (modelo por cotação). |
| Mitra/LUZ | **R$50k+/ano** | Contratos por cotação; sem preço público |
| cadoc.ai | **R$5–15k/ano** (assinatura de inteligência regulatória) | Hoje é produto "Norma" (monitor de 6.700+ normas BCB/CMN + NormaChat) — <https://cadoc.ai/sobre>; sem página pública de preço |
| Dattos | **R$5–10k+/ano** (conciliação + informes RC) | Site sem preço público, sales-led "Agendar demonstração" — <https://www.dattos.com.br>; produto IF: <https://www.dattos.com.br> (soluções Instituições Financeiras/IP) |
| Regulatório Mais / Celcoin (bundle BaaS) | **R$3–8k/ano** | Pacote de regulatório em bundles BaaS |
| ReconArt (global, categoria reconciliação) | entry **US$1.500/mês (5 usuários)** | ERP Research — <https://www.erpresearch.com/erp-add-ons/reconciliation/autorek> |

### 2.2 Custo da alternativa manual / terceirizada

| Alternativa | Custo | Fonte |
|---|---|---|
| Consultoria compliance terceirizada | **R$150–600/hora**; uso típico 3–4x/mês ⇒ **R$3–15k/mês** para CADOCs de IF pequena | ClickCompliance — <https://clickcompliance.com/compliance/profissional-de-compliance-quanto-custa> |
| Estruturar compliance completo de fintech B2B | **R$400 mil–R$1,5 mi/ano** | Baita (guia 9 pilares BACEN) — <https://baita.ac/insights/compliance-para-fintech-b2b-guia-pratico-mm3wlv4f> |
| Custo operacional recorrente de IF licenciada | ~**US$150 mil/mês** (inclui compliance) | Celcoin — <https://celcoin.com.br/articles/custo-para-lancar-fintech-brasil> |
| Analista de compliance (salarial, BR 2025) | **R$4,5–9k/mês** pleno; coordenador R$12,8k+ | Salário.com.br, Indeed, LEC Consultoria (compilado na pesquisa; médias R$5,4–6,3k) |
| Analista do BCB (referência teto do mercado) | R$13,6k (2025) → R$16,7k (2026) | Edital BCB |

### 2.3 Custo do erro (autuação)

- Lei 13.837/2019, art. 11: multa por atraso/falta de informação ao BCB **até R$250 mil**, com **aumento de 50%** em caso de não atendimento a intimação — <https://www.bcb.gov.br/meubc/faqs/p/tipos-de-penalidades-aplicadas-pelo-banco-central>
- Resolução BCB 131/2021 regulamenta processo sancionador específico de multa por atraso de remessa ("ato vinculado" — jurisprudência CRSFN) — <https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Resolu%C3%A7%C3%A3o%20BCB&numero=131>
- Escala antiga de referência (Res. 2.194): **R$150/dia útil** de atraso — mostra que atraso é multável por dia.

### 2.4 Síntese de WTP por persona

| Persona | Paga hoje (banda) | Gatilho de compra | Teto plausível de subscrição |
|---|---|---|---|
| SCD pequena (carteira R$50–500 mi) | R$36–180k/ano (consultoria + horas internas) | Primeira autuação; crescer carteira sem expandir compliance | **R$60–120k/ano** |
| IP (emissor/adquirente) | R$36–180k/ano | Doc 3044 + DIP 3501 + DRSAC (S1-S4) | **R$80–200k/ano** |
| Fintech BaaS | R$60–250k/ano | Obrigações próprias + serviço a tomadores (multi-CNPJ) | **R$150–400k/ano** |
| Banco S3-S4 / DTVM média | R$300k–1,5 mi/ano (equipe + sistemas legados) | DRSAC, densidade regulatória, auditoria | **R$250k–1 mi/ano** |

> **Conclusão WTP:** o preço atual da casa (Lite R$1,5k/mês) está **2–8x abaixo** do que o mercado já paga por soluções equivalentes ou piores, e **abaixo até do custo de consultoria manual**. Há espaço para elevar entry price sem sair da banda competitiva.

---

## 3. Modelos de pricing RegTech global e captura do flywheel

### 3.1 Panorama dos modelos (com valores reais)

| Modelo | Quem usa | Valor observado | Fonte |
|---|---|---|---|
| Enterprise quote (assinatura anual, por escopo) | Regnology, Duco, AutoRek, Trintech, Matera | Duco **~US$183k/ano** (faixa US$52–281k); Trintech **~US$178k** (até US$290k+); Cadency US$300k–1 mi+ | Vendr/compilados — <https://www.erpresearch.com/erp-add-ons/reconciliation/duco>; bankautomation.com; safeheld.com |
| Per-seat (usuário nomeado) | AutoRek, ReconArt | ReconArt entry **~US$1,5k/mês/5 usuários** | <https://www.erpresearch.com/erp-add-ons/reconciliation/autorek> |
| Per-entity / per-filing | Workiva (SOX/ESG por entidade/arquivo) | Mediana **US$49,4k/ano** (US$12,7–153k) | Vendr — <https://www.vendr.com> (dados de compra agregados) |
| Usage-based (volume de transações/dados) | Duco (volume de reconciliações), FactSet (market data) | Variável por volume | <https://safeheld.com/resources/duco-alternative>; FactSet licensing |
| Managed services (regulatory-as-a-service) | Regnology (experts como add-on) | Add-on sobre assinatura | <https://www.regnology.net/ecomaXL/files/RegOps-Whitepaper-Regnology-A-new-approach-towards-the-future-of-regulatory-reporting.pdf> |
| Freemium/PLG-assisted em compliance | Vanta (SOC 2) | entry **US$7,5–10k/ano**; mediana **~US$20k** | <https://www.vanta.com/pricing>; <https://www.trycomp.ai/vanta-pricing>; <https://www.vendr.com/marketplace/vanta> |
| Outcome/success-fee | Raro em compliance (risco de independência) | — | padrão de mercado: SLA/garantia em vez de success fee |

### 3.2 O que maximiza NRR (evidência)

- NRR mediana global: **101%** (Benchmarkit 2024/25, queda desde 2021) — <https://benchmarkit.ai/2025benchmarks>; por segmento: enterprise **118%**, mid-market 108%, SMB 97%.
- **Usage-based pricing → NRR maior:** "NRR acima de 120% é quase impossível com subscrição flat" (Lago, 2025 — <https://lago.dev>); 46% das empresas SaaS já usam modelo híbrido assinatura+uso (OpenView, via <https://metronome.com/state-of-usage-based-pricing-2025>); 78% dos adotantes de UBP implementaram nos últimos 5 anos (Metronome 2025).
- Analogia direta (compliance PLG): Vanta cresceu com entry self-serve + expansão por frameworks adicionais (~US$5k+ por framework extra) — <https://www.strac.io/blog/vanta-pricing>.

### 3.3 Como precificar "o sistema melhora com uso" (data moat / flywheel)

O valor composto do Radiant Norma vem de: (a) base de erros de leiaute já validados; (b) padrões de rejeição do CRD por segmento; (c) histórico normativo consolidado; (d) rede de IFs usando o mesmo validador. Estruturas que capturam isso:

1. **Métrica de valor que cresce com o cliente:** nº de entidades (CNPJ) remetentes + volume de operações SCR processadas — o cliente cresce, a conta cresce (usage) ⇒ NRR.
2. **Benchmark/inteligência como tier pago:** só quem contribui dados anonimizados acessa o benchmark do segmento ("quais campos reprovam mais no CRD por tipo de IF") — transformação de data moat em produto premium.
3. **Data credits em vez de desconto:** redução de preço concedida como "crédito" por opt-in de dados — protege o moat e ainda gera goodwill em procurement.
4. **Marketplace/white-label para BaaS:** o BaaS revende validação para seus tomadores — cada tomador é uma entidade cobrada (expansion via rede, não por vendas).
5. **Outcome como garantia, não como preço:** "Zero Autuação™" como SLA contratual (crédito/reembolso em caso de autuação por erro de leiaute do sistema). Success fee em compliance gera suspeita em procurement; SLA/garantia de-riska a compra.

---

## 4. PLG vs Sales-led no mercado regulado BR

### 4.1 Evidência

- **Dattos** (conciliação, IFs): sales-led — sem preço público, CTA "Agendar demonstração" (<https://www.dattos.com.br>). Dominante no mid-market de conciliação.
- **cadoc.ai**: content-led (newsletter, NormaCast, slides executivos, cursos) com assinatura de inteligência — provou que **conteúdo regulatório gera audiência ICP no BR** (<https://cadoc.ai/sobre>).
- **Vanta** (paralelo global): compliance automation com trial/self-serve + preço de entrada acessível; mediana de contrato ~US$20k — PLG funciona em compliance quando o produto elimina o pior risco (auditoria reprovada) — <https://www.vendr.com/marketplace/vanta>.
- **FBM Educação** vende formação "Especialista em CADOCs" para contadores/riscos/compliance — a audiência exata do ICP já está organizada em um canal educacional (<https://fbmeducacao.com.br/blog/o-curso-especialista-em-cadocs-e-para-mim-um-guia-para-profissionais-de-contabilidade-riscos-e-compliance>).

### 4.2 Recomendação de motion (híbrido, por persona)

| Persona | Motion | Ciclo realista | CAC alvo |
|---|---|---|---|
| SCD/IP/DTVM pequena | **PLG-assisted**: validador free → Starter self-serve (cartão) com assistência | 15–45 dias | **R$6–12k** (inbound + comunidade) |
| Fintech BaaS / multi-CNPJ | Sales-assisted (1 AE) | 2–4 meses | R$15–30k |
| Banco S3-S4 / cooperativa central | Sales-led + canal consultoria | **6–12 meses** (procurement, pentest, due diligence de terceiros) | R$80–150k |

Ciclo B2B bancário BR: benchmarks de mercado apontam 6–18 meses para enterprise regulado (média B2B global 10,1 meses — 6sense, via <https://komunikativa.com/blog/ciclo-de-vendas-b2b>; homologação técnica + compliance de fornecedor são os alongadores).

### 4.3 Canais BR que funcionam (com evidência)

1. **Consultorias CADOC/compliance** (ARR Consultoria, Merc Group etc.): revenda/implementação com rev-share 20% do 1º ano — <https://www.arrconsult.com.br>, <https://www.mercgroup.com.br/servicos/riscos-compliance-e-regulatorio/fintechs-instituicoes-de-pagamento-e-scd>
2. **FBM Educação**: co-marketing, aulas no curso de CADOCs, certificação conjunta — audiência qualificada e recorrente.
3. **ABBC (cursos/educação executiva)** e **ABFintechs (588 associadas)** / **Zetta**: eventos, normas comentadas, fila de indicação — <https://abbc.org.br/cursos>, <https://somoszetta.org.br/quem-faz>
4. **LinkedIn compliance BR**: conteúdo técnico (leiautes, cronogramas) — formato validado pelo cadoc.ai (NormaCast) e pela audiência da FBM.
5. **BaaS providers como canal white-label** (Celcoin, portos BaaS): o "Tomadores de Serviços BaaS" já é categoria no dataset do BCB — o BaaS empacota o regulatório do tomador; revenda natural do tier Scale.
6. **Marketplace BCB: não existe.** Não há programa de "parceiros homologados" do BCB para software de CADOC; a validação de facto é o arquivo passar no validador do CRD. Investir em "certificação BACEN" é dinheiro perdido (ver §7.4).

---

## 5. Unit economics de SaaS B2B regulado (BR + global)

### 5.1 Benchmarks

| Métrica | Benchmark | Fonte |
|---|---|---|
| CAC ratio (novo cliente) | **US$2,00 em S&M por US$1 de nova ARR** (mediana, subiu 14% em 2024) | Benchmarkit — <https://benchmarkit.ai/2025benchmarks> |
| Payback CAC | Mediana **16 meses**; top quartile <6; "bom" <18 | ScaleXP — <https://scalexp.com/blog/saas-benchmarks-cac-payback> |
| Churn mensal B2B BR early stage | **3–5%/mês** (ARR US$1–5 mi); 5–8% (<US$1 mi) | Growth System — <https://growthsystem.com.br/en/benchmarks-startups> |
| NRR BR (early stage) | **100–110%** (ARR US$1–5 mi) | Growth System (idem) |
| NRR global | Mediana 101%; enterprise 118% | Benchmarkit; Optifai (compilado) |
| Receita recorrente BR em escala | Totvs: ARR **R$7,7 bi**, receita recorrente 92,8% da receita, 2T26 | PortalERP — <https://portalerp.com.br/br/noticia/totvs-registra-ebitda-de-r-487-milhoes-no-2t26-com-avanco-da-ia-e-da-receita-recorrente>; RI: <https://ri.totvs.com> |
| Compliance como % do custo operacional do banco | **2–10%** | categoria RegTech (Regnology/Chartis, §3.1) |

### 5.2 Precificação local vs global

RegTech de reporte nos EUA: Duco/Trintech ~US$180k/ano; Workiva mediana US$49k; Vanta mediana US$20k. No BR, o teto observado é Matera/Mitra (R$30–50k+/ano) e o piso é o bundle BaaS (R$3–8k). **Conclusão:** ARPU viável no BR é ~20–35% do equivalente americano para produto comparável; a vantagem do entrante é operar lucrativamente nessa faixa (custo de infraestrutura Rust é trivial vs. preço) e subir ARPU com usage + módulos.

### 5.3 Metas de unit economics do Radiant Norma (12–24 meses)

| Métrica | Meta | Racional |
|---|---|---|
| Gross margin | ≥88% | Infra Rust de baixo custo; validação é CPU-bound barata |
| Churn logo mensal | ≤0,8% | Compliance é obrigatório e Switching cost alto (dados históricos no sistema) |
| NRR | **≥115%** | Usage (entidades/ops) + add-ons (DRSAC, benchmark) + white-label BaaS |
| Payback CAC blended | ≤12 meses | PLG no entry, consultoria com rev-share (sem custo fixo de AE para pequenas) |
| Rule of 40 (mês 24) | ≥40 | Escopo enxuto |

---

## 6. Recomendação: pricing final + projeção MRR 12 meses

### 6.1 Tabela de pricing final (vigência: launch Rust Q1/2027)

| | **Free — Validador** | **Starter** | **Pro** | **Scale** | **Enterprise** |
|---|---|---|---|---|---|
| **Preço mensal (anual)** | R$0 | **R$2.490** | **R$6.900** | **R$16.900** | **a partir de R$29.900** |
| Preço mês a mês | R$0 | R$2.990 | R$8.280 | R$19.900 | sob contrato |
| **Entidades remetentes (CNPJ)** | 1 | 1 | até 3 | até 10 | ilimitado/negociado |
| **Operações SCR (3040/3050)/mês** | 1 mil | 50 mil | 200 mil | 1 mi | ilimitado |
| Documentos | 3040 (gera+valida, envio manual) | 3040, 3050, 3044, DIPs core, DRSAC "zerado" | + DRSAC 2030 completo, dossiês, API | + todos os docs (SARO/DAS/CAS, Produb, consórcios), multi-cadastro consolidado | + segmentações custom, integrações core |
| **BCValidador** (leiaute oficial) | Sim | Sim | Sim | Sim | Sim |
| API / webhooks | — | — | Sim | Sim | Sim |
| White-label p/ tomadores BaaS | — | — | — | **Sim** | Sim |
| Benchmarking & Inteligência (data moat) | — | — | Básico | **Completo** (opt-in de dados) | Completo |
| SLA | — | 99% | 99,5% | 99,9% + "Zero Autuação™" | custom |
| Usuários | 1 | 5 | 10 | ilimitado | ilimitado + SSO/SCIM |

**Add-ons (usage/expansion):**

| Add-on | Preço | Quem compra |
|---|---|---|
| Entidade extra (CNPJ) | R$590/mês | BaaS, grupos com filiais, cooperativas |
| Volume SCR extra (bloco de 50 mil ops/mês) | R$190/mês | SCD/IP crescendo carteira |
| DRSAC completo no Starter | R$990/mês | S1-S4 pequenas na janela dez/26–jun/27 |
| Benchmark Premium (standalone p/ não-clientes) | R$1.490/mês | Bancos que só querem o dado |
| Implantação/homologação assistida (bancos) | R$15–50k one-off | S3-S4 |

**Gates de upgrade (os 4 alavancas de NRR):** nº de entidades · volume de operações SCR · DRSAC completo · white-label/benchmark.

### 6.2 Racional por persona

- **SCD pequena:** Starter a R$2.490 ≈ **1/2 do custo de consultoria manual** (R$3–15k/mês — §2.2) e **1/12 do Matera**. Entry atual de R$1,5k deixa dinheiro na mesa e sinaliza brinquedo; R$2,49k continua impulsivo p/ gestor de SCD (menor que 1 analista júnior).
- **IP:** Pro cobre DIP 3501 + 3044 + DRSAC (IPs S1-S4 obrigadas desde dez/2026). WTP de IP > SCD (arranjos, volume de transações).
- **Fintech BaaS:** Scale com white-label: o BaaS revende para tomadores (cada tomador = entidade cobrada) — expansion estrutural.
- **Banco S3-S4:** Enterprise. Aqui Matera/Mitra cobram R$30–50k+/ano só de módulo; R$29,9k+/mês com SLA "Zero Autuação" e implantação assistida é disruption defensável.

### 6.3 Projeção MRR — 12 meses (3 cenários)

**Premissas comuns:** launch Rust **Q1/2027**; **beta DRSAC já em dez/2026** (hook da janela: primeira remessa vence ~fev/2027, segunda ~ago/2027 — §1.3); churn 1%/mês; ARPU por mix; sem receita de implantação no MRR (contabilizada à parte).

| Cenário | Clientes ativos no m12 (dez/2027) | Mix no m12 | ARPU médio | **MRR m12** | ARR anualizado |
|---|---:|---|---:|---:|---:|
| **Conservador** | 20 | 80% Starter / 14% Pro / 5% Scale / 1% Ent | ~R$3,8k | **~R$76k** | ~R$0,9 mi |
| **Base** | 45 | 75% / 17% / 6% / 2% | ~R$4,7k | **~R$210k** | ~R$2,5 mi |
| **Agressivo** | 85 | 68% / 20% / 9% / 3% | ~R$5,1k | **~R$430k** | ~R$5,2 mi |

Rampa de MRR por trimestre (estimativa):

| | M0 (jan/27) | M3 (abr/27) | M6 (jul/27) | M9 (out/27) | M12 (dez/27) |
|---|---:|---:|---:|---:|---:|
| Conservador | 5k | 18k | 38k | 60k | **76k** |
| Base | 9k | 42k | 95k | 160k | **210k** |
| Agressivo | 15k | 75k | 165k | 290k | **430k** |

**Alavancas do cenário base → agressivo:** (1) 2–3 parceiros consultoria ativos com rev-share; (2) 1 white-label BaaS no ar; (3) pego-on de DRSAC na janela (add-on R$990); (4) 1 banco S3-S4 Enterprise.

**Por que 45 clientes em 12 meses é defensável:** SAM de ~1.100 IFs (§1.5); histórico do próprio nicho (cadoc.ai, Dattos e Regulatório Mais venderam nessa banda de preço sem produto de remessa completo); ciclo SCD/IP de 15–45 dias com PLG-assisted; janela DRSAC cria urgência com prazo regulatório fixo.

---

## 7. Posicionamento e GTM

### 7.1 O mapa competitivo real (nov/2026)

| | Matera | Mitra/LUZ | cadoc.ai | Dattos | **Radiant Norma (alvo)** |
|---|---|---|---|---|---|
| O que vende | Core bancário + RegTech (plataforma completa) | Compliance consultivo + sistema | **Pivotou para inteligência regulatória** ("Norma": 6.700+ normas, NormaChat, newsletter) | Conciliação + informes RC | **Geração+validação+remessa CADOC como sistema de registro** |
| Preço | R$30k+/ano | R$50k+/ano | R$5–15k/ano | R$5–10k/ano | R$30–200k/ano |
| Implantação | Meses (amarra com core) | Semanas-meses | Dias | Semanas | **15 min (self-serve)** |
| Fonte | §2.1 | §2.1 | <https://cadoc.ai/sobre> | <https://www.dattos.com.br> | — |

### 7.2 Mensagens que vencem procurement

1. **"BCValidador dentro"** — valida com o leiaute oficial antes do envio ao CRD; nenhum arquivo reprovado sai do sistema. (Contraste: incumbente exige implantação; consultoria revisa à mão.)
2. **"Zero Autuação™"** — garantia contratual: autuação por erro de leiaute gerado pelo sistema = crédito/reembolso. transforma o risco regulatório (multa até R$250k + 50%, §2.3) em risco do fornecedor.
3. **"Onboarding em 15 minutos"** — sem projeto, sem amarração de core; importa cadastro, valida, remete. Mata a objeção nº 1 contra Matera (implantação longa e dependente).
4. **"Preço público"** — os 4 concorrentes diretos são todos quote-only; preço na página encurta procurement e sinaliza confiança (híbrido: preço público como âncora, contrato enterprise quando exigido).
5. **"Cada norma nova, leiaute atualizado no dia"** — IN BCB 694 (dez/2025) mudou o leiaute DRSAC sem aviso de mercado amplo; atualização de leiaute contínua é o produto.

### 7.3 Marca: manter "Radiant Norma" ou trocar?

- **Risco:** "Norma" já é a marca do produto do cadoc.ai (Norma, NormaChat, NormaCast) — confusão de search e marca registrada alheia em uso comercial público.
- **Recomendação:** Radiant Norma segue como **entidade legal/investimento**; lançar **marca de produto BR-nativa, curta e pesquisável** (critérios: contém/evoca "CADOC", "remessa" ou "norma regulatória" sem colidir; domínio .com.br livre; pronunciável em PT; passa em procurement). Candidatos a testar com 10 ICPs: nomes na linha de *Remessa/Cadoc<*>/Envia* — decisão por pesquisa de marca e disponibilidade INPI antes de Q1/2027. (Estimativa de custo de rebrand: baixa — produto ainda pré-launch.)

### 7.4 "Certificação/approval no BACEN": vale?

- **Não existe** programa de homologação/aprovação de software pelo BCB para CADOC; a aprovação de facto é o arquivo passar no validador do **CRD** (<https://www3.bcb.gov.br/crd>) e manter histórico limpo de remessas.
- **Vale sim:** adesões setoriais (**ABBC**, **ABFintechs**, **Zetta**) — canal de distribuição e selo de credibilidade; e **ISO 27001/SOC 2 Type II** + LGPD, que são o que procurement bancário realmente pede em due diligence de terceiros.
- **Não vale:** gastar com "parceria BACEN" (não há) ou com marketplace (não existe).

### 7.5 Plano GTM 12 meses (síntese)

1. **Dez/2026:** beta fechado DRSAC com 5–10 IFs S1-S4 (urgência da primeira remessa) — captação via FBM/consultorias.
2. **Q1/2027:** launch Rust com Free Validador (PLG) + pricing público + 2 parceiros consultoria.
3. **Q2/2027:** conteúdo SEO/LinkedIn ("como enviar DRSAC sem consultoria"), webinar com ABBC/ABFintechs; primeiro white-label BaaS.
4. **Q3/2027:** benchmarking data como tier premium (após ~30 clientes contribuintes); caso de uso "zero autuação" documentado.
5. **Q4/2027:** primeiro Enterprise S3-S4; meta base: R$210k MRR.

---

## 8. Fontes

**Oficiais BCB (dados primários):**

1. OData Instituições em Funcionamento (contagens por segmento, nov/2026) — <https://olinda.bcb.gov.br/olinda/servico/Instituicoes_em_funcionamento/versao/v1/odata/>
2. Dataset Relação de Instituições em Funcionamento — <https://dadosabertos.bcb.gov.br/dataset/relacao-de-instituicoes-em-funcionamento-no-pais>
3. FAQ DRSAC Doc 2030 (PDF; obrigação S1-S4 + IPs, data-base dez/2026, prazo 10º dia útil do 2º mês) — <https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/drsac2030/Perguntas-Respostas-DRSAC.pdf>
4. Leiaute DRSAC — <https://www.bcb.gov.br/estabilidadefinanceira/leiaute-documento-2030>
5. SCR Doc 3040 — <https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040> · Doc 3050 — <https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3050> · FAQ 3050 — <https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3050_faq>
6. Catálogo de documentos CADOC — <https://www.bcb.gov.br/estabilidadefinanceira/catalogodocumentos> · CRD — <https://www3.bcb.gov.br/crd>
7. Segmentação prudencial (listas S1-S5) — <https://www.bcb.gov.br/estabilidadefinanceira/regprudencialsegmentacao>
8. Penalidades BCB / Lei 13.837/2019 — <https://www.bcb.gov.br/meubc/faqs/p/tipos-de-penalidades-aplicadas-pelo-banco-central> · Res. BCB 131/2021 — <https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Resolu%C3%A7%C3%A3o%20BCB&numero=131>
9. Fintechs/SCD/SEP (autorização) — <https://www.bcb.gov.br/estabilidadefinanceira/fintechs> · IPs — <https://www.bcb.gov.br/estabilidadefinanceira/instituicaopagamento>

**Mercado BR:**

10. Dados Bancários — distribuição S1-S5 — <https://dadosbancarios.com.br/analise-segmentos>
11. Brazil Journal — Matera (250+ IFs, rebrand) — <https://braziljournal.com/brands/matera-muda-marca-para-consolidar-sua-evolucao-e-lideranca-em-tecnologia-para-o-mercado-financeiro>
12. Finsiders — Matera compra Fáciltech — <https://finsidersbrasil.com.br/noticias-sobre-fintechs/matera-compra-faciltech-na-maior-aquisicao-de-sua-historia>
13. cadoc.ai (pivot "Norma") — <https://cadoc.ai/sobre>
14. Dattos — <https://www.dattos.com.br>
15. Credisoft — CADOC 3044 — <https://credisoft.com.br/cadoc-3044>
16. ClickCompliance — custo de compliance terceirizado — <https://clickcompliance.com/compliance/profissional-de-compliance-quanto-custa>
17. Baita — compliance fintech B2B R$400k–1,5 mi/ano — <https://baita.ac/insights/compliance-para-fintech-b2b-guia-pratico-mm3wlv4f>
18. Celcoin — custo de operar IF licenciada — <https://celcoin.com.br/articles/custo-para-lancar-fintech-brasil>
19. Azify — Res. BCB 494/2025 (autorização prévia de IP) — <https://azify.com/blog/licencas-banco-central-fintechs>
20. ABBC — <https://abbc.org.br> / cursos — <https://abbc.org.br/cursos> / IN 694 — <https://abbc.org.br/instrucao-normativa-bcb-n-694-instrucoes-drsac>
21. ABFintechs (724→588 associadas; ~2.100 fintechs) — cobertura Valor (via pesquisa; portal — <https://abfintechs.com.br>) · Zetta — <https://somoszetta.org.br/quem-faz>
22. FBM Educação — Especialista em CADOCs — <https://fbmeducacao.com.br/blog/o-curso-especialista-em-cadocs-e-para-mim-um-guia-para-profissionais-de-contabilidade-riscos-e-compliance>
23. ARR Consultoria — <https://www.arrconsult.com.br> · Merc Group — <https://www.mercgroup.com.br/servicos/riscos-compliance-e-regulatorio/fintechs-instituicoes-de-pagamento-e-scd>
24. CVM/Anbima — ~3.000 FIDCs (acordo jan/2025) — <https://www.anbima.com.br>

**Global (pricing/RegTech):**

25. ERP Research — AutoRek/Duco/ReconArt pricing — <https://www.erpresearch.com/erp-add-ons/reconciliation/autorek> e <https://www.erpresearch.com/erp-add-ons/reconciliation/duco>
26. Vendr — Workiva (~US$49,4k mediana), Trintech (~US$177,6k), Duco (~US$182,8k), Vanta — <https://www.vendr.com>
27. Safeheld — Duco alternative (volume) — <https://safeheld.com/resources/duco-alternative>
28. Regnology — RegOps/custo de regulatory reporting (~US$70 bi) — <https://www.regnology.net/ecomaXL/files/RegOps-Whitepaper-Regnology-A-new-approach-towards-the-future-of-regulatory-reporting.pdf>
29. Vanta pricing — <https://www.vanta.com/pricing> · <https://www.trycomp.ai/vanta-pricing> · <https://www.strac.io/blog/vanta-pricing>

**Benchmarks SaaS:**

30. Benchmarkit 2025 — <https://benchmarkit.ai/2025benchmarks>
31. ScaleXP — CAC payback — <https://scalexp.com/blog/saas-benchmarks-cac-payback>
32. Metronome — State of UBP 2025 — <https://metronome.com/state-of-usage-based-pricing-2025>
33. Growth System — benchmarks startups BR — <https://growthsystem.com.br/en/benchmarks-startups>
34. PortalERP — Totvs 2T26 (ARR R$7,7 bi) — <https://portalerp.com.br/br/noticia/totvs-registra-ebitda-de-r-487-milhoes-no-2t26-com-avanco-da-ia-e-da-receita-recorrente> · <https://ri.totvs.com>
35. Komunikativa/6sense — ciclo de vendas B2B — <https://komunikativa.com/blog/ciclo-de-vendas-b2b>

---

*Relatório gerado em 16/nov/2026. Contagens BCB extraídas em tempo real das APIs públicas na data — recomenda-se re-executar as consultas OData no dia do board para números correntes.*
