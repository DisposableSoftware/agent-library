# Agent Pattern Library

A Rust CLI tool for managing and providing human-vetted software patterns to AI agents. Runs as an MCP server (JSON-RPC 2.0 over stdio) and includes a TUI for human management.

## Architecture

Single binary (`agent-library`) with three subcommands:

- `agent-library init` — scaffold the library with seed patterns
- `agent-library serve` — run as an MCP server over stdio
- `agent-library tui` — open the terminal UI

## Storage

Patterns are Markdown files with YAML front-matter, stored under a configurable base directory:

- **macOS:** `~/.config/agent-library/`
- **Linux:** `$XDG_CONFIG_HOME/agent-library/` (falls back to `~/.config/agent-library/`)
- **Override (all platforms):** `$AGENT_LIBRARY_PATH`

Files are organized as `patterns/<task-type-slug>/<pattern-slug>.md`. `ROOT.md` lives at the base directory root and is auto-regenerated after every mutation — never edit it by hand.

## Pattern File Format

```markdown
---
name: REST API Design
enabled: true
date_added: 2026-05-12
task_type: Architecture
---
# Pattern content in Markdown here
```

Front-matter fields:
- `name` — display name (string)
- `enabled` — whether agents can retrieve it (bool)
- `date_added` — ISO date `YYYY-MM-DD`
- `task_type` — category string (e.g. `Architecture`, `Style Guide`, `Documentation`)

## MCP Tools

All four tools return MCP `text` content blocks.

| Tool | Params | Description |
|------|--------|-------------|
| `lookup_pattern` | `task_type: string` | All enabled patterns matching the type (case-insensitive) |
| `lookup_specific` | `name: string` | A single pattern by name (case-insensitive) |
| `save_pattern` | `name, task_type, content` | Write a new pattern; regenerates ROOT.md |
| `browse_patterns` | — | Returns ROOT.md verbatim |

## Source Layout

```
src/
├── main.rs          — subcommand dispatch
├── cli.rs           — clap CLI definitions
├── config.rs        — resolve_library_path() with platform/env logic
├── error.rs         — AppError enum
├── pattern.rs       — Pattern struct, front-matter parser, slug generation
├── seeds.rs         — embedded seed pattern content
├── store.rs         — PatternStore: load_all, save_pattern, update_pattern, regenerate_root
├── mcp/
│   ├── mod.rs       — stdio JSON-RPC read loop (newline-delimited, flush after every write)
│   ├── types.rs     — RpcRequest/Response/Error types and JSON-RPC error constants
│   └── tools.rs     — tool handler implementations
└── tui/
    ├── mod.rs       — terminal setup/teardown and event loop
    ├── app.rs       — App state (patterns list, cursor, mode)
    ├── ui.rs        — ratatui render functions
    └── events.rs    — crossterm events → AppEvent
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `serde` + `serde_json` | JSON serialization (MCP protocol) |
| `serde_yml` | YAML front-matter parsing (`serde_yaml` is deprecated — do not use it) |
| `ratatui` + `crossterm` | Terminal UI |

No async runtime. MCP stdio is synchronous; the TUI uses crossterm's blocking event reads.

## Key Implementation Notes

- **Stdout discipline:** MCP JSON-RPC is on stdout. All logging/debug output must go to stderr or it corrupts the protocol stream.
- **Flush after every MCP write:** `stdout.lock().flush()` after every `writeln!` — pipe-buffered stdout will hang the host.
- **Atomic file writes:** patterns are written to `.tmp` then renamed (POSIX atomic rename).
- **ROOT.md is auto-generated:** regenerated in full after every `save_pattern` or `update_pattern` call. Sorted by task type then name for deterministic output.
- **Notifications vs requests:** MCP `initialized` is a notification (no `id`); the server returns no response. Requests always have an `id`.
- **TUI + editor:** raw mode is disabled before spawning `$EDITOR` and re-enabled after. Editor lookup order: `$VISUAL` → `$EDITOR` → `vi`.
- **Slug generation:** pattern names are lowercased and non-alphanumeric characters replaced with hyphens. Collisions append `-2`, `-3`, etc.

## Testing

```sh
cargo test          # 38 unit tests across all modules
cargo clippy -- -D warnings
```

Tests use inline `#[cfg(test)]` modules. Filesystem tests create isolated temp directories using PID + atomic counter to avoid collisions between parallel tests and across runs.

## Nix

`flake.nix` provides a package and dev shell for `x86_64-linux`, `aarch64-linux`, `aarch64-darwin`, and `x86_64-darwin`. Uses `rustPlatform.buildRustPackage` with `cargoLock.lockFile` — `Cargo.lock` must remain committed.

```sh
nix build           # build the binary
nix develop         # enter dev shell with rustc, cargo, rust-analyzer, clippy, rustfmt
```
