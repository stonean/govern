#!/usr/bin/env bash
# Regenerate the runtime-MCP-tool permission blocks in
# framework/bootstrap/configure/claude.md and
# framework/bootstrap/configure/auggie.md from the canonical tool list
# in framework/runtime-tools.txt.
#
# Establishes the invariant: every `gov-rt:*` tool listed in
# runtime-tools.txt has a permission entry in both agents' configure
# sources. Adding or removing a tool in runtime-tools.txt flows through
# to both files on the next commit via the pre-commit hook.
#
# Marker pair (both files):
#   <!-- generated:mcp-allow:start -->
#   <!-- generated:mcp-allow:end -->
#
# Per-host mapping (deterministic; no host-presence detection):
#   gov-rt:<verb>-<noun>  →  Claude:  mcp__gov-rt__<verb>-<noun>
#                         →  Auggie:  toolName "mcp:gov-rt:<verb>-<noun>",
#                                     permission { type: "allow" }
#
# Exits non-zero if either marker is missing in either source file.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="$ROOT/framework/runtime-tools.txt"
CLAUDE_SRC="$ROOT/framework/bootstrap/configure/claude.md"
AUGGIE_SRC="$ROOT/framework/bootstrap/configure/auggie.md"

dry_run=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=1 ;;
    -h|--help)
      sed -n '2,21p' "$0" | sed 's/^# \{0,1\}//'
      echo
      echo "Usage: $(basename "$0") [--dry-run]"
      echo "  --dry-run  Report what would change; exit 1 if any source needs updating."
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if [ ! -f "$TOOLS" ]; then
  echo "Missing source: $TOOLS" >&2
  exit 3
fi

# Build the Claude block content (3-space indent matches sibling sub-sections
# in claude.md's allow-list). Stripping the gov-rt: prefix; Claude uses
# double-underscore separator.
claude_block_file="$(mktemp)"
auggie_block_file="$(mktemp)"

tool_count=0
while IFS= read -r tool; do
  case "$tool" in
    ''|'#'*) continue ;;
  esac
  # Trim trailing whitespace.
  tool="${tool%"${tool##*[![:space:]]}"}"
  [ -z "$tool" ] && continue
  short="${tool#gov-rt:}"
  printf '   - `mcp__gov-rt__%s`\n' "$short" >> "$claude_block_file"
  printf '   - `{ "toolName": "mcp:gov-rt:%s", "permission": { "type": "allow" } }`\n' "$short" >> "$auggie_block_file"
  tool_count=$((tool_count + 1))
done < "$TOOLS"

if [ "$tool_count" -eq 0 ]; then
  echo "No tools found in $TOOLS" >&2
  rm -f "$claude_block_file" "$auggie_block_file"
  exit 4
fi

# Splice a block file between markers in a target file. Fails when either
# marker is absent.
splice() {
  local file="$1"
  local block_file="$2"
  if ! grep -q '<!-- generated:mcp-allow:start -->' "$file"; then
    echo "Missing marker <!-- generated:mcp-allow:start --> in $file" >&2
    return 5
  fi
  if ! grep -q '<!-- generated:mcp-allow:end -->' "$file"; then
    echo "Missing marker <!-- generated:mcp-allow:end --> in $file" >&2
    return 5
  fi
  awk -v block_file="$block_file" '
    /<!-- generated:mcp-allow:start -->/ {
      print
      while ((getline line < block_file) > 0) print line
      close(block_file)
      in_block = 1
      next
    }
    /<!-- generated:mcp-allow:end -->/ {
      in_block = 0
      print
      next
    }
    !in_block { print }
  ' "$file"
}

# Process each file. Compare against the source; rewrite or report.
process() {
  local label="$1"
  local file="$2"
  local block_file="$3"
  local out
  out="$(mktemp)"
  if ! splice "$file" "$block_file" > "$out"; then
    rm -f "$out"
    return 5
  fi
  if cmp -s "$file" "$out"; then
    rm -f "$out"
    return 0
  fi
  if [ "$dry_run" -eq 1 ]; then
    rm -f "$out"
    echo "Would update $file"
    return 1
  fi
  mv "$out" "$file"
  echo "Updated $file"
  return 0
}

rc=0
process "claude" "$CLAUDE_SRC" "$claude_block_file" || rc=$?
process "auggie" "$AUGGIE_SRC" "$auggie_block_file" || rc=$?

rm -f "$claude_block_file" "$auggie_block_file"

if [ "$rc" -ne 0 ] && [ "$dry_run" -eq 1 ]; then
  exit 1
elif [ "$rc" -ne 0 ]; then
  exit "$rc"
fi

if [ "$dry_run" -eq 1 ]; then
  echo "No changes (mcp-allow blocks in sync)"
fi
exit 0
