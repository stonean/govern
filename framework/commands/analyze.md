---
description: Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. Read-only by default; --fix reverts a drifted done spec.
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

Audit a feature's spec, plan, tasks, and data model for consistency. Read-only by default — reports issues without modifying files. Use this to catch problems before the next pipeline gate fires. The one exception is the `--fix` flag, which reverts a status-`done` spec whose review block has drifted back to `in-progress` (the sole write this command performs; see Review state drift).

Renamed from `/validate` in spec 023 to align with the emerging spec-driven-development standard (GitHub Spec Kit uses `/analyze` for the same artifact-vs-artifact audit role). Complementary to `/{project}:review`, which audits **code** against rules.

## Context

Parse `$ARGUMENTS` for flags and an optional feature identifier:

- **Feature identifier** — a feature number, partial name, or full directory name. Overrides the session target.
- **`--all`** — scan all feature directories under `specs/` instead of a single target. Report results grouped by feature.

If `--all` is not present, use the feature identifier if provided, otherwise fall back to the session target from `.govern.session.toml`. If no target can be resolved, stop and tell the user to run `/{project}:target` first or use `--all`.

## Scope Boundaries

- Read-only by default — do NOT modify any files. The sole exception is `--fix`, which reverts a drifted `done` spec from `done` to `in-progress` via `set-status` (see Review state drift below); without `--fix`, no file is written.
- Read only files within the target feature's directory, the cross-spec files needed for reference checks (`specs/system.md`, `specs/events.md`, `specs/errors.md`, dependency spec files), and the project's installed command-source frontmatter for the project-level consistency section below (`{cli-config-dir}/commands/{project}/*.md` frontmatter only, plus `{cli-config-dir}/commands/govern.md` frontmatter for the bootstrap installer **if that file exists**). May invoke `scripts/gen-help-tables.sh --dry-run` and `scripts/gen-spec-deps.sh --dry-run` to surface generator drift, each only when that script exists in the project (`gen-help-tables.sh` is a govern-repo-only generator and is absent from adopters). Do NOT read source code or test files.
- Resolving the target spec's cross-service `references:` index additionally reads `.govern.toml` (the `[services]` registry) and the registered local checkouts' linked `spec.md` files — and nothing else; the canonical repo URL is **never fetched**. On the runtime path the host calls the resolve-references primitive per referencing spec; on the markdown-only path it reads those files with host file tools (see **Cross-service references** in the markdown-only reference below). This stays read-only.
- Reference: §spec-requirements, §grounding, §plan-phase, §tasks-phase, §readiness-check, §scenarios, §cross-spec-impact, §text-first-artifacts, §markdown-standards, §drift-prevention (constitution loaded by `/{project}:target` — do not re-read). See [030 — Cross-Service References](../../specs/030-cross-service-references/spec.md) for the reference semantics surfaced here.

## Instructions

> **For agent runtimes**: the Invoke steps below call the MCP tools of the optional gvrn runtime; the host-integration contract — bare↔prefixed tool names, lazy ToolSearch schema fetch, the no-shell-utilities rule, and the two-paths guarantee — lives once in the constitution, §runtime-host-integration. With no gvrn MCP server registered, walk the same prose using the host file-reading tools (Read, Edit, Write).

1. Invoke `read-spec` (with `include-body`) against the targeted feature to load frontmatter, sections, and the open-question count from the body. The result drives subsequent steps' tier classification (status governs which artifact-completeness checks apply); its parsed `sections` also feed step 12's `## Applicable Rules` scan, so no separate re-read of the body is needed.

2. Invoke `validate-frontmatter` against the spec path to check that the YAML block parses and that the required fields (status, dependencies) are present with valid values. `validate-frontmatter` emits each finding with `severity: blocking`; the host renders frontmatter findings in the report's hard-fail tier (the highest). The rest of the procedure still runs to surface every issue in a single pass.

3. Invoke `traverse-deps` against the feature to verify each dependency directory exists, carries a compatible status, and that the reachable dep subgraph is acyclic. Missing dependencies are blocking; an incompatible status (the edge's `compatible: false` — the dependency is below `planned`) is blocking when this spec is at `clarified` or later. `traverse-deps` reports per-edge `compatible` and `status` **unconditionally** (it never reads the consumer's own status), so apply that consumer-status conditioning host-side from the returned data rather than mapping the top-level `compatible` flag straight to a blocking finding. Any non-empty `cycles` entry — multi-node SCC or self-loop — is blocking. The cycle check is defense-in-depth that fires when the upstream `gen-spec-deps.sh` generator check (spec 017) was bypassed or stale frontmatter re-introduces an edge.

