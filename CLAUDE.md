# Governance Repo Rules

## Command Source of Truth

All slash command templates live in `framework/commands/`. The Claude Code instances under `.claude/commands/gov/` are **generated** from those sources by `scripts/gen-claude-commands.sh`.

Never edit `.claude/commands/gov/*.md` directly — your changes will be overwritten the next time the generator runs. Edit the source under `framework/commands/`, then run:

```bash
./scripts/gen-claude-commands.sh
```

The generator substitutes `{project}` → `gov` and `{cli-config-dir}` → `.claude` and writes the agent-specific setup file (`framework/commands/setup/claude.md`) as `setup.md` in the gov command directory.

`.claude/commands/gov/init.md` is the one exception — it is governance-specific (no source counterpart) and is hand-maintained. The generator leaves it untouched.
