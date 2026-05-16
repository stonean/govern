# 023 — `govern` Refinement Tasks

Tasks derived from the [plan](plan.md). Complete in order. Phase A (invariant + `gvrn` primitives) must ship as a tagged `gvrn` release before any phase B work begins. Within Phase A, the configure MCP allow-list generator lands first so the `runtime-tools.txt` → configure invariant holds at every commit.

## Phase A — invariant + `gvrn` primitives via spec 022 follow-on

### 1. Add MCP allow-list generator (invariant foundation)

- [x] Create `scripts/gen-configure-mcp.sh`. Reads `framework/runtime-tools.txt`. Emits Claude-format entries (`mcp__gov-rt__<verb>-<noun>`) into a managed block in `framework/bootstrap/configure/claude.md` (markers `<!-- generated:mcp-allow:start -->` / `<!-- generated:mcp-allow:end -->`). Emits Auggie-format entries into the same-named markers in `framework/bootstrap/configure/auggie.md`.
- [x] Read `framework/bootstrap/configure/auggie.md` once to confirm the Auggie `toolPermissions` schema; update the generator's per-host mapping if it differs from the placeholder pattern.
- [x] Add the marker blocks to both configure source files at the appropriate location in the canonical allow-list section.
- [x] Add `gen-configure-mcp.sh` to `.githooks/pre-commit` after the other generators.
- [x] Run the generator and verify both configure files contain entries for every `gov-rt:*` tool currently listed in `runtime-tools.txt` (pre-primitive-add: 21 entries).

Done when: dry-run reports "in sync" after a clean run; both configure files contain the managed-block entries for every existing tool; pre-commit hook executes the generator without error.

### 1b. Session-file allow entries

- [x] Edit `framework/bootstrap/configure/claude.md`: add explicit `Edit({cli-config-dir}/{project}-session.json)` and `Write({cli-config-dir}/{project}-session.json)` to the canonical `permissions.allow` array. Place them next to the existing bare `Edit` / `Write` entries (or in a "Govern state files" sub-section, matching the prose grouping pattern used elsewhere in the file).
- [x] Confirm `framework/bootstrap/configure/auggie.md`'s existing `save-file` and `str-replace-editor` allows cover session-file writes (read once, document if a path-restriction syntax is needed instead — Auggie's existing bare-allow pattern for these tools should suffice).
- [x] Run `scripts/gen-claude-commands.sh` and confirm `.claude/commands/gov/configure.md` regenerates with the new entries.
- [x] Run `npx markdownlint-cli2` against both configure sources — passes.

Done when: claude.md's permissions.allow contains the two explicit session-file entries; auggie.md is verified to need no addition; downstream `.claude/commands/gov/configure.md` reflects the change.

### 2. Open the cross-spec scenario on spec 022

- [x] Run `/gov:elaborate` against spec 022 (`/gov:target 022`) to create `specs/022-deterministic-runtime/scenarios/ask-consolidation.md`.
- [x] Fill the scenario's Context, Behavior, and Edge Cases sections describing the two new primitives and their argument/result shapes.
- [x] Confirm `/gov:elaborate` reopens spec 022's status `done → in-progress` and appends a linked task to 022's `tasks.md`.

Done when: scenario file exists, frontmatter `section` references spec 022's "Follow-on scenarios" section, 022's status is `in-progress`, and 022's `tasks.md` has the new task block.

### 3. Implement `create-scenario` primitive

- [ ] Create `runtime/src/primitives/create_scenario.rs` implementing the primitive: args (feature path, scenario slug, section frontmatter value, body content); writes `scenarios/{slug}.md` atomically via tempfile + rename; creates the scenarios subdirectory if absent.
- [ ] Register the primitive in `runtime/src/primitives/mod.rs`.
- [ ] Expose as MCP tool `gov-rt:create-scenario` via the runtime's MCP registration.
- [ ] Expose as CLI subcommand `runtime create-scenario` (or the equivalent existing primitive-CLI pattern).
- [ ] Add fixture tests covering: happy path, scenarios directory absent, slug conflict (file already exists), atomic-write semantics on simulated crash mid-write.

Done when: `cargo test` passes; the MCP tool responds correctly under fixture invocation; the CLI subcommand exits 0 on success and non-zero on conflict.

### 4. Implement `append-task` primitive

