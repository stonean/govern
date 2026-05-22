#!/usr/bin/env bash
# scripts/audit/migration-coverage.sh — Family 10 of /audit.
#
# Verifies framework/migrations.toml is internally consistent with the
# adjacent framework/migrations/*.md procedure files and with current
# framework/ state. Three checks:
#
#   10a no-orphan-procedure-files: every framework/migrations/*.md has a
#       matching TOML entry whose procedure_file points at it.
#   10b no-stale-target-paths: every active registry entry's framework/-
#       prefixed target_paths refers to a path that does NOT exist in
#       current framework/. (Non-framework/ target_paths are adopter-
#       relative and cannot be verified from this repo.)
#   10c no-broken-procedure-references: every TOML entry's procedure_file
#       points at an existing framework/migrations/{id}.md.
#
# Requires `python3` (standard on developer macOS / CI Ubuntu) for TOML
# parsing — Python 3.11+ ships with `tomllib`.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

REGISTRY="framework/migrations.toml"
PROCEDURES_DIR="framework/migrations"

drift=0

emit() {
  echo "migration-coverage | $1 | $2 | $3"
  drift=1
}

if [ ! -f "$REGISTRY" ]; then
  emit "$REGISTRY" "registry file missing" "create $REGISTRY per spec 027"
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  emit "$REGISTRY" "python3 not on PATH — cannot parse TOML" "install python3 (3.11+) and re-run"
  exit 1
fi
if ! python3 -c "import tomllib" 2>/dev/null; then
  emit "$REGISTRY" "python3 lacks tomllib (need Python 3.11+)" "upgrade python3 to 3.11+ and re-run"
  exit 1
fi
if ! python3 -c "import tomllib; tomllib.loads(open('$REGISTRY').read())" 2>/dev/null; then
  emit "$REGISTRY" "registry is not valid TOML" "fix the TOML syntax"
  exit 1
fi

# Extract per-entry data via python3 + tomllib.
# Output format, one row per entry: "{id}\t{procedure_file}\t{target_paths_pipe_separated}"
ENTRY_DATA=$(python3 <<'PY'
import tomllib
with open("framework/migrations.toml", "rb") as f:
    data = tomllib.load(f)
for m in data.get("migrations", []):
    id_ = m.get("id", "")
    pf = m.get("procedure_file", "")
    tps = "|".join(m.get("target_paths", []))
    print(f"{id_}\t{pf}\t{tps}")
PY
)

# 10a no-orphan-procedure-files: every framework/migrations/*.md must
# have a matching TOML entry whose procedure_file ends with that filename.
if [ -d "$PROCEDURES_DIR" ]; then
  for f in "$PROCEDURES_DIR"/*.md; do
    [ -e "$f" ] || continue
    if ! grep -qF "procedure_file = \"$f\"" "$REGISTRY"; then
      emit "$f" "orphan procedure file (no [[migrations]] entry references it)" \
        "add a [[migrations]] entry with procedure_file = \"$f\", or delete the file"
    fi
  done
fi

# 10c no-broken-procedure-references: every TOML procedure_file path
# must exist on disk.
while IFS=$'\t' read -r id pf tps; do
  [ -n "$id" ] || continue
  if [ ! -f "$pf" ]; then
    emit "$REGISTRY" "[[migrations]] entry '$id' references missing procedure_file: $pf" \
      "create $pf or correct the procedure_file value"
  fi
done <<< "$ENTRY_DATA"

# 10b no-stale-target-paths: every framework/-prefixed target_path in an
# active entry must NOT exist in current framework/.
# Glob patterns (containing *) are expanded; literal paths use test -e.
while IFS=$'\t' read -r id pf tps; do
  [ -n "$id" ] || continue
  IFS='|' read -r -a paths <<< "$tps"
  for p in "${paths[@]}"; do
    case "$p" in
      framework/*) ;;
      *) continue ;;  # adopter-relative; cannot verify from this repo
    esac
    if [[ "$p" == *"*"* ]]; then
      # Glob pattern — expand and check for any matches.
      # shellcheck disable=SC2206
      matches=( $p )
      for m in "${matches[@]}"; do
        if [ -e "$m" ]; then
          emit "$REGISTRY" "[[migrations]] entry '$id' claims '$p' was removed but '$m' still exists" \
            "delete $m or remove the target_path from the entry"
        fi
      done
    else
      if [ -e "$p" ]; then
        emit "$REGISTRY" "[[migrations]] entry '$id' claims '$p' was removed but it still exists" \
          "delete $p or remove the target_path from the entry"
      fi
    fi
  done
done <<< "$ENTRY_DATA"

# TODO: parse archived CHANGELOG.md entries for the same no-stale-target-paths check.
# Deferred until the first sunset commit so the archive format is established
# by example. Captured in spec 027 as a future Family 10 enhancement.

exit "$drift"
