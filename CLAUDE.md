# Governance Repo Rules

## Command File Parity

Commands exist in two locations: `commands/` (platform-agnostic templates) and `.claude/commands/gov/` (Claude Code instances). Any change to a command must be applied to both files. The `.claude/commands/gov/` versions use `/gov:` prefixes and `.claude/gov-session.json`; the `commands/` versions use `/{project}:` prefixes and `{cli-config-dir}/{project}-session.json`.
