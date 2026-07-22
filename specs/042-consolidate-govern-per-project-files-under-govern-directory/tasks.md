# 042 — Consolidate govern per-project files under a `.govern/` directory Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Runtime config + session path resolvers with legacy fallback

- [x] Add `config_path(repo)` and `session_path(repo)` helpers in the runtime config/host layer, each returning the `.govern/` path when it exists and the legacy root path otherwise (new-wins when both exist)
- [x] Route the six config reads through `config_path`: `host.rs::load_host_block`, `schema/paths.rs::Paths::load_configured`, `resolve_references.rs::load_services`, `dashboard.rs::load_review_state`, `discover_rule_files.rs::load_govern_toml`, and the `main.rs` exec walker config seed
- [x] Route the session readers through `session_path`: `dashboard.rs::load_session_target`, `host.rs::load_session_cli_config_dir`, and the `main.rs:353-367` exec walker session seed (replace the literal at `:362`)
- [x] Unit-test both resolvers across new-only, legacy-only, both-present (→ `.govern/`), and neither-present (→ default `.govern/`)

- **Done when**: every config/session read resolves `.govern/` first and falls back to the legacy root path, and unit tests cover all four presence cases for both resolvers.

## 2. Move the session constant and its compile-time guards

- [x] Change `SESSION_FILE` (`write_session.rs:39`) to `.govern/session.toml`
- [x] Update the literal-asserting guards/tests to the new value: `migrate_session_file.rs:189,345`; `write_session.rs:361,429`; `host.rs:262,275,292`
- [x] Confirm `migrate-session-file` (which writes `SESSION_FILE`) now lands the `session-file-consolidate` migration directly at `.govern/session.toml`

- **Done when**: the constant and all its guards/tests use `.govern/session.toml`, and the runtime crate compiles with the compile-time guard satisfied.

## 3. Writers target the active file (migration is the sole cutover)

