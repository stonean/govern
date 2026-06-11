#!/bin/sh
# govern installer — places the /govern bootstrap command for your AI coding agent.
#
# Usage:
#   curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/stonean/govern/main/install.sh | sh
#
# Pick an agent explicitly (default: claude):
#   ... | sh -s -- claude
#   ... | sh -s -- auggie
#   ... | sh -s -- antigravity   # 'agy' (the Antigravity CLI name) also works
#
# The script is idempotent — re-run it any time to refresh the bootstrap file.
# govern is live-on-main: the bootstrap (and everything /govern fetches) tracks
# main, so there is no release-pinning knob.
set -eu

RAW="https://raw.githubusercontent.com/stonean/govern/main/framework/bootstrap/govern.md"

# Resolve the target agent: the optional positional argument, defaulting to claude.
agent="${1:-claude}"

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
    agent="antigravity"  # 'agy' is the Antigravity CLI command name
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
    echo "govern: unknown agent '$agent' (expected: claude, auggie, antigravity, or agy)" >&2
    exit 1
    ;;
esac

echo "govern: installed the $agent bootstrap -> $dest"
echo "govern: now run '/govern <project-name>' in your agent to scaffold the project."
