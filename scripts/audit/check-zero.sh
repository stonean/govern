#!/usr/bin/env bash
# scripts/audit/check-zero.sh — `/audit`'s precondition pass.
#
# Invokes every generator (in --dry-run) and every lint script the
# framework ships. Any non-zero exit produces a `check-zero` finding to
# stdout pointing at the failing script. When this script exits non-zero,
# /audit halts before running the eight family checks — running them
# against known-stale generator output produces misleading findings (per
# spec 026's bootstrap-order resolution).
#
# Order matters: gen-spec-deps.sh runs first so downstream checks see
# fresh `dependencies:` frontmatter if anything was out of sync.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# Each entry is `script arg1 arg2 ...`. Run in order. Generators use the
# flag they support (`--dry-run` for the older generators; `--check` was
# added to `gen-claude-commands.sh` by spec 026 task 2 to close the gap).
# Lints already run in read-only mode by design — no flag.
checks=(
  "scripts/gen-spec-deps.sh --dry-run"
  "scripts/gen-readme-table.sh --dry-run"
  "scripts/gen-help-tables.sh --dry-run"
  "scripts/gen-configure-mcp.sh --dry-run"
  "scripts/gen-claude-commands.sh --check"
  "scripts/lint-rule-filenames.sh"
  "scripts/lint-frontmatter.sh"
  "scripts/lint-procedure-parseability.sh"
  "scripts/lint-tool-coverage.sh"
)

drift=0
for entry in "${checks[@]}"; do
  # Capture stdout+stderr; only print on failure to keep clean runs quiet.
  output="$(eval "$entry" 2>&1)" && status=0 || status=$?
  if [ "$status" -ne 0 ]; then
    drift=1
    script="${entry%% *}"
    # One pipe-separated finding line, plus the captured output indented
    # for readability. The aggregator surfaces the finding line; humans
    # read the indented output to diagnose.
    echo "check-zero | $script | precondition failed (exit $status) | re-run the script, fix what it reports, commit, and re-invoke /audit"
    while IFS= read -r line; do
      echo "             $line"
    done <<< "$output"
  fi
done

exit "$drift"
