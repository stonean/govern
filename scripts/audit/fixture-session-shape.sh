#!/usr/bin/env bash
# scripts/audit/fixture-session-shape.sh — Family 12 of /audit.
#
# Verifies every runtime/tests/fixtures/*/.govern.session.toml file:
#
#   12a parses cleanly as TOML.
#   12b does NOT use the legacy camelCase keys `scenarioPath` or
#       `setAt` — those were renamed to kebab-case in the 0.10.0
#       consolidation, and a fixture still using them would round-trip
#       silently broken (the runtime's reader is kebab-case-only).
#
# This is the test-data complement to Family 11 (consolidation-pair):
# Family 11 catches the live framework artifacts drifting; this one
# catches test fixtures drifting from the same shape.
#
# Requires `python3` (3.11+) for TOML parsing.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

drift=0

emit() {
  echo "fixture-session-shape | $1 | $2 | $3"
  drift=1
}

if ! command -v python3 >/dev/null 2>&1; then
  emit "(precondition)" "python3 not on PATH — cannot parse TOML" \
    "install python3 (3.11+) and re-run"
  exit 1
fi
if ! python3 -c "import tomllib" 2>/dev/null; then
  emit "(precondition)" "python3 lacks tomllib (need Python 3.11+)" \
    "upgrade python3 to 3.11+ and re-run"
  exit 1
fi

FIXTURES_DIR="runtime/tests/fixtures"
if [ ! -d "$FIXTURES_DIR" ]; then
  # No fixtures means nothing to check — exit clean.
  exit 0
fi

# Find every .govern.session.toml under the fixtures tree. Use a
# while-read loop instead of `mapfile` for portability — macOS ships
# bash 3.x and lacks `mapfile`.
SESSION_FILES=$(find "$FIXTURES_DIR" -name ".govern.session.toml" -print | sort)

if [ -z "$SESSION_FILES" ]; then
  # No fixtures use a session file — clean exit.
  exit 0
fi

while IFS= read -r f; do
  [ -n "$f" ] || continue
  # 12a parses-as-TOML.
  if ! python3 -c "import tomllib; tomllib.loads(open('$f').read())" 2>/dev/null; then
    err=$(python3 -c "import tomllib
try:
    tomllib.loads(open('$f').read())
except Exception as e:
    print(str(e))
" 2>&1)
    emit "$f" "fixture session TOML fails to parse: $err" \
      "fix the TOML syntax in $f"
    continue
  fi

  # 12b camelCase legacy keys MUST NOT appear at any nesting level.
  # The dashboard reader is kebab-case-only post-0.10.0; a fixture with
  # `scenarioPath` or `setAt` would round-trip as a missing field
  # silently. grep on the raw text catches the bug regardless of
  # whether the key is at top level or in a sub-table.
  for legacy in scenarioPath setAt; do
    if grep -qE "^\s*$legacy\s*=" "$f"; then
      emit "$f" \
        "fixture uses legacy camelCase key '$legacy'" \
        "rename to the kebab-case equivalent (scenario-path or set-at)"
    fi
  done
done <<< "$SESSION_FILES"

exit "$drift"
