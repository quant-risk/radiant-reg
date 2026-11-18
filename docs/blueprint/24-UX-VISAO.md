# 24 — UX VISÃO: a experiência do usuário do Radiant Norma v2

- **Data:** 2026-11-16
- **Status:** Blueprint de reconstrução — consolida `research/03-frontend-sdk-tooling-audit.md` (auditoria do frontend v3.36.4), `REDESIGN.md` (design system "Institutional Premium Noir"), `research/02-tese-vs-realidade.md` (placebos e doc-truth) e `research/05-concorrentes-mercado.md` (padrões de excelência 2026)
- **Pré-condições:** frontend permanece Next.js (11-TESE-V2 §6a); motor Rust conforme 20-ARQUITETURA-RUST.md (fases F0–F6); harness conforme 22-HARNESS-V4.md (flywheel M1–M10, Health Scorecard); copiloto conforme 21-ENGINE-IA.md
- **Tese de UX em uma frase:** em compliance, a interface não vende features — vende **confiança verificável**; cada tela deve provar de onde veio cada número, cada veredito e cada status, porque o comprador é um compliance officer que será auditado por aquilo que o sistema mostrou.
- **Escopo pós-fusão (16/11/2026):** este documento descreve a UX da **FASE 2** (Console Next.js). A **FASE 1 (wedge "Radiant Reg") é CLI-first** — a UX do wedge é o `reg-cli` determinístico (exit codes, `--offline`, `--json`) mais o **Dossiê A4 editorial** (22-HARNESS e 25 §0 S5). O Wizard 2.0 e as telas abaixo valem pós-N0 (primeiro contrato).

---

## 0. Sumário

| § | Conteúdo |
|---|---|
| 1 | Cinco princípios UX v2 (Trust-first · HITL 1-clique · Calendário como homepage · Copiloto citado · Status machine-generated) |
| 2 | O que KEEP do design system "Institutional Premium Noir" |
| 3 | O que FIX imediatamente (P0 — bugs e placebos antes de qualquer tela nova) |
| 4 | Wizard 2.0 (jornada hero, L1–L4 streaming, wireframe) |
| 5 | Novas telas: /normativa · /simulacao · /benchmark · /console/regras 2.0 · /console/health |
| 6 | UX da API/SDK para o persona técnico |
| 7 | UX do copiloto |
| 8 | Roadmap UX por fase do motor (F0–F6) |
| 9 | Instrumentação e métricas de UX · 10. Anti-objetivos |

---

## 1. Cinco princípios UX v2

A v3.36 provou que a **estética funciona** (REDESIGN.md é fiel ao código — research/03 §1.6). A v2 muda o **contrato de conteúdo** da UI: a auditoria encontrou placebos na tela (`stableCoverage`, `nextDeadline` — números derivados de hash), um passo morto no coração do wizard e botões sem handler. Nenhum comprador regulatório pode ser auditar uma tela e encontrar um número sem lastro. Os cinco princípios abaixo são derivados da tese v2 ("confiança é o produto"), das feridas da auditoria e do padrão de excelência global 2026 (provenance obrigatória, human-in-the-loop — Regnology "Agentic Gap", Norm AI, Fenergo rules-as-data).

### P1 — Trust-first: provenance clicável em toda verdict, zero número mágico

1. **Toda verdict carrega provenance clicável.** O motor Rust já produz `FieldMapping{Source, RuleRef, Confidence}` + citação normativa (11-TESE-V2 §6a). A UI renderiza cada erro/aviso/campo gerado como chip clicável `[V3040-0412 § Circular 3.995, art. 12, II ↗]` que abre o detalhe da regra: § fonte com link, vigência, versão (`rule_version.changed_by` — "veio do radar_event 812"), cobertura de eval (n casos, classes boundary/negative), histórico de vereditos e disputas com o BCValidador. Provenance que não abre nada é apêndice, não provenance.
2. **Zero número mágico.** Toda métrica exibida precisa ter um produtor verificável: um `eval_run` (fingerprint `engine_version + ruleset_version + judge_version + corpus_hash`), uma query SQL auditable ou um evento do Radar. Tooltip "como é calculado" em todo KPI, linkando o run/produtor. **Regra dura: se o número não tem query ou `eval_run` que o produza, não entra na tela.** É a correção definitiva dos placebos da v3.36: "cobertura de schema" por hash (`console/page.tsx:100-106`) morre; entra o **Rule Coverage Index medido pelo harness** (matriz regra×edge, 22-HARNESS §4.2). Deadline inventado por hash (`envios/page.tsx:81-88`) morre; entra o calendário normativo alimentado pelo Radar (P3).
3. **Confiabilidade se demonstra, não se afirma.** Nada de "99% de precisão" em marketing copy dentro da UI; o que existe é o scorecard append-only assinado e a curva pública de FN rate (22-HARNESS §7). O trust-first não é um selo — é a ausência física de números sem origem.

### P2 — Human-in-the-loop de 1 clique: approve/correct alimenta o flywheel M2

1. **O clique de correção é a interação mais valiosa do produto** — e custa exatamente o clique que o operador já daria de qualquer jeito (modelo Duco/Grammarly). Toda correção de valor antes do reenvio faz **snapshot automático before/after** e vira `eval_case(provenance='user_correction')` (M2, 22-HARNESS); `reason` é opcional, nunca bloqueante. Zero formulário extra, zero fricção.
2. **O valor do clique volta para a tela.** Depois de aprovar/corrigir, a UI confirma com feedback concreto: "Sua correção entrou no corpus como caso de teste (nº 4.218). A regra V3040-0412 ganhou cobertura boundary." Isso transforma o operador de digitador em treinador do motor — e é o argumento de retenção ("cada submissão deixa o Radiant — e todo o mercado — menos propenso a errar a próxima").
3. **Copiloto, não autopilot — elevado a interação de UI.** 89% das IFs não embarcaram IA em produção (Regnology Agentic Gap); o workflow formaliza isso: toda ação de IA termina em um diff-approve (P4/§7); envio ao STA exige clique humano explícito; triagem do harness é batch 1-clique semanal (único toque humano obrigatório do flywheel, 30–45 min).
4. **Bulk-first.** Toda fila de decisão (críticas, candidatos de regra, correções sugeridas) suporta approve/bulk com filtro por confiança — a v3.36 já acerta no toggle de regras persistido; v2 generaliza o padrão.

### P3 — Calendário normativo como homepage do operador

