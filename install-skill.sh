#!/bin/sh
# Install the /intake-pattern Claude Code skill to ~/.claude/commands/
# Works whether you're running from the cloned repo or piping from the web.

set -e

DEST="$HOME/.claude/commands/intake-pattern.md"
RAW_URL="https://codeberg.org/westongpt/agent-library/raw/branch/main/.claude/commands/intake-pattern.md"

mkdir -p "$(dirname "$DEST")"

if [ -f "$DEST" ]; then
    printf "intake-pattern is already installed at %s\n" "$DEST"
    printf "Overwrite? [y/N] "
    read -r reply
    case "$reply" in
        [Yy]*) ;;
        *) echo "Aborted."; exit 0 ;;
    esac
fi

# Prefer copying from the local repo if this script is run from within it.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LOCAL="$SCRIPT_DIR/.claude/commands/intake-pattern.md"

if [ -f "$LOCAL" ]; then
    cp "$LOCAL" "$DEST"
    echo "Installed from local repo: $DEST"
elif command -v curl >/dev/null 2>&1; then
    curl -fsSL "$RAW_URL" -o "$DEST"
    echo "Installed from $RAW_URL"
    echo "Saved to: $DEST"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$DEST" "$RAW_URL"
    echo "Installed from $RAW_URL"
    echo "Saved to: $DEST"
else
    echo "Error: neither curl nor wget found. Install one and retry, or copy manually:"
    echo "  $RAW_URL -> $DEST"
    exit 1
fi

echo ""
echo "Restart Claude Code (or run /reload) and use:"
echo "  /intake-pattern <file | url | text>"
