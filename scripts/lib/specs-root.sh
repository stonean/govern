#!/usr/bin/env bash
# scripts/lib/specs-root.sh — shared spec-root resolver for the generators.
#
# Sourced by gen-spec-deps.sh and gen-cross-service-refs.sh, which run in both
# govern's own pre-commit and adopter pre-commit hooks. One definition rather
# than a copy per generator (spec 040 review — no drift). Shipped to adopters
# via the Shared Files manifest in framework/bootstrap/govern.md alongside the
# generators; sourced by script-relative path so it resolves regardless of cwd
# or a --root override (which retargets $ROOT but not the script location).
#
# The sourcing script MUST define $ROOT (its repo-root variable) before calling
# resolve_specs_root; both generators set it from $(dirname "$0")/.. up top.

# Spec-root directory name from $ROOT/.govern.toml [paths] specs-root,
# defaulting to "specs" (spec 040). A value outside the [A-Za-z0-9_-] charset
# (path separators, "..", ".", or any regex metacharacter) falls back to the
# default — the same conservative charset the runtime's validate_specs_root
# enforces, so the name is safe both as a path component and when interpolated
# into the grep/awk regexes the generators build from it.
resolve_specs_root() {
  local toml="$ROOT/.govern.toml" name=""
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
