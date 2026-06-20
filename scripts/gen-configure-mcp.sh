#!/usr/bin/env bash
# Regenerate the runtime-MCP-tool permission blocks in
# framework/bootstrap/configure/{claude,auggie,antigravity,opencode}.md from the
# canonical tool list in framework/runtime-tools.txt.
#
# Establishes the invariant: every tool listed in runtime-tools.txt has
# a permission entry in both agents' configure sources. Adding or
# removing a tool in runtime-tools.txt flows through to both files on
# the next commit via the pre-commit hook.
#
# Marker pair (both files):
#   <!-- generated:mcp-allow:start -->
#   <!-- generated:mcp-allow:end -->
#
# Per-host mapping (deterministic; no host-presence detection). The
# `gvrn` server-name prefix comes from the adopter's `.mcp.json`
# registration; tool names in this list are bare `<verb>-<noun>`.
#   <verb>-<noun>  →  Claude:  mcp__gvrn__<verb>-<noun>
#                  →  Auggie:  toolName "mcp:gvrn:<verb>-<noun>",
#                              permission { type: "allow" }
#                  →  Antigravity: a single `mcp(gvrn/*)` wildcard (covers
#                                  every tool; not per-tool enumerated)
#                  →  OpenCode: a single `"gvrn*": "allow"` glob (covers
#                                  every tool; not per-tool enumerated)
#
# Exits non-zero if either marker is missing in either source file.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="$ROOT/framework/runtime-tools.txt"
CLAUDE_SRC="$ROOT/framework/bootstrap/configure/claude.md"
AUGGIE_SRC="$ROOT/framework/bootstrap/configure/auggie.md"
ANTIGRAVITY_SRC="$ROOT/framework/bootstrap/configure/antigravity.md"
OPENCODE_SRC="$ROOT/framework/bootstrap/configure/opencode.md"

# Track every mktemp we create so early-exit paths (set -e, signals,
# splice failures) don't leak temp files into $TMPDIR.
cleanup_files=()
cleanup() { [ "${#cleanup_files[@]}" -gt 0 ] && rm -f "${cleanup_files[@]}"; }
trap cleanup EXIT

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
# in claude.md's allow-list). Each tool name from runtime-tools.txt is a
# bare `<verb>-<noun>`; the `gvrn` server-name prefix is added per host.
claude_block_file="$(mktemp)"; cleanup_files+=("$claude_block_file")
auggie_block_file="$(mktemp)"; cleanup_files+=("$auggie_block_file")
# Antigravity uses one `mcp(gvrn/*)` wildcard covering every gvrn tool, rather
# than the per-tool enumeration Claude/Auggie need — so its block is a constant
# single line, built outside the per-tool loop below.
antigravity_block_file="$(mktemp)"; cleanup_files+=("$antigravity_block_file")
printf '   - `mcp(gvrn/*)`\n' > "$antigravity_block_file"
# OpenCode likewise uses one `"gvrn*": "allow"` glob (no dedicated mcp permission
# key; MCP tools are matched by tool-name pattern), so its block is also a
# constant single line built outside the per-tool loop.
opencode_block_file="$(mktemp)"; cleanup_files+=("$opencode_block_file")
printf '   - `"gvrn*": "allow"`\n' > "$opencode_block_file"

tool_count=0
while IFS= read -r tool; do
  case "$tool" in
    ''|'#'*) continue ;;
  esac
  # Trim trailing whitespace.
  tool="${tool%"${tool##*[![:space:]]}"}"
  [ -z "$tool" ] && continue
  printf '   - `mcp__gvrn__%s`\n' "$tool" >> "$claude_block_file"
  printf '   - `{ "toolName": "mcp:gvrn:%s", "permission": { "type": "allow" } }`\n' "$tool" >> "$auggie_block_file"
  tool_count=$((tool_count + 1))
done < "$TOOLS"

if [ "$tool_count" -eq 0 ]; then
  echo "No tools found in $TOOLS" >&2
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
# `out` is mktemp'd and registered for trap cleanup; the happy path
# either `mv`s it onto $file (consuming the temp) or leaves it to the
# trap.
process() {
  local file="$1"
  local block_file="$2"
  local out
  out="$(mktemp)"
  cleanup_files+=("$out")
  if ! splice "$file" "$block_file" > "$out"; then
    return 5
  fi
  if cmp -s "$file" "$out"; then
    return 0
  fi
  if [ "$dry_run" -eq 1 ]; then
    echo "Would update $file"
    return 1
  fi
  mv "$out" "$file"
  echo "Updated $file"
  return 0
}

rc=0
process "$CLAUDE_SRC" "$claude_block_file" || rc=$?
process "$AUGGIE_SRC" "$auggie_block_file" || rc=$?
process "$ANTIGRAVITY_SRC" "$antigravity_block_file" || rc=$?
process "$OPENCODE_SRC" "$opencode_block_file" || rc=$?

if [ "$rc" -ne 0 ] && [ "$dry_run" -eq 1 ]; then
  exit 1
elif [ "$rc" -ne 0 ]; then
  exit "$rc"
fi

if [ "$dry_run" -eq 1 ]; then
  echo "No changes (mcp-allow blocks in sync)"
fi
exit 0
