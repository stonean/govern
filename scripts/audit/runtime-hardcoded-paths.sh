#!/usr/bin/env bash
# scripts/audit/runtime-hardcoded-paths.sh — Family 13 of /audit.
#
# Catches regressions to hardcoded host-/project-specific paths in the
# runtime source. Spec 022 scenario `commands-dir-parameterization`
# replaced the literal `.claude/commands/gov/` (which baked in Claude
# Code's config-dir name AND this repo's slash-command namespace) with
# a `{cli-config-dir}/commands/{project}/` lookup driven by the
# `Host::load` config loader.
#
# Any new occurrence of `.claude/commands/gov/` in `runtime/src/` means
# someone added a fresh hardcode — quietly breaking every Auggie /
# Anvil / future-host adopter that doesn't match the defaults. This
# audit is the safety net.
#
# Scoped to `runtime/src/**` only. Specs, scenarios, and migration
# bodies may reference the prior path in prose; tests and fixtures
# may reference it as regression context. The runtime source is the
# one place the literal must not reappear.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

drift=0

emit() {
  echo "runtime-hardcoded-paths | $1 | $2 | $3"
  drift=1
}

# 13a — Scan runtime/src for any literal `.claude/commands/gov/`.
# This is the exact string the parameterization scenario eliminated.
# `git grep` keeps the search index-aware and ignores generated/build
# artifacts under runtime/target/.
matches="$(git grep -n -F '.claude/commands/gov/' -- 'runtime/src/' 2>/dev/null || true)"
if [ -n "$matches" ]; then
  while IFS= read -r line; do
    emit "$line" \
      "hardcoded host/project path in runtime source" \
      "replace with the parameterized form built from Host::load (see runtime/src/host.rs)"
  done <<< "$matches"
fi

exit "$drift"
