---
title: "013-text-first-artifacts — tasks"
---

# 013 — Text-First Artifacts Tasks

Tasks derived from the [plan](plan.md). Complete in order. Phase 1 must finish before Phase 2; within Phase 2, command-source updates may run in parallel; Phase 3 onward is sequential.

## Phase 1: Foundation

### 1. Add §text-first-artifacts to the constitution

- [x] Add a new `<!-- §text-first-artifacts -->` anchor and section to `framework/constitution.md`, placed immediately after `§scenarios` and before `§pipeline-boundaries`.
- [x] The section declares: (a) all governance artifacts are markdown by default; (b) structured metadata lives in YAML frontmatter at the top of each markdown file; (c) cross-artifact references use standard relative markdown links, not wiki-links; (d) source-of-truth artifacts are markdown — structured derived views (SQLite caches, JSON indexes) are permitted only as gitignored build artifacts; (e) exceptions to text-first source-of-truth require an explicit constitutional amendment.
- [x] Below the principle: a markdown table declaring the frontmatter schema for spec files (`status`, `dependencies` required; `tags` optional) and scenario files (`spec-ref` required; `tags` optional). Columns: `Field | Required | Type | Allowed values | Description`. Mirror `data-model.md`.
- [x] State the schema scope: applies to spec and scenario files only. Other artifacts (`system.md`, `errors.md`, `events.md`, `inbox.md`, plan/tasks/rule files) MAY include frontmatter when a consumer benefits.
- [x] State the open-schema rule: additional fields permitted, ignored by uninterested consumers.
- [x] Publish the starter tag vocabulary as a separate table (per `data-model.md`), framed as guidance, not enforcement.
- [x] **Done when:** `framework/constitution.md` contains the new section and lints clean.

### 2. Update spec, spec-and-plan, and scenario templates

- [x] Replace the `**Status:** draft` and `**Dependencies:** none` lines in `framework/templates/spec/spec.md` with a YAML frontmatter block at the top of the file: `status: draft`, `dependencies: []`, `tags: []`. Strip the body's bold-prefix lines.
- [x] Apply the same change to `framework/templates/spec/spec-and-plan.md`. (Preserved `track: lightweight` as an open-schema field.)
- [x] Replace the bold-prefix `**spec-ref:**` line in `framework/templates/spec/scenario.md` with a YAML frontmatter block: `spec-ref: ""`, `tags: []`. Strip the body's bold-prefix line.
- [x] **Done when:** all three templates lint clean and reflect the new format.

## Phase 2: Slash command sources

These tasks may proceed in parallel within a session. Each command file is touched once. Tasks 11–14 are audits — only modify if the command actually reads or writes spec/scenario metadata.

### 3. Update `/gov:specify`

- [x] Update `framework/commands/specify.md` to write spec metadata as YAML frontmatter rather than bold-prefix lines.
- [x] Add a tag prompt step: after the lightweight-track question and before file creation, instruct the command to read existing `tags` values from sibling specs in `specs/*/spec.md` and `specs/*/spec-and-plan.md` (frontmatter), display them as suggestions, and prompt the author for one or more tags. The author may pick from suggestions, enter new tags, or skip (writes `tags: []`).
- [x] **Done when:** the command instructions describe frontmatter-writing and the tag prompt; file lints clean.

### 4. Update `/gov:clarify`

- [x] Update `framework/commands/clarify.md` to read the spec status from the YAML frontmatter `status` field rather than the `**Status:** {value}` line.
- [x] Update the status-write step (advance to `clarified`) to update the frontmatter field.
- [x] Add a missing-tags advisory check inside the validation gate: if `tags` is missing or empty, surface as one of the gate findings with severity advisory. Does not block the transition.
- [x] Apply the same frontmatter parsing for the scenario-targeted path (`spec-ref` from frontmatter).
- [x] **Done when:** the command instructions read and write frontmatter, the advisory is described, and the file lints clean.

### 5. Update `/gov:plan`

- [x] Update `framework/commands/plan.md` to read the spec status from frontmatter and write the status update on advance to `planned` to the frontmatter field.
- [x] **Done when:** instructions reference frontmatter; file lints clean.

### 6. Update `/gov:implement`

- [x] Update `framework/commands/implement.md` to read/write the spec status via frontmatter (gate on `planned` or `in-progress`; advance to `in-progress` and `done`).
- [x] **Done when:** instructions reference frontmatter; file lints clean.

### 7. Update `/gov:status`

