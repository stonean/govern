---
status: done
dependencies: [022-deterministic-runtime]
review:
  last-run: 2026-05-19T00:00:00Z
  reviewed-against: e71bd410a7d8d6bdad82188bdc16a2af85ee945a
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 023 — `govern` Refinement

<!-- audit:ignore-introducing-drift:file -->
<!-- This spec IS the introducing spec for the /capture, /elaborate, /validate,
     and gov-rt: renames in the audit's RENAMED_TOKENS catalog. The old names
     are first-class subjects of the prose below (describing the rename action,
     the deleted command files, the comparative analysis that motivated the
     consolidation), not residual current-tense references. Per the introducing-
     drift script's file-scope exemption, this whole spec is skipped. -->

Consolidate the slash command surface so the pipeline feels less like a framework and more like a natural flow. Four changes, indivisible: drop the lightweight track entirely; merge `/capture` into `/specify`; merge `/elaborate` into `/amend`; rename `/validate` to `/analyze` to align with the emerging spec-driven-development standard.

## Motivation

`govern` exposes 14 user-visible slash commands across 5 categories. Two pairs do nearly the same thing, and one track concept exists because the model was wobbly on a question the framework already answers.

**`/specify` and `/capture` are the same artifact, two verbs.** Both create a numbered feature directory from a description and set it as session target. `/specify` asks lightweight-track qualifying questions and pushes for completeness; `/capture` doesn't. The user has to learn when to reach for which — that choice should be a property of the input, not a separate command.

**`/amend` and `/elaborate` are the same shape of action, two verbs.** Both append content to the targeted spec. `/amend` records a question and owns the `clarified|planned|in-progress → draft` back-edge. `/elaborate` records a scenario and owns the `done → in-progress` back-edge. To the user, the action is "I have a thing to add to this spec." Whether that thing is a question or a scenario is a classification problem the framework can solve, not a verb the user should pick.

**The lightweight track sneaks the wrong kind of work into the pipeline.** It was added to handle "small features," but the framework's own three-tier model (rules, specs, scenarios — [§rules](../../framework/constitution.md#rules), [§scenarios](../../framework/constitution.md#scenarios)) already says where trivial work belongs: a rule for cross-cutting concerns, a spec edit for feature-wide changes, a scenario for situational behavior. A one-line fix is none of those — it's a tier mismatch the lightweight track let through. Removing it makes every `specs/NNN-feature/` directory carry one signal: *this is a feature*. Smaller work routes to the right tier.

## Scope of change

### 1. Lightweight track removed

- Delete `framework/templates/spec/spec-and-plan.md`.
- Delete §lightweight-track from `framework/constitution.md`.
- Remove the "check for `spec.md` first, then `spec-and-plan.md`" fallback in every command that performs that detection.
- Drop the four qualifying questions from `/specify`.

