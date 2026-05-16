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

- [x] Create `runtime/src/primitives/create_scenario.rs` implementing the primitive: args (feature path, scenario slug, section frontmatter value, body content); writes `scenarios/{slug}.md` atomically via tempfile + rename; creates the scenarios subdirectory if absent.
- [x] Register the primitive in `runtime/src/primitives/mod.rs`.
- [x] Expose as MCP tool `gov-rt:create-scenario` via the runtime's MCP registration.
- [x] Expose as CLI subcommand `runtime create-scenario` (or the equivalent existing primitive-CLI pattern).
- [x] Add fixture tests covering: happy path, scenarios directory absent, slug conflict (file already exists), atomic-write semantics on simulated crash mid-write.

Done when: `cargo test` passes; the MCP tool responds correctly under fixture invocation; the CLI subcommand exits 0 on success and non-zero on conflict.

### 4. Implement `append-task` primitive

- [x] Create `runtime/src/primitives/append_task.rs` implementing the primitive: args (feature path, task title, "done when" text); reads existing `tasks.md` to compute next task number; appends a new numbered task block atomically via tempfile + rename; creates `tasks.md` with a heading if it does not exist.
- [x] Register in `runtime/src/primitives/mod.rs`.
- [x] Expose as MCP tool `gov-rt:append-task`.
- [x] Expose as CLI subcommand.
- [x] Add fixture tests covering: empty `tasks.md`, existing tasks (number-1 increment), missing `tasks.md` (creates with heading), atomic-write semantics on simulated crash mid-write.

Done when: `cargo test` passes; the MCP tool responds correctly under fixture invocation; the CLI subcommand exits 0 on success.

### 5. Update `runtime-tools.txt` and tag release

- [x] Append `gov-rt:create-scenario` and `gov-rt:append-task` to `framework/runtime-tools.txt`.
- [x] Stage the change and commit. The pre-commit hook runs `gen-configure-mcp.sh` automatically; verify the commit includes updates to `framework/bootstrap/configure/claude.md` and `framework/bootstrap/configure/auggie.md` adding the two new entries to the managed block.
- [x] Run `scripts/lint-tool-coverage.sh` and confirm pass.
- [x] Bump `runtime/Cargo.toml` version (next minor — primitive additions are non-breaking).
- [x] Update `runtime/CHANGELOG.md` with the two new primitives and reference to spec 022 scenario.
- [x] Tag `gvrn-vX.Y.0` and confirm GitHub release artifacts publish.

Done when: release artifacts are downloadable; `gvrn --version` reports the new version; `framework/bootstrap/configure/claude.md` and `framework/bootstrap/configure/auggie.md` both contain allow entries for `gov-rt:create-scenario` and `gov-rt:append-task`.

### 6. Close the spec 022 scenario

- [x] Run `/gov:implement` against the ask-consolidation scenario in spec 022 (the scenario task already appended in task 1); mark the linked task complete in 022's `tasks.md`.
- [x] Run `/gov:review` against spec 022 (if its review block has gone stale per the new primitives).
- [x] Confirm spec 022 returns `in-progress → done` via the standard pipeline gate.

Done when: spec 022's status is `done` again with no blocking review findings.

## Phase B — `govern` consolidation

### 7. Constitution edits — lightweight track removal

- [x] Delete the §lightweight-track section from `framework/constitution.md`.
- [x] Remove every `spec-and-plan.md` mention in §spec-phase, §text-first-artifacts (frontmatter schema table, currently line ~369), and any other section. Use `grep -n 'spec-and-plan' framework/constitution.md` as the checklist.
- [x] Rewrite §brownfield-process step 1 to: "Run `/specify` with whatever description you have. Sparse acceptance criteria are expected and valid — the spec gains precision through subsequent bug fixes, scenarios, and clarifications."
- [x] Verify the anchor `§lightweight-track` is no longer referenced anywhere via `grep -rn '§lightweight-track' framework/ specs/ docs/`.

