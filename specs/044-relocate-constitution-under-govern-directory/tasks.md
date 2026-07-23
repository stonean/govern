# 044 — Relocate the shipped constitution to `.govern/constitution.md` Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Register the `constitution-relocate` migration

- [x] Add the `[[migrations]]` entry to `framework/migrations.toml`: `id = "constitution-relocate"`, `introduced_in = "0.24.0"`, no `sunset_after` (with the indefinite-retention comment mirroring `govern-dir-consolidate`'s), `target_paths = ["constitution.md"]`, `procedure_file = "framework/migrations/constitution-relocate.md"`
- [x] Write `framework/migrations/constitution-relocate.md` per the plan: idempotency check, convergence rule, `git mv` move, seed-reference rewrites with the absent-file/altered-line rules, pin re-point, pinned-command warning, summary line

- **Done when**: the registry entry parses, the procedure file documents every behavior the spec's migration section names, and the audit's migration-coverage invariants (Family 10) pass.

## 2. Update the bootstrap manifest and prose

- [x] Change the Shared Files row `framework/constitution.md` → destination `.govern/constitution.md` (`framework/bootstrap/govern.md:614`)
- [x] Update the `[pinned] files` schema example (`govern.md:432`) to `.govern/constitution.md`

- **Done when**: `rg -n 'constitution' framework/bootstrap/govern.md` shows no adopter-root `constitution.md` destination — only the `framework/constitution.md` source and `.govern/constitution.md` destination forms.

## 3. Sweep the shipped command bodies

- [x] Update the bare adopter-path references to `.govern/constitution.md`: `target.md:19`, `specify.md:25,68`, `groom.md:64`, `clarify.md:100`, `analyze.md:123,178,216,220,228`
- [x] Update `analyze.md:48`'s dual-path rule: `framework/constitution.md` in govern's own repo; `.govern/constitution.md` at the adopter repo root
- [x] Run the full sweep grep over `framework/commands/` to catch references my scoping grep filtered (lines mentioning both path forms)

- **Done when**: `rg -n '\bconstitution\.md' framework/commands/` returns only `framework/constitution.md` and `.govern/constitution.md` forms.

## 4. Update the project seed templates

- [x] `claude-md.md:3` → `@import .govern/constitution.md`
- [x] `agents.md:9,53` → link/list `.govern/constitution.md`
- [x] `project-readme.md:20,35` → Documentation bullet and pipeline link target `.govern/constitution.md` (no new governance blurb, per the resolved question)

- **Done when**: `rg -n 'constitution' framework/templates/project/` shows only `.govern/constitution.md` references.

## 5. Update govern's own docs

- [x] Fix `AGENTS.md:9`'s dead root link to `framework/constitution.md`
- [x] Reword `AGENTS.md:15`'s sync-target parenthetical to name `.govern/constitution.md` as the adopter destination
- [x] Update `README.md:210,217,284` pinned/strategy examples to `.govern/constitution.md`

- **Done when**: no bare root `constitution.md` reference remains in `AGENTS.md` / `README.md`, and every constitution link in them resolves to an existing file.

## 6. Cut the runtime version

- [x] Bump `runtime/Cargo.toml` to 0.24.0
- [x] Add the `runtime/CHANGELOG.md` 0.24.0 entry: no runtime behavior changes; the version anchors `constitution-relocate`'s `introduced_in` (043 precedent)

- **Done when**: `cargo build` succeeds at 0.24.0 and the changelog's top entry documents the no-behavior-change cut.

## 7. Verification sweep

- [x] Repo-wide stale-reference grep over live artifacts (`framework/`, `runtime/`, `docs/`, `README.md`, `AGENTS.md`, `specs/`): no adopter-root `constitution.md` reference remains outside historical spec bodies that describe the old layout as old
- [x] `npx markdownlint-cli2` clean over touched markdown
- [x] `scripts/audit/cross-doc-consistency.sh` and `scripts/audit/ssot-invariants.sh` pass
- [x] `cargo test` parity suite green with no golden/fixture diffs (confirms the synthetic-fixture finding)

- **Done when**: all four checks pass with no re-blessed goldens.

## 8. Publish gvrn-v0.24.0

- [ ] Tag and publish the `gvrn-v0.24.0` release so the registry entry's `introduced_in` references a published release

- **Done when**: the `gvrn-v0.24.0` release is published and `framework/migrations.toml`'s entry points at a live version.