1. **O dia do operador começa por "o que muda e quando", não por métricas.** Pós-login, a home é `/normativa` (§5.1): data-base corrente, próximas vigências (2030 DRSAC dez/2026 — IN 694; 3040 NR3 — onda 3 da IN 733 nov/2026; DLO 2061 75% — IN 750; GRSAC 1º/1/2027; 3060 mensal 1º–10º dia útil), cada evento linkando o diff de leiaute, as regras do catálogo afetadas e o checklist de reteste. O dashboard executivo/KPI vira aba secundária — inclusive para o executivo, que primeiro quer saber se a IF bate a próxima janela.
2. **Todo deadline na UI vem do Radar ou da fonte oficial** — nunca de constante no código. O padrão Regnology Nexus (calendário + briefing + mudança→regra→reteste) é o referencial; cadoc.ai provou o apetite pelo lado informativo — falta no mercado o link mudança→regra→reteste, que é exatamente esta tela.
3. **O diff entre ondas é produto, não documento** (11-TESE-V2 §6b): cada mudança de leiaute aparece com diff visual de campos (novo/alterado/removido), impacto por CADOC e um botão "Simular minha carteira sob o leiaute novo" (§5.2). Time-to-update medido aparece como SLA público no topo (T2U ≤10 dias úteis — "no dia da norma").

### P4 — Copiloto com evidência citada

1. **Sem citação, não sai.** Toda resposta do copiloto carrega chips de citação (regra do catálogo + § do normativo + campo do Canonical). Resposta sem fonte verificável é bug, não estilo (guardrail de 21-ENGINE-IA §1.3).
2. **O LLM não calcula — o engine calcula, o LLM interpreta.** Qualquer número na conversa vem do motor Rust (veredito, diff, soma); o LLM explica, contextualiza e propõe. Isso neutraliza hallucination por construção: não há caminho em que o modelo invente um saldo.
3. **A interação central é o diff-approve, não o chat.** O copiloto não conversa sobre o problema abstratamente: propõe a mudança como diff no Canonical, o humano aprova/corrige, o motor regenera e re-valida (§7). Chat é o veículo; diff-approve é o contrato.

### P5 — Status machine-generated: nada escrito à mão

1. **A UI nunca declara estado — ela exibe o que o gate mediu.** STATUS.md machine-generated desde F0 (20-ARQUITETURA F0; ADR-0013 "status do produto é machine-generated"): pipeline `script → JSON → UI` alimenta badges de versão, cobertura, saúde do harness e readiness. O badge do rodapé mostra o fingerprint completo (`engine 3.36.2 · ruleset 2026-11 · judge v7 · corpus a9f3`) — clicável, com link para o `eval_run`.
2. **Elimina a classe de desonestidade C1–C4 da auditoria** (research/02: quarters "已完成", sprints de 2027 fechadas em 2026, 4 números de cobertura circulando no mesmo README): texto de status com origem humana é bug de UI, rastreável pelo placeholder lint generalizado para o frontend.
3. **Honestidade de estado em toda superfície:** conectores stub exibem "planejado — F4" (não um botão morto), STA homolog vs produção é sempre explicitado no botão de envio, gap de regras aparece como `GAP{risk}` do gate de completeza — nunca como silêncio. A UI herda o padrão "stubs honestos" que foi a única coisa que impediu o v1 de virar 100% vapor (research/02 §3.5).

---

## 2. O que KEEP do design system "Institutional Premium Noir"

O REDESIGN.md foi verificado ponto a ponto contra o código (research/03 §1.6): dos 9 itens auditáveis, 8 são REAL. O sistema sobrevive inteiro à reconstrução — é o ativo de frontend mais barato de portar e o mais visível na demo. Nada de redesign por gosto: **v2 herda o sistema e adiciona 7 primitivos de domínio.**

| Ativo | O que é (evidência v3.36) | Decisão v2 |
|---|---|---|
| Tokens light+dark | CSS variables RGB `globals.css`; paleta Midnight Blue → Platinum (`#1E2A5E → #94A3B8`), surfaces warm off-white / noir `#0A0A0B` | **KEEP** — base da identidade; nova paleta só para diff (`added/removed/changed`) e data-viz de severidade, sempre como tokens |
| Escala tipográfica `text-2xs…display-lg` | 13 tokens em `tailwind.config.ts` replicando a tabela do REDESIGN | **KEEP** — densidade calibrada 32/24/16px por contexto |
| Fraunces / Inter / JetBrains Mono | Serif display / sans UI / mono para CADOCs, hashes, valores (§3 do REDESIGN) | **KEEP** o contrato (P0 de carregamento em §3.4); hashes, protocolos STA e sha256 **sempre** em JetBrains Mono — é o token visual de "dado verificável" |
| Motion 180–240ms | `duration-180/240` + `cubic-bezier(0.16,1,0.3,1)` (`ease-out-expo`), 12 keyframes, sem bounce | **KEEP** — v2 adiciona: barras de streaming L1–L4 em `duration-320`, badges SSE com `pulse-soft`, `prefers-reduced-motion` continua cortando tudo |
| AppShell | Sidebar 256px (grupos "Operação"/"Inteligência") + topbar glass sticky; active state rail 2px gradient | **KEEP** — sidebar ganha grupo "Normativo" (/normativa sobe para o topo) |
| Command palette ⌘K | `command-palette.tsx` 395 LOC: busca regras/alertas/schemas/páginas reais, navegação por teclado, aria, **focus trap** com restauração de foco | **KEEP** e expandir: passa a indexar regras por código/§, CADOCs por data-base, cenários de simulação salvos e comandos ("rodar simulação", "abrir fila de candidatos") |
| Primitivas (10) + `Button asChild` | Card/Badge/EmptyState/Skeleton/Kbd/Tooltip/Divider/StatCard/AlertCard/SectionHeader + polimorfismo semântico | **KEEP** — StatCard é o molde dos novos KPIs com provenance |
| Severity com 3 sinais | Glyph (AlertTriangle/Circle/Info) + badge dot+texto + rail 3px gradiente (rose/amber/sky) — WCAG 1.4.1 | **KEEP** — severidade do catálogo BACEN (A/B/I) mapeia nos mesmos 3 sinais; nunca cor sozinha |
| Empty states + loading | Editorial (símbolo Fraunces `∅` + título + CTA); skeleton shimmer, "nunca spinner solto" | **KEEP** — streaming do wizard usa skeleton por camada, não spinner |
| A11y + segurança | 58 `aria-*`/`role`, `focus-visible` ring global, landmarks, `aria-modal`+focus trap, CSP com nonce por request, HSTS, cookies httpOnly | **KEEP** — e sustituir a auditoria manual por axe-core no CI (§3.8) |
| SSE hook | `use-event-stream.ts` 227 LOC: reconexão exponencial 1s→30s, heartbeat 60s, filtros por `kind` | **KEEP** — vira a base da validação streaming e do feed Radar |

