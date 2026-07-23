---
status: in-progress
dependencies: [027-bootstrap-migration-registry, 040-configurable-specs-dir]
review:
  last-run: 2026-07-23T00:01:01Z
  reviewed-against: 3d135ca0da936dfbb8b3c78014a5f296638c8769
  must-violations: 0
  should-violations: 1
  low-confidence: 1
  blocking: false
---

# 042 — Consolidate govern per-project files under a `.govern/` directory

govern scatters its per-project files across the adopter's repo root: `.govern.toml` (committed config), `.govern.session.toml` (gitignored session state), and the adopter-facing generator scripts that land in the project's root `scripts/`. This feature consolidates all three under a single `.govern/` directory — `.govern/config.toml`, `.govern/session.toml`, and `.govern/scripts/` — so govern owns one predictable namespace instead of polluting the repo root and colliding with a project's own `scripts/`. `/govern` detects a project still on the old layout and reorganizes it through a new adopter migration; the runtime reads the new locations with a fallback to the legacy root paths so an adopter who upgrades gvrn before re-running `/govern` is never broken.

## Motivation

Every govern adopter gets three intrusions into their repo root:

- **`.govern.toml`** at the repo root — the committed config and persisted-decisions store (`[project]`, `[paths]`, `[rules]`, `[pinned]`, `[migrations]`, `[workflows]`, `[review]`, `[services]`).
- **`.govern.session.toml`** at the repo root — the gitignored, per-contributor session target.
- **Three scripts scaffolded into the adopter's root `scripts/`** — `gen-spec-deps.sh`, `gen-cross-service-refs.sh`, and `lib/specs-root.sh`, installed via the Shared Files manifest with `update` strategy.

The scripts intrusion is the sharpest: a well-organized project frequently already owns a `scripts/` directory with its own conventions, and dropping govern's generators into it creates naming-collision risk and mixes framework-managed files (rewritten on every `/govern` run) with the project's own. The two config files add two more govern-specific dotfiles to a root that adopters would rather keep clean. Consolidating under `.govern/` gives govern one directory to own, keeps framework-managed files clearly separated from the project's own, and makes the govern footprint greppable and removable as a unit.

This mirrors the single-namespace convention adopters already expect from tools like `.git/`, `.github/`, and `.vscode/`: one top-level directory holds the tool's project-scoped state.

## Target layout

```text
.govern/
├── config.toml     # committed — replaces root .govern.toml
├── session.toml    # gitignored — replaces root .govern.session.toml
└── scripts/        # committed — replaces the adopter-facing scripts in root scripts/
    ├── gen-spec-deps.sh
    ├── gen-cross-service-refs.sh
    └── lib/
        └── specs-root.sh
```

- **`.govern/config.toml`** carries every section `.govern.toml` carries today, unchanged in schema. Only its location moves.
- **`.govern/session.toml`** carries the same session-target and per-contributor `cli-config-dir` content as `.govern.session.toml` today. It stays gitignored.
- **`.govern/scripts/`** holds only the adopter-facing generators (the three files above). It is committed, and the framework-managed files under it are still rewritten on every `/govern` run and still honor `[pinned] files`.

The `.govern/` directory itself is committed; only `.govern/session.toml` is gitignored. The gitignore entry names the session file specifically (`/.govern/session.toml`), never the whole directory, so the committed config and scripts are tracked.

## Config file: `.govern/config.toml`

The config file's schema, all its sections, and every behavior it drives are unchanged — this feature moves the file, it does not touch what the file means. Every reader and writer resolves the config path to `.govern/config.toml`, falling back to the legacy root `.govern.toml` when the new file is absent (see [Transition and fallback](#transition-and-fallback)).

- All pipeline commands that read config (`link`, `review`, `analyze`, `status`, `specify`, `groom`) and the `/govern` bootstrap resolve the new location.
- The runtime's config readers (host block, `[paths] specs-root`, `[services]`, `[review]`, `[rules]`) resolve the new location, consistent with the markdown-only path — the same runtime/markdown-agreement contract [040-configurable-specs-dir](../040-configurable-specs-dir/spec.md) established for reading `[paths] specs-root` out of the config file.
- The config file is both an input to and a target of the govern-directory migration: `[migrations].last_applied` lives inside it, yet the migration moves the file. The bootstrap resolves the config location once (new-or-legacy) for the whole run, so reading `last_applied` at the start and writing it back after each migration both target the correct location, and the govern-directory migration's own `last_applied` write lands in `.govern/config.toml`.

