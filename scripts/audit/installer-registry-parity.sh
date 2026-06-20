#!/usr/bin/env bash
# scripts/audit/installer-registry-parity.sh — Family 14 of /audit.
#
# Verifies the one-line installer (install.sh) and the agent registry in
# framework/bootstrap/govern.md agree on which agents exist and where each
# one's `govern` bootstrap is placed.
#
# install.sh hard-codes one `case` arm per agent (`<key>) ... dest="..."`).
# The registry's §Derived values table derives each agent's `govern` install
# path from its row by layout:
#
#   claude-style → {config_dir}/commands/govern.md
#   antigravity  → {config_dir}/skills/govern/SKILL.md
#   opencode     → {config_dir}/command/govern.md
#
# The check enforces per-key parity in three directions:
#
#   1. Every registry agent has a matching install.sh `case` arm whose
#      dest equals the registry-derived path. (Catches: an agent added to
#      the registry but not the installer — the gap the "single registry
#      row plus a permission file" claim would otherwise hide.)
#   2. Every install.sh `case` arm names a registry agent and installs to
#      that agent's derived path. (Catches: a stale or mis-mapped arm.)
#   3. Every settings file install.sh pre-seeds matches that agent's
#      registry settings_template, compared as JSON. (Catches: the seeded
#      permission copy silently drifting from the registry it duplicates.)
#
# Directions 1-2 are pure text extraction — no jq, no associative arrays
# (macOS bash 3.2). Direction 3 uses python3 (already a govern bootstrap
# dependency, and used by sibling audit scripts) because the three permission
# formats make an order-insensitive JSON compare the only reliable check.
# This is the audit check spec 003's curl-sh-installer scenario calls for,
# resolving its installer<->registry parity open question per the
# "never depend on human diligence" design principle.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

GOVERN="framework/bootstrap/govern.md"
INSTALLER="install.sh"

drift=0
emit() {
  echo "installer-registry-parity | $1 | $2 | $3"
  drift=1
}

if [ ! -f "$GOVERN" ]; then
  emit "$GOVERN" "agent registry source missing" "restore $GOVERN"
  exit 1
fi
if [ ! -f "$INSTALLER" ]; then
  emit "$INSTALLER" "installer missing" "restore $INSTALLER or remove this audit family"
  exit 1
fi

# Derive an agent's `govern` install path from its config_dir + layout,
# mirroring the §Derived values "govern install path" row.
derive_path() {
  case "$2" in
    claude-style) printf '%s/commands/govern.md\n' "$1" ;;
    antigravity)  printf '%s/skills/govern/SKILL.md\n' "$1" ;;
    opencode)     printf '%s/command/govern.md\n' "$1" ;;
    *)            printf '\n' ;;  # unknown layout — signalled by empty result
  esac
}

