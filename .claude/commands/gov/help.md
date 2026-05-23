---
description: Display an overview of the pipeline and its slash commands.
---

# Help

Display an overview of the pipeline and how to use its slash commands.

## Instructions

Print the following guide exactly (do not scan files or run commands):

---

## gov — Spec-Driven Development Pipeline

gov is a set of slash commands that guide features from idea to implementation through a structured pipeline.

### Pipeline States

```text
draft → clarified → planned → in-progress → done
```

Two back-edges keep the lifecycle honest:

- `/gov:ask` reverts a `clarified`, `planned`, or `in-progress` spec to `draft` when a new open question surfaces — `draft` is the only status that tolerates open questions. The next `/gov:clarify` resolves the question and the spec advances forward again.
- `/gov:ask` reverts a `done` spec to `in-progress` when a new scenario is added (the scenario route) — the scenario captures the change, the spec evolves with it.

Each feature lives in `specs/NNN-feature-name/` and progresses through these states by running the corresponding command.

### Commands

#### Pipeline (advance state)

<!-- generated:commands-pipeline:start -->

| Command | Pipeline Gate | Description |
| --- | --- | --- |
| `/gov:specify` | → draft | Create a new feature spec. |
| `/gov:clarify` | draft → clarified | Resolve open questions and advance a spec from draft to clarified. |
| `/gov:plan` | clarified → planned | Create a technical plan and task breakdown for a clarified spec. |
| `/gov:implement` | planned → in-progress → done | Execute implementation tasks for the targeted feature. |
| `/gov:review` | blocks `done` (MUST violations) | Audit code against rules — security, reuse, quality, efficiency, simplicity. Writes review.md; blocks done on MUST violations. |
| `/gov:analyze` | — | Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. Read-only. |

<!-- generated:commands-pipeline:end -->

#### Refine

<!-- generated:commands-refine:start -->

| Command | Description |
| --- | --- |
| `/gov:ask` | Add a question or a scenario to the targeted spec (classifier-driven). |

<!-- generated:commands-refine:end -->

#### Brownfield (absorb existing reality)

<!-- generated:commands-brownfield:start -->

| Command | Description |
| --- | --- |
| `/gov:log` | Record a raw item to the inbox. |
| `/gov:groom` | Walk the inbox and route each item to its proper home. |

<!-- generated:commands-brownfield:end -->

#### Orient

<!-- generated:commands-orient:start -->

| Command | Description |
| --- | --- |
| `/gov:target` | Set the working feature (and optionally scenario) for this session. |
| `/gov:status` | Display the pipeline view for all feature specs. |
| `/gov:help` | Display an overview of the pipeline and its slash commands. |

<!-- generated:commands-orient:end -->

#### Bootstrap (one-time per project)

<!-- generated:commands-bootstrap:start -->

| Command | Description |
| --- | --- |
| `/govern` | Adopt or update govern in an existing project. |
| `/gov:configure` | Configure settings.local.json with permissions for slash commands. |

<!-- generated:commands-bootstrap:end -->

### Typical Session

```text
/gov:configure                 # first time only
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
- **Inbox** — `specs/inbox.md` is a temporary inbox for known issues. Items are recorded with `/gov:log` and groomed into specs or scenarios with `/gov:groom`.
- **Finish before moving on** — Prefer completing a feature through the full pipeline before starting the next. Depth-first keeps context focused.

---