Existing `spec-and-plan.md` files in adopter projects are frozen archaeology per [§drift-prevention](../../framework/constitution.md#drift-prevention) — `govern` never rewrites them. The framework simply stops emitting new ones. See **Open Questions** for how long pipeline commands continue to *read* them.

### 2. `/capture` consolidated into `/specify`

`/specify` becomes the single entry point for creating a feature. The brownfield vs greenfield distinction is input-driven, not flag-driven:

- A terse description with no acceptance criteria lands a `draft` spec with sparse AC — the `/capture` outcome.
- A rich description with concrete AC lands a `draft` spec with the AC filled in — the `/specify` outcome.

The user does not pick a mode. `/specify` always produces the same kind of artifact; richness scales with input.

`framework/commands/capture.md` is deleted.

### 3. `/elaborate` consolidated into `/amend`

`/amend` becomes the single entry point for adding to a targeted spec. During the existing refinement loop in `amend.md`, `/amend` classifies the input as a question or a scenario:

- **Question** (interrogative phrasing, undecided behavior) → record under `## Open Questions`. Continues to own the `clarified|planned|in-progress → draft` back-edge.
- **Scenario** (declarative phrasing, concrete behavior) → walk the existing `/elaborate` decision tree (does a spec exist? is the spec ambiguous? is it situational behavior?), create `scenarios/{slug}.md` from the scenario template, append a linked task to `tasks.md`. Now also owns the `done → in-progress` back-edge.

The classification surfaces during the user-approves-the-refined-form gate that already exists in `/amend` — the user can override the framework's choice with one click.

The current `/amend` refusal on `done` specs goes away: scenario routing handles `done` specs natively.

`framework/commands/elaborate.md` is deleted.

### 4. `/validate` renamed to `/analyze`

`/validate` audits artifacts against each other — the same role GitHub Spec Kit assigns to `/analyze`. Spec Kit's name has become the emerging-standard term for this gate, and the rename closes a gratuitous naming divergence. The command's purpose, scope boundaries, behavior, frontmatter `parity:` contract, and runtime primitive bindings do not change — only the file name, the H1, and every consumer reference are touched.

- Rename `framework/commands/validate.md` → `framework/commands/analyze.md`. Update the H1 from "# Validate" to "# Analyze".
- Sweep every reference to `/{project}:validate`, `/gov:validate`, and `validate.md` across `framework/`, `scripts/`, `.github/`, `README.md`, `AGENTS.md`, `docs/`, and `specs/README.md`. Replace with `/{project}:analyze`, `/gov:analyze`, and `analyze.md` respectively. Audit count at spec time: 28 occurrences across 11 files.
- Update `scripts/gen-help-tables.sh` — the pipeline-table builder references `validate.md`; rename to `analyze.md`.
- Update `scripts/lint-frontmatter.sh` if it references `validate.md` directly.
- Done specs under `specs/NNN-*/` are NOT rewritten — their references to `/gov:validate` stay per [§drift-prevention](../../framework/constitution.md#drift-prevention) ("Done specs are frozen archaeology"). The rename is recorded once in `specs/README.md` under a "Past Renames" cross-cutting note so historical references stay discoverable.
- `framework/commands/validate.md`'s frontmatter `parity:` block stays intact under the new filename — the runtime parser is name-agnostic and primitive names (`gov-rt:validate-frontmatter`, `gov-rt:resolve-anchor`, etc.) are not touched.

The rename ships alongside a description tightening for both `/analyze` and `/review` so the artifact-vs-code distinction lands at first glance in `/help`'s tables and in agent-host descriptions. The two commands keep their separate identities (no rename of `/review`) but their `description:` frontmatter is rewritten to a parallel "Audit X..." form:

- **`/analyze`**: `Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. Read-only.`
- **`/review`**: `Audit code against rules — security, reuse, quality, efficiency, simplicity. Writes review.md; blocks done on MUST violations.`

The parallel "Audit artifacts" / "Audit code" opening is the disambiguation surface. The rest of each description names what's actually compared. The descriptions feed `scripts/gen-help-tables.sh` and downstream consumers, so the tightening propagates automatically.

### 5. `/configure` allow entries for govern-owned state files

Adopters running pipeline commands are repeatedly prompted to confirm writes to files `govern` itself owns — most visibly `.govern.session.toml`, which `/target`, `/specify`, and `/amend`'s scenario branch all mutate as routine session-tracking steps. Asking the user to authorize a write to a file the framework owns is exactly the confirmation theater this refinement spec aims to remove.

`/configure` extends its canonical allow set with explicit per-path write entries for `govern`-owned state. Initial scope: the session file. The principle generalizes — any future `govern`-owned state file (e.g., additional sections of `.govern.toml`) gets the same explicit allow treatment as it's introduced.

- **Claude**: `Edit(.govern.session.toml)` and `Write(.govern.session.toml)` added alongside the existing bare `Edit` and `Write` entries. The explicit path entries make intent clear and disambiguate from any agent-host heuristic that otherwise prompts on the session-file path. (The pre-0.10.0 form used `{cli-config-dir}/{project}-session.json`; consolidated onto the repo-root `.govern.session.toml` in spec 022 task 40.)
- **Auggie**: the existing bare `save-file` and `str-replace-editor` allows already cover writes to any path including the session file. No Auggie-side change required; the verification step confirms no permission gap remains.

### 6. `/configure` allow statements for runtime MCP tools

`/configure` currently populates `settings.local.json` with bash, file, and web permissions, but has no entries for the optional runtime's MCP tools. Adopters who install `gvrn` and register it as an MCP server today hit the per-call permission prompt on every `gov-rt:*` invocation. The fix: extend the canonical allow set in `framework/bootstrap/configure/claude.md` (and the equivalent in `framework/bootstrap/configure/auggie.md`) to include every tool listed in `framework/runtime-tools.txt`.

Permissions are added unconditionally — `/configure` does not detect whether the runtime is installed. An allow entry for an unregistered MCP tool is a no-op; the alternative (gate by detected presence) adds complexity for no benefit and forces a second `/configure` run after later runtime installation.

The list is sourced from `framework/runtime-tools.txt` to avoid drift. Each tool name `gov-rt:<verb>-<noun>` maps to the host-specific permission entry shape (Claude Code: `mcp__gov-rt__<verb>-<noun>`; Auggie: per its tool-permission format). The exact format mapping is a plan-phase decision; the spec commits to the requirement that every tool in `runtime-tools.txt` is permitted by default after `/configure` runs.

## Acceptance Criteria

- [x] `framework/templates/spec/spec-and-plan.md` is deleted.
- [x] `framework/constitution.md` no longer contains §lightweight-track and no longer references `spec-and-plan.md` in any section.
- [x] `framework/constitution.md` references no deleted verbs (`/capture`, `/elaborate`) anywhere in its body. Every prior mention is rewritten to the post-consolidation verb (`/specify` for `/capture`; `/amend` for `/elaborate`). Sections known to require sweeping: §spec-lifecycle (back-edge ownership), §three-cycles (Brownfield and Reopen cycles), §scenario-promotion, §brownfield-process (intro, Capture phase, Inbox integration), §runtime-boundary (semantic-judgment example list).
- [x] `framework/commands/capture.md` is deleted; `.claude/commands/gov/capture.md` is regenerated as deleted.
- [x] `framework/commands/elaborate.md` is deleted; `.claude/commands/gov/elaborate.md` is regenerated as deleted.
- [x] `framework/commands/specify.md` no longer prompts qualifying questions and always copies the `spec.md` template.
- [x] `framework/commands/amend.md` classifies input as question or scenario, routes scenario inputs through the decision tree formerly in `/elaborate`, creates `scenarios/{slug}.md`, and appends a linked task to `tasks.md` on the scenario branch.
- [x] `framework/commands/amend.md` documents the classification heuristic in its prose Instructions section (question signals, scenario signals, status tiebreaker) and surfaces the chosen route in the refinement-approval gate with a one-input override (`flip`) that redrafts under the alternate route.
- [x] `framework/commands/amend.md` owns both back-edges: `clarified|planned|in-progress → draft` (on question record) and `done → in-progress` (on scenario record).
- [x] No command source under `framework/commands/` retains the `spec.md`-then-`spec-and-plan.md` detection fallback on either the read or write side.
- [x] `framework/bootstrap/govern.md` performs a one-pass migration check on every run: lists any `spec-and-plan.md` files under `specs/` and offers to rename each to `spec.md`. The changelog accompanying this spec's release documents the rename.
- [x] `framework/bootstrap/configure/claude.md` includes explicit `Edit({cli-config-dir}/{project}-session.json)` and `Write({cli-config-dir}/{project}-session.json)` entries in the canonical `permissions.allow` array so pipeline commands do not prompt on session-file writes.
- [x] `framework/bootstrap/configure/auggie.md`'s existing bare `save-file` and `str-replace-editor` allows are confirmed to cover session-file writes; no Auggie-side addition required (verified at implementation time).
- [x] `framework/bootstrap/configure/claude.md` includes a Claude-format permission entry for every MCP tool listed in `framework/runtime-tools.txt`, added to the canonical `permissions.allow` array unconditionally (no runtime-presence detection).
- [x] `framework/bootstrap/configure/auggie.md` includes an Auggie-format permission entry for every MCP tool listed in `framework/runtime-tools.txt`, added to the canonical permission set unconditionally.
- [x] The two configure sources draw from `framework/runtime-tools.txt` such that adding a new MCP tool to that file flows through to both agents on the next bootstrap (no manual edit per agent). The drift detection mechanism is a plan-phase choice (inline list with a generator that syncs from `runtime-tools.txt`, or runtime read at configure time).
- [x] A new scenario at `specs/022-deterministic-runtime/scenarios/ask-consolidation.md` introduces two primitives — `create-scenario` (writes `scenarios/{slug}.md` atomically with section frontmatter and body) and `append-task` (appends a numbered task block to `tasks.md` atomically). The scenario is created via `/elaborate` (current verb), spec 022 reopens to `in-progress`, and 022 returns to `done` after the scenario's task is implemented.
- [x] The `gvrn` release that ships with this spec exposes `create-scenario` and `append-task` as both CLI subcommands and MCP tools under the `gov-rt:` namespace. `framework/runtime-tools.txt` is updated to include both names.
- [x] `framework/commands/amend.md` (the rewritten version) invokes `create-scenario` and `append-task` on the scenario branch and falls back to host-side prose execution when the runtime is absent (per spec 022's markdown-only-path discipline).
- [x] `framework/commands/validate.md` is renamed to `framework/commands/analyze.md`; the H1 reads "# Analyze".
- [x] Across `framework/`, `scripts/`, `.github/`, `docs/`, `README.md`, `AGENTS.md`, and `specs/NNN-*/` (now uniformly live per the `living-specs` follow-on scenario), no file contains a current-usage reference to the old names `/validate`, `/gov:validate`, `/{project}:validate`, or `validate.md`. All current-usage references have been swept to the new names (`/analyze`, `/gov:analyze`, `/{project}:analyze`, `analyze.md`). Spec bodies of *introducing* specs (this one plus a few earlier ones tracked in the inbox) retain references to old names solely as historical-action descriptions in their own prose — these are accurate-as-of-then descriptions, not stale current-usage refs.
- [x] `scripts/gen-help-tables.sh` builds the pipeline table from `analyze.md`.
- [x] `specs/README.md` recorded `/validate → /analyze` under "Past Renames" at this spec's merge time so historical references in done specs were discoverable. The §Past Renames section was later deleted by the `living-specs` follow-on scenario, which removed the frozen-archaeology carve-out and brought done-spec bodies into the live-artifacts set; rename history now lives in git log.
- [x] `framework/commands/analyze.md`'s frontmatter `description:` reads exactly `Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. Read-only.`
- [x] `framework/commands/review.md`'s frontmatter `description:` reads exactly `Audit code against rules — security, reuse, quality, efficiency, simplicity. Writes review.md; blocks done on MUST violations.`
- [x] The "Audit artifacts" / "Audit code" parallelism is preserved verbatim — both descriptions begin with that exact phrase so the distinction is visible at first glance in `/help` tables and in any consumer that surfaces command descriptions.
- [x] `/gov:analyze` passes against this spec with no hard-fail or blocking findings (replacing the prior AC referencing `/gov:validate`).
- [x] `README.md`, `AGENTS.md`, `specs/README.md`, `framework/commands/help.md`, `docs/introduction.md`, `framework/templates/project/project-readme.md`, `framework/templates/project/agents.md`, `framework/bootstrap/govern.md`, and any other prose under `framework/`, `specs/`, and `docs/` no longer reference `/capture`, `/elaborate`, or the lightweight track. Help tables regenerate cleanly via `scripts/gen-help-tables.sh`.
- [x] `framework/constitution.md` §brownfield-process retains its three-phase structure ("Capture → incremental growth → promotion"); step 1 rewrites to point at `/specify` and explicitly notes sparse acceptance criteria are valid for brownfield use. The §brownfield-process anchor name is preserved (no cascading reference updates required).
- [x] The Status → next action tables in `framework/commands/target.md` and `framework/commands/status.md` point at `/amend` for the `done` row instead of `/elaborate`.
- [x] `scripts/lint-tool-coverage.sh` passes after the rewrites; the runtime's parseability check passes against the rewritten `framework/commands/specify.md` and `framework/commands/amend.md`.
- [x] The markdown-only CI workflow (`.github/workflows/markdown-only-pipeline.yml`) passes with the runtime absent from `PATH`; the runtime CI workflow continues to pass.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Impact on spec [022](../022-deterministic-runtime/spec.md) (deterministic runtime).** Two new primitives are needed — `create-scenario` and `append-task` — landing via a new follow-on scenario on spec 022. The scenario branch of `/amend` needs three deterministic operations: create `scenarios/{slug}.md`, append a linked task to `tasks.md`, and (on `done` specs) flip status via the existing `set-status` primitive. Existing primitives don't cover the first two: `substitute-templates` is shaped for bulk template→destination tree copy (bootstrap-grade complexity for a one-file write), and `mark-task` flips an existing checkbox but does not append. Falling back to a host-side `Edit` was rejected — it breaks the spec 022 pattern ("every mechanical step is a primitive") and bypasses the runtime's atomic-write semantics. The two new primitives: **`create-scenario`** (args: feature path, scenario slug, `section` frontmatter value, body content — writes the scenarios subdirectory if absent and the file atomically) and **`append-task`** (args: feature path, task title, "done when" text — computes the next task number from existing tasks.md and appends the new task block atomically). Per [§cross-spec-impact](../../framework/constitution.md#cross-spec-impact), the change is recorded in the affected spec: a new scenario lands at `specs/022-deterministic-runtime/scenarios/ask-consolidation.md` introducing the two primitives; spec 022 reopens `done → in-progress` while that scenario's task is worked, then returns to `done`. **Plan-phase order-of-operations note**: until 023 ships, the verb for creating the 022 scenario is `/elaborate`; after 023 ships, it would be `/amend`. The plan sequences `gvrn` primitive release first, then `framework/commands/amend.md` rewrite calling them, then 023 merge.
- **Brownfield entry point messaging.** Keep the Brownfield workflow section in `README.md`, `docs/introduction.md`, and `framework/constitution.md` §brownfield-process as a distinct mental model — the workflow itself (existing code, no specs, incremental capture, scenario decomposition over time) is real and valuable. Only the verb references change: every mention of `/capture` becomes `/specify` with explicit guidance that sparse acceptance criteria are expected and valid for brownfield use. §brownfield-process keeps its "Capture → incremental growth → promotion" three-phase structure; only step 1 rewrites to "Run `/specify` with whatever description you have. Sparse acceptance criteria are expected and valid — the spec gains precision through subsequent bug fixes, scenarios, and clarifications." `/log` and `/groom` remain brownfield-distinct (inbox flow); the Brownfield category in `help.md` keeps them. Rejected: merging §brownfield-process into §spec-phase (the workflow is structurally different enough to warrant its own section); renaming the §brownfield-process anchor (anchor renames cascade through every command's Scope Boundaries citation, and the term "brownfield" still describes the situation accurately even though the verb changed).
- **Help category collapse.** Rename the "Elaborate (add precision)" category in `help.md` to **"Refine"** and keep `/amend` as its sole entry. Folding `/amend` into Pipeline was rejected: Pipeline is forward-state-advancement (`draft → done`), and `/amend` mutates spec content and owns both back-edges — it's not on the forward axis. A single-row category signals the framework's economy ("the surface for 'add to a spec' is intentionally one verb") rather than indicating a missing entry. "Elaborate" no longer fits because the verb was the deleted command's name; "Refine" describes what `/amend` does (sharpen the spec by adding a question or scenario) and aligns with prose patterns in the constitution. Other candidates considered and rejected: "Augment" (clinical), "Add to spec" (longer than other category names), "Sharpen" (less idiomatic). Final help.md category set: Pipeline (`/specify`, `/clarify`, `/plan`, `/implement`, `/review`, `/validate`); Refine (`/amend`); Brownfield (`/log`, `/groom` — `/capture` moves into `/specify`); Orient (`/target`, `/status`, `/help`); Bootstrap (`/govern`, `/configure`).
- **Classification mechanism in `/amend`.** Heuristic in the prose Instructions section, with a user-overridable surface during the existing refinement-approval gate. Adding a new LLM extension point (`classifyAskInput`) was rejected as heavy infrastructure for a binary classification — it would add a fourth request/response schema alongside the three shipped in spec 022, with parser changes, parseability-check updates, and host-side integration in every agent host. Skipping classification entirely and asking the user every time was rejected on the "humans only do what computers can't" principle. The heuristic: **question signals** are terminal `?`, interrogative starters (how/what/when/should/could/would/is/are/do/does/can), and hedge words (maybe/perhaps/not sure); **scenario signals** are declarative or imperative phrasing ("when X happens, Y"; "X must Y"), concrete event/state language (on/when/if/after), and no terminal `?`. **Status tiebreaker**: on a `done` spec, scenario is the default when signals are mixed — under the merged model the back-edge from `done` is owned by the scenario path. **Override surface**: the refinement-approval gate displays "Recording as [question|scenario] — preview drafted at [`## Open Questions` entry | `scenarios/{slug}.md`]"; the user types `flip` to switch routes and the refinement loop redrafts under the new classification.
- **Read-fallback lifetime for adopter `spec-and-plan.md` files.** Rip the fallback immediately. `govern` is live-on-main per the README — there is no version a project can pin to, so a deprecation-cycle option has no load-bearing semantics. The read-side fallback exists in every pipeline command (`clarify`, `plan`, `implement`, `review`, `validate`, `target`, `status`, `amend`, `elaborate`); carrying it forward would undermine the simplicity goal of this spec. The adopter migration is trivial — `mv specs/NNN/spec-and-plan.md specs/NNN/spec.md` (the file format is identical; the lightweight track was a routing decision encoded in the filename, not a different schema). `done` specs with `spec-and-plan.md` are frozen archaeology and stay on disk untouched. **Mitigation requirement (folded into AC):** `/govern` bootstrap performs a one-pass migration check on every run, lists any `spec-and-plan.md` files it finds, and offers to rename them; the changelog accompanying this spec's release documents the rename. Adopters who upgrade blind without re-running `/govern` see "Spec does not exist. Run `/gov:specify` first." on the first pipeline command — the bootstrap check is what makes that case rare.
