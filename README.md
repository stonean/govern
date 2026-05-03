# Governance

Standards and conventions for spec-driven software development. This project defines how we run projects — the workflow, spec structure, principles, and quality rules that apply regardless of tech stack.

## Contents

- [framework/](framework/) — Everything that ships to adopting projects
  - [constitution.md](framework/constitution.md) — Guiding principles, development pipeline, spec lifecycle, quality standards. Authoritative.
  - [framework/rules/](framework/rules/) — Domain rule sets adopted by reference
    - [security-backend.md](framework/rules/security-backend.md) — Enforceable backend security rules (RFC 2119)
    - [security-frontend.md](framework/rules/security-frontend.md) — Enforceable frontend security rules (RFC 2119)
  - [framework/templates/](framework/templates/) — Starter files customized per project, split by consumer
    - `templates/spec/` — Templates consumed during the pipeline (spec, plan, tasks, data-model, research, scenario, spec-and-plan)
    - `templates/project/` — Templates consumed during adoption (agents.md, claude-md.md, system.md, errors.md, events.md, project-readme.md, gitignore, inbox.md, initialize.md)
  - [framework/commands/](framework/commands/) — Slash command sources for the operational commands
  - [framework/workflows/](framework/workflows/) — Tech-stack-specific workflow files (lint, test, format, migrate) plus `registry.json` mapping stack selections to workflows
  - [framework/bootstrap/](framework/bootstrap/) — The `govern.md` installer plus per-agent permission files (`configure/{agent}.md`)
- [docs/introduction.md](docs/introduction.md) — Long-form pitch for spec-driven development. The constitution is authoritative for normative rules.
- [specs/](specs/) — Feature specs for governance itself (dogfooding the pipeline)
- [scripts/](scripts/) — Maintenance scripts (e.g., regenerate `.claude/commands/gov/` from `framework/commands/`)

## Feature Specs

Governance uses its own spec-driven pipeline to develop itself.

See [specs/README.md](specs/README.md) for cross-cutting decisions and deferred work.

| Spec | Status | Dependencies | Description |
| --- | --- | --- | --- |
| [000-slash-commands](specs/000-slash-commands/spec.md) | done | none | Generic slash command templates that operationalize the pipeline |
| [001-system-spec-templates](specs/001-system-spec-templates/spec.md) | done | none | Templates for system.md, errors.md, and events.md |
| [002-project-scaffolding](specs/002-project-scaffolding/spec.md) | done | 000, 001 | README, .gitignore, CLAUDE.md, and session file templates |
| [003-bootstrap-automation](specs/003-bootstrap-automation/spec.md) | done | 000, 001, 002 | Slash commands and /gov:init for scaffolding new projects |
| [004-tech-stack-selection](specs/004-tech-stack-selection/spec.md) | done | 003 | Interactive tech stack questionnaire during init that populates AGENTS.md |
| [005-workflows](specs/005-workflows/spec.md) | done | 004 | Recommend and scaffold development workflows (lint, test, format, migrate) based on tech stack during init |
| [006-bug-workflow](specs/006-bug-workflow/spec.md) | done | none | Scenario support, bug decision tree, and brownfield triage |
| [007-govern-workflow](specs/007-govern-workflow/spec.md) | done | 003 | Self-contained govern command to bootstrap and update governance in existing projects |
| [008-security-rules](specs/008-security-rules/spec.md) | done | 007 | Enforceable backend and frontend security rules distributed via `/govern` |
| [009-scenario-targeting](specs/009-scenario-targeting/spec.md) | done | 006 | Promote scenarios to first-class pipeline targets for question, clarify, status, and implement commands |
| [010-agent-autonomy](specs/010-agent-autonomy/spec.md) | done | 000 | Evaluate and adopt agent orchestration capabilities (skills, complexity routing, stuck detection, autonomy) |
| [011-brownfield-process](specs/011-brownfield-process/spec.md) | done | 006, 007 | Formalized process for initializing and incrementally building out specs in brownfield projects |
| [012-multi-agent-govern](specs/012-multi-agent-govern/spec.md) | done | 007 | Unified govern command with runtime agent selection — supports adopting multiple AI CLIs in one project and adding agents on re-run |
| [013-text-first-artifacts](specs/013-text-first-artifacts/spec.md) | done | 000, 007, 012 | Declare text-first artifacts principle, adopt YAML frontmatter for spec metadata, migrate adopted projects on next /govern |
| [014-reclarify-backedge](specs/014-reclarify-backedge/spec.md) | done | 000, 009, 013 | Wire up `/ask` to own the documented `clarified/planned/in-progress → draft` back-edge so questions surfacing mid-pipeline are captured and the spec's lifecycle invariant is maintained automatically |

