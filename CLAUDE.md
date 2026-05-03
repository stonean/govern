# `govern` Repo Rules

## Framework Layout

The `govern` framework is the everything-that-ships portion of this repo. It is laid out by IA primary purpose, not by file kind:

- `framework/constitution.md` — the law (singular, authoritative)
- `framework/rules/` — domain rule sets adopted projects can reference (security-backend, security-frontend, …)
- `framework/templates/spec/` — templates consumed by an agent during the pipeline (spec, plan, tasks, data-model, research, scenario, spec-and-plan)
- `framework/templates/project/` — templates consumed once at adoption (agents, claude-md, system, errors, events, project-readme, gitignore, inbox, initialize)
- `framework/commands/` — operational slash command sources only
- `framework/workflows/` — tech-stack-specific workflow files (lint, test, format) plus `registry.json` mapping stack selections to workflows
- `framework/bootstrap/` — the `govern.md` installer plus per-agent permission files at `bootstrap/configure/{key}.md`

When adding files, place them by purpose, not by extension.

## Command Source of Truth

All operational slash command templates live in `framework/commands/`. The agent-specific permission files live in `framework/bootstrap/configure/{key}.md`. The Claude Code instances under `.claude/commands/gov/` are **generated** from those sources by `scripts/gen-claude-commands.sh`.

Workflow files in `framework/workflows/` ship as-is — they are not generator inputs and have no `govern`-side `gov:workflows:*` counterpart. Adopting projects scaffold them via `/govern` (or `/gov:init`).

Never edit `.claude/commands/gov/*.md` directly — your changes will be overwritten the next time the generator runs. Edit the source under `framework/commands/` (or `framework/bootstrap/configure/claude.md` for the `configure` command), then run:

```bash
./scripts/gen-claude-commands.sh
```

The generator substitutes `{project}` → `gov` and `{cli-config-dir}` → `.claude` and writes the Claude-specific permission file (`framework/bootstrap/configure/claude.md`) as `configure.md` in the gov command directory.

`.claude/commands/gov/init.md` is the one exception — it is `govern`-specific (no source counterpart) and is hand-maintained. The generator leaves it untouched.