**Novos primitivos v2 (7):** `<ProvenanceChip>` (regra+§ clicável), `<RuleCitation>` (modal detalhe), `<DiffView>` (inline/side-by-side para XML e leiaute), `<VerdictRow>` (verdict + provenance + confiança), `<FixSuggestionCard>` (before/after + Approve/Corrigir/Dispensar), `<FingerprintBadge>` (engine·ruleset·corpus hash), `<WizardStepper>`. Todos construídos sobre os tokens existentes — zero cor hardcoded (REDESIGN §9).

---

## 3. O que FIX imediatamente (P0 — antes de qualquer tela nova)

A ordem é: **credibilidade primeiro, telas depois.** Cada item abaixo é um buraco que um comprador regulatório encontra em 5 minutos de demo.

| # | Bug | Evidência | Fix | Por quê P0 |
|---|---|---|---|---|
| 3.1 | **Passo "Validar" do wizard 404** — `fetch('/v1/validate')` same-origin sem rewrite; o Next responde 404 no browser. O passo central da jornada hero ("gera, valida e envia") nunca funcionou end-to-end | `wizard-client.tsx:554`; sem rewrites em `next.config.js`; os outros passos usam handlers `/api/*` corretos | Route handler BFF `/api/validate` seguindo o padrão dos demais (ou rewrite explícito). No v2 o passo é substituído pelo streaming L1–L4 (§4), mas o fix é pré-requisito de qualquer demo | É o coração do produto morto no browser |
| 3.2 | **Métricas placebo por hash** — `stableCoverage` ("cobertura de schema" derivada de hash) e `nextDeadline` (deadline inventado) | `console/page.tsx:100-106`; `envios/page.tsx:81-88` | Remover. Cobertura → **Rule Coverage Index** do harness (só existe quando o harness medir); deadline → **calendário do Radar** (só existe quando o Radar detectar). Princípio P1.2 vira teste no CI: nenhum KPI sem produtor declarado | Um comprador que auditar a tela encontra número falso — veneno fatal num produto de confiança |
| 3.3 | **Botão "Configurar" morto** em /console/ingest — sem `onClick`/`href` | `console/ingest/page.tsx` (lista hardcoded; o próprio arquivo documenta os 4 conectores stub) | Ou handler real, ou estado desabilitado + tooltip "Disponível na F4" + empty state editorial honesto. Mesma regra para `totalGenerated = 0` com TODO em /console/generate (endpoint de stats ou card removido) | Botão que não faz nada = promessa falsa na interface |
| 3.4 | **Inter/Fraunces/JetBrains Mono não carregadas** — `layout.tsx` usa fallbacks (Georgia/SF Mono) porque `next/font/google` falha no ambiente (comentário Sprint 34); o look editorial degrada | `frontend/src/app/layout.tsx` | Self-host WOFF2 no repo (`next/font/local` ou @fontsource) — resolvido com build offline, sem rede no ambiente. Sub-item: auditar `font-display` e preload | A tipografia É a identidade "Institutional Premium Noir"; Georgia não é Fraunces |
| 3.5 | **6 dependências mortas** — react-query, axios, zod, react-hook-form, zustand, recharts declaradas com ~zero uso (QueryClientProvider montado sem nenhum `useQuery`; `lib/api.ts` axios nunca importado; heatmap é CSS) | `package.json` + greps em research/03 §1.5 | Remover todas. Decisão explícita de stack: server components + SSE hook para fetching; se o Wizard 2.0 adotar react-hook-form+zod **com uso e teste reais** (validação de formulário de mapeamento), entram de volta com propósito — senão ficam fora. Recharts entra só se a /simulacao demandar gráficos reais (decisão na F5) | Build limpo = superfície de ataque e de confusão menor; dependência placebo é o sintoma do problema de cultura que gerou os placebos de UI |
| 3.6 | **Estado do wizard frágil** — XML trafega por `(window as any).__wizardXml`; `parsedDoc` em `useState`; refresh no passo 3+ perde tudo (o comentário do código promete o contrário) | `wizard-client.tsx` | Wizard sessions server-side (a migration 025 do Go já tinha o conceito): estado do wizard persistido no backend, resumível por URL. Hack global de window banido | A jornada hero não pode quebrar num refresh durante uma demo |
| 3.7 | **Fontes de verdade duplicadas** — /regras com fallback que parseia `docs/rules-3040-catalog.md` do disco com regex; /console/generate com `CADOCS[]` hardcoded no TSX | `regras-client.tsx`; `generate/page.tsx` | API única (o catálogo normalizado do v2, 20-ARQUITETURA F0). Regex-fallback e constantes hardcoded removidos; se a API falha, empty state editorial com retry — nunca dado embutido que envelhece | Dado hardcoded é placebo futuro |
| 3.8 | **Zero testes no frontend** — nenhum `*.test.*`, nenhum e2e, nenhum axe | research/03 §1.1 | Mínimo obrigatório no CI: vitest (primitivos + lógica do wizard) + Playwright (jornada do wizard end-to-end contra sandbox) + axe-core nos 10 primitivos + teste de regressão visual dos componentes do design system. Gate: PR de UI sem teste de jornada não merge | O design system é o ativo a proteger; hoje uma regressão quebra a demo sem ninguém saber |
| 3.9 | **Login de produção 404** — IdP externo esperado, não implementado; só dev-token | `api/login` handler | Não é fix de código, é decisão: fluxo OIDC (SSO) entra na F5 com o launch comercial; até lá, a demo roda em ambiente de homologação com login dev explícito e banner "ambiente de demonstração" | Impede demo com cliente real; banner evita o anti-padrão "dev disfarçado de prod" |

Critério de saída do bloco P0: wizard end-to-end green no Playwright (upload→mapear→preview→validar→gerar), zero KPI sem produtor, zero botão morto, fontes carregadas, `pnpm build` sem dependência morta, suite de UI no CI bloqueando.

---

## 4. Wizard 2.0 — a jornada hero

