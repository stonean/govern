# Validate

Check a feature's artifacts for consistency and cross-spec alignment.

## Purpose

Audit a feature's spec, plan, tasks, and data model for consistency. By default, reports issues without modifying files. With `--fix`, automatically corrects fixable checkbox state mismatches. Use this to catch problems before advancing to the next pipeline phase.

## Context

Parse `$ARGUMENTS` for flags and an optional feature identifier:

- **Feature identifier** — a feature number, partial name, or full directory name. Overrides the session target.
- **`--fix`** — enable fix mode (see Fix Mode section below).
- **`--all`** — scan all feature directories under `specs/` instead of a single target. Report results grouped by feature.

Flags can be combined (e.g., `--all --fix`, `001 --fix`).

If `--all` is not present, use the feature identifier if provided, otherwise fall back to the session target from `{cli-config-dir}/{project}-session.json`. If no target can be resolved, stop and tell the user to run `/{project}:target` first or use `--all`.

## Scope Boundaries

- By default, this is a read-only command. Do NOT modify any files.
- In fix mode (`--fix`), modify only checkbox state (`- [ ]` → `- [x]`) in spec and task files where the fix is mechanically safe (see Fix Mode section below). Do not modify any other content.
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
- [ ] No implementation code blocks (function signatures, package paths, language-specific snippets) in the spec — those belong in plan.md. Format examples, directory structures, and user-facing commands are acceptable when they define behavioral contracts.

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

- [ ] All `.md` files in the feature directory pass `npx markdownlint-cli2`

### Report

Separate results into two sections:

1. **Blocking** — issues that must be fixed before the spec can advance. List these first.
2. **Advisory** — issues that should be fixed but do not block advancement.

For each FAIL, include: what failed, what was expected, what was found, and a suggested fix.

## Fix Mode

When `$ARGUMENTS` contains `--fix`, after running all checks, automatically correct fixable checkbox mismatches:

### Fixable (auto-correct)

- Acceptance criteria checkboxes (`- [ ]` → `- [x]`) in specs with status `done`
- Task checkboxes (`- [ ]` → `- [x]`) in `tasks.md` where all sub-item checkboxes are already `- [x]`
- Scenario-linked task checkboxes (`- [ ]` → `- [x]`) where the spec status is `done`

### Not fixable (report only)

- Checkboxes in specs with status `in-progress` — cannot determine which criteria are truly met without verification
- Missing artifacts (no plan, no tasks) — structural issues require human decisions
- Lint failures — require manual correction
- Any non-checkbox issue

### Fix mode behavior

1. Run all checks as normal.
2. For each fixable issue, display the file, the checkbox line, and the correction being made.
3. Apply the corrections to the files.
4. Run `npx markdownlint-cli2` on modified files.
5. Report a summary: number of fixes applied, number of remaining issues (non-fixable).
6. If no fixable issues are found, report "No fixes needed."