4. Invoke `resolve-anchor` against the spec path **with `markers-path` set to the constitution file** (`framework/constitution.md` in govern's own repo; `constitution.md` at the adopter repo root) to confirm every `§<name>` reference in the spec resolves to a `<!-- §name -->` marker in the constitution. The `markers-path` is essential: a spec carries no markers of its own, so resolving against the spec itself would flag *every* reference as unresolved; resolving against the constitution flags only a reference to a section that was renamed or restructured without updating callers. Unresolved anchors are advisory. With no gvrn runtime, walk the markdown-only path.

5. Invoke `check-rule-ids` against the spec path with the project's rule files. Cited rule IDs that are missing are blocking; cited rule IDs marked deprecated are advisory.

6. Invoke `run-generator` against scripts/gen-spec-deps.sh to detect drift in the body inline links and frontmatter dependencies. A non-zero exit surfaces as an advisory drift finding — the pre-commit hook resolves these on the next commit.

7. Invoke `lint-markdown` against the markdown files in the feature directory. Each returned violation is surfaced as an advisory finding.

8. Invoke `check-artifacts` against the feature to run the four residual deterministic check families: artifact completeness per status tier (plan.md/tasks.md required at planned+ — the *conditional* data-model.md requirement is a semantic judgment and stays on the markdown-only path), task numbering and done-when consistency (the "tasks reference the plan" link is a semantic judgment and stays on the markdown-only path), scenario→task mapping (a spent task pruned per §tasks-phase never counts against its scenario, and the family is **skipped entirely on a `done` spec**, whose tasks may already be pruned), and review-state drift on done specs. Each returned finding carries its family, severity tier, and location. The primitive mechanizes the **deterministic subset** of the markdown-only reference's Artifact completeness, Task consistency, Scenario consistency, and Review state drift sections; the semantic items noted above (data-model necessity, tasks-reference-plan, and a scenario's own Context/Behavior sections) stay on the markdown-only path, as does Command-frontmatter completeness (Project-level consistency), which reads the host's command directory the runtime does not own.

9. <!-- llm:assessSpecQuality --> For every loaded MUST-tier rule whose Verification trigger fires against the spec, request a semantic assessment via the extension point. The host responds with a structured finding carrying severity, rule-id, location, and message. MUST-tier findings join the Blocking tier in the rendered report. Otherwise, fall back to the markdown-only path.

10. <!-- llm:assessSpecQuality --> For every loaded SHOULD-tier rule whose Verification trigger fires against the spec, request a semantic assessment via the extension point. SHOULD-tier findings join the Advisory tier in the rendered report. Otherwise, fall back to the markdown-only path.

<!-- audit:ignore-promotion -->
11. Resolve the target spec's cross-service **references** (deterministic; advisory). When the spec's derived `references:` index is non-empty, resolve each entry by the procedure in the **Cross-service references** section of the markdown-only reference below — the resolve-references primitive on the runtime path; host file tools (read `.govern.toml` and the linked checkout) on the markdown-only path. A **broken** outcome — the service is registered and its checkout reachable, but the target spec does not resolve (renamed, moved, deleted, or mistyped upstream, or a malformed URL) — is an **Advisory** finding, the cross-repo analog of a broken sibling link. The informational unknowns are **not** findings: **unregistered** (the repo matches no `[services]` entry — surface a pointer to `/{project}:link` to register the service), **not checked out**, and **status unreadable** each record what could not be proven, without flagging a defect. Skip this step when the target spec declares no references.

<!-- audit:ignore-promotion -->
12. Parse the spec body for a `## Applicable Rules` section and collect every rule ID cited there. For each cited ID that did **not** appear in the set of rules whose Verification triggers fired in steps 9 or 10, emit an advisory finding: `Applicable Rules citation does not fire: {rule-id} is listed under ## Applicable Rules, but the rule's Verification trigger did not fire against any spec artifact. Either remove the citation, or extend the spec to bring the cited surface into scope.` Skip this step when the spec has no `## Applicable Rules` section. Citations whose IDs do not resolve to any loaded rule are handled earlier in step 5 and not reprocessed here. See **Applicable Rules citation consistency** in the markdown-only reference for the full semantics and the promotion criterion that governs when this check graduates from advisory to blocking.

