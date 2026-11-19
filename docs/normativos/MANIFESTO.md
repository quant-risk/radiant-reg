# MANIFESTO — Normativos a re-baixar (saneamento F0′, 17/11/2026)

Arquivos removidos do repo no saneamento porque eram **páginas de erro disfarçadas**
(sem magic bytes `%PDF`/`PK`/OLE2 — o mecanismo que impede isso de volta está no Loop 1
do harness, `docs/blueprint/22-HARNESS-V4.md`). Todo arquivo novo nesta pasta deve:

1. Ter magic bytes verificados (`%PDF` para PDF, `PK\x03\x04` para XLSX, OLE2 `\xd0\xcf` para XLS);
2. Registrar SHA-256 e data de captura abaixo;
3. Vir da fonte oficial (páginas BCB/CVM renderizadas via /browser — os shells Angular
   de 2.871 bytes não servem).

## Requisições

| # | Arquivo esperado | Fonte oficial | Substitui | Motivo da remoção |
|---|---|---|---|---|
| 1 | `STA_Manual.pdf` | Manual STA Web (portal BCB) | `_referencias/STA_Manual.pdf` (6.245B, Apache 404) | página de erro |
| 2 | `IN_BCB_222_DRSAC.pdf` (e atual: IN 694/2025) | `exibenormativo` BCB → download PDF | `_normativos/IN_BCB_222_DRSAC.pdf` (404) | página de erro; normativo trocado por IN 694/2025 |
| 3 | `IN_BCB_328_DRSAC.pdf` (idem) | idem | `_normativos/IN_BCB_328_DRSAC.pdf` (404) | página de erro |
| 4 | `Carta_Circular_3588_STA.pdf` | portal BCB | `_referencias/Carta_Circular_3588_STA.pdf` (725B, HTML SharePoint) | redirect de erro |
| 5 | `FAQ_Doc3050.html`/PDF | página do BCB (renderizada) | `3050/FAQ_Doc3050.html` (shell 2.871B) | shell SPA vazio |
| 6 | `COSIF_Leiautes_Page.html` | idem | `_referencias/COSIF_Leiautes_Page.html` (shell) | shell SPA vazio |
| 7 | `STA_Entenda.html` | idem | `_referencias/STA_Entenda.html` (shell) | shell SPA vazio |
| 8 | `STA_Page.html` | idem | `_referencias/STA_Page.html` (shell) | shell SPA vazio |
| 9 | `Validador_XML_Info.html` | idem | `_referencias/Validador_XML_Info.html` (shell) | shell SPA vazio |
| 10 | `Leiaute_Documentos_CRD.html` | idem | `_referencias/Leiaute_Documentos_CRD.html` (shell) | shell SPA vazio |

## Pacote prioritário do wedge V1 (S1, pós-IN 694/2025 + IN 729/2026)

