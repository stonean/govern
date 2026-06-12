#!/usr/bin/env bash
# scripts/audit/runtime-probe-parity.sh — Family 15 of /audit.
#
# Verifies the gvrn binary probe stays in parity between an agent's bootstrap
# permission *seed* (the settings_template blob in the §Agent Registry table of
# framework/bootstrap/govern.md) and its steady-state permission set
# (framework/bootstrap/configure/{key}.md).
#
# Spec 029 wired a `command -v gvrn`-equivalent detection probe into BOTH places
# per agent so a routine /govern run does not re-prompt for the State-B/State-C
# probe. Nothing else guards the pairing: a maintainer who adds or removes the
# probe in one place but not the other ships a silent gap that only surfaces when
# a run re-prompts. This family catches that asymmetry.
#
# Scope is the probe ONLY — not the whole seed. The seed and the configure set
# legitimately diverge: the seed grants bootstrap-only commands (tar, mktemp,
# git rev-parse, git ls-files, the Read(...govern-*...) temp globs) the
# steady-state configure files omit, and the configure files grant pipeline
# commands the seed omits. Neither is a subset of the other, so a whole-seed
# parity check would false-positive on the correct repo. The probe is the one
# permission 029 deliberately placed in both artifacts.
#
# Fixed-string presence assertion (no regex interpretation of the grammar),
# per agent, comparing presence in the registry seed against presence in the
# configure file:
#   - present in both     -> ok (parity holds)
#   - present in neither   -> ok (probe deliberately removed from both)
#   - present in one only  -> finding (the drift this family guards), either way
#
# Each agent's probe is written in that agent's native grammar:
#   claude       Bash(command -v *)
#   auggie       "^command -v "  (the launch-process shellInputRegex)
#   antigravity  command(which)
#
# Adding a fourth agent that wires the probe is one extra check_agent line below.
# macOS bash 3.2: no associative arrays, no mapfile.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

GOVERN="framework/bootstrap/govern.md"

drift=0
emit() {
  echo "runtime-probe-parity | $1 | $2 | $3"
  drift=1
}

if [ ! -f "$GOVERN" ]; then
  emit "$GOVERN" "agent registry source missing" "restore $GOVERN"
  exit 1
fi

# check_agent <key> <configure-file> <probe-literal>
#
# Seed presence is a whole-file fixed-string match against govern.md: each probe
# literal is a permission-grammar string (Bash(...), a ^-anchored regex, or
# command(...)) that occurs only in its agent's §Agent Registry settings_template
# cell and nowhere else in the file, so no section scoping is needed and seed-side
# detection mirrors the configure-side grep exactly.
check_agent() {
  key="$1"; configure="$2"; probe="$3"

  if [ ! -f "$configure" ]; then
    emit "$configure" "configure file for agent '$key' missing" "restore $configure"
    return
  fi

  seed_has=0
  if grep -qF -- "$probe" "$GOVERN"; then
    seed_has=1
  fi

  cfg_has=0
  if grep -qF -- "$probe" "$configure"; then
    cfg_has=1
  fi

  if [ "$seed_has" -eq 1 ] && [ "$cfg_has" -eq 0 ]; then
    emit "$configure (agent $key)" \
      "registry settings_template seeds the probe '$probe' but the configure file does not grant it" \
      "add '$probe' to the canonical permission set in $configure"
  elif [ "$seed_has" -eq 0 ] && [ "$cfg_has" -eq 1 ]; then
    emit "$GOVERN (agent $key)" \
      "configure file grants the probe '$probe' but the registry settings_template seed does not" \
      "add '$probe' to the '$key' settings_template in the §Agent Registry table of $GOVERN"
  fi
}

check_agent "claude"      "framework/bootstrap/configure/claude.md"      "Bash(command -v *)"
check_agent "auggie"      "framework/bootstrap/configure/auggie.md"      "^command -v "
check_agent "antigravity" "framework/bootstrap/configure/antigravity.md" "command(which)"

exit "$drift"
