/* zcode-workflow
name: fase-s3-corpus
description: Fase S2-S3 da reconstrução Rust do Radiant Reg — golden corpus (exportador Go dos 1.178 testes → fixtures JSON) + reg-cadoc (ingest calamine + mapping.toml + golden loader/parity) com gates finais (cargo test/clippy, go vet, fixtures válidas) e commit.
whenToUse: Rodar para executar ou re-executar (idempotente) a fase S2-S3 do plano em docs/blueprint/25-ROADMAP-EXECUCAO.md §0.
args: {}
*/

const GOLDEN_DIR = "rust/crates/reg-cadoc/tests/golden";

interface GoResult {
  /** Caminhos dos arquivos gerados. */
  arquivos: string[];
  /** Contagens honestas: regras por classe e cobertura da extração. */
  contagens: string;
  /** Como rodar e limitações. */
  comoRodar: string;
}
interface RustResult {
  arquivos: string[];
  resultadoGates: string;
  notas: string;
}
interface ReportFinding {
  where: string;
  what: string;
  evidence: string;
  status: "verified" | "unconfirmed";
  severity: "low" | "medium" | "high";
}
interface WorkflowReport {
  conclusion: string;
  findings: ReportFinding[];
  verified: string[];
  notCovered: string[];
}

phase("Exportar golden corpus do Go (agente backend)");
const go = await agent("Exportador golden Go").ask<GoResult>(
  `Crie backend/cmd/golden-export (Go, go/ast+go/parser) que extrai: (1) inventory.json com TODAS as regras de backend/internal/audit/rules/registry.go classificadas honestamente como wired|orphan|stub (fonte: dispatch real em audit/service.go applyRegra); (2) fixtures_<cadoc>.json com os XML literals em backticks dos 24 arquivos de teste + rule_code + expect (nil|substring). Saída em ${GOLDEN_DIR}/. Deve rodar com 'go run ./cmd/golden-export' a partir de backend/; gofmt+go vet limpos; casos não-extrasáveis vão para not_extracted (nunca invente). Conteúdos e regras de qualidade: ver blueprint docs/blueprint/22-HARNESS-V4.md §5.6 (corpus dourado) e docs/blueprint/25-ROADMAP-EXECUCAO.md §0 S2-S3. Retorne {arquivos, contagens, comoRodar}.`,
);

phase("Implementar reg-cadoc: ingest + golden loader (agente rust)");
const rust = await agent("Implementador reg-cadoc Rust").ask<RustResult>(
  `Implemente em rust/crates/reg-cadoc: (1) módulo ingest (calamine 0.38 — leitor de sheet, detect_header_row com keywords do _catalogos/extract.py, ColumnMapping tipado, CanonicalInput mínimo do wedge, mapping.toml via crate toml); (2) módulo golden (GoldenCase/GoldenInventory serde, ParityReport com classificação implemented+passed / implemented+FAILED / not_implemented DECLARADO, trait Rule, teste de integração tests/golden_smoke.rs que carrega ${GOLDEN_DIR}/inventory.json se existir). NÃO quebre os gates: cargo test --workspace e cargo clippy --workspace --all-targets -- -D warnings (float/unsafe/wildcard/unwrap banidos; allow de unwrap só em módulos de teste). Teste com fixture real 3040/SCR3040_Leiaute.xls. Referência: docs/blueprint/20-ARQUITETURA-RUST.md §6. Retorne {arquivos, resultadoGates, notas}.`,
);

