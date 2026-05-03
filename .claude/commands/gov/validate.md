---
description: Check a feature's artifacts for consistency and cross-spec alignment.
argument-hint: "[--fix] [--all] [feature]"
---

# Validate

Check a feature's artifacts for consistency and cross-spec alignment.

## Purpose

Audit a feature's spec, plan, tasks, and data model for consistency. By default, reports issues without modifying files. With `--fix`, automatically corrects fixable checkbox state mismatches. Use this to catch problems before the next pipeline gate fires.

## Context

Parse `$ARGUMENTS` for flags and an optional feature identifier:

- **Feature identifier** — a feature number, partial name, or full directory name. Overrides the session target.
- **`--fix`** — enable fix mode (see Fix Mode section below).
- **`--all`** — scan all feature directories under `specs/` instead of a single target. Report results grouped by feature.

Flags can be combined (e.g., `--all --fix`, `001 --fix`).

If `--all` is not present, use the feature identifier if provided, otherwise fall back to the session target from `.claude/gov-session.json`. If no target can be resolved, stop and tell the user to run `/gov:target` first or use `--all`.

## Scope Boundaries

- By default, this is a read-only command. Do NOT modify any files.
- In fix mode (`--fix`), modify only checkbox state (`- [ ]` → `- [x]`) in spec and task files where the fix is mechanically safe (see Fix Mode section below). Do not modify any other content.
- Read only files within the target feature's directory and the cross-spec files needed for reference checks (`specs/system.md`, `specs/events.md`, `specs/errors.md`, dependency spec files). Do NOT read source code or test files.
- Reference: §spec-requirements, §plan-phase, §tasks-phase, §readiness-check, §scenarios, §cross-spec-impact, §text-first-artifacts, §markdown-standards (constitution loaded by `/gov:target` — do not re-read).

## Instructions

Read every file in `specs/{feature}/` and run the following checks. Each check is classified by severity:

- **Hard fail (blocking)** — required-field violations and malformed frontmatter. The spec is not valid until these are fixed; pipeline advancement is blocked.
- **Blocking** — structural or content issues that must be fixed before the next pipeline gate fires.
- **Advisory** — issues that should be fixed but do not block advancement.
- **Informational** — observations that may warrant attention but are neither errors nor warnings.

### Frontmatter schema (hard fail)

For each spec file (`spec.md`, `spec-and-plan.md`):

- [ ] A YAML frontmatter block exists at the top of the file (delimited by `---` lines).
- [ ] The frontmatter parses as valid YAML.
- [ ] The `status` field is present and one of: `draft`, `clarified`, `planned`, `in-progress`, `done`.
- [ ] The `dependencies` field is present and is a list (empty list permitted).

For each scenario file (`scenarios/{slug}.md`):

- [ ] A YAML frontmatter block exists at the top of the file.
- [ ] The frontmatter parses as valid YAML.
- [ ] The `spec-ref` field is present and non-empty.

Reference: the schema is canonically declared in `framework/constitution.md` §text-first-artifacts.

### Frontmatter schema (advisory)

- [ ] Spec files have a non-empty `tags` field. Empty or missing `tags` is reported as an advisory finding ("Tags help cross-cutting graph views; consider adding some.") but does not block.

### Frontmatter schema (informational)

- [ ] Unknown fields beyond the declared schema are permitted and reported as informational findings (no action required).

### Spec integrity (blocking)

- [ ] Acceptance criteria section exists with at least one checkbox item
- [ ] No placeholder or empty acceptance criteria
- [ ] Open questions consistent with status (`clarified` or later must have none). When this check fails — a spec at `clarified` / `planned` / `in-progress` with one or more open questions in the body — the spec is in the recovery state defined by spec 014. Suggested fix: run `/gov:clarify` (its recovery path will revert status to `draft` and walk the questions), or `/gov:ask` on a fresh question (which performs the back-edge automatically).
- [ ] No implementation code blocks (function signatures, package paths, language-specific snippets) in the spec — those belong in plan.md. Format examples, directory structures, and user-facing commands are acceptable when they define behavioral contracts.

### Artifact completeness (blocking)

