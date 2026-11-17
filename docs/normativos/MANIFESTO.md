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

- [ ] 3040: leiaute + instruções + críticas vigentes (pós-IN 733 onda 3 nov/2026 + IN 735) — `https://www.bcb.gov.br/estabilidadefinanceira/scrdoc3040`
- [ ] 2030: leiaute + instruções + FAQ (data-base dez/2026) — `https://www.bcb.gov.br/estabilidadefinanceira/leiaute-documento-2030`
- [ ] Validador oficial SCR3040 (.jar) — página do BCValidador
- [ ] Literais verificados: RC 18/2025, RC 16/2025, INs 694/729/733/735, Comunicados 45.206/45.339
- [ ] `STA_Manual_WebServices.pdf` v1.5 (referência do port Rust do STA client)

## Registro de capturas

_(vazio — nenhuma captura verificada ainda)_
