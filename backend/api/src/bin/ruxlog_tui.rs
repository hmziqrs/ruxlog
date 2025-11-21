use std::error::Error;
use std::str::FromStr;

use clap::Parser;

use ruxlog::core::config::CoreConfig;
use ruxlog::tui::{app::run_tui, theme::ThemeKind};

#[derive(Parser, Debug)]
#[command(name = "ruxlog_tui", about = "Ruxlog TUI (auth + tags)")]
struct Args {
    /// Theme name: dracula | onedark | material
    #[arg(long, default_value = "dracula")]
    theme: String,
    /// Optional positional theme override (e.g. `ruxlog_tui dracula`)
    #[arg()]
    theme_positional: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Try standard `.env` first (for direct cargo runs)
    dotenvy::dotenv().ok();
    // If POSTGRES_USER is still missing, fall back to ../../.env.dev
    if std::env::var("POSTGRES_USER").is_err() {
        let _ = dotenvy::from_filename("../../.env.dev");
    }

    let args = Args::parse();
    // Allow both `--theme foo` and bare `foo` positional.
    let theme_name = args
        .theme_positional
        .get(0)
        .map(String::as_str)
        .unwrap_or(&args.theme);
    let theme = ThemeKind::from_str(theme_name).unwrap_or(ThemeKind::Dracula);

    let core_config = CoreConfig::from_env();

    if let Err(err) = run_tui(core_config, theme).await {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

