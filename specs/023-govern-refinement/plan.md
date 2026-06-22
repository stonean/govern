# 023 — `govern` Refinement Plan

Implements [023 — `govern` Refinement](spec.md).

## Overview

The work splits into two phases: (A) extend the `gvrn` runtime with two new primitives via a follow-on scenario on spec [022](../022-deterministic-runtime/spec.md), then (B) consolidate the slash command surface in `govern` and sweep the dependent prose. Phase A must ship before phase B because `framework/commands/amend.md` (rewritten in phase B) invokes the new primitives on its scenario branch.

Within phase B, the four scope-of-change items from the spec land in this order: `/configure` MCP allow-list (independent), lightweight track removal (constitution and command sweep), `/capture` merge into `/specify`, `/amend` rewrite. The prose sweep (`README`, `docs/introduction.md`, help tables, brownfield-process body) runs last because it depends on the final verb set.

## Technical Decisions

### Phase sequencing — invariant first, `gvrn` primitives second, `govern` consolidation third

Phase A has six tasks in this order:

1. **Add the MCP allow-list generator** (`scripts/gen-configure-mcp.sh`) and wire it into pre-commit. This establishes the invariant: every tool listed in `framework/runtime-tools.txt` is allowed by both `claude.md` and `auggie.md`. The generator is empty-load at this point — no new tools have been added yet — so it simply emits the current canonical set into the managed blocks.
2. Open the spec 022 follow-on scenario via `/elaborate` (today's verb, before consolidation lands). Spec 022 reopens `done → in-progress`.
3. Implement `create-scenario` primitive in `gvrn`.
4. Implement `append-task` primitive in `gvrn`.
5. Append `gov-rt:create-scenario` and `gov-rt:append-task` to `framework/runtime-tools.txt`. The pre-commit hook runs `gen-configure-mcp.sh` in the same commit, propagating the entries into `claude.md` and `auggie.md` automatically. Tag the `gvrn` release with the new binary; the framework files (including the updated configure sources) ship through the same govern commit.
6. Close the spec 022 scenario; spec 022 returns to `done`.

Only after Phase A completes does Phase B begin — the `framework/commands/amend.md` rewrite calls primitives that exist in `gvrn` and are already allowed by the configure files.

Rationale: deferring the primitive landing to mid-023 would force the `amend.md` rewrite to ship in lockstep with `gvrn` primitives in the same PR, blurring the per-spec scope and making rollback noisy. Two clean releases (`gvrn` first, `govern` second) keep each landing independently revertible. The generator-before-primitive ordering inside Phase A means the `runtime-tools.txt` → configure invariant holds at every commit on `main` — no transient window where the canonical allow set lags the published tool list.

### MCP allow-list generator — `scripts/gen-configure-mcp.sh`

New generator following the same pattern as `gen-readme-table.sh`, `gen-help-tables.sh`, `gen-spec-deps.sh`. Reads `framework/runtime-tools.txt`, emits per-agent permission entries into managed blocks in `framework/bootstrap/configure/claude.md` and `framework/bootstrap/configure/auggie.md`. Marker shape: `<!-- generated:mcp-allow:start -->` / `<!-- generated:mcp-allow:end -->`.

Per-host format mapping baked into the generator:

- **Claude**: each `gov-rt:<verb>-<noun>` tool becomes the permission entry `mcp__gov-rt__<verb>-<noun>`.
- **Auggie**: each tool becomes the entry shape Auggie's `toolPermissions` schema expects (verified against the current `auggie.md` source at implementation time; placeholder pattern documented as `mcp:gov-rt:<verb>-<noun>` until the schema is confirmed by reading the file).

Wired into `.githooks/pre-commit` alongside the existing generators so any change to `runtime-tools.txt` flows through to both agent permission files on the next commit. `scripts/lint-tool-coverage.sh` already verifies `runtime-tools.txt` against command-source references; the new generator adds the third leg (`runtime-tools.txt` → configure sources).

Critically, the generator lands **before** the two new primitives are appended to `runtime-tools.txt` (Phase A task 1 vs. task 5 ordering). This guarantees the configure files are updated in the same commit that adds the tool names — the invariant "every tool in `runtime-tools.txt` is allowed by the canonical configure set" holds at every commit on `main`.

### Lightweight track sweep strategy

One pass over every command source under `framework/commands/`. The sweep replaces three patterns: "Check for `spec.md` first, then `spec-and-plan.md`" → "Use `spec.md`."; "If the spec file is `spec-and-plan.md` (lightweight track), [branch]" → delete the branch; any prose referencing the lightweight track concept → delete or rewrite without the concept.

Constitution edit deletes §lightweight-track wholesale (lines around 181-194), removes the `spec-and-plan.md` row from the frontmatter schema table (§text-first-artifacts, line 369), and drops references in §spec-phase. §brownfield-process step 1 rewrites to point at `/specify` with sparse-AC guidance per the resolved question.

### Constitution slash-command sweep

Separate from the lightweight-track sweep, the constitution carries eight references to deleted verbs (`/capture`, `/elaborate`) outside the lightweight-track section. Each is rewritten to the post-consolidation verb in the same edit pass:

| Line | Section | Current text | Rewrite |
| --- | --- | --- | --- |
| 99 | §spec-lifecycle | `/elaborate` adds a scenario | `/amend` adds a scenario |
| 108 | §three-cycles (Brownfield) | `/capture` (sketch spec) → … → `/elaborate` to add a scenario | `/specify` (sketch spec) → … → `/amend` to add a scenario |
| 109 | §three-cycles (Reopen) | `/elaborate` adds a scenario | `/amend` adds a scenario |
| 260 | §scenario-promotion | `/specify` (for new behavior) or `/capture` (for another existing feature) | `/specify` (covers both) |
| 335 | §brownfield-process intro | The `/capture` command initializes a skeleton spec | The `/specify` command initializes a skeleton spec; sparse acceptance criteria are valid for brownfield use |
| 339 | §brownfield-process Capture phase | `/capture` drafts a skeleton spec | `/specify` drafts a skeleton spec |
| 350 | §brownfield-process Inbox integration | directs the user to run `/capture` … `/capture` creates specs | directs the user to run `/specify` … `/specify` creates specs |
| 409 | §runtime-boundary (principle 2 example list) | `/capture` sketching | `/specify` sketching |

The §brownfield-process anchor name is preserved (no cascading anchor reference updates) per the resolved question on brownfield messaging.

`framework/templates/spec/spec-and-plan.md` is deleted. The pre-commit generator catches references that survive the sweep (`scripts/lint-tool-coverage.sh` does not currently check for the literal string `spec-and-plan.md`, but a one-shot `grep` invocation in the validation pass covers it — extending `lint-tool-coverage.sh` is out of scope for this spec).

### `/capture` deletion strategy

`framework/commands/capture.md` is deleted. The Claude-commands generator (`scripts/gen-claude-commands.sh:44-55`) already prunes obsolete `.claude/commands/gov/*.md` files; deletion of the source flows through automatically on the next pre-commit. No manual `.claude/commands/gov/capture.md` removal needed.

`/specify` absorbs the brownfield use case by accepting sparse acceptance criteria as valid. The spec template already documents "At least one concrete, testable criterion is required before `/{project}:clarify` will advance the spec" — that gate stays. A brownfield-adopter who runs `/specify` with no AC lands at `draft` and can advance later as real work fills the section.

### `/amend` rewrite — classifier + scenario branch

Three additions to `framework/commands/amend.md`:

1. **Classifier prose** — a paragraph in the Instructions section names the heuristic signals (question signals, scenario signals, status tiebreaker) and instructs the host to apply them at the refinement step. No new LLM extension point; the host (LLM walking prose) executes the heuristic directly.
2. **Scenario branch** — invoked when the classifier (or user override) selects scenario. Adopts the decision tree currently in `framework/commands/elaborate.md` (does a spec exist? is the spec ambiguous? is the behavior situational?). Calls `gov-rt:create-scenario` to write `scenarios/{slug}.md` and `gov-rt:append-task` to extend `tasks.md`. On a `done` spec, calls `gov-rt:set-status` to reopen `done → in-progress`. Updates the session target to point at the new scenario (host responsibility — no session primitive).
3. **`flip` override** — the existing user-approves-the-refined-form gate displays "Recording as [question|scenario] — preview drafted at [...]". When the user enters `flip` (case-insensitive), the refinement loop redrafts the input under the alternate route and re-presents.

`framework/commands/elaborate.md` is deleted after the rewrite passes the parseability check.

### `/validate` → `/analyze` rename

Pure rename with no behavior change. The command's frontmatter `parity:` block stays unchanged under the new filename — the runtime parser is name-agnostic and primitive names (`gov-rt:validate-frontmatter`, etc.) are not touched. Three categories of work:

1. **File rename** — `git mv framework/commands/validate.md framework/commands/analyze.md`. Update the H1 from "# Validate" to "# Analyze".
2. **Reference sweep** — replace `/{project}:validate`, `/gov:validate`, and `validate.md` with their `analyze` counterparts across 11 files (per the audit count in the spec). Specifically `framework/commands/help.md`, `framework/commands/review.md`, `framework/constitution.md`, `framework/commands/validate.md` (becoming `analyze.md`), `framework/bootstrap/govern.md`, `framework/templates/spec/spec.md`, `framework/templates/project/project-readme.md`, `scripts/gen-help-tables.sh`, `scripts/lint-frontmatter.sh`, `README.md`, `docs/introduction.md`. The Claude-commands generator picks up the rename on its next run and prunes the old `.claude/commands/gov/validate.md`.
3. **Frozen-archaeology accommodation** — done specs under `specs/NNN-*/` retain their `/gov:validate` references. A single "Past Renames" note in `specs/README.md` records the change so readers can map old to new without per-spec signposts.

The rename lands in Phase B between `/elaborate` deletion (task 12) and the help-tables generator update (task 13). The help-tables generator must be updated in the same commit as the rename or pre-commit hooks will fail (the script tries to read `validate.md` which no longer exists).

### Description tightening for `/analyze` and `/review`

The rename ships alongside a description tightening on both commands so `/help` and any consumer that surfaces command descriptions makes the artifact-vs-code distinction obvious. Both descriptions adopt the parallel "Audit X — [enumeration]. [Behavior]." shape:

- `/analyze`: `Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. Read-only.`
- `/review`: `Audit code against rules — security, reuse, quality, efficiency, simplicity. Writes review.md; blocks done on MUST violations.`

`scripts/gen-help-tables.sh` reads `description:` frontmatter directly (see the `read_description` function at lines 39-59) so the help.md tables update automatically on the next generator run. No script changes needed beyond what the rename already requires.

### Help-tables generator update

`scripts/gen-help-tables.sh` currently builds five tables. The `elaborate_table` (lines 100-103) lists `/amend` and `/elaborate`; the `brownfield_table` (lines 105-109) lists `/capture`, `/log`, `/groom`. Changes:

- Rename marker `commands-elaborate` → `commands-refine` in the generator and in `framework/commands/help.md`.
- Drop the `/elaborate` row from the refine table; the table becomes a single-row table with `/amend` only.
- Drop the `/capture` row from the brownfield table.
- Rename the surrounding heading in `help.md` from "Elaborate (add precision)" to "Refine".

The script's existing splice logic handles marker renames without code changes — the markers are data, not hardcoded structure. The five table-build invocations in the script update inline.

### Migration check in `/govern` bootstrap

Prose-only step added to `framework/bootstrap/govern.md`. Runs after archive fetch/extract but before the manifest apply phase. Logic: walk `specs/*/spec-and-plan.md`; for each match, prompt the user with the source path and the proposed destination (`specs/{NNN-feature}/spec.md`); on confirm, run `mv`; on decline, log a warning that subsequent pipeline commands will fail on those features until renamed manually.

The check is idempotent — finds nothing on second run. No new primitive needed; the existing shell-out for `find` plus host-level `Edit`-equivalent file operations cover it. The completion message gains one line: "Migrated N `spec-and-plan.md` files to `spec.md`" (or omitted if N=0).

The changelog entry accompanying the gvrn / govern release pair documents the rename for adopters who upgrade without re-running `/govern`.

### Validation strategy

The acceptance criteria are concrete enough that a `grep`-based pass against the repo verifies most of them. Order of checks before declaring the spec done:

1. `git ls-files | xargs grep -l 'spec-and-plan'` → zero hits under `framework/`, `docs/`, `README.md`.
2. `git ls-files | xargs grep -l '/capture\b\|/elaborate\b'` → zero hits in the same scope.
3. `scripts/gen-help-tables.sh --dry-run` → reports "in sync".
4. `scripts/lint-tool-coverage.sh` → passes.
5. `scripts/gen-spec-deps.sh --dry-run` → reports "in sync".
6. `npx markdownlint-cli2 '**/*.md'` → passes.
7. `runtime` parseability check on rewritten `specify.md` and `amend.md` → passes.
8. `/gov:validate` on spec 023 → no hard-fail or blocking.
9. Markdown-only CI workflow (locally simulated) → passes.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `specs/022-deterministic-runtime/scenarios/ask-consolidation.md` | Create | Cross-spec impact deliverable: introduce `create-scenario` and `append-task` primitives. |
| `specs/022-deterministic-runtime/spec.md` | Modify | Status flip `done → in-progress` then `→ done` via the reopen cycle. |
| `runtime/src/primitives/create_scenario.rs` | Create | New gvrn primitive — writes `scenarios/{slug}.md` atomically. |
| `runtime/src/primitives/append_task.rs` | Create | New gvrn primitive — appends a numbered task block to `tasks.md` atomically. |
| `runtime/src/primitives/mod.rs` | Modify | Register the two new primitives. |
| `runtime/src/mcp/` | Modify | Register `gov-rt:create-scenario` and `gov-rt:append-task` MCP tools. |
| `runtime/Cargo.toml` | Modify | Version bump. |
| `runtime/CHANGELOG.md` | Modify | Note the two new primitives. |
| `runtime/tests/` | Create/Modify | Fixture tests for both primitives. |
| `framework/runtime-tools.txt` | Modify | Add `gov-rt:create-scenario` and `gov-rt:append-task`. |
| `framework/templates/spec/spec-and-plan.md` | Delete | Lightweight track template no longer used. |
| `framework/constitution.md` | Modify | Delete §lightweight-track; prune `spec-and-plan.md` references; rewrite §brownfield-process step 1; sweep eight deleted-verb references (§spec-lifecycle, §three-cycles, §scenario-promotion, §brownfield-process intro/Capture/Inbox, §runtime-boundary). |
| `framework/commands/specify.md` | Modify | Drop qualifying questions and `spec-and-plan.md` branch; always use `spec.md`. |
| `framework/commands/amend.md` | Modify | Add classifier heuristic, scenario branch, both back-edges, `flip` override. |
| `framework/commands/capture.md` | Delete | Consolidated into `/specify`. |
| `framework/commands/elaborate.md` | Delete | Consolidated into `/amend`. |
| `framework/commands/clarify.md` | Modify | Drop `spec-and-plan.md` fallback; rewrite "Spec File Detection" section. |
| `framework/commands/plan.md` | Modify | Drop `spec-and-plan.md` fallback and "skip plan creation" branch. |
| `framework/commands/implement.md` | Modify | Drop `spec-and-plan.md` references in setup, scope boundaries, gate. |
| `framework/commands/review.md` | Modify | Drop `spec-and-plan.md` reference in Inputs section. |
| `framework/commands/validate.md` → `framework/commands/analyze.md` | Rename + Modify | Rename for spec-driven-development standard alignment; update H1 to "# Analyze"; drop `spec-and-plan.md` references; tighten frontmatter schema text. |
| `scripts/lint-frontmatter.sh` | Modify | Update any direct `validate.md` reference to `analyze.md`. |
| `framework/commands/target.md` | Modify | Drop fallback; update Status → next action table (`done` → `/amend`). |
| `framework/commands/status.md` | Modify | Drop fallback; update Status → next action table (`done` → `/amend`). |
| `framework/commands/help.md` | Modify | Rename "Elaborate" heading to "Refine"; update marker names. |
| `framework/commands/groom.md` | Modify | Update references that point at `/elaborate` to point at `/amend`. |
| `framework/bootstrap/configure/claude.md` | Modify | Add managed block holding the MCP allow-list entries. |
| `framework/bootstrap/configure/auggie.md` | Modify | Add managed block holding the MCP allow-list entries (Auggie format). |
| `scripts/gen-help-tables.sh` | Modify | Rename `commands-elaborate` marker → `commands-refine`; drop `/elaborate` and `/capture` rows. |
| `scripts/gen-configure-mcp.sh` | Create | New generator: emits MCP allow-list into both configure sources from `runtime-tools.txt`. |
| `.githooks/pre-commit` | Modify | Wire `gen-configure-mcp.sh` into the hook. |
| `README.md` | Modify | Remove `/capture`, `/elaborate`, and lightweight-track references; update Slash Commands tables; drop `spec-and-plan.md` from templates table. |
| `AGENTS.md` | Modify | Drop `spec-and-plan` from the framework templates list (line 17). |
| `specs/README.md` | Modify | Remove the "Lightweight track detection" bullet from §Design Decisions (active-design statement being undone); add a "Past Renames" note recording `/validate → /analyze`. |
| `docs/introduction.md` | Modify | Sweep deleted-verb references (lines 24, 31, 32, 65, 66) and lightweight-track mentions; update the help-tables-mirroring table to match the new category set. |
| `framework/templates/project/agents.md` | Modify | Drop `spec-and-plan` from the templates list (line 43) and remove the `spec-and-plan.md` row from the templates description (line 46). |
| `framework/templates/project/project-readme.md` | Modify | Drop `spec-and-plan` from the templates list (line 26). |
| `framework/bootstrap/govern.md` | Modify | Add migration check for `spec-and-plan.md` files (see Phase B task 14); sweep all `spec-and-plan` and deleted-verb references (lines ~296, 311, 384, 442, 479, 481) including manifest entries for `capture.md`, `elaborate.md`, and `spec-and-plan.md`. |
| `specs/023-govern-refinement/tasks.md` | Create | Task breakdown. |

## Trade-offs

### Considered and rejected

- **Ship `gvrn` and `govern` changes in one combined release.** Rejected — couples two scopes that benefit from independent revert. The dependency points one way (`govern` needs the new `gvrn` primitives) so the staged release model adds no extra risk and isolates rollback.
- **Extend `lint-tool-coverage.sh` to grep for `spec-and-plan.md` as a sanity check.** Rejected for this spec — out of scope, and the validation pass's one-shot grep covers the same need without expanding the lint surface. If the literal string sneaks back in a future change, that's a job for an `/audit` command (deferred per the inbox).
- **Embed the MCP allow-list inline in `configure/claude.md` and `configure/auggie.md` (no generator).** Rejected — two-file copy with manual sync is exactly the drift pattern `gen-*.sh` scripts exist to prevent. The generator is ~50 lines and pays itself back the first time the runtime tool list changes.
- **Detect runtime presence in `/configure` and gate the MCP entries on it.** Rejected (also recorded as a Resolved Question on the spec) — adds complexity for no benefit; allow entries for unregistered tools are no-ops.
- **Migrate adopter `spec-and-plan.md` files automatically during `/govern` bootstrap without prompting.** Rejected — silent file renames on adopter projects violate the user-confirms-irreversible-actions principle. The prompt is one line; the user keeps the override.
- **Rename §brownfield-process anchor to something verb-neutral.** Rejected (Resolved Question) — anchor renames cascade through every command's Scope Boundaries citation; the term "brownfield" still describes the situation.

### Known limitations

- The migration check in `/govern` bootstrap relies on shell `find` and host-level file operations, not a new primitive. This is acceptable because the migration is a one-time event per adopter; adding a `migrate-spec-and-plan` primitive would carry maintenance cost long after the migration is done across the adopter base.
- Auggie's MCP permission format may differ from the placeholder shape (`mcp:gov-rt:<verb>-<noun>`) assumed here. The generator implementation reads `framework/bootstrap/configure/auggie.md` at implementation time to confirm the schema; if the placeholder is wrong, the generator updates to match. This is a small implementation-time correction, not a spec-time risk.
- The `flip` keyword in `/amend`'s refinement-approval gate may conflict with a legitimate question or scenario whose text starts with the literal word "flip". Mitigation: the gate matches `flip` only as a standalone command at the prompt — text that includes "flip" mid-sentence as part of a refined question or scenario is recognized as user-provided content via the existing approve/refine selector, not as the override keyword.
