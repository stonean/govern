# {project}

{Brief description of what this project does.}

## Quick Start

```bash
# Add project-specific commands here
```

## Getting Started

1. Run `/{project}:setup` to configure permissions and tooling
2. Run `/{project}:status` to see the current state of all feature specs
3. Pick a spec and set it as your target: `/{project}:target 000`
4. Advance through the pipeline: `/{project}:clarify`, `/{project}:plan`, `/{project}:implement`

## Documentation

- [constitution.md](constitution.md) — Guiding principles, development pipeline, spec lifecycle, quality standards
- [AGENTS.md](AGENTS.md) — Tech stack, project structure, code style, conventions, and boundaries
- [specs/system.md](specs/system.md) — System architecture, request lifecycle, shared infrastructure
- [specs/errors.md](specs/errors.md) — Error handling conventions
- [specs/events.md](specs/events.md) — Global event catalog
- [specs/templates/](specs/templates/) — Templates for spec, plan, and tasks documents

### Feature Specs

| Spec | Status | Dependencies | Description |
| --- | --- | --- | --- |

## Development Pipeline

{project} follows a spec-driven workflow: **spec → clarify → plan → implement**. See [constitution.md](constitution.md#development-pipeline) for the full pipeline definition, spec lifecycle states, and readiness checks.

### Pipeline

```text
/{project}:specify  →  /{project}:clarify  →  /{project}:plan  →  /{project}:implement
   (draft)               (clarified)            (planned)            (done)
```

Each command enforces its pipeline gate — you cannot plan without a clarified spec, and you cannot implement without a plan.

### Slash Commands

| Command | Purpose |
| --- | --- |
| `/{project}:about` | Overview of the pipeline and command usage |
| `/{project}:target` | Set the working feature for the session |
| `/{project}:status` | Dashboard of all specs and their pipeline state |
| `/{project}:specify` | Create a new feature spec from template |
| `/{project}:clarify` | Resolve open questions, advance `draft` → `clarified` |
| `/{project}:plan` | Create technical plan and tasks, advance `clarified` → `planned` |
| `/{project}:implement` | Walk through tasks step by step, advance `planned` → `done` |
| `/{project}:validate` | Check artifacts for consistency and cross-spec alignment |
| `/{project}:setup` | Configure permissions for common operations |

### Working on Existing Specs

1. Set the target: `/{project}:target 000`
2. Run the next pipeline command — the commands enforce ordering

When a spec's status or dependencies change, update the feature table above to match.
