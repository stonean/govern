---
status: draft
---

# 020 — `/gov:review` Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create `framework/commands/review.md`

- [x] Copy the embedded `framework/commands/review.md` artifact from [spec.md](spec.md) verbatim into `framework/commands/review.md`. The embedded content is the canonical source of the command's behavior, including the tech-stack alignment check, empty-scope short-circuit, cross-pass dedupe, `[review] tech-stack-verified` persistence flow, and waiver semantics resolved during clarify.
- [x] Verify the file passes `npx markdownlint-cli2`.
- [x] (Scope addition during implement) Added YAML frontmatter (`description`, `argument-hint`) to both the embedded artifact in `spec.md` and the new command file — required by the help-table and README generators. Added a `## Blocking message` section to the embedded artifact to satisfy the `#blocking-message` link from the tech-stack alignment step.
- [x] (Scope addition during implement) Updated `scripts/gen-help-tables.sh` to include `/gov:review` in the pipeline command list with gate `blocks \`done\` (MUST violations)` and regenerated `framework/commands/help.md`.
- **Done when**: the file exists, is byte-identical (modulo the outer 4-backtick fence) to the embedded artifact, and lints clean.

## 2. Edit `framework/commands/implement.md` — pre-`done` review gate

- [x] Locate the section that transitions `status: in-progress` → `status: done`.
- [x] Insert the **Pre-`done` review gate** block from the embedded artifact in [spec.md](spec.md) immediately before the transition. The block reads `review.last-run` and `review.blocking`, halts with the documented messages, and lets the `done` write proceed only when both checks pass.
- [x] Verify the file passes `npx markdownlint-cli2`.
- **Done when**: a `done` transition halts when `review.last-run` is missing or `review.blocking: true`, with the blocking messages from [spec.md](spec.md) §Blocking message.

## 3. Edit `framework/commands/analyze.md` — review-drift check

- [x] Add a **Review drift** check to the audit section: for each spec at `status: done`, record a violation when `review.last-run` is missing or `review.blocking: true`. Use the messages from the embedded `framework/commands/analyze.md` edits in [spec.md](spec.md).
- [x] Wire `/gov:analyze --fix` to revert affected specs from `done` → `in-progress` and emit a one-line notice per spec (never silent). Re-running `/gov:review` is left to the operator.
- [x] Verify the file passes `npx markdownlint-cli2`.
- [x] (Scope addition during implement) Added `--fix` to analyze.md's `argument-hint` — it was documented in the 000-slash-commands `validate-fix-mode` scenario but absent from the command file's frontmatter, a pre-existing documentation drift that this task surfaces and fixes.
- **Done when**: `/gov:analyze` flags drifted `done` specs; `/gov:analyze --fix` reverts and notices each one.

## 4. Edit `framework/templates/spec/spec.md` — add `review:` block

- [x] Add the `review:` frontmatter block from [data-model.md](data-model.md) §"Spec frontmatter `review:` block" with safe defaults (`last-run: null`, `reviewed-against: null`, all counts `0`, `blocking: false`, no `waivers` field).
- [x] Verify the file passes `npx markdownlint-cli2`.
- [x] (Scope addition during implement) Updated implement.md and analyze.md checks to treat `null` and missing as equivalent — the template's safe default is `null`, and a `done` spec with `null` means "no review actually ran" (same blocking condition as missing).
- **Done when**: a freshly-scaffolded spec ships with the `review:` block at its safe defaults.

## 5. Edit `framework/templates/spec/spec-and-plan.md` — add `review:` block

- [x] Same `review:` block addition as task 4, applied to the lightweight-track template.
- [x] Verify the file passes `npx markdownlint-cli2`.
- **Done when**: a freshly-scaffolded `spec-and-plan.md` ships with the `review:` block at its safe defaults.

## 6. Edit `framework/templates/ci/adopter-generators.yml` — review-blocking check

