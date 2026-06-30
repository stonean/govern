#!/usr/bin/env bash
# scripts/lib/specs-root.sh — shared spec-tree helpers for the generators.
#
# Sourced by gen-spec-deps.sh and gen-cross-service-refs.sh, which run in both
# govern's own pre-commit and adopter pre-commit hooks. One definition rather
# than a copy per generator (spec 040 review — no drift). Shipped to adopters
# via the Shared Files manifest in framework/bootstrap/govern.md alongside the
# generators; sourced by script-relative path so it resolves regardless of cwd
# or a --root override (which retargets $ROOT but not the script location).
#
# The sourcing script MUST define $ROOT (its repo-root variable) before calling
# any of these, and $SPECS_ROOT (resolve_specs_root's result) before calling
# list_specs / staged_specs; both generators set them up top.

# Spec-root directory name from a given .govern.toml's [paths] specs-root,
# defaulting to "specs" (spec 040) when the file is absent, the key is missing,
# or the value is outside the [A-Za-z0-9_-] charset (path separators, "..",
# ".", or any regex metacharacter). That conservative charset matches the
# runtime's validate_specs_root, so the name is safe both as a path component
# and when interpolated into the grep/awk regexes the generators build from it.
#
# Takes the .govern.toml path as an argument so callers can resolve THIS repo's
# root (resolve_specs_root, below) or a *referenced* service's own root from its
# local checkout (gen-cross-service-refs.sh's checkout-reachable matcher tier —
# scenario referenced-service-spec-root).
specs_root_of() {
  local toml="$1" name=""
  if [ -f "$toml" ]; then
    name="$(awk '
      /^\[/ { in_paths = ($0 ~ /^\[paths\][[:space:]]*$/); next }
      in_paths && /^[[:space:]]*specs-root[[:space:]]*=/ {
        line = $0
        sub(/^[^=]*=[[:space:]]*/, "", line)
        if (match(line, /"[^"]*"/)) { print substr(line, RSTART + 1, RLENGTH - 2) }
        else { sub(/[[:space:]]*(#.*)?$/, "", line); print line }
        exit
      }
    ' "$toml")"
  fi
  case "$name" in
    "" | *[!A-Za-z0-9_-]*) name="specs" ;;
  esac
  printf '%s' "$name"
}

# Spec-root directory name for THIS repo ($ROOT/.govern.toml).
resolve_specs_root() {
  specs_root_of "$ROOT/.govern.toml"
}

# Feature-spec files to process, scoped to the git index (tracked + staged)
# rather than a worktree glob. Untracked, in-progress drafts — e.g. a /specify
# spec the author has not `git add`ed yet — are intentionally excluded so they
# are never rewritten, never enter a generator's graph, and never block an
# unrelated commit (spec 017 / tracked-specs-not-worktree). Falls back to a
# worktree glob only outside a git repo, where there is no index.
list_specs() {
  if git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1; then
    git -C "$ROOT" ls-files -- "$SPECS_ROOT" \
      | { grep -E "^$SPECS_ROOT/[0-9][0-9][0-9]-[^/]+/(spec|spec-and-plan)\.md$" || true; } \
      | while IFS= read -r rel; do printf '%s/%s\n' "$ROOT" "$rel"; done
  else
    local f
    for f in "$ROOT"/"$SPECS_ROOT"/[0-9][0-9][0-9]-*/spec.md "$ROOT"/"$SPECS_ROOT"/[0-9][0-9][0-9]-*/spec-and-plan.md; do
      [ -e "$f" ] && printf '%s\n' "$f"
    done
  fi
}

# Feature-spec files staged in the git index for the pending commit — the
# --staged rewrite set (the adopter pre-commit path), so committing one spec
# never rewrites the derived frontmatter of unrelated specs. Empty outside a
# git repo. How each generator combines this with list_specs is its own
# concern (see the call sites).
staged_specs() {
  git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1 || return 0
  git -C "$ROOT" diff --cached --name-only -- "$SPECS_ROOT" \
    | { grep -E "^$SPECS_ROOT/[0-9][0-9][0-9]-[^/]+/(spec|spec-and-plan)\.md$" || true; } \
    | while IFS= read -r rel; do printf '%s/%s\n' "$ROOT" "$rel"; done
}