- [ ] Create `runtime/src/primitives/append_task.rs` implementing the primitive: args (feature path, task title, "done when" text); reads existing `tasks.md` to compute next task number; appends a new numbered task block atomically via tempfile + rename; creates `tasks.md` with a heading if it does not exist.
- [ ] Register in `runtime/src/primitives/mod.rs`.
- [ ] Expose as MCP tool `gov-rt:append-task`.
- [ ] Expose as CLI subcommand.
- [ ] Add fixture tests covering: empty `tasks.md`, existing tasks (number-1 increment), missing `tasks.md` (creates with heading), atomic-write semantics on simulated crash mid-write.

Done when: `cargo test` passes; the MCP tool responds correctly under fixture invocation; the CLI subcommand exits 0 on success.

### 5. Update `runtime-tools.txt` and tag release

- [ ] Append `gov-rt:create-scenario` and `gov-rt:append-task` to `framework/runtime-tools.txt`.
- [ ] Stage the change and commit. The pre-commit hook runs `gen-configure-mcp.sh` automatically; verify the commit includes updates to `framework/bootstrap/configure/claude.md` and `framework/bootstrap/configure/auggie.md` adding the two new entries to the managed block.
- [ ] Run `scripts/lint-tool-coverage.sh` and confirm pass.
- [ ] Bump `runtime/Cargo.toml` version (next minor — primitive additions are non-breaking).
- [ ] Update `runtime/CHANGELOG.md` with the two new primitives and reference to spec 022 scenario.
- [ ] Tag `gvrn-vX.Y.0` and confirm GitHub release artifacts publish.

Done when: release artifacts are downloadable; `gvrn --version` reports the new version; `framework/bootstrap/configure/claude.md` and `framework/bootstrap/configure/auggie.md` both contain allow entries for `gov-rt:create-scenario` and `gov-rt:append-task`.

### 6. Close the spec 022 scenario

- [ ] Run `/gov:implement` against the ask-consolidation scenario in spec 022 (the scenario task already appended in task 1); mark the linked task complete in 022's `tasks.md`.
- [ ] Run `/gov:review` against spec 022 (if its review block has gone stale per the new primitives).
- [ ] Confirm spec 022 returns `in-progress → done` via the standard pipeline gate.

Done when: spec 022's status is `done` again with no blocking review findings.

## Phase B — `govern` consolidation

### 7. Constitution edits — lightweight track removal

- [ ] Delete the §lightweight-track section from `framework/constitution.md`.
- [ ] Remove every `spec-and-plan.md` mention in §spec-phase, §text-first-artifacts (frontmatter schema table, currently line ~369), and any other section. Use `grep -n 'spec-and-plan' framework/constitution.md` as the checklist.
- [ ] Rewrite §brownfield-process step 1 to: "Run `/specify` with whatever description you have. Sparse acceptance criteria are expected and valid — the spec gains precision through subsequent bug fixes, scenarios, and clarifications."
- [ ] Verify the anchor `§lightweight-track` is no longer referenced anywhere via `grep -rn '§lightweight-track' framework/ specs/ docs/`.

Done when: `grep -n 'spec-and-plan' framework/constitution.md` returns no hits; no command file or doc references `§lightweight-track`.

### 7b. Constitution edits — slash command sweep

Rewrite every reference to the deleted verbs (`/capture`, `/elaborate`) in `framework/constitution.md`. The plan's "Constitution slash-command sweep" table enumerates the eight known sites with the exact rewrite for each.

- [ ] Edit §spec-lifecycle (line ~99): rewrite the `/elaborate` back-edge owner to `/ask`.
- [ ] Edit §three-cycles Brownfield (line ~108): rewrite `/capture` → `/specify`; rewrite `/elaborate` → `/ask`.
- [ ] Edit §three-cycles Reopen (line ~109): rewrite `/elaborate` → `/ask`.
- [ ] Edit §scenario-promotion (line ~260): collapse "`/specify` (for new behavior) or `/capture` (for another existing feature)" to "`/specify` (covers both)".
- [ ] Edit §brownfield-process intro (line ~335): rewrite "`/capture` command initializes a skeleton spec" to "`/specify` command initializes a skeleton spec; sparse acceptance criteria are valid for brownfield use".
- [ ] Edit §brownfield-process Capture phase (line ~339): rewrite `/capture` → `/specify`.
- [ ] Edit §brownfield-process Inbox integration (line ~350): rewrite the two `/capture` references to `/specify`.
- [ ] Edit §runtime-boundary (line ~409, principle 2 example list): rewrite `/capture` sketching → `/specify` sketching.
- [ ] Verify `grep -n '/capture\b\|/elaborate\b' framework/constitution.md` returns no hits.
- [ ] Re-run `scripts/lint-tool-coverage.sh` — passes (no broken tool references introduced).