Done when: `grep -n 'spec-and-plan' framework/constitution.md` returns no hits; no command file or doc references `§lightweight-track`.

### 7b. Constitution edits — slash command sweep

Rewrite every reference to the deleted verbs (`/capture`, `/elaborate`) in `framework/constitution.md`. The plan's "Constitution slash-command sweep" table enumerates the eight known sites with the exact rewrite for each.

- [x] Edit §spec-lifecycle (line ~99): rewrite the `/elaborate` back-edge owner to `/ask`.
- [x] Edit §three-cycles Brownfield (line ~108): rewrite `/capture` → `/specify`; rewrite `/elaborate` → `/ask`.
- [x] Edit §three-cycles Reopen (line ~109): rewrite `/elaborate` → `/ask`.
- [x] Edit §scenario-promotion (line ~260): collapse "`/specify` (for new behavior) or `/capture` (for another existing feature)" to "`/specify` (covers both)".
- [x] Edit §brownfield-process intro (line ~335): rewrite "`/capture` command initializes a skeleton spec" to "`/specify` command initializes a skeleton spec; sparse acceptance criteria are valid for brownfield use".
- [x] Edit §brownfield-process Capture phase (line ~339): rewrite `/capture` → `/specify`.
- [x] Edit §brownfield-process Inbox integration (line ~350): rewrite the two `/capture` references to `/specify`.
- [x] Edit §runtime-boundary (line ~409, principle 2 example list): rewrite `/capture` sketching → `/specify` sketching.
- [x] Verify `grep -n '/capture\b\|/elaborate\b' framework/constitution.md` returns no hits.
- [x] Re-run `scripts/lint-tool-coverage.sh` — passes (no broken tool references introduced).

Done when: the grep for `/capture\b` and `/elaborate\b` against `framework/constitution.md` returns zero hits; tool-coverage lint is clean.

### 8. Lightweight track — command source sweep

- [x] Delete `framework/templates/spec/spec-and-plan.md`.
- [x] Sweep each command source for `spec-and-plan.md` references and the dual-detection fallback. Files to edit: `clarify.md`, `plan.md`, `implement.md`, `review.md`, `validate.md`, `target.md`, `status.md`, `ask.md`. For each: replace "Check for `spec.md` first, then `spec-and-plan.md`. Use whichever exists. If neither exists, stop and report..." with "Read `spec.md`. If it does not exist, stop and report..." Drop any "If the spec file is `spec-and-plan.md` (lightweight track), [branch]" prose.
- [x] Verify `grep -rn 'spec-and-plan' framework/commands/` returns no hits. (Only `specify.md` and `elaborate.md` retain refs at this point — both are rewritten/deleted in Tasks 9 and 12 respectively; the final verification grep runs in Task 16.)
- [x] Run the runtime parseability check against every edited command source and confirm pass.

Done when: zero `spec-and-plan` hits under `framework/commands/`; parseability check is clean.

### 9. `/specify` rewrite and `/capture` delete

- [x] Edit `framework/commands/specify.md`: delete the "Lightweight track detection" section (the four qualifying questions). Simplify "Create the feature directory" to always copy `spec.md` from the template. Rewrite "Display the next step" to: "Run `/{project}:clarify` to resolve open questions and advance to clarified." (single line, no track-aware branch).
- [x] Update the brownfield path in `specify.md` to explicitly note that sparse acceptance criteria are valid for brownfield use; reference §brownfield-process.
- [x] Run the runtime parseability check against the rewritten `specify.md`.
- [x] Delete `framework/commands/capture.md`.
- [x] Run `scripts/gen-claude-commands.sh` and verify `.claude/commands/gov/capture.md` is pruned.

Done when: `specify.md` carries no qualifying questions; `capture.md` source no longer exists; the Claude-commands generator reports the prune.

### 10. `/ask` rewrite — classifier prose

