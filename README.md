# Agent Pattern Library

A tool for managing and providing human-vetted software patterns to AI agents. Provides an MCP server that agents query for patterns, and a TUI for humans to manage the library.

## How it works

Patterns are Markdown files with YAML front-matter stored on disk. The MCP server exposes four tools that agents use to look up and save patterns. The TUI lets you browse, toggle, and edit patterns interactively.

## Installation

### From source (requires Rust)

```sh
cargo install --path .
```

### With Nix

```sh
nix run github:yourname/agent-library
```

Or enter a dev shell:

```sh
nix develop
```

## Quick start

1. Initialize the library with seed patterns:

```sh
agent-library init
```

This creates `~/.config/agent-library/` (or `$XDG_CONFIG_HOME/agent-library/` on Linux, or whatever `$AGENT_LIBRARY_PATH` points to) with three example patterns and a `ROOT.md` index.

2. Manage patterns with the TUI:

```sh
agent-library tui
```

3. Register the MCP server with Claude Code:

```sh
claude mcp add agent-library -- agent-library serve
```

## Storage

| Platform | Default path |
|----------|-------------|
| macOS    | `~/.config/agent-library/` |
| Linux    | `$XDG_CONFIG_HOME/agent-library/` (falls back to `~/.config/agent-library/`) |

Override on any platform with the `AGENT_LIBRARY_PATH` environment variable:

```sh
export AGENT_LIBRARY_PATH=/path/to/my/patterns
```

## TUI controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Space` / `e` | Toggle enabled/disabled |
| `o` / `Enter` | Open in `$EDITOR` |
| `r` | Refresh from disk |
| `q` / `Ctrl-c` | Quit |

## MCP tools

| Tool | Description |
|------|-------------|
| `lookup_pattern(task_type)` | Returns all enabled patterns for a task type (case-insensitive) |
| `lookup_specific(name)` | Returns a specific pattern by name (case-insensitive) |
| `save_pattern(name, task_type, content)` | Saves a new pattern to the library |
| `browse_patterns()` | Returns the full pattern index (`ROOT.md`) |

## Pattern file format

```markdown
---
name: My Pattern
enabled: true
date_added: 2026-05-12
task_type: Architecture
---
# My Pattern

Pattern content in Markdown here.
```

Patterns are organized into subdirectories by task type (e.g. `patterns/architecture/my-pattern.md`). `ROOT.md` is auto-regenerated whenever patterns change.

## Running tests

```sh
cargo test
```

## Linting

```sh
cargo clippy -- -D warnings
```
