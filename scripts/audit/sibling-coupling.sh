#!/usr/bin/env bash
# scripts/audit/sibling-coupling.sh — Family 7 of /audit.
#
# Detect "bundling candidate" sibling specs — pairs of non-`done` specs
# that (a) inline-link each other in their body AND (b) share at least
# one row in their `## Affected Files` tables. The maintainer either
# folds the second-drafted spec into the first, or records a split-
# rationale entry that silences the finding.
#
# Suppression contract (per spec 026 Q5): grep the second-drafted spec's
# `## Resolved Questions` for the literal phrase
#   `Why split from {first-spec-slug}:`
# If present, the pair is suppressed.
#
# Identifying "second-drafted spec": the spec whose NNN prefix is higher.
# (The lower NNN was created first; the higher one is the candidate for
# folding or for recording the split rationale.)
#
# Background from spec 026: 024 (rule-file loader) and 025 (rule-file
# opt-out) shipped as two specs that should have been one — both touched
# framework/commands/review.md §Behavior step 5. This check would have
# surfaced the pair at 025's draft time.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

drift=0

emit() {
  echo "sibling-coupling | $1 | $2 | $3"
  drift=1
}

# Collect non-done spec directories.
non_done_specs=()
for spec_dir in specs/[0-9][0-9][0-9]-*/; do
  spec_file="$spec_dir/spec.md"
  [ -f "$spec_file" ] || continue
  # Extract status from frontmatter (assumes the standard YAML block).
  status="$(awk '
    /^---$/ { count++; if (count == 2) exit; next }
    count == 1 && /^status:/ {
      sub(/^status: *"?/, "")
      sub(/"?$/, "")
      print
      exit
    }
  ' "$spec_file")"
  if [ "$status" != "done" ]; then
    non_done_specs+=("$(basename "$spec_dir")")
  fi
done

# Nothing to compare when fewer than two non-done specs exist.
if [ "${#non_done_specs[@]}" -lt 2 ]; then
  exit 0
fi

# Helper: extract inline markdown links to sibling spec dirs from a body.
# Returns the set of referenced spec slugs (e.g., "024-rule-loader").
extract_sibling_links() {
  local file="$1"
  grep -oE '\(\.\./[0-9][0-9][0-9]-[a-z][a-z0-9-]*/[^)]*\)' "$file" \
    | sed -E 's|\(\.\./([^/)]+)/.*\)|\1|' \
    | sort -u
}

# Helper: extract Affected Files paths from a plan body.
# Looks for table rows whose first column is a backticked path.
extract_affected_files() {
  local plan="$1"
  [ -f "$plan" ] || return
  awk '
    # Detect Affected Files heading.
    /^## Affected Files/ { in_section = 1; next }
    /^## / { in_section = 0; next }
    in_section && /^\|/ {
      # Extract first column between | markers.
      match($0, /\| *`([^`]+)` *\|/, m)
      if (RLENGTH > 0) print m[1]
    }
  ' "$plan" | sort -u
}

# Pairwise check.
for i in "${!non_done_specs[@]}"; do
  for j in "${!non_done_specs[@]}"; do
    [ "$i" -ge "$j" ] && continue
    spec_a="${non_done_specs[$i]}"
    spec_b="${non_done_specs[$j]}"
    # Order: spec_a has the lower NNN (first-drafted), spec_b the higher.

    # (a) Bidirectional inline-link check.
    links_a="$(extract_sibling_links "specs/$spec_a/spec.md")"
    links_b="$(extract_sibling_links "specs/$spec_b/spec.md")"
    if ! grep -qFx "$spec_b" <<< "$links_a"; then continue; fi
    if ! grep -qFx "$spec_a" <<< "$links_b"; then continue; fi

    # (b) Affected-files overlap check.
    files_a="$(extract_affected_files "specs/$spec_a/plan.md")"
    files_b="$(extract_affected_files "specs/$spec_b/plan.md")"
    overlap="$(comm -12 <(printf '%s' "$files_a") <(printf '%s' "$files_b"))"
    [ -z "$overlap" ] && continue

    # Suppression check: look for `Why split from {spec_a}:` in spec_b's
    # Resolved Questions section.
    resolved="$(awk '
      /^## Resolved Questions/ { in_section = 1; next }
      /^## / { in_section = 0; next }
      in_section { print }
    ' "specs/$spec_b/spec.md")"
    # spec_a's slug-without-NNN-prefix is the lookup pattern.
    a_slug="${spec_a#[0-9][0-9][0-9]-}"
    if grep -qE "Why split from .*${a_slug}:" <<< "$resolved"; then
      continue
    fi

    # Unsuppressed bundling candidate — emit finding.
    overlap_summary="$(echo "$overlap" | tr '\n' ',' | sed 's/,$//')"
    emit "specs/$spec_b/spec.md" "bundling candidate with specs/$spec_a (overlapping Affected Files: $overlap_summary)" "either fold $spec_b into $spec_a (delete dir, merge ACs and open questions), or append a 'Why split from $a_slug: <reason>' entry to $spec_b/spec.md's ## Resolved Questions section to suppress"
  done
done

exit "$drift"
