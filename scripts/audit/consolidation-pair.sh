#!/usr/bin/env bash
# scripts/audit/consolidation-pair.sh — Family 11 of /audit.
#
# Verifies that every "consolidated artifact" (a single canonical path
# that multiple loosely-coupled files reference) agrees across all
# referencing sources. The motivating case is .govern.session.toml,
# which appears in:
#
#   1. The runtime's SESSION_FILE constant (write_session.rs).
#   2. The migration body that translates legacy files into it
#      (framework/migrations/session-file-consolidate.md).
#   3. The framework-managed gitignore block
#      (framework/templates/project/gitignore).
#   4. The Claude configure-permission file's per-path allow entries
#      (framework/bootstrap/configure/claude.md).
#
# If any one of these renames its reference without the others, the
# system silently breaks — the runtime writes one file, the migration
# translates to a different one, gitignore tracks a third, and the
# permission allowlist allows a fourth. This audit catches that drift
# at /audit time.
#
# The check is structural (grep-based): it extracts the literal string
# from one authoritative source (the SESSION_FILE constant) and
# confirms every other source mentions exactly that string at least
# once.

set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

drift=0

emit() {
  echo "consolidation-pair | $1 | $2 | $3"
  drift=1
}

# 11a — Extract SESSION_FILE from write_session.rs. The constant is the
# runtime's authoritative source for the consolidated session-file path.
SESSION_RS="runtime/src/primitives/write_session.rs"
if [ ! -f "$SESSION_RS" ]; then
  emit "$SESSION_RS" "missing — cannot anchor the consolidation-pair check" \
    "restore the runtime crate or remove this audit family"
  exit 1
fi

# Match: pub(crate) const SESSION_FILE: &str = "<value>";
SESSION_FILE=$(awk -F'"' '/SESSION_FILE: &str = "/ { print $2; exit }' "$SESSION_RS")
if [ -z "$SESSION_FILE" ]; then
  emit "$SESSION_RS" "SESSION_FILE constant not found in expected form" \
    "ensure the constant declaration matches: pub(crate) const SESSION_FILE: &str = \"…\";"
  exit 1
fi

# 11b — Migration body must mention the SESSION_FILE value as its
# destination. The migration translates legacy files INTO this path, so
# a mismatch means the runtime and the bootstrap migration disagree
# about where the session lives.
MIGRATION_BODY="framework/migrations/session-file-consolidate.md"
if [ -f "$MIGRATION_BODY" ]; then
  if ! grep -qF "$SESSION_FILE" "$MIGRATION_BODY"; then
    emit "$MIGRATION_BODY" \
      "migration body does not reference SESSION_FILE value '$SESSION_FILE'" \
      "update the migration body to name '$SESSION_FILE' as the consolidated destination"
  fi
else
  emit "$MIGRATION_BODY" "missing — registry references it but the file is absent" \
    "create $MIGRATION_BODY or remove the registry entry"
fi

# 11c — Framework gitignore template must list the SESSION_FILE so
# adopters who run /govern get the new file gitignored.
GITIGNORE_TEMPLATE="framework/templates/project/gitignore"
if [ -f "$GITIGNORE_TEMPLATE" ]; then
  if ! grep -qF "$SESSION_FILE" "$GITIGNORE_TEMPLATE"; then
    emit "$GITIGNORE_TEMPLATE" \
      "framework gitignore does not list '$SESSION_FILE'" \
      "add '$SESSION_FILE' to the framework-managed gitignore block"
  fi
else
  emit "$GITIGNORE_TEMPLATE" "missing" "restore the framework gitignore template"
fi

# 11d — Claude configure-permission file must allow Edit/Write on the
# SESSION_FILE so /gov:* commands don't trigger per-write prompts. The
# legacy reference (.claude/gov-session.json) would also leak through
# here if not swept, since this file is what spec 023 wired up.
CONFIGURE_CLAUDE="framework/bootstrap/configure/claude.md"
if [ -f "$CONFIGURE_CLAUDE" ]; then
  if ! grep -qF "$SESSION_FILE" "$CONFIGURE_CLAUDE"; then
    emit "$CONFIGURE_CLAUDE" \
      "configure-permission file does not reference '$SESSION_FILE'" \
      "ensure Edit($SESSION_FILE) and Write($SESSION_FILE) entries exist"
  fi
else
  emit "$CONFIGURE_CLAUDE" "missing" "restore the Claude configure template"
fi

# 11e — Migration body must mention BOTH camelCase legacy keys
# (scenarioPath, setAt) AND their kebab-case replacements
# (scenario-path, set-at) so the rename contract is auditable. If a
# rename is silently dropped from the body, adopters keep the legacy
# camelCase keys in TOML, which the runtime's reader (kebab-case-only)
# ignores. The data-loss failure mode is silent.
if [ -f "$MIGRATION_BODY" ]; then
  for pair in "scenarioPath:scenario-path" "setAt:set-at"; do
    legacy="${pair%:*}"
    new="${pair#*:}"
    if ! grep -qF "$legacy" "$MIGRATION_BODY"; then
      emit "$MIGRATION_BODY" \
        "migration body does not name the legacy key '$legacy'" \
        "include '$legacy' in the rename contract section"
    fi
    if ! grep -qF "$new" "$MIGRATION_BODY"; then
      emit "$MIGRATION_BODY" \
        "migration body does not name the new key '$new'" \
        "include '$new' in the rename contract section"
    fi
  done
fi

exit "$drift"