O wizard é a materialização da tese original: "sobe os dados, recebe o CADOC gerado e validado, aperta um botão" (MASTER_PLAN:16). O mapeamento colunas→canônico com perfil por CADOC e auto-match é **a melhor UX do produto v1** (research/03 §10) — mantido e elevado. A v2 corrige o que a v1 não entrega: validação real (era 404), visibilidade das camadas, correção assistida e envio com prova.

**Os 6 passos:**

1. **Ingest** — upload CSV/XLSX (parse server-side → `CanonicalDocument` via `/v1/generate/file/parse`); conectores File/API entram como fontes alternativas na F4. Feedback imediato: preview de linhas detectadas, colunas reconhecidas, problemas de encoding (B04) já no parse.
2. **Mapear** — colunas → campos canônicos com auto-match e **confiança por campo**; perfil de mapeamento salvo por CADOC (herança do `radiant:mapping_profiles`, agora server-side por tenant). Regra de confiança: match ≥0.9 auto-aplicado e visível; <0.9 exige confirmação explícita — **nada é mapeado em silêncio** (o "default silencioso" é a classe de bug que o harness caça).
3. **Preview canônico** — tabela tipada do CanonicalDocument com `<ProvenanceChip>` por campo (Source/RuleRef/Confidence); transformações anotadas célula a célula; campos ignorados aparecem, nunca desaparecem.
4. **Validar** — L1→L4 **streaming por SSE**: camadas completam visualmente (L1 estrutural/XSD → L2 semântica/regras → L3 cross-doc → L4 histórico), críticas aparecem ao vivo agrupadas por severidade (A bloqueia / B atenção / I informativo), cada uma com citação. Progresso mostrado é o real do worker (nada animado "por fora").
5. **Resolver** — para cada crítica, `FixSuggestionCard` com valor atual → sugerido, origem do cálculo e confiança; **Aprovar** (aplica + snapshot M2) / **Corrigir à mão** (edição no Canonical, nunca no XML) / **Dispensar** (com justificativa opcional — vira `rule_candidate(origin='dispute')` se indicar falso positivo). A cada lote aprovado: regeneração + **re-validação delta** (só as regras tocadas re-executam — latência percebida cai de segundos para centésimos). O diff visual do XML (inline, campo a campo, com provenance) mostra exatamente o que mudou.
6. **Emitir & enviar** — XML final com tamanho + sha256 (JetBrains Mono, clicável), preview do payload STA, seletor de ambiente com estado explícito (homolog `sta-h` / produção), `Idempotency-Key` sugerida e auto-gerada, submit com protocolo 18 dígitos e webhook `sta.receipt`. Produção exige confirmação secundária digitada (o CADOC + data-base) — o sistema nunca envia em produção por clique acidental.

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  Radiant Norma · Gerar CADOC 3040 · data-base 31/10/2026          [⌘K]   f3a9c2e1    │
├──────────────────────────────────────────────────────────────────────────────────────┤
│  ① Ingest ─── ② Mapear ─── ③ Preview ─── ④ Validar ─── ⑤ Resolver ─── ⑥ Emitir      │
│   (done)      (done)       (done)      ● ACTIVE      (2 pend.)      (bloq. até ⑤)   │
├───────────────────────────────────┬──────────────────────────────────────────────────┤
│  CAMADAS DE VALIDAÇÃO (SSE)       │  CRÍTICAS  12   A:2  B:7  I:3        [filtro ▾]  │
│                                   │                                                  │
│  L1 Estrutural/XSD   ██████ 100%  │  ▼ SEVERIDADE A — bloqueia envio (2)             │
│     leiaute NR3 · 0 erros         │  ┌────────────────────────────────────────────┐  │
│  L2 Semântica        ███░░  62%   │  │ V3040-0412  Saldo vencido ≠ soma parcelas  │  │
│     361 regras · 9 violadas       │  │ Quadro 3 · campo B3.Vencido                │  │
│  L3 Cross-doc        ░░░░░ agd.   │  │ diff R$ 1.204,30 na conta 4721             │  │
│     12 regras XD01–XD12           │  │ [V3040-0412 § Circ. 3.995 art.12 II ↗]     │  │
│  L4 Histórico        ░░░░░ agd.   │  │ ── SUGESTÃO (confiança 0.94) ────────────  │  │
│     6 meses de base               │  │  B3.Vencido: 84.312,00 → 85.516,30         │  │
│                                   │  │  origem: soma(17 parcelas, conta 4721)     │  │
│  ── 9 críticas · 3,2s ── ● live   │  │ [✓ Aprovar] [✎ Corrigir à mão] [Dispensar] │  │
│  (reconexão auto 1s→30s)          │  └────────────────────────────────────────────┘  │
├───────────────────────────────────┴──────────────────────────────────────────────────┤
│  ▸ DIFF DO XML (última regeneração) — inline, provenance por célula                  │
│  − <SldVencido>84312.00</SldVencido>              (valor anterior)                  │
│  + <SldVencido>85516.30</SldVencido>              [V3040-0412 · sugestão aprovada]  │
├──────────────────────────────────────────────────────────────────────────────────────┤
│  engine f3a9 · ruleset 2026-11 · corpus a9f3      [◀ Voltar] [Re-validar delta ▶]   │
└──────────────────────────────────────────────────────────────────────────────────────┘

  PASSO ⑥ — EMITIR & ENVIAR (compacto)
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  XML 3040 final · 12.431 bytes · sha256 f3a9…e1   [baixar] [ver XML]                 │
│  PAYLOAD STA (preview)                                                               │
│  { "cadoc": "3040", "data_base": "2026-10-31", "leiaute": "NR3 (IN 733/2026)",       │
│    "ambiente": "homolog · sta-h", "idempotency_key": "env-202611-3040-001" }         │
│  validação: L1–L4 100% green · paridade BCValidador ✓ · 0 críticas A                 │
│  [Enviar ao STA (homolog)]        produção: exige confirmação digitada               │
│  enviado · protocolo 202611304012345678 · webhook sta.receipt agendado               │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

**Comportamentos que fazem o wizard ser confiável:** passos anteriores permanecem navegáveis (voltar nunca perde estado — P0 3.6); o stepper bloqueia o passo 6 enquanto existir crítica A; dispensar uma A exige justificativa e registra no audit log hash-chained; todo envio (homolog inclusive) gera entrada auditável com fingerprint.

---

## 5. Novas telas

### 5.1 `/normativa` — calendário normativo + feed Radar (a homepage, P3)

