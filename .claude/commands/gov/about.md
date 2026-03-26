# About gov

Display an overview of the pipeline and how to use its slash commands.

## Instructions

Print the following guide exactly (do not scan files or run commands):

---

## gov — Spec-Driven Development Pipeline

gov is a set of slash commands that guide features from idea to implementation through a structured pipeline.

### Pipeline Stages

```text
draft → clarified → planned → in-progress → done
```

Each feature lives in `specs/NNN-feature-name/` and progresses through these stages by running the corresponding command.

### Commands

#### Workflow (run in order)

| Command | Stage Transition | Description |
| --- | --- | --- |
| `/gov:specify` | → draft | Create a new numbered feature spec. Pass a short description, e.g. `/gov:specify webhook delivery`. |
| `/gov:clarify` | draft → clarified | Resolve open questions in the spec. Works on the session target or pass a feature identifier. |
| `/gov:plan` | clarified → planned | Generate `plan.md` and `tasks.md` with implementation details. |
| `/gov:implement` | planned → in-progress → done | Execute the tasks for the targeted feature. |

#### Bug Workflow

| Command | Description |
| --- | --- |
| `/gov:scenario` | Create a scenario file for the targeted feature. Walks the bug decision tree, creates the file in `scenarios/`, and appends a task to `tasks.md`. |
| `/gov:inbox` | Review `specs/inbox.md` and migrate items to the appropriate spec or scenario. |

#### Utilities

| Command | Description |
| --- | --- |
| `/gov:target` | Set the active feature for the session. Pass a number (`001`), partial name (`api-versioning`), or full directory name. |
| `/gov:status` | Dashboard showing every feature's progress, dependencies, artifacts, and blockers. |
| `/gov:validate` | Audit artifacts for consistency, completeness, and cross-spec alignment. |
| `/gov:setup` | Configure `.claude/settings.local.json` so commands run without manual approval prompts. |

### Typical Session

```text
/gov:setup                     # first time only
/gov:status                    # see where everything stands
/gov:target 000                # pick a feature to work on
/gov:clarify                   # resolve open questions
/gov:plan                      # generate implementation plan
/gov:implement                 # write the code
```

### Key Concepts

- **Session target** — The feature you're currently working on, stored in `.claude/gov-session.json`. Most commands operate on the target by default.
- **Dependencies** — Features declare dependencies in their spec. A feature is blocked until its dependencies reach `clarified` or later.
- **Artifacts** — Each feature directory can contain `spec.md`, `plan.md`, `tasks.md`, `data-model.md`, and a `scenarios/` subdirectory.
- **Scenarios** — A scenario is a spec at a lower level of abstraction. Scenarios live in `specs/NNN-feature/scenarios/slug.md` and capture bugs, edge cases, and detailed behavior. Each scenario gets a linked task in `tasks.md`.
- **Bug decision tree** — When a bug is reported: (1) no spec → write the spec first, (2) spec is ambiguous → fix the spec, (3) spec is clear → add a scenario.
- **Inbox** — `specs/inbox.md` is a temporary inbox for known issues. Items are migrated to specs or scenarios as the project adopts governance.
- **Finish before moving on** — Prefer completing a feature through the full pipeline before starting the next. Depth-first keeps context focused.

---
