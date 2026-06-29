#!/usr/bin/env bash
# Verify rule-ID grammar and per-file uniqueness in framework/rules/*.md.
#
# Per the rule schema, every level-3 heading in a rule file is a rule ID and
# carries nothing but the ID. Each MUST match the grammar
# `{SURFACE}-{CATEGORY}-{NNN}`:
#   - SURFACE   one of BE, FE, CFG, QUAL (registered surfaces)
#   - CATEGORY  [A-Z][A-Z0-9]*  (uppercase alphanumeric; first char a letter)
#   - NNN       a 3- or 4-digit zero-padded sequence number
# IDs MUST be unique within a file.
#
# This catches the failure mode where a category abbreviation drifts from the
# schema grammar that the runtime `check-rule-ids` harvester relies on
# (e.g. a digit-bearing category the harvester regex does not match).
#
# Source of truth: specs/008-security-rules/data-model.md (BE/FE surfaces),
#                  specs/017-derive-dont-ask/data-model.md (CFG surface),
#                  specs/036-quality-cross-rules/data-model.md (QUAL surface),
#                  framework/constitution.md §rules
# Consumed by: .github/workflows/markdown-only-pipeline.yml

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

for arg in "$@"; do
  case "$arg" in
    -h|--help)
      sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'
      echo
      echo "Usage: $(basename "$0")"
      echo "  Exits 0 when every rule-ID heading is well-formed and unique per file."
      echo "  Exits 1 when any heading fails (errors printed to stdout)."
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

shopt -s nullglob

# Registered surfaces. Extend deliberately when a new rule-introducing spec
# registers a surface (mirrors the intent of the closed-suffix filename policy).
id_re='^(BE|FE|CFG|QUAL)-[A-Z][A-Z0-9]*-[0-9]{3,4}$'

errors=0
files=("$ROOT"/framework/rules/*.md)

for f in "${files[@]}"; do
  [ -f "$f" ] || continue
  rel="${f#"$ROOT"/}"
  seen=""

  # Level-3 headings only: the schema reserves `### ` for rule IDs.
  while IFS= read -r line; do
    id="${line#"### "}"
    id="${id%%[[:space:]]*}"   # defensive: drop any trailing content

    if [[ ! "$id" =~ $id_re ]]; then
      echo "$rel: malformed rule ID heading: '### $id' (expected {BE|FE|CFG|QUAL}-{CATEGORY}-{NNN})"
      errors=$((errors + 1))
      continue
    fi

    case " $seen " in
      *" $id "*)
        echo "$rel: duplicate rule ID: $id"
        errors=$((errors + 1))
        ;;
      *) seen="$seen $id" ;;
    esac
  done < <(grep -E '^### ' "$f" || true)
done

if [ "$errors" -gt 0 ]; then
  exit 1
fi
exit 0