## Session file: `.govern/session.toml`

The session file's content and single-target semantics ([constitution §concurrent-features](../../framework/constitution.md)) are unchanged; only its location moves. Every reader and writer resolves `.govern/session.toml`, falling back to the legacy root `.govern.session.toml` when the new file is absent.

- The runtime holds the session path as a single constant with compile-time guards; the constant and its guards move to the new path. The `write-session` and `dashboard` primitives, the exec walker seed, and the host `cli-config-dir` reader all resolve the new location.
- The pipeline commands that read the session target (`target`, `amend`, `implement`, `prune`, `plan`, `clarify`, `analyze`, `groom`, `specify`, `status`, `help`) and those that write it resolve the new location.
- The session-file gitignore entry moves from `.govern.session.toml` to `/.govern/session.toml` in both the shipped gitignore template and govern's own `.gitignore`.
- This feature composes with the earlier [session-file-consolidate](../027-bootstrap-migration-registry/spec.md) migration: an adopter far enough behind runs that migration (legacy per-agent JSON → root TOML) and then this one (root TOML → `.govern/session.toml`) in order, each idempotent.

## Scripts: `.govern/scripts/`

Only the three adopter-facing generators move; govern's own maintainer-only tooling (audit families, linters, maintainer generators) stays in the framework repo's root `scripts/` and is never shipped to adopters.

- The Shared Files manifest ships the three generators to `.govern/scripts/` (same relative sub-layout, `lib/specs-root.sh` preserved). The `update` strategy and `[pinned] files` opt-out are unchanged.
- Every verbatim `scripts/…` string literal that names one of the three **shipped** generators resolves to `.govern/scripts/…`: the command bodies that invoke `gen-spec-deps.sh` via `run-generator` (`implement`, `clarify`, `amend`, `plan`, `target`, `specify`, `analyze`), the adopter pre-commit hook (`govern-pre-commit`, which runs `gen-spec-deps.sh` and `gen-cross-service-refs.sh`), the adopter CI generator template, the constitution's generator-provenance notes, and the per-agent permission allowlists for all four agents. References to **maintainer-only** scripts that stay at root `scripts/` are left unchanged — notably `analyze`'s existence-gated `gen-help-tables.sh` (govern-repo-only, never shipped) and the constitution's `lint-rule-filenames.sh` provenance note.
- The runtime's default write-boundary seed and generator-script detection recognize the new `.govern/scripts/` location.
- Because govern dogfoods its own commands, the framework repo's own copies of the three generators (and `lib/`) move to `.govern/scripts/` so the shipped `scripts/…` string resolves in govern's own tree as well as in adopters'. Govern's maintainer-only scripts stay at the root `scripts/`.
- The generators source their shared lib self-relatively (`$(dirname "$0")/lib/…`) and the shellcheck config resolves sources self-relatively, so a wholesale move of the generator set with its `lib/` keeps sourcing intact.

## `/govern` migration: reorganize a project on the old layout

`/govern` detects a project still on the old layout and reorganizes it, driven by a new entry in the [bootstrap migration registry](../027-bootstrap-migration-registry/spec.md). The migration is idempotent (a no-op when no legacy files are present) and moves all three concerns in one pass:

- Move `.govern.toml` → `.govern/config.toml`.
- Move `.govern.session.toml` → `.govern/session.toml`.
- Move the adopter-facing generators from root `scripts/` to `.govern/scripts/`.

Moves preserve git history (`git mv` when the source is tracked, `mv` otherwise) and preserve any adopter customization of the three generators. The move runs under the bootstrap's existing batch migration consent (the `§Pre-run Migrations` "apply N migrations?" gate) with **no additional per-file prompt** — `governance-config-rename` and `session-file-consolidate` also move silently, and the bootstrap's `§Procedural-fidelity` posture discourages extra prompts on routine runs; `rule-files-relocate`'s inner prompt existed only because a rule file at the `specs/` root is a genuinely ambiguous choice, whereas govern unambiguously owns these files.

