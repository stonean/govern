---
spec: 041-task-pruning
reviewed-at: 2026-07-11T12:12:24Z
reviewed-against: c0bc8697bdef33bbb2024585fa75bd4299889fca
diff-base: 9ab3163db47064584fd29ef4a7eb041865be3767
must-violations: 0
should-violations: 1
low-confidence: 2
captured-issues: 0
skipped-passes: []
---

# Review — 041-task-pruning

## Summary

Clean review. The implementation — the `prune-tasks` runtime primitive plus the
`/gov:prune` command, the shared `SkipScanner` parser fix, and the framework
consistency edits — carries **no MUST violations** and is not blocking. Rule
files applied: the backend + cross set (`security-backend`, `api-backend`,
`concurrency-backend`, `observability-backend`, `performance-backend`,
`reliability-backend`, `configuration-cross`, `quality-cross`); the frontend
rule files were not selected (no frontend surface in scope). One advisory
SHOULD (a reuse duplication) and two low-confidence notes are recorded below;
none blocks `done`. The code is covered by 489 passing library tests plus the
integration suites (parity, MCP), with `clippy -D warnings` and `fmt` clean, and
`scripts/audit/run-all.sh` reporting zero findings.

Security posture: the backend security rules govern authentication, credentials,
sessions, tokens, and JWTs — none of which this feature introduces. `prune-tasks`
performs a local, confirmed rewrite of a single `tasks.md` within the resolved
feature directory; it opens no network, handles no secrets, and persists no
credentials. No security finding.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

### SHOULD: QUAL-REUSE — numeric-heading helpers triplicated across tasks parsers

- **File**: `runtime/src/primitives/prune_tasks.rs:~370` (`heading_is_numeric`, `split_numbered_heading`)
- **Rule**: quality-cross — prefer extracting logic duplicated across modules into shared code rather than re-implementing it.
- **Finding**: `prune_tasks.rs` defines local `heading_is_numeric` and `split_numbered_heading` helpers that duplicate `read_tasks.rs`'s module-local `split_numbered_heading`/`heading_starts_with_number` and `mod.rs`'s `heading_starts_with_number`. The numeric-heading check now exists in three modules. Note this follows an existing, deliberate convention — `read_tasks.rs` documents keeping its copy "module-local to avoid widening the crate-internal surface" — so this is consistent with the codebase, not a regression.
- **Auto-fixable**: no
- **Suggested fix**: optionally promote a single `pub(crate) fn split_numbered_heading` / `heading_is_numeric` to `primitives::mod` and have `read_tasks`, `prune_tasks` (and the `mod.rs` copy) call it. Advisory — defer if the module-local convention is preferred.

## Low-confidence findings

### quality — keep-pending rewrites a file whose only reducible content is an empty phase container

- **File**: `runtime/src/primitives/prune_tasks.rs` (`reduce_keep_pending`, `dropped_any`)
- **Finding** (confidence ~55): in phased mode, a `## Phase …` container with zero (or only spent) task sections is dropped, which sets `dropped_any` and therefore writes even when `removed_count == 0`. A user running `/gov:prune` on a file whose only "prunable" element is an empty phase heading gets a write rather than a "nothing to prune" report. This matches the documented data-model behavior ("drop a phase container with no surviving task section") and is an unusual hand-edited state, so it is recorded as low-confidence, not a finding to fix.

### security (defense-in-depth) — `feature` arg is not run through `validate_no_traversal`

- **File**: `runtime/src/primitives/prune_tasks.rs:110-118`
- **Finding** (confidence ~40): `run` builds `repo.join(&root).join(&args.feature)` and gates on `is_dir()` without calling `validate_no_traversal(&args.feature)`. This is identical to every sibling feature-name primitive (`read-tasks`, `mark-task`, `set-status`, `check-stuck`, `derive-boundary`), where `feature` is a host-resolved directory slug, not free caller input; the traversal guard is reserved for caller-supplied _path_ arguments (`feature-path`, `slug`) in `append-task`/`create-scenario`. Flagging prune-tasks alone would be inconsistent with the established, previously-reviewed convention. Recorded as a low-confidence, codebase-wide defense-in-depth observation, not a 041 regression.

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no additions to `specs/inbox.md` in the review window._

## Skipped passes

_None — all five passes (security, reuse, quality, efficiency, simplicity) ran._