<!-- audit:ignore-promotion -->
13. Scan the spec body (loaded in step 1) and `plan.md` (read it if present) for **ungrounded factual claims about the existing system** — assertions about how current code behaves, what a schema or interface contains, or what an external system returns, stated as fact but carrying neither a citation to a primary source (a `path:line` reference, a named query, a command, or a link to a substantiating artifact) nor an explicit assumption / Open Question marker. Descriptive claims about existing reality need grounding; **prescriptive requirements** about the feature under design (what it MUST do) are contracts, not claims, and are never flagged — the descriptive-vs-prescriptive call is the semantic judgment this step turns on. This is a *form* check: do NOT read source code to confirm a claim (out of scope; see Scope Boundaries), only verify the artifact sources or hedges it. Apply to the spec body at status `clarified` or later and to `plan.md` at `planned` or later; skip on a `draft` spec. Emit each as an **Advisory** finding per the **Grounding** section of the markdown-only reference below.

<!-- audit:ignore-promotion -->
14. Render the report (host responsibility): list hard-fail and blocking findings first, advisory findings next, then informational. For each finding, include what failed, what was expected, what was found, and a suggested fix. With `--fix` set, additionally revert any status-done spec whose review block has drifted to blocking — the guarded set-status revert (`from: done`, `to: in-progress`), detailed in the Review state drift section in the markdown-only reference below.

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
- Either the `section` field (new schema) or the legacy `spec-ref` field is present and non-empty. New scenarios written by `/{project}:amend` use `section`; pre-017 scenarios written before `section` existed may still carry `spec-ref`. Either field satisfies the check.

Reference: the schema is canonically declared in `framework/constitution.md` §text-first-artifacts.

### Spec integrity (blocking)

- Acceptance criteria section exists with at least one checkbox item
- No placeholder or empty acceptance criteria
- Open questions consistent with status (`clarified` or later must have none). When this check fails — a spec at `clarified` / `planned` / `in-progress` with one or more open questions in the body — the spec is in the recovery state defined by spec 014. Suggested fix: run `/{project}:clarify` (its recovery path will revert status to `draft` and walk the questions), or `/{project}:amend` on a fresh question (which performs the back-edge automatically).
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

### Grounding (advisory)

Enforces `constitution.md` §grounding against the spec and plan bodies: a factual claim about the **existing system** must be grounded — either cited to a primary source or marked as an assumption — rather than asserted from conjecture. `/{project}:analyze` checks the *form* of grounding (is the claim sourced or hedged), not its truth: confirming a claim against the code would require reading source, which is out of this command's scope (see Scope Boundaries). The truth check is the agent's job at authoring time (§grounding) and `/{project}:review`'s job against code.

Applies to the spec body at status `clarified` or later, and to `plan.md` at `planned` or later (when it exists). `draft` specs are exempt — claims are still forming, the same way open questions are tolerated only at `draft`.

Flag a passage when **all** hold:

- It is a **descriptive claim about existing reality** — how current code behaves, what a schema/table/column/interface contains, what an external service returns, what a config value is. Prescriptive requirements about the feature under design ("the endpoint MUST reject unsigned requests") are contracts, not claims, and are never flagged. The descriptive-vs-prescriptive call is the semantic judgment this check turns on.
- It is **asserted as fact** — not already framed as an assumption, an open question, or a proposal.
- It carries **no grounding** — no citation to a primary source (a `path:line` reference, a named query, a command and its output, or a link to an artifact that substantiates it) and no explicit assumption / Open Question marker.

Suggested fix per finding: ground the claim (read the code, query the dev database, run the command — then cite the source), or, when no reachable source can settle it, restate it as an assumption (in a plan) or an Open Question (in a spec, which reverts the spec to `draft` per §spec-lifecycle).

**Severity:** advisory in v1 — grounding is a semantic judgment with false-positive risk, and forcing it blocking before the signal is proven would erode trust the way any noisy gate does. **Promotion criterion:** promote to blocking when a single `/{project}:analyze --all` run reports 5 or more ungrounded claims across the repo on two consecutive runs (the second-run requirement guards against transient mid-authoring states where a claim lands before its source is wired in). This mirrors the **Applicable Rules citation consistency** promotion path below.

### Scenario consistency (advisory)

- Every scenario file has Context and Behavior sections (frontmatter `spec-ref` is checked under Frontmatter schema above)
- Every scenario file in `scenarios/` has a corresponding task in `tasks.md` **only while that task is still pending**. `tasks.md` is an ephemeral tracking artifact (§tasks-phase) that `/{project}:prune` reduces once work is complete, so a *missing* scenario task is a finding only when the scenario is unimplemented; do NOT flag a scenario whose task was completed and pruned, and do NOT flag any scenario under a `done` spec (its tasks may have been pruned, or the file reset to template state). The durable record of an implemented scenario is the scenario file, the code, and git history — not a retained checkbox.
- A scenario task that is *still present* in `tasks.md` is marked complete when the spec status is `done`; an absent (pruned) scenario task is not treated as incomplete.

