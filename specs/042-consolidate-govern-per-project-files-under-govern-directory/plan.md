# 042 — Consolidate govern per-project files under a `.govern/` directory Plan

Implements [042 — Consolidate govern per-project files under a `.govern/` directory](spec.md).

## Overview

Move govern's three per-project artifacts off the repo root into one `.govern/` directory — `.govern/config.toml` (was `.govern.toml`), `.govern/session.toml` (was `.govern.session.toml`), and `.govern/scripts/` (was the three adopter-facing generators in root `scripts/`) — and teach every reader, writer, migration, and doc to use the new locations. The work spans four surfaces:

1. **Runtime (`gvrn`)** — two shared path resolvers (config, session) that return the `.govern/` path when it exists and fall back to the legacy root path otherwise; every ad-hoc `repo.join(".govern.toml")` and the `SESSION_FILE` constant route through them. Writers target the *active* file (never a partial `.govern/` file).
2. **Bootstrap migration (`/govern`)** — a new `govern-dir-consolidate` registry entry (`introduced_in = 0.22.0`, no `sunset_after`) plus its procedure file; the sole cutover that moves the three artifacts, converging any split state.
3. **Markdown surface** — the Shared Files manifest, the shipped command bodies / hooks / CI template / permission allowlists that name the three generators by literal path, the gitignore template, and the framework docs (constitution, `AGENTS.md`, `README.md`).
4. **Dogfooding + tests** — govern's own repo moves to the new layout (config, session, the three generators, its pre-commit hook, `.gitignore`, `.shellcheckrc`), the runtime fixtures move to the new layout, and the maintainer audit scripts follow.

Two load-bearing invariants: **(a) nothing breaks before `/govern` runs** — readers fall back to the legacy path and writers stay on the active file, so an adopter who upgrades gvrn first sees identical behavior; **(b) only the three adopter-facing generators move** — every maintainer-only script (audit, lints, `gen-help-tables.sh`, `gen-configure-mcp.sh`, `gen-claude-commands.sh`, `install-hooks.sh`) stays at root `scripts/`, so `.govern/scripts/` structurally marks "ships to adopters."

## Technical Decisions

### Two shared runtime path resolvers with legacy fallback

Today the runtime resolves both files as bare `repo.join("<literal>")` at every consumer — the session file through one constant, the config file ad-hoc per reader. Add two resolvers in the config/host layer (alongside the existing `[services]`/`[rules]`/`specs-root` parsing):

```rust
// <repo>/.govern/config.toml if it exists, else <repo>/.govern.toml (legacy)
fn config_path(repo: &Path) -> PathBuf
// <repo>/.govern/session.toml if it exists, else <repo>/.govern.session.toml (legacy)
fn session_path(repo: &Path) -> PathBuf
```

Both encode **new-if-exists-else-legacy**, so a split state (both present) resolves to `.govern/` (new-wins). The config resolver replaces the six ad-hoc reads: `host.rs::load_host_block` (`runtime/src/host.rs:95-108`), `schema/paths.rs::Paths::load_configured` (`runtime/src/schema/paths.rs:110-140`), `primitives/resolve_references.rs::load_services` (`:96-97`), `primitives/dashboard.rs::load_review_state` (`:483-508`), `primitives/discover_rule_files.rs::load_govern_toml` (`:336-337`), and the `run_exec` walker seed in `runtime/src/main.rs` (currently a literal at `:362`). The session resolver is consumed by the readers — `dashboard.rs::load_session_target` (`:534-558`), `host.rs::load_session_cli_config_dir` (`:114-118`), and the exec walker seed (`main.rs:353-367`).

One resolver per file (not a fallback re-implemented per call site) keeps the fallback and the "new-wins" rule in one place — the drift-prevention discipline applied to code. This edits existing path construction plus adds two helpers; it introduces **no new primitive**, so the new-primitive wiring checklist (schema/parser/interpreter/main/server/`runtime-tools.txt`) does not apply.

### Session constant + compile-time guards move to the new path

`SESSION_FILE` (`runtime/src/primitives/write_session.rs:39`) becomes `.govern/session.toml`. Because `migrate-session-file` writes its destination as that constant (`runtime/src/primitives/migrate_session_file.rs:25,49,66,96`), the older `session-file-consolidate` migration then lands an ancient adopter's per-agent JSON **directly** at `.govern/session.toml` — no root-path staging (spec Resolved Questions). The literal-asserting guards and tests move to the new value: the compile-time guard `migrate_session_file.rs:345` and result assertion `:189`; `write_session.rs:361,429`; `host.rs:262,275,292`.

