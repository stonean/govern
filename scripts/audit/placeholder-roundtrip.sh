#!/usr/bin/env bash
# scripts/audit/placeholder-roundtrip.sh — Family 4 of /audit.
#
# Scan framework/commands/*.md for hardcoded tokens that should be
# placeholders per spec 012's multi-agent contract. Three patterns:
#
#   .claude/    should be {cli-config-dir}/
#   /gov:       should be /{project}:
#   `gov:`      should be `{project}:` (backticked command-prefix mentions)
#
# Allowlist (two forms):
#   1. Line scope: a line preceded by `<!-- audit:ignore-placeholders -->`
#      on the previous (non-blank) line is skipped.
#   2. File scope: a file containing `<!-- audit:ignore-placeholders:file -->`
#      anywhere is skipped wholesale. Use for maintainer-only command files
#      that are not scaffolded into adopters (e.g., framework/commands/audit.md).
#      The line marker doesn't compose with markdown tables — inserting a
#      comment between table rows breaks the table — so file scope is the
#      right tool when /gov: references span a table or are pervasive.
#
# False-positive shape: prose that mentions `.claude/` for documentary
# purposes (e.g., "Auggie reads `CLAUDE.md` natively — no `.claude/` dir
# is created"). When this surfaces, the right fix is the ignore-comment;
# do NOT widen the regex to match audience-specific prose.
#
# Expected: this check fires on /gov: throughout the framework's command
# sources today — the framework files are pre-substituted for the gov
# project rather than properly templated with /{project}: placeholders.
# This is a known framework-level drift /audit is designed to surface.
# The maintainer's next move after /audit ships is either to (a) fix the
# templating across all command files, or (b) declare /gov: as the
# canonical literal and adjust the audit. Both are valid decisions made
# AFTER /audit exists to surface the gap. v1 of /audit emits the findings;
# resolution is a follow-on framework refactor.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

drift=0

emit() {
  echo "placeholder-roundtrip | $1 | $2 | $3"
  drift=1
}

# Walk every framework/commands/*.md file, looking for the three patterns.
# Per-file: skip lines preceded by the ignore comment.
for file in framework/commands/*.md; do
  # File-level skip: bail before walking lines if the marker is present.
  if grep -q '<!-- audit:ignore-placeholders:file -->' "$file"; then
    continue
  fi
  prev_was_ignore=0
  line_no=0
  while IFS= read -r line; do
    line_no=$((line_no + 1))
    trimmed="${line#"${line%%[![:space:]]*}"}"  # left-trim
    # Track ignore marker on the previous content line.
    if [ "$trimmed" = "<!-- audit:ignore-placeholders -->" ]; then
      prev_was_ignore=1
      continue
    fi
    # Blank lines do not reset the ignore state — the marker still applies
    # to the next non-blank content line. (Common case: the marker sits
    # one blank line above the content.)
    if [ -z "$trimmed" ]; then
      continue
    fi
    if [ "$prev_was_ignore" -eq 1 ]; then
      prev_was_ignore=0
      continue
    fi
    # Pattern checks. Each token is an inline literal that should have
    # been a placeholder.
    if [[ "$line" == *".claude/"* ]]; then
      emit "$file:$line_no" "hardcoded \`.claude/\` should be \`{cli-config-dir}/\`" "replace with the placeholder, or add <!-- audit:ignore-placeholders --> if literal is intentional"
    fi
    if [[ "$line" == *"/gov:"* ]]; then
      emit "$file:$line_no" "hardcoded \`/gov:\` should be \`/{project}:\`" "replace with the placeholder, or add <!-- audit:ignore-placeholders --> if literal is intentional"
    fi
    # Match backticked `gov:` mentions (e.g., \`gov:specify\`). The earlier
    # check catches `/gov:` with a leading slash; this one catches the
    # bare prefix that appears in code spans.
    if [[ "$line" == *'`gov:'* ]]; then
      emit "$file:$line_no" "hardcoded \`gov:\` should be \`{project}:\`" "replace with the placeholder, or add <!-- audit:ignore-placeholders --> if literal is intentional"
    fi
  done < "$file"
done

exit "$drift"
