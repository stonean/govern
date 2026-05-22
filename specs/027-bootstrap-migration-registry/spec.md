---
status: in-progress
dependencies: [026-framework-self-audit]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 027 — Bootstrap Migration Registry

Replace the monotonically-growing prose-encoded Pre-run Migrations section in [framework/bootstrap/govern.md](../../framework/bootstrap/govern.md) with a machine-readable registry of convention removals. Adopters record the last migration they applied; bootstrap runs only newer entries. After a per-entry sunset, migrations drop from the active registry but stay documented in an adopter-facing changelog. A new [`/audit`](../026-framework-self-audit/spec.md) family fails when a removed convention has no registry entry, so future removals can't ship without an adopter path.

## Motivation

Every convention removal `govern` has shipped — `governance` → `govern` rename, `# Governance` gitignore marker, `spec-and-plan.md` sunset, rule-file relocation, `skills/` → `workflows/` rename, post-005 workflow filename rename, `configuration.md` → `configuration-cross.md` — added a bespoke prose section to `framework/bootstrap/govern.md`. Each section scans the adopter's filesystem on **every** `/govern` re-run, forever. A fresh adopter who installed at the latest gvrn release still pays the cost of looking for legacy artifacts that have not existed in the framework for months.

Two compounding problems follow:

1. **Bootstrap work grows monotonically.** The Pre-run Migrations section can only grow because there is no sunset path. Every release that removes a convention adds another permanent scan.
2. **Future removals are unsafe.** Nothing prevents a maintainer from deleting a template, command, or filename convention without writing an adopter migration. Adopters silently break on the next pipeline command.

This spec introduces (a) a machine-readable registry of convention removals, (b) an adopter-side `last_applied_migration` field in `.govern.toml` so bootstrap can skip already-applied entries, (c) a sunset mechanism that prunes old entries from the active registry into an adopter-facing changelog, and (d) an `/audit` family that enforces the registry as the only way to ship a removal.

## Registry Shape

A new file at `framework/migrations.toml` (location and format are open questions — see below) lists every active convention removal. Each entry carries:

- **id** — stable identifier the adopter's `.govern.toml` references. Format is an open question.
- **introduced_in** — the gvrn version (or framework commit) that shipped the removal.
- **sunset_after** — the gvrn version past which this migration drops from the active registry into the adopter changelog.
- **summary** — one-line human-readable description (e.g., "spec-and-plan.md → spec.md").
- **procedure** — either an inline declarative step (rename, delete, replace-token-in-file) or a reference to a markdown procedure file under `framework/migrations/{id}.md` for entries that need user prompting or branching logic.
- **idempotency** — every entry must be a no-op when its target artifact is absent. This is an invariant, not a field.

## Adopter State

`.govern.toml` gains a single new field (section name open):

```toml
[migrations]
last_applied = "<entry-id>"
```

When the field is absent, bootstrap treats it as "no migrations have been applied" and runs every active entry. When present, bootstrap runs only entries newer than the recorded id (registry order is authoritative; alphabetical or chronological is an open question). After all applicable entries succeed, bootstrap updates the field to the id of the newest entry in the active registry.

## Bootstrap Loop

The Pre-run Migrations section in `framework/bootstrap/govern.md` is replaced by a single procedure that:

1. Reads `framework/migrations.toml` from the fetched archive.
2. Reads `.govern.toml` `[migrations].last_applied`.
3. For each registry entry newer than `last_applied`, executes the entry's procedure (declarative step or referenced markdown).
4. Updates `.govern.toml` `[migrations].last_applied` to the newest entry id.
5. Reports each applied migration in the post-scaffolding summary using the entry's `summary` field.

Entries past their `sunset_after` version are excluded from this loop entirely — they live in the adopter changelog only.

## Sunset and Adopter Changelog

When an entry's `sunset_after` version is older than the current gvrn release, the entry is removed from `framework/migrations.toml` and its text is appended to an adopter-facing changelog (location open — `CHANGELOG-FRAMEWORK.md` at the repo root, or `framework/migrations-archive.md`). Adopters who are far enough behind to need an expired migration must apply it manually using the archive entry as a recipe.

The sunset window is per-entry, not global. High-impact removals (like a directory rename) can stay active longer than low-impact ones (like a gitignore-marker rename). The sunset value is chosen when the entry is written.

