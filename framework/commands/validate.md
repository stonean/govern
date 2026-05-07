---
description: Check a feature's artifacts for consistency and cross-spec alignment.
argument-hint: "[--all] [feature]"
---

# Validate

Check a feature's artifacts for consistency and cross-spec alignment.

## Purpose

Audit a feature's spec, plan, tasks, and data model for consistency. Read-only; reports issues without modifying files. Use this to catch problems before the next pipeline gate fires.

## Context

Parse `$ARGUMENTS` for flags and an optional feature identifier:

- **Feature identifier** — a feature number, partial name, or full directory name. Overrides the session target.
- **`--all`** — scan all feature directories under `specs/` instead of a single target. Report results grouped by feature.

If `--all` is not present, use the feature identifier if provided, otherwise fall back to the session target from `{cli-config-dir}/{project}-session.json`. If no target can be resolved, stop and tell the user to run `/{project}:target` first or use `--all`.

## Scope Boundaries

- This is a read-only command. Do NOT modify any files.
- Read only files within the target feature's directory, the cross-spec files needed for reference checks (`specs/system.md`, `specs/events.md`, `specs/errors.md`, dependency spec files), and the project's installed command-source frontmatter for the project-level consistency section below (`{cli-config-dir}/commands/{project}/*.md` frontmatter only, plus `{cli-config-dir}/commands/govern.md` frontmatter for the bootstrap installer **if that file exists**). May invoke `scripts/gen-readme-table.sh --dry-run`, `scripts/gen-help-tables.sh --dry-run`, and `scripts/gen-spec-deps.sh --dry-run` to surface generator drift. Do NOT read source code or test files.
- Reference: §spec-requirements, §plan-phase, §tasks-phase, §readiness-check, §scenarios, §cross-spec-impact, §text-first-artifacts, §markdown-standards, §drift-prevention (constitution loaded by `/{project}:target` — do not re-read).

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
- [ ] The `section` field is present and non-empty.

Reference: the schema is canonically declared in `framework/constitution.md` §text-first-artifacts.

### Frontmatter schema (informational)

- [ ] Unknown fields beyond the declared schema are permitted and reported as informational findings (no action required). This includes stale fields in done specs (`title`, `tags`, `spec-ref`, `track`) per the open-schema rule.

### Generator drift (advisory)

Generated content blocks should reflect their sources. Run each generator in `--dry-run` mode and report any diff:

- [ ] `scripts/gen-spec-deps.sh --dry-run` against the target spec — surface as `Body inline links and frontmatter dependencies are out of sync; the next commit will resolve.`
- [ ] `scripts/gen-readme-table.sh --dry-run` (project-level, see Project-level consistency below)
- [ ] `scripts/gen-help-tables.sh --dry-run` (project-level)

These drifts are advisory because the pre-commit hook resolves them on the next commit. They surface only when running validate against an uncommitted state.

### Spec integrity (blocking)

- [ ] Acceptance criteria section exists with at least one checkbox item
- [ ] No placeholder or empty acceptance criteria
- [ ] Open questions consistent with status (`clarified` or later must have none). When this check fails — a spec at `clarified` / `planned` / `in-progress` with one or more open questions in the body — the spec is in the recovery state defined by spec 014. Suggested fix: run `/{project}:clarify` (its recovery path will revert status to `draft` and walk the questions), or `/{project}:ask` on a fresh question (which performs the back-edge automatically).
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
- `specs/configuration.md`

Each file is independently optional — only the files that exist in the project are loaded. New rule files are introduced via their own feature spec; when a new rule file ships, the rule-file list above is updated in the same change. The schema each rule file follows is canonically declared in its introducing spec's data-model (`specs/008-security-rules/data-model.md` for the security files; `specs/017-derive-dont-ask/data-model.md` for the configuration file).

**Rule file integrity** — for each present rule file:

- [ ] Every rule heading is level-3 and contains only the rule ID (no surrounding text)
- [ ] Every rule has the three required fields: a block-quoted Statement, `**Rationale:**` paragraph, and `**Verification:**` paragraph
- [ ] Every rule's ID matches the format declared in the rule file's introducing-spec data-model (`{BE|FE}-{CATEGORY}-{NNN}` for security files; `CFG-{CONST|ENV}-{NNN}` for configuration)
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

These checks span the project's installed command set and constitution rather than the target feature. They catch drift in the framework files `govern` ships, surfaced per the Drift Prevention principles in `constitution.md` §drift-prevention. Run once per `/{project}:validate` invocation regardless of which feature is targeted; with `--all`, run once before per-feature output.

Read inputs:

- `constitution.md` (already loaded by `/{project}:target`)
- `{cli-config-dir}/commands/{project}/help.md`
- The full set of `.md` files in `{cli-config-dir}/commands/{project}/` (frontmatter only — do not read bodies for these checks)
- `{cli-config-dir}/commands/govern.md` if it exists (frontmatter only — the bootstrap installer lives outside the project namespace)

Checks that reference `{cli-config-dir}/commands/govern.md` are skipped (silently, no finding) when that file does not exist. This covers the `govern` framework repo's own case — the bootstrap installer source lives at `framework/bootstrap/govern.md` but is not installed on the framework repo itself, so `/govern`-row equivalence and frontmatter checks would have nothing to compare against.

Checks:

- [ ] **Generator drift** — run `scripts/gen-readme-table.sh --dry-run` and `scripts/gen-help-tables.sh --dry-run` (when the scripts exist in the project). Non-empty diff means the README Feature Specs table or the help.md command tables are out of sync with their sources. Report each as `Generator out of sync: {script}; the next commit will resolve.` This replaces the per-row help-equivalence check — the table is generated, so structural sync is the only meaningful check.
- [ ] **Anchor resolution** — every `§<anchor>` reference in any installed command file (typically in "Reference: §X, §Y" Scope-Boundaries lines) resolves to a corresponding `<!-- §<anchor> -->` marker in `constitution.md`. A broken reference indicates the constitution was renamed or restructured without updating callers. Report each broken reference with the source command and the unresolved anchor.
- [ ] **Command frontmatter completeness** — every `.md` file in the installed commands directory has a `description:` frontmatter field; the same check applies to `{cli-config-dir}/commands/govern.md` when that file exists. Files whose body documents an `$ARGUMENTS` parameter additionally have `argument-hint:`. Report missing fields; do not check value content.

These are advisory, not blocking — they signal framework drift that the project should resolve at its convenience. They do not prevent pipeline advancement on the target feature.

### Report

Separate results into sections by severity:

1. **Hard fail** — required-field violations and malformed frontmatter. The spec is not valid; pipeline advancement is blocked. List these first.
2. **Blocking** — structural or content issues that must be fixed before the next pipeline gate fires.
3. **Advisory** — issues that should be fixed but do not block advancement.
4. **Informational** — observations (e.g., unknown frontmatter fields) that may warrant attention but are neither errors nor warnings.

For each FAIL, include: what failed, what was expected, what was found, and a suggested fix.
