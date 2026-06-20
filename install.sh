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
#   ... | sh -s -- opencode
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
    # Pre-seed permissions so the first /govern run does not prompt for its
    # bootstrap shell commands (see the antigravity arm for the rationale).
    # Written only when absent — /govern owns additive merges. Keep in sync with
    # the claude settings_template in framework/bootstrap/govern.md (§Agent Registry).
    if [ ! -f .claude/settings.local.json ]; then
      cat > .claude/settings.local.json <<'JSON'
{
  "permissions": {
    "allow": [
      "Bash(curl *)",
      "Bash(ls *)",
      "Bash(tar *)",
      "Bash(mktemp *)",
      "Bash(git status *)",
      "Bash(git config *)",
      "Bash(git rev-parse *)",
      "Bash(git diff *)",
      "Bash(git ls-files *)",
      "Bash(chmod *)",
      "Bash(awk *)",
      "Bash(command -v *)",
      "Read(/private/var/folders/**/T/govern-*/**)",
      "Read(//private/var/folders/**/T/govern-*/**)",
      "Read(/var/folders/**/T/govern-*/**)",
      "Read(//var/folders/**/T/govern-*/**)",
      "Read(/tmp/govern-*/**)",
      "Read(//tmp/govern-*/**)"
    ],
    "deny": []
  }
}
JSON
    fi
    ;;
  auggie)
    dest=".augment/commands/govern.md"
    mkdir -p .augment/commands
    cp "$tmp" "$dest"
    # Pre-seed permissions so the first /govern run does not prompt for its
    # bootstrap shell commands (see the antigravity arm for the rationale).
    # Written only when absent — /govern owns additive merges. Keep in sync with
    # the auggie settings_template in framework/bootstrap/govern.md (§Agent Registry).
    if [ ! -f .augment/settings.local.json ]; then
      cat > .augment/settings.local.json <<'JSON'
{
  "toolPermissions": [
    { "toolName": "launch-process", "shellInputRegex": "^curl ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^ls ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^tar ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^mktemp ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^git status ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^git config ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^git rev-parse ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^git diff ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^git ls-files ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^chmod ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^awk ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^command -v ", "permission": { "type": "allow" } }
  ]
}
JSON
    fi
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
    # Pre-seed the permission file so the first /govern run does not prompt for
    # its bootstrap shell commands. Antigravity loads permissions at session
    # start, so govern.md's in-run Permission Setup seed lands too late for the
    # first run. Written only when absent — /govern owns additive merges into an
    # existing settings.json. Keep this allow-list in sync with the antigravity
    # settings_template in framework/bootstrap/govern.md (§Agent Registry).
    if [ ! -f .agents/settings.json ]; then
      cat > .agents/settings.json <<'JSON'
{
  "permissions": {
    "allow": [
      "command(curl)",
      "command(ls)",
      "command(tar)",
      "command(mktemp)",
      "command(git status)",
      "command(git config)",
      "command(git rev-parse)",
      "command(git diff)",
      "command(git ls-files)",
      "command(chmod)",
      "command(awk)",
      "command(which)"
    ],
    "deny": [],
    "ask": []
  }
}
JSON
    fi
    ;;
  opencode)
    dest=".opencode/command/govern.md"
    mkdir -p .opencode/command
    cp "$tmp" "$dest"
    # Pre-seed permissions so the first /govern run does not prompt for its
    # bootstrap shell commands. OpenCode keeps both MCP wiring and permissions in
    # one committed opencode.json; this seeds only the permission block, and only
    # when neither opencode.json nor opencode.jsonc exists — /govern owns additive
    # merges. Keep in sync with the opencode settings_template in
    # framework/bootstrap/govern.md (§Agent Registry).
    if [ ! -f opencode.json ] && [ ! -f opencode.jsonc ]; then
      cat > opencode.json <<'JSON'
{
  "$schema": "https://opencode.ai/config.json",
  "permission": {
    "bash": {
      "curl *": "allow",
      "ls *": "allow",
      "tar *": "allow",
      "mktemp *": "allow",
      "git status *": "allow",
      "git config *": "allow",
      "git rev-parse *": "allow",
      "git diff *": "allow",
      "git ls-files *": "allow",
      "chmod *": "allow",
      "awk *": "allow",
      "command -v *": "allow"
    }
  }
}
JSON
    fi
    ;;
  *)
    echo "govern: unknown agent '$agent' (expected: claude, auggie, antigravity, agy, or opencode)" >&2
    exit 1
    ;;
esac

echo "govern: installed the $agent bootstrap -> $dest"
echo "govern: now run '/govern <project-name>' in your agent to scaffold the project."