## Audit Family

A new `/audit` family extends [spec 026](../026-framework-self-audit/spec.md) (`scripts/audit/migration-coverage.sh` or similar). It cross-references conventions known to have been removed against the active registry plus the archive, and fails when a removal exists without a corresponding entry. The detection mechanism is an open question — git archaeology of `framework/templates/` and `framework/commands/` deletions vs. registry id list, or a curated invariant list, or a hybrid.

This family is the gate that makes the registry load-bearing: a maintainer who removes a template without writing a migration entry breaks `/audit` and cannot ship.

## Acceptance Criteria

### Registry shape

- [ ] `framework/migrations.toml` exists with one `[[migrations]]` array-of-tables entry per active convention removal. Each entry carries the fields: `id` (slug), `introduced_in` (SemVer string), `sunset_after` (SemVer string or omitted), `summary` (one-line string), `target_paths` (array of strings), `procedure_file` (path string).
- [ ] Six back-filled entries cover every convention removal currently encoded in `framework/bootstrap/govern.md` Pre-run Migrations: `.governance.toml` rename, `# Governance` gitignore marker, `spec-and-plan.md` sunset, rule-file relocation (subsuming `configuration.md` → `configuration-cross.md`), legacy `skills/` directory, post-005 workflow filename rename.
- [ ] Each back-filled entry's `introduced_in` matches the gvrn version (or framework commit) where the removal actually shipped, derived from `git log` at registry-introduction time.
- [ ] Each back-filled entry's `sunset_after` is set to `registry_introduction_version + 2 minor versions` (uniform).
- [ ] Each back-filled entry has a corresponding `framework/migrations/{id}.md` procedure file containing the migration's prose logic (skip conditions, prompts, summary reporting). No declarative-step DSL is introduced.

### Adopter state

- [ ] `.govern.toml` schema documents a `[migrations]` section with a `last_applied` field (string, slug-valued). Absent field means "no migrations applied" — bootstrap runs every active entry.
- [ ] On `/govern` re-run against an adopter whose `last_applied` equals the newest active entry's `id`, the migration loop performs zero filesystem reads beyond loading the registry and reports zero migrations applied.
- [ ] On `/govern` re-run against an adopter whose `last_applied` is older than the newest entry, only entries newer than `last_applied` (per SemVer comparison on `introduced_in`, lexicographic tie-break on `id`) execute.
- [ ] After each migration completes successfully, `last_applied` is updated to that entry's `id` before the next entry begins. An aborted batch resumes from the next-pending entry on the following `/govern` run.

### Bootstrap loop

- [ ] `framework/bootstrap/govern.md` Pre-run Migrations section is replaced by a single procedure that iterates the registry; no per-migration prose blocks remain in `govern.md`.
- [ ] When pending migrations exist, bootstrap emits a single batch prompt listing each pending entry by `id` and `introduced_in`, then asks `Apply now? (Y/n)`. Decline emits a warning summary and makes no filesystem changes.
- [ ] The post-scaffolding summary lists each applied migration by its `summary` field. Entries with no work to do (target artifact absent) emit nothing — the procedure file's per-entry idempotency check is invariant.

### Sunset and archive

- [ ] An entry whose `sunset_after` is older than or equal to the current gvrn release is excluded from `framework/migrations.toml` (removed at sunset commit) and its procedure-file content is appended to `CHANGELOG.md` at the repo root under a heading naming the entry's `id`, `introduced_in`, and `sunset_after` version.
- [ ] When a migration is sunsetted: the registry entry, the `framework/migrations/{id}.md` file, and the `CHANGELOG.md` append all land in the same commit. `framework/migrations/{id}.md` no longer exists after the sunset commit.
- [ ] The bootstrap loop never reads `CHANGELOG.md` (verified by tracing reads in the `/govern` procedure).

### Audit (Family 10)

- [ ] A new `scripts/audit/migration-coverage.sh` (Family 10 of [026](../026-framework-self-audit/spec.md)) runs three static checks against current state:
  - [ ] **No orphan procedure files**: every `framework/migrations/{id}.md` has a matching TOML entry with `procedure_file` pointing at it. Failures: orphan files exist.
  - [ ] **No stale target paths**: every `target_paths` entry across `framework/migrations.toml` *and* archived entries in `CHANGELOG.md` refers to a path that does *not* exist in current `framework/`. Failures: a registry entry says "removed" but the path still exists.
  - [ ] **No broken procedure references**: every TOML entry's `procedure_file` field points at an existing `framework/migrations/{id}.md`. Failures: dangling references.
