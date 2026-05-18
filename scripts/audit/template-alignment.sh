#!/usr/bin/env bash
# scripts/audit/template-alignment.sh — Family 5 of /audit.
#
# Verify the spec/plan/tasks/scenario templates under framework/templates/spec/
# scaffold the fields that framework/commands/analyze.md's blocking checks
# require. v1 covers the canonical invariants explicitly; future extensions
# parse analyze.md to derive the required-field list automatically.
#
# v1 checks (hardcoded canonical invariants):
#
#   spec.md template must scaffold:
#     - frontmatter `status:` field
#     - frontmatter `dependencies:` field
#     - `## Acceptance Criteria` section
#     - `## Open Questions` section
#
#   scenario.md template must scaffold:
#     - frontmatter `section:` field (post-017 schema)
#       (or `spec-ref:` for the legacy pre-017 schema; either satisfies)
#     - `## Context` section
#     - `## Behavior` section
#
#   plan.md template must scaffold:
#     - `## Technical Decisions` section
#     - `## Affected Files` section
#
#   tasks.md template must scaffold:
#     - At least one example numbered task (## N. ... or ### N. ...)
#
# A future scenario will parse analyze.md's blocking-check sections and
# derive the field list dynamically — for v1 the hardcoded list keeps the
# check actionable while the canonical fields are stable.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

TEMPLATE_DIR="framework/templates/spec"
drift=0

emit() {
  echo "template-alignment | $1 | $2 | $3"
  drift=1
}

check_file_exists() {
  local file="$1"
  if [ ! -f "$file" ]; then
    emit "$file" "template file missing" "create $file per spec 022's template scaffolding"
    return 1
  fi
  return 0
}

check_contains() {
  local file="$1" pattern="$2" description="$3" fix="$4"
  if [ -f "$file" ] && ! grep -qE "$pattern" "$file"; then
    emit "$file" "missing $description" "$fix"
  fi
}

# spec.md template
if check_file_exists "$TEMPLATE_DIR/spec.md"; then
  check_contains "$TEMPLATE_DIR/spec.md" "^status:" "frontmatter status: field" "add 'status:' to the template's YAML frontmatter block"
  check_contains "$TEMPLATE_DIR/spec.md" "^dependencies:" "frontmatter dependencies: field" "add 'dependencies:' to the template's YAML frontmatter block"
  check_contains "$TEMPLATE_DIR/spec.md" "^## Acceptance Criteria" "## Acceptance Criteria section" "add the section to the template"
  check_contains "$TEMPLATE_DIR/spec.md" "^## Open Questions" "## Open Questions section" "add the section to the template"
fi

# scenario.md template
if check_file_exists "$TEMPLATE_DIR/scenario.md"; then
  if ! grep -qE "^section:|^spec-ref:" "$TEMPLATE_DIR/scenario.md"; then
    emit "$TEMPLATE_DIR/scenario.md" "missing frontmatter section: (or legacy spec-ref:) field" "add 'section:' to the template's YAML frontmatter block"
  fi
  check_contains "$TEMPLATE_DIR/scenario.md" "^## Context" "## Context section" "add the section to the template"
  check_contains "$TEMPLATE_DIR/scenario.md" "^## Behavior" "## Behavior section" "add the section to the template"
fi

# plan.md template
if check_file_exists "$TEMPLATE_DIR/plan.md"; then
  check_contains "$TEMPLATE_DIR/plan.md" "^## Technical Decisions" "## Technical Decisions section" "add the section to the template"
  check_contains "$TEMPLATE_DIR/plan.md" "^## Affected Files" "## Affected Files section" "add the section to the template"
fi

# tasks.md template
if check_file_exists "$TEMPLATE_DIR/tasks.md"; then
  if ! grep -qE "^(##|###) [0-9]+\." "$TEMPLATE_DIR/tasks.md"; then
    emit "$TEMPLATE_DIR/tasks.md" "no example numbered task" "add at least one ## N. (flat) or ### N. (phased) example task"
  fi
fi

exit "$drift"
