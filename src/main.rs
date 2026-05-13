mod cli;
mod config;
mod document;
mod error;
mod mcp;
mod seeds;
mod store;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};
use config::resolve_library_path;
use store::DocumentStore;

fn main() {
    let cli = Cli::parse();
    let store = DocumentStore::new(resolve_library_path());

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

fn cmd_init(store: &DocumentStore) -> Result<(), error::AppError> {
    let dir = store.documents_dir();
    if dir.exists() {
        eprintln!(
            "Document library already exists at {}",
            store.base_dir.display()
        );
        eprintln!("Remove it or set AGENT_LIBRARY_PATH to a different location.");
        std::process::exit(1);
    }

    std::fs::create_dir_all(&dir)?;
    seeds::seed_documents(store)?;

    println!(
        "Initialized agent-library at {}",
        store.base_dir.display()
    );
    println!("Created 3 seed documents. Run `agent-library tui` to manage them.");
    Ok(())
}
