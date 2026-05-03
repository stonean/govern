# {project}

{Brief description of what this project does.}

## Quick Start

```bash
# Add project-specific commands here
```

## Getting Started

1. Run `/{project}:configure` to configure permissions and tooling
2. Run `/{project}:status` to see the current state of all feature specs
3. Pick a spec and set it as your target: `/{project}:target 000`
4. Advance through the pipeline: `/{project}:clarify`, `/{project}:plan`, `/{project}:implement`

## Documentation

- [constitution.md](constitution.md) — Guiding principles, development pipeline, spec lifecycle, quality standards
- [AGENTS.md](AGENTS.md) — Tech stack, project structure, code style, conventions, and boundaries
- [specs/system.md](specs/system.md) — System architecture, request lifecycle, shared infrastructure
- [specs/errors.md](specs/errors.md) — Error handling conventions
- [specs/events.md](specs/events.md) — Global event catalog
- [specs/inbox.md](specs/inbox.md) — Temporary inbox for known issues during brownfield adoption
- [specs/templates/](specs/templates/) — Templates for spec, plan, tasks, data-model, research, scenario, and spec-and-plan documents

### Feature Specs

| Spec | Status | Dependencies | Description |
| --- | --- | --- | --- |

## Development Pipeline

{project} follows a spec-driven workflow. See [constitution.md](constitution.md#development-pipeline) for the full pipeline definition, spec lifecycle states, and readiness checks.

### Pipeline

```text
/{project}:specify ──▶ draft ──/{project}:clarify──▶ clarified ──/{project}:plan──▶ planned ──/{project}:implement──▶ in-progress ──▶ done
```

Each command enforces its pipeline gate — you cannot plan without a clarified spec, and you cannot implement without a plan. Two back-edges exist: `/{project}:ask` reverts a `clarified`, `planned`, or `in-progress` spec to `draft` when a new open question surfaces (the only state that tolerates open questions), and `/{project}:elaborate` reverts a `done` spec to `in-progress` when a new scenario is added. The next pipeline command resumes from there — the spec evolves rather than spawning a new one.

Three cycles are supported:

- **Greenfield** — `/{project}:specify` → clarify → plan → implement, aiming for completeness up front.
- **Brownfield** — `/{project}:capture` initializes a skeleton spec from what is known about an existing feature, then bug fixes and enhancements add precision over time.
- **Reopen** — `/{project}:elaborate` adds a scenario to a `done` spec, reverting it to `in-progress` until the scenario's task ships.

### Slash Commands

| Command | Purpose |
| --- | --- |
| `/{project}:help` | Overview of the pipeline and command usage |
| `/{project}:target` | Set the working feature for the session |
| `/{project}:status` | Dashboard of all specs and their pipeline state |
| `/{project}:specify` | Create a new feature spec from template |
| `/{project}:clarify` | Resolve open questions, advance `draft` → `clarified` |
| `/{project}:plan` | Create technical plan and tasks, advance `clarified` → `planned` |
| `/{project}:implement` | Walk through tasks step by step, advance `planned` → `done` |
| `/{project}:validate` | Check artifacts for consistency and cross-spec alignment |
| `/{project}:ask` | Append an open question to the targeted spec or scenario |
| `/{project}:elaborate` | Add a scenario to elaborate a section of the targeted feature |
| `/{project}:capture` | Initialize a skeleton spec for an existing feature |
| `/{project}:log` | Record a raw item to the inbox for later grooming |
| `/{project}:groom` | Walk the inbox and route each item to its proper spec or scenario |
| `/{project}:configure` | Configure permissions for common operations |
| `/{project}:spawn` | Spawn a new project from this one |
| `/govern` | Adopt or update `govern` in this project (top-level command, no project namespace) |

### Working on Existing Specs

1. Set the target: `/{project}:target 000`
2. Run the next pipeline command — the commands enforce ordering

When a spec's status or dependencies change, update the feature table above to match.
