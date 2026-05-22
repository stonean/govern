# 027 — Bootstrap Migration Registry Plan

Implements [027 — Bootstrap Migration Registry](spec.md).

## Overview

The implementation consolidates two scattered sets of adopter-cleanup prose in `framework/bootstrap/govern.md` — the `## Pre-run Migrations` section (`.governance.toml`, gitignore marker, `spec-and-plan.md`, rule-file relocation) and the `## Workflow recommendation` legacy-cleanup sub-sections (`skills/` directory, workflow filename rename) — into one registry-driven loop. Six bespoke prose blocks become six TOML entries plus six small markdown procedure files plus one consolidated bootstrap step. Sunset is delivered via a `CHANGELOG.md` at the repo root and a Family 10 audit that enforces registry/state consistency.

No new runtime primitive at v1 (Q11). Per-entry idempotency is preserved as an invariant: every procedure file's first action is a target-presence check that exits silently when nothing to do.

## Technical Decisions

### Registry schema (`framework/migrations.toml`)

TOML array-of-tables, one entry per active migration:

```toml
[[migrations]]
id = "spec-and-plan-sunset"
introduced_in = "0.4.0"
sunset_after  = "0.X.0"  # registry_introduction_version + 2 minor versions; see §Back-fill values
summary       = "Rename specs/*/spec-and-plan.md → specs/*/spec.md (lightweight-track sunset)"
target_paths  = ["specs/*/spec-and-plan.md"]
procedure_file = "framework/migrations/spec-and-plan-sunset.md"
```

Field semantics:

- `id` — slug, stable, lowercase-hyphenated. The `.govern.toml` `[migrations].last_applied` references this string.
- `introduced_in` — SemVer string; back-filled per `git log` for the existing six entries.
- `sunset_after` — SemVer string or omitted. Entry expires when current gvrn ≥ `sunset_after`. Omitted means "active indefinitely."
- `summary` — one-line human-readable description used by the post-scaffolding summary line and the eventual CHANGELOG heading.
- `target_paths` — array of paths or globs (relative to adopter project root). Read by Family 10 to verify the removed convention is actually absent from current `framework/`.
- `procedure_file` — path (relative to repo root) to the markdown procedure file. Always present; there is no declarative-step DSL (Q4).

Ordering rule (Q3): by `introduced_in` SemVer ascending, lexicographic tie-break on `id`. File order in the TOML is not authoritative — TOML parsers don't guarantee array-of-tables order preservation across rewrites.

### `.govern.toml` `[migrations]` section

```toml
[migrations]
# Slug of the newest migration applied. Bootstrap runs only entries newer
# than this. Absent field means "no migrations applied" — runs every active entry.
last_applied = "rule-files-relocate"
```

Absent section is the green-field default for new adopters scaffolded after the registry ships. Their first `/govern` run applies every active entry (most of which are no-ops because their targets are absent), then writes the section with `last_applied = <newest entry's id>`.

