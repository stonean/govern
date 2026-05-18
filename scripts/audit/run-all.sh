#!/usr/bin/env bash
# scripts/audit/run-all.sh — `/audit` aggregator.
#
# Runs the check-zero precondition pass followed by the eight family
# check scripts. Aggregates findings to stdout under per-family headers
# and exits 1 when any family (or check-zero) produced findings.
#
# This script IS the implementation of `/audit`. The framework/commands/
# audit.md slash-command file is documentation that invokes this
# orchestrator via the runtime's `run-generator` primitive (single call,
# no per-step args needed — sidesteps the runtime parser's lack of
# per-step argument binding for procedural commands).

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

drift=0

run_check() {
  local label="$1" script="$2"
  if [ ! -x "$script" ]; then
    echo "$label | $script | check script missing or not executable | chmod +x $script"
    drift=1
    return
  fi
  local output
  if output="$("$script" 2>&1)"; then
    # Exit 0 = no findings; emit nothing under the header.
    :
  else
    echo "=== $label ==="
    echo "$output"
    echo
    drift=1
  fi
}

run_check "check-zero (precondition)" "scripts/audit/check-zero.sh"
if [ "$drift" -eq 1 ]; then
  echo "(family checks skipped — check-zero failed; resolve the precondition findings and re-run /audit)"
  exit 1
fi

run_check "Family 1 — cross-doc claim consistency" "scripts/audit/cross-doc-consistency.sh"
run_check "Family 2 — manifest parity" "scripts/audit/manifest-parity.sh"
run_check "Family 3 — registry equivalence" "scripts/audit/registry-equivalence.sh"
run_check "Family 4 — placeholder roundtrip" "scripts/audit/placeholder-roundtrip.sh"
run_check "Family 5 — template alignment" "scripts/audit/template-alignment.sh"
run_check "Family 6 — SSOT invariants" "scripts/audit/ssot-invariants.sh"
run_check "Family 7 — sibling-spec coupling" "scripts/audit/sibling-coupling.sh"
run_check "Family 8 — introducing-spec body drift" "scripts/audit/introducing-drift.sh"

exit "$drift"
