#!/usr/bin/env bash
# Regenerate the Feature Specs table in README.md from spec frontmatter.
#
# Walks specs/NNN-*/spec.md and specs/NNN-*/spec-and-plan.md, extracts the
# status field (frontmatter), dependencies field (frontmatter), and the
# description (first non-blockquote paragraph of the body after the H1
# heading, truncated to its first sentence), and rewrites the table
# between the marker comments:
#
#   <!-- generated:feature-specs:start -->
#   ...generated table...
#   <!-- generated:feature-specs:end -->
#
# Exits non-zero if either marker is missing from README.md.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
README="$ROOT/README.md"

dry_run=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=1 ;;
    -h|--help)
      sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
      echo
      echo "Usage: $(basename "$0") [--dry-run]"
      echo "  --dry-run  Report what would change; exit 1 if README needs updating."
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if ! grep -q '<!-- generated:feature-specs:start -->' "$README"; then
  echo "Missing marker <!-- generated:feature-specs:start --> in $README" >&2
  exit 3
fi
if ! grep -q '<!-- generated:feature-specs:end -->' "$README"; then
  echo "Missing marker <!-- generated:feature-specs:end --> in $README" >&2
  exit 3
fi

shopt -s nullglob

# Enumerate feature-spec files scoped to the git index (tracked + staged), so
# the README table never lists an untracked `/specify` draft that is not part of
# the committed tree (spec 017 / tracked-specs-not-worktree). Falls back to a
# worktree glob outside a git repo. Mirrors gen-spec-deps.sh.
list_specs() {
  if git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1; then
    git -C "$ROOT" ls-files -- specs \
      | { grep -E '^specs/[0-9][0-9][0-9]-[^/]+/(spec|spec-and-plan)\.md$' || true; } \
      | while IFS= read -r rel; do printf '%s/%s\n' "$ROOT" "$rel"; done
  else
    local f
    for f in "$ROOT"/specs/[0-9][0-9][0-9]-*/spec.md "$ROOT"/specs/[0-9][0-9][0-9]-*/spec-and-plan.md; do
      [ -e "$f" ] && printf '%s\n' "$f"
    done
  fi
}

# Build the table body.
table="$(
  printf '| Spec | Status | Dependencies | Description |\n'
  printf '| --- | --- | --- | --- |\n'

  while IFS= read -r spec; do
    [ -f "$spec" ] || continue
    slug="$(basename "$(dirname "$spec")")"
    relpath="specs/$slug/$(basename "$spec")"

    # Extract status, dependencies, and the description (first non-blockquote
    # paragraph after the H1, truncated to its first sentence). Dependencies
    # are reduced to just the NNN prefix to match the compact README style.
    awk -v slug="$slug" -v relpath="$relpath" '
      BEGIN { fm_seen = 0; in_fm = 0; status = ""; deps = ""; in_body = 0; saw_h1 = 0; desc = ""; desc_done = 0 }
      /^---[[:space:]]*$/ {
        if (!fm_seen) { in_fm = 1; fm_seen = 1; next }
        if (in_fm)    { in_fm = 0; in_body = 1; next }
      }
      in_fm && /^status:[[:space:]]/   { status = $0; sub(/^status:[[:space:]]*/, "", status); next }
      in_fm && /^dependencies:[[:space:]]/ {
        deps = $0
        sub(/^dependencies:[[:space:]]*/, "", deps)
        sub(/^\[/, "", deps)
        sub(/\][[:space:]]*$/, "", deps)
        next
      }
      in_fm { next }
      in_body && !saw_h1 && /^# / { saw_h1 = 1; next }
      in_body && saw_h1 && !desc_done {
        if ($0 ~ /^[[:space:]]*$/) {
          if (desc != "") desc_done = 1
          next
        }
        if ($0 ~ /^#/)   { desc_done = 1; next }
        if ($0 ~ /^<!--/) { next }
        if ($0 ~ /^>/)   { next }   # skip blockquote signposts/notes
        if (desc == "") desc = $0
        else desc = desc " " $0
        next
      }
      END {
        # Reduce deps to NNN-only, comma-separated.
        if (deps == "" || deps == "[]") {
          deps_display = "none"
        } else {
          n = split(deps, parts, /[[:space:]]*,[[:space:]]*/)
          deps_display = ""
          for (i = 1; i <= n; i++) {
            slug_i = parts[i]
            sub(/-.*/, "", slug_i)
            deps_display = deps_display (i == 1 ? "" : ", ") slug_i
          }
        }
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", desc)
        # Truncate to first sentence: terminate at ". " or "! " or "? ".
        if (match(desc, /[\.!?][[:space:]]/)) {
          desc = substr(desc, 1, RSTART)
        }
        if (desc == "") desc = "(no description)"
        printf("| [%s](%s) | %s | %s | %s |\n", slug, relpath, status, deps_display, desc)
      }
    ' "$spec"
  done < <(list_specs | sort)
)"

# Splice the new table between the markers.
table_file="$(mktemp)"
printf '%s\n' "$table" > "$table_file"

tmp="$(mktemp)"
awk -v table_file="$table_file" '
  /<!-- generated:feature-specs:start -->/ {
    print
    print ""
    while ((getline line < table_file) > 0) print line
    close(table_file)
    print ""
    in_block = 1
    next
  }
  /<!-- generated:feature-specs:end -->/ {
    in_block = 0
    print
    next
  }
  !in_block { print }
' "$README" > "$tmp"

rm "$table_file"

if cmp -s "$README" "$tmp"; then
  rm "$tmp"
  echo "No changes (README in sync)"
  exit 0
fi

if [ "$dry_run" -eq 1 ]; then
  rm "$tmp"
  echo "Would update $README"
  exit 1
fi

mv "$tmp" "$README"
echo "Updated $README"