- [x] Make config writers (`merge-managed-block` host block, `write-review`, and the bootstrap's `last_applied` / project-inputs / workflow-decline writes) and `write-session` target the active file the resolver returns, not the new location unconditionally
- [x] Ensure the bootstrap resolves the config path once per run so `last_applied` read and write-back agree
- [x] Unit-test that a config write with a legacy file present and no `.govern/config.toml` writes to the legacy file (no partial `.govern/` file created)

- **Done when**: no writer outside the migration creates a partial `.govern/config.toml`, and a test proves a pre-migration write stays on the legacy file.

## 4. Runtime write-boundary and generator-script detection recognize `.govern/scripts/`

- [x] Grounded finding: the runtime has **no** hardcoded `scripts/**` default write-boundary. The boundary is entirely git-derived — `derive-boundary` emits a `{dir}/**` zone per touched directory, so `.govern/scripts/` becomes a zone automatically — plus any host/session seed. The `scripts/**` literals in `interpreter/mod.rs` and `derive_boundary.rs` are test scenarios, not production defaults; nothing to extend.
- [x] Grounded finding: generator-script detection is not path-hardcoded — `run-generator` resolves the caller-supplied path, so `.govern/scripts/gen-spec-deps.sh` (from the task-8 command literals) resolves cleanly; the `scripts/*.sh` literals in `primitives/mod.rs` are test example-data.
- [x] `run_generator.rs` needs no change (resolves the caller-supplied path)

- **Done when**: verified the runtime is path-agnostic for scripts — no production code hardcodes `scripts/`, so `.govern/scripts/` is handled by boundary derivation + caller-supplied paths with zero runtime change (mirrors 040's finding that `run_generator` needed no change).

## 5. Runtime integration coverage for both layouts

- [x] Reevaluated the "relocate fixture dirs" plan: the new-then-legacy fallback makes the existing root-layout fixtures valid end-to-end *fallback* coverage, and moving the fixture dirs churned the parity stream-goldens for zero correctness gain — so fixtures stay at the legacy root layout as the fallback proof
- [x] Prove the new layout end-to-end: `exec_subprocess.rs` seeds `.govern/session.toml` and the exec walker resolves it via `session_path`; `paths.rs` unit tests cover both resolvers across all four presence cases (new-only / legacy-only / both→new / neither→default)
- [x] Legacy fallback proven end-to-end by the parity fixtures (root `.govern.session.toml`) and `specs_root_override` (root `.govern.toml`)
- [x] `.gitignore`'s root-anchored `/.govern.session.toml` keeps the tracked fixture session files tracked (fixtures unchanged at root)

- **Done when**: the runtime suite (unit + integration) passes proving new-layout resolution (exec + units) and legacy fallback (parity + specs_root_override). Discovered + logged to inbox: the subprocess tests' `ensure_binary_built()` only builds the release binary when absent, so it must be rebuilt (`cargo build --release`) for exec/parity tests to reflect current code.

## 6. Author the `govern-dir-consolidate` migration

- [x] Add the `[[migrations]]` entry to `framework/migrations.toml` (`id = govern-dir-consolidate`, `introduced_in = 0.22.0`, no `sunset_after`, adopter-relative `target_paths`, `procedure_file`)
- [x] Write `framework/migrations/govern-dir-consolidate.md`: idempotency scan → `git mv`/`mv` the three concerns → converge-on-collision (identical→silent remove, divergent→warn+remove) → pinned-invoker warning → conditional summary line, no inner prompt
- [x] Verify audit Family 10 passes (procedure file exists, no orphan, no stale framework-prefixed target path)

- **Done when**: the registry entry and procedure file exist, the migration is idempotent and converges a split layout, and `scripts/audit/migration-coverage.sh` passes.

## 7. Update the bootstrap `govern.md`

- [x] Change the Shared Files manifest rows (`:635-637`) so source and dest are `.govern/scripts/{gen-spec-deps.sh, gen-cross-service-refs.sh, lib/specs-root.sh}`
- [x] Update config/session path prose to `.govern/config.toml` / `.govern/session.toml` (Instructions steps, §Project Configuration, §Session state, Pre-run Migrations, workflow-decline writes)
- [x] Document the write-active-file / migration-is-sole-cutover behavior

- **Done when**: `govern.md` ships the three generators to `.govern/scripts/`, names the new config/session paths throughout, and documents the cutover rule.

## 8. Repoint shipped-generator literals across commands, hook, CI, and permissions

- [x] Command bodies: `gen-spec-deps.sh` references in `implement.md:98`, `clarify.md:102`, `amend.md:43`, `plan.md:74`, `target.md:39`, `specify.md:39,75`, `analyze.md:34,52` → `.govern/scripts/`; leave `analyze.md`'s `gen-help-tables.sh` (`:34,227`) at `scripts/`
- [x] Adopter hook `framework/bootstrap/hooks/govern-pre-commit:27-28` → `.govern/scripts/` for both shipped gens
- [x] CI template `adopter-generators.yml:8,28,45,50` → `.govern/scripts/`
- [x] Permission allowlists (`configure/claude.md:77-80`, `auggie.md:62-63`, `antigravity.md:56-60`, `opencode.md:58-61`): `gen-*` globs → `.govern/scripts/`; leave `install-hooks.sh` at `scripts/`; update `.govern.toml`/`.govern.session.toml` permission entries to `.govern/…`

- **Done when**: every shipped-generator literal resolves to `.govern/scripts/…`, maintainer-only literals are untouched, and the four agents' permission sets cover the new paths.

## 9. gitignore template supersession

- [x] Change `framework/templates/project/gitignore` session entry to `/.govern/session.toml`
- [x] Confirm the `/govern` managed-block rewrite supersedes an adopter's old `.govern.session.toml` line (no dangling entry) and leaves `.govern/config.toml` / `.govern/scripts/` tracked

- **Done when**: a `/govern` run leaves `.govern/session.toml` ignored, no dangling `.govern.session.toml` line, and config/scripts tracked.

## 10. Documentation and canonical sources

- [ ] Constitution: update the canonical-sources table, the `§concurrent-features` session-state paragraph (`:569`), the generator-provenance notes (`:421,422` → `.govern/scripts/`; leave `:323` `lint-rule-filenames.sh` at `scripts/`), and `[services]`/`[paths]`/`[review]` path mentions
- [ ] `AGENTS.md`: update session/config path notes (`:43-49`) and the three-site wiring rule (`:61`) to name `.govern/scripts/` as the shipped-generator home
- [ ] `README.md`: update `.govern.toml` schema mentions and `gen-cross-service-refs.sh` paths (`:236,247`)
- [ ] Update config/session path prose in the reading commands (`link`, `review`, `status`, `analyze`, `groom`, `target`, `amend`, `implement`, `prune`, `plan`, `clarify`, `specify`, `help`)

- **Done when**: no framework doc or command body carries a stale root-path reference to the three moved artifacts, and the three-site wiring rule names the new generator home.

## 11. Dogfood the new layout in govern's own repo

- [ ] `git mv .govern.toml .govern/config.toml`; move gitignored `.govern.session.toml` → `.govern/session.toml`; `git mv` the three generators (with `lib/`) into `.govern/scripts/`
- [ ] Fix `.gitignore:9` to `/.govern/session.toml`; update `.githooks/pre-commit:32-36` (two shipped gens → `.govern/scripts/`, three maintainer gens stay at `scripts/`); update `.shellcheckrc` comments
- [ ] Run a full `/gov:*` cycle and the pre-commit generators on the dogfooded layout with no path errors

- **Done when**: govern's own config/session/generators live under `.govern/`, maintainer scripts remain at `scripts/`, and the pipeline plus pre-commit generators run clean.

## 12. Update maintainer audit scripts

- [ ] `scripts/audit/fixture-session-shape.sh:53` — find `.govern/session.toml` under the fixtures tree
- [ ] `scripts/audit/consolidation-pair.sh` — update the motivating-case path reference
- [ ] Confirm `scripts/audit/migration-coverage.sh` accepts the new registry entry, and run the full `scripts/audit/run-all.sh`

- **Done when**: `scripts/audit/run-all.sh` passes on the dogfooded layout with the new migration entry.

## 13. End-to-end verification

- [ ] Fresh-adopter path: a first `/govern` run scaffolds directly to `.govern/` with no root-level govern files
- [ ] Migration path: a legacy-layout fixture project runs `/govern`, converges to `.govern/`, and is a no-op on re-run
- [ ] Pre-migration fallback: `/gov:status` on a legacy-layout project (before `/govern`) reads correctly via fallback with no path error

- **Done when**: fresh-adopter, migration, and pre-migration-fallback paths all pass, and the runtime + audit suites are green.