Because the new location wins on read (see [Transition and fallback](#transition-and-fallback)), the migration **converges** rather than skip-and-leaves — this is the one point where it must differ from `rule-files-relocate`. When a destination already exists, a legacy file identical to the new one is removed silently; a legacy file that diverges from the new one is removed with a prominent warning naming it (the divergent legacy content was already inert under new-wins, and the git-tracked config makes the removal recoverable). A stale legacy file is never left in place. A pinned generator moves together with its pin, so a customized script stays both discoverable at the new path and protected from overwrite. The migration emits a single summary line naming what moved (omitted when nothing moved), and is registered in `framework/migrations.toml` with its procedure body at `framework/migrations/{id}.md`, satisfying the audit's migration-coverage invariants ([bootstrap migration registry](../027-bootstrap-migration-registry/spec.md), Family 10).

## Transition and fallback

The migration only fires during `/govern`, but the runtime primitives (`/gov:status`, `/gov:review`, and every other command that reads config or session state) run independently. An adopter who upgrades gvrn and runs one of those commands *before* re-running `/govern` must not break. Therefore, every config and session reader resolves the new `.govern/` location first and falls back to the legacy root path when the new file is absent. The new location always wins when both exist.

The `/govern` migration is the **sole cutover**: until it runs, writers target the same active file readers resolve (the `.govern/` file when it exists, otherwise the legacy file). This matters for the config file specifically — a runtime write that named the new location unconditionally (e.g. `/gov:review` writing `[review]` before the migration) would create a *partial* `.govern/config.toml` holding only that section, and new-wins-on-read would then strand the legacy file's other sections (`[pinned]`, `[services]`, `[project]`, …). Writing the active file instead keeps every section together until the migration moves the whole file as one unit. A fresh adopter with no legacy file writes directly to `.govern/`.

The fallback is **indefinite** — it is never removed, and the migration carries no `sunset_after`, mirroring the high-impact-directory-reorg precedent set by the `session-file-consolidate` migration. This guarantees an adopter who upgrades gvrn but never re-runs `/govern` is never orphaned.

The fallback is a **bridge, not a resting state**. A stale layout — a lingering legacy file after the new location has taken over writes — must be actively corrected, because a divergent legacy file that reads ignore is exactly what breaks functionality. Two mechanisms enforce convergence:

- **New-wins on read**, so a split state (both the legacy root file and the new `.govern/` file present) never reads stale content — the legacy file is dead the moment the new one exists.
- **The migration always converges.** Because it never sunsets, every `/govern` run on a project still holding a legacy file completes the move to `.govern/`, so a split or stale layout is always resolved on the next run rather than left to rot. When the new-location destination already exists (a prior write created it), the migration resolves the split rather than silently leaving the divergent legacy file in place (exact collision handling is specified in the `/govern` migration section below).

## Documentation and canonical sources

The [constitution §drift-prevention](../../framework/constitution.md) canonical-sources table, the constitution's session-state and generator-provenance prose, `AGENTS.md`, and `README.md` reference these files by their current root paths and are updated to the `.govern/` paths. The change is a location rename across documentation, not a schema change; the canonical home of each fact is unchanged.

## Edge cases and transition hazards

- **`.gitignore` supersession.** Moving the session file is not enough — the stale `/.govern.session.toml` (or unanchored `.govern.session.toml`) ignore line must be superseded by `/.govern/session.toml`, or the moved session file is no longer ignored and gets accidentally committed. Govern's own repo carries the session-ignore line *outside* the managed block (hand-maintained, anchored `/.govern.session.toml`); adopters carry it *inside* the shipped gitignore managed block. Both forms are superseded so the session file stays ignored and no dangling ignore line remains.
- **Pinned invoker referencing the old script path.** The three generators' invokers (command bodies, the `govern-pre-commit` hook) are normally rewritten to `.govern/scripts/…` by scaffolding, but an adopter who has pinned an invoker keeps its old `scripts/…` reference. Since the scripts move, a pinned invoker would call a now-missing path. Pinning opts out of updates, so the migration does **not** rewrite pinned files; instead it surfaces a warning naming any pinned invoker that still references an old `scripts/…` generator path, so the breakage is visible and the adopter updates their pinned copy — never a silent failure.
- **Interrupted or partial migration.** `[migrations].last_applied` advances only after the whole govern-directory procedure completes, and each of the three moves is independently idempotent, so an interrupted run re-runs the whole entry on the next `/govern` and converges without double-moving.
- **Config-absent adopter.** Many adopters run without a config file. The migration no-ops on the absent config and still moves session/scripts if present; config readers fall through new-then-legacy to default behavior. Recording `[migrations].last_applied` after the run creates `.govern/config.toml` at the new location if it did not exist — consistent with today's behavior of creating `.govern.toml` to hold the marker.

## Acceptance Criteria

- [x] A freshly adopted project (first `/govern` run) has `.govern/config.toml`, a gitignored `.govern/session.toml`, and the three adopter-facing generators under `.govern/scripts/`; no `.govern.toml`, `.govern.session.toml`, or govern generators land at the repo root or in a root `scripts/`.
- [x] The `.govern/config.toml` schema is byte-for-byte the same set of sections and keys as the former root `.govern.toml`; only the file location changed.
- [x] `.govern/session.toml` carries the same session-target and `cli-config-dir` content as the former `.govern.session.toml`, and is gitignored via a `/.govern/session.toml` entry that leaves `.govern/config.toml` and `.govern/scripts/` tracked.
- [x] Running `/govern` in a project on the old layout moves `.govern.toml` → `.govern/config.toml`, `.govern.session.toml` → `.govern/session.toml`, and the three generators from root `scripts/` → `.govern/scripts/`, preserving git history for tracked sources, and is a no-op on a project already on the new layout.
- [x] The migration runs under the bootstrap's existing batch migration consent with no additional per-file prompt, and preserves adopter customization of the three generators (a pinned generator moves together with its pin).
- [x] On a destination collision the migration converges rather than skip-and-leaves: an identical legacy file is removed silently, a divergent legacy file is removed with a prominent warning naming it, no stale legacy file is left in place, and a single summary line names what moved (omitted when nothing moved).
- [x] The migration is registered in `framework/migrations.toml` with a procedure file at `framework/migrations/{id}.md`, and the audit's migration-coverage invariants pass (no orphan procedure file, no broken procedure reference, no stale framework-prefixed target path).
- [x] Every runtime primitive and pipeline command that reads config or session state resolves `.govern/config.toml` / `.govern/session.toml`, falling back to the legacy root path when the new file is absent, with the new location winning when both exist.
- [x] An adopter who upgrades gvrn and runs `/gov:status` (or any read-only pipeline command) *before* re-running `/govern` sees correct output with no path error, sourced from the legacy root files via fallback.
- [x] The fallback and the migration are both indefinite (no `sunset_after`); a split layout (a legacy root file present alongside its `.govern/` counterpart) reads only the `.govern/` file, and the next `/govern` run converges it to the new layout rather than leaving the divergent legacy file in place.
- [x] Config and session writes target the active file (the `.govern/` file when it exists, else the legacy file, defaulting to `.govern/` when neither exists), so no runtime write outside the migration ever creates a partial `.govern/config.toml` that strands other config sections; the `/govern` migration is the sole cutover. The bootstrap resolves the config location once per run so `[migrations].last_applied` read and write-back agree even though the config file is itself a migration target.
- [x] Every verbatim `scripts/…` reference to one of the three shipped generators (command bodies, adopter pre-commit hook, adopter CI template, constitution generator-provenance notes, all four agents' permission allowlists) resolves to `.govern/scripts/…`, and the shipped generators still source `lib/specs-root.sh` correctly from the new location.
- [x] After the migration runs, `.govern/session.toml` is gitignored and no dangling `.govern.session.toml` ignore line remains — both the shipped managed-block form and govern's own out-of-block anchored form are superseded by `/.govern/session.toml`.
- [x] Moving the generators does not silently break a pinned invoker: a pinned command body or `govern-pre-commit` hook still referencing an old `scripts/…` generator path is left unmodified (pinning opts out of updates) but is named in a migration warning.
- [x] govern's own repository dogfoods the new layout: its config, session, and the three adopter-facing generators live under `.govern/`, its maintainer-only scripts remain at root `scripts/`, and its own pipeline (`/gov:*`) and pre-commit generators run without path errors.
- [x] The constitution's canonical-sources table and session-state / generator-provenance prose, `AGENTS.md`, and `README.md` reference the `.govern/` paths; no stale root-path reference to the moved files remains in framework documentation.

## Open Questions

*None — all resolved. See Resolved Questions below.*

## Resolved Questions

- **Interaction with `session-file-consolidate`'s runtime constant.** Resolved (grounded in the runtime source): the two migrations compose cleanly with no root-path staging — the `SESSION_FILE` constant (`runtime/src/primitives/write_session.rs:39`) moves directly to `.govern/session.toml`. Because `migrate-session-file` writes its destination as that constant (`runtime/src/primitives/migrate_session_file.rs:25,49,66,96`), `session-file-consolidate` migrates an ancient adopter's per-agent JSON **directly** to `.govern/session.toml`, never staging through the root path; the govern-directory migration then no-ops on the session file (no root `.govern.session.toml` was written). A mid-era adopter (root `.govern.session.toml` present, no JSON) sees `session-file-consolidate` no-op and the govern-directory migration move root → `.govern/session.toml`. Ordering (`0.10.0` before `0.22.0`) and idempotency hold; the Q4 converge-on-collision rule covers the rare both-present state. Required runtime edits: move the constant's value and update its two compile-time guards plus the literal-asserting tests (`migrate_session_file.rs:189,345`; `write_session.rs:361,429`; `host.rs:262,275,292`) and the tracked session fixtures.

- **Migration confirmation prompt and collision handling.** Resolved on both sub-points. **(a) No inner prompt** — the move runs under the bootstrap's existing batch "apply N migrations?" consent, matching `governance-config-rename` and `session-file-consolidate` and the `§Procedural-fidelity` posture against extra routine prompts; `rule-files-relocate`'s inner prompt existed only because a rule file at the `specs/` root is an ambiguous choice, which does not apply here. **(b) Converge, don't skip-and-leave** — this is the one point the migration must differ from `rule-files-relocate`: because new-location-wins makes a lingering legacy file dead, on a destination collision the migration removes an identical legacy file silently and removes a divergent legacy file with a prominent warning naming it (the divergent content was already inert under new-wins; git-tracked config makes removal recoverable), never leaving a stale legacy file in place. Accepted trade-off: the divergent case deletes a legacy file a user may have hand-edited after cutover — acceptable because the edit was already being ignored, the file is recoverable from git, and the warning surfaces it; rejected the safer "keep the divergent file with a warning" because it leaves the stale layout the fallback resolution ruled out.
- **Scope of the framework's own `scripts/` move.** Resolved: only the three adopter-facing generators (plus `lib/specs-root.sh`) move into govern's own `.govern/scripts/`; the ~27 maintainer-only scripts (audit families, linters, maintainer generators, CI infra) stay at root `scripts/`. `.govern/` is the footprint an adopter gets, and the three shipped generators must live there so the verbatim `scripts/…` strings in the shipped command bodies resolve in govern's own tree. A useful consequence: the `.govern/scripts/` location becomes the **structural marker of an adopter-facing script** — the audience is encoded by where the file lives, not only by the Shared Files manifest, so a contributor adding a generator immediately sees whether it ships. `AGENTS.md`'s "three-site wiring" rule is updated to name `.govern/scripts/` as the shipped-generator home. Rejected moving all of `scripts/`: it would churn ~27 maintainer files plus the audit/CI machinery and blur the adopter-vs-maintainer line this split makes crisp.
- **`introduced_in` version.** Resolved: the migration and its runtime changes ship under `introduced_in = "0.22.0"` (the next minor cut from the current `0.21.1`). A minor bump matches the change's weight — a new migration, a moved runtime session-path constant, config-path resolution, and new fallback behavior — where a patch would undersell it. With the fallback resolved as indefinite (no `sunset_after`), the version sets only `introduced_in`.
- **Fallback lifetime and sunset.** Resolved: the runtime fallback is **indefinite** and the migration carries **no `sunset_after`** — both stay active forever, mirroring the `session-file-consolidate` high-impact-directory-reorg precedent (which also omits `sunset_after`). The fallback's per-read cost is negligible (one extra path probe only when the new file is absent) and indefinite retention guarantees an adopter who upgrades gvrn but never re-runs `/govern` is never orphaned. Crucially, the fallback is a **bridge, not a resting state**: a stale layout (a lingering legacy file after writes have moved to `.govern/`) will break functionality if left, so convergence is mandatory — new-location-wins on read makes a split state read non-stale immediately, and because the migration never sunsets, every `/govern` run converges a project still holding a legacy file to the new layout rather than leaving the divergent file to rot. Rejected option (b) (time-boxed `sunset_after` + eventual fallback removal): it trades a real "silently stops reading your config" failure mode for stragglers against a negligible amount of permanent fallback code.