Stale-reference behavior (edge case from spec): if `last_applied` references an id that no longer exists in the active registry (sunsetted since the adopter's last run), bootstrap treats the field as "before the oldest active entry," runs every active entry, and emits a warning naming the retired id with a pointer to `CHANGELOG.md`.

### Procedure file shape (`framework/migrations/{id}.md`)

Each file is a small markdown procedure with a fixed top-level shape. The bootstrap loop reads the file and executes its prose as the active step's body. Conventions:

```markdown
# {id}

**Introduced in:** {gvrn version}
**Summary:** {one-line summary, matches TOML `summary` field}

## Procedure

1. **Idempotency check.** {How to detect "already migrated" or "target absent" — exit silently with no summary line.}
2. {Migration step 1 — typically a prompt + action.}
3. {Migration step N.}
4. **Summary line.** Report `{summary text}` in the post-scaffolding summary.
```

Idempotency check is step 1 of every procedure file, mechanically. The bootstrap loop does not enforce idempotency itself — the procedure file owns it. Family 10 does not verify idempotency (untestable from static state); discipline + integration tests cover it.

### Bootstrap loop placement and shape

The `## Pre-run Migrations` section in `framework/bootstrap/govern.md` (lines ~190–250) and the two scattered "Legacy X cleanup" sub-sections inside `## Workflow recommendation` (lines ~570–598) collapse into one new `## Pre-run Migrations` section with this shape:

```markdown
## Pre-run Migrations

Read `framework/migrations.toml` from the fetched archive. Read `.govern.toml`'s
`[migrations].last_applied` (treat absence as null).

Filter the registry to entries where:
  - `introduced_in` is newer than `last_applied` (SemVer compare, lex tie-break on id), AND
  - current gvrn version is less than `sunset_after` (or `sunset_after` is absent).

If the filtered list is empty, emit nothing and proceed.

Otherwise, prompt once with text of the form:

      N framework migrations are pending since your last /govern run:
        - {id} (introduced {introduced_in})
        ...
      Apply now? (Y/n)

On decline: emit `warning: N migrations skipped; pipeline commands may fail on
legacy artifacts until applied. Re-run /govern to apply.` and proceed without
filesystem changes.

On confirm: for each entry in filter order:
  1. Read `framework/migrations/{id}.md` from the fetched archive.
  2. Execute its `## Procedure` steps.
  3. Update `.govern.toml` `[migrations].last_applied = "{id}"` atomically
     (tempfile + rename, matching existing `.govern.toml` write semantics).
  4. If the procedure aborts (rare — only via explicit user "stop everything"
     path inside the procedure), halt the loop. The next /govern run resumes
     from the next-pending entry.
```

The loop runs **before** the existing `## Workflow recommendation` section (since some legacy migrations affect workflows the recommendation flow then reads). It runs **after** the `## Project Configuration` section (so `.govern.toml` is already loaded).

The two legacy-cleanup sub-sections inside `## Workflow recommendation` (lines 570 and 586) are deleted entirely — their work is now done by the registry-driven loop earlier in the procedure. The `enforce-manifest` invocation at line 36 keeps its other duties (slash-command manifest enforcement) but loses its legacy-cleanup roles (those move to the registry).

### SemVer comparison (no new primitive)

SemVer compare is a small shell helper, not a runtime primitive. Two options the procedure file can use:

- `sort -V` on a two-line input (BSD/GNU coreutils both support `-V`).
- A 10-line Python one-liner via `python3 -c 'from packaging.version ...'` if `sort -V` proves insufficient for pre-release tags.

The bootstrap loop's filter operation is the only consumer of SemVer compare; it lives in the consolidated `## Pre-run Migrations` prose, not in a script. The procedure files themselves never compare versions.

If, after implementation, a SemVer helper proves common across multiple procedure files or scripts, Family 9 of `/audit` (`primitive-promotion-candidates.sh`) will surface it and we promote then.

### Sunset commit shape

When a maintainer sunsets a migration (current gvrn ≥ entry's `sunset_after`), the work is one commit:

1. Delete the `[[migrations]]` entry from `framework/migrations.toml`.
2. Append the procedure file's content to `CHANGELOG.md` at the repo root under:

   ```markdown
   ## {id} — sunset {gvrn version}

   Introduced in gvrn {introduced_in}; sunset after gvrn {sunset_after}.
   Adopters who skipped past this window must apply manually.

   {full content of framework/migrations/{id}.md}
   ```

3. Delete `framework/migrations/{id}.md`.
4. `/audit`'s Family 10 verifies the consistency of steps 1+3 (no orphan procedure files, no broken refs); it does not verify the CHANGELOG append (no static check can — discipline + code review).

`CHANGELOG.md` is created in this spec's implementation with a seed entry describing the registry's introduction (no historical entries yet — back-filled migrations are still active).

### Family 10 design (`scripts/audit/migration-coverage.sh`)

Shell script under the existing `scripts/audit/` directory. Follows the same contract as Families 1–9: stdout findings, exit 0/1, read-only. Three checks:

```bash
# 10a no-orphan-procedure-files
for f in framework/migrations/*.md; do
  id=$(basename "$f" .md)
  grep -q "^id *= *\"$id\"" framework/migrations.toml || \
    echo "Family 10 | $f | orphan procedure file (no TOML entry) | add a [[migrations]] entry or delete the file"
done

# 10b no-stale-target-paths
# Parse target_paths from migrations.toml AND from CHANGELOG.md archived entries.
# For each path/glob, verify it does NOT exist in current framework/.
# (Glob expansion via find; literal paths via test -e.)

# 10c no-broken-procedure-references
# Parse procedure_file from each migrations.toml entry.
# Verify each path exists.
```

CHANGELOG.md parsing for 10b: archived entries have a recognizable heading (`## {id} — sunset {version}`) followed by a TOML-fenced block (or the raw procedure content). The script grep's for the `target_paths` line within archived sections. If the format changes, the script needs updating — captured as a future Family 10 brittleness risk.

Wiring: append one line to `scripts/audit/run-all.sh` after the Family 9 invocation:

```bash
run_check "Family 10 — migration coverage" "scripts/audit/migration-coverage.sh"
```

And append a numbered step to `framework/commands/audit.md`'s Markdown-only reference (step 11), matching the existing pattern. No new MCP primitive — `run-all.sh` is the orchestrator; `/audit` itself invokes it via `run-generator`.

### CHANGELOG.md seed format

Top of file:

```markdown
# CHANGELOG

Adopter-facing record of framework conventions that have been removed.
Each entry is a recipe for adopters who skipped past the active sunset
window and need to apply the removal manually. See spec
[027 — Bootstrap Migration Registry](specs/027-bootstrap-migration-registry/spec.md)
for the governing pipeline.

The runtime crate's own release notes live at `runtime/CHANGELOG.md`.

## Archived migrations

*None yet — registry introduced in gvrn {registry_introduction_version}.*
```

Future maintainers append archived migrations under `## Archived migrations` as they sunset, with most-recent at the top.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/migrations.toml` | Create | Registry; back-filled with six entries |
| `framework/migrations/spec-and-plan-sunset.md` | Create | Procedure for lightweight-track sunset |
| `framework/migrations/skills-to-workflows.md` | Create | Procedure for `skills/` directory removal |
| `framework/migrations/workflow-filename-rename.md` | Create | Procedure for post-005 workflow filename cleanup |
| `framework/migrations/rule-files-relocate.md` | Create | Procedure for rule-file relocation (subsumes `configuration.md` rename) |
| `framework/migrations/governance-config-rename.md` | Create | Procedure for `.governance.toml` → `.govern.toml` |
| `framework/migrations/gitignore-marker-rename.md` | Create | Procedure for `# Governance` → `# govern` gitignore marker |
| `framework/bootstrap/govern.md` | Modify | Replace `## Pre-run Migrations` section; delete two legacy-cleanup sub-sections inside `## Workflow recommendation`; add `.govern.toml` `[migrations]` section to the §Project Configuration schema |
| `scripts/audit/migration-coverage.sh` | Create | Family 10 — three static checks |
| `scripts/audit/run-all.sh` | Modify | Append Family 10 invocation |
| `framework/commands/audit.md` | Modify | Append step for Family 10 in the Markdown-only reference |
| `CHANGELOG.md` | Create | Adopter-facing archive (seed content; no entries yet) |
| `.claude/commands/gov/init.md` | Generate | Regenerated by `scripts/gen-claude-commands.sh` (pre-commit hook) if it mirrors govern.md content |
| `specs/026-framework-self-audit/scenarios/family-10-migration-coverage.md` | Create | Cross-spec impact: 026 is `done`; Family 10 extension lands as a scenario with back-link to 027 |
| `specs/027-bootstrap-migration-registry/spec.md` | Modify | Status `clarified → planned` at the end of this phase, then `→ in-progress` and `→ done` during implementation |

The runtime write boundary that `/gov:implement` derives from git history will include the above plus incidental files (CHANGELOG entries on this spec's commits, README spec-status table regeneration via the pre-commit hook).

## Trade-offs

### Considered: declarative-step DSL for simple migrations (rejected)

Q4. A small DSL of `rename` / `delete` / `regex-replace` step types could express the gitignore-marker migration in two TOML lines. Rejected because five of six existing migrations have twists (skip-if-both-exist, per-file Y/n, pinned-files exclusion, batch prompts, collision handling) that would require extending the DSL until it became a programming language. Markdown procedure files are honest about being prose; the TOML stays pure metadata.

### Considered: `apply-migrations` runtime primitive (deferred)

Q11. A primitive that reads the registry, computes the filter, and dispatches procedure files could replace the bootstrap loop's prose. Deferred to a follow-on once Family 9 (`primitive-promotion-candidates.sh`) detects pattern duplication that justifies the maintenance overhead. The migration loop contains user prompts and per-file branching, which is host-side territory by 022's existing boundary — a primitive would carry user-interaction protocol design that doesn't exist elsewhere in the runtime.

### Considered: git archaeology inside Family 10 (rejected)

Q9. A check that walks `git log -- framework/templates/ framework/commands/` to detect deletions without a corresponding registry entry would catch "removal without migration" mistakes automatically. Rejected because (a) it couples `/audit` to commit history shape (rebases, squashes lose signal), (b) it breaks the "current-state only" contract every other family follows, and (c) it requires distinguishing "convention removal" from "internal refactor that renamed a file" — open-ended classification. The cleaner alternative is a separate pre-commit hook (captured as out-of-scope follow-on) that compares PR diffs against registry changes.

### Considered: unifying `enforce-manifest`'s legacy-cleanup work with the registry (accepted)

The existing `enforce-manifest` invocation in govern.md line 36 already removes the legacy `skills/` directory and legacy workflow filenames as a side effect. Leaving those two migrations out of the registry (since `enforce-manifest` handles them) would be coherent but creates two places to look for "what does bootstrap clean up on every run?" Decision: register both anyway, and modify `enforce-manifest`'s contract so it no longer handles legacy-cleanup work (just slash-command manifest enforcement). This keeps the registry as the *one* source of truth for adopter-cleanup, at the cost of trimming `enforce-manifest`. The trim is a small refactor of the primitive's expected-list construction — no behavior change for the slash-command-manifest path.

## Known Limitations

### Back-fill sunset values are uniform, not per-entry-tuned

All six back-filled entries get the same `sunset_after = registry_introduction_version + 2 minor versions`. We don't have per-entry signal on how long each migration needs to stay active. Maintainers can tune individual entries later by editing their `sunset_after` value. The cost is that low-impact migrations (gitignore marker) and high-impact ones (rule-file relocation) sunset together; in practice the manual recipe in `CHANGELOG.md` covers either case.

### "Removal must ship with migration" is discipline-only at v1

Family 10 verifies registry/state consistency, not registry/diff coverage. A maintainer who deletes `framework/templates/foo.md` without adding a `[[migrations]]` entry breaks adopters silently — Family 10 only catches the divergence after the registry diverges from filesystem state, not at commit time. Captured as a follow-on pre-commit hook spec/scenario.

### CHANGELOG.md append at sunset is maintainer-discipline

When a migration sunsets, three actions land in one commit: delete TOML entry, append CHANGELOG entry, delete procedure file. Family 10 catches two of three (no-orphan, no-broken-ref) but cannot verify the CHANGELOG actually received the content. A maintainer who deletes the TOML entry and procedure file without appending to CHANGELOG breaks the "adopters skipping past sunset have a recipe" promise. Code review is the gate; a stronger automated check could parse the previous registry's sunsetted entry against CHANGELOG diff in CI — captured as a future enhancement.

### Procedure file content format is loosely specified

The procedure file shape above is a convention, not a parsed schema. A maintainer who writes a procedure file without a "Summary line" step would silently break the post-scaffolding summary contract. Acceptable for v1 because (a) the existing migration prose is the back-fill source, (b) six entries are small enough that code review catches deviations, (c) the runtime never parses procedure files (the bootstrap loop reads and dispatches; the LLM/host executes them). If procedure files proliferate, a parseable shape (matching 022's parseable command convention) is a natural promotion path.