Done when: the grep for `/capture\b` and `/elaborate\b` against `framework/constitution.md` returns zero hits; tool-coverage lint is clean.

### 8. Lightweight track — command source sweep

- [ ] Delete `framework/templates/spec/spec-and-plan.md`.
- [ ] Sweep each command source for `spec-and-plan.md` references and the dual-detection fallback. Files to edit: `clarify.md`, `plan.md`, `implement.md`, `review.md`, `validate.md`, `target.md`, `status.md`, `ask.md`. For each: replace "Check for `spec.md` first, then `spec-and-plan.md`. Use whichever exists. If neither exists, stop and report..." with "Read `spec.md`. If it does not exist, stop and report..." Drop any "If the spec file is `spec-and-plan.md` (lightweight track), [branch]" prose.
- [ ] Verify `grep -rn 'spec-and-plan' framework/commands/` returns no hits.
- [ ] Run the runtime parseability check against every edited command source and confirm pass.

Done when: zero `spec-and-plan` hits under `framework/commands/`; parseability check is clean.

### 9. `/specify` rewrite and `/capture` delete

- [ ] Edit `framework/commands/specify.md`: delete the "Lightweight track detection" section (the four qualifying questions). Simplify "Create the feature directory" to always copy `spec.md` from the template. Rewrite "Display the next step" to: "Run `/{project}:clarify` to resolve open questions and advance to clarified." (single line, no track-aware branch).
- [ ] Update the brownfield path in `specify.md` to explicitly note that sparse acceptance criteria are valid for brownfield use; reference §brownfield-process.
- [ ] Run the runtime parseability check against the rewritten `specify.md`.
- [ ] Delete `framework/commands/capture.md`.
- [ ] Run `scripts/gen-claude-commands.sh` and verify `.claude/commands/gov/capture.md` is pruned.

Done when: `specify.md` carries no qualifying questions; `capture.md` source no longer exists; the Claude-commands generator reports the prune.

### 10. `/ask` rewrite — classifier prose

- [ ] Edit `framework/commands/ask.md`: under the existing "Refine the question" section (or an adjacent new "Classify the input" section before refinement), add prose naming the heuristic — question signals (terminal `?`; interrogative starters how/what/when/should/could/would/is/are/do/does/can; hedge words maybe/perhaps/not sure); scenario signals (declarative or imperative; concrete event/state language on/when/if/after; no terminal `?`); status tiebreaker (on a `done` spec, scenario is the default for mixed signals).
- [ ] Update the existing user-approves-the-refined-form gate prose to display "Recording as [question|scenario] — preview drafted at [`## Open Questions` entry | `scenarios/{slug}.md`]" and accept `flip` as a standalone override that re-routes through the alternate path's drafting.
- [ ] Run the parseability check.

Done when: the prose names the heuristic and override surface explicitly; parseability is clean.

### 11. `/ask` rewrite — scenario branch and back-edges

- [ ] Add a "Scenario branch" subsection to `framework/commands/ask.md` covering: the decision tree (does a spec exist? is the spec ambiguous? is the behavior situational?); the invocation of `gov-rt:create-scenario` to write `scenarios/{slug}.md` from the scenario template; the invocation of `gov-rt:append-task` to add the linked task to `tasks.md`; the session-target update to point at the new scenario.
- [ ] Update the gate logic in `ask.md`: the `done` spec refusal goes away. On a `done` spec, the input routes to the scenario branch by default; on confirmation, `gov-rt:set-status` flips `done → in-progress` before scenario creation.
- [ ] Document the back-edge ownership update in the "Status mutation summary" table — both back-edges now belong to `/ask`.
- [ ] Run the parseability check.

Done when: scenario branch is fully described; both back-edges are documented; parseability is clean.

### 12. `/elaborate` delete and dependent prose update

- [ ] Delete `framework/commands/elaborate.md`.
- [ ] Update `framework/commands/groom.md`: replace the existing reference to running `/elaborate` separately for a deeper walk with the equivalent `/ask` reference.
- [ ] Update `framework/commands/clarify.md`: the recovery-path gate currently mentions `/elaborate` on the `done` row — rewrite to reference `/ask`.
- [ ] Update Status → next action tables in `framework/commands/target.md` and `framework/commands/status.md`: `done` row's next action becomes `/ask` (scenario branch) instead of `/elaborate`.
- [ ] Run the parseability check against every edited command.
- [ ] Run `scripts/gen-claude-commands.sh` and verify `.claude/commands/gov/elaborate.md` is pruned.