## Adopting in an Existing Project

For brownfield projects, install the govern command and run it — no clone required. Once adopted, use `/capture` to initialize skeleton specs for existing features and let them gain precision incrementally through bug fixes, enhancements, and clarification.

Governance operates a **live-on-main** model — the snippets below fetch the latest from `main`. Tagged releases (`v0.1.0`, etc.) mark milestones for changelogs and release notes, not pinning targets. Adopters who want to lock individual files they've customized use `.governance.toml` (see [Pinning files with .governance.toml](#pinning-files-with-governancetoml) below).

### Claude Code

```bash
mkdir -p .claude/commands
curl -fsSL https://raw.githubusercontent.com/stonean/govern/main/framework/bootstrap/govern.md \
  > .claude/commands/govern.md
```

Then run `/govern {project-name}`.

### Auggie

```bash
mkdir -p .augment/commands
curl -fsSL https://raw.githubusercontent.com/stonean/govern/main/framework/bootstrap/govern.md \
  > .augment/commands/govern.md
```

Then run `/govern {project-name}`.

The command fetches governance files, scaffolds the spec directory, installs slash commands, and displays next steps. It is idempotent — safe to run again to pick up new governance files.

The same `govern.md` supports every CLI listed above. Use whichever curl snippet matches the agent you want to start with — adopting additional agents later does not require a second curl. Re-run `/govern --add-agent` from any adopted agent to pick up the others, and the unified file scaffolds them alongside the existing setup.

## Slash Commands

Adoption installs a full set of slash commands that operationalize the pipeline. All commands are verb-named and session-aware — use `/target` to switch to an existing feature; `/specify` and `/capture` create a new feature and set it as the session target automatically.

### Pipeline (advance state)

| Command | Purpose |
| --- | --- |
| `/specify` | Create a new feature spec — asks qualifying questions to choose standard or lightweight track |
| `/clarify` | Resolve open questions in the current spec, advance status to `clarified` |
| `/plan` | Create plan.md with technical decisions, affected files, and resolved questions |
| `/implement` | Work through tasks, update spec status to `in-progress` then `done` |
| `/validate` | Audit spec, plan, tasks, and scenarios for completeness and consistency. `--all` scans every feature. `--fix` auto-corrects fixable checkbox mismatches. Composable: `--all --fix` |

### Elaborate (add precision)

| Command | Purpose |
| --- | --- |
| `/ask` | Append an open question to the targeted spec or scenario for resolution during clarify |
| `/elaborate` | Add a scenario to elaborate a section of the targeted feature (bug fix, edge case, detailed behavior) |

### Brownfield (absorb existing reality)

| Command | Purpose |
| --- | --- |
| `/capture` | Initialize a skeleton spec from a freeform description of an existing feature |
| `/log` | Record a raw item to `specs/inbox.md` for later grooming |
| `/groom` | Walk `specs/inbox.md` and route each item to its proper spec or scenario via the bug decision tree |

### Orient

| Command | Purpose |
| --- | --- |
| `/target` | Set the working feature (or `feature/scenario`) for the session |
| `/status` | Dashboard showing all features' progress, or focused view of the current target |
| `/help` | Display project overview and slash command reference |

### Bootstrap (one-time per project)

| Command | Purpose |
| --- | --- |
| `/govern` | Adopt or update governance in an existing project (the installer that placed every other command) |
| `/configure` | Configure agent permissions for governance commands |
| `/spawn` | Spawn a new project from this one — copies specs, commands, configuration, and (if present) implementation code |

## Starting a New Project