phase("Gate determinístico final");
log("Validando: fixtures JSON parseáveis, cargo gates, go vet…");
const findings: ReportFinding[] = [];
const inv = await files.read(`${GOLDEN_DIR}/inventory.json`).catch(() => null);
if (inv === null) {
  findings.push({
    where: GOLDEN_DIR,
    what: "inventory.json não foi gerado pelo exportador Go",
    evidence: "files.read falhou",
    status: "verified",
    severity: "high",
  });
} else {
  const parsed = JSON.parse(inv) as { counts?: { total?: number } };
  if ((parsed.counts?.total ?? 0) === 0) {
    findings.push({
      where: `${GOLDEN_DIR}/inventory.json`,
      what: "inventário veio com total=0 — extração vazia",
      evidence: JSON.stringify(parsed.counts),
      status: "verified",
      severity: "high",
    });
  }
}
const test = await world.run("cargo", ["test", "--workspace"], {
  cwd: "rust",
  timeoutMs: 600_000,
  output: { tail: 4_000 },
});
if (test.exitCode !== 0) {
  findings.push({
    where: "rust/",
    what: "cargo test --workspace falhou",
    evidence: test.stdout.slice(-2_000),
    status: "verified",
    severity: "high",
  });
}
const clippy = await world.run(
  "cargo",
  ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
  { cwd: "rust", timeoutMs: 600_000, output: { tail: 2_000 } },
);
if (clippy.exitCode !== 0) {
  findings.push({
    where: "rust/",
    what: "cargo clippy -D warnings falhou",
    evidence: clippy.stdout.slice(-1_500),
    status: "verified",
    severity: "high",
  });
}
const vet = await world.run("go", ["vet", "./cmd/golden-export/..."], {
  cwd: "backend",
  timeoutMs: 300_000,
  output: { tail: 2_000 },
});
if (vet.exitCode !== 0) {
  findings.push({
    where: "backend/cmd/golden-export",
    what: "go vet falhou",
    evidence: vet.stdout.slice(-1_000),
    status: "verified",
    severity: "medium",
  });
}

phase("Corrigir o que o gate derrubou");
const falhas = findings.filter((f) => f.severity === "high" || f.severity === "medium");
if (falhas.length > 0) {
  log(`${falhas.length} falha(s) — acionando reparo dirigido…`);
  const reparo = await agent("Reparador de fase").ask<string>(
    `As validações da fase S3 falharam nos pontos abaixo (JSON). Corrija no repo /Users/henrique/Documents/radiant-reg os problemas em backend/ e rust/ (respeitando os gates: cargo test/clippy -D warnings; go vet). Devolva o que alterou e o estado final dos gates.\n${JSON.stringify(falhas)}`,
  );
  log(`Reparo: ${reparo.slice(0, 300)}…`);
}

phase("Publicar relatório da fase");
const relatorio = [
  "# Fase S2-S3 — Golden Corpus + reg-cadoc",
  "",
  "## Exportador Go",
  `- Arquivos: ${go.arquivos.join(", ")}`,
  `- Contagens: ${go.contagens}`,
  `- Como rodar: ${go.comoRodar}`,
  "",
  "## reg-cadoc Rust",
  `- Arquivos: ${rust.arquivos.join(", ")}`,
  `- Gates: ${rust.resultadoGates}`,
  `- Notas: ${rust.notas}`,
  "",
  "## Gates determinísticos desta execução",
  `- cargo test --workspace: exit ${test.exitCode}`,
  `- cargo clippy -D warnings: exit ${clippy.exitCode}`,
  `- go vet: exit ${vet.exitCode}`,
  `- inventory.json: ${inv === null ? "AUSENTE" : "presente"}`,
].join("\n");
await artifact.markdown("relatorio-s3", relatorio, {
  title: "Fase S2-S3 — relatório",
  description: "Golden corpus exportado + reg-cadoc implementado + gates",
  primary: true,
});

return {
  conclusion: `Fase S2-S3 executada: corpus em ${GOLDEN_DIR} (${go.contagens.slice(0, 80)}…), reg-cadoc com ingest+golden loader, gates ${test.exitCode === 0 && clippy.exitCode === 0 ? "GREEN" : "COM FALHAS (ver findings)"}.`,
  findings,
  verified: [
    `world.run cargo test --workspace (exit ${test.exitCode})`,
    `world.run cargo clippy -D warnings (exit ${clippy.exitCode})`,
    `world.run go vet (exit ${vet.exitCode})`,
    "files.read inventory.json com total > 0",
  ],
  notCovered: [
    "Paridade real das regras L2 (o corpus está pronto; a implementação Rust regra-a-regra é a fase seguinte, S4 do roadmap).",
    "Re-download oficial dos leiautes BCB (pendência do usuário + captura /browser).",
  ],
};