Fontes: eventos do Radar normativo (crawler real no worker v1; v2 com diff semântico), leiautes do schema registry, catálogo de regras. Estados: evento novo destacado; regras afetadas linkadas; checklist de reteste com progresso (suite da nova vigência green = pronto). Diff de leiaute visual: campos novos/alterados/removidos com `<DiffView>` — o produto que ninguém no BR entrega ("o diff entre ondas é produto", 11-TESE-V2 §6b).

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  /normativa — Calendário normativo                    SLA T2U: 8 dias úteis ✓ (medido)│
├───────────────────────────────┬──────────────────────────────────────────────────────┤
│  CALENDÁRIO (data-base)       │  RADAR — feed de mudanças (SSE live)                 │
│                               │                                                      │
│  NOV/2026 ● data-base corrente│  ▸ 12/11 · IN 733 onda 3 — leiaute 3040 → NR3        │
│   3040 NR3 (IN 733) vigente   │    diff: 3 campos novos · 1 domínio alterado         │
│   ✓ nosso leiaute atualizado  │    [ver diff visual] · 14 regras afetadas            │
│                               │    reteste: 2 green · 12 pendentes [checklist →]     │
│  DEZ/2026 ◆ janela crítica    │                                                      │
│   2030 DRSAC "Carteira Total" │  ▸ 28/10 · novos códigos de avaliação 2030           │
│   (IN 694) · 35 regras novas  │    taxonomia ESG · [diff § a §]                      │
│   checklist de reteste ▸      │                                                      │
│                               │  ▸ 19/10 · DLO 2061 fase 75% (IN 750)                │
│  JAN/2027                     │    518 regras do catálogo no escopo                  │
│   GRSAC (Res. 586 + IN 772)   │    [plano de atualização · T2U 8/10 dias]            │
│   DRL 2160 (IN 755)           │                                                      │
│                               │  baseline atualizado · crawler 06:00 · 1 URL 404 ⚠   │
│  MENSAL · 3060 juros cartão   │                                                      │
│   remessa 1º–10º dia útil     │                                                      │
├───────────────────────────────┴──────────────────────────────────────────────────────┤
│  DIFF DE LEIAUTE (visual) — 3040 NR2 → NR3                                           │
│  + <PddVincendioLongoPrazo> N19,2 · obrigatório                                      │
│  ~ <TpGarantia> domínio: +{FIDC, COE} −{NENHUM}                                      │
│  − <SldTotalNacional> removido (migrado p/ Quadro 7)                                 │
│  [Simular minha carteira sob NR3 →]  [Regras afetadas (14) →]                        │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 `/simulacao` — what-if (liquidar X% da carteira → indicadores)

Replay determinístico: transformações aplicadas ao Canonical da última data-base → o **motor real** regenera e re-valida os CADOCs do cenário → indicadores before/after com delta pills. Guardrails: watermark permanente "SIMULAÇÃO — não enviável"; cenários salvos versionados com provenance; nada de simulação toca produção ou audit chain. Segunda aba (F6): "leiaute futuro" — validar a carteira sob a próxima vigência (2030 dez/2026) antes do deadline.

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  /simulacao — What-if                                 ⚠ SIMULAÇÃO — não enviável      │
├─────────────────────────────┬────────────────────────────────────────────────────────┤
│  CENÁRIO                    │  INDICADORES (base 31/10/2026 → simulado)              │
│                             │                                                        │
│  Ação: liquidar             │  Saldo carteira    R$ 412,0 mi → R$ 329,6 mi  (−20,0%) │
│   X% = [ 20 ]% ▬▬▬●▬▬▬      │  PDD total         R$ 8,1 mi  → R$ 6,6 mi     (−18,4%) │
│  Segmento: [todos ▾]        │  RWA estimado      R$ 286 mi   → R$ 231 mi    (−19,2%) │
│  Rating: [BB..C ▾]          │  Top-10 concentração  31% → 34%        (▲ atenção)     │
│  Vencimento: [≤ 90d ▾]      │  Mix rating: BB 42%→39% · C 7%→9%                      │
│  [▶ Rodar simulação]        │                                                        │
│                             │  ── Validade regulatória do cenário ──                 │
│  Transformações aplicadas   │  ✓ 3040 regerado · L1–L4 green (0 críticas A)          │
│  1. liquidacao(proporcional)│  ✓ 3050 regerado · 1 crítica B (limite TXB)            │
│  2. pdd_pro.rateio()        │  ⚠ 4010: lançamentos contábeis não simulados (F6)      │
│  3. rwa_peso.rating()       │                                                        │
│                             │  [Comparar XML base ↔ simulado]  [Salvar cenário]      │
│  [Gerar CADOC simulado]     │                                                        │
├─────────────────────────────┴────────────────────────────────────────────────────────┤
│  cenários salvos: "estresse inadimplência 30%" · "venda carteira BBB"     [novo]     │
│  provenance: Canonical 31/10 · engine f3a9 · ruleset 2026-11 · execução 2,4s         │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### 5.3 `/benchmark` — tier opt-in anonimizado

Conforme 22-HARNESS §7.3 e 23-PRODUTO (Benchmark Premium R$1.490/mês standalone; desconto como "crédito de dados", nunca por dinheiro): comparação **anonimizada** com pares (mesmo porte/tipo de IF) sobre precision/recall por documento, FN rate, Time-to-Green e cadência de correções. Garantia de privacidade por design: agregados só com **k ≥ 5 contribuintes** por célula (k-anonymity), nunca dado por-IF, opt-out a qualquer momento. Cada correção do cliente melhora o próprio scorecard e o benchmark — o incentivo do opt-in é embutido na mecânica M2.

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  /benchmark — Comparação de segmento (opt-in · anonimizado)                          │
├──────────────────────────────────────────────────────────────────────────────────────┤
│  Seu par: SCD · porte R$ 100M–1B · 7 contribuintes (k≥5)      [política de dados ↗]  │
│                                                                                      │
│  FN rate 3040 (30d)   VOCÊ 0,8% · p50 pares 1,9% · melhor 0,3%   [▇▇▃▁▂ distribuição]│
│  Precision média      VOCÊ 96,2% · p50 pares 93,4%                                   │
│  Time-to-Green p50    VOCÊ 3d · p50 6d                                               │
│  Correções/mês        VOCÊ 12 · p50 9    ← cada correção melhora seu score           │
│                                                                                      │
│  Por documento: 3040 ▾      (3050 · 3044 · 2030)                                     │
│                                                                                      │
│  ── Ainda não contribui? ──                                                          │
│  [Ativar opt-in]  "dados anonimizados · agregados só com k≥5 · nunca por-IF"         │
│  opt-out a qualquer momento · desconto "crédito de dados" conforme pricing           │
│                                                                                      │
│  fingerprint do agregado: bench-2026-11-04 · gerado pelo harness · [raw JSON ↗]      │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### 5.4 `/console/regras` 2.0 — catálogo 1.099 navegável + fila de rule_candidates

