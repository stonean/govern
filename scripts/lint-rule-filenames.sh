#!/usr/bin/env bash
# Verify the closed-suffix policy for rule files.
#
# Every `framework/rules/*.md` file MUST end in one of:
#   - `-backend.md`   (loaded for backend stacks)
#   - `-frontend.md`  (loaded for frontend stacks)
#   - `-cross.md`     (loaded for all stacks; cross-cutting)
#
# The closed-suffix policy is the surface signal `/gov:review` and
# `/gov:analyze` use to derive rule-file selection without a hardcoded
# allowlist (see framework/constitution.md §rules).
#
# Source of truth: framework/constitution.md §rules
# Consumed by: .github/workflows/markdown-only-pipeline.yml

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

for arg in "$@"; do
  case "$arg" in
    -h|--help)
      sed -n '2,14p' "$0" | sed 's/^# \{0,1\}//'
      echo
      echo "Usage: $(basename "$0")"
      echo "  Exits 0 when every framework/rules/*.md filename ends in a valid suffix."
      echo "  Exits 1 when any file fails (errors printed to stdout)."
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

shopt -s nullglob

errors=0
files=("$ROOT"/framework/rules/*.md)

for f in "${files[@]}"; do
  [ -f "$f" ] || continue
  rel="${f#"$ROOT"/}"
  base="$(basename "$f")"

  case "$base" in
    *-backend.md|*-frontend.md|*-cross.md) ;;
    *)
      echo "$rel: filename does not end in -backend.md, -frontend.md, or -cross.md"
      errors=$((errors + 1))
      ;;
  esac
done

if [ "$errors" -gt 0 ]; then
  exit 1
fi
exit 0
