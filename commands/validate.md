# Validate

Check a feature's artifacts for consistency and cross-spec alignment.

## Purpose

Read-only audit of a feature's spec, plan, tasks, and data model. Reports issues without modifying files. Use this to catch problems before advancing to the next pipeline phase.

## Context

Use the session target from `{cli-config-dir}/{project}-session.json`. If `$ARGUMENTS` is provided, use it to override the session target. If no session target is set and no arguments provided, stop and tell the user to run `/{project}:target` first.

## Scope Boundaries

- This is a read-only command. Do NOT modify any files.
- Read only files within the target feature's directory and the cross-spec files needed for reference checks (`specs/system.md`, `specs/events.md`, `specs/errors.md`, dependency spec files). Do NOT read source code or test files.
- Reference: §spec-requirements, §plan-phase, §tasks-phase, §readiness-check, §scenarios (constitution loaded by `/{project}:target` — do not re-read).

## Instructions

Read every file in `specs/{feature}/` and run the following checks. Each check is classified as **blocking** (must fix before advancing to the next pipeline phase) or **advisory** (should fix but does not block advancement).

### Spec integrity (blocking)

- [ ] Status field is present and valid (draft, clarified, planned, in-progress, done)
- [ ] Dependencies field is present
- [ ] Acceptance criteria section exists with at least one checkbox item
- [ ] No placeholder or empty acceptance criteria
- [ ] Open questions consistent with status (`clarified` or later must have none)
- [ ] No code blocks, function signatures, or package paths in the spec (those belong in plan.md)

### Artifact completeness (blocking)

- [ ] If status is `planned` or later: plan.md exists (or spec-and-plan.md contains a plan section)
- [ ] If status is `planned` or later and feature involves persistence: data-model.md exists
- [ ] If status is `planned` or later: tasks.md exists

### Plan consistency (blocking if plan exists)

- [ ] Plan references the spec
- [ ] Technical decisions section has at least one decision with rationale
- [ ] Affected files section lists specific file paths
- [ ] Plan does not contradict `specs/system.md`

### Task consistency (blocking if tasks exist)

- [ ] Tasks reference the plan
- [ ] Each task has a "done when" condition
- [ ] Tasks are numbered and ordered

### Scenario consistency (advisory)

- [ ] Every scenario file has a spec-ref, Context, and Behavior section
- [ ] Every scenario file in `scenarios/` has a corresponding task in `tasks.md`
- [ ] Scenario-linked tasks in `tasks.md` are marked complete if the spec status is `done`

### Dependencies (blocking)

- [ ] All listed dependencies exist as spec directories
- [ ] Dependencies are at `clarified` or later (if this spec is `clarified` or later)

### Cross-spec references (advisory)

- [ ] Event types mentioned in spec or plan align with `specs/events.md`
- [ ] Error codes follow the convention from `specs/errors.md`
- [ ] Data model tables do not conflict with other specs' data-model.md files

### Markdown lint (advisory)

- [ ] All `.md` files in the feature directory pass `markdownlint-cli2`

### Report

Separate results into two sections:

1. **Blocking** — issues that must be fixed before the spec can advance. List these first.
2. **Advisory** — issues that should be fixed but do not block advancement.

For each FAIL, include: what failed, what was expected, what was found, and a suggested fix.
