# Plan

Create a technical plan and task breakdown for a clarified spec.

## Purpose

Pipeline gate: `clarified` → `planned`. A spec cannot be implemented until it has a plan with technical decisions, affected files, and an ordered task list. This command produces both `plan.md` and `tasks.md`.

## Context

Use the session target from `.claude/gov-session.json`. If `$ARGUMENTS` is provided, use it to override the session target. If no session target is set and no arguments provided, stop and tell the user to run `/gov:target` first.

## Spec File Detection

Check for `spec.md` first, then `spec-and-plan.md`. Use whichever exists. If neither exists, stop and report: "Spec does not exist. Run `/gov:specify` first."

## Gate

Read the spec status. If the status is not `clarified`, stop and report:

- `draft` → "Spec has unresolved open questions. Run `/gov:clarify` first."
- `planned` or later → "Already planned. Run `/gov:implement` to begin implementation."

## Scope Boundaries

- Read only files needed for planning: the target spec, `specs/system.md`, and cross-spec files per the checklist below. Do NOT read source code, test files, or unrelated specs beyond what the checklist requires.
- Do NOT begin implementation. This command produces `plan.md` and `tasks.md` only.
- Reference: §plan-phase, §tasks-phase, §readiness-check (constitution loaded by `/gov:target` — do not re-read).

## Instructions

### Cross-spec context checklist

Before creating the plan, load only the cross-spec context this feature actually needs:

- **Always read:** `specs/system.md` — architecture patterns and shared conventions.
- **Read if the feature emits or consumes events:** `specs/events.md` — check for naming conflicts and reuse opportunities.
- **Read if the feature introduces error codes:** `specs/errors.md` — check code ranges and format conventions.
- **Read if the feature has dependencies:** the spec file (not plan or tasks) of each dependency — confirm status and understand the contracts this feature builds on.
- **Read if the feature involves persistence:** `data-model.md` files from related specs — check for table/column conflicts.
- **Do NOT read** plans, tasks, scenarios, or source code from other features.

### Create the plan

If the spec file is `spec-and-plan.md` (lightweight track), the plan section is already in the combined document. Skip plan creation and proceed to tasks. Otherwise:

1. Copy `specs/templates/plan.md` into the feature directory as `plan.md`.
2. Fill in:
   - **Technical Decisions**: each decision with rationale. Code snippets, function signatures, and package paths belong here.
   - **Affected Files**: every file that will be created or modified.
   - **Data Model**: database schema changes. Create `data-model.md` if the feature involves persistence.
   - **Trade-offs**: what was considered and rejected, known limitations.
3. Cross-validate against the files loaded in the checklist above:
   - Plan must not conflict with `specs/system.md`.
   - Data model must be consistent with related specs.
   - Event types must align with `specs/events.md`.

### Create the task breakdown

1. Copy `specs/templates/tasks.md` into the feature directory as `tasks.md`.
2. Break the plan into discrete, ordered work items:
   - Each task is small enough to complete and verify in a single session.
   - Each task has a clear "done when" condition.
   - Tasks respect dependency order.
   - Tasks are derived from the plan, not invented independently.

### Readiness check

Before updating status, verify:

- [ ] Acceptance criteria are concrete and testable
- [ ] All open questions are resolved
- [ ] Data model exists if the feature involves persistence
- [ ] Plan does not conflict with `system.md` or other feature specs
- [ ] Data model is consistent with related specs
- [ ] Event types align with `events.md`
- [ ] Tasks are ordered and each has a clear definition of done

If any check fails, report the failure and do not update status.

### Finalize

1. Present a summary of the plan, task breakdown, and readiness check results. Ask the user to approve the transition to `planned`. Do not update the status until the user confirms.
2. Update spec status to `planned`.
3. Run `markdownlint-cli2` on all new files.
4. Display the next step: "Run `/gov:implement` to begin implementation."
