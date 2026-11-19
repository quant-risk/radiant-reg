//! Smoke test do golden loader + runner de paridade (docs/blueprint/
//! 20-ARQUITETURA-RUST.md §6).
//!
//! O corpus `tests/golden/*.json` é gerado por outro agente (o gerador do
//! corpus). Este teste é IGNORÁVEL até lá: se `tests/golden/inventory.json`
//! não existir, retorna cedo — a ausência do corpus não é falha. Mas se
//! existir, o desalinhamento entre corpus materializado e inventário
//! declarado É falha (contagem honesta nos dois lados) e o runner
//! `run_parity` é EXECUTADO sobre o corpus real — com o registry de regras
//! portadas que existir (hoje: nenhum regra 3040/2030 portada em Rust, então
//! o registry é vazio e TODO caso é skip DECLARADO — a paridade executiva é
//! zero e o relatório diz isso em vez de prometer).

use std::path::Path;

use reg_cadoc::golden::{honest_counts, load_cases, load_inventory, parity};

#[test]
#[allow(clippy::unwrap_used)]
fn golden_corpus_smoke() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let inv_path = dir.join("inventory.json");
    if !inv_path.exists() {
        // Corpus ainda não gerado — ignorável por design (o consumidor vem
        // pronto, o conteúdo vem depois). Nunca falhar aqui.
        println!(
            "golden corpus ausente ({}), smoke ignorable",
            inv_path.display()
        );
        return;
    }

    let inv = load_inventory(&dir).unwrap();
    let cases = load_cases(&dir).unwrap();
    let honest = honest_counts(&cases);

    println!(
        "golden corpus: {} casos, {} regras",
        cases.len(),
        inv.rules.len()
    );
    for (rule, n) in &honest {
        println!("  {rule}: {n}");
    }

    // Honestidade: toda regra exercida pelos casos materializados existe no
    // inventário (o inverso não vale — regras orphan/stub podem não ter caso).
    // Casos com rule_code vazio são derivação pendente do GERADOR do corpus:
    // declarados aqui via println, nunca contados como regra exercida.
    let mut sem_regra = 0usize;
    for (rule, n) in &honest {
        if rule.is_empty() {
            sem_regra = *n;
            continue;
        }
        assert!(
            inv.rules.iter().any(|r| &r.code == rule),
            "regra {rule} materializada ({n} casos) não está no inventário"
        );
    }
    if sem_regra > 0 {
        println!("AVISO: {sem_regra} casos com rule_code vazio (derivação pendente no gerador)");
    }
    println!(
        "casos materializados: {} ({} regras exercidas)",
        cases.len(),
        honest.len() - usize::from(sem_regra > 0)
    );

    // Gate de paridade §6 EXECUTADO sobre o corpus real (não só prometido):
    // o registry abaixo é o lugar onde as regras portadas em Rust se
    // registram. Hoje NENHUMA regra do corpus está portada (v1 Go → Rust é
    // pendência declarada), então o registry é vazio e o relatório declara
    // skip total — a "paridade" honesta de hoje é zero implementadas.
    let registry: [&dyn parity::Rule; 0] = [];
    let report = parity::run_parity(&cases, &registry);

    // A contagem do runner fecha com a contagem materializada:
    assert_eq!(report.total, cases.len(), "runner consumiu todos os casos");
    let total_casos_com_regra: usize = honest.values().sum::<usize>();
    assert_eq!(
        report.skipped_not_implemented + report.expect_nao_interpretadas,
        total_casos_com_regra,
        "todo caso com regra exercida é skip DECLARADO (registry vazio hoje)"
    );
    assert_eq!(report.failed, 0, "registry vazio não produz regressão");
    // Comparação honest_counts × inventário: counts.total do inventário ==
    // regras listadas (validado por load_inventory) e toda regra exercida
    // ∈ inventário (assert acima) — os dois lados fecham.
    assert_eq!(inv.total(), Some(inv.rules.len()));

    println!(
        "paridade §6 (corpus real): implemented={} passed={} failed={} skipped_not_implemented={} superficiais={} expect_nao_interpretadas={}",
        report.implemented,
        report.passed,
        report.failed,
        report.skipped_not_implemented,
        report.superficiais,
        report.expect_nao_interpretadas
    );
    println!(
        "skip declarado por regra (as {} regras do corpus ainda não portadas):",
        report.skipped_by_rule.len()
    );
    for (rule, n) in &report.skipped_by_rule {
        println!("  {rule}: {n} casos");
    }
    // Sem regras portadas, o gate VERDE de hoje é o de skip DECLARADO —
    // honesto, não uma promessa de paridade executiva.
    assert!(
        report.is_green(),
        "skip declarado fecha o gate, regressão não"
    );
}
