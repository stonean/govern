#!/usr/bin/env bash
# Regenerate .claude/commands/gov/*.md from framework/commands/*.md and
# framework/bootstrap/configure/claude.md.
#
# Substitutes {project} -> gov and {cli-config-dir} -> .claude.
# The configure command is sourced from framework/bootstrap/configure/claude.md.
# init.md is governance-specific and hand-maintained — never touched.
# Files in .claude/commands/gov/ that do not correspond to a current source
# (and are not init.md) are removed so renames flow through cleanly.
#
# Flags:
#   --check    Compare generated content against the current destination;
#              exit 0 when in sync, 1 when drift exists (prints a unified
#              diff to stdout). Used by `/audit`'s check-zero precondition
#              pass to surface generator drift without writing.

set -euo pipefail

check_mode=0
for arg in "$@"; do
  case "$arg" in
    --check) check_mode=1 ;;
    -h|--help)
      sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/framework/commands"
CONFIGURE_SRC="$ROOT/framework/bootstrap/configure/claude.md"
DEST="$ROOT/.claude/commands/gov"

PROJECT="gov"
CONFIG_DIR=".claude"

substitute() {
  sed -e "s/{project}/$PROJECT/g" -e "s|{cli-config-dir}|$CONFIG_DIR|g"
}

# Track expected destination filenames so we can prune obsolete generated files.
expected=()

if [ "$check_mode" -eq 1 ]; then
  # --check mode: generate into a tempdir, diff against DEST, report drift
  # without modifying anything on disk.
  tmpdir="$(mktemp -d -t gen-claude-commands-check-XXXXXX)"
  trap 'rm -rf "$tmpdir"' EXIT
  for src in "$SRC"/*.md; do
    name="$(basename "$src")"
    substitute < "$src" > "$tmpdir/$name"
    expected+=("$name")
  done
  substitute < "$CONFIGURE_SRC" > "$tmpdir/configure.md"
  expected+=("configure.md")
  expected+=("init.md")

  drift=0
  # Compare every expected file against DEST.
  for name in "${expected[@]}"; do
    src_path="$tmpdir/$name"
    dest_path="$DEST/$name"
    if [ "$name" = "init.md" ]; then
      # Hand-maintained — never compared.
      continue
    fi
    if [ ! -f "$dest_path" ]; then
      echo "missing in DEST: $name"
      drift=1
      continue
    fi
    if ! diff -q "$src_path" "$dest_path" >/dev/null 2>&1; then
      echo "drift in $name:"
      diff -u "$dest_path" "$src_path" || true
      drift=1
    fi
  done
  # Detect orphans — files in DEST that no longer have a source.
  for existing in "$DEST"/*.md; do
    name="$(basename "$existing")"
    keep=0
    for e in "${expected[@]}"; do
      if [ "$name" = "$e" ]; then keep=1; break; fi
    done
    if [ "$keep" -eq 0 ]; then
      echo "orphan in DEST (no source): $name"
      drift=1
    fi
  done
  exit "$drift"
fi

# Write mode (default): regenerate destination from source.
mkdir -p "$DEST"

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
