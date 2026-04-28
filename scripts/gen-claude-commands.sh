#!/usr/bin/env bash
# Regenerate .claude/commands/gov/*.md from framework/commands/*.md.
#
# Substitutes {project} -> gov and {cli-config-dir} -> .claude.
# The setup command is sourced from framework/commands/setup/claude.md.
# init.md is governance-specific and hand-maintained — never touched.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/framework/commands"
DEST="$ROOT/.claude/commands/gov"

PROJECT="gov"
CONFIG_DIR=".claude"

mkdir -p "$DEST"

substitute() {
  sed -e "s/{project}/$PROJECT/g" -e "s|{cli-config-dir}|$CONFIG_DIR|g"
}

# Generate one command per source file in framework/commands/, excluding:
#   - govern.md (the installer keeps placeholders intact)
#   - setup/ subdirectory (handled separately below)
for src in "$SRC"/*.md; do
  name="$(basename "$src")"
  case "$name" in
    govern.md) continue ;;
  esac
  substitute < "$src" > "$DEST/$name"
done

# Setup is sourced from the agent-specific permission file and renamed to setup.md.
substitute < "$SRC/setup/claude.md" > "$DEST/setup.md"

echo "Regenerated $(ls "$DEST"/*.md | wc -l | tr -d ' ') files in $DEST/"
echo "(init.md is hand-maintained and was not touched)"
