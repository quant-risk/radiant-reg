//! Gate arquitetural: prova a direção das dependências entre camadas.
//!
//! reg-core ← reg-cadoc ← reg-cli
//! reg-core ← reg-ledger ← reg-cli
//! reg-cadoc ✗ reg-ledger (camadas 1 e 2 não se conhecem no wedge)
//!
//! Como crates Rust não exportam grafos de dependência em runtime, o gate
//! funciona por compilação-cruzada + inspeção do próprio Cargo.toml fonte:
//! se alguém adicionar uma dependência ilegal, este teste falha lendo o TOML.

fn read_toml(name: &str) -> String {
    let base = std::fs::canonicalize(env!("CARGO_MANIFEST_DIR"))
        .unwrap_or_else(|e| panic!("canonicalizar CARGO_MANIFEST_DIR: {e}"));
    let path = base.join("..").join(name).join("Cargo.toml");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("ler {}: {e}", path.display()))
}

#[test]
fn reg_core_nao_depende_de_nenhum_irmao() {
    let toml = read_toml("reg-core");
    for irmao in ["reg-cadoc", "reg-ledger", "reg-cli"] {
        assert!(
            !toml.contains(irmao),
            "reg-core não pode depender de {irmao} (camada 0 é folha)"
        );
    }
}

#[test]
fn reg_cadoc_soh_depende_de_reg_core() {
    let toml = read_toml("reg-cadoc");
    assert!(toml.contains("reg-core"), "reg-cadoc depende de reg-core");
    for proibido in ["reg-ledger", "reg-cli"] {
        assert!(
            !toml.contains(proibido),
            "reg-cadoc não pode depender de {proibido} (camada 1 só vê a 0)"
        );
    }
}

#[test]
fn reg_ledger_nao_depende_de_reg_cadoc() {
    let toml = read_toml("reg-ledger");
    assert!(toml.contains("reg-core"), "reg-ledger depende de reg-core");
    for proibido in ["reg-cadoc", "reg-cli"] {
        assert!(
            !toml.contains(proibido),
            "reg-ledger não pode depender de {proibido} (camada 2 só vê a 0)"
        );
    }
}

#[test]
fn reg_cli_importa_as_tres_camadas() {
    let toml = read_toml("reg-cli");
    for camada in ["reg-core", "reg-cadoc", "reg-ledger"] {
        assert!(toml.contains(camada), "reg-cli deve importar {camada}");
    }
}

#[test]
fn floats_banidos_do_workspace() {
    // A doutrina zero-float é lint (workspace.lints.clippy), mas o texto do
    // Cargo.toml raiz é a prova documental — se alguém remover os lints,
    // este teste aponta o dedo.
    let base = std::fs::canonicalize(env!("CARGO_MANIFEST_DIR"))
        .unwrap_or_else(|e| panic!("canonicalizar CARGO_MANIFEST_DIR: {e}"));
    // reg-cli fica em rust/crates/reg-cli → a raiz do workspace é dois níveis acima.
    let root = base.join("..").join("..").join("Cargo.toml");
    let toml =
        std::fs::read_to_string(&root).unwrap_or_else(|e| panic!("ler {}: {e}", root.display()));
    for lint in ["float_arithmetic", "float_cmp", "cast_precision_loss", "wildcard_enum_match_arm"] {
        assert!(toml.contains(lint), "workspace Cargo.toml perdeu o lint {lint}");
    }
}