- [ ] Family 10 exits `0` when all three checks pass, `1` otherwise. Output format matches the other families (header + findings rows on stdout). The orchestrator `scripts/audit/run-all.sh` invokes Family 10.

### Idempotency

- [ ] Every registry entry's procedure file is idempotent: re-running it against a state where the target artifact is already migrated or absent produces no filesystem changes, no error, and emits nothing to the post-scaffolding summary.

### Edge cases and failure modes

- [ ] Empty registry (`framework/migrations.toml` exists but contains zero `[[migrations]]` entries): the bootstrap loop is a no-op, no batch prompt is emitted, no `last_applied` write occurs.
- [ ] `.govern.toml` exists but has no `[migrations]` section: treated as "no migrations applied" — bootstrap runs every active entry. Subsequent run writes the `[migrations]` section.
- [ ] `.govern.toml` `[migrations].last_applied` references an `id` that no longer exists in the active registry (sunsetted since the adopter's last run): bootstrap treats the field as "before the oldest active entry" and runs every active entry. A warning is emitted: `last_applied was "{id}" which has been retired; see CHANGELOG.md for its recipe`.
- [ ] Two TOML entries share the same `id`: bootstrap aborts at registry-parse time with a clear error. Family 10's no-orphan check would also fail on a subsequent `/audit`.
- [ ] Malformed `framework/migrations.toml` (TOML parse error): bootstrap aborts before running any migration, matching existing 022 TOML-parse-error semantics.
- [ ] `framework/migrations/{id}.md` referenced by a TOML entry does not exist in the fetched archive: bootstrap aborts the batch at that entry, reports `migration {id}: procedure file missing from archive`, and `last_applied` retains its prior value. Family 10's no-broken-references check would have caught this at maintainer time.
- [ ] User confirms the outer batch prompt but a per-entry inner prompt within a procedure file is declined: the procedure handles the decline (typically emits a warning and skips that file), the migration still "completes" from the loop's perspective, and `last_applied` advances. Behavior matches the existing prose migrations.
- [ ] Adopter's installed gvrn version is older than an entry's `introduced_in`: that should be impossible (the adopter's bootstrap is necessarily current-gvrn or newer than the registry it just fetched), but if observed (e.g., manual archive override), the entry runs anyway — `introduced_in` gates registry order, not adopter eligibility.

### Out of scope (captured for follow-on)

- "Removal must ship with a migration entry" diff-check is implemented as a separate pre-commit hook in a follow-on spec or scenario — not in this spec. Reason: it's a diff operation against HEAD, not a static-state check, and would couple `/audit` to git history shape.
- `apply-migrations` as a runtime primitive is deferred until Family 9 (`primitive-promotion-candidates.sh`) detects pattern duplication across procedure files.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Registry format** — TOML. The registry's primary content is structured metadata per entry (`id`, `introduced_in`, `sunset_after`, target paths, procedure type), which is settings, not instructions. Markdown in `framework/` is reserved for instructions (commands, constitution, bootstrap procedure, templates). Entries whose procedure cannot be expressed as a declarative step reference a separate `framework/migrations/{id}.md` file by id, preserving the split: settings in TOML, instructions in markdown.
- **Entry id format** — slug-only (e.g., `spec-and-plan-sunset`), with ordering derived from a separate `introduced_in` field and lexicographic tie-break on id. Rules out content-hash (rewording orphans adopter state), monotonic numbers (concurrent-PR conflicts), and version-prefixed ids (entries get retargeted to later releases). The slug is what the entry *is*; release/date metadata lives in dedicated fields. Idempotency invariant makes a rare slug rename safe — adopter re-runs the entry, no-op against already-migrated target.
- **Registry order** — by `introduced_in` (SemVer comparison, not lexicographic — `0.4.0 < 0.10.0`), with lexicographic tie-break on `id` for entries shipped in the same release. File order in the TOML is not authoritative because TOML parsers don't guarantee array-of-tables order preservation across rewrites. The `introduced_in` comparison matches the mental model: "shipped in 0.4.0" applies to adopters who installed at <0.4.0.
- **Procedure declaration** — every entry references a markdown procedure file at `framework/migrations/{id}.md`; no declarative step DSL. The TOML entry stays pure metadata (`id`, `introduced_in`, `sunset_after`, `summary`, `target_paths`, `procedure_file`). All migration logic — skip conditions, prompts, pinned-file exclusions, collision handling, summary reporting — lives in the markdown procedure file. Of the six existing migrations only one (gitignore marker rename) is genuinely one-step-declarative; the rest have twists that would require inventing a small DSL for one consumer. `/audit`'s migration-coverage check uses the `target_paths` metadata field, not the procedure body, so opaque procedure files are fine.
- **Sunset format** — SemVer version string, same format as `introduced_in`. An entry expires when current gvrn ≥ `sunset_after`. Omitted/null means "active indefinitely." Mirrors `introduced_in` for self-consistency; matches the mental model "active across `[introduced_in, sunset_after)`." Calendar dates decouple from release cadence; count-of-releases is just an indirect encoding of the same field. Default sunset policy is a maintainer-authoring convention documented separately, not part of this field's type.
- **Sunset archive location** — `CHANGELOG.md` at the repo root. Paths disambiguate; the runtime crate's `runtime/CHANGELOG.md` stays where Cargo expects it. Adopters who hit a removed convention with no active migration browse the repo root first; that's where the recipe-of-record belongs. Category separation: `framework/` holds in-force conventions, the repo-root `CHANGELOG.md` holds the historical record. Append-only, one file, easy to grep — no directory traversal needed. When a migration sunsets: its procedure file is appended to `CHANGELOG.md` under a heading, the procedure file is deleted, and the TOML entry is removed in the same commit. The bootstrap loop never reads `CHANGELOG.md` — it's for humans, not tooling.
- **Back-filled migration values** — hybrid: accurate `introduced_in` per git history (one-time `git log` for six entries), uniform `sunset_after = registry_introduction_version + 2 minor versions` for the back-fill. Accurate `introduced_in` makes the eventual CHANGELOG entries read correctly. Uniform sunset because we lack per-entry signal; +2 minor versions gives adopters two release cycles of grace. Spec doesn't pin the actual `registry_introduction_version` — acceptance criterion phrases it relative to whatever release ships the registry.
- **Batch behavior** — single prompt at start listing all pending migrations with `introduced_in` annotation, then run sequentially. `last_applied` updates after each migration completes (not at the very end), so an aborted batch resumes from the next run. Inner per-migration interactions (e.g., spec-and-plan's per-file Y/n) are unaffected — the outer prompt only gates "attempt the batch." Declining the outer prompt emits a warning summary and makes no filesystem changes.
- **`/audit` detection mechanism** — Family 10 runs three static current-state checks: no-orphan-procedure-files (every `framework/migrations/{id}.md` has a TOML entry), no-stale-target-paths (every registry/archive `target_paths` refers to something absent from current `framework/`), and no-broken-procedure-references (every TOML `procedure_file` points at an existing markdown file). The "removal must ship with a migration" enforcement moves to a separate pre-commit hook (out of scope for this spec — captured as follow-on) because it's fundamentally a diff operation against HEAD, not a static-state check. `/audit`'s contract stays current-state-only, matching every other family.
- **Removal target type taxonomy** — no type field; per-entry free-form is sufficient. Motivation evaporated when Q4 dropped the declarative DSL (no defaults to drive). No consumer benefits from a type enum: the bootstrap loop and Family 10 both read `target_paths` + `procedure_file`, and humans reading `CHANGELOG.md` get the human-readable framing from the `summary` string. Adding a taxonomy creates maintenance cost (categorizing each new entry, evolving the enum) without a corresponding consumer.
- **Interaction with 022 (runtime primitive vs host-side)** — host-side procedure at v1; no new primitive. Migration loop contains user prompts and per-file branching, which is host-side territory by 022's boundary. The mechanical TOML-read is the only new operation and doesn't warrant a primitive by itself. Defer promotion until Family 9 (`primitive-promotion-candidates.sh`) detects the same pattern across multiple procedure files — then promote whatever small operation duplicates (atomic-rename, semver-compare, registry-read).
