# Project: {project-name}

{One-line project description.}

> **Agents:** this file is the committed home for project rules — append durable learnings to the matching section (Gotchas, Workflow, Boundaries, Code Style, Testing). Add a new section only when none fits.

## Constitution

See [constitution.md](constitution.md) — guiding principles, development pipeline, spec lifecycle, and quality standards that govern this project.

## Tech Stack

<!-- Define your project's tech stack here. Example:

| Layer | Technology | Role |
| --- | --- | --- |
| **Language** | Go v1.26.0 | Application logic |
| **Database** | PostgreSQL v18 | Primary data store |

-->

## Commands

<!-- Define your project's common commands. Example:

- Dev: `make dev`
- Build: `make build`
- Test: `make test`
- Lint: `make lint`

-->

## Project Structure

- `constitution.md` -- Principles, pipeline, quality standards
- `AGENTS.md` -- Agent rules: tech stack, conventions, workflow, gotchas, boundaries
- `CLAUDE.md` -- `@AGENTS.md` + Claude Code-specific configuration
- `specs/`
  - `system.md` -- Architecture, shared conventions
  - `events.md` -- Global event catalog
  - `errors.md` -- Error handling conventions and codes
  - `inbox.md` -- Temporary inbox for known issues not yet assigned to a spec
  - `templates/` -- Starter files for spec, plan, tasks, data-model, research, scenario
  - `{NNN-feature-name}/`
    - `spec.md` -- Requirements, contracts, acceptance criteria
    - `research.md` -- *(optional)* Background research, prior art, context
    - `plan.md` -- Implementation approach, technical decisions
    - `data-model.md` -- *(optional)* Generated during plan phase
    - `tasks.md` -- Discrete work items derived from the plan
    - `scenarios/` -- Bug fixes, edge cases, detailed behavior elaborations
      - `{slug}.md` -- Individual scenario created via `/{project}:ask` (scenario route)

<!-- Add project-specific directories below (e.g., src/, cmd/, modules/) -->

## Skills

<!-- Optional. List skill files (Anthropic/Claude Code "skills" — context-loaded
     instruction packs) that augment AGENTS.md for specific task types. This is
     distinct from this repo's `framework/workflows/` — workflows are `govern`-
     scaffolded slash commands for routine ops (lint, test, format), while skills
     are an agent-platform feature for richer in-context instruction packs. Leave
     empty if you don't decompose into skills.

     Example (file paths are illustrative — adopters place skill files wherever
     their agent platform expects them):
     | Skill | Activates on |
     | --- | --- |
     | `code-review` skill | Code review on auth or session paths |
     | `db-migration` skill | Editing migration files |

     Per-platform mapping (Claude Code skills, Cursor rules, etc.) is the
     adopter's call — `govern` defines the index pattern, not the location.
-->

## Code Style

<!-- Define code patterns and conventions specific to your tech stack. -->

## Testing

<!-- Define testing conventions, test types, and file placement. -->

## Workflow

<!-- Process rules — pre-action checks, required ordering, lookup steps. "Always do X
     before Y" style. Distinct from Boundaries (hard "never" limits) and Gotchas
     (technical/framework quirks).
     Example:

- Before working on any spec beyond its spec.md, verify all dependency specs have status `done`. If any dependency is not done, work on the earliest incomplete dependency instead.
- Read `framework/commands/{name}.md` before recommending or describing a slash command — don't guess from the name.
- Run `make lint` after edits in `pkg/` before marking a task complete.

-->

## Gotchas

<!-- Document things agents consistently get wrong — framework quirks, version-specific behavior,
     and non-obvious conventions that waste cycles. Focus on what's surprising, not what's standard.
     Example:

- pgx v5 uses `pgx.RowToStructByName`, not the older `Scan` pattern
- templ components must not import `net/http` — pass values through parameters
- `air` does not watch `.sql` files by default — add the pattern to `.air.toml`
- NATS JetStream consumer names must be globally unique, not just per-stream

-->

## Boundaries

- Follow tasks.md literally — do not skip ahead to later pipeline phases. When tasks say to set status to `planned`, stop there. The user advances to the next phase explicitly.

<!-- Define additional project-specific boundaries. Common patterns:

- Never implement without a spec — follow the pipeline: spec → plan → tasks → implement
- Never commit secrets or .env files
- Never modify CI/CD config without asking
- Ask before adding new dependencies
- Ask before changing database schema

-->
