# `govern`

The `govern` framework — a pipeline-driven spec-to-implementation flow scaffolded into adopted projects.

> **Agents:** this file is the committed home for project rules — append durable learnings to the matching section (Gotchas, Workflow, Boundaries, Code Style, Testing). Add a new section only when none fits.

## Constitution

See [constitution.md](constitution.md) — guiding principles, development pipeline, spec lifecycle, and quality standards that govern this project.

## Project Structure

The `framework/` directory is govern's source — everything that ships to adopted projects via `/govern`. It is laid out by IA primary purpose, not by file kind:

- `framework/constitution.md` — the constitution shipped to adopted projects (sync target of root `constitution.md`)
- `framework/rules/` — domain rule sets adopted projects can reference (security-backend, security-frontend, …)
- `framework/templates/spec/` — templates consumed by an agent during the pipeline (spec, plan, tasks, data-model, research, scenario, spec-and-plan)
- `framework/templates/project/` — project document templates consumed once at adoption (agents, claude-md, system, errors, events, project-readme, gitignore, inbox)
- `framework/templates/commands/` — slash command stubs scaffolded once at adoption (initialize)
- `framework/commands/` — operational slash command sources only
- `framework/workflows/` — tech-stack-specific workflow files (lint, test, format) plus `registry.json` mapping stack selections to workflows
- `framework/bootstrap/` — the `govern.md` installer plus per-agent permission files at `bootstrap/configure/{key}.md`

When adding files under `framework/`, place them by purpose, not by extension.

## Workflow

- `constitution.md` (root, what govern lives by) and `framework/constitution.md` (what ships to adopted projects) are intentionally separate but start identical. After editing either, mirror to the other:
  - Update to `framework/constitution.md` → also update root `constitution.md` (unless the change is template-only).
  - Update to root `constitution.md` → also update `framework/constitution.md` (unless the change is govern-internal).

  Note deliberate divergence in the commit message.
- Read `framework/commands/{name}.md` before recommending, describing, or disambiguating a slash command — don't guess from the name. Source files are authoritative; the generated `.claude/commands/gov/*.md` copies are not.
- After editing any file under `framework/commands/` or `framework/bootstrap/configure/claude.md`, run `./scripts/gen-claude-commands.sh` to regenerate `.claude/commands/gov/*.md`.

## Gotchas

- Use `npx markdownlint-cli2` to run markdown linting — do not suggest installing it globally.
- The command generator substitutes `{project}` → `gov` and `{cli-config-dir}` → `.claude`, and writes the Claude-specific permission file (`framework/bootstrap/configure/claude.md`) as `configure.md` in the gov command directory.
- `framework/workflows/` files ship as-is — they are not generator inputs and have no `govern`-side `gov:workflows:*` counterpart. Adopting projects scaffold them via `/govern` (or `/gov:init`).
- `.claude/commands/gov/init.md` is the one exception to the generator rule — it is `govern`-specific (no source counterpart) and is hand-maintained. The generator leaves it untouched.

## Boundaries

- Never edit `.claude/commands/gov/*.md` directly — your changes will be overwritten the next time the generator runs. Edit the source under `framework/commands/` (or `framework/bootstrap/configure/claude.md` for the `configure` command).
