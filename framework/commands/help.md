---
description: Display an overview of the pipeline and its slash commands.
---

# Help

Display an overview of the pipeline and how to use its slash commands.

## Instructions

Print the following guide exactly (do not scan files or run commands):

---

## {project} — Spec-Driven Development Pipeline

{project} is a set of slash commands that guide features from idea to implementation through a structured pipeline.

### Pipeline States

```text
draft → clarified → planned → in-progress → done
```

A `done` spec re-enters the pipeline at `in-progress` when a new scenario is added — the scenario captures the change, the spec evolves with it.

Each feature lives in `specs/NNN-feature-name/` and progresses through these states by running the corresponding command.

### Commands

#### Pipeline (advance state)

| Command | Pipeline Gate | Description |
| --- | --- | --- |
| `/{project}:specify` | → draft | Create a new numbered feature spec. Pass a short description, e.g. `/{project}:specify webhook delivery`. |
| `/{project}:clarify` | draft → clarified | Resolve open questions in the spec. Works on the session target or pass a feature identifier. |
| `/{project}:plan` | clarified → planned | Generate `plan.md` and `tasks.md` with implementation details. |
| `/{project}:implement` | planned → in-progress → done | Execute the tasks for the targeted feature. |
| `/{project}:validate` | — | Audit artifacts for consistency, completeness, and cross-spec alignment. |

#### Elaborate (add precision)

| Command | Description |
| --- | --- |
| `/{project}:ask` | Append an open question to the targeted spec or scenario for resolution during clarify. |
| `/{project}:elaborate` | Create a scenario file for the targeted feature. Walks the bug decision tree, creates the file in `scenarios/`, and appends a task to `tasks.md`. |

#### Brownfield (absorb existing reality)

| Command | Description |
| --- | --- |
| `/{project}:capture` | Initialize a skeleton spec from a freeform description of an existing feature. |
| `/{project}:log` | Record a raw item to `specs/inbox.md` for later grooming. |
| `/{project}:groom` | Walk `specs/inbox.md` and route each item to its proper spec or scenario via the bug decision tree. |

#### Orient

| Command | Description |
| --- | --- |
| `/{project}:target` | Set the working feature (or feature/scenario) for the session. Pass a number (`001`), partial name (`api-versioning`), or full directory name. |
| `/{project}:status` | Dashboard showing every feature's progress, dependencies, artifacts, and blockers. |
| `/{project}:help` | This overview. |

#### Bootstrap (one-time per project)

| Command | Description |
| --- | --- |
| `/govern` | Adopt or update governance in an existing project. Installed in your agent's command directory; no project namespace. |
| `/{project}:configure` | Configure `{cli-config-dir}/settings.local.json` so commands run without manual approval prompts. |
| `/{project}:spawn` | Spawn a new project from this one — copies specs, commands, and configuration. |

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

- **Session target** — The feature you're currently working on, stored in `{cli-config-dir}/{project}-session.json`. Most commands operate on the target by default.
- **Dependencies** — Features declare dependencies in their spec. A feature is blocked until its dependencies reach `clarified` or later.
- **Artifacts** — Each feature directory can contain `spec.md`, `plan.md`, `tasks.md`, `data-model.md`, and a `scenarios/` subdirectory.
- **Scenarios** — A scenario is a spec at a lower level of abstraction. Scenarios live in `specs/NNN-feature/scenarios/slug.md` and capture bugs, edge cases, and detailed behavior. Each scenario gets a linked task in `tasks.md`.
- **Bug decision tree** — When a bug is reported: (1) no spec → write the spec first, (2) spec is ambiguous → fix the spec, (3) spec is clear → add a scenario.
- **Inbox** — `specs/inbox.md` is a temporary inbox for known issues. Items are recorded with `/{project}:log` and groomed into specs or scenarios with `/{project}:groom`.
- **Finish before moving on** — Prefer completing a feature through the full pipeline before starting the next. Depth-first keeps context focused.

---