The fastest path is `/govern` (see [Adopting in an Existing Project](#adopting-in-an-existing-project) above) — it scaffolds a working setup into a fresh repo. The manual steps below are listed for reference or for agents that don't yet support `/govern`.

### 1. Bootstrap project structure

```bash
mkdir my-project && cd my-project
git init
```

### 2. Copy governance files

Copy these files from governance into your project root:

| Source | Destination | Purpose |
| --- | --- | --- |
| `framework/constitution.md` | `constitution.md` | Principles, pipeline, spec lifecycle — customize the intro, keep the rest |
| `framework/templates/project/agents.md` | `AGENTS.md` | Agent rules template — fill in every section for your tech stack |
| `.markdownlint-cli2.jsonc` | `.markdownlint-cli2.jsonc` | Markdown linting config — use as-is |

### 3. Fill in AGENTS.md

Open `AGENTS.md` and replace every placeholder section:

- **Tech Stack** — list your languages, frameworks, databases, and versions
- **Commands** — define `dev`, `build`, `test`, `lint` (or your equivalents)
- **Project Structure** — map out your directory layout
- **Code Style** — show idiomatic patterns with code examples
- **Testing** — define test types, file placement, and tooling conventions
- **Gotchas** — document framework quirks and non-obvious behavior
- **Boundaries** — define what agents must never do without asking

### 4. Set up AI agent configuration

Create a `CLAUDE.md` (or equivalent for your agent) that imports the constitution and agent rules:

```text
@import constitution.md
@import AGENTS.md
```

### 5. Create the specs directory

```bash
mkdir specs
```

Write `specs/system.md` describing your architecture — server lifecycle, request flow, shared infrastructure, and module pattern. Add `specs/errors.md` and `specs/events.md` if your project uses structured errors or event-driven communication.

### 6. Add your first feature spec

Run `/specify` to create a numbered feature directory with a spec from template. The command asks qualifying questions to determine whether the feature uses the standard track (separate spec, plan, and tasks) or the lightweight track (combined spec-and-plan for small, single-module features).

Alternatively, create one manually:

```bash
mkdir specs/000-skeleton
cp /path/to/governance/framework/templates/spec/spec.md specs/000-skeleton/spec.md
```

### 7. Work through the pipeline

Follow the pipeline defined in `constitution.md`:

1. **Spec** — resolve all open questions, update status to `clarified`
2. **Plan** — run `/plan` to create plan.md (technical decisions, affected files) and tasks.md (ordered work items) in one step. If the feature involves persistence, also add data-model.md. Updates spec status to `planned`
3. **Implement** — follow the tasks list, update spec status to `in-progress`, then `done`

Run `/validate` any time to audit a feature's artifacts; it is not a pipeline gate, but it is the recommended check before starting `/implement`.

## Security Rules

The governance framework includes enforceable security rules for backend and frontend code, distributed via `/govern`. Rules use RFC 2119 language: **MUST/MUST NOT** are blocking violations, **SHOULD/SHOULD NOT** are advisory warnings.

- [framework/rules/security-backend.md](framework/rules/security-backend.md) — Authentication, authorization, input validation, data protection, API security, logging, dependency management, and error handling
- [framework/rules/security-frontend.md](framework/rules/security-frontend.md) — XSS prevention, CSRF protection, secure storage, authentication handling, content security, and dependency management

Projects can reference these rules in their AGENTS.md or validate command to enforce security standards during development.

## Templates Reference

Spec-pipeline templates (consumed by an agent during the pipeline):

| Template | When to use |
| --- | --- |
| [spec.md](framework/templates/spec/spec.md) | Starting a new feature — requirements, acceptance criteria, open questions |
| [spec-and-plan.md](framework/templates/spec/spec-and-plan.md) | Lightweight track — combined spec and plan for small, single-module features |
| [plan.md](framework/templates/spec/plan.md) | Planning phase — technical decisions, affected files, resolved questions |
| [tasks.md](framework/templates/spec/tasks.md) | Tasks phase — ordered work items derived from the plan |
| [data-model.md](framework/templates/spec/data-model.md) | Plan phase — when the feature involves database persistence |
| [research.md](framework/templates/spec/research.md) | Optional — background research, prior art, references |
| [scenario.md](framework/templates/spec/scenario.md) | Brownfield/elaborate workflow — scenario capturing specific behavior, edge case, or bug fix |

Project-scaffolding templates (consumed once at adoption):

| Template | Purpose |
| --- | --- |
| [agents.md](framework/templates/project/agents.md) | `AGENTS.md` — tech stack, conventions, code style, boundaries |
| [claude-md.md](framework/templates/project/claude-md.md) | `CLAUDE.md` — Claude Code-specific configuration |
| [project-readme.md](framework/templates/project/project-readme.md) | Starter project README |
| [system.md](framework/templates/project/system.md) | `specs/system.md` — architecture and shared conventions |
| [errors.md](framework/templates/project/errors.md) | `specs/errors.md` — error handling conventions |
| [events.md](framework/templates/project/events.md) | `specs/events.md` — global event catalog |
| [inbox.md](framework/templates/project/inbox.md) | `specs/inbox.md` — temporary inbox for known issues during brownfield adoption |
| [gitignore](framework/templates/project/gitignore) | Governance-related patterns merged into `.gitignore` |
| [initialize.md](framework/templates/project/initialize.md) | Hook for `/spawn` to do tech-stack-specific post-copy work |

## Bug Workflow

Bugs are unwritten scenarios. The governance framework treats every bug as evidence that a spec is missing, ambiguous, or violated.

### Decision tree

When a bug is reported, follow in order:

1. **No spec exists** — write the spec first, then fix the code
2. **Spec is ambiguous** — fix the spec, then fix the implementation
3. **Spec is clear, implementation is wrong** — add a scenario, then fix the code

### Scenarios

A scenario is a spec at a lower level of abstraction. Scenarios live in `specs/NNN-feature/scenarios/slug.md` and capture edge cases, bug fixes, and detailed behavior. Each scenario gets a linked task in the parent spec's `tasks.md`. Scenarios can be targeted directly with `/target feature/scenario-slug` for focused work.

### Inbox

For brownfield projects, `specs/inbox.md` is a temporary inbox. Items are recorded with `/log` and groomed into specs or scenarios with `/groom`. The goal is for the inbox to eventually be empty.

## Updating an Adopted Project

Projects that were bootstrapped with `/govern` or adopted governance manually can pull the latest governance files by running the govern command again. It uses three strategies to decide how each file is handled:

| Strategy | Behavior | Examples |
| --- | --- | --- |
| `update` | Always overwritten with the latest governance version | `constitution.md`, spec templates, slash commands |
| `create` | Created on first run, skipped on re-run | `specs/system.md`, `specs/errors.md`, initialize command |
| `skip` | Never overwritten | `AGENTS.md`, `CLAUDE.md` |

The `.gitignore` uses a `merge` strategy — governance patterns are appended below a `# Governance` marker if the marker is not already present.

Re-running `/govern` always pulls from `main` — the project does not pin to a specific tag. To track a tag instead, edit the `https://raw.githubusercontent.com/.../main/` URL inside your installed `govern.md` to point at the tag (e.g., `v0.1.0`). Most adopters won't need to — the per-file pinning below is finer-grained and usually the right tool.

### Pinning files with .governance.toml

If your project has customized a file that governance normally overwrites (`update` strategy), you can pin it to prevent `/govern` from overwriting your changes. Create a `.governance.toml` file in your project root:

```toml
[pinned]
# Files listed here use 'skip' instead of 'update'.
# Use destination paths (after placeholder resolution).
files = [
  ".claude/commands/myapp/implement.md",
  "constitution.md",
]
```

Any file listed in `pinned.files` is treated as `skip` instead of `update` when `/govern` runs. Pinned files are reported in the post-scaffolding summary.

### Manual updates

If you prefer not to use `/govern`, governance is a reference, not a dependency. Review the governance changelog or diff, decide which changes apply to your project, and update your copies at your own pace.

## Platform Support

Governance currently distributes to two AI coding agents:

- **Claude Code** — `.claude/` paths, `/govern` and `/gov:*` commands
- **Auggie** — `.augment/` paths, `/govern` command variant

Adding a new agent is a single registry row plus an agent-specific `framework/bootstrap/configure/{key}.md` permission file — see [framework/bootstrap/govern.md](framework/bootstrap/govern.md#agent-registry) for the full rules.

## Viewing artifacts

Governance artifacts are plain markdown with YAML frontmatter, so any markdown viewer or PKM tool can browse them. Pick whichever fits your workflow:

- **GitHub** — push `specs/` and browse inline; relative links resolve natively, no tooling required
- **[Obsidian](https://obsidian.md)** — point at the repo and open; graph view and backlinks out of the box
- **[Logseq](https://logseq.com)** — open-source PKM with a similar graph model
- **[Foam](https://foambubble.github.io/foam/)** — VS Code extension for markdown knowledge graphs
- **[Quartz](https://quartz.jzhao.xyz)** — publish a static graph-style site; see Quartz's docs for setup
- **[MkDocs](https://www.mkdocs.org)** — static documentation site generator
- Plain `cat`, GitHub PR review, or any markdown editor — no viewer required

The principle is that artifacts stay portable and source-of-truth markdown, with structured viewers as derived views (see [constitution.md](framework/constitution.md#text-first-artifacts)).

## Markdown

All `.md` files must pass `npx markdownlint-cli2` using the project config. See [constitution.md](framework/constitution.md#markdown-standards) for the full rule set.