- [ ] If status is `planned` or later: plan.md exists (or spec-and-plan.md contains a plan section)
- [ ] If status is `planned` or later and feature introduces or modifies domain entities or data structures: data-model.md exists
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

- [ ] Every scenario file has Context and Behavior sections (frontmatter `spec-ref` is checked under Frontmatter schema above)
- [ ] Every scenario file in `scenarios/` has a corresponding task in `tasks.md`
- [ ] Scenario-linked tasks in `tasks.md` are marked complete if the spec status is `done`

### Dependencies (blocking)

- [ ] Every entry in this spec's frontmatter `dependencies` list exists as a spec directory under `specs/`
- [ ] Each dependency's frontmatter `status` is at `clarified` or later (if this spec is `clarified` or later)

### Cross-spec references (advisory)

- [ ] Event types mentioned in spec or plan align with `specs/events.md`
- [ ] Error codes follow the convention from `specs/errors.md`
- [ ] Data model definitions do not conflict with other specs' data-model.md files

### Security rules (blocking and advisory)

Load `specs/security-backend.md` and `specs/security-frontend.md` if either is present in the project. Each file is independently optional — only the files that exist are loaded. The rule-file schema is canonically declared in `specs/008-security-rules/data-model.md`.

**Rule file integrity** — for each present rule file:

- [ ] Every rule heading is level-3 and contains only the rule ID (no surrounding text)
- [ ] Every rule has the three required fields: a block-quoted Statement, `**Rationale:**` paragraph, and `**Verification:**` paragraph
- [ ] Every rule's ID matches the format `{BE|FE}-{CATEGORY}-{NNN}` (zero-padded) with `CATEGORY` drawn from the data-model's per-surface set
- [ ] No two rules in the same file share an ID

If any check above fails, the affected rule file is treated as unloadable for the remainder of this validate pass — no rules from that file are applied to the per-rule check below. Emit one of:

- `Malformed security rule file {path} at {location}: {reason}` — for missing required fields, ID-format violations, or malformed headings (**blocking**)
- `Duplicate rule ID {ID} in {file}; refusing to load` — when two rules in the same file share an ID (**blocking**)

**No rule files present**:

- [ ] If neither `specs/security-backend.md` nor `specs/security-frontend.md` is present, emit `No security rule files found, skipping security checks` (**advisory**) and skip the per-rule and reference checks below

**Per-rule check** — when at least one rule file is loaded and well-formed, iterate every loaded rule and execute its **Verification** instruction against the project's `spec.md`, `spec-and-plan.md`, `plan.md`, `scenarios/*.md`, and `specs/system.md` content:

- [ ] For each MUST or MUST NOT rule whose Verification trigger fires against an artifact that does not include the required commitment, emit `{Rule ID}: {artifact path} — {one-line gap summary}` (**blocking**)
- [ ] For each SHOULD or SHOULD NOT rule whose trigger fires, emit `{Rule ID}: {artifact path} — {one-line gap summary}` (**advisory**)
- [ ] A rule whose Verification trigger does not fire against any artifact produces no finding (silently inert — the contextual-application property)

**Rule references** — scan all project artifacts for inline rule-ID references (e.g., `BE-AUTHN-001`, `FE-XSS-002`):

- [ ] If an artifact references an ID not present in any loaded rule file, emit `Spec at {path} references unknown rule {ID}` (**blocking**)
- [ ] If an artifact references an ID that exists but is marked `DEPRECATED`, emit `Spec at {path} references deprecated rule {ID}; targeted for removal in {version}` (**advisory**)

Findings produced by this section are surfaced under validate's existing severity headers in the report — blocking findings join **Blocking**, advisory findings join **Advisory**.

### Markdown lint (advisory)

- [ ] All `.md` files in the feature directory pass `npx markdownlint-cli2`

### Report

Separate results into sections by severity:

1. **Hard fail** — required-field violations and malformed frontmatter. The spec is not valid; pipeline advancement is blocked. List these first.
2. **Blocking** — structural or content issues that must be fixed before the next pipeline gate fires.
3. **Advisory** — issues that should be fixed but do not block advancement.
4. **Informational** — observations (e.g., unknown frontmatter fields) that may warrant attention but are neither errors nor warnings.

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
