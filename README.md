# Governance

Standards and conventions for spec-driven software development. This project defines how we run projects — the workflow, spec structure, principles, and quality rules that apply regardless of tech stack.

## Contents

- [constitution.md](constitution.md) — Guiding principles, development pipeline, spec lifecycle, and quality standards
- [templates/](templates/) — Starter files for specs, plans, tasks, data models, and research
- [AGENTS.md](AGENTS.md) — Agent rules template for AI-assisted development
- [specs/](specs/) — Feature specs for governance itself (dogfooding the pipeline)

## Feature Specs

Governance uses its own spec-driven pipeline to develop itself.

| Spec | Status | Dependencies | Description |
| --- | --- | --- | --- |
| [000-slash-commands](specs/000-slash-commands/spec.md) | done | none | Generic slash command templates that operationalize the pipeline |
| [001-system-spec-templates](specs/001-system-spec-templates/spec.md) | done | none | Templates for system.md, errors.md, and events.md |
| [002-project-scaffolding](specs/002-project-scaffolding/spec.md) | done | 000, 001 | README, .gitignore, CLAUDE.md, and session file templates |
| [003-bootstrap-automation](specs/003-bootstrap-automation/spec.md) | done | 000, 001, 002 | Slash commands and /gov:init for scaffolding new projects |
| [005-skills-and-plugins](specs/005-skills-and-plugins/spec.md) | draft | 004 | Recommend and scaffold skills/plugins based on tech stack during init |
| [006-bug-workflow](specs/006-bug-workflow/spec.md) | in-progress | none | Scenario support, bug decision tree, and brownfield triage |
| [007-adopt-workflow](specs/007-adopt-workflow/spec.md) | done | 003 | Self-contained govern command to bootstrap governance in existing projects |
| [008-security-rules](specs/008-security-rules/spec.md) | draft | 007 | Enforceable backend and frontend security rules distributed via adopt |
| [009-scenario-targeting](specs/009-scenario-targeting/spec.md) | draft | 006 | Promote scenarios to first-class pipeline targets for question, clarify, status, and implement commands |
| [010-agent-autonomy](specs/010-agent-autonomy/spec.md) | draft | 000 | Evaluate and adopt agent orchestration capabilities (skills, complexity routing, stuck detection, autonomy) |

## Starting a New Project

### 1. Bootstrap project structure

```bash
mkdir my-project && cd my-project
git init
```

### 2. Copy governance files

Copy these files from governance into your project root:

| File | Purpose |
| --- | --- |
| `constitution.md` | Principles, pipeline, spec lifecycle — customize the intro, keep the rest |
| `AGENTS.md` | Agent rules template — fill in every section for your tech stack |
| `.markdownlint-cli2.jsonc` | Markdown linting config — use as-is |

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

Copy `templates/spec.md` into a numbered feature directory:

```bash
mkdir specs/000-skeleton
cp /path/to/governance/templates/spec.md specs/000-skeleton/spec.md
```

Fill in the spec: describe what the feature does, define acceptance criteria, and list open questions. Set status to `draft`.

### 7. Work through the pipeline

Follow the pipeline defined in `constitution.md`:

1. **Spec** — resolve all open questions, update status to `clarified`
2. **Plan** — copy `templates/plan.md`, make technical decisions, list affected files. If the feature involves persistence, copy `templates/data-model.md`
3. **Tasks** — copy `templates/tasks.md`, break the plan into ordered work items. Update spec status to `planned`
4. **Readiness check** — verify all gates pass before writing code
5. **Implement** — follow the tasks list, update spec status to `in-progress`, then `done`

## Adopting in an Existing Project

For brownfield projects, install the govern command and run it — no clone required.

### Claude Code

```bash
curl -fsSL https://raw.githubusercontent.com/stonean/govern/main/govern/govern.md \
  > .claude/commands/govern.md
```

Then run `/govern {project-name}`.

### Auggie

```bash
curl -fsSL https://raw.githubusercontent.com/stonean/govern/main/govern/govern-auggie.md \
  > .augment/commands/govern.md
```

Then run `/govern {project-name}`.

The command fetches governance files, scaffolds the spec directory, installs slash commands, and displays next steps. It is idempotent — safe to run again to pick up new governance files.

## Templates Reference

| Template | When to use |
| --- | --- |
| [spec.md](templates/spec.md) | Starting a new feature — requirements, acceptance criteria, open questions |
| [spec-and-plan.md](templates/spec-and-plan.md) | Lightweight track — combined spec and plan for small, single-module features |
| [plan.md](templates/plan.md) | Planning phase — technical decisions, affected files, resolved questions |
| [tasks.md](templates/tasks.md) | Tasks phase — ordered work items derived from the plan |
| [data-model.md](templates/data-model.md) | Plan phase — when the feature involves database persistence |
| [research.md](templates/research.md) | Optional — background research, prior art, references |
| [scenario.md](templates/scenario.md) | Bug workflow — scenario capturing specific behavior, edge case, or bug fix |
| [triage.md](templates/triage.md) | Bug workflow — temporary inbox for known issues during brownfield adoption |

## Bug Workflow

Bugs are unwritten scenarios. The governance framework treats every bug as evidence that a spec is missing, ambiguous, or violated.

### Decision tree

When a bug is reported, follow in order:

1. **No spec exists** — write the spec first, then fix the code
2. **Spec is ambiguous** — fix the spec, then fix the implementation
3. **Spec is clear, implementation is wrong** — add a scenario, then fix the code

### Scenarios

A scenario is a spec at a lower level of abstraction. Scenarios live in `specs/NNN-feature/scenarios/slug.md` and capture edge cases, bug fixes, and detailed behavior. Each scenario gets a linked task in the parent spec's `tasks.md`.

### Triage

For brownfield projects, `specs/triage.md` is a temporary inbox. Items are migrated to specs or scenarios as the project adopts governance. The goal is for triage to eventually be empty.

## Updating an Adopted Project

Projects that were bootstrapped with `/gov:init` or adopted governance manually can pull the latest governance files by running the govern command. It uses three strategies to decide how each file is handled:

| Strategy | Behavior | Examples |
| --- | --- | --- |
| `update` | Always overwritten with the latest governance version | `constitution.md`, spec templates, slash commands |
| `create` | Created on first run, skipped on re-run | `specs/system.md`, `specs/errors.md`, initialize command |
| `skip` | Never overwritten | `AGENTS.md`, `CLAUDE.md` |

The `.gitignore` uses a `merge` strategy — governance patterns are appended below a `# Governance` marker if the marker is not already present.

### Pinning files with .governance.toml

If your project has customized a file that governance normally overwrites (`update` strategy), you can pin it to prevent adopt from overwriting your changes. Create a `.governance.toml` file in your project root:

```toml
[pinned]
# Files listed here use 'skip' instead of 'update'.
# Use destination paths (after placeholder resolution).
files = [
  ".claude/commands/myapp/implement.md",
  "constitution.md",
]
```

Any file listed in `pinned.files` is treated as `skip` instead of `update` when adopt runs. Pinned files are reported in the post-scaffolding summary.

### Manual updates

If you prefer not to use adopt, governance is a reference, not a dependency. Review the governance changelog or diff, decide which changes apply to your project, and update your copies at your own pace.

## Markdown

All `.md` files must pass `markdownlint-cli2` using the project config. See [constitution.md](constitution.md#markdown-standards) for the full rule set.
