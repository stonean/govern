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
- `framework/commands/` — operational slash command sources only
- `framework/workflows/` — tech-stack-specific workflow files (lint, test, format) plus `registry.json` mapping stack selections to workflows
- `framework/bootstrap/` — the `govern.md` installer plus per-agent permission files at `bootstrap/configure/{key}.md`

When adding files under `framework/`, place them by purpose, not by extension.

## Workflow

- Read `framework/commands/{name}.md` before recommending, describing, or disambiguating a slash command — don't guess from the name. Source files are authoritative; the generated `.claude/commands/gov/*.md` copies are produced by the pre-commit hook.

## Gotchas

- Use `npx markdownlint-cli2` to run markdown linting — do not suggest installing it globally.
- The command generator substitutes `{project}` → `gov` and `{cli-config-dir}` → `.claude`, and writes the Claude-specific permission file (`framework/bootstrap/configure/claude.md`) as `configure.md` in the gov command directory.
- `framework/workflows/` files ship as-is — they are not generator inputs and have no `govern`-side `gov:workflows:*` counterpart. Adopting projects scaffold them via `/govern` (or `/gov:init`).
- `.claude/commands/gov/init.md` is the one exception to the generator rule — it is `govern`-specific (no source counterpart) and is hand-maintained. The generator leaves it untouched.

## Boundaries

- Never edit `.claude/commands/gov/*.md` directly — your changes will be overwritten the next time the generator runs. Edit the source under `framework/commands/` (or `framework/bootstrap/configure/claude.md` for the `configure` command).

## Design Principles

- **Never design framework features that depend on human diligence or discipline.** Any artifact section, frontmatter field, command behavior, or workflow step that requires an author to *remember* to fill it in, set a flag, update a doc alongside code, or otherwise be careful will fail in practice — silently and asymmetrically (the cases where it gets skipped are exactly the cases where it mattered most). When proposing a new input, ask "what happens when an author forgets?" If the answer is "the feature degrades silently," redesign the input as **derived** (extracted from existing artifacts, frontmatter, git history, code analysis) or don't ship it. Reason: surfaced 2026-05-06 when evaluating an optional `## Upgrade Impact` spec section as a way to capture cross-version migration notes; rejected on this principle and the topic was tabled to inbox until a derivable design is found. How to apply: this is a hard filter on framework proposals, not a tiebreaker — if the only viable design relies on author discipline, the right answer is to defer the feature, not to ship the disciplined version "for now."
