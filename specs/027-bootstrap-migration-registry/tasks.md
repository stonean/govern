# 027 — Bootstrap Migration Registry Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create the empty registry and seed CHANGELOG.md

- [x] Create `framework/migrations.toml` containing only a header comment that explains the file's purpose and links to spec 027.
- [x] Create `CHANGELOG.md` at the repo root with the seed content described in plan §CHANGELOG.md seed format (no archived entries yet; just the introduction).
- [x] Verify both files pass `npx markdownlint-cli2` (TOML file is excluded from lint; CHANGELOG.md is included).

- **Done when**: `framework/migrations.toml` (header only) and a seeded root `CHANGELOG.md` exist and pass `npx markdownlint-cli2`.

## 2. Author the six procedure files

For each of the six back-filled migrations, create `framework/migrations/{id}.md` following the plan §Procedure file shape convention. Each procedure file lifts its body content from the corresponding existing prose in `framework/bootstrap/govern.md`.

- [x] `framework/migrations/governance-config-rename.md` (lifted from `framework/bootstrap/govern.md` `### .governance.toml → .govern.toml`).
- [x] `framework/migrations/gitignore-marker-rename.md` (lifted from `### # Governance gitignore marker → # govern`).
- [x] `framework/migrations/spec-and-plan-sunset.md` (lifted from `### spec-and-plan.md → spec.md (lightweight-track sunset)`).
- [x] `framework/migrations/rule-files-relocate.md` (lifted from `### Rule files: relocate to specs/rules/`, subsuming `configuration.md` rename).
- [x] `framework/migrations/skills-to-workflows.md` (lifted from `### Legacy skills/ directory cleanup`).
- [x] `framework/migrations/workflow-filename-rename.md` (lifted from `Legacy workflow cleanup` step 1 inside `## Workflow recommendation`).
- [x] Each procedure file starts with an idempotency check (step 1) that exits silently when the target artifact is absent.
- [x] Each procedure file ends with the post-scaffolding summary line step.
- [x] All six files pass `npx markdownlint-cli2`.

- **Done when**: all six `framework/migrations/{id}.md` procedure files exist — each opening with an idempotency check and closing with the summary-line step — and pass `npx markdownlint-cli2`.

## 3. Back-fill the registry with six entries

Look up `introduced_in` per migration via `git log` against the commits that shipped each removal. Use `registry_introduction_version + 2 minor versions` for every `sunset_after` (where `registry_introduction_version` is the gvrn release this spec lands in — pinned at release time).

- [x] Run `git log --diff-filter=D --name-only -- framework/templates/spec-and-plan.md framework/commands/spawn.md 'framework/skills/*'` (and similar) to find each migration's commit and corresponding gvrn tag.
- [x] Append the six `[[migrations]]` entries to `framework/migrations.toml` with: `id`, `introduced_in`, `sunset_after`, `summary`, `target_paths`, `procedure_file`.
- [x] Verify ordering: entries sort by `introduced_in` SemVer ascending, lex tie-break on `id`. File order in TOML is not authoritative but should match for human readability.
- [x] Run `tq` or equivalent to confirm `framework/migrations.toml` parses cleanly.

- **Done when**: `framework/migrations.toml` carries the six `[[migrations]]` entries (each with `id`, `introduced_in`, `sunset_after`, `summary`, `target_paths`, `procedure_file`), ordered by `introduced_in` SemVer, and parses cleanly.

## 4. Rewrite `framework/bootstrap/govern.md` Pre-run Migrations section

