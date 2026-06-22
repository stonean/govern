#!/usr/bin/env bash
# scripts/audit/cross-doc-consistency.sh — Family 1 of /audit.
#
# Two sub-checks for cross-doc claim consistency:
#
#   1b. Pipeline-diagram status sequence — extracts the ordered status names
#       (draft → clarified → planned → in-progress → done) from
#       framework/constitution.md §spec-lifecycle, docs/introduction.md, and
#       framework/templates/project/project-readme.md. Verifies all three
#       visit the same five states in the same order. Command-name and gate
#       annotation differences between docs are allowed (the canonical
#       audience differs per file — project-readme uses {project}: placeholders);
#       only the state sequence itself is invariant.
#
#   1c. Back-edge wording — extracts the back-edge transitions from
#       framework/constitution.md §spec-lifecycle (the canonical source) and
#       confirms each is referenced by framework/commands/amend.md (which owns
#       the back-edges per spec 014) and framework/commands/target.md's
#       Status→next-action table.
#
# Each sub-check emits findings to stdout in pipe-separated format. Script
# exits 0 when no findings, 1 when any sub-check produced findings.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

drift=0

emit() {
  echo "cross-doc | $1 | $2 | $3"
  drift=1
}

# -- 1b: Pipeline-diagram status sequence -----------------------------------

# Extract the ordered state sequence from a file's pipeline diagram. The
# diagram is identified as a line containing all five canonical states
# AND at least one arrow glyph (▶ or →) — that distinguishes the diagram
# itself from prose paragraphs that happen to list the same five states.
# Returns the states in order, one per line.
extract_diagram_states() {
  local file="$1"
  # Find a line with arrows + the canonical states.
  local line
  line="$(grep -m 1 -E '[▶→]' "$file" | grep -E 'draft.*clarified.*planned.*in-progress.*done' || true)"
  if [ -z "$line" ]; then
    # Fallback: any single line containing both an arrow glyph and all
    # five states. The two-stage grep above can miss when the diagram
    # uses ── only (no ▶). Re-attempt with a per-line loop.
    line="$(awk '/[▶→]/ && /draft/ && /clarified/ && /planned/ && /in-progress/ && /done/ {print; exit}' "$file")"
  fi
  if [ -z "$line" ]; then
    return 1
  fi
  # Extract just the five state words in the order they appear. Use grep
  # -oE for portability with macOS's BSD grep.
  printf '%s\n' "$line" \
    | grep -oE 'draft|clarified|planned|in-progress|done' \
    | awk '!seen[$0]++'
}

sub_1b() {
  # Three files to compare; constitution is canonical. macOS ships bash 3.2
  # which lacks associative arrays, so use parallel scalars.
  local constitution="framework/constitution.md"
  local introduction="docs/introduction.md"
  local project_readme="framework/templates/project/project-readme.md"

  local seq_constitution seq_introduction seq_project_readme
  local missing=0
  if ! seq_constitution="$(extract_diagram_states "$constitution")"; then
    emit "$constitution" "pipeline diagram not found (no line mentions all five states)" "add a pipeline diagram referencing draft → clarified → planned → in-progress → done"
    missing=1
  fi
  if ! seq_introduction="$(extract_diagram_states "$introduction")"; then
    emit "$introduction" "pipeline diagram not found (no line mentions all five states)" "add a pipeline diagram referencing draft → clarified → planned → in-progress → done"
    missing=1
  fi
  if ! seq_project_readme="$(extract_diagram_states "$project_readme")"; then
    emit "$project_readme" "pipeline diagram not found (no line mentions all five states)" "add a pipeline diagram referencing draft → clarified → planned → in-progress → done"
    missing=1
  fi
  if [ "$missing" -eq 1 ]; then
    return
  fi
  if [ "$seq_introduction" != "$seq_constitution" ]; then
    emit "$introduction" "pipeline diagram state sequence differs from framework/constitution.md" "reconcile the diagram to visit the same five states in the same order as the constitution"
  fi
  if [ "$seq_project_readme" != "$seq_constitution" ]; then
    emit "$project_readme" "pipeline diagram state sequence differs from framework/constitution.md" "reconcile the diagram to visit the same five states in the same order as the constitution"
  fi
}

# -- 1c: Back-edge wording ---------------------------------------------------

sub_1c() {
  # The two canonical back-edges per spec 014 and the constitution:
  #   * clarified/planned/in-progress → draft  (new question via /amend)
  #   * done → in-progress                      (new scenario via /amend)
  # Both should be referenced from framework/commands/amend.md (which owns
  # the back-edges) and framework/commands/target.md (which surfaces the
  # next-action implications).
  local consumers=("framework/commands/amend.md" "framework/commands/target.md")
  for c in "${consumers[@]}"; do
    if [ ! -f "$c" ]; then
      emit "$c" "expected back-edge reference missing — file does not exist" "create the file or update the family-1c consumer list"
      continue
    fi
    # Heuristic: each consumer should reference both back-edges. We accept
    # any of several phrasings.
    if ! grep -qE 'clarified.*planned.*in-progress.*draft|→ draft|reverts.*to.*draft' "$c"; then
      emit "$c" "no reference to the clarified/planned/in-progress → draft back-edge" "add a sentence referencing the back-edge per framework/constitution.md §spec-lifecycle"
    fi
    if ! grep -qE 'done.*in-progress|→ in-progress|reopens.*to.*in-progress|reverts.*to.*in-progress' "$c"; then
      emit "$c" "no reference to the done → in-progress back-edge" "add a sentence referencing the back-edge per framework/constitution.md §spec-lifecycle"
    fi
  done
}

sub_1b
sub_1c

exit "$drift"