### Cross-spec references (advisory)

- Event types mentioned in spec or plan align with `specs/events.md`
- Error codes follow the convention from `specs/errors.md`
- Data model definitions do not conflict with other specs' data-model.md files

### Cross-service references (advisory)

A spec's derived `references:` frontmatter index records each cross-service reference as a `{service, spec}` pair, harvested from body links to a registered service's canonical repo URL (see [030 — Cross-Service References](../../specs/030-cross-service-references/spec.md)). On the runtime path the `resolve-references` primitive classifies each entry; when the runtime is unavailable, classify each entry with the host's file tools — read `.govern.toml` and the linked spec directly, with **no shell-pipeline substitution**. The repo URL is identity and navigation only and is **never fetched**; status is read from the local checkout.

For each `{service, spec}` entry, in index order, decide the outcome by what can be proven, then map it to a severity:

- **`broken`** (Advisory finding) — the service is registered in `.govern.toml` `[services]` and its checkout `path` is reachable, but the target `spec.md` does not resolve (renamed, moved, deleted, or mistyped upstream, or the URL is malformed). A provable defect in *this* spec — the cross-repo analog of a broken sibling link — surfaced on every run as an **Advisory** finding (non-blocking, because references are informative and never load-bearing). Suggested fix: correct or remove the reference link in the spec body, then re-run the harvest generator.
- **`unregistered`** (informational, not a finding) — the reference's repo matches no `[services]` entry. A plain navigational link; status was not attempted, so nothing is broken. Surface it with a pointer to `/{project}:link` to register the service.
- **`not-checked-out`** (informational, not a finding) — registered, but the local `path` is missing or not a usable checkout. Nothing can be proven without a checkout, so this is **never** reported as broken.
- **`status-unreadable`** (informational, not a finding) — the target file exists but its `status` cannot be read (no or malformed frontmatter, missing or out-of-set `status`, or the link targets a scenario, which has no status). The defect is upstream's, not this spec's.
- **`ok`** (no finding) — the reference resolves and the linked lifecycle `status` is readable. A clean reference.

The load-bearing line is **provably broken** (a finding) versus **can't check** (an informational unknown): a broken link never hides behind a benign unknown, and an unknown is never escalated to a defect. This classification matches the `resolve-references` primitive and the `/{project}:status` readout exactly — the three surfaces share one contract and none wraps another.

### Review state drift (blocking)

For each spec at `status: done`, read the spec's frontmatter `review:` block:

- `review.last-run` is set to a non-null timestamp. If the `review:` block is **present** but `last-run` is missing or `null`, report `Review drift: done spec missing review — run /{project}:review` (**blocking**)
- `review.blocking` is `false`. If `true`, report `Review drift: done spec has unresolved MUST violations — see review.md` (**blocking**)

**Grandfather rule.** A `done` spec whose frontmatter has no `review:` block at all is treated as pre-`/{project}:review` and exempt from this check. The block is added by the spec template (so every newly-scaffolded spec ships with it) and by `/{project}:review` on first run; its absence on a done spec means the spec reached done before `/{project}:review` existed. Adopters who want retroactive review run `/{project}:review` against the spec to populate the block, after which the spec is subject to the drift check on every subsequent analyze.

Specs not at `status: done` are silently exempt — the `review:` block is populated lazily on first `/{project}:review` run, so its absence on `draft` / `clarified` / `planned` / `in-progress` specs is normal.

When `--fix` is set, this check additionally reverts affected specs from `done` to `in-progress` — via `set-status` (`from: done`, `to: in-progress`) on the runtime path, a direct frontmatter edit on the markdown-only path — and emits a one-line notice for each (`reverted: specs/{feature}/{file} from done to in-progress — re-run /{project}:review`). The revert is never silent; the notice is the point of the action. Re-running `/{project}:review` on each reverted spec is left to the operator — auto-running it during `--fix` is out of scope. The grandfather rule applies under `--fix` too: pre-feature `done` specs with no `review:` block are never reverted.

### Rules (blocking and advisory)

Rules are the cross-cutting tier of the framework's three-tier requirement model (see §rules in `constitution.md`). Discover rule files by directory walk: list every `*.md` file in the project's rule-file directory and classify each by basename suffix per the closed-suffix policy declared in `constitution.md` §rules — `*-backend.md`, `*-frontend.md`, `*-cross.md`, or unrecognized. `/{project}:analyze` loads **every** discovered file regardless of detected stack — and regardless of the project's `[rules] surfaces` setting (`govern.md` §Project Configuration) — because citation verification spans surfaces: a backend project that cites `FE-XSS-001` in a scenario covering HTML output still needs that citation verified. `[rules] surfaces` scopes `/{project}:review` enforcement only (which surface's rules are checked against code); it never prunes the rule-file set `/{project}:analyze` loads for citation resolution.

