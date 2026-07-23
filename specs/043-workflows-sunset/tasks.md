# 043 — Workflows sunset Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Register the workflows-sunset migration

- [x] Write `framework/migrations/workflows-sunset.md` (exact-set deletion of the 22 filenames, legacy `skills/` removal, synced `workflows/registry.json` removal, `[workflows]` section removal from the active config file — per-file pinned checks, summary lines, idempotency check first)
- [x] Add the `[[migrations]]` entry to `framework/migrations.toml` (`introduced_in = "0.23.0"`, `sunset_after = "0.25.0"`, target paths incl. `framework/workflows/`), relocating the audit-convention comment from the `skills-to-workflows` entry

- **Done when**: the registry parses, the new entry's `procedure_file` exists, and its target paths cover every path the two subsumed entries listed plus the synced registry copy and `framework/workflows/`.

## 2. Archive the two 005-era migrations

- [x] Append both procedure texts to `CHANGELOG.md` § Archived migrations (headings naming id, `introduced_in`, `sunset_after`), replacing the *"None yet"* placeholder
- [x] Remove both `[[migrations]]` entries from `framework/migrations.toml` and delete `framework/migrations/skills-to-workflows.md` and `framework/migrations/workflow-filename-rename.md`

- **Done when**: neither id appears in `framework/migrations.toml`, both recipes are readable in `CHANGELOG.md`, and no `framework/migrations/*.md` file is orphaned from the registry.

## 3. Delete `framework/workflows/` and excise the bootstrap flow

- [x] `git rm` the 14 files under `framework/workflows/`
- [x] Remove from `framework/bootstrap/govern.md`: §Workflow recommendation (incl. Tech Stack parsing + Auggie note), the manifest row, the `[workflows]` config schema block and `declined_categories` paragraph, and the mentions at the procedural-fidelity preamble, managed-block preserve list, enforce-manifest note, write-policy paragraph, manifest-scope sentence, and both per-layout skip instructions

- **Done when**: `grep -i workflow framework/bootstrap/govern.md` returns only generic uses (tar-xzf sentence, PKM tip) and `framework/workflows/` does not exist.

## 4. Sweep constitution, command sources, templates

- [x] `framework/constitution.md`: drop the Workflow registry map row; reword §runtime-boundary criterion 2(b)
- [x] `framework/commands/groom.md` (2 sites) and `framework/commands/link.md` (preserve list): reword per plan
- [x] `framework/templates/project/agents.md`: drop the workflows disambiguation from the Skills comment
- [x] Regenerate `.claude/commands/gov/` copies via `scripts/gen-claude-commands.sh`

- **Done when**: no framework artifact outside `CHANGELOG.md` references the feature, and regenerated copies match their sources.

## 5. Hand-sweep `init.md`, README, AGENTS.md

- [x] `.claude/commands/gov/init.md`: remove §8, renumber later steps, reword the `:40` inference justification
- [x] `README.md`: drop the `[workflows]` config docs, example TOML section, repo-layout row; run the prose-claim pass (scaffold/recommend/decline phrasings)
- [x] `AGENTS.md`: drop the `framework/workflows/` project-structure bullet, the ships-as-is gotcha, and the "per-category workflow prompts" phrase in the procedural-fidelity mirror

- **Done when**: the meaning-based sweep over README, AGENTS.md, and `.claude/commands/` finds no claim that govern offers, scaffolds, or records declines for workflows.

## 6. Runtime comment sweep, version bump, changelog

- [x] `runtime/src/schema/paths.rs` doc comment: drop `[workflows]` from the config-writes enumeration
- [x] `runtime/src/primitives/enforce_manifest.rs` module doc: generalize the legacy-conventions example
- [x] Bump `runtime/Cargo.toml` to `0.23.0`; add the `runtime/CHANGELOG.md` 0.23.0 section
- [x] `cargo build --release` (refresh the parity binary) and `cargo test` under `runtime/`; re-bless only a golden whose embedded command text changed — never for the version line

- **Done when**: `cargo test` passes with the version rendered via the `{{runtime-version}}` placeholder and no golden diff attributable to the bump.

## 7. Annotate the sibling done specs

- [x] `specs/005-workflows/spec.md`: post-completion sunset blockquote pointing at 043 (`status: done` untouched)
- [x] One-line historical-surface notes in `specs/004-tech-stack-selection/spec.md`, `specs/010-agent-autonomy/spec.md`, `specs/019-config-decisions/spec.md`

- **Done when**: each note links `../043-workflows-sunset/spec.md`, no sibling spec's `status` changed, and `gen-spec-deps.sh --dry-run` reports only expected link-derived changes.

## 8. Full-sweep verification and audit

- [ ] Run the token sweep + prose-claim pass from the plan's Sweep verification section; classify every remaining hit
- [ ] Run `scripts/audit/run-all.sh` and `npx markdownlint-cli2` over touched markdown

- **Done when**: every sweep hit falls in an allowed class, the audit passes clean, and lint reports no new violations.

## 9. Release gate

- [ ] Commit to `main` (explicit paths, no `git add -A`) and push after review
- [ ] Tag `gvrn-v0.23.0` at that commit and push the tag; confirm the release pipeline's self-audit gate passes

- **Done when**: the `gvrn-v0.23.0` tag is published and the runtime-release workflow completes green.