- [x] Replace the existing `## Pre-run Migrations` section (lines ~190–250) with the registry-driven loop described in plan §Bootstrap loop placement and shape.
- [x] Delete the `### Legacy skills/ directory cleanup` sub-section inside `## Workflow recommendation` (line ~570).
- [x] Delete the `### Legacy workflow cleanup` content (step 1 of the workflow recommendation procedure, line ~586).
- [x] Update the `### Legacy directory note` at the end of the workflow recommendation section (line ~677) to reference the registry instead of the deleted sub-sections.
- [x] Update the procedural-fidelity rule at line 24 to drop the "legacy `spec-and-plan.md` rename" exception (the registry-driven loop's outer batch prompt subsumes it).
- [x] Add the `[migrations]` section to the `.govern.toml` schema documented in `## Project Configuration` (line ~252+). Document `last_applied` field with its absence semantics.
- [x] Update the `enforce-manifest` step at line 36 to drop the "legacy `skills/` directory removal" and "legacy workflow filename removal" mentions from its summary line. The primitive's expected-list construction loses the legacy paths.
- [x] Run `npx markdownlint-cli2` on `framework/bootstrap/govern.md`.

- **Done when**: `framework/bootstrap/govern.md`'s `## Pre-run Migrations` is the registry-driven loop, the legacy cleanup sub-sections are removed, the `.govern.toml` `[migrations]` schema is documented, and the file passes `npx markdownlint-cli2`.

## 5. Trim `enforce-manifest` primitive's expected-list contract

- [x] In the runtime crate (`runtime/src/primitives/enforce_manifest.rs` or equivalent), remove the legacy-path inclusion logic from the expected-list construction. (The primitive's behavior was already generic — caller supplies `expected`/`pinned` and the runtime never hardcoded legacy paths. The trim was the module docstring's claim that the primitive replaced "three legacy cleanup loops"; the docstring now scopes the primitive to slash-command manifest enforcement only, with adopter-cleanup owned by the registry-driven `## Pre-run Migrations` loop.)
- [x] Update tests under `runtime/tests/` that assert the legacy-path removal behavior. Replace with tests asserting the legacy paths are NOT touched by `enforce-manifest`. (No integration test previously asserted legacy-path removal — the bootstrap caller already passed only `framework/commands/` as the enforce directory, and the fixture's only "legacy" file was a slash-command path (`legacy-cmd.md`). Added a new pre-seeded fixture file at `runtime/tests/fixtures/govern-basic/project/framework/skills/old-skill.md` and a parity-test assertion that the file survives the bootstrap end-to-end, locking the contract trim in place against regression.)
- [x] Add a CHANGELOG entry to `runtime/CHANGELOG.md` describing the contract trim. (Patch bump 0.7.1 → 0.7.2 — no behavior change for the slash-command path, no API change, no wire-format change; the trim is docstring + regression-guard test only.)
- [x] Run `cargo test` from the runtime workspace. (All 325 tests pass; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean.)

- **Done when**: `enforce-manifest`'s docstring scopes it to slash-command manifest enforcement, a regression test locks that legacy paths survive, `runtime/CHANGELOG.md` records the trim, and `cargo test` / `clippy` / `fmt --check` are clean.

## 6. Implement Family 10 audit script

- [x] Create `scripts/audit/migration-coverage.sh` matching plan §Family 10 design.
- [x] Implement check 10a — no orphan procedure files.
- [x] Implement check 10b — no stale target paths (parse both `framework/migrations.toml` and `CHANGELOG.md` archived sections). CHANGELOG.md archived-entry parsing deferred until the first sunset commit establishes the archive format by example — TODO in the script.
- [x] Implement check 10c — no broken procedure references.
- [x] Make the script executable (`chmod +x`).
- [x] Verify the script exits 0 against the post-back-fill state (all six entries valid, all procedure files present).

- **Done when**: `scripts/audit/migration-coverage.sh` implements checks 10a–10c, is executable, and exits 0 against the post-back-fill state.

## 7. Wire Family 10 into the audit orchestrator and command doc

- [x] Append the Family 10 `run_check` line to `scripts/audit/run-all.sh` after the Family 9 invocation.
- [x] Append a numbered step for Family 10 in `framework/commands/audit.md`'s Markdown-only reference.
- [x] Update the "eight family check" / "nine family check" prose in `framework/commands/audit.md` to a count-agnostic phrasing ("family check scripts"). The 026 spec body's references to family counts are deferred to the Task 8 scenario.
- [x] Run `/audit` end-to-end (`bash scripts/audit/run-all.sh`) and verify exit 0.

- **Done when**: Family 10 is invoked from `scripts/audit/run-all.sh` and documented in `framework/commands/audit.md`, and `bash scripts/audit/run-all.sh` exits 0.

## 8. Record cross-spec impact on 026

- [x] Create `specs/026-framework-self-audit/scenarios/family-10-migration-coverage.md` with a back-link to 027 documenting the Family 10 extension.
- [x] The scenario file describes Family 10's three checks at scenario-level detail and notes that 027 is the driving spec.
- [x] Run `npx markdownlint-cli2` on the new scenario file.

- **Done when**: `specs/026-framework-self-audit/scenarios/family-10-migration-coverage.md` exists with a back-link to 027 and passes `npx markdownlint-cli2`.

## 9. Update CLAUDE.md / AGENTS.md if needed

- [x] Check whether `AGENTS.md` mentions any of the legacy migrations or `.govern.toml` schema details. Update if any text references the old prose-encoded migrations or omits the new `[migrations]` section.
- [x] No changes if AGENTS.md is silent on these topics — the bootstrap procedure is the canonical reference. Updated line 43's procedural-fidelity mirror (spec-and-plan rename → registry-driven migration prompts) and incidentally fixed a stale spec reference at line 45 (the reverted 027-command-source-templating → 027-bootstrap-migration-registry).

- **Done when**: `AGENTS.md` no longer references the old prose-encoded migrations — the procedural-fidelity mirror points at the registry-driven prompts and the stale spec reference is corrected.

## 10. End-to-end verification

- [x] Manually run through the bootstrap loop's prose against a fresh fixture: empty `.govern.toml`, expect all six entries to run, expect `last_applied` written. (Trace: `last_applied = null`; sunset filter passes for all six (current gvrn 0.7.2 < 0.10.0). Filter order, by `introduced_in` ascending with lex tie-break on `id`: `gitignore-marker-rename` → `governance-config-rename` → `skills-to-workflows` → `workflow-filename-rename` → `spec-and-plan-sunset` → `rule-files-relocate`. Prompt: "6 framework migrations are pending…". On confirm, all six procedures dispatch in that order; `[migrations].last_applied` is rewritten per entry, ending at `"rule-files-relocate"`.)
- [x] Manually run against an updated fixture: `last_applied = "rule-files-relocate"`, expect only `skills-to-workflows` and `workflow-filename-rename` to run. (Discrepancy: as the registry was actually back-filled in task 3, `rule-files-relocate` is the **newest** entry (`introduced_in = 0.6.0`), not a mid-point. Per the prose filter (`introduced_in > last_applied.introduced_in`, lex tie-break on `id`), `last_applied = "rule-files-relocate"` yields **zero** qualifying entries — identical to State 3. A faithful mid-point trace, substituting `last_applied = "governance-config-rename"` (0.2.0, lex pos #2 of the four 0.2.0 entries), yields four entries in filter order: `skills-to-workflows` (0.2.0, lex-after) → `workflow-filename-rename` (0.2.0, lex-after) → `spec-and-plan-sunset` (0.5.0) → `rule-files-relocate` (0.6.0). The filter logic is sound; the task's example id was written before the back-fill pinned `rule-files-relocate` as the newest.)
- [x] Manually run against an up-to-date fixture: `last_applied = "<newest entry id>"`, expect zero entries to run and zero filesystem reads beyond the registry. (Trace: `last_applied = "rule-files-relocate"` (introduced_in 0.6.0). Filter rejects every entry (introduced_in ≤ 0.6.0, lex tie-break rejects equal-id). Loop step 4 ("If the filtered list is empty, emit nothing and proceed") fires; no `framework/migrations/*.md` files are read, no `.govern.toml` write, no prompt. Bootstrap proceeds directly to the next section.)
- [x] Run `/audit` (`bash scripts/audit/run-all.sh`); confirm exit 0. (Ran clean: exit 0, no findings.)
- [x] Run `npx markdownlint-cli2` against the entire feature directory and all modified framework files. (Feature directory + modified framework files: 0 errors. Note: `runtime/CHANGELOG.md` carries two pre-existing MD038 findings in its 0.6.1 entry (a code span containing the literal `` `### 19. Dedup `/configure` permission entries` ``) — confirmed pre-existing via `git show HEAD~7:runtime/CHANGELOG.md`. Out of scope for this spec; would belong to a separate runtime-lint cleanup.)

- **Done when**: the registry-driven bootstrap loop is traced against empty, partial, and up-to-date `.govern.toml` fixtures with the expected entries running each time, and `/audit` plus `npx markdownlint-cli2` are clean.

## 11. Final review and status advancement

- [x] Run `/gov:review` on the spec to surface any rule violations. (Ran 2026-05-22T02:32:17Z against HEAD `3e0053f`; loaded `security-backend.md`, `api-backend.md`, `configuration-cross.md`; skipped frontend rules (no frontend code in scope); tech-stack alignment skipped per `[review] tech-stack-verified = true`. Report at [review.md](review.md): 0 MUST / 0 SHOULD / 0 low-confidence across all five passes; `blocking: false`. Spec frontmatter updated with the review result.)
- [x] Resolve any MUST findings; record SHOULD findings as scenarios if deferred. (No findings to resolve; nothing to defer.)
- [x] Advance `spec.md` `status` from `in-progress` to `done` via the implement flow's done gate. (Flipped all 29 acceptance criteria to `[x]` first so the spec body reflects completed work, then ran `set-status` from `in-progress` → `done`. Final state: status=done, review.blocking=false, review.last-run set, all ACs checked. `/gov:analyze` re-run post-flip confirms no Review-state-drift finding (done spec, review block populated and unblocked).)

- **Done when**: `/gov:review` records 0 MUST violations with `blocking: false`, and `spec.md` is advanced to `done` via the done gate with all acceptance criteria checked.
