use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "agent-library",
    version,
    about = "MCP server and TUI for managing AI agent documents",
    long_about = "MCP server and TUI for managing AI agent documents.

STORAGE

  macOS   ~/.config/agent-library/
  Linux   $XDG_CONFIG_HOME/agent-library/ (falls back to ~/.config/agent-library/)

  Override on any platform with the AGENT_LIBRARY_PATH environment variable.

  Library layout:
    <library>/
    ├── documents/   markdown files organized by task type
    ├── themes/      base16 YAML theme files
    └── ROOT.md      auto-generated index (never edit by hand)

DOCUMENT FORMAT

  Documents are Markdown files with YAML front-matter:

    ---
    name: My Document
    enabled: true
    date_added: 2026-05-12
    task_type: Architecture
    ---
    # Content here

  Add isSkill: true to expose a document as a callable MCP tool. The tool name
  is the slugified document name (e.g. my-workflow). Only enabled skill documents
  are exposed."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch the MCP server (JSON-RPC 2.0 over stdio)
    #[command(long_about = "Launch the MCP server (JSON-RPC 2.0 over stdio).

MCP TOOLS

  browse_documents                    Returns ROOT.md — index grouped by task type
  lookup_specific(name)               Returns a document by name (case-insensitive)
  save_document(name, task_type, content)
                                      Writes a new document; regenerates ROOT.md
  update_document(name, [content], [task_type], [enabled], [isSkill])
                                      Updates fields on an existing document
  <skill-name>()                      Any enabled document with isSkill: true is
                                      exposed as its own tool

RECOMMENDED AGENT WORKFLOW

  1. Call browse_documents to see all available documents.
  2. Call lookup_specific(name) for the exact document needed, or invoke its
     skill tool directly if it has one.")]
    Serve,
    /// Open the TUI for managing documents
    #[command(long_about = "Open the TUI for managing documents.

CONTROLS

  j / ↓       Move down
  k / ↑       Move up
  Space / e   Toggle enabled/disabled
  s           Toggle skill (exposes document as a callable MCP tool)
  o / Enter   Open in $EDITOR
  d           Delete (prompts for confirmation)
  r           Refresh from disk
  R           Rebuild ROOT.md
  T           Open theme picker
  q / Ctrl-c  Quit

  Skill documents are marked with ★. The bottom bar shows keybindings; status
  feedback appears on the line below it.

THEMES

  The TUI uses base16 themes. Four Catppuccin variants are bundled:

    Catppuccin Mocha      Dark (default)
    Catppuccin Macchiato  Dark
    Catppuccin Frappe     Medium dark
    Catppuccin Latte      Light

  Theme files live in <library>/themes/ as standard base16 YAML. Any *.yaml or
  *.yml file in that directory appears in the picker. Press T to browse; navigate
  with j/k, confirm with Enter, cancel with Esc. The selection is session-only —
  use --theme to set a default on launch.

  Custom theme format:

    scheme: \"My Theme\"
    author: \"you\"
    base00: \"1e1e2e\"   # background
    base01: \"181825\"   # lighter background
    base02: \"313244\"   # selection background
    base03: \"45475a\"   # comments / subtle text
    base04: \"585b70\"   # dark foreground
    base05: \"cdd6f4\"   # default foreground
    base06: \"f5e0dc\"   # light foreground
    base07: \"b4befe\"   # light background
    base08: \"f38ba8\"   # red (disabled, errors)
    base09: \"fab387\"   # peach (constants)
    base0a: \"f9e2af\"   # yellow (skills ★)
    base0b: \"a6e3a1\"   # green (enabled)
    base0c: \"94e2d5\"   # teal (status messages)
    base0d: \"89b4fa\"   # blue (headings)
    base0e: \"cba6f7\"   # mauve (keywords)
    base0f: \"f2cdcd\"   # flamingo (deprecated)

  Restore bundled themes: agent-library init --seed-themes")]
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
