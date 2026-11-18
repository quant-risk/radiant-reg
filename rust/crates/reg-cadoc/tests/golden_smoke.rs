//! Smoke test do golden loader (docs/blueprint/20-ARQUITETURA-RUST.md §6).
//!
//! O corpus `tests/golden/*.json` é gerado por outro agente (o gerador do
//! corpus). Este teste é IGNORÁVEL até lá: se `tests/golden/inventory.json`
//! não existir, retorna cedo — a ausência do corpus não é falha. Mas se
//! existir, o desalinhamento entre corpus materializado e inventário
//! declarado É falha (contagem honesta nos dois lados).

use std::path::Path;

use reg_cadoc::golden::{honest_counts, load_cases, load_inventory};

#[test]
#[allow(clippy::unwrap_used)]
fn golden_corpus_smoke() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let inv_path = dir.join("inventory.json");
    if !inv_path.exists() {
        // Corpus ainda não gerado — ignorável por design (o consumidor vem
        // pronto, o conteúdo vem depois). Nunca falhar aqui.
        println!("golden corpus ausente ({}), smoke ignorable", inv_path.display());
        return;
    }

    let inv = load_inventory(&dir).unwrap();
    let cases = load_cases(&dir).unwrap();
    let honest = honest_counts(&cases);

    println!("golden corpus: {} casos, {} regras", cases.len(), inv.rules.len());
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
    println!("casos materializados: {} ({} regras exercidas)", cases.len(), honest.len() - usize::from(sem_regra > 0));
}
