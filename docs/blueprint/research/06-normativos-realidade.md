# 06 — Realidade Regulatória BACEN (verificação ao vivo em 16/nov/2026)

> **Método**: verificação direta nas páginas oficiais do BCB (bcb.gov.br, renderizadas ao vivo em 16/11/2026), no Portal Olinda/dados abertos do BCB e em fontes de mercado (mercado regulatório/compliance BR). Cada item traz: o que está vigente, o que muda, prazos e URLs.
> **Nota de confiança**: itens marcados `[VERIFICADO — fonte oficial, 16/11/2026]` foram lidos diretamente no site do BCB no dia da pesquisa. Itens de mercado (law firms/consultorias) são marcados `[MERCADO]`.

---

## TL;DR — O que muda para o produto

1. **Tese jul/2026 estava ~80% correta**: IN 694/2025 (DRSAC) vigente desde dez/2026; IN 733/2026 muda o 3040 em 3 ondas (mai, jul e **nov/2026 — esta última acabou de entrar**); STA/WS com `sta-h.bcb.gov.br/staws` + senhaws (`www3.bcb.gov.br/senhaws` produção / `www9.bcb.gov.br/senhaws` homologação) continuam idênticos.
2. **Erros/desatualizações da tese**: DRSAC aplica-se a **S1–S4 (S5 dispensado)** e é **semestral** (jun/dez) — não há cronograma S1-S5 de adesão; CC 3989 está **revogada** (era câmbio/ouro, não SCR) e Circular 3958 é **Open Finance/dados abertos**, não SCR; "PSTA" não é um "plano" — é a transação Sisbacen **PSTA300** que credencia operadores do STA.
3. **Novidades que a tese não capturou**: **Res CMN 5.296/2026 + Res BCB 560/2026** (LCR estendido ao S2 e novo LCRS para S3/S4, fase de 90% em 1º/1/2027→30/6/2027 e 100% em 1º/7/2027); **Res BCB 586/2026 + IN BCB 772/2026** (GRSAC novo, vigência 1º/1/2027); **IN BCB 755/2026** (DRL 2160 novo leiaute desde **jul/2026**, aplicável a **S1–S4**); **doc 2090** (ICAAP/TEBU, IN 685/2025); **docs 5710/5711** (ativos virtuais/cripto, Res BCB 520/2025 + IN 713/2026); **Res BCB 470/2025** (RWA carteira de negociação — FRTB padronizado, vigência **2027**); **Resolução Conjunta CMN/BCB 14/2025** (capital social mínimo, 679 IFs se adequando até **31/12/2026**).
4. **Universe (contagem oficial BCB via API Olinda, 16/11/2026)**: 1.722 entidades — 154 bancos/Caixa, 194 IPs, 138 SCDs, 80 financeiras (SCFI), 736 cooperativas de crédito, 105 corretoras (TVM+câmbio), 101 distribuidoras, 129 consórcios, 21 SCMs, 11 SEPs, etc.
5. **2027 é o ano da dor**: LCR/LCRS (jan e jul), GRSAC novo (1º/1), FRTB padronizado (2027), subconsolidado DRL sem penalidade só até 31/12/2026, dedução compulsório-poupança +1,5 p.p./ano a partir de 2027.

---

## 1. DRSAC 2030 — IN BCB 694/2025 e a família SAC/GRSAC

### 1.1 Linhagem normativa do CADOC 2030 `[VERIFICADO]`
Fonte: https://www.bcb.gov.br/estabilidadefinanceira/leiaute-documento-2030

| Norma | Data | Papel |
|---|---|---|
| Resolução BCB 151/2021 | 6/10/2021 | Base (DRSAC como exigência) |
| IN BCB 222/2021 | 28/12/2021 | Procedimentos de elaboração/remessa do 2030 |
| IN BCB 328/2022 | 23/11/2022 | Alterações |
| IN BCB 423/2023 | 24/11/2023 | Alterações |
| **IN BCB 694/2025** | **23/12/2025** | **Novas Instruções de Preenchimento (reescritas em formato consolidado) + novo Leiaute, vigentes a partir da data-base de dezembro de 2026** |

Texto integral da IN 694 (lido ao vivo): https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Instru%C3%A7%C3%A3o%20Normativa%20BCB&numero=694

### 1.2 Conteúdo da IN 694/2025 `[VERIFICADO]`
- **Vigência**: data-base **dez/2026** (confirmado na página do leiaute: "(Em vigor) Leiaute do documento 2030 a partir de dez/26"; leiaute antigo "(Obsoleto) até jun/26").
- Renomeia grupos ("Exposição ao Risco de Ativos", "Exposição ao Risco do Setor Econômico"), corrige tags/atributos, consolida instruções.
- Novos fatores/códigos de avaliação: Anexo 08 (risco climático físico — novos fatores, ex. "04 Intempéries: seca", "05 chuvas intensas", "06 outras") e Anexo 09 (novos níveis de exposição) — códigos antigos ("02 Intempéries") aceitos apenas em registros já existentes até dez/2026; depois, obrigatórios os novos.
- Transição extra: códigos 01–04 (CNAE/CPF-CNPJ/IPOC) não serão aceitos em registros inexistentes no documento de **junho de 2027**.
- Emissor: **Desig** (Departamento de Monitoramento do Sistema Financeiro).

