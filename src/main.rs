mod cli;
mod config;
mod error;
mod mcp;
mod pattern;
mod seeds;
mod store;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};
use config::resolve_library_path;
use store::PatternStore;

fn main() {
    let cli = Cli::parse();
    let store = PatternStore::new(resolve_library_path());

    let result = match cli.command {
        Commands::Init => cmd_init(&store),
        Commands::Serve => mcp::run_server(store),
        Commands::Tui => tui::run_tui(store),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn cmd_init(store: &PatternStore) -> Result<(), error::AppError> {
    let dir = store.patterns_dir();
    if dir.exists() {
        eprintln!(
            "Pattern library already exists at {}",
            store.base_dir.display()
        );
        eprintln!("Remove it or set AGENT_LIBRARY_PATH to a different location.");
        std::process::exit(1);
    }

    std::fs::create_dir_all(&dir)?;
    seeds::seed_patterns(store)?;

    println!(
        "Initialized agent-library at {}",
        store.base_dir.display()
    );
    println!("Created 3 seed patterns. Run `agent-library tui` to manage them.");
    Ok(())
}