- [x] 3040: leiaute + instruções + críticas vigentes (pós-IN 733 onda 3 nov/2026 + IN 735) — `https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040` — **Verificado 2026-11-19**: `3040/SCR3040_Criticas.xls` (246.784 B, OLE2, **255 regras**: `226 Habilitado=s`, `29 Habilitado=n`), `3040/SCR3040_Leiaute.xls` (290.304 B, OLE2, atualizado até IN 733/2026 e IN 735/2026), `3040/SCR3040_RegrasValidacaoBacen.xls` (181.248 B, OLE2), `3040/SCR_InstrucoesPreenchimento_Doc3040.pdf` (1.618.203 B, `%PDF-1.7`, SHA-256 idêntico ao portal BCB), `3040/Manual_Validador_SCR3040.pdf`, `3040/Manual_Envio_3040.pdf`, `3040/SCR3040_DetalhamentoFIDC.pdf` e `3040/SCR3040_Dispensa_Envio.pdf`.
- [x] 2030: leiaute + instruções + FAQ (data-base dez/2026) — `https://www.bcb.gov.br/estabilidadefinanceira/leiaute-documento-2030` — **Verificado 2026-11-19**: `2030/Leiaute-DRSAC-valido-a-partir-de-dez-2026.zip` (782.272 B, `PK\x03\x04`, contendo `1-Leiaute-DRSAC-valido-a-partir-de-dez-2026.xlsx`, `2-Instrucoes-de-preenchimento-DRSAC-validas-a-partir-de-dez-2026.docx`, `3-Leiaute_DRSAC_v2.03-valido-a-partir-de-dez-2026.xsd`, `4-Críticas-Validação-DRSAC - valido a partir de dez-26.xlsx` e `5-Perguntas-Respostas-DRSAC-validas-a-partir-de-dez-2026.pdf`) e `2030/Perguntas-Respostas-DRSAC.pdf` (437.269 B, `%PDF-1.7`).
- [x] Validador oficial SCR3040 (.jar) — página do `scrdoc3040` / BCValidador — **Verificado 2026-11-19**: `tools/bacen/SCR3040_Validador.zip` (11.605.657 B, `PK\x03\x04`, SHA-256 `e3f487af45ffa279684a2a8d77a09241c51767f61a45467414051666f077b04f`), descompactado em `tools/bacen/validador/` e ignorado no `.gitignore` (> 2 MB). **Observação**: o `.zip` oficial não possui um `.jar` único do validador; o código BCB vem em `tools/bacen/validador/classes/` (entrypoint CLI `br.gov.bcb.scr2.validador.linhacomando.ValidadorIfLinhaComando`, `Release 14345`, XSDs até `scr3040.202607.xsd`) acompanhado de 17 dependências `.jar` em `tools/bacen/validador/lib/`. *(Nota: `bcb.gov.br/estabilidadefinanceira/bcvalidador` retorna HTTP 404; o validador XML genérico encontra-se em `bcb.gov.br/estabilidadefinanceira/validador_xml_info`.)*
- [x] Literais verificados: RC 18/2025, RC 16/2025, INs 694/729/733/735, Comunicados 45.206/45.339 — **Verificado 2026-11-19**: consultados via Playwright/Chrome headless em `bcb.gov.br/estabilidadefinanceira/exibenormativo?...` e documentados em [`docs/normativos/LITERALS_BCB_2026.md`](LITERALS_BCB_2026.md).
- [x] `STA_Manual_WebServices.pdf` v1.5 (referência do port Rust do STA client) — **Verificado 2026-11-19**: `_referencias/STA_Manual_WebServices.pdf` (1.128.517 B, `%PDF-1.7`, capa "Versão 1.5 julho/2022", SHA-256 `9179ec02b930c1bd44744d784680b21f6ebcb4c1fccbfe1a5bc0f64df7559d58`), complementado pelos certificados oficiais em `_referencias/certs/`.

## Registro de capturas

Captura e verificação executadas em **2026-11-19**. Todos os arquivos foram validados contra HTML de erro (`6.245B`/`2.871B`) e conferidos por magic bytes e SHA-256 real.