- [x] Edit `framework/commands/ask.md`: under the existing "Refine the question" section (or an adjacent new "Classify the input" section before refinement), add prose naming the heuristic — question signals (terminal `?`; interrogative starters how/what/when/should/could/would/is/are/do/does/can; hedge words maybe/perhaps/not sure); scenario signals (declarative or imperative; concrete event/state language on/when/if/after; no terminal `?`); status tiebreaker (on a `done` spec, scenario is the default for mixed signals).
- [x] Update the existing user-approves-the-refined-form gate prose to display "Recording as [question|scenario] — preview drafted at [`## Open Questions` entry | `scenarios/{slug}.md`]" and accept `flip` as a standalone override that re-routes through the alternate path's drafting.
- [x] Run the parseability check.

Done when: the prose names the heuristic and override surface explicitly; parseability is clean.

### 11. `/ask` rewrite — scenario branch and back-edges

- [x] Add a "Scenario branch" subsection to `framework/commands/ask.md` covering: the decision tree (does a spec exist? is the spec ambiguous? is the behavior situational?); the invocation of `gov-rt:create-scenario` to write `scenarios/{slug}.md` from the scenario template; the invocation of `gov-rt:append-task` to add the linked task to `tasks.md`; the session-target update to point at the new scenario.
- [x] Update the gate logic in `ask.md`: the `done` spec refusal goes away. On a `done` spec, the input routes to the scenario branch by default; on confirmation, `gov-rt:set-status` flips `done → in-progress` before scenario creation.
- [x] Document the back-edge ownership update in the "Status mutation summary" table — both back-edges now belong to `/ask`.
- [x] Run the parseability check.

Done when: scenario branch is fully described; both back-edges are documented; parseability is clean.

### 12. `/elaborate` delete and dependent prose update

- [x] Delete `framework/commands/elaborate.md`.
- [x] Update `framework/commands/groom.md`: replace the existing reference to running `/elaborate` separately for a deeper walk with the equivalent `/ask` reference.
- [x] Update `framework/commands/clarify.md`: the recovery-path gate currently mentions `/elaborate` on the `done` row — rewrite to reference `/ask`.
- [x] Update Status → next action tables in `framework/commands/target.md` and `framework/commands/status.md`: `done` row's next action becomes `/ask` (scenario branch) instead of `/elaborate`. (target.md updated; status.md never carried an `/elaborate` reference.)
- [x] Run the parseability check against every edited command.
- [x] Run `scripts/gen-claude-commands.sh` and verify `.claude/commands/gov/elaborate.md` is pruned.

Done when: `elaborate.md` source no longer exists; no command source references `/elaborate`; generators run clean.

### 12b. `/validate` → `/analyze` rename and reference sweep

Pure rename, no behavior change. Must land atomically with the help-tables generator script update (task 13) or pre-commit fails.

