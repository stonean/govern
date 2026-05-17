---
description: Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. Read-only.
argument-hint: "[--all] [--fix] [feature]"
parity:
  semantic-fields:
    - "findings[].message"
  strict-fields:
    - "findings[].rule-id"
    - "findings[].severity"
---

# Analyze

Audit a feature's artifacts against each other and against the framework's rule set.

## Purpose

Audit a feature's spec, plan, tasks, and data model for consistency. Read-only; reports issues without modifying files. Use this to catch problems before the next pipeline gate fires.

Renamed from `/validate` in spec 023 to align with the emerging spec-driven-development standard (GitHub Spec Kit uses `/analyze` for the same artifact-vs-artifact audit role). Complementary to `/gov:review`, which audits **code** against rules.

## Context

Parse `$ARGUMENTS` for flags and an optional feature identifier:

- **Feature identifier** — a feature number, partial name, or full directory name. Overrides the session target.
- **`--all`** — scan all feature directories under `specs/` instead of a single target. Report results grouped by feature.

If `--all` is not present, use the feature identifier if provided, otherwise fall back to the session target from `.claude/gov-session.json`. If no target can be resolved, stop and tell the user to run `/gov:target` first or use `--all`.

## Scope Boundaries

- This is a read-only command. Do NOT modify any files.
- Read only files within the target feature's directory, the cross-spec files needed for reference checks (`specs/system.md`, `specs/events.md`, `specs/errors.md`, dependency spec files), and the project's installed command-source frontmatter for the project-level consistency section below (`.claude/commands/gov/*.md` frontmatter only, plus `.claude/commands/govern.md` frontmatter for the bootstrap installer **if that file exists**). May invoke `scripts/gen-readme-table.sh --dry-run`, `scripts/gen-help-tables.sh --dry-run`, and `scripts/gen-spec-deps.sh --dry-run` to surface generator drift. Do NOT read source code or test files.
- Reference: §spec-requirements, §plan-phase, §tasks-phase, §readiness-check, §scenarios, §cross-spec-impact, §text-first-artifacts, §markdown-standards, §drift-prevention (constitution loaded by `/gov:target` — do not re-read).

## Instructions

