mod cli;
mod config;
mod document;
mod error;
mod mcp;
mod prefs;
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
        Commands::Init { seed_themes, seed_documents } => cmd_init(&store, seed_themes, seed_documents),
        Commands::Serve => mcp::run_server(store),
        Commands::Tui { theme } => tui::run_tui(store, theme),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn cmd_init(store: &DocumentStore, seed_themes: bool, seed_documents: bool) -> Result<(), error::AppError> {
    // Selective seeding: skip the fresh-install check when a flag is given.
    if seed_themes || seed_documents {
        if seed_themes {
            seeds::seed_themes(&store.base_dir)?;
            println!("Wrote 4 Catppuccin theme files to {}/themes/.", store.base_dir.display());
        }
        if seed_documents {
            std::fs::create_dir_all(store.documents_dir())?;
            seeds::seed_documents(store)?;
            println!("Added seed documents to {}.", store.base_dir.display());
        }
        return Ok(());
    }

    // Full init: require a clean slate.
    let dir = store.documents_dir();
    if dir.exists() {
        eprintln!(
            "Document library already exists at {}",
            store.base_dir.display()
        );
        eprintln!("Remove it or set AGENT_LIBRARY_PATH to a different location.");
        eprintln!("To add themes to an existing library: agent-library init --seed-themes");
        std::process::exit(1);
    }

    std::fs::create_dir_all(&dir)?;
    seeds::seed_documents(store)?;
    seeds::seed_themes(&store.base_dir)?;

    println!("Initialized agent-library at {}.", store.base_dir.display());
    println!("Created 3 seed documents and 4 Catppuccin themes.");
    println!("Run `agent-library tui` to manage documents. Press T to pick a theme.");
    Ok(())
}
