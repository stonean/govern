#!/usr/bin/env bash
# Regenerate .claude/commands/gov/*.md from framework/commands/*.md and
# framework/bootstrap/configure/claude.md.
#
# Substitutes {project} -> gov and {cli-config-dir} -> .claude.
# The configure command is sourced from framework/bootstrap/configure/claude.md.
# init.md is governance-specific and hand-maintained — never touched.
# Files in .claude/commands/gov/ that do not correspond to a current source
# (and are not init.md) are removed so renames flow through cleanly.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/framework/commands"
CONFIGURE_SRC="$ROOT/framework/bootstrap/configure/claude.md"
DEST="$ROOT/.claude/commands/gov"

PROJECT="gov"
CONFIG_DIR=".claude"

mkdir -p "$DEST"

substitute() {
  sed -e "s/{project}/$PROJECT/g" -e "s|{cli-config-dir}|$CONFIG_DIR|g"
}

# Track expected destination filenames so we can prune obsolete generated files.
expected=()

# Generate one command per source file in framework/commands/.
for src in "$SRC"/*.md; do
  name="$(basename "$src")"
  substitute < "$src" > "$DEST/$name"
  expected+=("$name")
done

# Configure is sourced from the agent-specific permission file, named configure.md.
substitute < "$CONFIGURE_SRC" > "$DEST/configure.md"
expected+=("configure.md")

# init.md is hand-maintained — preserve it.
expected+=("init.md")

# Prune any .md files in DEST that are no longer in the expected set.
for existing in "$DEST"/*.md; do
  name="$(basename "$existing")"
  keep=0
  for e in "${expected[@]}"; do
    if [ "$name" = "$e" ]; then keep=1; break; fi
  done
  if [ "$keep" -eq 0 ]; then
    rm -- "$existing"
    echo "Removed obsolete: $name"
  fi
done

echo "Regenerated $(ls "$DEST"/*.md | wc -l | tr -d ' ') files in $DEST/"
echo "(init.md is hand-maintained and was not touched)"
