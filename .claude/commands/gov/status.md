---
description: Display the pipeline dashboard for all feature specs.
---

# Status

Display the pipeline dashboard for all feature specs.

## Purpose

Read-only overview of every feature's progress through the pipeline. Shows which specs are ready to advance, which are blocked, and what the current session target is.

## Scope Boundaries

- This is a read-only command. Do NOT modify any files.
- For each feature, read only the spec file (`spec.md` or `spec-and-plan.md`) to extract `status`, `dependencies`, `tags`, and open question count. Do NOT read plans, tasks, scenarios, source code, or other artifact contents.
- Check file existence (`plan.md`, `tasks.md`, `data-model.md`, `scenarios/`) without reading them.
- Reference: §text-first-artifacts (the schema is the authoritative source for which fields to read).

## Instructions

**Steps 1–2 must complete before any other work. Do NOT read spec directories, list files, or perform any dashboard work until step 2 resolves.**

1. Read `.claude/gov-session.json` for the current session target (if any), including optional `scenario` and `scenarioPath` fields.
2. If a session target exists, read **only** the target spec's `spec.md` (or `spec-and-plan.md`) to extract the YAML frontmatter `status` field and count entries in the body's `## Open Questions` section.
   - If `status` is **not** `done`: display the target feature name and status. If a scenario is targeted, also read the scenario file (frontmatter `spec-ref` plus body) and display scenario detail: scenario name, spec-ref, context summary, and open question count. Then prompt the next pipeline command:
     - If a scenario is targeted **and** the scenario has one or more open questions → `/gov:clarify` (scenario-targeted, resolves scenario-level open questions regardless of parent spec status).
     - **Recovery state — `(status ∈ {clarified, planned, in-progress}, open-question count ≥ 1)`** → `/gov:clarify` (the recovery path will surface the inconsistency before any forward action). This state usually arises from a manual frontmatter edit; the normal back-edge via `/gov:ask` keeps spec status and open-question presence in sync.
     - Otherwise, prompt based on the spec's status: `draft` → `/gov:clarify`, `clarified` → `/gov:plan`, `planned`/`in-progress` → `/gov:implement`.

     **Stop here — do not build the full dashboard.**
   - If `status` **is** `done`: continue to step 3 to build the full dashboard.
3. List directories under `specs/` matching the `NNN-*` pattern.
4. For each feature directory, read **only** `spec.md` (or `spec-and-plan.md` if `spec.md` does not exist). Do not read or list any other files. Parse the YAML frontmatter block at the top of the file and extract:
   - `status` (allowed values: `draft`, `clarified`, `planned`, `in-progress`, `done`)
   - `dependencies` (list of spec slugs; empty list permitted)
   - `tags` (list of free-form strings; may be empty or absent — treat absent as empty)
   - Open question count from the body's `## Open Questions` section
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
   - If at least one spec has non-empty `tags`, list the union of tags in use across the repo (one line, comma-separated). Skip the line entirely if no spec has tags.
9. List any non-done specs (excluding the current target) and prompt the user to run `/gov:target` to select one.
