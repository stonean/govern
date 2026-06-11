#!/bin/sh
# govern installer — places the /govern bootstrap command for your AI coding agent.
#
# Usage:
#   curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/stonean/govern/main/install.sh | sh
#
# Pick an agent explicitly (default: autodetect, falling back to claude):
#   ... | sh -s -- claude
#   ... | sh -s -- auggie
#   ... | sh -s -- antigravity
#
# The script is idempotent — re-run it any time to refresh the bootstrap file.
# govern is live-on-main: the bootstrap (and everything /govern fetches) tracks
# main, so there is no release-pinning knob.
set -eu

RAW="https://raw.githubusercontent.com/stonean/govern/main/framework/bootstrap/govern.md"

# Resolve the target agent: explicit arg > GOVERN_AGENT env > autodetect > claude.
agent="${1:-${GOVERN_AGENT:-}}"
if [ -z "$agent" ]; then
  matches=""
  [ -d .claude ] && matches="claude $matches"
  [ -d .augment ] && matches="auggie $matches"
  [ -d .agents ] && matches="antigravity $matches"
  # shellcheck disable=SC2086 # intentional word-split to count matches
  set -- $matches
  if [ "$#" -eq 1 ]; then agent="$1"; else agent="claude"; fi
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "govern: curl is required but was not found on PATH" >&2
  exit 1
fi

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
curl --proto '=https' --tlsv1.2 -fsSL "$RAW" > "$tmp"

case "$agent" in
  claude)
    dest=".claude/commands/govern.md"
    mkdir -p .claude/commands
    cp "$tmp" "$dest"
    ;;
  auggie)
    dest=".augment/commands/govern.md"
    mkdir -p .augment/commands
    cp "$tmp" "$dest"
    ;;
  antigravity | agy)
    agent="antigravity"
    dest=".agents/skills/govern/SKILL.md"
    mkdir -p .agents/skills/govern
    # Antigravity discovers dir-form skills: wrap govern.md's body in skill
    # frontmatter, dropping govern.md's own frontmatter (everything up to and
    # including the second `---`).
    {
      printf -- '---\nname: govern\n---\n'
      awk 'p{print} /^---[[:space:]]*$/{c++; if(c==2)p=1}' "$tmp"
    } > "$dest"
    ;;
  *)
    echo "govern: unknown agent '$agent' (expected: claude, auggie, antigravity)" >&2
    exit 1
    ;;
esac

echo "govern: installed the $agent bootstrap -> $dest"
echo "govern: now run '/govern <project-name>' in your agent to scaffold the project."
