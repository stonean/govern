#!/usr/bin/env bash
# Test surface for scripts/gen-configure-mcp.sh — the wildcard-block splice
# targets (Antigravity from spec 028, OpenCode from spec 032).
#
# Coverage:
#   A. `--dry-run` reports all four configure sources in sync (no drift)
#   B. antigravity.md's generated block is the single `mcp(gvrn/*)` wildcard
#   C. claude.md / auggie.md still carry their per-tool blocks (regression)
#   D. opencode.md's generated block is the single `"gvrn*": "allow"` glob
#
# Usage: scripts/tests/test-gen-configure-mcp.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
GEN="$REPO_ROOT/scripts/gen-configure-mcp.sh"
ANTIGRAVITY="$REPO_ROOT/framework/bootstrap/configure/antigravity.md"
CLAUDE="$REPO_ROOT/framework/bootstrap/configure/claude.md"
AUGGIE="$REPO_ROOT/framework/bootstrap/configure/auggie.md"
OPENCODE="$REPO_ROOT/framework/bootstrap/configure/opencode.md"

failures=0
pass() { printf '  PASS  %s\n' "$1"; }
fail() { printf '  FAIL  %s\n' "$1" >&2; failures=$((failures + 1)); }

# Extract the lines between the generated:mcp-allow markers of a file.
mcp_block() {
  awk '/generated:mcp-allow:start/{p=1;next} /generated:mcp-allow:end/{p=0} p' "$1"
}

echo "Running gen-configure-mcp tests..."

# A. all three sources in sync
if "$GEN" --dry-run >/dev/null 2>&1; then
  pass "A: gen-configure-mcp --dry-run is in sync"
else
  fail "A: --dry-run reports drift — run scripts/gen-configure-mcp.sh and commit"
fi

# B. antigravity block is exactly one mcp(gvrn/*) entry
ag_block="$(mcp_block "$ANTIGRAVITY")"
if printf '%s\n' "$ag_block" | grep -qF -- '- `mcp(gvrn/*)`' \
   && [ "$(printf '%s\n' "$ag_block" | grep -c '`')" -eq 1 ]; then
  pass "B: antigravity.md block is the single mcp(gvrn/*) wildcard"
else
  fail "B: antigravity.md block unexpected: $(printf '%s' "$ag_block" | tr '\n' '|')"
fi

# C. claude / auggie regression — per-tool blocks still present
if mcp_block "$CLAUDE" | grep -qF -- '- `mcp__gvrn__read-spec`'; then
  pass "C: claude.md per-tool block intact"
else
  fail "C: claude.md per-tool block missing or changed"
fi
if mcp_block "$AUGGIE" | grep -qF -- 'mcp:gvrn:read-spec'; then
  pass "C: auggie.md per-tool block intact"
else
  fail "C: auggie.md per-tool block missing or changed"
fi

# D. opencode block is exactly one `"gvrn*": "allow"` entry
oc_block="$(mcp_block "$OPENCODE")"
if printf '%s\n' "$oc_block" | grep -qF -- '- `"gvrn*": "allow"`'; then
  pass 'D: opencode.md block is the single "gvrn*": "allow" glob'
else
  fail "D: opencode.md block unexpected: $(printf '%s' "$oc_block" | tr '\n' '|')"
fi

if [ "$failures" -gt 0 ]; then
  echo "$failures test(s) failed" >&2
  exit 1
fi
echo "All gen-configure-mcp tests passed"
