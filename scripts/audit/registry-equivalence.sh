#!/usr/bin/env bash
# scripts/audit/registry-equivalence.sh — Family 3 of /audit.
#
# Verifies framework/workflows/registry.json and framework/workflows/*.md
# agree on three invariants:
#
#   1. Every registry entry's `template` field names a file that exists
#      under framework/workflows/.
#   2. Every workflow `.md` file under framework/workflows/ appears as a
#      registry entry's `template`.
#   3. For every (registry, workflow file) pair: the registry's
#      `description` field matches the workflow file's frontmatter
#      `description:` field.
#
# Requires `jq` (standard on developer macOS / CI Ubuntu).

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

REGISTRY="framework/workflows/registry.json"
WORKFLOWS_DIR="framework/workflows"

drift=0

emit() {
  echo "registry-equivalence | $1 | $2 | $3"
  drift=1
}

if [ ! -f "$REGISTRY" ]; then
  emit "$REGISTRY" "workflow registry file missing" "create $REGISTRY per the workflow registry schema"
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  emit "$REGISTRY" "jq not on PATH — cannot validate registry" "install jq (brew install jq / apt install jq) and re-run"
  exit 1
fi
if ! jq empty "$REGISTRY" >/dev/null 2>&1; then
  emit "$REGISTRY" "registry.json is not valid JSON" "fix the JSON syntax"
  exit 1
fi

# Set of templates referenced by the registry.
registry_templates="$(jq -r '.[] | .template' "$REGISTRY" | sort -u)"
# Set of `.md` files under the workflows dir (basenames).
file_templates="$(ls -1 "$WORKFLOWS_DIR"/*.md 2>/dev/null | xargs -I{} basename {} | sort -u)"

# 1. Every registry entry references a real file.
while IFS= read -r tpl; do
  [ -z "$tpl" ] && continue
  if [ ! -f "$WORKFLOWS_DIR/$tpl" ]; then
    emit "$REGISTRY" "registry references missing file: $tpl" "either add $WORKFLOWS_DIR/$tpl or remove the registry entry"
  fi
done <<< "$registry_templates"

# 2. Every workflow file appears in the registry.
while IFS= read -r f; do
  [ -z "$f" ] && continue
  if ! grep -qFx "$f" <<< "$registry_templates"; then
    emit "$WORKFLOWS_DIR/$f" "workflow file not registered in registry.json" "add a registry entry whose template field is \"$f\""
  fi
done <<< "$file_templates"

# 3. Description parity (only for entries whose file exists).
while IFS=$'\t' read -r tpl reg_desc; do
  [ -z "$tpl" ] && continue
  file="$WORKFLOWS_DIR/$tpl"
  if [ ! -f "$file" ]; then
    continue
  fi
  # Extract `description:` from frontmatter (first occurrence between two --- lines).
  file_desc="$(awk '
    /^---$/ { count++; if (count == 2) exit; next }
    count == 1 && /^description:/ {
      sub(/^description: */, "")
      sub(/^"/, ""); sub(/"$/, "")
      print
      exit
    }
  ' "$file")"
  if [ "$file_desc" != "$reg_desc" ]; then
    emit "$file" "description differs from registry.json entry for $tpl" "reconcile: registry says \"$reg_desc\"; file says \"$file_desc\""
  fi
done < <(jq -r '.[] | "\(.template)\t\(.description)"' "$REGISTRY")

exit "$drift"
