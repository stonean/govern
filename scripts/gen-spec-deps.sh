#!/usr/bin/env bash
# Regenerate frontmatter `dependencies:` for every feature spec from
# inline body links to sibling specs.
#
# Walks specs/NNN-*/spec.md and specs/NNN-*/spec-and-plan.md, finds inline
# markdown links matching ](../NNN-slug/...) or ](specs/NNN-slug/...) that
# are outside fenced code blocks, outside blockquote-prefixed lines
# (signposts on done specs use blockquotes; their forward-pointer links
# are not implement-time dependencies), and outside any `## See also`
# section (informational/navigational pointers — author opt-out for links
# that should not induce a dependency edge; the opt-out ends at the next
# heading at level 2 or shallower; `## References` is NOT an opt-out and
# continues to produce edges), computes the union of unique sibling slugs,
# and rewrites the YAML frontmatter `dependencies:` field as a sorted YAML
# list. If a spec body has no such links the field becomes `[]`. Self-links
# are recorded (not stripped) so the cycle check below surfaces them.
#
# After the rewrite, runs an SCC-based cycle check across the derived dep
# graph. Any cycle — including self-cycles — is reported on stderr as
# `cycle: a -> b -> ... -> a` and the script exits non-zero. The pre-commit
# hooks propagate the failure to block the commit.
#
# Body inline links are authoritative; the frontmatter is a derived index.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

dry_run=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=1 ;;
    --root=*)  ROOT="$(cd "${arg#--root=}" && pwd)" ;;
    -h|--help)
      sed -n '2,23p' "$0" | sed 's/^# \{0,1\}//'
      echo
      echo "Usage: $(basename "$0") [--dry-run] [--root=PATH]"
      echo "  --dry-run    Report what would change; exit 1 if any spec needs updating."
      echo "  --root=PATH  Run against PATH as the repo root (default: script's parent dir)."
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

shopt -s nullglob

# Enumerate feature-spec files to process, scoped to the git index (tracked +
# staged) rather than a worktree glob. Untracked, in-progress drafts — e.g. a
# `/specify` spec the author has not `git add`ed yet — are intentionally
# excluded so they are never rewritten, never enter the dependency/cycle graph,
# and never block an unrelated commit (spec 017 / tracked-specs-not-worktree).
# Falls back to a worktree glob only outside a git repo, where there is no index.
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

graph_file="$(mktemp)"
trap 'rm -f "$graph_file"' EXIT

