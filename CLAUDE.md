# Governance Repo Rules

## Govern File Parity

The `govern/` directory contains platform-specific govern files (e.g., `govern.md` for Claude Code, `govern-auggie.md` for Auggie). These files must stay in sync: any change to shared logic, file manifests, strategies, post-scaffolding output, or workflow in one govern file must be reflected in all other govern files. Platform-specific sections (permission setup, config directory paths, CLI config references) use platform-appropriate values but must follow the same structure.

When adding or modifying a govern file, review all other `govern/govern-*.md` files and `govern/govern.md` to ensure parity.

## Command File Parity

Commands exist in two locations: `commands/` (platform-agnostic templates) and `.claude/commands/gov/` (Claude Code instances). Any change to a command must be applied to both files. The `.claude/commands/gov/` versions use `/gov:` prefixes and `.claude/gov-session.json`; the `commands/` versions use `/{project}:` prefixes and `{cli-config-dir}/{project}-session.json`.