| Caminho | Bytes | Magic bytes | SHA-256 | Fonte URL | Data |
|---|---:|---|---|---|---|
| `3040/SCR3040_Criticas.xls` | 246.784 | `OLE2 (d0cf11e0a1b11ae1)` | `163b679f018a2215bc062970e250863921c31c8bc3a4ecf4d788a0c14a355b6b` | `https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/SCR3040_Criticas.xls` | 2026-11-19 |
| `3040/SCR3040_Leiaute.xls` | 290.304 | `OLE2 (d0cf11e0a1b11ae1)` | `0eb74aa1245fd24bed37036d68c69d2045272e53f5fb1d96af336a376f1b1ba2` | `https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/SCR3040_Leiaute.xls` | 2026-11-19 |
| `3040/SCR3040_RegrasValidacaoBacen.xls` | 181.248 | `OLE2 (d0cf11e0a1b11ae1)` | `11321c35dd32225c0335ecdcf6fd77f20a615a2a029648d3c226d693ec1c0905` | `https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/SCR3040_RegrasValidacaoBacen.xls` | 2026-11-19 |
| `3040/Manual_Validador_SCR3040.pdf` | 875.991 | `%PDF (255044462d312e35)` | `c650e3e484a8554648eb0ab6110d8ac4e91194aa11881c1888c15b2ea2e5dfc7` | `https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/ManualValidador3040.pdf` | 2026-11-19 |
| `3040/Manual_Envio_3040.pdf` | 567.482 | `%PDF (255044462d312e35)` | `0e3f98539f5abe34bf9d18f481bbd138fae0ef80a6675da4a25c6b088b21b416` | `https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/ManualEnvio3040.pdf` | 2026-11-19 |
| `3040/SCR3040_DetalhamentoFIDC.pdf` | 261.715 | `%PDF (255044462d312e35)` | `8fee9d2d2fe67aa87fe975dbb69582843c42f16c46b5ca644a6a21b6be2a20ff` | `https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/SCR3040_DetalhamentoFIDC.pdf` | 2026-11-19 |
| `3040/SCR3040_Dispensa_Envio.pdf` | 85.513 | `%PDF (255044462d312e35)` | `a8eca7b991efc413cb47d4932fbf81c9122cd6659158deab195dd6a462e05904` | `https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/SCR3040_FIDC_DispensasEnvio.pdf` | 2026-11-19 |
| `3040/SCR_InstrucoesPreenchimento_Doc3040.pdf` | 1.618.203 | `%PDF (255044462d312e37)` | `3a2f9dff406b404b26617ab0dce79b5992cc67ac410ff60047236c73b11daf8b` | `https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/SCR_InstrucoesDePreenchimento_Doc3040.pdf` | 2026-11-19 |
| `3040/Manual_Particionamento_3040.pdf` | 186.048 | `%PDF (255044462d312e35)` | `b2ca38137c1133dfd1fa032dcccd0dc0f08328caecee77875b1526ab9cc5a1fc` | `https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040` *(existente no repo)* | 2026-11-19 |
| `3040/SCR3040_ConceitosDeConsultas.pdf` | 885.873 | `%PDF (255044462d312e35)` | `17c463d8b178c93266f5468e1b413a440febe06567b2a65b00a5a72ac0f3d62b` | `https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040` *(existente no repo)* | 2026-11-19 |
| `3040/SCR3040_Manual_Consulta_Web_Service.pdf` | 963.110 | `%PDF (255044462d312e35)` | `c45569941622aa959724a14a80e34e3bd98615926c3ff0560b1fabcdc6cd4155` | `https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040` *(existente no repo)* | 2026-11-19 |
| `3040/SCR3045.xsd` | 2.773 | `XML (3c3f786d6c207665)` | `a583940c01ec6cc96687cce162d13aac584e0f6ad916e724b08de735e150d01b` | `https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040` *(existente no repo)* | 2026-11-19 |
| `3040/3040_exemploCessao.xml` | 4.655 | `XML (3c3f786d6c207665)` | `c5da82093d20d5e86619b5f93c168df6146ae6d4acdb7187100ea58875192e79` | `https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040` *(existente no repo)* | 2026-11-19 |
| `3040/exemploDesempenhoOperacao.xml` | 4.832 | `XML (3c3f786d6c207665)` | `cb64db909a9ae9ef6ef976b8b7ab5bd402c54332ad96530393745dbe3b13de09` | `https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040` *(existente no repo)* | 2026-11-19 |
| `3040/noticias_Doc3040.xml` | 1.108 | `XML (efbbbf3c3f786d6c)` | `f5771f85c7c1dbce070535535182e8aacfa703c6611290f7ec6e1e59bf045c84` | `https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040` *(existente no repo)* | 2026-11-19 |
| `2030/Leiaute-DRSAC-valido-a-partir-de-dez-2026.zip` | 782.272 | `PK\x03\x04 (504b030414000800)` | `da82efc1f67aa3215530787b47a650bdcb4a3e74a01bc671476aba2f195e7100` | `https://www.bcb.gov.br/content/estabilidadefinanceira/drsac2030-docs/Leiaute-DRSAC-valido-a-partir-de-dez-2026.zip` | 2026-11-19 |
| `2030/Perguntas-Respostas-DRSAC.pdf` | 437.269 | `%PDF (255044462d312e37)` | `09c29c1dbaf5bc4ec764440f49fdb3028ca5409be526d946b142f86a4e1103a8` | `https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/drsac2030/Perguntas-Respostas-DRSAC.pdf` | 2026-11-19 |
| `_referencias/certs/star_bcb_gov_br_fullchain.zip` | 4.122 | `PK\x03\x04 (504b030414000000)` | `bc0ea3cc830de0acc8e36ac0a4e11ecda2384b708273d105574343563f24945c` | `https://www.bcb.gov.br/content/estabilidadefinanceira/certificados-digitais/star_bcb_gov_br_fullchain.zip` | 2026-11-19 |
| `_referencias/certs/br.gov.bcb.ac_BCNET_2026.cer.zip` | 1.767 | `PK\x03\x04 (504b030414000800)` | `36347613b5c9a76a26284f413650330f88514ea2812e20929c5731427e59331e` | `https://www.bcb.gov.br/content/estabilidadefinanceira/certificados-digitais/br.gov.bcb.ac_BCNET_2026.cer.zip` | 2026-11-19 |
| `_referencias/STA_Manual_WebServices.pdf` | 1.128.517 | `%PDF (255044462d312e37)` | `9179ec02b930c1bd44744d784680b21f6ebcb4c1fccbfe1a5bc0f64df7559d58` | Manual STA Web Services v1.5 (julho/2022, existente no repo) | 2026-11-19 |
| `tools/bacen/SCR3040_Validador.zip` *(gitignored)* | 11.605.657 | `PK\x03\x04 (504b03040a000000)` | `e3f487af45ffa279684a2a8d77a09241c51767f61a45467414051666f077b04f` | `https://www.bcb.gov.br/content/estabilidadefinanceira/Leiaute_de_documentos/scrdoc3040/SCR3040_Validador.zip` | 2026-11-19 |
| `tools/bacen/validador/classes/br/gov/bcb/scr2/validador/linhacomando/ValidadorIfLinhaComando.class` *(gitignored)* | 3.216 | `CAFEBABE (cafebabe00000034)` | `97fd8897a1362d2abfb83fa7709a64ec6471de32b86449e8a52e0f7a99eab7a6` | Extraído de `SCR3040_Validador.zip` | 2026-11-19 |