> **For agent runtimes**: backticked primitive names in this section map to MCP tools the optional [gvrn runtime](https://crates.io/crates/gvrn) exposes under bare `<primitive>` names (e.g., `read-spec`). Hosts wrap them with a server-name prefix taken from `.mcp.json` (Claude: `mcp__gvrn__read-spec`; Auggie: `mcp:gvrn:read-spec`). When the server is registered for your session, **call the corresponding tool** for each step listed below — that is the deterministic path. When the server is not registered, walk the prose to produce the same result. The two paths share a contract; neither one wraps the other.

1. Invoke `read-spec` (MCP: `read-spec`) against the targeted feature to load frontmatter, sections, and the open-question count from the body. The result drives subsequent steps' tier classification (status governs which artifact-completeness checks apply).

2. Invoke `validate-frontmatter` (MCP: `validate-frontmatter`) against the spec path to check that the YAML block parses and that the required fields (status, dependencies) are present with valid values. Frontmatter findings are hard-fail tier; the rest of the procedure still runs to surface every issue in a single pass.

3. Invoke `traverse-deps` (MCP: `traverse-deps`) against the feature to verify each dependency directory exists and carries a compatible status. Missing dependencies are blocking; incompatible statuses are blocking when this spec is at clarified or later.

4. Invoke `resolve-anchor` (MCP: `resolve-anchor`) against the spec path to confirm every section reference of the form §anchor resolves to a corresponding marker comment. Unresolved anchors are advisory — they usually indicate the constitution was renamed or restructured without updating callers. Otherwise, fall back to the markdown-only path.

5. Invoke `check-rule-ids` (MCP: `check-rule-ids`) against the spec path with the project's rule files. Cited rule IDs that are missing are blocking; cited rule IDs marked deprecated are advisory. Otherwise, follow the markdown-only path.

6. Invoke `run-generator` (MCP: `run-generator`) against scripts/gen-spec-deps.sh to detect drift in the body inline links and frontmatter dependencies. A non-zero exit surfaces as an advisory drift finding — the pre-commit hook resolves these on the next commit. Otherwise, follow the markdown-only path.

7. Invoke `lint-markdown` (MCP: `lint-markdown`) against the markdown files in the feature directory. Each returned violation is surfaced as an advisory finding. Otherwise, follow the markdown-only path.

8. <!-- llm:assessSpecQuality --> For every loaded MUST-tier rule whose Verification trigger fires against the spec, request a semantic assessment via the extension point. The host responds with a structured finding carrying severity, rule-id, location, and message. MUST-tier findings join the Blocking tier in the rendered report. Otherwise, fall back to the markdown-only path.

9. <!-- llm:assessSpecQuality --> For every loaded SHOULD-tier rule whose Verification trigger fires against the spec, request a semantic assessment via the extension point. SHOULD-tier findings join the Advisory tier in the rendered report. Otherwise, fall back to the markdown-only path.

10. Parse the spec body for a `## Applicable Rules` section and collect every rule ID cited there. For each cited ID that did **not** appear in the set of rules whose Verification triggers fired in steps 8 or 9, emit an advisory finding: `Applicable Rules citation does not fire: {rule-id} is listed under ## Applicable Rules, but the rule's Verification trigger did not fire against any spec artifact. Either remove the citation, or extend the spec to bring the cited surface into scope.` Skip this step when the spec has no `## Applicable Rules` section. Citations whose IDs do not resolve to any loaded rule are handled earlier in step 5 and not reprocessed here. See **Applicable Rules citation consistency** in the markdown-only reference for the full semantics and the promotion criterion that governs when this check graduates from advisory to blocking.

11. Render the report (host responsibility): list hard-fail and blocking findings first, advisory findings next, then informational. For each finding, include what failed, what was expected, what was found, and a suggested fix. With `--fix` set, additionally revert any status-done spec whose review block has drifted to blocking — see the Review state drift section in the markdown-only reference below.

## Markdown-only reference

The full set of checks (frontmatter schema, spec integrity, artifact completeness, plan consistency, task consistency, scenario consistency, cross-spec references, review state drift, rule integrity, project-level consistency, severity classification, and report shape) is documented below for the markdown-only path. The numbered steps above invoke the mechanical primitives that automate the deterministic checks; the host applies the same checks against the markdown-only path when the runtime is unavailable.

### Frontmatter schema (hard fail)

For each spec file (`spec.md`):

- A YAML frontmatter block exists at the top of the file (delimited by `---` lines).
- The frontmatter parses as valid YAML.
- The `status` field is present and one of: `draft`, `clarified`, `planned`, `in-progress`, `done`.
- The `dependencies` field is present and is a list (empty list permitted).

For each scenario file (`scenarios/{slug}.md`):

- A YAML frontmatter block exists at the top of the file.
- The frontmatter parses as valid YAML.
- Either the `section` field (new schema) or the legacy `spec-ref` field is present and non-empty. New scenarios written by `/gov:ask` use `section`. Pre-017 scenarios keep `spec-ref` per the frozen-archaeology rule; either field satisfies the check.

Reference: the schema is canonically declared in `framework/constitution.md` §text-first-artifacts.

### Spec integrity (blocking)

- Acceptance criteria section exists with at least one checkbox item
- No placeholder or empty acceptance criteria
- Open questions consistent with status (`clarified` or later must have none). When this check fails — a spec at `clarified` / `planned` / `in-progress` with one or more open questions in the body — the spec is in the recovery state defined by spec 014. Suggested fix: run `/gov:clarify` (its recovery path will revert status to `draft` and walk the questions), or `/gov:ask` on a fresh question (which performs the back-edge automatically).
- No implementation code blocks (function signatures, package paths, language-specific snippets) in the spec — those belong in plan.md. Format examples, directory structures, and user-facing commands are acceptable when they define behavioral contracts.

### Artifact completeness (blocking)

- If status is `planned` or later: plan.md exists
- If status is `planned` or later and feature introduces or modifies domain entities or data structures: data-model.md exists
- If status is `planned` or later: tasks.md exists

### Plan consistency (blocking if plan exists)

- Plan references the spec
- Technical decisions section has at least one decision with rationale
- Affected files section lists specific file paths
- Plan does not contradict `specs/system.md`

### Task consistency (blocking if tasks exist)

- Tasks reference the plan
- Each task has a "done when" condition
- Tasks are numbered and ordered

### Scenario consistency (advisory)

- Every scenario file has Context and Behavior sections (frontmatter `spec-ref` is checked under Frontmatter schema above)
- Every scenario file in `scenarios/` has a corresponding task in `tasks.md`
- Scenario-linked tasks in `tasks.md` are marked complete if the spec status is `done`

### Cross-spec references (advisory)

- Event types mentioned in spec or plan align with `specs/events.md`
- Error codes follow the convention from `specs/errors.md`
- Data model definitions do not conflict with other specs' data-model.md files

### Review state drift (blocking)

For each spec at `status: done`, read the spec's frontmatter `review:` block:

- `review.last-run` is set to a non-null timestamp. If the `review:` block is **present** but `last-run` is missing or `null`, report `Review drift: done spec missing review — run /gov:review` (**blocking**)
- `review.blocking` is `false`. If `true`, report `Review drift: done spec has unresolved MUST violations — see review.md` (**blocking**)

**Grandfather rule.** A `done` spec whose frontmatter has no `review:` block at all is treated as pre-`/gov:review` and exempt from this check. The block is added by the spec template (so every newly-scaffolded spec ships with it) and by `/gov:review` on first run; its absence on a done spec means the spec reached done before `/gov:review` existed. Adopters who want retroactive review run `/gov:review` against the spec to populate the block, after which the spec is subject to the drift check on every subsequent analyze.

Specs not at `status: done` are silently exempt — the `review:` block is populated lazily on first `/gov:review` run, so its absence on `draft` / `clarified` / `planned` / `in-progress` specs is normal.

When `--fix` is set, this check additionally reverts affected specs from `done` to `in-progress` and emits a one-line notice for each (`reverted: specs/{feature}/{file} from done to in-progress — re-run /gov:review`). The revert is never silent; the notice is the point of the action. Re-running `/gov:review` on each reverted spec is left to the operator — auto-running it during `--fix` is out of scope. The grandfather rule applies under `--fix` too: pre-feature `done` specs with no `review:` block are never reverted.

### Rules (blocking and advisory)

Rules are the cross-cutting tier of the framework's three-tier requirement model (see §rules in `constitution.md`). Discover rule files by directory walk: list every `*.md` file in the project's rule-file directory and classify each by basename suffix per the closed-suffix policy declared in `constitution.md` §rules — `*-backend.md`, `*-frontend.md`, `*-cross.md`, or unrecognized. `/gov:analyze` loads **every** discovered file regardless of detected stack — citation verification spans surfaces, so a backend project that cites `FE-XSS-001` in a scenario covering HTML output still needs that citation verified.

For each file with an unrecognized suffix, emit one stdout line:

```text
rule file <name> has unrecognized suffix — loading for all stacks; rename to -backend.md, -frontend.md, or -cross.md
```

Then emit a single stdout line naming what was selected:

```text
loading rule files: <comma-separated basenames>
```

New rule files are introduced via their own feature spec; the suffix governs which stacks see them at `/gov:review` time, but `/gov:analyze` loads them all unconditionally.

For each loaded rule file:

- Every rule heading is level-3 and contains only the rule ID (no surrounding text)
- Every rule has the three required fields: a block-quoted Statement, `**Rationale:**` paragraph, and `**Verification:**` paragraph
- Every rule's ID matches the format declared in the rule file's introducing-spec data-model (`{BE|FE}-{CATEGORY}-{NNN}` for security files; `CFG-{CONST|ENV}-{NNN}` for configuration)
- No two rules in the same file share an ID

If any check above fails, the affected rule file is treated as unloadable for the remainder of this analyze pass.

#### Applicable Rules citation consistency (advisory)

The rule-citation audit runs in both directions:

- **Rule fires; not cited (existing).** For every loaded rule whose Verification trigger fires against the target spec, the per-rule semantic assessment (steps 8 and 9) emits a finding when the spec does not address the rule. This direction has been live since 008.
- **Cited; rule does not fire (new in 016).** For every rule ID listed under the spec's optional `## Applicable Rules` section that did NOT appear in the fired set from the existing direction, emit an advisory finding. The author either removes a decorative citation or extends the spec to bring the cited surface into scope; either resolution keeps the section honest.

The check assumes every citation resolves to a real rule — citations to unknown rule IDs are caught earlier by the rule-integrity check (step 5) and are not reprocessed here. Specs without an `## Applicable Rules` section are silently exempt (no citations to police).

**Severity:** advisory in v1. **Promotion criterion:** promote to blocking when a single `/gov:analyze --all` run reports 5 or more stale citations across the repo, with the threshold met on two consecutive runs (the second-run requirement guards against transient mid-implement states where citations land before the AC that exercises them). Until that threshold is sustained, the check stays advisory so forward-looking citations remain a usable planning signal rather than a friction point.

### Project-level consistency (advisory)

These checks span the project's installed command set and constitution rather than the target feature. They catch drift in the framework files `govern` ships, surfaced per the Drift Prevention principles in `constitution.md` §drift-prevention. Run once per `/gov:analyze` invocation regardless of which feature is targeted; with `--all`, run once before per-feature output.

Read inputs:

- `constitution.md` (already loaded by `/gov:target`)
- `.claude/commands/gov/help.md`
- The full set of `.md` files in `.claude/commands/gov/` (frontmatter only — do not read bodies for these checks)
- `.claude/commands/govern.md` if it exists (frontmatter only — the bootstrap installer lives outside the project namespace)

Checks:

- **Generator drift** — run `scripts/gen-readme-table.sh --dry-run` and `scripts/gen-help-tables.sh --dry-run` (when the scripts exist in the project). Non-empty diff means the README Feature Specs table or the help.md command tables are out of sync with their sources. Report each as `Generator out of sync: {script}; the next commit will resolve.`
- **Anchor resolution** — every §anchor reference in any installed command file (typically in "Reference: §X, §Y" Scope-Boundaries lines) resolves to a corresponding marker in `constitution.md`.
- **Command frontmatter completeness** — every `.md` file in the installed commands directory has a `description:` frontmatter field; the same check applies to `.claude/commands/govern.md` when that file exists. Files whose body documents an `$ARGUMENTS` parameter additionally have `argument-hint:`. Report missing fields; do not check value content.

These are advisory, not blocking — they signal framework drift that the project should resolve at its convenience. They do not prevent pipeline advancement on the target feature.

### Severity tiers

- **Hard fail (blocking)** — required-field violations and malformed frontmatter. The spec is not valid until these are fixed; pipeline advancement is blocked.
- **Blocking** — structural or content issues that must be fixed before the next pipeline gate fires.
- **Advisory** — issues that should be fixed but do not block advancement.
- **Informational** — observations that may warrant attention but are neither errors nor warnings.