changed=0
while IFS= read -r spec; do
  [ -f "$spec" ] || continue
  own_slug="$(basename "$(dirname "$spec")")"

  # Extract sorted unique sibling slugs from body inline links (skipping code
  # fences, blockquote-prefixed lines, and `## See also` opt-out sections).
  deps_csv="$(awk -v own="$own_slug" '
    BEGIN { fm_seen = 0; in_fm = 0; in_fence = 0; in_see_also = 0 }
    /^---[[:space:]]*$/ {
      if (!fm_seen) { in_fm = 1; fm_seen = 1; next }
      if (in_fm)    { in_fm = 0; next }
    }
    in_fm { next }
    /^[[:space:]]*```/ { in_fence = !in_fence; next }
    in_fence { next }
    /^[[:space:]]*>/ { next }
    # `## See also` opt-out: a level-1 or level-2 heading toggles the region.
    # Matching heading text (case-insensitive) is exactly "See also".
    # Any other top-level heading ends the opt-out; deeper subheadings inherit
    # it. `## References` is intentionally NOT an opt-out (task 29 uses it as
    # the formal body-authored dependency section).
    /^#+[[:space:]]/ {
      hashes = 0
      for (i = 1; i <= length($0); i++) {
        if (substr($0, i, 1) == "#") hashes++
        else break
      }
      if (hashes <= 2) {
        htxt = substr($0, hashes + 1)
        sub(/^[[:space:]]+/, "", htxt)
        sub(/[[:space:]]+$/, "", htxt)
        ltxt = tolower(htxt)
        if (ltxt == "see also") in_see_also = 1
        else in_see_also = 0
      }
    }
    in_see_also { next }
    {
      line = $0
      while (match(line, /\]\((\.\.\/|specs\/)[0-9][0-9][0-9]-[a-z0-9-]+/)) {
        m = substr(line, RSTART, RLENGTH)
        sub(/^\]\((\.\.\/|specs\/)/, "", m)
        # Self-references are recorded so the downstream cycle check can
        # surface them as 1-cycles (per spec 017 / detect-dependency-cycles —
        # the generator does not silently strip self-references).
        slugs[m] = 1
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

  # Record this spec's outgoing edges (post-rewrite) for the cycle check.
  echo "$own_slug|$deps_csv" >> "$graph_file"
done < <(list_specs)

if [ "$changed" -eq 0 ]; then
  echo "No changes (all specs in sync)"
fi

# Cycle check: runs after the frontmatter rewrite so any diff is visible in the
# working tree even when the run fails (per spec 017 / detect-dependency-cycles).
# Tarjan's SCC algorithm over the derived graph; any SCC of size > 1 or any
# self-loop is reported on stderr as `cycle: a -> b -> ... -> a`. Exits 1 on
# cycle detection — the pre-commit hooks propagate the failure and block the
# commit.
cycle_rc=0
awk -F '|' '
  {
    slug = $1
    slugs[slug] = 1
    if ($2 != "") {
      n = split($2, parts, ",")
      for (i = 1; i <= n; i++) {
        d = parts[i]
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", d)
        if (d != "") {
          degree[slug]++
          adj[slug, degree[slug]] = d
        }
      }
    }
  }
  function strongconnect(v,    k, w) {
    vindex[v] = idx
    vlowlink[v] = idx
    idx++
    stack[stacksize++] = v
    onstack[v] = 1
    for (k = 1; k <= degree[v] + 0; k++) {
      w = adj[v, k]
      if (!(w in slugs)) continue
      if (!(w in vindex)) {
        strongconnect(w)
        if (vlowlink[w] < vlowlink[v]) vlowlink[v] = vlowlink[w]
      } else if (w in onstack) {
        if (vindex[w] < vlowlink[v]) vlowlink[v] = vindex[w]
      }
    }
    if (vlowlink[v] == vindex[v]) {
      cur_size = 0
      do {
        w = stack[--stacksize]
        delete onstack[w]
        scc_member[scc_count, cur_size++] = w
      } while (w != v)
      scc_sizes[scc_count] = cur_size
      scc_count++
    }
  }
  END {
    idx = 0; stacksize = 0; scc_count = 0; found_cycle = 0
    # Slug-sorted traversal order for deterministic output across runs.
    n = 0
    for (s in slugs) sorted[++n] = s
    for (i = 2; i <= n; i++) {
      key = sorted[i]; j = i - 1
      while (j > 0 && sorted[j] > key) { sorted[j+1] = sorted[j]; j-- }
      sorted[j+1] = key
    }
    for (i = 1; i <= n; i++) {
      if (!(sorted[i] in vindex)) strongconnect(sorted[i])
    }
    # Sort SCCs by their lex-min member so the output order is stable.
    for (i = 0; i < scc_count; i++) {
      min_member = ""
      for (k = 0; k < scc_sizes[i]; k++) {
        m = scc_member[i, k]
        if (min_member == "" || m < min_member) min_member = m
      }
      scc_key[i] = min_member
      scc_order[i] = i
    }
    for (a = 1; a < scc_count; a++) {
      key = scc_key[scc_order[a]]; oa = scc_order[a]; b = a - 1
      while (b >= 0 && scc_key[scc_order[b]] > key) { scc_order[b+1] = scc_order[b]; b-- }
      scc_order[b+1] = oa
    }
    for (rank = 0; rank < scc_count; rank++) {
      i = scc_order[rank]
      sz = scc_sizes[i]
      if (sz > 1) {
        # Sort members for deterministic in-cycle order.
        for (k = 0; k < sz; k++) members[k] = scc_member[i, k]
        for (a = 1; a < sz; a++) {
          mk = members[a]; b = a - 1
          while (b >= 0 && members[b] > mk) { members[b+1] = members[b]; b-- }
          members[b+1] = mk
        }
        msg = "cycle: "
        for (k = 0; k < sz; k++) {
          if (k > 0) msg = msg " -> "
          msg = msg members[k]
        }
        msg = msg " -> " members[0]
        print msg > "/dev/stderr"
        found_cycle = 1
      } else {
        v = scc_member[i, 0]
        for (k = 1; k <= degree[v] + 0; k++) {
          if (adj[v, k] == v) {
            print "cycle: " v " -> " v > "/dev/stderr"
            found_cycle = 1
            break
          }
        }
      }
    }
    if (found_cycle) exit 1
  }
' "$graph_file" || cycle_rc=$?

if [ "$cycle_rc" -ne 0 ]; then
  echo "" >&2
  echo "gen-spec-deps: dependency graph contains cycles (see above)." >&2
  echo "The body inline links above induced cycles in the derived dep graph." >&2
  echo "Remove or move the offending links under '## See also' before committing." >&2
  exit 1
fi

if [ "$changed" -gt 0 ] && [ "$dry_run" -eq 1 ]; then
  exit 1
fi
