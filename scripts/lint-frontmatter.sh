#!/usr/bin/env bash
# Verify the frontmatter shape of every spec and scenario file.
#
# For each specs/**/spec.md, specs/**/spec-and-plan.md, and
# specs/**/scenarios/*.md file:
#   1. The file starts with a `---` delimited frontmatter block.
#   2. If a `status:` field is present, its value is one of:
#      draft, clarified, planned, in-progress, done.
#   3. If a `dependencies:` field is present, it parses as either an
#      inline bracketed list (`dependencies: [...]`) or a YAML block
#      list (`dependencies:` followed by `- item` lines).
#
# Shape-only — not a full YAML parser. /gov:validate's hard-fail tier
# is the rigorous check; this lint is a CI-side smoke test.
#
# Source of truth: framework/constitution.md §runtime-boundary
# Consumed by: .github/workflows/markdown-only-pipeline.yml

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

for arg in "$@"; do
  case "$arg" in
    -h|--help)
      sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'
      echo
      echo "Usage: $(basename "$0")"
      echo "  Exits 0 when every spec and scenario file passes shape checks."
      echo "  Exits 1 when any file fails (errors printed to stdout)."
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

shopt -s nullglob

errors=0
files=(
  "$ROOT"/specs/[0-9][0-9][0-9]-*/spec.md
  "$ROOT"/specs/[0-9][0-9][0-9]-*/spec-and-plan.md
  "$ROOT"/specs/[0-9][0-9][0-9]-*/scenarios/*.md
)

for f in "${files[@]}"; do
  [ -f "$f" ] || continue
  rel="${f#"$ROOT"/}"

  # Check 1: frontmatter delimited block at top.
  first_line="$(head -n 1 "$f")"
  if [ "$first_line" != "---" ]; then
    echo "$rel: missing opening --- frontmatter delimiter on line 1"
    errors=$((errors + 1))
    continue
  fi

  # Find closing --- (line number of second occurrence).
  close_line="$(awk 'NR>1 && /^---[[:space:]]*$/ { print NR; exit }' "$f")"
  if [ -z "$close_line" ]; then
    echo "$rel: missing closing --- frontmatter delimiter"
    errors=$((errors + 1))
    continue
  fi

  fm="$(sed -n "2,$((close_line - 1))p" "$f")"

  # Check 2: status field, if present, is in the enum.
  status_value="$(printf '%s\n' "$fm" | awk -F: '/^status:/ { sub(/^[[:space:]]+/, "", $2); sub(/[[:space:]]+$/, "", $2); print $2; exit }')"
  if [ -n "$status_value" ]; then
    case "$status_value" in
      draft|clarified|planned|in-progress|done) ;;
      *) echo "$rel: status '$status_value' is not one of draft|clarified|planned|in-progress|done"; errors=$((errors + 1)) ;;
    esac
  fi

  # Check 3: dependencies field, if present, is a list (inline or block).
  if printf '%s\n' "$fm" | grep -q '^dependencies:'; then
    deps_line="$(printf '%s\n' "$fm" | grep '^dependencies:' | head -n 1)"
    rest="${deps_line#dependencies:}"
    rest="${rest# }"
    if [ -z "$rest" ]; then
      # Block-list form — verify next non-blank line is `- item` or end of fm.
      block_ok=0
      after_deps="$(printf '%s\n' "$fm" | awk '/^dependencies:[[:space:]]*$/ { found = 1; next } found { print }')"
      first_after="$(printf '%s\n' "$after_deps" | awk 'NF { print; exit }')"
      if [ -z "$first_after" ] || [[ "$first_after" =~ ^[[:space:]]*-[[:space:]] ]]; then
        block_ok=1
      fi
      if [ "$block_ok" -ne 1 ]; then
        echo "$rel: dependencies has no inline list and no block list following"
        errors=$((errors + 1))
      fi
    elif [[ "$rest" =~ ^\[.*\]$ ]]; then
      : # inline list — OK
    else
      echo "$rel: dependencies value '$rest' is not a list (expected [..] or block-list form)"
      errors=$((errors + 1))
    fi
  fi
done

if [ "$errors" -gt 0 ]; then
  exit 1
fi
exit 0