# registry_map / installer_map: newline-delimited "key<TAB>path" records.
# Extract the agent-registry table rows: scoped to the `## Agent Registry`
# section, up to the next heading. Columns (split on `|`): 2=key,
# 4=config_dir, 5=layout. Backticks and spaces are stripped; the header row
# (key == "key") and the `---` separator are skipped. No registry cell
# contains a literal `|`, so the split is safe.
registry_map="$(
  awk '
    /^## Agent Registry/ { inseg = 1; next }
    inseg && /^#/        { inseg = 0 }
    inseg && /^\|/ {
      n = split($0, c, "|")
      key = c[2]; cd = c[4]; lay = c[5]
      gsub(/[`[:space:]]/, "", key)
      gsub(/[`[:space:]]/, "", cd)
      gsub(/[`[:space:]]/, "", lay)
      if (key == "" || key == "key" || key ~ /^-+$/) next
      print key "\t" cd "\t" lay
    }
  ' "$GOVERN"
)"

# Extract install.sh's case-arm key -> dest mapping. Track the current
# arm's primary key (first token before `|`), and bind it to the first
# `dest="..."` that follows. The `*)` default arm has no dest and is skipped.
installer_map="$(
  awk '
    /case[[:space:]]+"\$agent"[[:space:]]+in/ { incase = 1; next }
    /^esac/ { incase = 0 }
    incase && /^[[:space:]]*[A-Za-z*][A-Za-z0-9_| ]*\)/ {
      line = $0
      sub(/\).*/, "", line)
      gsub(/[[:space:]]/, "", line)
      split(line, toks, "|")
      curkey = toks[1]
      next
    }
    incase && curkey != "" && curkey != "*" && /dest="/ {
      d = $0; sub(/.*dest="/, "", d); sub(/".*/, "", d)
      print curkey "\t" d
      curkey = ""
    }
  ' "$INSTALLER"
)"

lookup() { awk -F'\t' -v k="$2" '$1 == k { print $NF; exit }' <<EOF
$1
EOF
}

# Direction 1: every registry agent has a matching installer arm.
while IFS="$(printf '\t')" read -r key cd lay; do
  [ -n "$key" ] || continue
  want="$(derive_path "$cd" "$lay")"
  if [ -z "$want" ]; then
    emit "$GOVERN (agent $key)" "unrecognized layout '$lay' — cannot derive install path" \
      "add a '$lay' branch to derive_path() in scripts/audit/installer-registry-parity.sh and a matching install.sh case arm"
    continue
  fi
  got="$(lookup "$installer_map" "$key")"
  if [ -z "$got" ]; then
    emit "$INSTALLER" "registry agent '$key' has no install.sh case arm" \
      "add a '$key)' arm to install.sh placing the bootstrap at $want"
  elif [ "$got" != "$want" ]; then
    emit "$INSTALLER (agent $key)" "installs to '$got' but the registry derives '$want'" \
      "fix the '$key)' dest in install.sh to match the registry-derived path"
  fi
done <<EOF
$registry_map
EOF

# Direction 2: every installer arm names a known registry agent.
while IFS="$(printf '\t')" read -r key dest; do
  [ -n "$key" ] || continue
  if [ -z "$(lookup "$registry_map" "$key")" ]; then
    emit "$INSTALLER (agent $key)" "installs to '$dest' but '$key' is not in the agent registry" \
      "add '$key' to the §Agent Registry table in $GOVERN, or remove its arm from install.sh"
  fi
done <<EOF
$installer_map
EOF

# Direction 3: settings-template parity. install.sh pre-seeds each agent's
# permission file (so the first /govern run does not prompt for its bootstrap
# shell commands) by hard-coding a copy of that agent's registry settings_template.
# That duplicate must not silently drift. The three permission formats (claude
# Bash()/Read(), auggie toolPermissions/regex, antigravity command()) make a text
# diff unreliable, so this pass uses python3 for an order-insensitive JSON compare
# of each install.sh seed against its §Agent Registry settings_template.
seed_drift="$(
python3 - "$GOVERN" "$INSTALLER" <<'PY'
import json, re, sys
govern, installer = sys.argv[1], sys.argv[2]

# Registry: agent key -> settings_template JSON (column 5 of the §Agent Registry
# table). The cell is backtick-wrapped JSON containing no literal '|'.
rows = {}
in_reg = False
for line in open(govern):
    if line.startswith("## Agent Registry"):
        in_reg = True
        continue
    if in_reg and line.startswith("#"):
        break
    if in_reg and line.lstrip().startswith("|"):
        c = line.split("|")
        if len(c) < 7:
            continue
        key = c[1].strip().strip("`").strip()
        if key in ("", "key") or set(key) == {"-"}:
            continue
        rows[key] = c[5].strip().strip("`").strip()

# Installer: settings-file path -> seeded JSON heredoc body.
text = open(installer).read()
PATH2KEY = {
    ".claude/settings.local.json": "claude",
    ".augment/settings.local.json": "auggie",
    ".agents/settings.json": "antigravity",
    "opencode.json": "opencode",
}
seeds = {}
for m in re.finditer(r"cat > (\S+) <<'JSON'\n(.*?)\nJSON", text, re.S):
    path, body = m.group(1), m.group(2)
    key = PATH2KEY.get(path)
    if key:
        seeds[key] = (path, body)

def norm(x):
    if isinstance(x, dict):
        return {k: norm(v) for k, v in x.items()}
    if isinstance(x, list):
        return sorted((norm(i) for i in x), key=lambda e: json.dumps(e, sort_keys=True))
    return x

def emit(loc, msg, fix):
    print(f"installer-registry-parity | {loc} | {msg} | {fix}")

for key, (path, body) in seeds.items():
    if key not in rows:
        emit(f"{installer} ({path})",
             f"seeds settings for '{key}' but '{key}' is not in the agent registry",
             f"add '{key}' to the §Agent Registry table in {govern}, or remove its seed from {installer}")
        continue
    try:
        reg = norm(json.loads(rows[key]))
    except json.JSONDecodeError as e:
        emit(f"{govern} (agent {key})", f"settings_template is not valid JSON: {e}",
             "repair the registry settings_template cell")
        continue
    try:
        seed = norm(json.loads(body))
    except json.JSONDecodeError as e:
        emit(f"{installer} ({path})", f"seeded settings JSON is malformed: {e}",
             "repair the install.sh heredoc")
        continue
    if reg != seed:
        emit(f"{installer} ({path})",
             f"settings seed for '{key}' drifts from the registry settings_template",
             f"re-sync the '{path}' heredoc in {installer} with the '{key}' row in {govern} §Agent Registry")
PY
)"
if [ -n "$seed_drift" ]; then
  printf '%s\n' "$seed_drift"
  drift=1
fi

exit "$drift"
