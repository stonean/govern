---
description: Display an overview of the pipeline and its slash commands.
---

# Help

Display an overview of the pipeline and how to use its slash commands.

## Purpose

A static, at-a-glance guide to the pipeline: the states a spec moves through, the command for each transition, and the key concepts (session target, inbox, rules). Printed verbatim — it scans no files and runs no commands.

## Scope Boundaries

- Prints a fixed guide. Do NOT read spec files, list directories, run generators, or invoke any primitive — this command has no runtime path and no side effects.
- The command tables in the guide are generated from each command's frontmatter `description:` by `scripts/gen-help-tables.sh` (kept in sync by the pre-commit hook and `/{project}:audit`), not assembled at print time.

## Instructions

Print the following guide exactly (do not scan files or run commands):

---

## {project} — Spec-Driven Development Pipeline

{project} is a set of slash commands that guide features from idea to implementation through a structured pipeline.

### Pipeline States

```text
draft → clarified → planned → in-progress → done
```

Two back-edges keep the lifecycle honest:

- `/{project}:amend` reverts a `clarified`, `planned`, or `in-progress` spec to `draft` when a new open question surfaces — `draft` is the only status that tolerates open questions. The next `/{project}:clarify` resolves the question and the spec advances forward again.
- `/{project}:amend` reverts a `done` spec to `in-progress` when a new scenario is added (the scenario route) — the scenario captures the change, the spec evolves with it.

Each feature lives in `specs/NNN-feature-name/` and progresses through these states by running the corresponding command.

### Commands

#### Pipeline (advance state)

<!-- generated:commands-pipeline:start -->

| Command | Pipeline Gate | Description |
| --- | --- | --- |
| `/{project}:specify` | → draft | Create a new feature spec. |
| `/{project}:clarify` | draft → clarified | Resolve open questions and advance a spec from draft to clarified. |
| `/{project}:plan` | clarified → planned | Create a technical plan and task breakdown for a clarified spec. |
| `/{project}:implement` | planned → in-progress → done | Execute implementation tasks for the targeted feature. |
| `/{project}:review` | blocks `done` (MUST violations) | Audit code against rules — security, reuse, quality, efficiency, simplicity. Writes review.md; blocks done on MUST violations. |
| `/{project}:analyze` | — | Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. Read-only by default; --fix reverts a drifted done spec. |

<!-- generated:commands-pipeline:end -->

#### Refine

<!-- generated:commands-refine:start -->

| Command | Description |
| --- | --- |
| `/{project}:amend` | Add a question or a scenario to the targeted spec (classifier-driven). |
| `/{project}:prune` | Prune a feature's tasks.md — drop spent task sections, or reset to template state. |

<!-- generated:commands-refine:end -->

#### Brownfield (absorb existing reality)

<!-- generated:commands-brownfield:start -->

| Command | Description |
| --- | --- |
| `/{project}:log` | Record a raw item to the inbox. |
| `/{project}:groom` | Walk the inbox and route each item to its proper home. |

<!-- generated:commands-brownfield:end -->

#### Orient

<!-- generated:commands-orient:start -->

| Command | Description |
| --- | --- |
| `/{project}:target` | Set the working feature (and optionally scenario) for this session. |
| `/{project}:link` | Register a service so cross-service references resolve to its lifecycle status. |
| `/{project}:status` | Display the pipeline view for all feature specs. |
| `/{project}:help` | Display an overview of the pipeline and its slash commands. |

<!-- generated:commands-orient:end -->

#### Bootstrap (one-time per project)

<!-- generated:commands-bootstrap:start -->

| Command | Description |
| --- | --- |
| `/govern` | Adopt or update govern in an existing project. |
| `/{project}:configure` | Configure settings.local.json with permissions for slash commands. |

<!-- generated:commands-bootstrap:end -->

### Typical Session

```text
/{project}:configure                 # first time only
/{project}:status                    # see where everything stands
/{project}:target 000                # pick a feature to work on
/{project}:clarify                   # resolve open questions
/{project}:plan                      # generate implementation plan
/{project}:implement                 # write the code
```

### Key Concepts

- **Session target** — The feature you're currently working on, stored in `.govern/session.toml`. Most commands operate on the target by default.
- **Dependencies** — Features declare dependencies in their spec. A feature is blocked until its dependencies reach `clarified` or later.
- **Artifacts** — Each feature directory can contain `spec.md`, `plan.md`, `tasks.md`, `data-model.md`, and a `scenarios/` subdirectory.
- **Scenarios** — A scenario is a spec at a lower level of abstraction. Scenarios live in `specs/NNN-feature/scenarios/slug.md` and capture bugs, edge cases, and detailed behavior. Each scenario gets a linked task in `tasks.md`.
- **Bug decision tree** — When a bug is reported: (1) no spec → write the spec first, (2) spec is ambiguous → fix the spec, (3) spec is clear → add a scenario.
- **Inbox** — `specs/inbox.md` is a temporary inbox for known issues. Items are recorded with `/{project}:log` and groomed into specs or scenarios with `/{project}:groom`.
- **Finish before moving on** — Prefer completing a feature through the full pipeline before starting the next. Depth-first keeps context focused.

---