- [x] Update `framework/commands/status.md` to extract `status`, `dependencies`, and `tags` from each spec's frontmatter rather than from `**Status:** {value}` and `**Dependencies:** {value}` lines.
- [x] Update any extraction prose that mentions "look for the bold-prefix lines" to "parse the YAML frontmatter block."
- [x] Confirm the dashboard rendering still works for specs with empty `tags` — chose to display tags only as an aggregate "tags in use" line below the table (skipped entirely when no spec has tags); kept the table columns unchanged for now.
- [x] **Done when:** the dashboard reads from frontmatter; file lints clean.

### 8. Update `/gov:target`

- [x] Update `framework/commands/target.md` to read the spec status from frontmatter when displaying the target detail view.
- [x] **Done when:** instructions reference frontmatter; file lints clean.

### 9. Update `/gov:analyze`

- [x] Update `framework/commands/analyze.md` with the strict/advisory split per `data-model.md`'s severity table.
- [x] Hard-fail conditions: missing/malformed frontmatter, missing/invalid `status`, missing/invalid `dependencies`, missing `spec-ref` on scenarios.
- [x] Advisory: empty `tags`, unknown fields (informational), existing checkbox/cross-reference checks.
- [x] The command's report format clearly separates hard fails, blocking, advisory, and informational findings.
- [x] **Done when:** instructions describe the split; file lints clean.

### 10. Update `/gov:specify`

- [x] Update `framework/commands/capture.md` to write frontmatter for new sketch specs (with `status: draft`, `dependencies: []`, `tags: []`).
- [x] **Done when:** instructions reference frontmatter; file lints clean.

### 11. Audit and update `/gov:groom`

- [x] Read `framework/commands/groom.md` and check whether it parses spec metadata as part of routing inbox items.
- [x] Audit conclusion: groom does not read or write spec metadata fields. No edit needed.
- [x] **Done when:** audit complete; any required updates lint clean.

### 12. Audit and update `/gov:ask`

- [x] Read `framework/commands/elaborate.md` and check whether it writes scenario `spec-ref` or otherwise touches scenario metadata.
- [x] Updated the "Update spec status" section to read/write the frontmatter `status` field. (Scenario creation uses the scenario template, which already emits frontmatter from Task 2.)
- [x] **Done when:** scenario creation produces frontmatter-formatted scenarios; file lints clean.

### 13. Audit and update `/gov:ask`

- [x] Read `framework/commands/ask.md` and check whether it reads spec/scenario metadata to identify the target.
- [x] Updated scope boundaries to declare that `status` is read from YAML frontmatter; references §text-first-artifacts.
- [x] **Done when:** audit complete; any required updates lint clean.

### 14. Regenerate Claude command instances

- [x] Run `./scripts/gen-claude-commands.sh` to regenerate `.claude/commands/gov/*.md` from the updated `framework/commands/` and `framework/bootstrap/configure/claude.md` sources.
- [x] Spot-checked the regenerated files: 15 files regenerated, all lint clean.
- [x] **Done when:** generation completes without error; spot-check passes.

## Phase 3: Migration logic in `/govern`

### 15. Add migration step to `framework/bootstrap/govern.md`

- [x] Add a new section to `framework/bootstrap/govern.md`, positioned between Project Configuration (which reads `.governance.toml`) and File Fetching, titled "Frontmatter Migration."
- [x] Step 1: run `git status --porcelain -- specs/` (project-relative). If the output is non-empty, refuse with a clear message ("Migration requires a clean working tree under `specs/`. Commit or stash your changes, then re-run.") and exit before any modifications.
- [x] Step 2: walk `specs/**/spec.md`, `specs/**/spec-and-plan.md`, and `specs/**/scenarios/*.md`.
- [x] Step 3: for each file, check whether the first non-blank line is `---`. If yes, skip with reason "already frontmatter." If no and bold-prefix metadata lines are present, convert: insert frontmatter block at top, remove redundant body lines.
- [x] Step 4: for each file, also check `.governance.toml` `pinned.files`. If pinned, skip with reason "pinned."
- [x] Step 5: print a per-file summary at the end of the run (`migrated`, `skipped (already frontmatter)`, `skipped (pinned)`, `skipped (no metadata to migrate)`, `skipped (malformed metadata)`). Surface to the user.
- [x] Added an explicit early-return when `specs/` does not exist (first run — nothing to migrate).
- [x] Added an Edge Cases subsection covering partially-migrated files, malformed metadata, and custom open-schema fields.
- [x] **Done when:** the migration section is present, idempotent, scoped, and respects pinning; file lints clean.

### 16. Add Quartz tip to govern.md post-run output

- [x] Added a one-line tip to both the First-run and Update-mode output blocks: `Tip: \`npx quartz specs/\` renders your specs as a navigable graph view in the browser. Other PKM tools (Obsidian, Logseq, MkDocs) work unchanged.`
- [x] Positioned at the end of each block (after the existing next-steps content) so it's discoverable but not load-bearing.
- [x] **Done when:** the tip appears in the post-run output; file lints clean.

