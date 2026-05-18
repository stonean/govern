#!/usr/bin/env bash
# scripts/audit/introducing-drift.sh — Family 8 of /audit.
#
# Detect current-tense or imperative prose referencing renamed commands /
# files / identifiers in done spec bodies. After a rename sweep (per the
# 023 living-specs scenario), introducing-spec bodies retain references
# to old names as historical-action prose. A reader treating the spec
# body as current-state truth gets misled when verb tense and surrounding
# language implies the old name is still in active use.
#
# v1 implementation:
#
#   - Rename catalog is hardcoded below (RENAMED_TOKENS) rather than
#     derived from git log. Future scenario derives the catalog
#     automatically from commit messages.
#   - For each old-name token, grep done spec bodies for the token in
#     backticked code-span form (e.g., `/capture`).
#   - Each match emits a finding with file:line and a suggested past-
#     tense rewrite. False positives expected (heuristic doesn't
#     distinguish current-tense from past-tense surrounding prose);
#     maintainer dismisses per-spec via a small /gov:ask cycle that
#     adds a past-tense rewrite or accepts the prose as-is.
#
# Exemptions (two forms):
#   - Line scope: lines starting with `>` (blockquote) are skipped —
#     this is the canonical pattern for citing an old token in a
#     signpost without triggering the audit.
#   - File scope: a file containing `<!-- audit:ignore-introducing-drift:file -->`
#     anywhere is skipped wholesale. Use for the *introducing* spec of
#     a cataloged rename (e.g., 023 itself for /capture, /elaborate,
#     /validate, gov-rt:) where the old names are first-class subjects
#     of the prose, not residual drift.
#
# Background: spec 026 lists ~9 specs (011, 014, 017, 020, 021, 022, 023,
# 024, 000) that retain backticked old names after the 023 living-specs
# sweep. This check surfaces them so cleanup happens organically when
# authors touch those specs for other reasons.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# Old-name tokens to scan for. Each entry is `old-name|new-name|fix-hint`
# (pipe-separated). To add a rename here: include the slash-prefixed form
# for commands, the full filename for files, or the bare identifier for
# other tokens.
RENAMED_TOKENS=(
  "/capture|/specify|consolidated into /specify (spec 023)"
  "/elaborate|/ask|consolidated into /ask (spec 023)"
  "/validate|/analyze|renamed to /analyze (spec 023)"
  "/gov:validate|/gov:analyze|renamed to /analyze (spec 023)"
  "/gov:capture|/gov:specify|consolidated into /specify (spec 023)"
  "/gov:elaborate|/gov:ask|consolidated into /ask (spec 023)"
  "gov-rt:|gvrn:|MCP server name changed (spec 022 task 28)"
)

drift=0

emit() {
  echo "introducing-drift | $1 | $2 | $3"
  drift=1
}

# Iterate done spec bodies. A "done" spec has `status: done` in its
# frontmatter.
for spec_file in specs/[0-9][0-9][0-9]-*/spec.md; do
  status="$(awk '
    /^---$/ { count++; if (count == 2) exit; next }
    count == 1 && /^status:/ {
      sub(/^status: *"?/, "")
      sub(/"?$/, "")
      print
      exit
    }
  ' "$spec_file")"
  [ "$status" != "done" ] && continue
  # File-level skip: bail before walking lines if the marker is present.
  if grep -q '<!-- audit:ignore-introducing-drift:file -->' "$spec_file"; then
    continue
  fi
  # For each renamed token, grep the spec body for the backticked form.
  for entry in "${RENAMED_TOKENS[@]}"; do
    old="${entry%%|*}"
    rest="${entry#*|}"
    new="${rest%%|*}"
    hint="${rest#*|}"
    # Find each line containing `<old-name>` in code-span form.
    while IFS=: read -r line_no line_content; do
      [ -z "$line_no" ] && continue
      # Skip signpost / blockquote lines that intentionally name the old token.
      if [[ "$line_content" =~ ^[[:space:]]*\> ]]; then
        continue
      fi
      emit "$spec_file:$line_no" "references old name \`$old\` ($hint)" "rewrite to past tense or replace with \`$new\`; see scenarios/living-specs.md pattern (small /gov:ask cycle)"
    done < <(grep -nF "\`$old\`" "$spec_file" 2>/dev/null || true)
  done
done

exit "$drift"