- [x] Add a step that scans `specs/*/spec.md` and `specs/*/spec-and-plan.md` and exits non-zero when any file with `status: done` has `review.blocking: true` or is missing `review.last-run`.
- [x] The step does not invoke `/gov:review` itself; it is a frontmatter-state backstop only (per the plan's "CI template stays minimal" decision).
- [x] Verify the YAML parses (parsed with Ruby's YAML — no Python yaml module locally; GHA runners have both).
- [x] Tested the gate logic locally with bash against `find specs -maxdepth 2 …`: correctly flagged all 20 existing `done` specs (which lack the `review:` block entirely) and skipped the in-progress `020-code-review`. This is the expected behavior — Task 11 will resolve the backfill for govern's own specs.
- **Done when**: a deliberate test where a `done` spec has `review.blocking: true` causes the CI step to fail.

## 7. Edit `framework/constitution.md` — reference the review gate

- [x] In the §pipeline / §spec-lifecycle section, add a sentence noting that `/gov:review` runs after `/gov:implement` and that a `done` transition is gated by `review.blocking`. Cross-reference the §pipeline-boundaries "never depend on human diligence" rationale already in the file.
- [x] Update the lifecycle diagram: replaced the final `/implement` transition with `[/review gate]` so the gate is visible in the canonical ASCII rendering.
- [x] Added a bullet under §implement-phase #Implementation requirements stating that the `in-progress → done` transition is gated by `/review`.
- [x] Verify the file passes `npx markdownlint-cli2`.
- **Done when**: reading the constitution alone makes clear that `/gov:review` is part of the path to `done`.

## 8. Edit `README.md` — `/gov:review` row, Waivers subsection, pipeline diagrams

- [x] Add a `/gov:review` row to the **Pipeline (advance state)** slash-commands table with a one-line purpose.
- [x] Add a short **Waivers** subsection under Slash Commands, summarizing the `--waive --reason` flow and pointing at [data-model.md](data-model.md) §"Waiver record" for the schema.
- [x] Update the verb-named command list in the intro paragraph and the numbered pipeline steps under "Work through the pipeline" to include `/review` and a separate `Done` step gated by `review.blocking: false`.
- [x] Skipped: editing the description of `done` spec 003 (which enumerates the dogfooded command list as of its merge) — that's archival and falls under §drift-prevention "done specs are frozen archaeology."
- [x] Verify the file passes `npx markdownlint-cli2`.
- **Done when**: README readers can discover `/gov:review` from the commands table, the diagrams, and the Waivers subsection.

## 9. Regenerate `.claude/commands/gov/review.md`

- [x] Ran `scripts/gen-claude-commands.sh` (the actual script name; the task originally guessed `regenerate-commands.sh`). Regenerated 16 files; `init.md` was correctly skipped (hand-maintained exception).
- [x] Verified `.claude/commands/gov/review.md` exists with frontmatter and body matching `framework/commands/review.md`. The harness's skill registry now lists `gov:review` — end-to-end discoverability confirmed.
- **Done when**: `.claude/commands/gov/review.md` exists and `git status` shows only generator output, not hand edits.

## 10. Add scenario `specs/020-code-review/scenarios/waiver-expiry.md`

- [x] Create the scenario with `section: "Waivers"` in frontmatter.
- [x] Document Behavior, Edge Cases (file renamed, file deleted, rule renamed/removed, same rule firing at a different location), and Context per [framework/templates/spec/scenario.md](../../framework/templates/spec/scenario.md).
- [x] This very task (task 10) serves as the parent-spec task referencing the scenario per §scenarios — no separate task entry needed.
- [x] Verify the file passes `npx markdownlint-cli2`.
- **Done when**: the scenario file documents each of the four edge cases with the expected outcome (drop waiver / keep waiver / re-block).

## 11. Run `/gov:analyze --all` against the govern repo

- [x] After all prior tasks land, run `/gov:analyze --all` to confirm no existing `govern` specs broke. Specifically: `done` specs in `govern` itself will not yet have `review.last-run`, so the new drift check will flag them. Decide per spec whether to backfill (run `/gov:review` against each, accept any findings) or waive on adoption.
- [x] Capture decisions in this task's notes; commit any frontmatter updates that result.
- [x] **Resolution: introduced a grandfather rule during this task.** The validate review-drift check and the CI gate exempt `done` specs whose frontmatter has no `review:` block at all — these predate `/gov:review` and require no backfill. Specs that have the block but `last-run: null` are still flagged (operator has signaled opt-in but hasn't reviewed). Verified the gate exits clean against the current repo (20 existing `done` specs grandfathered; in-progress 020 silently exempt by status).
- [x] Updated `framework/commands/analyze.md` (Review state drift section) and `framework/templates/ci/adopter-generators.yml` (Review-blocking gate step) with the grandfather logic.
- **Done when**: `/gov:analyze --all` exits clean, or every flagged spec has a documented disposition (review run + clean, or waiver recorded).