O catálogo deixa de ser lista flat com fallback regex (P0 3.7) e vira o **sistema de registro das regras**: filtros por CADOC/severidade/vigência/status de implementação (`IMPLEMENTED | DATA_ENTRY | GAP{risk}` — o tri-estado do gate de completeza, 22-HARNESS §5.1); detalhe com § fonte clicável, versão e `changed_by`, cobertura de eval (células da matriz regra×edge), histórico de vereditos e disputas com o BCValidador. A fila **rule_candidates** (M7) é o diferencial: clusters de correções/discrepâncias sem regra mapeada, com hipótese gerada e 10 exemplos; promover a draft é 1 clique e alimenta o catálogo — a "receita de catálogo" que nenhum concorrente tem.

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  /console/regras — Catálogo 1.099 regras   [⌘K]   filtros: 3040▾ · sev A▾ · vigente▾ │
├───────────────────────────────────┬──────────────────────────────────────────────────┤
│  LISTA (1.099)                    │  DETALHE — V3040-0412                            │
│  □ código · resumo · sev · cobertura                                                │
│  □ V3040-0412 saldo≠parcelas  A ▂ │  status: IMPLEMENTED · vigência 2026-05 → hoje   │
│  □ V3040-0517 PDD ≥ 0         A ▇ │  fonte: § Circular 3.995, art. 12, II  [abrir ↗] │
│  □ V3040-0328 dom. TpGarantia B ▃ │  versão v3 · changed_by: radar_event:812         │
│  □ V3040-0090 UF válida       I   │  cobertura eval: 14 casos (4 boundary·3 negat…)  │
│  …                                │  RCI da célula: 7/9 · histórico de vereditos ▸   │
│  impl.: [todos|IMPL|DATA_ENTRY|   │  disputas: 1 (BC diverge) → rule_candidate       │
│          GAP] · gap: 62 regras    │                                                  │
├───────────────────────────────────┴──────────────────────────────────────────────────┤
│  FILA DE RULE_CANDIDATES (M7) — 4 candidatos                         [triagem semanal]│
│  ▸ cluster #12 "conta 4721 arredondamento" · 23 correções · 3 tenants                │
│    hipótese: "soma de parcelas com arredondamento de centavo diverge"                │
│    [ver 10 exemplos]  [Promover a draft de regra]  [Dispensar cluster]               │
│  ▸ cluster #9 "3044 cessão mai/26" · 11 correções · 2 tenants                        │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### 5.5 `/console/health` — Harness Health Scorecard público

A página mais trust-first do produto: **pública** (sem login, cache curto, sem dado de tenant), emitida pelo harness no nightly, append-only, com as 8 métricas da 22-HARNESS §4.2 e os alarmes visíveis quando disparam. É a resposta UI à lição P5 ("o harness mede a si mesmo") e a prova pública do Accuracy SLA vendido na F5/F6. Nada aqui é digitado à mão — a própria página declara isso.

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  /console/health — Harness Health Scorecard      PÚBLICO · append-only · [raw JSON ↗]│
├──────────────────────────────────────────────────────────────────────────────────────┤
│  Mutation Detection ≥75%   81% ▇▇▇▇▇▇▇▇▁ ✓     Judge TPR/TNR ≥95%   96/95      ✓    │
│  Canary pass 100%          100%          ✓     Held-out leak 0  0            ✓      │
│  Rule Coverage ≥80%        74% ▂ ALARME       Shadow-rule <5%  3,1%         ✓       │
│  Audit agreement ≥97%      98%           ✓     Time-to-Green ≤7d 4,2d        ✓       │
│                                                                                      │
│  fingerprint: engine f3a9 · ruleset 2026-11 · judge v7 · corpus a9f3 · run 03:12     │
│  tendência 6 meses: FN rate 2,1% → 0,8% (curva pública · scorecards append-only)     │
│  T2U norma→leiaute: 8 dias úteis (medido na onda IN 733)                             │
│  ── este número foi emitido pelo harness às 03:12; nada aqui é digitado à mão ──     │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. UX da API/SDK — o persona técnico

Roberto (CIO) e o dev que integra têm um critério de confiança próprio: **DX é trust também**. A auditoria v1 mostrou o custo de não levar isso a sério (duas gerações de SDK com nomes colidindo, spec OpenAPI errada no crossdoc que quebrou a SDK nova — `documents` vs `cadocs`). A v2 trata API/SDK como superfície de UX de 1ª classe.

1. **Contrato único e gerado.** OpenAPI 3.1 emitida pelo próprio servidor (utoipa, 20-ARQUITETURA F1) → tipos TS/Python/Go gerados no CI. Fim das interfaces manuais (`api-types.d.ts` nunca gerado) e do drift spec↔backend↔SDK. **Uma geração de SDK por linguagem** (elimina a colisão `radiant-norma` no PyPI e o module path duplicado em Go), com contract tests rodando contra o backend real no CI — o bug crossdoc vira impossível por teste, não por memória.
2. **Quickstart de 5 linhas** — o dev avalia a API em minutos:

```python
from radiant import Radiant

c = Radiant(api_key="rn_live_...")                     # sandbox: Radiant(sandbox=True)
doc = c.generate_cadoc("3040", canonical=canonical)    # Canonical → XML + sha256
r = c.validate(doc.xml, cadoc="3040", stream=True)     # L1→L4 (SSE) com provenance
c.sta_submit(doc.xml, idempotency_key="env-202611-3040-001", ambiente="homolog")
print(doc.sha256, r.summary.fn_rate_estimate)          # hash + estimativa do harness
```