### Contagem verificada em `3040/SCR3040_Criticas.xls` (`xlrd 2.0.2`)

- **Total de regras**: `255`
- **`Habilitado=s`**: `226`
- **`Habilitado=n`**: `29`
- **Distribuição por aba**:
  - `Básicas`: 19 (`s=19`, `n=0`)
  - `Campos Obrigatórios`: 81 (`s=79`, `n=2`)
  - `Formato`: 5 (`s=4`, `n=1`)
  - `Semântica`: 111 (`s=97`, `n=14`)
  - `Individualizadas`: 15 (`s=9`, `n=6`)
  - `Agregadas`: 14 (`s=8`, `n=6`)
  - `Negócio`: 1 (`s=1`, `n=0`)
  - `Substituição Parcial`: 9 (`s=9`, `n=0`)

### Conteúdo interno verificado dos pacotes `.zip`

- **`2030/Leiaute-DRSAC-valido-a-partir-de-dez-2026.zip`**:
  - `1-Leiaute-DRSAC-valido-a-partir-de-dez-2026.xlsx`: 355.885 B | `PK\x03\x04` | `64bb57972ab5d1ac8ce5e72b3e7f0c6c977e42a3a2e3c8db1713155431a9fc47`
  - `2-Instrucoes-de-preenchimento-DRSAC-validas-a-partir-de-dez-2026.docx`: 61.060 B | `PK\x03\x04` | `ee99c222094b029fb54e51f89a06199ce4dd70865c57898501227ae4c08dd365`
  - `3-Leiaute_DRSAC_v2.03-valido-a-partir-de-dez-2026.xsd`: 30.702 B | XML (`<xs:schema`) | `6d29707e1ba1ec313eb9b88ccb4d4683c292255db3afff3ebac91f1ed9f0bca6`
  - `4-Críticas-Validação-DRSAC - valido a partir de dez-26.xlsx`: 21.188 B | `PK\x03\x04` | `34b5009fa0b90e49faee05f7a4c377f10ffa1b9f7954ed77bfe92b07ba591cdd`
  - `5-Perguntas-Respostas-DRSAC-validas-a-partir-de-dez-2026.pdf`: 436.672 B | `%PDF` | `6cbc04aa8203404143006f14e8e2516833aeb6065062a8542d1e4ab49fe315d4`
- **`_referencias/certs/star_bcb_gov_br_fullchain.zip`**:
  - `star_bcb_gov_br.crt`: 2.210 B | PEM | `2fb98b33f81ddd875446992446139b6ea1275fec42bb3cd68caef18fc0522812`
  - `TrustedRoot.crt`: 1.316 B | PEM | `d6d71565725851c8594379585a29ce9f8c1689b2c87d1064158a1aa8d0422bfc`
  - `DigiCertCA.crt`: 1.662 B | PEM | `a67ff02c79f73d123fe763988e44afb9bb1e3aba6b2a6172ee06b22b1e5ac271`