For each file with an unrecognized suffix, emit one stdout line:

```text
rule file <name> has unrecognized suffix — loading for all stacks; rename to -backend.md, -frontend.md, or -cross.md
```

Then emit a single stdout line naming what was selected:

```text
loading rule files: <comma-separated basenames>
```

New rule files are introduced via their own feature spec; the suffix governs which stacks see them at `/{project}:review` time, but `/{project}:analyze` loads them all unconditionally.

For each loaded rule file:

- Every rule heading is level-3 and contains only the rule ID (no surrounding text)
- Every rule has the three required fields: a block-quoted Statement, `**Rationale:**` paragraph, and `**Verification:**` paragraph
- Every rule's ID matches the format declared in the rule file's introducing-spec data-model (`{BE|FE}-{CATEGORY}-{NNN}` for security files; `CFG-{CONST|ENV}-{NNN}` for configuration)
- No two rules in the same file share an ID

If any check above fails, the affected rule file is treated as unloadable for the remainder of this analyze pass.

#### Applicable Rules citation consistency (advisory)

The rule-citation audit runs in both directions:

- **Rule fires; not cited (existing).** For every loaded rule whose Verification trigger fires against the target spec, the per-rule semantic assessment (steps 9 and 10) emits a finding when the spec does not address the rule. This direction has been live since 008.
- **Cited; rule does not fire (new in 016).** For every rule ID listed under the spec's optional `## Applicable Rules` section that did NOT appear in the fired set from the existing direction, emit an advisory finding. The author either removes a decorative citation or extends the spec to bring the cited surface into scope; either resolution keeps the section honest.

The check assumes every citation resolves to a real rule — citations to unknown rule IDs are caught earlier by the rule-integrity check (step 5) and are not reprocessed here. Specs without an `## Applicable Rules` section are silently exempt (no citations to police).

**Severity:** advisory in v1. **Promotion criterion:** promote to blocking when a single `/{project}:analyze --all` run reports 5 or more stale citations across the repo, with the threshold met on two consecutive runs (the second-run requirement guards against transient mid-implement states where citations land before the AC that exercises them). Until that threshold is sustained, the check stays advisory so forward-looking citations remain a usable planning signal rather than a friction point.

### Project-level consistency (advisory)

These checks span the project's installed command set and constitution rather than the target feature. They catch drift in the framework files `govern` ships, surfaced per the Drift Prevention principles in `constitution.md` §drift-prevention. Run once per `/{project}:analyze` invocation regardless of which feature is targeted; with `--all`, run once before per-feature output.

Read inputs:

- `constitution.md` (already loaded by `/{project}:target`)
- `{cli-config-dir}/commands/{project}/help.md`
- The full set of `.md` files in `{cli-config-dir}/commands/{project}/` (frontmatter only — do not read bodies for these checks)
- `{cli-config-dir}/commands/govern.md` if it exists (frontmatter only — the bootstrap installer lives outside the project namespace)

Checks:

- **Generator drift** — run `scripts/gen-help-tables.sh --dry-run` (via the `run-generator` primitive on the runtime path, the same way step 6 runs `gen-spec-deps.sh`; when the script exists in the project). Non-empty diff means the help.md command tables are out of sync with their sources. Report it as `Generator out of sync: {script}; the next commit will resolve.`
- **Anchor resolution** — every `§<name>` reference in any installed command file (typically in `Reference: §<first>, §<second>` Scope-Boundaries lines) resolves to a corresponding marker in `constitution.md`.
- **Command frontmatter completeness** — every `.md` file in the installed commands directory has a `description:` frontmatter field; the same check applies to `{cli-config-dir}/commands/govern.md` when that file exists. Files whose body documents an `$ARGUMENTS` parameter additionally have `argument-hint:`. Report missing fields; do not check value content.

These are advisory, not blocking — they signal framework drift that the project should resolve at its convenience. They do not prevent pipeline advancement on the target feature.

### Severity tiers

- **Hard fail (blocking)** — required-field violations and malformed frontmatter. The spec is not valid until these are fixed; pipeline advancement is blocked.
- **Blocking** — structural or content issues that must be fixed before the next pipeline gate fires.
- **Advisory** — issues that should be fixed but do not block advancement.
- **Informational** — observations that may warrant attention but are neither errors nor warnings.
