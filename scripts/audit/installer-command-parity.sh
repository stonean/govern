#!/usr/bin/env bash
# scripts/audit/installer-command-parity.sh — Family 16 of /audit.
#
# The /govern bootstrap (framework/bootstrap/govern.md) scaffolds an
# adopter's slash commands from a HARDCODED manifest table in its
# §Per-Agent Scaffolding → Slash commands section — one
# `framework/commands/<name>.md` row per installable command. That table
# is maintained by hand and drifts silently: when a new command is added
# under framework/commands/ (spec 041's prune.md was the first to bite),
# every other integration point tends to get updated (help.md,
# configure/*.md, runtime-tools.txt) while the installer manifest is easy
# to miss. The result is a command that exists in the framework and is
# dogfooded here (gen-claude-commands.sh globs all of framework/commands/)
# yet never reaches adopters via /govern — and, because §Slash command
# cleanup deletes any unlisted file, would be removed if hand-placed.
#
# This family pins the manifest to the source of truth: the set of
# `framework/commands/*.md` rows in govern.md must equal the set of
# framework/commands/*.md files, minus the maintainer-only commands that
# are intentionally not shipped to adopters (see `excl` below).
#
# Family 2 (manifest-parity.sh) explicitly deferred the installer file
# list as out of scope; this family closes that gap for the command
# manifest specifically, anchored on the real source files rather than on
# a second prose list.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

GOVERN="framework/bootstrap/govern.md"
SRC_DIR="framework/commands"
SECTION="$GOVERN §Per-Agent Scaffolding → Slash commands"

drift=0
emit() {
  echo "installer-command-parity | $1 | $2 | $3"
  drift=1
}

if [ ! -f "$GOVERN" ]; then
  emit "$GOVERN" "bootstrap file not found" "restore framework/bootstrap/govern.md"
  exit "$drift"
fi

# Manifest command names: the `| `framework/commands/<name>.md` | ... |`
# table rows in govern.md. Anchored on a leading `| ` + backticked source
# path so prose references to framework/commands/*.md elsewhere in the file
# (e.g. the review.disabled-rule-files pointer) are never mistaken for rows.
manifest="$(grep -oE '^\| `framework/commands/[a-z-]+\.md`' "$GOVERN" \
  | sed -E 's/.*framework\/commands\/([a-z-]+)\.md.*/\1/' | sort -u)"

# Actual source files under framework/commands/.
actual="$(for f in "$SRC_DIR"/*.md; do basename "$f" .md; done | sort -u)"

# Maintainer-only commands intentionally excluded from the adopter manifest.
# audit is maintainer-only (see framework/commands/audit.md's own header).
# Add a line here when a new command is deliberately withheld from /govern.
excl="$(sort -u <<'EOF'
audit
EOF
)"

# Expected manifest = actual source files minus maintainer-only exclusions.
expected="$(comm -23 <(printf '%s' "$actual") <(printf '%s' "$excl"))"

# Missing: a shippable command with no manifest row (the prune bug).
while IFS= read -r name; do
  [ -z "$name" ] && continue
  emit "$SECTION" \
    "framework/commands/$name.md has no installer manifest row — /govern never scaffolds it for adopters" \
    "add | \`framework/commands/$name.md\` | \`{config_dir}/commands/{project}/$name.md\` | to the Slash commands table, then bump the two 'N framework/commands/*.md rows' counts"
done <<< "$(comm -23 <(printf '%s' "$expected") <(printf '%s' "$manifest"))"

# Stale: a manifest row whose source file no longer exists.
while IFS= read -r name; do
  [ -z "$name" ] && continue
  emit "$SECTION" \
    "manifest row framework/commands/$name.md has no source file under framework/commands/" \
    "remove the stale row, or restore/rename the source file"
done <<< "$(comm -23 <(printf '%s' "$manifest") <(printf '%s' "$actual"))"

# Wrongly shipped: a maintainer-only command that appears in the manifest.
while IFS= read -r name; do
  [ -z "$name" ] && continue
  emit "$SECTION" \
    "maintainer-only command framework/commands/$name.md appears in the installer manifest — it would be shipped to adopters" \
    "remove the row, or drop $name from the excl list in this script if it is now adopter-facing"
done <<< "$(comm -12 <(printf '%s' "$manifest") <(printf '%s' "$excl"))"

exit "$drift"