### 17. Verify migration on a test fixture

- [x] Created `/tmp/govern-013-fixture/` with: `specs/000-foo/spec.md` (typical bold-prefix), `specs/000-foo/scenarios/edge-case.md` (bold-prefix `spec-ref`), `specs/001-bar/spec.md` (pinned via `.governance.toml`), `specs/002-already-migrated/spec.md` (already in frontmatter), plus a `.governance.toml` pinning `001-bar`.
- [x] Confirmed clean-tree precheck behavior: clean → empty `git status --porcelain -- specs/` → migration proceeds; dirty → non-empty output → migration refuses.
- [x] Walked the convert step on `000-foo/spec.md` and `000-foo/scenarios/edge-case.md`. Both produced correctly-structured frontmatter (em-dash in `spec-ref` properly quoted), bodies preserved, `# Heading` placed after the frontmatter block. Output lints clean.
- [x] Verified pinning: `001-bar/spec.md` was untouched (matches `.governance.toml` `pinned.files`).
- [x] Verified idempotency: `002-already-migrated/spec.md` starts with `---` on the first non-blank line and would be skipped per the prose.
- [x] No rough edges discovered. The migration prose in `govern.md` is sufficient as written.
- [x] **Done when:** the fixture migration runs cleanly end-to-end; documented findings (if any) are incorporated. (Fixture left in `/tmp/govern-013-fixture/` — `/tmp` clears on reboot.)

## Phase 4: Self-migration of governance's own specs

### 18. Migrate existing governance specs to frontmatter

- [x] For each spec under `specs/000-*` through `specs/012-*`: open `spec.md`, insert a frontmatter block with the existing `status` and `dependencies` values, and remove the bold-prefix lines from the body. Tags remain empty (`tags: []`) — backfill is organic per Q2's resolution. (All 13 specs use `spec.md`; no `spec-and-plan.md` files exist in this repo.)
- [x] Migrate `specs/013-text-first-artifacts/spec.md` last so the migration process operates on the spec that motivated it. (Tags populated as `[format, migration, pipeline]` reflecting the spec's actual concerns.)
- [x] Discovered and corrected a pre-existing README/spec-file mismatch for 012: spec file said `done`, README said `clarified`. README updated to match the spec file (source of truth).
- [x] **Done when:** every existing governance spec uses frontmatter format.

## Phase 5: Documentation

### 19. Add "Viewing artifacts" section to README.md

- [x] Added a new "Viewing artifacts" section to the root `README.md`, positioned just before the "Markdown" section.
- [x] Documents `npx quartz build --input specs/ --serve` as the recommended viewer; references `framework/constitution.md` §text-first-artifacts.
- [x] Lists portable alternatives (Obsidian, Logseq, Foam, MkDocs, plain `cat`/GitHub PR review) framed as "pick whichever fits your workflow, or none."
- [x] **Done when:** the section is present, accurate, and the README lints clean.

## Phase 6: Verification

### 20. Lint and validate

- [x] Ran `npx markdownlint-cli2 "**/*.md" "!CLAUDE.md"` across the repo: 100 files, 0 errors.
- [x] Walked through the validate prose against representative migrated specs (013, 000) — hard-fail checks pass for all, advisory triggers correctly on empty `tags` for migrated specs, and informational unknown-field rule does not raise spurious findings. The strict/advisory split functions as designed in `framework/commands/analyze.md`.
- [x] **Done when:** lint clean across the repo, validate behaves per the strict/advisory split, all specs report as valid.

### 21. Final acceptance-criteria sweep

- [x] Walked the spec's Acceptance Criteria list and verified each item against the implementation.
- [x] Marked each acceptance criterion `[x]` as it is verified — all criteria are satisfied.
- [x] **Done when:** every acceptance criterion in `spec.md` is checked off and verified.

## Phase 7: Post-done refinements

### 22. Tag-curation pass across all specs

- [x] Walked every `specs/*/spec.md` and reviewed the `tags:` frontmatter. 10 specs (000, 001, 002, 003, 004, 006, 007, 009, 011, 012) had empty `tags: []`; populated each. 4 specs (005, 008, 010, 013) had pre-existing tags; reviewed and kept as-is.
- [x] Used a coherent taxonomy drawn from values already in use (`bootstrap`, `templates`, `format`, `migration`, `pipeline`, `agent`, `process`, `security`) plus three new shared values (`commands`, `scenarios`, `brownfield`) where existing tags would have been misleading.
- [x] All values are lowercase, short, and 1–3 tags per spec.
- [x] Repo lints clean (verified at end of Phase 8 after task 14 in 000 also lands).

Done when: every `specs/*/spec.md` has a non-empty, coherent `tags` value and the repo lints clean.
