---
description: Create a technical plan and task breakdown for a clarified spec.
argument-hint: "[feature]"
---

# Plan

Create a technical plan and task breakdown for a clarified spec.

## Purpose

Pipeline gate: `clarified` → `planned`. A spec cannot be implemented until it has a plan with technical decisions, affected files, and an ordered task list. This command produces both `plan.md` and `tasks.md`.

## Context

Use the session target from `.claude/gov-session.json`. If `$ARGUMENTS` is provided, use it to override the session target. If no session target is set and no arguments provided, stop and tell the user to run `/gov:target` first.

## Spec File Detection

Check for `spec.md` first, then `spec-and-plan.md`. Use whichever exists. If neither exists, stop and report: "Spec does not exist. Run `/gov:specify` first."

## Gate

Read the spec's `status` field from the YAML frontmatter at the top of the file. If `status` is not `clarified`, stop and report:

- `draft` → "Spec has unresolved open questions. Run `/gov:clarify` first."
- `planned` or later → "Spec is already planned. Run `/gov:implement` to begin implementation."

## Scope Boundaries

- Read only files needed for planning: the target spec, `specs/system.md`, and cross-spec files per the checklist below. Do NOT read source code, test files, or unrelated specs beyond what the checklist requires.
- Do NOT begin implementation. This command produces `plan.md` and `tasks.md` only.
- Reference: §plan-phase, §tasks-phase, §readiness-check, §text-first-artifacts (constitution loaded by `/gov:target` — do not re-read).

## Instructions

### Detect existing artifacts

Before generating any artifacts, check the feature directory for existing plan files. This protects work the user may have already invested — including plans that survived a `/gov:ask` back-edge cycle (clarified+ → draft → clarified again) and any other re-run.

1. Check the feature directory for `plan.md`, `tasks.md`, and `data-model.md`. (If the spec file is `spec-and-plan.md`, this check still runs — `data-model.md` is the only artifact that can pre-exist in lightweight-track features; the plan and tasks live inside the combined document.)
2. If none of those files exist, skip this section and proceed to the cross-spec context checklist with the standard template-copy flow unchanged.
3. If any of those files exists, list each one that exists with its last-modified timestamp, then prompt:
   > Plan artifacts exist from a prior `/gov:plan` run. Keep them and run the readiness check, or replace with fresh templates?

   The default is **keep**.
4. **Keep** — skip the template copy entirely. Do not overwrite or modify the existing artifacts during this step. Proceed to the cross-spec context checklist; in **Create the plan** and **Create the task breakdown**, skip the "copy template" steps and treat the existing files as the working artifacts. Then run the validation gate. Advance status to `planned` only if all readiness checks pass; on failure, report the specific failures and exit without advancing — the user fixes the kept artifacts and re-runs.
5. **Replace** — copy fresh templates over the existing files (`specs/templates/plan.md` → `plan.md`, `specs/templates/tasks.md` → `tasks.md`, and a fresh `data-model.md` only if the prior one existed and the feature still needs one). The user is responsible for re-applying any kept content. Then proceed with the standard plan flow below.

### Cross-spec context checklist

Before creating the plan, load only the cross-spec context this feature actually needs:

- **Always read:** `specs/system.md` — architecture patterns and shared conventions.
- **Read if the feature emits or consumes events:** `specs/events.md` — check for naming conflicts and reuse opportunities.
- **Read if the feature introduces error codes:** `specs/errors.md` — check code ranges and format conventions.
- **Read if the feature has dependencies:** the spec file (not plan or tasks) of each dependency listed in this spec's frontmatter `dependencies` field — confirm `status` and understand the contracts this feature builds on.
- **Read if the feature introduces or modifies domain entities or data structures:** `data-model.md` files from related specs — check for structural conflicts.
- **Do NOT read** plans, tasks, scenarios, or source code from other features.

### Create the plan

If the spec file is `spec-and-plan.md` (lightweight track), the plan section is already in the combined document. Skip plan creation and proceed to tasks. Otherwise:

1. **If the user picked "keep" in the existing-artifact prompt above**, skip the template copy — `plan.md` is already on disk and is the working artifact. Otherwise (no prior artifacts, or "replace"), copy `specs/templates/plan.md` into the feature directory as `plan.md`.
2. Fill in (or, on the keep path, edit/extend the existing content):
   - **Technical Decisions**: each decision with rationale. Code snippets, function signatures, and package paths belong here.
   - **Affected Files**: every file that will be created or modified.
   - **Data Model**: data structure definitions. Create `data-model.md` if the feature introduces or modifies domain entities or data structures.
   - **Trade-offs**: what was considered and rejected, known limitations.
3. Cross-validate against the files loaded in the checklist above:
   - Plan must not conflict with `specs/system.md`.
   - Data model must be consistent with related specs.
   - Event types must align with `specs/events.md`.

### Create the task breakdown

1. **If the user picked "keep" in the existing-artifact prompt above**, skip the template copy — `tasks.md` is already on disk and is the working artifact. Otherwise (no prior artifacts, or "replace"), copy `specs/templates/tasks.md` into the feature directory as `tasks.md`.
2. Break the plan into discrete, ordered work items:
   - Each task is small enough to complete and verify in a single session.
   - Each task has a clear "done when" condition.
   - Tasks respect dependency order.
   - Tasks are derived from the plan, not invented independently.
3. Propose `[simple]` tier markers on trivial tasks. After the task list is written, scan each task and append `[simple]` to the header (e.g., `## 4. Update README link [simple]`) when the task is genuinely trivial — single small file edit, no logic, no schema change, no new behavior. When in doubt, leave the marker off; default tier is the right call for any task that touches more than one file or carries non-obvious decisions. The marker convention is documented in the tasks template (`specs/templates/tasks.md`); see that file for the full rule.

### Validation gate

Before proposing the status transition, run the readiness check. All checks must pass — failures block the transition.

- [ ] Acceptance criteria are concrete and testable
- [ ] All open questions are resolved
- [ ] Data model exists if the feature introduces or modifies domain entities or data structures
- [ ] Plan does not conflict with `system.md` or other feature specs
- [ ] Data model is consistent with related specs
- [ ] Event types align with `events.md`
- [ ] Tasks are ordered and each has a clear definition of done
- [ ] All `.md` files in the feature directory pass `npx markdownlint-cli2`

If any check fails, report the specific failures and do not propose the transition. The user fixes the issues and re-runs the command.

### Finalize

1. Present a summary of the plan, task breakdown, and validation gate results. List which tasks (if any) were proposed `[simple]` so the user can add, remove, or accept markers before approving. Ask the user to approve the transition to `planned`. Do not update the status until the user confirms.
2. On confirmation, update the spec's frontmatter `status` field from `clarified` to `planned`.
3. Display the next step: "Run `/gov:implement` to begin implementation."
