use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "agent-library",
    version,
    about = "MCP server and TUI for managing AI agent patterns"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch the MCP server (JSON-RPC 2.0 over stdio)
    Serve,
    /// Open the TUI for managing patterns
    Tui,
    /// Scaffold the pattern library with seed patterns
    Init,
}
