# 027 — Bootstrap Migration Registry Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create the empty registry and seed CHANGELOG.md

- [x] Create `framework/migrations.toml` containing only a header comment that explains the file's purpose and links to spec 027.
- [x] Create `CHANGELOG.md` at the repo root with the seed content described in plan §CHANGELOG.md seed format (no archived entries yet; just the introduction).
- [x] Verify both files pass `npx markdownlint-cli2` (TOML file is excluded from lint; CHANGELOG.md is included).

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

## 3. Back-fill the registry with six entries

Look up `introduced_in` per migration via `git log` against the commits that shipped each removal. Use `registry_introduction_version + 2 minor versions` for every `sunset_after` (where `registry_introduction_version` is the gvrn release this spec lands in — pinned at release time).

- [ ] Run `git log --diff-filter=D --name-only -- framework/templates/spec-and-plan.md framework/commands/spawn.md 'framework/skills/*'` (and similar) to find each migration's commit and corresponding gvrn tag.
- [ ] Append the six `[[migrations]]` entries to `framework/migrations.toml` with: `id`, `introduced_in`, `sunset_after`, `summary`, `target_paths`, `procedure_file`.
- [ ] Verify ordering: entries sort by `introduced_in` SemVer ascending, lex tie-break on `id`. File order in TOML is not authoritative but should match for human readability.
- [ ] Run `tq` or equivalent to confirm `framework/migrations.toml` parses cleanly.

## 4. Rewrite `framework/bootstrap/govern.md` Pre-run Migrations section

- [ ] Replace the existing `## Pre-run Migrations` section (lines ~190–250) with the registry-driven loop described in plan §Bootstrap loop placement and shape.
- [ ] Delete the `### Legacy skills/ directory cleanup` sub-section inside `## Workflow recommendation` (line ~570).
- [ ] Delete the `### Legacy workflow cleanup` content (step 1 of the workflow recommendation procedure, line ~586).
- [ ] Update the `### Legacy directory note` at the end of the workflow recommendation section (line ~677) to reference the registry instead of the deleted sub-sections.
- [ ] Update the procedural-fidelity rule at line 24 to drop the "legacy `spec-and-plan.md` rename" exception (the registry-driven loop's outer batch prompt subsumes it).
- [ ] Add the `[migrations]` section to the `.govern.toml` schema documented in `## Project Configuration` (line ~252+). Document `last_applied` field with its absence semantics.
- [ ] Update the `enforce-manifest` step at line 36 to drop the "legacy `skills/` directory removal" and "legacy workflow filename removal" mentions from its summary line. The primitive's expected-list construction loses the legacy paths.
- [ ] Run `npx markdownlint-cli2` on `framework/bootstrap/govern.md`.

## 5. Trim `enforce-manifest` primitive's expected-list contract

- [ ] In the runtime crate (`runtime/src/primitives/enforce_manifest.rs` or equivalent), remove the legacy-path inclusion logic from the expected-list construction.
- [ ] Update tests under `runtime/tests/` that assert the legacy-path removal behavior. Replace with tests asserting the legacy paths are NOT touched by `enforce-manifest`.
- [ ] Add a CHANGELOG entry to `runtime/CHANGELOG.md` describing the contract trim (likely a minor version bump).
- [ ] Run `cargo test` from the runtime workspace.

## 6. Implement Family 10 audit script

- [ ] Create `scripts/audit/migration-coverage.sh` matching plan §Family 10 design.
- [ ] Implement check 10a — no orphan procedure files.
- [ ] Implement check 10b — no stale target paths (parse both `framework/migrations.toml` and `CHANGELOG.md` archived sections).
- [ ] Implement check 10c — no broken procedure references.
- [ ] Make the script executable (`chmod +x`).
- [ ] Verify the script exits 0 against the post-back-fill state (all six entries valid, all procedure files present).

## 7. Wire Family 10 into the audit orchestrator and command doc

- [ ] Append the Family 10 `run_check` line to `scripts/audit/run-all.sh` after the Family 9 invocation.
- [ ] Append a numbered step for Family 10 in `framework/commands/audit.md`'s Markdown-only reference.
- [ ] Update the "eight family check" / "nine family check" prose in `framework/commands/audit.md` and `specs/026-framework-self-audit/spec.md` references to "ten family check" — or accept that the count is now derivable and remove the magic number.
- [ ] Run `/audit` end-to-end (`bash scripts/audit/run-all.sh`) and verify exit 0.

## 8. Record cross-spec impact on 026

- [ ] Create `specs/026-framework-self-audit/scenarios/family-10-migration-coverage.md` with a back-link to 027 documenting the Family 10 extension.
- [ ] The scenario file describes Family 10's three checks at scenario-level detail and notes that 027 is the driving spec.
- [ ] Run `npx markdownlint-cli2` on the new scenario file.

## 9. Update CLAUDE.md / AGENTS.md if needed

- [ ] Check whether `AGENTS.md` mentions any of the legacy migrations or `.govern.toml` schema details. Update if any text references the old prose-encoded migrations or omits the new `[migrations]` section.
- [ ] No changes if AGENTS.md is silent on these topics — the bootstrap procedure is the canonical reference.

## 10. End-to-end verification

- [ ] Manually run through the bootstrap loop's prose against a fresh fixture: empty `.govern.toml`, expect all six entries to run, expect `last_applied` written.
- [ ] Manually run against an updated fixture: `last_applied = "rule-files-relocate"`, expect only `skills-to-workflows` and `workflow-filename-rename` to run.
- [ ] Manually run against an up-to-date fixture: `last_applied = "<newest entry id>"`, expect zero entries to run and zero filesystem reads beyond the registry.
- [ ] Run `/audit` (`bash scripts/audit/run-all.sh`); confirm exit 0.
- [ ] Run `npx markdownlint-cli2` against the entire feature directory and all modified framework files.

## 11. Final review and status advancement

- [ ] Run `/gov:review` on the spec to surface any rule violations.
- [ ] Resolve any MUST findings; record SHOULD findings as scenarios if deferred.
- [ ] Advance `spec.md` `status` from `in-progress` to `done` via the implement flow's done gate.