3. **Idempotency-Key** obrigatória em todo POST de geração/envio (20-ARQUITETURA): retry de rede nunca duplica remessa; resposta repete o resultado original e marca `idempotency-replayed: true`. A UI web usa a mesma mecânica (o wizard gera a key automaticamente).
4. **Webhooks com replay** — eventos versionados (`validation.completed`, `sta.receipt`, `radar.layout_diff`, `rule.updated`), assinatura HMAC `X-Radiant-Signature`, retry exponencial e **delivery log com Replay** (UI e API): o integrador reproduz qualquer entrega histórica para depurar sem re-disparar processos. Segredo rotacionável sem downtime.

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  /console/webhooks — entregas   endpoint: https://if.com/hooks/radiant (HMAC ok ✓)   │
│  eventos: validation.completed · sta.receipt · radar.layout_diff · rule.updated      │
│  ── entregas (24h) ───────────────────────────────────────────────────────────────── │
│  09:12  sta.receipt        200 ✓  retry 0  latency 180ms          [payload] [Replay] │
│  09:12  validation.compl.  200 ✓  retry 0                           [payload] [Replay]│
│  08:57  radar.layout_diff  500 ✗  retry 2 → sucesso (backoff 1s→4s) [payload] [Replay]│
│  assinatura: X-Radiant-Signature: sha256=…            [segredo]  [rotacionar]        │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

5. **Sandbox com leiautes de teste.** Chaves `rn_test_` vs `rn_live_`; sandbox servido com fixtures por CADOC/data-base (carteiras sintéticas por porte de IF — o corpus do harness vira dado de teste do cliente), STA apontando para `sta-h` (homologação) por padrão, e **leiautes futuros disponíveis antes da vigência** (NR3 e 2030 dez/2026 no sandbox antes da data-base) — é o T2U vendido, experimentável.
6. **Erros que ensinam.** Todo erro de validação retorna estrutura: `rule_cod`, mensagem do catálogo, `§` fonte, campo, sugestão de correção quando existir — o mesmo contrato `<VerdictRow>` da UI. O dev que integra pela API recebe a mesma evidência que o operador vê na tela.
7. **Playground na docs.** Colar um Canonical (ou usar fixture) → validar no sandbox → assistir o streaming L1–L4 na página de docs — a mesma UX do wizard, sem auth de produção. Docs versionadas por changelog da API; breaking change = nova versão com janela de EOL de 90 dias (herança ADR-0004).

---

## 7. UX do copiloto

O backend v1 já tem `/v1/insights/ask`, `ask/stream` (SSE) e `history` — **sem nenhuma UI** (research/03 §9). A v2 entrega o copiloto conforme 21-ENGINE-IA: IA na fronteira, verificação objetiva downstream, humano aprova.

1. **Conversa contextual, não chatbook genérico.** O painel abre a partir de uma verdict/critique com o contexto pré-carregado (CADOC, data-base, campo, regra, último run). Pergunta típica: "por que essa crítica disparou?" — resposta com o cálculo do engine + citação.
2. **Citações como cidadãs do texto.** Cada parágrafo carrega chips `[V3040-0412 § Circ. 3.995 art. 12 II ↗]` clicáveis (mesmo modal da P1). Números na conversa são sempre do engine — o copiloto declara a fonte do número ("cálculo feito pelo engine, não por mim").
3. **Diff-approve como interação central.** A resposta que muda dados vira `FixSuggestionCard` com diff **no Canonical**: [✓ Aprovar & regenerar] → motor regenera XML → re-validação delta → protocolo atualizado. [✎ Corrigir] edita manualmente (e alimenta M2). O copiloto **nunca edita XML direto** — guardrail arquitetural da ADR-0007 ("LLM escreve Canonical, nunca XML") exposto como propriedade visível da UI.
4. **Histórico por CADOC/data-base.** Conversas anexadas à sessão de geração (CADOC + data-base + versão do motor), append-only e auditáveis: o auditor interno consegue ler, junto do envio, a conversa que justificou cada decisão. O delete do endpoint v1 vira "arquivar" — histórico regulatório não se apaga.
5. **Guardrails visíveis no produto (não só no prompt):**
   - sem citação, não responde — a UI recusa renderizar resposta sem fonte verificável;
   - números sempre do engine; LLM interpreta, nunca calcula;
   - nenhuma ação destrutiva ou envio executa sem aprovação humana explícita (diff-approve);
   - streaming com cancelamento e fallback determinístico (se o LLM cai, o wizard segue — o copiloto é opcional em tudo);
   - sanitizador de PII antes de qualquer chamada externa de modelo (F4: router/sanitizador);
   - banner de contexto: o copiloto declara em qual vigência/ruleset está raciocinando.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│  COPILOTO · contexto: 3040 · db 31/10 · crítica V3040-0412      [histórico ▾]    │
├──────────────────────────────────────────────────────────────────────────────────┤
│  você: por que essa crítica disparou?                                            │
│  copiloto: O campo B3.Vencido (84.312,00) difere da soma das 17 parcelas da      │
│  conta 4721 (85.516,30). [V3040-0412 § Circ. 3.995 art. 12 II ↗] A regra vale    │
│  para esta vigência desde 2026-05. O cálculo é do engine (não meu):              │
│  diff R$ 1.204,30.                                                               │
│                                                                                  │
│  ── PROPOSTA (diff no Canonical — nunca no XML) ──                               │
│  B3.Vencido  84.312,00 → 85.516,30                                               │
│  [✓ Aprovar & regenerar]  [✎ Corrigir à mão]  [Dispensar com justificativa]      │
│  aprovar → regenera XML → re-valida delta (2 regras tocadas) → protocolo novo    │
│                                                                                  │
│  você: e se eu dispensar?                                                        │
│  copiloto: A crítica permanece severidade A e o envio segue bloqueado; a         │
│  dispensa fica registrada no audit log e abre dispute p/ triagem.                │
│  [digite… ▸]   guardrail: sem citação não respondo · números vêm do engine       │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Roadmap UX por fase do motor (F0–F6)

Cada fase da reconstrução (20-ARQUITETURA §558–564) tem um entregável de UI cujo critério de pronto é **medido**, não declarado — coerente com P5.