### Write policy — the `/govern` migration is the sole cutover

Writers target the **active file** the resolver returns (new if it exists, else legacy, defaulting to new when neither exists), *not* the new location unconditionally. This is a correction surfaced in planning: a runtime config write that always named `.govern/config.toml` — e.g. `write-review`'s `[review] tech-stack-verified` from `/gov:review` run before `/govern` — would create a partial file holding only `[review]`, and new-wins-on-read would strand the legacy file's `[pinned]`/`[services]`/`[project]`. Writing the active file keeps every section together until the migration moves the whole file as one unit. Affects the config writers (`merge-managed-block` against the host block, `write-review`, and the bootstrap's `[migrations].last_applied` / project-inputs / workflow-decline writes) and `write-session`. The bootstrap resolves the config path **once per run** so reading `last_applied` at the start and writing it back after each migration agree, even though the config file is itself a migration target.

### The `govern-dir-consolidate` migration

A new `[[migrations]]` entry in `framework/migrations.toml` (`id = "govern-dir-consolidate"`, `introduced_in = "0.22.0"`, **no** `sunset_after`, `procedure_file = "framework/migrations/govern-dir-consolidate.md"`, adopter-relative `target_paths` = `.govern.toml`, `.govern.session.toml`, `scripts/gen-spec-deps.sh`, `scripts/gen-cross-service-refs.sh`, `scripts/lib/specs-root.sh`) plus the procedure file. Structurally modeled on `rule-files-relocate.md` (idempotency scan → move → conditional summary) with three deliberate differences:

- **Three concerns, one pass** — move `.govern.toml` → `.govern/config.toml`, `.govern.session.toml` → `.govern/session.toml`, and the three generators (with `lib/`) from `scripts/` → `.govern/scripts/`, via `git mv` when tracked (`mv` otherwise), preserving history and any adopter customization.
- **Converge, don't skip-and-leave** — on a destination collision, remove an identical legacy file silently and a divergent legacy file with a prominent warning naming it (never leave a stale legacy file; spec Q4). This is the one point it must differ from `rule-files-relocate`'s skip-and-warn.
- **No inner prompt** — runs under the bootstrap's existing batch "apply N migrations?" consent (`governance-config-rename` and `session-file-consolidate` are also silent).

`target_paths` are adopter-relative only, so audit Family 10b (no-stale-framework-target-paths) does not assert against them; Family 10a/10c (procedure-file existence, no orphan) are satisfied by the paired registry entry + procedure file. Because `target_paths` carry no `framework/`-prefixed path, govern's own generators moving to `.govern/scripts/` is a source relocation, not a registry-tracked removal.

Migration marker: `last_applied` advances only after the whole procedure completes; each of the three moves is independently idempotent, so an interrupted run re-runs the entry and converges without double-moving. A config-absent adopter no-ops on config, moves session/scripts if present, and the marker write creates `.govern/config.toml` (consistent with today creating `.govern.toml`).

### gitignore supersession

The shipped template `framework/templates/project/gitignore` changes its session entry to `/.govern/session.toml`; because `/govern` rewrites the framework-managed gitignore block on every run (`merge-managed-block`), adopters pick up the superseded entry automatically — no per-adopter migration logic needed. govern's own `.gitignore` carries the session-ignore line *outside* the managed block (anchored `/.govern.session.toml`, `.gitignore:9`); that is fixed by hand in the dogfooding step to `/.govern/session.toml`. Root-anchoring is retained deliberately so the tracked fixture session files at `runtime/tests/fixtures/*/.govern/session.toml` stay tracked (the anchored ignore matches only the repo-root file).

### Markdown surface — only the three shipped generators' literals move

Every literal naming a shipped generator moves to `.govern/scripts/`; maintainer-only literals stay at `scripts/`:

- **Shared Files manifest** (`framework/bootstrap/govern.md:635-637`) — source *and* dest become `.govern/scripts/…` (govern dogfoods, so its own copies live there and ship from the archive; `.govern/config.toml` and `.govern/scripts/` are committed, hence present in the GitHub tarball).
- **Command bodies** — `implement.md:98`, `clarify.md:102`, `amend.md:43`, `plan.md:74`, `target.md:39`, `specify.md:39,75`, `analyze.md:34,52` (the `gen-spec-deps.sh` references) → `.govern/scripts/gen-spec-deps.sh`. `analyze.md:34,227`'s existence-gated `gen-help-tables.sh` **stays** at `scripts/` (maintainer-only, govern-repo-only).
- **Adopter hook** `framework/bootstrap/hooks/govern-pre-commit:27-28` — both `gen-spec-deps.sh` and `gen-cross-service-refs.sh` → `.govern/scripts/`.
- **CI template** `framework/templates/ci/adopter-generators.yml:8,28,45,50` — the `gen-*` filter and run line → `.govern/scripts/`.
- **Permission allowlists** — `configure/claude.md:77-80`, `auggie.md:62-63`, `antigravity.md:56-60`, `opencode.md:58-61`: the `gen-*` globs move to `.govern/scripts/gen-*.sh`; `install-hooks.sh` (maintainer-only) stays at `scripts/`.
- **Constitution** — the generator-provenance notes `framework/constitution.md:421,422` (`gen-spec-deps.sh`, `gen-cross-service-refs.sh`) → `.govern/scripts/`; the `lint-rule-filenames.sh` note (`:323`) stays at `scripts/`.

### Config/session doc + canonical-source updates

Path references in prose move to `.govern/…` as a uniform token substitution (constitution mechanical-edit rule — no back-edge on the touched done specs). Sites: the constitution's canonical-sources table + `§concurrent-features` session-state paragraph (`:569`) + `[services]`/`[paths]`/`[review]` mentions; `AGENTS.md` (the session-file permission note `:48-49`, the `.govern.toml`-as-shared-DB rule `:43-47`, and the **three-site wiring** rule `:61` — updated to name `.govern/scripts/` as the shipped-generator home); `README.md` (`.govern.toml` schema mentions, `gen-cross-service-refs.sh` at `:236,247`). The pipeline command bodies that read config/session (`link`, `review`, `analyze`, `status`, `specify`, `groom`, `target`, `amend`, `implement`, `prune`, `plan`, `clarify`, `help`) update their path prose to `.govern/…`; the runtime does the actual resolution.

### Runtime write-boundary + generator-script detection

The default write-boundary seed (`interpreter/mod.rs`) and generator-script detection (`derive_boundary.rs`, `primitives/mod.rs`) recognize `.govern/scripts/**` in addition to `scripts/**`, so `/gov:implement` can write the moved generators and `run-generator` still classifies them. `run_generator.rs` itself needs no change — it resolves whatever caller-supplied path the (now-updated) command literal passes.

### Dogfooding + fixtures

govern's own repo moves to the new layout in one step: `git mv .govern.toml .govern/config.toml`; move the gitignored `.govern.session.toml` → `.govern/session.toml` and fix `.gitignore`; `git mv scripts/gen-spec-deps.sh scripts/gen-cross-service-refs.sh scripts/lib/specs-root.sh` into `.govern/scripts/`; update `.githooks/pre-commit:32-36` (the two shipped gens → `.govern/scripts/`, the three maintainer gens stay); update `.shellcheckrc` comments. For the runtime fixtures, the fallback changes the calculus: because the new-then-legacy resolver reads the existing root-layout fixtures correctly, they become valid end-to-end *fallback* coverage as-is, and relocating the fixture dirs only churns the parity stream-goldens for no correctness gain. So the fixtures stay at the legacy root layout (fallback proof), the new layout is proven end-to-end by pointing `exec_subprocess.rs` at `.govern/session.toml`, and the resolvers are exhaustively unit-tested in `paths.rs` (see task 5).

### No data model

The feature relocates files and adds path resolvers — no domain entity or data structure, so no `data-model.md` (the readiness-check data-model item is N/A). The config and session on-disk schemas are unchanged.

### Test strategy

Runtime unit tests for both resolvers (new-only, legacy-only, both-present→new, neither→default). Parity/fixture suites move to the new layout and stay green; one legacy-layout fixture proves fallback. A migration test proves converge-on-collision (identical→silent remove, divergent→warn+remove) and idempotency. Audit scripts `fixture-session-shape.sh` (`:53` `find -name .govern.session.toml`), `consolidation-pair.sh`, and `migration-coverage.sh` update to the new paths and pass. The govern-repo pre-commit generators and full `/gov:*` cycle run without path errors on the dogfooded layout.

## Affected Files

<!-- Planning aid only — /gov:implement derives the real write boundary from git. -->

| File | Action | Purpose |
| --- | --- | --- |
| `runtime/src/host.rs` | Modify | Add `config_path`/`session_path` resolvers; route host-block + cli-config-dir reads |
| `runtime/src/schema/paths.rs` | Modify | Resolve config via `config_path` for `[paths] specs-root` |
| `runtime/src/primitives/write_session.rs` | Modify | `SESSION_FILE` → `.govern/session.toml`; writer targets active file; tests |
| `runtime/src/primitives/migrate_session_file.rs` | Modify | Guards/asserts to new value (dest follows constant) |
| `runtime/src/primitives/dashboard.rs` | Modify | Session + review reads via resolvers |
| `runtime/src/primitives/resolve_references.rs` | Modify | `[services]` read via `config_path` |
| `runtime/src/primitives/discover_rule_files.rs` | Modify | `[rules]`/`[review]` read via `config_path` |
| `runtime/src/main.rs` | Modify | Exec walker seed via resolvers (replace `:362` literal) |
| `runtime/src/interpreter/mod.rs`, `primitives/derive_boundary.rs`, `primitives/mod.rs` | Modify | Recognize `.govern/scripts/**` in default boundary + generator detection |
| `runtime/tests/fixtures/*/` | Modify | Move fixtures to `.govern/{session,config}.toml`; add legacy-layout fixture |
| `runtime/tests/{parity.rs,exec_subprocess.rs,specs_root_override.rs,cross_service.rs}` | Modify | Literal path updates |
| `framework/migrations.toml` | Modify | Add `govern-dir-consolidate` entry |
| `framework/migrations/govern-dir-consolidate.md` | Create | Migration procedure (3 moves, converge, no inner prompt) |
| `framework/bootstrap/govern.md` | Modify | Manifest rows `.govern/scripts/…`; config/session path prose; write-active-file note |
| `framework/commands/{implement,clarify,amend,plan,target,specify,analyze,link,review,status,groom,prune,help}.md` | Modify | Generator literals (shipped only) + config/session path prose |
| `framework/bootstrap/hooks/govern-pre-commit` | Modify | Both shipped gens → `.govern/scripts/` |
| `framework/bootstrap/configure/{claude,auggie,antigravity,opencode}.md` | Modify | `gen-*` permission globs → `.govern/scripts/`; config/session permission paths |
| `framework/templates/ci/adopter-generators.yml` | Modify | Filter + run line → `.govern/scripts/` |
| `framework/templates/project/gitignore` | Modify | Session entry → `/.govern/session.toml` |
| `framework/constitution.md` | Modify | Canonical-sources table, session-state para, generator-provenance notes (shipped only) |
| `AGENTS.md` | Modify | Session/config path notes; three-site wiring rule names `.govern/scripts/` |
| `README.md` | Modify | `.govern.toml`/generator path mentions |
| `.govern/config.toml`, `.govern/session.toml`, `.govern/scripts/{gen-spec-deps.sh,gen-cross-service-refs.sh,lib/specs-root.sh}` | Create (git mv) | govern's own dogfooded layout |
| `.gitignore`, `.githooks/pre-commit`, `.shellcheckrc` | Modify | Dogfooded layout wiring |
| `scripts/audit/{fixture-session-shape.sh,consolidation-pair.sh,migration-coverage.sh}` | Modify | New paths in fixture/consistency audits |

## Trade-offs

- **Migration-owns-cutover vs. writes-always-target-new** — chose the migration as the sole cutover with writers on the active file. Rejected unconditional new-location writes: a runtime write before the migration would create a partial `.govern/config.toml` and new-wins would strand the legacy file's other sections — the exact stale-layout breakage this feature exists to prevent.
- **Indefinite fallback vs. time-boxed sunset** — chose indefinite (no `sunset_after`), mirroring `session-file-consolidate`. Rejected a sunset: removing the fallback would silently stop reading a straggler's config; the permanent fallback cost is one path probe.
- **Only three generators move vs. all of `scripts/`** — chose the split so `.govern/scripts/` marks adopter-facing. Rejected moving maintainer tooling: churns ~27 files and the audit/CI machinery and blurs the audience line.
- **Two file-specific resolvers vs. one generic `.govern/`-vs-root helper** — chose two named resolvers (config, session) for readable call sites and because the two files differ (config committed, session gitignored; different consumers). Minor duplication, clearer intent.
- **Reuse `migrate-session-file`'s constant vs. a new session-migration path** — chose to let the constant move so `session-file-consolidate` lands ancient adopters directly at `.govern/session.toml`; the compile-time guards keep the migration and the read/write path from drifting.
- **Known limitations** — (1) A pinned invoker (command body or hook) referencing the old `scripts/…` path is not rewritten (pinning opts out); the migration warns but the adopter owns updating their pinned copy. (2) An adopter who never runs `/govern` stays on the legacy layout indefinitely (safe via fallback; the reorg is deliberately a `/govern` event). (3) Adopter CI globs or editor scopes hardcoding root `scripts/` for govern generators must be updated by the adopter.
