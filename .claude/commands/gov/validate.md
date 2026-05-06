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
- In fix mode (`--fix`), modify checkbox state (`- [ ]` → `- [x]`) in spec and task files where the fix is mechanically safe, and write the `title:` frontmatter field on templated artifacts where it is missing, still a literal placeholder, or does not match the canonical `"{folder-name} — {artifact-suffix}"` value (see Fix Mode section below). Do not modify any other content.
- Read only files within the target feature's directory, the cross-spec files needed for reference checks (`specs/system.md`, `specs/events.md`, `specs/errors.md`, dependency spec files), and the project's installed command-source frontmatter for the project-level consistency section below (`.claude/commands/gov/*.md` frontmatter only, plus `.claude/commands/govern.md` frontmatter for the bootstrap installer **if that file exists**, plus `help.md` body for the table comparison). Do NOT read source code or test files.
- Reference: §spec-requirements, §plan-phase, §tasks-phase, §readiness-check, §scenarios, §cross-spec-impact, §text-first-artifacts, §markdown-standards, §drift-prevention (constitution loaded by `/gov:target` — do not re-read).

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

### PKM title field (advisory)

The `title:` frontmatter field gives PKM tools (Obsidian graph, Quartz, Logseq) a unique node label per artifact, since every feature directory contains files with the same basename (`spec.md`, `plan.md`, `tasks.md`). Without it, PKM graphs collapse all artifacts of the same kind into indistinguishable nodes.

For each templated artifact in the feature directory — `spec.md`, `spec-and-plan.md`, `plan.md`, `tasks.md`, `data-model.md`, `research.md`, and any file under `scenarios/` — check that:

- [ ] A `title:` field is present in the frontmatter
- [ ] The value is not the literal placeholder `"{NNN-feature-name} — ..."` — a literal placeholder indicates the substitution was forgotten when the template was scaffolded
- [ ] The value matches `"{folder-name} — {artifact-suffix}"`, where `{folder-name}` is the feature directory's basename and `{artifact-suffix}` is derived from the filename:

  | File | Expected suffix |
  | --- | --- |
  | `spec.md` | `spec` |
  | `spec-and-plan.md` | `spec+plan` |
  | `plan.md` | `plan` |
  | `tasks.md` | `tasks` |
  | `data-model.md` | `data-model` |
  | `research.md` | `research` |
  | `scenarios/{slug}.md` | `scenario: {slug}` |

Files outside this set (custom artifacts) are not checked. The check is fixable in `--fix` mode (see Fix Mode below).

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

### Rules (blocking and advisory)

Rules are the cross-cutting tier of the framework's three-tier requirement model (see §rules in `constitution.md`). Load each rule file in the project's rule-file list. The list currently consists of:

- `specs/security-backend.md`
- `specs/security-frontend.md`

Each file is independently optional — only the files that exist in the project are loaded. New rule files are introduced via their own feature spec; when a new rule file ships, the rule-file list above is updated in the same change. The schema each rule file follows is canonically declared in its introducing spec's data-model (currently `specs/008-security-rules/data-model.md` for the security files).

**Rule file integrity** — for each present rule file:

- [ ] Every rule heading is level-3 and contains only the rule ID (no surrounding text)
- [ ] Every rule has the three required fields: a block-quoted Statement, `**Rationale:**` paragraph, and `**Verification:**` paragraph
- [ ] Every rule's ID matches the format declared in the rule file's introducing-spec data-model (currently `{BE|FE}-{CATEGORY}-{NNN}` for security files, with `CATEGORY` drawn from the data-model's per-surface set)
- [ ] No two rules in the same file share an ID

If any check above fails, the affected rule file is treated as unloadable for the remainder of this validate pass — no rules from that file are applied to the per-rule check below. Emit one of:

- `Malformed rule file {path} at {location}: {reason}` — for missing required fields, ID-format violations, or malformed headings (**blocking**)
- `Duplicate rule ID {ID} in {file}; refusing to load` — when two rules in the same file share an ID (**blocking**)

**No rule files present**:

- [ ] If no rule file in the rule-file list is present in the project, emit `No rule files found, skipping rule checks` (**advisory**) and skip the per-rule and reference checks below

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

### Project-level consistency (advisory)

These checks span the project's installed command set and constitution rather than the target feature. They catch drift in the framework files `govern` ships, surfaced per the Drift Prevention principles in `constitution.md` §drift-prevention. Run once per `/gov:validate` invocation regardless of which feature is targeted; with `--all`, run once before per-feature output.

Read inputs:

- `constitution.md` (already loaded by `/gov:target`)
- `.claude/commands/gov/help.md`
- The full set of `.md` files in `.claude/commands/gov/` (frontmatter only — do not read bodies for these checks)
- `.claude/commands/govern.md` if it exists (frontmatter only — the bootstrap installer lives outside the project namespace)

Checks that reference `.claude/commands/govern.md` are skipped (silently, no finding) when that file does not exist. This covers the `govern` framework repo's own case — the bootstrap installer source lives at `framework/bootstrap/govern.md` but is not installed on the framework repo itself, so `/govern`-row equivalence and frontmatter checks would have nothing to compare against.

Checks:

- [ ] **Help equivalence** — for each command listed in any table in `help.md`, the command's `description:` frontmatter exists and matches (modulo trailing punctuation) the one-line description in the help table. Resolve a `/gov:foo` row to `.claude/commands/gov/foo.md`, and the `/govern` row to `.claude/commands/govern.md`. Mismatches indicate `help.md` was edited without updating the command source, or vice versa. Per the skip rule above, the `/govern` row check is skipped when `.claude/commands/govern.md` is absent.
- [ ] **Anchor resolution** — every `§<anchor>` reference in any installed command file (typically in "Reference: §X, §Y" Scope-Boundaries lines) resolves to a corresponding `<!-- §<anchor> -->` marker in `constitution.md`. A broken reference indicates the constitution was renamed or restructured without updating callers. Report each broken reference with the source command and the unresolved anchor.
- [ ] **Command frontmatter completeness** — every `.md` file in the installed commands directory has a `description:` frontmatter field; the same check applies to `.claude/commands/govern.md` when that file exists. Files whose body documents an `$ARGUMENTS` parameter additionally have `argument-hint:`. Report missing fields; do not check value content.

These are advisory, not blocking — they signal framework drift that the project should resolve at its convenience. They do not prevent pipeline advancement on the target feature.

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
- **PKM `title:` frontmatter field** — when missing, still set to the literal `"{NNN-feature-name} — ..."` placeholder, or value does not match `"{folder-name} — {artifact-suffix}"`, write the canonical value derived from the folder name and filename (mapping in the PKM title field section above). If the file has no frontmatter block at all, prepend one containing only the `title` field.

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