| Fase | Semanas | Entregas de UI | Critério de done (medido) |
|---|---|---|---|
| **F0** — Fundação | 1–2 | Pipeline STATUS machine-generated (`script→JSON→UI`); `<FingerprintBadge>` no rodapé; port do design system (tokens + 10 primitivas) sem alteração visual; suite UI (vitest+Playwright+axe) no CI | badge exibe fingerprint do build real; suite de UI bloqueia 1 PR real |
| **F1** — Domínio | 3–6 | /console/regras v1.5 lendo o catálogo normalizado (schema canônico único, chaves unificadas, dedup); fontes self-host carregadas; P0 3.1–3.9 concluído | catálogo 1.099 navegável 100% via API; zero fallback regex/hardcode; fontes Fraunces/Inter/JBM visíveis no screenshot de teste |
| **F2** — L1–L4 + paridade | 7–12 | **Wizard 2.0 passos ①–④** (ingest, mapear, preview, validar streaming SSE); `<ProvenanceChip>`/`<VerdictRow>`/`<RuleCitation>`; wizard sessions server-side | golden suite 100% green + XML passa no BCValidador — o passo Validar roda L1–L4 real no Playwright e2e |
| **F3** — API + STA | 13–18 | Wizard passos ⑤–⑥ (fix-suggestions + approve/correct com M2; diff visual; payload STA + hash + idempotency); envio homolog real; /console/health v1 (interno); fix-suggestion alimenta corpus (feedback "entrou no corpus nº X") | **primeiro envio STA-h homolog real** feito pela UI; 1 correção de UI virando `eval_case(provenance='user_correction')` |
| **F4** — Radar + IA estágio 1 | 19–24 | **/normativa** (calendário + Radar feed + diff de leiaute visual); **copiloto com citação** (prod interna, diff-approve central); **/console/regras 2.0** com fila rule_candidates; conectores File/API com onboarding honesto | Radar detecta 1 diff real → checklist de reteste na UI; copiloto responde com citação em 100% das respostas auditadas; 1º `rule_candidate(origin='error_cluster')` promovido pela fila |
| **F5** — Launch comercial | 25–30 | **/simulacao** (what-if + leiaute futuro); **/benchmark** (opt-in, k≥5); /console/health público; onboarding self-service ≤15 min (SSO OIDC, quickstart API, sandbox com leiautes de teste); webhooks UI com Replay; páginas de pricing live; white-label theming (tokens por tenant) | **N0 do wedge já fechado** (2–4 contratos até 31/12/2026 — a FASE 2 só começa depois dele); **1 contrato pagante (N4)** assinado com demo dessas telas; TTFD ≤15 min medido em 3 IFs piloto |
| **F6** — Flywheel contínuo | contínuo | Accuracy SLA dashboard (curva FN rate pública); Relatório de Confiabilidade mensal como deliverable (scorecard assinado + changelog de regras); /normativa com IA de impacto por IF ("esta onda afeta X regras suas"); simulação de lançamentos contábeis (4010) | FN rate caindo medido por 4+ semanas; benchmark com ≥15 contribuintes opt-in; health scorecard público estável |

**Leitura transversal:** a UI cresce de dentro para fora — F0–F2 consertam a confiança estrutural (status verdadeiro, validação real, provenance); F3 fecha o loop de correção (M2 visível); F4 abre as telas de inteligência (Radar, copiloto, candidatos); F5 empacota tudo como produto comercial; F6 transforma uso em ativo composto (SLA, benchmark, relatório). Em nenhum momento a UI exibe capacidade que a fase do motor não entrega — o roadmap UX é subserviente ao roadmap do motor por design (P5).

---

## 9. Instrumentação e métricas de UX

| Métrica | Definição | Alvo | Coleta |
|---|---|---|---|
| TTFD (time-to-first-doc) | cadastro → 1º CADOC gerado+validado | ≤15 min (Moat 5 da tese) | eventos de onboarding |
| Wizard e2e rate | sessões que completam ①→⑥ sem abandono | ≥80% na 2ª tentativa | telemetria do wizard (sem PII) |
| Approve latency | tempo entre sugestão e decisão (approve/correct) | p50 ≤2 min | snapshots M2 |
| Correction→eval rate | correções virando `eval_case` | 100% automático | harness M2 |
| Provenance engagement | % de verdicts com chip de regra aberto (primeiro uso) | ≥30% — proxy de confiança ativa | telemetria UI |
| Trust audit | KPIs na UI sem produtor verificável | **0** (gate no CI) | teste automatizado do P1.2 |
| A11y | violações axe nos primitivos | 0 critical/serious | CI |
| NPS operador | survey pós-envio (1 pergunta, opcional) | ≥40 (M2 da tese) | in-app |

---

## 10. Anti-objetivos UX

- ❌ Dashboard de vaidade: KPI que não responde "de onde veio esse número" não entra na tela (P1.2).
- ❌ Autopilot: nenhuma tela envia, corrige ou decide sem aprovação humana explícita (P2/P4).
- ❌ Status escrito à mão: nenhum texto de estado com origem humana na UI (P5).
- ❌ Redesign estético: o "Institutional Premium Noir" não muda por gosto; muda por evidência de uso.
- ❌ Wizard com estado global hackeado (`window.*`), spinner solto, skeleton ausente, modal sem focus trap, cor como único sinal de severidade — todos violações do REDESIGN §6–7 já banidas.
- ❌ Números "que impressionam" sem lastro (a lição mais cara da v1: um só `stableCoverage` por hash custa a credibilidade de toda a demo).

---

## Fontes citadas neste documento

- `docs/blueprint/research/03-frontend-sdk-tooling-audit.md` — mapa completo do frontend v3.36.4 (bug 404 wizard `wizard-client.tsx:554`; placebos `console/page.tsx:100-106` e `envios/page.tsx:81-88`; dependências mortas; SSE hook; fidelidade ao REDESIGN)
- `REDESIGN.md` (raiz do repo) — design system "Institutional Premium Noir" v3.36: tokens, tipografia tripla, motion, ⌘K com focus trap, severity 3 sinais, a11y, CSP
- `docs/blueprint/research/02-tese-vs-realidade.md` — contradições C1–C4 (doc-truth), ADR-0007 (LLM escreve Canonical), "stubs honestos", lição de honestidade incremental
- `docs/blueprint/research/05-concorrentes-mercado.md` — padrões de excelência 2026 (Regnology Nexus/calendário, Norm AI evidência citada, data quality pré-submissão, Agentic Gap 89%), calendário normativo 2026–2027 (IN 733, 694, 750; 3060; GRSAC)
- `docs/blueprint/11-TESE-V2.md` §6 — pilares de produto v2 (provenance em toda verdict; T2U; copiloto com evidência; Accuracy SLA)
- `docs/blueprint/20-ARQUITETURA-RUST.md` — fases F0–F6, ADR-0013 (status machine-generated), idempotency-key, SSE, OpenAPI utoipa
- `docs/blueprint/21-ENGINE-IA.md` — guardrails do copiloto (citação obrigatória, sanitização, verificação objetiva)
- `docs/blueprint/22-HARNESS-V4.md` — M2 (correction→evalpair), Health Scorecard §4.2 (8 métricas), benchmarking opt-in §7.3, gate de completeza §5.1
- `docs/blueprint/23-PRODUTO-PRICING-GTM.md` — tiers, Benchmark Premium R$1.490, desconto como "crédito de dados", gates de upgrade
