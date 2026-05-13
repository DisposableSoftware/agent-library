use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "agent-library",
    version,
    about = "MCP server and TUI for managing AI agent documents"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch the MCP server (JSON-RPC 2.0 over stdio)
    Serve,
    /// Open the TUI for managing documents
    Tui {
        /// Theme name to start with (e.g. "Catppuccin Mocha", "Catppuccin Latte").
        /// Press T inside the TUI to open the theme picker.
        #[arg(long)]
        theme: Option<String>,
    },
    /// Scaffold the document library with seed documents and themes
    Init {
        /// Write default theme files to <library>/themes/, overwriting existing ones
        #[arg(long)]
        seed_themes: bool,
        /// Add default seed documents to the library
        #[arg(long)]
        seed_documents: bool,
    },
}
