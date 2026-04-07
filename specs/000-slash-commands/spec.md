# 000 — Slash Command Templates

**Status:** done
**Dependencies:** none

Generic, project-agnostic slash command templates that operationalize the governance development pipeline. Projects copy these commands into their `.claude/commands/{project}/` directory and customize the project name and any project-specific paths.

## Problem

The constitution defines the pipeline (spec, plan, tasks, implement) and the spec lifecycle (draft, clarified, planned, in-progress, done), but provides no interactive tooling to enforce it. Projects like anvil have built their own slash commands from scratch, duplicating the pipeline logic that should be shared.

## Behavior

Governance provides a set of `.md` command templates in a `commands/` directory. Each template uses a placeholder `{project}` that adopters replace with their project name. The commands enforce the pipeline gates defined in the constitution.

### Command Set

Ten commands organized into two groups:

#### Pipeline commands (run in order)

- **specify** — prompt qualifying questions to detect lightweight track, create spec (or spec-and-plan) from template, set as session target, add to README
- **clarify** — resolve open questions, enumerate edge cases, verify acceptance criteria, advance draft to clarified
- **plan** — generate plan.md and tasks.md, run readiness check, advance clarified to planned
- **implement** — walk through tasks, write code/tests, verify acceptance criteria, advance planned to done

#### Utility commands

- **about** — print a fixed overview of the pipeline and command usage (no file reads)
- **target** — set the working feature for the session, persisted in a session file
- **status** — read-only dashboard of all specs, their status, artifacts, dependencies, and next actions
- **next** — auto-advance the targeted feature by running the appropriate pipeline command
- **validate** — read-only audit of artifacts for consistency, completeness, and cross-spec alignment
- **setup** — configure `.claude/settings.local.json` with permissions needed for commands to run

### Parameterization

Each command template must work for any project by replacing a single placeholder:

- `{project}` — the project name, used in command references (e.g., `/{project}:clarify`) and file paths (e.g., `.claude/commands/{project}/`)

Commands reference the session file as `.claude/{project}-session.json`.

### Session State

Commands share state through a session file that tracks the current working feature:

```json
{
  "feature": "{NNN-feature-name}",
  "path": "specs/{NNN-feature-name}",
  "setAt": "{ISO 8601 timestamp}"
}
```

### Gate Enforcement

Pipeline commands enforce gates before executing:

- **clarify** gate: spec must be at `draft`; if already `clarified` or later, report and stop
- **plan** gate: spec must be at `clarified`; if `draft`, direct to clarify first
- **implement** gate: spec must be at `planned` or `in-progress`; if earlier, direct to the right command

### Lightweight Track Detection

The `specify` command determines whether a feature qualifies for the lightweight track by prompting the user with qualifying questions:

- Does this touch more than one module or package?
- Are there open questions or unknowns about the approach?
- Does it involve data model changes beyond trivial?
- Will it be more than ~50 lines of spec?

If all answers indicate "small and clear," specify creates `spec-and-plan.md` from a combined template instead of `spec.md`. The `clarify` and `plan` commands detect which file exists and adapt: `clarify` works on whichever file is present, and `plan` skips plan creation if `spec-and-plan.md` already contains the plan section.

### Template References

Pipeline commands reference spec templates from the project's `specs/templates/` directory, not from governance. Each project copies the governance templates into their own `specs/templates/` during bootstrap.

## Acceptance Criteria

- [x] Ten command template files exist in `commands/` directory
- [x] Each template uses `{project}` as the only project-specific placeholder
- [x] Pipeline commands (specify, clarify, plan, implement) enforce gates matching the constitution's spec lifecycle
- [x] The `about` command prints a self-contained guide without reading any files
- [x] The `target` command writes a session file and displays feature status
- [x] The `status` command scans all spec directories and displays a dashboard table
- [x] The `next` command maps current status to the correct pipeline command
- [x] The `validate` command checks spec integrity, artifact completeness, plan consistency, task consistency, dependencies, and cross-spec references
- [x] The `setup` command configures permissions for common operations (git, lint, file reads)
- [x] The `specify` command determines the next feature number, creates the spec directory, and updates README
- [x] Commands reference `specs/templates/` for templates (not governance templates)
- [x] Commands reference `.claude/{project}-session.json` for session state
- [x] The `validate` command runs `markdownlint-cli2` on the feature's files as part of its checks
- [x] The `specify` command prompts qualifying questions and creates `spec-and-plan.md` for lightweight track features
- [x] Pipeline commands detect and handle both `spec.md` and `spec-and-plan.md`

## Resolved Questions

- **Setup and web fetch permissions** — leave to project-specific customization. The setup command handles universal operations (git, lint, file reads). Projects add their own web fetch domains.
- **Validate and markdown lint** — validate includes a markdownlint check as part of its PASS/FAIL report. Lint compliance is a quality gate defined in the constitution.
- **Specify and dependencies** — specify accepts only a description. Dependencies are set during writing and clarifying, not at creation time.
- **Retire/archive command** — deferred. See [specs/spec.md](../spec.md#future-considerations). Projects can manually update status or delete directories.
- **Lightweight track handling** — the `specify` command detects lightweight track eligibility by prompting qualifying questions. Creates `spec-and-plan.md` when all answers indicate small and clear. Pipeline commands adapt based on which file exists.