### 1.3 Quem envia, quando `[VERIFICADO — FAQ oficial DRSAC]`
Fonte: FAQ PDF https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/drsac2030/Perguntas-Respostas-DRSAC.pdf

| Pergunta | Resposta oficial |
|---|---|
| Quem é obrigado | **S1 a S4, inclusive IPs** enquadradas nesses segmentos. **S5 dispensado** |
| Periodicidade | **Semestral** — datas-bases **junho e dezembro** |
| Prazo de envio | Até o **10º dia útil do 2º mês subsequente** à data-base |
| Sem exposição | Enviar "documento zerado" via **CRD** (https://www3.bcb.gov.br/crd) |
| Validação local | **Validador XML** (aplicativo Windows, uso do **XSD** distribuído pelo BCB) |
| Críticas | **Não há planilha pública de críticas** para o 2030 (diferente do 3040/3050). Regras semânticas espalhadas nas instruções + FAQ; regras BCB-side não publicadas em planilha |

**Conclusão item 1**: a premissa da tese ("críticas restritas ao login BACEN") está **correta na prática** para o 2030: só há XSD público + validador + FAQ; a planilha de críticas estilo 3040/3050 não existe para o 2030.

### 1.4 PRSAC / GRSAC (Res CMN 4945/2021 e Res CMN 4557/2017) `[VERIFICADO]`
- Resolução CMN 4557/2017 (SGRAC) e CMN 4945/2021 (PRSAC) continuam sendo a base obrigacional.
- **Novo (set/2026)**: **Resolução BCB 586, de 3/9/2026** — divulgação do Relatório GRSAC, **substitui a Res BCB 139/2021**; **IN BCB 772, de 3/9/2026** — **tabelas padronizadas** do Relatório GRSAC, **revoga a IN BCB 153/2021**, **vigor 1º/1/2027**. Impacta instituições S1–S4.
  - Res 586: https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Resolu%C3%A7%C3%A3o%20BCB&numero=586
  - IN 772: https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Instru%C3%A7%C3%A3o%20Normativa%20BCB&numero=772
  - Origem: Consulta Pública BCB 517/2026 (jan/2026) sobre o novo modelo do relatório `[MERCADO]`.
- **Correção à tese**: as INs 222/328/423 não foram "mexidas" por outras INs além da 694; a atualização relevante de 2026 foi na **divulgação** (GRSAC), não no CADOC.

---

## 2. SCR — 3040/3042/3044/3050

### 2.1 Base normativa vigente `[VERIFICADO]`
Fonte: https://www.bcb.gov.br/estabilidadefinanceira/leiautedocumentoscrd (linha "3026 3040 3042 3044 — SCR")

- **Resolução CMN 5.037/2022** (consolida o SCR), **Circular 3.870/2017**, **Carta-Circular 3.869/2018** (procedimentos/leiautes; alterada por IN 530/2024, IN 733/2026 e outras).
- **IN BCB 602/2025** — regras de remessa dos docs 3040 e 3044.
- **IN BCB 682/2025 (19/11/2025)** — formaliza a exigência do **documento 3044 — Dados de Eventos em Operações de Crédito** a partir de **1º/11/2025** (concessões, pagamentos, liquidações, renegociações etc. — reporte de eventos, não de foto).

### 2.2 IN BCB 733/2026 — 3 ondas confirmadas `[VERIFICADO]`
Texto integral lido ao vivo: https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Instru%C3%A7%C3%A3o%20Normativa%20BCB&numero=733 (IN de 4/5/2026, Desig)

| Onda | Marca no leiaute | Data-base de vigência | Conteúdo principal |
|---|---|---|---|
| 1 | **(NR)** | **maio/2026** | Anexo 26 InfosAdicionais: domínios 04/06 ganham 4 novos registradores/subdomínios (12 CIP/NUCLEA, 13 TAG IMF, 14 SPC Grafeno, 15 QS – Quick Soft); descrição de "Renda do mês" (ContInstFinRes4966) |
| 2 | **(NR2)** | **julho/2026** | Domínio 14 (Vinculação Legal): exclusão do subdomínio 05; domínio 21 (Empréstimo entre pessoas): marcação de **Investidor Qualificado** (CVM 30/2021); Anexo 37 Tipo de Uso Regulatório: domínio 12 — crédito imobiliário (Res 4.676/2018 + Res BCB 188/2022); **novos Anexos 50 (Especificação crédito imobiliário) e 51 (Complemento crédito imobiliário)** |
| 3 | **(NR3)** | **novembro/2026** | Anexo 8 CaracEspecial: domínio 25 passa a se chamar "Limite sujeito à provisão" |

- **Motivação declarada**: Res CMN 5.255/2025 + Res BCB 512/2025 (direcionamento de poupança e compulsório — IFs passam a marcar quais operações servem de dedução do compulsório; dedução cresce +1,5 p.p./ano **a partir de 2027**, teto 20%).
- **Implicação produto**: 3 migrações de domínios/anexos em 6 meses — o validador/generador Rust precisa de leiaute versionado por data-base (mecânica idêntica à já prevista, mas com uma onda **a mais em nov/2026**).

### 2.3 3044 (eventos) e 3050 (estatísticas) `[VERIFICADO]`
- **3044**: leiaute criado pela IN 530/2024; remessa conforme IN 602/2025; obrigatório desde **1º/11/2025** (IN 682/2025). Instruções: https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/SCR_InstrucoesDePreenchimento_Doc3044.pdf . O 3044 é XML (não "JSON"); fluxo é **diário/eventual por eventos**.
- **3050**: leiaute e **críticas públicas** por período; versão vigente **"a partir de 07/11/2025"** (leiaute + dicionário + críticas XLSX). Fonte: https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3050
- **Equivalências 3040↔3050** e modalidades publicadas pelo Desig: disponíveis na página do 3040.

### 2.4 Correções à tese
- **CC 3989**: revogada desde 1º/7/2021 (tratava de exposições em ouro/câmbio — hoje sob IN BCB 118/2021/doc 2011). Não é normativa SCR.
- **Circular/CC 3958 (2019)**: trata de **dados abertos (Open Finance)**, não de SCR. As cartas-circulares SCR relevantes hoje são **CC 3.869/2018** e CC 3.659/2014 (3050).
- **"3044 eventos JSON"**: o 3044 é XML. JSON existe hoje em outro lugar: **9010/9011/9030/9031** (demonstrações financeiras) com remessa em **JSON** desde a IN BCB 601/2025 (Json Schema público para 9011/9031). `[VERIFICADO — leiautedocumentoscrd]`

---

## 3. Capital e liquidez — o que é vigente e o que chega em 2027

### 3.1 Arcabouço basal (vigente)
- **Res CMN 4.955/2021** — PR e alavancagem: https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Resolu%C3%A7%C3%A3o%20CMN&numero=4955 `[VERIFICADO]`
- **Res CMN 4.956/2021** — requisitos mínimos de capital (RWA); **Res CMN 4.958/2021** + **Res BCB 111/2021** (carteira de negociação); **Res CMN 4.959/2021** (operacional) etc.
- **Res CMN 4.966/2021** — gerenciamento de riscos prudenciais e **segmentação S1–S5** (a tese acerta ao citá-la como referência de segmentos).
- Transições internas confirmadas na leitura da CMN 4.955: reconhecimento de fundos (FGC/FPS) no Nível II cai 30%→20% em **2027**, 10% em **2028**, 0% em **2029**; transitionais de provisionamento (CLFPS "pisos") terminam — **50% em 2026, 25% em 2027, 0% a partir de 1º/1/2028** `[VERIFICADO — texto da norma]`.
- **Output floor (piso de capital)**: régua final de Basileia (72,5% do RWA padronizado) em transição anual; fontes de mercado indicam conclusão da fase final no ciclo **2026–2028** (adensamento +5 p.p./ano), com piso pleno no fim da régua brasileira `[MERCADO]` — recomendação: tratar como constante anual em versão do motor de capital, não hardcode.

### 3.2 Liquidez — o grosso de 2027 `[VERIFICADO/MERCADO]`
| Norma | Data | O que faz | Vigência/prazo |
|---|---|---|---|
| Res CMN 4.909/2021 | 2021 | LCR (exigível ao S1) | vigente |
| Res CMN 4.914/2021 | 2021 | NSFR | vigente |
| **Res CMN 5.296/2026** | 23/4/2026 | Limites mínimos **LCR e LCRS** para **S1–S4** | fase: **90% em 1º/1/2027–30/6/2027**; **100% a partir de 1º/7/2027** |
| **Res BCB 560/2026** | 23/4/2026 | Complementa: LCR ao **S2**; **LCRS** (com ALAQ simplificado) para **S3/S4** com captação de público | idem |
| Res BCB 207/2022 + IN BCB 399/2023 | — | Remessa DRL/LCR (doc 2160) | vigente |
| **IN BCB 755/2026** | 1/7/2026 | Novo leiaute/instruções do **2160 DRL modelo I** — válido desde **data-base julho/2026**; DRL aplicável a **S1–S4** (com Res BCB 436/2024 e Res BCB 577/2026); envio subconsolidado em homologação, substituição sem penalidade **até 31/12/2026** | jul/2026 |
| Res BCB 577/2026 | 23/6/2026 | Apresentação em base subconsolidada (art. 13-A CMN 4.950) | 2026 |

Fontes: CMN 5.296 — https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Resolu%C3%A7%C3%A3o%20CMN&numero=5296 ; BCB 560 — https://www.legisweb.com.br/legislacao/?legislacao=494808 ; IN 755 — https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Instru%C3%A7%C3%A3o%20Normativa%20BCB&numero=755 `[VERIFICADO]` ; análise de mercado — https://www.mercgroup.com.br/insights/liquidez-lcr-lcrs-cmn-5296-bcb-560-segmento-s2-s3-s4-ativos-alta-liquidez-indicador-monitoramento-controles-auditoria `[MERCADO]`

- **Correção à tese**: "Res BCB 131" é o Regulamento do acordo de leniência (processo administrativo sancionador), **não liquidez**; "Res BCB 347" é **IN BCB 347/2023** (requisitos mínimos de sistemas internos de liquidez), **revogada pela IN BCB 589/2025** desde 1º/4/2025. As normas de reporte de liquidez hoje: Res BCB 207/2022 + IN 399/2023 (+IN 755/2026).

### 3.3 Risco de mercado (FRTB-BR) `[MERCADO]`
- **Res BCB 470/2025 (30/4/2025)** — abordagem **padronizada** do RWA da carteira de negociação (RWA SENS), alinhada ao FRTB; altera Res BCB 200/111/265/313; **vigência a partir de 2027**.
- **Res BCB 592/2026 (30/9/2026)** — altera 5 normas prudenciais de uma vez (classificação em carteira de negociação; novos ponderadores CSR NSEC).
- Fonte: Pinheiro Guimarães (2025), resumos de mercado.
- **Impacto produto**: novo conjunto de campos de carteira de negociação nos CADOCs de mercado (2036/2060 família) a partir de 2027 — módulo de leiautes precisa estar pronto para versão 2027.

### 3.4 Capital social mínimo / PL `[MERCADO]`
- **Resolução Conjunta CMN/BCB nº 14, de 3/11/2025** — nova metodologia do limite mínimo de capital social integralizado e PL; ~**679 instituições** precisam se adequar, com cronograma até o fim de **2026** (CNN/lefosse).

---

## 4. STA / senhaws / PSTA — status 2026

Tudo `[VERIFICADO]` (páginas e manuais oficiais lidos em 16/11/2026).

### 4.1 O que vigora
- STA instituído pela **Carta-Circular 3.588/2013**; HTTPS; arquivos ZIP/GZ; integridade **SHA-256**.
- **Ambientes**: produção `https://sta.bcb.gov.br/sta` e WS `https://sta.bcb.gov.br/staws`; homologação `https://sta-h.bcb.gov.br/sta` e WS `https://sta-h.bcb.gov.br/staws`.
- **Senhaws**: produção `https://www3.bcb.gov.br/senhaws`; homologação `https://www9.bcb.gov.br/senhaws` (endpoints REST: `/senhaws/senha`, `/senhaws/senha/vencimento` etc. — Manual STA Web Services).
- **WS STA é REST/HTTPS** (Basic Auth + certificado digital ICP-Brasil; endpoints `/staws/arquivos`, `/staws/arquivos/disponiveis`, `/staws/arquivos/{protocolo}/conteudo`).
- **Credenciamento**: usuário máster Sisbacen credenciado na transação **PSTA300**; reabilitação de senha via **BC Correio** (bccorreio.bcb.gov.br) ao Deinf/Diate.
- Manuais: https://www.bcb.gov.br/content/acessoinformacao/sisbacen_docs/Manual_STA_Web_Services.pdf ; FAQ: https://www.bcb.gov.br/content/acessoinformacao/sisbacen_docs/STA_FAQ.pdf ; página: https://www.bcb.gov.br/acessoinformacao/sistematransferenciaarquivos

### 4.2 O que NÃO mudou / o que mudou
- **Não há migração/substituição do STA em curso** nem "adesão" nova: nenhum comunicado de fim do STA ou novo canal de envio foi encontrado em 2026. A tese de jul/2026 estava **correta e ainda atual**.
- **"PSTA"**: não é "Plano STA" — é a transação PSTA300. Corrigir nomenclatura no produto.
- Mudanças 2026 foram **de conteúdo, não de canal**: novos documentos no STA (C225/C230 eFX desde 1º/10/2026 — IN 747/2026; C212 leiaute atualizado 8/9/2026 — IN 693/2025; XSD ACNV1501 novo desde jul/2026).

---

## 5. 4111 / 4060 / 4066 / 4076 / DRO 5050 — vigência e mudanças

Fonte única: https://www.bcb.gov.br/estabilidadefinanceira/leiautedocumentoscrd `[VERIFICADO]`

| Doc | Nome | Base normativa | Versão vigente / mudança recente |
|---|---|---|---|
| **4111** | Saldos Contábeis Diários | Res BCB 208/2022 + **IN BCB 524 (18/9/2024)** | Leiaute e XSD "(Após 2/1/25)" — sem mudança adicional em 2026 |
| 5005 | CDB — depósitos bancários | Res BCB 208/2022 + IN BCB 411/2023 | vigente |
| 5500 | Poupança diária | Res BCB 208/2022 + IN BCB 355/2023 | vigente |
| **4060/4066** (e 4010/4016/4090/4096/4413/4423/4433/4500/4510) | COSIF | "Ver normativos" (Res CMN 4.950/2021 e correlatas) | Leiautes públicos; sem onda de mudança identificada em 2026 |
| **4076** | Relatório do Conglomerado Prudencial | Res CMN 4.911/2021 + Res BCB 146/2021 + IN BCB 311/2022 | **Leiaute e XSD "a partir de 01/2025"** (versão anterior até 12/2024) — nenhuma alteração nova em 2026 |
| **5050** | DRO | Circular 3.979/2020 + IN BCB 33/2020 + IN BCB 117/2021 | vigente; sem mudança identificada |
| **2011** (tese chamou "2070 cód 2011") | DDR — parcelas de requerimento de capital e limites operacionais | Circular 3.947/2019 + Res BCB 100/2021 + IN BCB 118/2021 | vigente |
| 2060/2061/2062 | DRM / DLO / DLI | Res BCB 84/2021+IN 101/2021; CMN 4.955/4.957/4.958/4.995; IN 85/2021 + CMN 5.237/2025 (DLI) | DLI recebeu Res CMN 5.237/2025 |
| 2160/2170 | DRL / DLP | ver seção 3.2 | DRL mudou em **jul/2026** (IN 755) |
| **2090** | **ICAAP/ICAAPSimp/TEBU** | **IN BCB 685 (5/12/2025)** | **Novo documento** com leiaute, **XSD** e Cenários TEBU (data-base dez/25) — atenção para GTM (reporte de ICAAP + testes de estresse TEBU) |
| 9010/9011/9030/9031 | Demonstrações financeiras | CMN 4.818/4.911/4.950/4.966 + **IN BCB 601 (27/3/2025)** | **Remessa em JSON disponível** (Json Schema 9011/9031 público) |
| 5710/5711 | Ativos virtuais (provas de reserva/staking; custódia) | **Res BCB 520 (10/11/2025) + IN BCB 713 (27/2/2026)** | 5710: mensal (data-base último dia do mês, envio até 5 dias úteis); 5711: diário — aplica-se a VASPs |
| 3060 | Juros acumulados em cartões | Res BCB 468/2025 + IN BCB 621/2025 | novo em 2025 |

---

## 6. BCValidador, críticas, XSDs e APIs públicas de leiaute

`[VERIFICADO]` — páginas do SCR (scrdoc3040), DRSAC (leiaute-documento-2030), 3050 (scrdoc3050) e validador XML genérico (validador_xml_info).

### 6.1 Como o BCB distribui validação hoje
| Camada | Artefato público | URL base |
|---|---|---|
| SCR 3040 | **Aplicativo Validador (ZIP)** — release **14345** (atual) / 13891 (anterior); **Manual de Utilização**; **Planilha de Críticas (XLS)** — regras sintáticas/semânticas; **Regras de Validação Bacen (XLS)** — regras intra/interdocumentos aplicadas no BCB | https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040 |
| DRSAC 2030 | **Validador XML** (aplicativo Windows) + **XSD** dentro do ZIP do leiaute; FAQ; **sem planilha de críticas** | https://www.bcb.gov.br/estabilidadefinanceira/leiaute-documento-2030 |
| 3050 | Validador próprio + **Críticas de validação XLSX públicas por período** (vigente: a partir de 07/11/2025) | https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3050 |
| 7110/7120 (auditoria cooperativa) | Leiaute + XSD + **"Programa Validador"** | leiautedocumentoscrd |
| Validador XML genérico | **BCValidador** — programa em **Java** (requer JRE), distribuído como ZIP (~2,7 MB), doc técnico de 2013/2010 — usado p.ex. para o 2030 | https://www.bcb.gov.br/estabilidadefinanceira/validador_xml_info |
| Respostas BCB | **CRD** (Controle de Remessa de Documentos, https://www3.bcb.gov.br/crd) + **SCR2 Painel de Gestão** (dispensa de envio, autorização de envio por terceiros, Q72) | scrdoc3040 |

### 6.2 XSDs oficiais disponíveis (públicos)
2030 (no ZIP do leiaute), 2090, 3026, 3045 (consulta), ACNV1501 (desde jul/2026), 4076, 4111, 5300, 5401/5402, 5500, 5710/5711, 7110/7120, JSON Schema 9011/9031. **Sem XSD público para 3040/3050** (leiaute XLSX + planilha de críticas).

### 6.3 "SGT" e APIs de leiaute
- Não há API pública REST de leiautes ("SGT" citado na tese não existe como catálogo de leiautes CADOC; o catálogo é a página **Leiaute de arquivos e base normativa** — `/estabilidadefinanceira/leiautedocumentoscrd`).
- APIs públicas relevantes do BCB (para enriquecer produto, não para envio): **Olinda OData** (ex.: Instituições_em_funcionamento — usada neste relatório para contagens), **SGS** (`api.bcb.gov.br/dados/serie/...`), dados abertos CKAN (dadosabertos.bcb.gov.br).
- **Implicação de produto**: motor de leiautes deve ingerir os XLS/ZIP/XSD oficiais (download programático das URLs estáveis `/content/estabilidadefinanceira/...`) e versionar por data-base; a UX de "dif de leiaute" entre ondas (NR/NR2/NR3) é diferenciador direto.

---

## 7. Universo de IFs — números oficiais (16/11/2026)

Fonte: API Olinda BCB "Instituicoes_em_funcionamento" (dados da Relação de Instituições em Funcionamento) — consulta executada em 16/11/2026 `[VERIFICADO]`
Endpoint: https://olinda.bcb.gov.br/olinda/servico/Instituicoes_em_funcionamento/versao/v1/odata (entidades SedesBancoComMultCE, SedesSociedades, SedesCooperativas, SedesConsorcios)

| Segmento | Quantidade (sedes, 16/11/2026) |
|---|---|
| Banco Múltiplo (incl. BB/CEF) | 129 (+1 BB específico) |
| Banco Comercial | 14 |
| Banco Comercial Estrangeiro — filial | 4 |
| Banco Múltiplo Cooperativo | 2 |
| Banco de Câmbio | 4 |
| **Instituição de Pagamento (IP)** | **194** |
| **Sociedade de Crédito Direto (SCD)** | **138** |
| Sociedade de Crédito ao Microempreendedor (SCM) | 21 |
| Sociedade de Empréstimo entre Pessoas (SEP) | 11 |
| Sociedade de Crédito, Financiamento e Investimento (financeiras) | 80 |
| Sociedade Corretora de TVM | 57 |
| Sociedade Corretora de Câmbio | 47 |
| Sociedade Distribuidora de TVM | 101 |
| Sociedade de Arrendamento Mercantil | 16 |
| Companhia Hipotecária | 5 |
| Agência de Fomento | 16 |
| Banco de Investimento | 12 |
| Banco de Desenvolvimento + BNDES | 4 |
| Associação de Poupança e Empréstimo | 1 |
| **Cooperativas de crédito** | **736** |
| Administradoras de consórcio | 129 |
| **TOTAL** | **1.722** |

Corroboração de mercado: BC monitora ~1.751 instituições; 679 se adequando à nova régua de capital mínimo (RC 14/2025) — https://www.cnnbrasil.com.br/economia/money/bc-muda-metodologia-do-limite-minimo-de-capital-de-instituicoes-financeiras `[MERCADO]`

SPI (dez/2025, Relatório Anual SPI): 70 instituições bancárias, 87 financeiras não bancárias, **576 cooperativas**, 127 outras (IPs etc.) — https://www.bcb.gov.br/content/estabilidadefinanceira/relatorios_SPI/relatorio_anual_spi_2025.pdf `[VERIFICADO]`

**Tendência**: SCD/IP entraram em **consolidação** (138 SCDs/194 IPs em 2026 vs. picos anteriores >140/>200; o BCB endureceu autorizações desde 2023 e o ciclo de M&A/encerramento de licenças seguiu em 2025-26) `[MERCADO — qualitativo]`. Cooperativas (736 + confederações) e financeiras permanecem o volume estrutural de "long tail" regulatório — o segmento-alvo natural de um SaaS de CADOCs é **S5/S4 + IPs/SCDs que remetem SCR 3040 e 4111**.

**Segmentação S1–S5** (Res CMN 4.966/2021): classifica **conglomerados prudenciais** (vulto/porte/atividades internacionais); a composição exata dos S1–S3 é pequena (poucas dezenas), o bulk (S4/S5) concentra o long tail. Não há API pública com o enquadramento S por IF; obter via If.data/agências ou declaração da própria IF.

---

## 8. Calendário de dor regulatória 2026–2028 (por tipo de IF)

### 2026 (restante)
| Data-base/Evento | O quê | Quem |
|---|---|---|
| **dez/2026** | Novo leiaute/instruções **DRSAC 2030** (IN 694/2025); novos códigos Anexos 08/09 | S1–S4 (incl. IPs) |
| contínuo | **3040 ondas IN 733** — NR3 vigente desde data-base **nov/2026** ("Limite sujeito à provisão") | todos os remetentes SCR |
| jul–dez/2026 | **DRL 2160 novo leiaute** (IN 755); subconsolidado em homologação (substituição sem penalidade **até 31/12/2026**) | S1–S4 |
| **31/12/2026** | Adequação à **RC CMN/BCB 14/2025** (capital social mínimo/PL — ~679 IFs) | IPs, SCDs, cooperativas, demais |
| 1º/10/2026 | eFX novo leiaute C225/C230 (IN 747/2026) | prestadores de eFX |
| 2026 | Docs **5710/5711 cripto** já em produção (IN 713/2026); doc **2090** ICAAP/TEBU (IN 685/2025, cenários dez/25) | VASPs; S1–S3 (TEBU/ICAAP) |

### 2027 — o ano do GTM
| Quando | O quê | Quem |
|---|---|---|
| **1º/1/2027** | **Novo Relatório GRSAC** (Res BCB 586 + tabelas IN 772; revogam Res 139/2021 e IN 153/2021) | S1–S4 |
| **1º/1/2027** | **LCR/LCRS** limite mínimo em **90%** (Res CMN 5.296 + Res BCB 560) — S2 passa a calcular LCR; S3/S4 LCRS | S1–S4 |
| 2027 | **FRTB padronizado** — RWA SENS (Res BCB 470/2025) vigente | detentores de carteira de negociação (S1–S3 majoritariamente) |
| 2027 | **Compulsório-poupança**: dedução +1,5 p.p./ano (Res BCB 512/2025) → nova mecânica de marcação no SCR 3040 (Anexos 50/51) | detentoras de poupança |
| 30/6/2027 | Fim da fase de 90% do LCR/LCRS | S1–S4 |
| jun/2027 | Códigos 01–04 do DRSAC deixam de ser aceitos em registros novos (transição IN 694) | S1–S4 |
| **1º/7/2027** | LCR/LCRS em **100%** | S1–S4 |

### 2028–2029
| Quando | O quê |
|---|---|
| 1º/1/2028 | Transições de provisionamento (pisos CLFPS) = 0 (CMN 4.955); fundos no Nível II = 10% |
| 1º/1/2029 | Fundos no Nível II = 0%; fim pleno das transições do pacote Basileia final |
| contínuo | Output floor em fase final (72,5%) — acompanhar CMN 4.955/4.956 consolidadas |

### Leitura GTM (ordem de valor)
1. **Agora (dez/2026–jun/2027)**: migração DRSAC dez/26 + ondas 3040 + GRSAC jan/27 + LCR/LCRS jan/27 = janela de conversão máxima (dor simultânea em 4 frentes para S2–S4 e fintechs).
2. **Fintechs/long tail**: SCDs (138), SCMs (21), SEPs (11), IPs com carteira de crédito, financeiras (80), cooperativas singulares (736) — dor principal em **3040/3050/3044 + 4111 + SCR consultas**, não em capital (dispensas RC 14/4955 para S5, mas SCR obrigatório para todas as operantes).
3. **S2–S4**: upsell para módulos DRL (novo leiaute jul/26), LCR/LCRS (2027), 2090 TEBU/ICAAP.

---

## 9. IA e modernização no BACEN

`[MERCADO + VERIFICADO]`
- IA é **prioridade regulatória formal 2025/2026** do BCB (estudo de riscos/impactos do uso de IA pelas IFs): https://www.bcb.gov.br/detalhenoticia/20645/noticia
- BCB criou o **Centro de Excelência em Ciência de Dados e IA (CdE IA)** (ago/2024) com diretrizes de governança de IA (uso interno): https://www.bcb.gov.br/detalhenoticia/20292/noticia
- **Ainda não existe resolução dedicada de governança de IA** para IFs; consultas em curso (FSB sobre IA; ajustes de resiliência operacional/Res BCB 304 citados por mercado) — produto deve preparar "governança de IA" como recurso de compliance defensivo, não como requisito normativo hoje.
- **Modernização real do reporting**: remessa **JSON** (9010/9011/9030/9031 — IN 601/2025), **Scr consulta via Web Service** (3045/3046 + WS), **SCR2/Painel de Gestão** e CRD web, **dados abertos API Olinda/SGS**. O canal de envio continua STA-REST/XML — não há API "moderna" de envio CADOC além do STA.

---

## 10. Pendências de verificação (para o próximo ciclo de pesquisa)
1. Composição exata S1–S4 por conglomerado (não há API; exigiria mineração de If.data/relatórios dos conglomerados).
2. Texto consolidado do cronograma do output floor (arts. do Regulamento de Requisitos Mínimos de Capital — confirmar 2028 vs 2029 para 72,5%).
3. Detalhe das "Regras de Validação Bacen" do 3040 (a planilha XLS pública lista os códigos; extração completa cabe ao módulo de parser).
4. Impacto prático da Res BCB 577/2026 (subconsolidado) nos leiautes DRL por modelo (I/II).
5. Monitorar consultas públicas 2026-27: Res BCB 277 (aprimoramentos prudenciais — CP 126/2025), novo modelo GRSAC (CP 517/2026 já virou Res 586/IN 772).

---

## Anexo A — Fontes primárias (todas acessadas 16/11/2026)

**BCB — páginas oficiais**
- Leiautes e base normativa (catálogo mestre STA): https://www.bcb.gov.br/estabilidadefinanceira/leiautedocumentoscrd
- DRSAC 2030: https://www.bcb.gov.br/estabilidadefinanceira/leiaute-documento-2030
- IN BCB 694/2025 (texto): https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Instru%C3%A7%C3%A3o%20Normativa%20BCB&numero=694
- IN BCB 733/2026 (texto): https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Instru%C3%A7%C3%A3o%20Normativa%20BCB&numero=733
- IN BCB 755/2026 (texto): https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Instru%C3%A7%C3%A3o%20Normativa%20BCB&numero=755
- IN BCB 713/2026 (texto): https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Instru%C3%A7%C3%A3o%20Normativa%20BCB&numero=713
- Res BCB 586/2026: https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Resolu%C3%A7%C3%A3o%20BCB&numero=586
- IN BCB 772/2026: https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Instru%C3%A7%C3%A3o%20Normativa%20BCB&numero=772
- Res CMN 5.296/2026: https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Resolu%C3%A7%C3%A3o%20CMN&numero=5296
- Res CMN 4.955/2021: https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Resolu%C3%A7%C3%A3o%20CMN&numero=4955
- SCR doc 3040/3042/3044: https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040
- SCR doc 3050: https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3050
- FAQ DRSAC (PDF): https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/drsac2030/Perguntas-Respostas-DRSAC.pdf
- Instruções 3044 (PDF): https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/SCR_InstrucoesDePreenchimento_Doc3044.pdf
- STA: https://www.bcb.gov.br/acessoinformacao/sistematransferenciaarquivos ; Manual WS: https://www.bcb.gov.br/content/acessoinformacao/sisbacen_docs/Manual_STA_Web_Services.pdf ; FAQ: https://www.bcb.gov.br/content/acessoinformacao/sisbacen_docs/STA_FAQ.pdf
- Validador XML genérico: https://www.bcb.gov.br/estabilidadefinanceira/validador_xml_info
- CRD: https://www3.bcb.gov.br/crd (doc: https://www.bcb.gov.br/estabilidadefinanceira/controledocumentosif)
- Relação de instituições: https://www.bcb.gov.br/estabilidadefinanceira/relacao_instituicoes_funcionamento + API https://olinda.bcb.gov.br/olinda/servico/Instituicoes_em_funcionamento/versao/v1/odata
- Relatório Anual SPI 2025: https://www.bcb.gov.br/content/estabilidadefinanceira/relatorios_SPI/relatorio_anual_spi_2025.pdf
- Prioridades regulatórias 2025/2026: https://www.bcb.gov.br/detalhenoticia/20645/noticia ; CdE IA: https://www.bcb.gov.br/detalhenoticia/20292/noticia

**Fontes de mercado (compliance/fintech BR)**
- LCR/LCRS (CMN 5.296 + BCB 560): https://www.mercgroup.com.br/insights/liquidez-lcr-lcrs-cmn-5296-bcb-560-segmento-s2-s3-s4-ativos-alta-liquidez-indicador-monitoramento-controles-auditoria ; https://gsga.com.br/pt/informativo/resolucao-bcb-n-5602026-estabelece-limites-minimos-de-liquidez-de-curto-prazo-para-instituicoes-financeiras-e-de-pagamento ; https://souzaokawa.com.br/cmn-e-bcb-aprimoram-regras-de-liquidez-e-ajustam-regime-de-contribuicoes-ao-fgc
- BCB 560 (texto): https://www.legisweb.com.br/legislacao/?legislacao=494808
- Movimentos regulatórios início de 2026 (VASPs etc.): https://lefosse.com/noticias/os-movimentos-regulatorios-que-marcaram-o-inicio-de-2026
- Capital mínimo (RC 14/2025, 679 IFs): https://www.cnnbrasil.com.br/economia/money/bc-muda-metodologia-do-limite-minimo-de-capital-de-instituicoes-financeiras
- IN 694 (análise): https://abbc.org.br/instrucao-normativa-bcb-n-694-instrucoes-drsac ; https://cadoc.ai/post/instrucao-normativa-bcb-694 ; https://www.legisweb.com.br/legislacao/?id=488596 ; novos códigos 2030: https://www.b3bee.com.br/site/2026/06/03/novos-codigos-de-avaliacoes-do-2030 ; GRSAC S1–S4: https://www3.ibgc.org.br/blog/relatorio-banco-central-2026
- FRTB/Res BCB 470 e 592: resumos Pinheiro Guimarães/Mattos Filho (2025-2026, via busca — link primário: https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Resolu%C3%A7%C3%A3o%20BCB&numero=470)
- CC 3989 (revogada) e Circular 3958 (dados abertos): https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Carta%20Circular&numero=3989 ; https://www.bcb.gov.br/estabilidadefinanceira/exibenormativo?tipo=Circular&numero=3958
- SCR normas em vigor: https://www.bcb.gov.br/estabilidadefinanceira/scrnormasemvigor
- Balancetes: https://www.bcb.gov.br/estabilidadefinanceira/balancetesbalancospatrimoniais
