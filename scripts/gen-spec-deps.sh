#!/usr/bin/env bash
# Regenerate frontmatter `dependencies:` for every feature spec from
# inline body links to sibling specs.
#
# Walks specs/NNN-*/spec.md and specs/NNN-*/spec-and-plan.md, finds inline
# markdown links matching ](../NNN-slug/...) or ](specs/NNN-slug/...) that
# are outside fenced code blocks, computes the union of unique sibling
# slugs (excluding self), and rewrites the YAML frontmatter `dependencies:`
# field as a sorted YAML list. If a spec body has no such links the field
# becomes `[]`.
#
# Body inline links are authoritative; the frontmatter is a derived index.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

dry_run=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=1 ;;
    -h|--help)
      sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
      echo
      echo "Usage: $(basename "$0") [--dry-run]"
      echo "  --dry-run  Report what would change; exit 1 if any spec needs updating."
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

shopt -s nullglob

changed=0
for spec in "$ROOT"/specs/[0-9][0-9][0-9]-*/spec.md "$ROOT"/specs/[0-9][0-9][0-9]-*/spec-and-plan.md; do
  [ -f "$spec" ] || continue
  own_slug="$(basename "$(dirname "$spec")")"

  # Extract sorted unique sibling slugs from body inline links (skipping code fences).
  deps_csv="$(awk -v own="$own_slug" '
    BEGIN { fm_seen = 0; in_fm = 0; in_fence = 0 }
    /^---[[:space:]]*$/ {
      if (!fm_seen) { in_fm = 1; fm_seen = 1; next }
      if (in_fm)    { in_fm = 0; next }
    }
    in_fm { next }
    /^[[:space:]]*```/ { in_fence = !in_fence; next }
    in_fence { next }
    {
      line = $0
      while (match(line, /\]\((\.\.\/|specs\/)[0-9][0-9][0-9]-[a-z0-9-]+/)) {
        m = substr(line, RSTART, RLENGTH)
        sub(/^\]\((\.\.\/|specs\/)/, "", m)
        if (m != own) slugs[m] = 1
        line = substr(line, RSTART + RLENGTH)
      }
    }
    END {
      n = 0
      for (s in slugs) arr[++n] = s
      # Insertion sort (n is small).
      for (i = 2; i <= n; i++) {
        key = arr[i]; j = i - 1
        while (j > 0 && arr[j] > key) { arr[j+1] = arr[j]; j-- }
        arr[j+1] = key
      }
      sep = ""
      for (i = 1; i <= n; i++) { printf("%s%s", sep, arr[i]); sep = "," }
    }
  ' "$spec")"

  if [ -z "$deps_csv" ]; then
    new_line="dependencies: []"
  else
    new_line="dependencies: [$(echo "$deps_csv" | sed 's/,/, /g')]"
  fi

  # Replace the first `dependencies:` line that appears inside the frontmatter.
  tmp="$(mktemp)"
  awk -v new="$new_line" '
    BEGIN { fm_seen = 0; in_fm = 0; replaced = 0 }
    /^---[[:space:]]*$/ {
      if (!fm_seen) { in_fm = 1; fm_seen = 1; print; next }
      if (in_fm)    { in_fm = 0; print; next }
    }
    in_fm && !replaced && /^dependencies:/ { print new; replaced = 1; next }
    { print }
  ' "$spec" > "$tmp"

  if ! cmp -s "$spec" "$tmp"; then
    if [ "$dry_run" -eq 1 ]; then
      echo "Would update $spec"
      rm "$tmp"
    else
      mv "$tmp" "$spec"
      echo "Updated $spec"
    fi
    changed=$((changed + 1))
  else
    rm "$tmp"
  fi
done

if [ "$changed" -eq 0 ]; then
  echo "No changes (all specs in sync)"
elif [ "$dry_run" -eq 1 ]; then
  exit 1
fi