- **`_referencias/certs/br.gov.bcb.ac_BCNET_2026.cer.zip`**:
  - `br.gov.bcb.ac_BCNET_2026.cer`: 2.210 B | PEM | `4170f154dc2fa0e3567d5a5eef405a25fb1b16ac8afd325374b9ed06f56f6e16`
- **Dependências `.jar` em `tools/bacen/validador/lib/`** *(sem `.jar` monolítico do validador; classes BCB ficam soltas em `tools/bacen/validador/classes/`)*:
  - `caffeine-2.7.0.jar`: 846.706 B | `PK\x03\x04` | `170f2986d707c0df94228f57d5257b7f11f886fd42b53b61316a7498ac7e07ff`
  - `commons-collections-3.2.jar`: 571.259 B | `PK\x03\x04` | `093fea360752de55afcb80cf713403eb1a66cb7dc0d529955b6f4a96f975df5c`
  - `commons-compress-1.3.jar`: 224.548 B | `PK\x03\x04` | `56291a5427f6c0abc0b34fbf9bfafa0c21b60d503ece95c38741424d76e2aa04`
  - `commons-io-1.4.jar`: 109.043 B | `PK\x03\x04` | `a7f713593007813bf07d19bd1df9f81c86c0719e9a0bb2ef1b98b78313fc940d`
  - `commons-lang-2.3.jar`: 245.274 B | `PK\x03\x04` | `069361c71f22f8d7fbd4c3a568c015e2809327fc2e68699aeb63a64178cde56f`
  - `commons-lang3-3.8.1.jar`: 501.879 B | `PK\x03\x04` | `dac807f65b07698ff39b1b07bfef3d87ae3fd46d91bbf8a2bc02b2a831616f68`
  - `datedFileAppender-1.0.2.jar`: 2.394 B | `PK\x03\x04` | `f10a456d88e4025427407b4cbef47ba3b5abf7de757af7c5251dca3fe4b52d64`
  - `hibernate-core-3.6.8.Final.jar`: 3.119.425 B | `PK\x03\x04` | `25f488192a6cd344b7906fb2137e383d043d9047176a57a64c071195a6336b11`
  - `hibernate-tools-3.2.0.ga.jar`: 349.357 B | `PK\x03\x04` | `9d44c2fa0878723434cfa19f6a4aca45f64fb1a44b6c97d3d297cd6521cf0d9c`
  - `jackson-annotations-2.13.5.jar`: 75.718 B | `PK\x03\x04` | `80aea8ed7232db5040ced4b3f982f29da95bb3d802343dbf6fd82ccd98c21c4f`
  - `jackson-core-2.13.5.jar`: 375.186 B | `PK\x03\x04` | `48f36a025311d0464ad8dda4512a20c79e279a9550f63f3179d731d94482474b`
  - `jackson-databind-2.13.5.jar`: 1.537.543 B | `PK\x03\x04` | `5fedb24b2356491815d18267f65da9a21dd67413345ad7795f221afa25c78984`
  - `joda-time-2.1-tz2019c.jar`: 566.712 B | `PK\x03\x04` | `26d8f93438a8731995555ca247288badc60dbec232296dfa76b13a485cc0ff7b`
  - `json-schema-validator-1.4.0.jar`: 529.981 B | `PK\x03\x04` | `5d7b6ce4c7b2a3ed189511cbaa913808c7cd2b570d70b923426352785bffbdd0`
  - `reload4j-1.2.22.jar`: 332.398 B | `PK\x03\x04` | `222f90d3e69541218ef6e70547749d693bd4c1846817e5bd7949b3e28950f99f`
  - `slf4j-api-2.0.18.jar`: 69.982 B | `PK\x03\x04` | `44508fd1576500688c790b190acdd16fec4f8c79a3e0b900afd70503cf055f55`
  - `slf4j-reload4j-2.0.18.jar`: 10.726 B | `PK\x03\x04` | `37dc7cb891448c3c0db454d9e84c54b3f4e6a5f84b0db35917e7d0f322ae60df`