Done when: `elaborate.md` source no longer exists; no command source references `/elaborate`; generators run clean.

### 12b. `/validate` → `/analyze` rename and reference sweep

Pure rename, no behavior change. Must land atomically with the help-tables generator script update (task 13) or pre-commit fails.

- [ ] `git mv framework/commands/validate.md framework/commands/analyze.md`.
- [ ] Edit `framework/commands/analyze.md`: change the H1 from "# Validate" to "# Analyze"; replace the frontmatter `description:` value with exactly `Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. Read-only.`
- [ ] Edit `framework/commands/review.md`: replace the frontmatter `description:` value with exactly `Audit code against rules — security, reuse, quality, efficiency, simplicity. Writes review.md; blocks done on MUST violations.`
- [ ] Run `scripts/gen-help-tables.sh` and confirm both new descriptions propagate to `framework/commands/help.md`'s pipeline table.
- [ ] Sweep references in command sources: `framework/commands/help.md`, `framework/commands/review.md`, `framework/commands/analyze.md` (self-references in the body), and any other `framework/commands/*.md` with `/validate` mentions.
- [ ] Sweep references in `framework/constitution.md` (9 occurrences per the audit) — every `/{project}:validate` or `/gov:validate` becomes `/{project}:analyze` / `/gov:analyze`.
- [ ] Sweep references in `framework/bootstrap/govern.md` (manifest rows and any prose) — `validate.md` → `analyze.md`; `/{project}:validate` → `/{project}:analyze`.
- [ ] Sweep references in `framework/templates/spec/spec.md` (the template's example references) and `framework/templates/project/project-readme.md`.
- [ ] Edit `scripts/gen-help-tables.sh`: the pipeline-table builder references `validate.md` and `'/{project}:validate'` — both update to `analyze.md` and `'/{project}:analyze'`.
- [ ] Edit `scripts/lint-frontmatter.sh`: update any direct `validate.md` reference.
- [ ] Sweep references in `README.md` (5 occurrences) and `docs/introduction.md`.
- [ ] Edit `specs/README.md`: add a new "Past Renames" section recording `/validate → /analyze` (one bullet, references this spec).
- [ ] Verify done specs under `specs/NNN-*/` are NOT modified — `git diff --stat specs/0[0-2][0-9]-*/` should be empty (excluding 023 itself).
- [ ] Run `scripts/gen-claude-commands.sh` and verify `.claude/commands/gov/validate.md` is pruned and `.claude/commands/gov/analyze.md` is created.
- [ ] Verify `grep -rn '/validate\b\|validate\.md\|/gov:validate\|/{project}:validate' framework/ scripts/ docs/ README.md AGENTS.md` returns zero hits (excluding the migration-check step in `govern.md` if any, and excluding spec 023's own files).

Done when: source is at `framework/commands/analyze.md`; the grep returns no hits in current-state files; done-spec bodies are untouched; pre-commit hooks pass with the rename + help-generator script update in one commit.

### 13. Help-tables generator update

- [ ] Edit `scripts/gen-help-tables.sh`: rename the variable `elaborate_table` to `refine_table`; drop the `/elaborate` row from its build invocation; drop the `/capture` row from `brownfield_table`'s invocation; rename the marker name `commands-elaborate` to `commands-refine` throughout the script.
- [ ] Edit `framework/commands/help.md`: rename the heading `#### Elaborate (add precision)` to `#### Refine`; update the marker pair `commands-elaborate:start` / `commands-elaborate:end` to `commands-refine:start` / `commands-refine:end`; drop the `/capture` row's static reference text in the brownfield subsection if present.
- [ ] Run `scripts/gen-help-tables.sh` and verify the diff is clean.

Done when: dry-run reports "in sync"; help.md shows the renamed category with `/ask` only and the brownfield table with two rows.

### 14. `/govern` bootstrap — migration check and prose sweep

- [ ] Edit `framework/bootstrap/govern.md`: add a step after the archive extract phase and before the manifest apply phase. The step shells `find specs -maxdepth 2 -name spec-and-plan.md` (or equivalent), iterates results, and prompts the user for each: "Found `{path}` — rename to `{path}/../spec.md`? (Y/n)". On confirm: `mv`. On decline: log a warning.
- [ ] Update the bootstrap's completion message to include "Migrated N `spec-and-plan.md` files" when N > 0; omit the line when N=0.
- [ ] Sweep `framework/bootstrap/govern.md` for existing `spec-and-plan` and deleted-verb references. Specifically: remove `specs/**/spec-and-plan.md` from the pinned-files / migration-targets list (line ~296); drop `spec-and-plan.md` from the spec-files pattern list (line ~311); remove the manifest row mapping `framework/templates/spec/spec-and-plan.md` → `specs/templates/spec-and-plan.md` (line ~384); remove `spec-and-plan.md` from the artifacts-in-scope enumeration (line ~442); remove the manifest rows for `framework/commands/capture.md` and `framework/commands/elaborate.md` (lines ~479, 481).
- [ ] Add a changelog entry to `runtime/CHANGELOG.md` (or a project-level changelog if introduced) referencing spec 023 and the rename requirement.
- [ ] Run the parseability check on the modified `govern.md`.
- [ ] Verify `grep -n 'spec-and-plan\|/capture\b\|/elaborate\b' framework/bootstrap/govern.md` returns no hits except inside the migration-check step itself (which references `spec-and-plan.md` by design — the file pattern it's looking for).

Done when: bootstrap runs against a fixture project containing `spec-and-plan.md` and offers the rename; declining leaves the file in place with a warning; running again with the file already renamed completes silently; the grep returns only migration-check matches.

### 15. Prose sweep — root docs, adopter templates, specs/README

- [ ] Edit `README.md`: remove or rewrite every reference to `/capture`, `/elaborate`, and the lightweight track. Update the Slash Commands tables under "Pipeline", "Elaborate" (rename to "Refine"), and "Brownfield" to match the post-consolidation surface (Pipeline 6, Refine 1, Brownfield 2). Update "Adopting in an Existing Project" prose to point at `/specify` for brownfield use with sparse-AC guidance. Drop the `spec-and-plan.md` row from the templates table (line ~280) and rewrite the scenario.md description that mentions "elaborate workflow" (line ~285).
- [ ] Edit `AGENTS.md` (govern repo root): drop `spec-and-plan` from the framework templates list (line ~17).
- [ ] Edit `specs/README.md`: remove the "Lightweight track detection" bullet from §Design Decisions — this documents an active design that's being undone; deletion is the correct action, not signposting (the file is a cross-cutting decisions doc, not a done spec).
- [ ] Edit `docs/introduction.md`: sweep deleted-verb references (lines ~24, 31, 32, 65, 66) and lightweight-track mentions; rewrite the back-edges paragraph so both back-edges name `/ask`; update the help-tables-mirroring table to match the new category set (Pipeline 6 / Refine 1 / Brownfield 2 / Orient 3 / Bootstrap 2).
- [ ] Edit `framework/templates/project/agents.md`: drop `spec-and-plan` from the templates list (line ~43); remove the dedicated `spec-and-plan.md` description row (line ~46) including the `*(lightweight track)*` annotation.
- [ ] Edit `framework/templates/project/project-readme.md`: drop `spec-and-plan` from the templates list (line ~26).
- [ ] Run `grep -rn '/capture\b\|/elaborate\b\|lightweight track\|spec-and-plan' README.md AGENTS.md specs/README.md docs/ framework/` and confirm zero hits (excluding the resolved questions and decision tables in spec 023's own body, which are frozen historical record, and excluding the migration-check step in `framework/bootstrap/govern.md` that legitimately references the `spec-and-plan.md` filename pattern).

Done when: the four `grep` patterns return no hits outside the spec 023 directory and the migration-check step in `govern.md`.

### 16. Validation pass and pre-merge checks

- [ ] Run `scripts/lint-tool-coverage.sh` → passes.
- [ ] Run `scripts/gen-help-tables.sh --dry-run` → "in sync".
- [ ] Run `scripts/gen-spec-deps.sh --dry-run` → "in sync".
- [ ] Run `scripts/gen-readme-table.sh --dry-run` → "in sync".
- [ ] Run `scripts/gen-configure-mcp.sh --dry-run` → "in sync".
- [ ] Run `npx markdownlint-cli2 '**/*.md'` → passes.
- [ ] Run runtime parseability check across all rewritten commands → passes.
- [ ] Run `/gov:analyze` against spec 023 → no hard-fail or blocking findings.
- [ ] Push to a branch and confirm CI passes — both `markdown-only-pipeline.yml` and `runtime.yml`.

Done when: every check in the list above passes locally; CI workflows report green on the PR branch.

### 17. Code review gate

- [ ] Run `/gov:review` against spec 023.
- [ ] Resolve any MUST violations or record waivers.
- [ ] Confirm `review.blocking: false` in spec 023's frontmatter.

Done when: `/gov:review` returns clean and the spec frontmatter's `review.blocking` is `false`.
