# Agent Document Library

![screenshot](img/screenshot_1.png)

A tool for managing and providing human-vetted software documents to AI agents. Provides an MCP server that agents query for documents, and a TUI for humans to manage the library.

I built this after being moved between 3 different AI tools at work, while I could technically implement these things in skills, it meant working out the configuration for each tool and lugging my files around all over the place. This MCP provides a centralized location for snippets, documents and style guides.

## Disclosure

This was built using Claude Code with Opus 4.7.

## How it works

Documents are Markdown files with YAML front-matter stored on disk. The MCP server exposes tools that agents use to look up and save documents. The TUI lets you browse, toggle, edit, and delete documents interactively.

Documents marked `isSkill: true` are exposed as their own callable MCP tools — the agent can invoke them directly by name rather than looking them up.

## Installation

### From source (requires Rust)

```sh
cargo install --path .
```

### With Nix

```sh
nix run git+https://codeberg.org/westongpt/agent-library
```

Or enter a dev shell:

```sh
nix develop git+https://codeberg.org/westongpt/agent-library
```

## Quick start

1. Initialize the library with seed documents and themes:

```sh
agent-library init
```

This creates `~/.config/agent-library/` (or `$XDG_CONFIG_HOME/agent-library/` on Linux, or whatever `$AGENT_LIBRARY_PATH` points to) with three example documents, a `ROOT.md` index, and four Catppuccin theme files.

2. Manage documents with the TUI:

```sh
agent-library tui
```

Start with a specific theme using `--theme`:

```sh
agent-library tui --theme "Catppuccin Latte"
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
export AGENT_LIBRARY_PATH=/path/to/my/documents
```

The library directory contains:

```
<library>/
├── documents/        # markdown documents, organized by task type
├── themes/           # base16 YAML theme files
└── ROOT.md           # auto-generated index (never edit by hand)
```

## TUI controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Space` / `e` | Toggle enabled/disabled |
| `s` | Toggle skill (exposes document as a callable MCP tool) |
| `o` / `Enter` | Open in `$EDITOR` |
| `d` | Delete (prompts for confirmation) |
| `r` | Refresh from disk |
| `R` | Rebuild `ROOT.md` |
| `T` | Open theme picker |
| `q` / `Ctrl-c` | Quit |

Skill documents are marked with a `★` in the list. The bottom bar always shows the keybindings; status feedback appears on the line below it.

### Theme picker

Press `T` to open the theme picker. Navigate with `j`/`k`, confirm with `Enter`, cancel with `Esc`. The selection is session-only — use `--theme` to set a default on launch.

## Themes

The TUI uses [base16](https://github.com/chriskempson/base16) themes. Four [Catppuccin](https://github.com/catppuccin/catppuccin) variants are bundled:

| Theme | Style |
|-------|-------|
| Catppuccin Mocha | Dark (default) |
| Catppuccin Macchiato | Dark |
| Catppuccin Frappe | Medium dark |
| Catppuccin Latte | Light |

Theme files live in `<library>/themes/` as standard base16 YAML. You can edit or delete the bundled themes, and add your own — any `*.yaml` or `*.yml` file in that directory will appear in the picker.

**Custom theme format:**

```yaml
scheme: "My Theme"
author: "you"
base00: "1e1e2e"   # background
base01: "181825"   # lighter background
base02: "313244"   # selection background
base03: "45475a"   # comments / subtle text
base04: "585b70"   # dark foreground
base05: "cdd6f4"   # default foreground
base06: "f5e0dc"   # light foreground
base07: "b4befe"   # light background
base08: "f38ba8"   # red (disabled, errors)
base09: "fab387"   # peach (constants)
base0a: "f9e2af"   # yellow (skills ★)
base0b: "a6e3a1"   # green (enabled)
base0c: "94e2d5"   # teal (status messages)
base0d: "89b4fa"   # blue (headings)
base0e: "cba6f7"   # mauve (keywords)
base0f: "f2cdcd"   # flamingo (deprecated)
```

To restore the bundled themes after editing or deleting them:

```sh
agent-library init --seed-themes
```

To re-add the seed documents to an existing library:

```sh
agent-library init --seed-documents
```

## MCP tools

| Tool | Description |
|------|-------------|
| `lookup_specific(name)` | Returns a specific document by name (case-insensitive) |
| `save_document(name, task_type, content)` | Saves a new document to the library |
| `update_document(name, [content], [task_type], [enabled], [isSkill])` | Updates an existing document by name; supply at least one optional field |
| `browse_documents()` | Returns the document index (`ROOT.md`) grouped by task type |
| `<skill-name>()` | Any document with `isSkill: true` is exposed as its own tool, named by slugifying the document name |

### Recommended agent workflow

1. Call `browse_documents` to see all available documents.
2. Call `lookup_specific(name)` for the exact document needed, or invoke its skill tool directly.

## Claude Code skill: `/intake-document`

The repo ships a Claude Code slash command that turns any document into a library entry in one step.

### Install

**One-liner** (installs to `~/.claude/commands/` so it's available in every project):

```sh
curl -fsSL https://codeberg.org/westongpt/agent-library/raw/branch/main/install-skill.sh | sh
```

**From a clone:**

```sh
./install-skill.sh
```

Restart Claude Code (or run `/reload`) after installing.

### Usage

(Requires the MCP server to be registered — see [Quick start](#quick-start).)

```
/intake-document path/to/document.md
/intake-document https://example.com/some-guide
/intake-document Use dependency injection for all service constructors...
```

The skill accepts a file path, a URL, or inline text. It will:

1. Read or fetch the content.
2. Call `browse_documents` to check existing task types and prefer matching one over creating a new one.
3. Infer a concise `name` and the best-fit `task_type`.
4. Reformat the content into clean Markdown if needed, preserving well-structured documents as-is.
5. Call `save_document` and report the name, task type, and saved file path.

## Document file format

```markdown
---
name: My Document
enabled: true
date_added: 2026-05-12
task_type: Architecture
---
# My Document

Document content in Markdown here.
```

To expose a document as a callable MCP tool, add `isSkill: true`:

```markdown
---
name: My Workflow
enabled: true
date_added: 2026-05-12
task_type: Workflow
isSkill: true
---
# My Workflow
...
```

The tool name is the slugified document name (e.g. `my-workflow`). It appears in `tools/list` and can be called directly by the agent. Only enabled skill documents are exposed.

Documents are organized into subdirectories by task type (e.g. `documents/architecture/my-document.md`). `ROOT.md` is auto-regenerated whenever documents change — never edit it by hand.

## Running tests

```sh
cargo test
```

## Linting

```sh
cargo clippy -- -D warnings
```
