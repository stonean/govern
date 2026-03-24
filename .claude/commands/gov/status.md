# Status

Display the pipeline dashboard for all feature specs.

## Purpose

Read-only overview of every feature's progress through the pipeline. Shows which specs are ready to advance, which are blocked, and what the current session target is.

## Scope Boundaries

- This is a read-only command. Do NOT modify any files.
- For each feature, read only the spec file (`spec.md` or `spec-and-plan.md`) to extract status, dependencies, and open question count. Do NOT read plans, tasks, scenarios, source code, or other artifact contents.
- Check file existence (`plan.md`, `tasks.md`, `data-model.md`, `scenarios/`) without reading them.

## Instructions

**Steps 1–2 must complete before any other work. Do NOT read spec directories, list files, or perform any dashboard work until step 2 resolves.**

1. Read `.claude/gov-session.json` for the current session target (if any), including optional `scenario` and `scenarioPath` fields.
2. If a session target exists, read **only** the target spec's `spec.md` (or `spec-and-plan.md`) to get its status.
   - If the target spec's status is **not** `done`: display the target feature name and status. If a scenario is targeted, also read the scenario file and display scenario detail: scenario name, spec-ref, context summary, and open question count. Then prompt the next pipeline command based on status (`draft` → `/gov:clarify`, `clarified` → `/gov:plan`, `planned`/`in-progress` → `/gov:implement`). **Stop here — do not build the full dashboard.**
   - If the target spec's status **is** `done`: continue to step 3 to build the full dashboard.
3. List directories under `specs/` matching the `NNN-*` pattern.
4. For each feature directory, read **only** `spec.md` (or `spec-and-plan.md` if `spec.md` does not exist). Do not read or list any other files. Extract:
   - Status from the `**Status:** {value}` line (valid values: draft, clarified, planned, in-progress, done)
   - Dependencies from the `**Dependencies:** {value}` line (comma-separated spec slugs, or "none")
   - Open question count
5. Check whether these files exist (do not read them): `plan.md`, `tasks.md`, `data-model.md`.
6. Count `.md` files in the `scenarios/` subdirectory (if it exists) without reading them.
7. Display a table:

   | Feature | Status | Dependencies | Artifacts | Scenarios | Next Action |
   | --- | --- | --- | --- | --- | --- |

   - Mark the session target with `>>`.
   - Scenarios column shows the count of `.md` files in the feature's `scenarios/` directory (0 if none).
   - Next Action based on status: clarify, plan, implement, or done.

8. Below the table, show:
   - Count of specs at each status level.
   - Which specs are blocked (dependencies not at `clarified` or later).
9. List any non-done specs (excluding the current target) and prompt the user to run `/gov:target` to select one.
