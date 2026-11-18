//! reg-cli — Camada 3: composition root do wedge V1.
//!
//! CLI determinística (exit codes 0 aceito / 1 rejeição-transporte /
//! 2 validação-cliente / 3 erro-formal-BACEN), `--offline` por padrão,
//! `--json` para pipelines. Subcomandos a implementar em S2-S6:
//! `generate`, `validate`, `xray`, `dossier`, `sta`, `senhaws`, `shadow-diff`, `mutate`.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "reg-cli",
    about = "Radiant Reg — motor de CADOC determinístico (wedge V1)",
    version
)]
struct Cli {
    /// Rodar sem qualquer acesso de rede (padrão de produção on-perimeter).
    /// Desligue com `--offline=false` — a saída da rede é sempre opt-out
    /// explícito, nunca o contrário.
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    offline: bool,
    /// Saída em JSON estruturado (para Airflow/Dagster).
    #[arg(long)]
    json: bool,
}

fn main() -> std::process::ExitCode {
    let _cli = Cli::parse();
    // S2-S6 substituem este retorno pelos subcomandos reais.
    eprintln!("reg-cli: esqueleto da Fase 1 — nenhum subcomando implementado ainda");
    std::process::ExitCode::from(2)
}
