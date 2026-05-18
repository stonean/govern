#!/usr/bin/env bash
# scripts/audit/primitive-promotion-candidates.sh — Family 9 of /audit.
#
# Scan framework/commands/*.md Instructions sections for numbered steps
# that have neither a backtick-quoted runtime-primitive name nor an
# `<!-- llm:* -->` extension-point marker. Each such "prose-only" step is
# a candidate for primitive promotion (deterministic logic that could
# become a `gvrn` primitive) or for an LLM-marker annotation (when the
# step requires semantic judgment but the marker is missing).
#
# Allowlist: a numbered step preceded by `<!-- audit:ignore-promotion -->`
# on the previous content line is exempt. Genuine host-responsibility prose
# (e.g., "render the dashboard", "aggregate findings") gets the annotation.
#
# Method:
#   1. Read framework/runtime-tools.txt to load the set of primitive names.
#   2. Walk each framework/commands/*.md file.
#   3. Find the ## Instructions section; iterate numbered steps within it.
#   4. For each step, check whether it contains a backticked primitive name
#      OR an `<!-- llm:* -->` marker. If neither AND no ignore-promotion
#      annotation on the previous content line, emit a finding.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

MANIFEST="framework/runtime-tools.txt"
LEGACY_ALLOWLIST="runtime/legacy-prose-commands.txt"

drift=0

emit() {
  echo "primitive-promotion | $1 | $2 | $3"
  drift=1
}

# Load primitive names (skip blank lines and # comments).
primitives=()
while IFS= read -r raw; do
  line="${raw%%#*}"
  line="${line#"${line%%[![:space:]]*}"}"
  line="${line%"${line##*[![:space:]]}"}"
  [ -z "$line" ] && continue
  primitives+=("$line")
done < "$MANIFEST"

# Load legacy-prose allowlist: command files in this list have not yet
# been rewritten to the parseable conventions. They're already known to
# be prose-only and out of /audit's scope — lint-procedure-parseability
# uses the same allowlist. Family 9 inherits it for symmetry.
legacy_files=()
if [ -f "$LEGACY_ALLOWLIST" ]; then
  while IFS= read -r raw; do
    line="${raw%%#*}"
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line%"${line##*[![:space:]]}"}"
    [ -z "$line" ] && continue
    legacy_files+=("$line")
  done < "$LEGACY_ALLOWLIST"
fi

is_legacy() {
  local target="$1"
  for entry in "${legacy_files[@]}"; do
    if [ "$entry" = "$target" ]; then
      return 0
    fi
  done
  return 1
}

# Walk each command file.
for file in framework/commands/*.md; do
  if is_legacy "$file"; then
    continue
  fi
  # State: are we inside ## Instructions? Did the previous content line
  # carry the audit:ignore-promotion marker?
  in_instructions=0
  ignore_next=0
  step_start_line=0
  step_buffer=""
  step_has_primitive=0
  step_has_llm_marker=0
  step_has_ignore=0
  line_no=0

  flush_step() {
    if [ "$step_start_line" -eq 0 ]; then
      return
    fi
    if [ "$step_has_ignore" -eq 1 ]; then
      :  # allowlisted
    elif [ "$step_has_primitive" -eq 0 ] && [ "$step_has_llm_marker" -eq 0 ]; then
      # First-line summary of the step for the finding (truncate to 120 chars).
      summary="$(printf '%s' "$step_buffer" | head -n 1 | cut -c 1-120)"
      emit "$file:$step_start_line" "prose-only step without primitive call or <!-- llm:* --> marker: $summary" "either invoke a runtime primitive, add an <!-- llm:* --> marker, or annotate with <!-- audit:ignore-promotion --> on the preceding line"
    fi
    step_start_line=0
    step_buffer=""
    step_has_primitive=0
    step_has_llm_marker=0
    step_has_ignore=0
  }

  while IFS= read -r line; do
    line_no=$((line_no + 1))
    # Detect section boundaries.
    if [[ "$line" =~ ^##[[:space:]]+Instructions[[:space:]]*$ ]]; then
      in_instructions=1
      continue
    fi
    # Any other H2 ends the Instructions section. Flush a pending step
    # before moving on.
    if [[ "$line" =~ ^##[[:space:]] ]] && [ "$in_instructions" -eq 1 ]; then
      flush_step
      in_instructions=0
      continue
    fi
    [ "$in_instructions" -eq 0 ] && continue

    # Track ignore marker for the next step.
    trimmed="${line#"${line%%[![:space:]]*}"}"
    if [ "$trimmed" = "<!-- audit:ignore-promotion -->" ]; then
      ignore_next=1
      continue
    fi

    # Detect a numbered-step line: "N. ..." at start of line (with or
    # without leading whitespace for sub-steps, but we only care about
    # top-level for promotion candidates).
    if [[ "$line" =~ ^[0-9]+\.[[:space:]] ]]; then
      flush_step
      step_start_line=$line_no
      step_buffer="$line"
      if [ "$ignore_next" -eq 1 ]; then
        step_has_ignore=1
        ignore_next=0
      fi
      # Check the opening line for primitive backticks or llm marker.
      for prim in "${primitives[@]}"; do
        if [[ "$line" == *"\`${prim}\`"* ]]; then
          step_has_primitive=1
          break
        fi
      done
      if [[ "$line" == *"<!-- llm:"* ]]; then
        step_has_llm_marker=1
      fi
      continue
    fi

    # Continuation lines of the current step (until next numbered step or
    # H2 section).
    if [ "$step_start_line" -gt 0 ]; then
      step_buffer+=$'\n'"$line"
      for prim in "${primitives[@]}"; do
        if [[ "$line" == *"\`${prim}\`"* ]]; then
          step_has_primitive=1
          break
        fi
      done
      if [[ "$line" == *"<!-- llm:"* ]]; then
        step_has_llm_marker=1
      fi
    fi
  done < "$file"

  # Flush any final pending step at EOF.
  flush_step
done

exit "$drift"