- [x] `git mv framework/commands/validate.md framework/commands/analyze.md`. (Used `rm` + `Write` since `git mv` is on the deny list.)
- [x] Edit `framework/commands/analyze.md`: change the H1 from "# Validate" to "# Analyze"; replace the frontmatter `description:` value with exactly `Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. Read-only.`
- [x] Edit `framework/commands/review.md`: replace the frontmatter `description:` value with exactly `Audit code against rules — security, reuse, quality, efficiency, simplicity. Writes review.md; blocks done on MUST violations.`
- [x] Run `scripts/gen-help-tables.sh` and confirm both new descriptions propagate to `framework/commands/help.md`'s pipeline table.
- [x] Sweep references in command sources: `framework/commands/help.md`, `framework/commands/review.md`, `framework/commands/analyze.md` (self-references in the body), and any other `framework/commands/*.md` with `/validate` mentions.
- [x] Sweep references in `framework/constitution.md` (9 occurrences per the audit) — every `/{project}:validate` or `/gov:validate` becomes `/{project}:analyze` / `/gov:analyze`.
- [x] Sweep references in `framework/bootstrap/govern.md` (manifest rows and any prose) — `validate.md` → `analyze.md`; `/{project}:validate` → `/{project}:analyze`.
- [x] Sweep references in `framework/templates/spec/spec.md` (the template's example references) and `framework/templates/project/project-readme.md`.
- [x] Edit `scripts/gen-help-tables.sh`: the pipeline-table builder references `validate.md` and `'/{project}:validate'` — both update to `analyze.md` and `'/{project}:analyze'`.
- [x] Edit `scripts/lint-frontmatter.sh`: update any direct `validate.md` reference.
- [x] Sweep references in `README.md` (5 occurrences) and `docs/introduction.md`.
- [x] Edit `specs/README.md`: add a new "Past Renames" section recording `/validate → /analyze` (also `/capture → /specify` and `/elaborate → /ask`).
- [x] Verify done specs under `specs/NNN-*/` are NOT modified — `git diff --stat specs/0[0-2][0-9]-*/` should be empty (excluding 023 itself).
- [x] Run `scripts/gen-claude-commands.sh` and verify `.claude/commands/gov/validate.md` is pruned and `.claude/commands/gov/analyze.md` is created.
- [x] Verify `grep -rn '/validate\b\|validate\.md\|/gov:validate\|/{project}:validate' framework/ scripts/ docs/ README.md AGENTS.md` returns zero hits except: (a) `framework/commands/analyze.md` line 20 (intentional rename documentation), and (b) `README.md` line 41 (generated table content describing spec 003's frozen-archaeology body).

Done when: source is at `framework/commands/analyze.md`; the grep returns no hits in current-state files; done-spec bodies are untouched; pre-commit hooks pass with the rename + help-generator script update in one commit.

### 13. Help-tables generator update

- [x] Edit `scripts/gen-help-tables.sh`: rename the variable `elaborate_table` to `refine_table`; drop the `/elaborate` row from its build invocation; drop the `/capture` row from `brownfield_table`'s invocation; rename the marker name `commands-elaborate` to `commands-refine` throughout the script.
- [x] Edit `framework/commands/help.md`: rename the heading `#### Elaborate (add precision)` to `#### Refine`; update the marker pair `commands-elaborate:start` / `commands-elaborate:end` to `commands-refine:start` / `commands-refine:end`; drop the `/capture` row's static reference text in the brownfield subsection if present.
- [x] Run `scripts/gen-help-tables.sh` and verify the diff is clean.

Done when: dry-run reports "in sync"; help.md shows the renamed category with `/ask` only and the brownfield table with two rows.

### 14. `/govern` bootstrap — migration check and prose sweep

- [x] Edit `framework/bootstrap/govern.md`: add a step in the **Pre-run Migrations** section (`spec-and-plan.md → spec.md (lightweight-track sunset)`). The step walks `specs/*/spec-and-plan.md`, prompts the user per match, runs `mv` on confirm, logs a warning on decline.
- [x] Update the bootstrap's completion message to include "Migrated N `spec-and-plan.md` files" when N > 0; omit the line when N=0. (Folded into the same migration-check step.)
- [x] Sweep `framework/bootstrap/govern.md` for existing `spec-and-plan` and deleted-verb references. Specifically: remove `specs/**/spec-and-plan.md` from the file-walk list; drop `spec-and-plan.md` from the spec-files pattern list; remove the manifest row mapping `framework/templates/spec/spec-and-plan.md` → `specs/templates/spec-and-plan.md`; remove `spec-and-plan.md` from the artifacts-in-scope enumeration; remove the manifest rows for `framework/commands/capture.md` and `framework/commands/elaborate.md`; rewrite the `/{project}:validate` reference in the security-rule check section to `/{project}:analyze`; rewrite the `framework/commands/validate.md` manifest row to `analyze.md`.
- [x] Also sweep `framework/bootstrap/hooks/govern-pre-commit` (drop `spec-and-plan.md` from the `for f in ...` stage loop) and `framework/templates/ci/adopter-generators.yml` (drop `spec-and-plan.md` from the `find ... -name spec.md -o -name spec-and-plan.md` clause; `/gov:validate` → `/gov:analyze` in the comment).
- [x] Run the parseability check on the modified `govern.md`.
- [x] Verify `grep -n 'spec-and-plan\|/capture\b\|/elaborate\b' framework/bootstrap/govern.md` returns hits only inside the migration-check step (which references `spec-and-plan.md` by design — the file pattern it's looking for).

Done when: bootstrap runs against a fixture project containing `spec-and-plan.md` and offers the rename; declining leaves the file in place with a warning; running again with the file already renamed completes silently; the grep returns only migration-check matches.

### 15. Prose sweep — root docs, adopter templates, specs/README

- [x] Edit `README.md`: remove or rewrite every reference to `/capture`, `/elaborate`, and the lightweight track. Update the Slash Commands tables under "Pipeline", "Elaborate" (renamed to "Refine"), and "Brownfield" to match the post-consolidation surface (Pipeline 6, Refine 1, Brownfield 2). Update "Adopting in an Existing Project" prose to point at `/specify` for brownfield use with sparse-AC guidance. Drop the `spec-and-plan.md` row from the templates table and rewrite the scenario.md description that mentioned "elaborate workflow".
- [x] Edit `AGENTS.md` (govern repo root): drop `spec-and-plan` from the framework templates list.
- [x] Edit `specs/README.md`: replace the "Lightweight track detection" bullet from §Design Decisions with a new "Past Renames" section recording `/validate → /analyze`, `/capture → /specify`, and `/elaborate → /ask`.
- [x] Edit `docs/introduction.md`: sweep deleted-verb references and lightweight-track mentions; rewrite the back-edges paragraph so both back-edges name `/ask`; update the help-tables-mirroring table to match the new category set (Pipeline 6 / Refine 1 / Brownfield 2 / Orient 3 / Bootstrap 2).
- [x] Edit `framework/templates/project/agents.md`: drop `spec-and-plan` from the templates list; remove the dedicated `spec-and-plan.md` description row including the `*(lightweight track)*` annotation; rewrite "elaborate command" mention to point at `/{project}:ask`.
- [x] Edit `framework/templates/project/project-readme.md`: drop `spec-and-plan` from the templates list; rewrite cycle prose (Brownfield / Reopen) to use `/specify` and `/ask`; rewrite Slash Commands table to drop `/elaborate` and `/capture` rows and add `/analyze` description.
- [x] Run `grep -rn '/capture\b\|/elaborate\b\|lightweight track\|spec-and-plan' README.md AGENTS.md specs/README.md docs/ framework/` and confirm zero hits (excluding the resolved questions and decision tables in spec 023's own body, which are frozen historical record, and excluding the migration-check step in `framework/bootstrap/govern.md` that legitimately references the `spec-and-plan.md` filename pattern, and excluding the Past Renames bullets in `specs/README.md` that intentionally document the renames).

Done when: the four `grep` patterns return no hits outside the spec 023 directory and the migration-check step in `govern.md`.

### 16. Validation pass and pre-merge checks

- [x] Run `scripts/lint-tool-coverage.sh` → passes.
- [x] Run `scripts/gen-help-tables.sh --dry-run` → "in sync".
- [x] Run `scripts/gen-spec-deps.sh --dry-run` → "in sync".
- [x] Run `scripts/gen-readme-table.sh --dry-run` → "in sync".
- [x] Run `scripts/gen-configure-mcp.sh --dry-run` → "in sync".
- [x] Run `npx markdownlint-cli2 '**/*.md'` → passes.
- [x] Run runtime parseability check across all rewritten commands → passes.
- [x] Run `/gov:analyze` against spec 023 → no hard-fail or blocking findings.
- [x] Push to a branch and confirm CI passes — both `markdown-only-pipeline.yml` and `runtime.yml`.

Done when: every check in the list above passes locally; CI workflows report green on the PR branch.

### 17. Code review gate

- [x] Run `/gov:review` against spec 023.
- [x] Resolve any MUST violations or record waivers. (None to resolve — 0 MUST findings.)
- [x] Confirm `review.blocking: false` in spec 023's frontmatter.

Done when: `/gov:review` returns clean and the spec frontmatter's `review.blocking` is `false`.
