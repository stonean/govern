---
spec: 022-deterministic-runtime
scenario: review-exec-wiring
reviewed-at: 2026-07-11T01:10:55Z
reviewed-against: 2733d9d
diff-base: a525beb
must-violations: 0
should-violations: 0
low-confidence: 3
captured-issues: 1
skipped-passes: []
---

# Review — 022-deterministic-runtime

## Summary

Scope: 60 files modified since the spec re-entered `in-progress` at `a525beb`
(the tasks 45–46 work — the review primitives, the exec-walker result-threading,
the rewritten command set, and the `review-basic` fixture), reviewed against the
11 loaded rule files across five passes. Posture is strong: sound path-containment
and secret screening on the writeCode plan reader, TOCTOU-safe atomic writes,
data-only deserialization, and no frontend code in scope. The two MUST violations
found on the prior run (2026-07-07) — a path-traversal gap in the `performReview`
scope-file reader and a silent-pass-through where waiver results failed to thread
into `write-review` on the exec path — were **fixed directly this run** and are
recorded under Resolved this run. **No MUST violations remain — not blocking.**
Three low-confidence findings (non-blocking) and one captured issue remain.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

### LOW · BE-INPUT-004 — write-review builds its write path from an unvalidated `feature`

- **File**: `runtime/src/primitives/write_review.rs:46-91`
- **Finding**: `run()` computes `feature_dir = repo.join(&root).join(&args.feature)`
  and `write_atomic`s `review.md` / rewrites `spec.md` there with no
  traversal/containment check on `feature` (the sibling `create_scenario`
  validates its feature path for exactly this). A `feature` like
  `../../../elsewhere` would direct writes outside the specs root, gated only by a
  pre-existing `spec.md` at the target. Low confidence: `feature` is a
  semi-trusted session/orchestrator routing key (validated to an existing specs
  dir by `/gov:target`) rather than raw external input, and the pattern is shared
  with the read-only review primitives. **Auto-fixable**: no. **Suggested fix**:
  mirror `create_scenario`'s `validate_no_traversal` on `feature`.

### LOW · QUAL-STUB-001 — read_plan_affected parses a bullet list but real plans use tables

- **File**: `runtime/src/primitives/compute_review_scope.rs:130-155`
- **Finding**: `read_plan_affected` only accepts dash-prefixed bullet items under
  `## Affected Files`, but the canonical format (the plan template and all real
  `specs/*/plan.md`) is a Markdown table, which the pre-existing
  `payload.rs::parse_affected_files` already parses. Every table row is skipped,
  so `plan_affected` is silently `[]` for every real plan and
  compute-review-scope's "larger set wins" branch is dead in production — scope
  always collapses to `modified_since`. Low confidence: maps to QUAL-STUB-001 by
  symptom (a reachable, work-implying path returning empty with no loud signal)
  but is arguably a plain correctness/reuse-divergence bug. **This is the same
  issue tracked under Captured issues below.** **Auto-fixable**: yes — promote
  `payload::parse_affected_files` to a shared helper and call it here (and switch
  the misleading bullet-form test fixture to a table).

### LOW · QUAL-STUB-001 — process-waivers `fired` is never populated on the exec path

- **File**: `runtime/src/schema/primitives.rs:116-119`
- **Finding**: `ProcessWaiversArgs.fired` is read from context key `fired`, but the
  walker accumulates findings under `findings`, and the `process-waivers` step
  (review step 3) runs before the five `performReview` passes (steps 4–8) that
  produce them. So `fired` is always empty on `runtime exec review`: no rule ever
  "still fires," `applied` is always empty, and every waiver is classified
  expired. Low confidence: the root cause is the linear-walker step ordering,
  partly acknowledged as a known exec-path limitation in the `review-exec-wiring`
  scenario's Edge Cases, and it is partly masked by (now that the key mismatch is
  fixed) the applied/expired threading. **Auto-fixable**: no — needs walker-level
  ordering/conditional support (a separate follow-on). The
  `applied_waiver_threads_from_process_waivers_into_write_review` walker test
  exercises the threading by seeding `fired` directly.

## Waived findings

*None.*

## Captured issues (pending /gov:groom)

One issue was logged to `specs/inbox.md` during the work window (informational —
not a review finding, does not affect the blocking count):

- Latent bug: `compute-review-scope`'s `read_plan_affected`
  (`runtime/src/primitives/compute_review_scope.rs:133`) parses `## Affected Files`
  as a bullet list, but every real plan and the `/gov:plan` template emit a
  Markdown table (which `payload.rs::parse_affected_files` parses correctly), so
  for real plans `read_plan_affected` returns empty and review scope silently
  drops the plan-affected half. Surfaced 2026-07-06 during task 46b. Route with
  `/gov:groom`. (Also recorded as a low-confidence finding above.)

## Resolved this run

The two MUST violations from the 2026-07-07 run were fixed directly (code + tests)
before this report was regenerated:

- **BE-INPUT-004 — performReview scope reader path traversal**
  (`runtime/src/interpreter/payload.rs`). Added a shared `classify_contained`
  helper (canonicalize + repo-root containment) and applied it in
  `load_scope_files` (the finding) and `load_rule_files` (defensively);
  refactored `load_plan_relevant_files` to share it, preserving its
  error-on-escape behavior. An absolute or traversing `scope` entry is now
  skipped, never read into the review payload. Test:
  `payload::tests::load_scope_files_confines_reads_to_the_repo_root`.
- **QUAL-STUB-001 — waiver results silently dropped on the exec path**
  (`runtime/src/schema/primitives.rs`). Added `#[serde(alias = "applied")]` /
  `#[serde(alias = "expired")]` to `WriteReviewArgs.applied_waivers` /
  `expired_waivers`, so `write-review` reads `process-waivers`' bare result keys
  as the walker threads them (backward-compatible — the kebab-case
  `applied-waivers` / `expired-waivers` still deserialize). Test:
  `walker::applied_waiver_threads_from_process_waivers_into_write_review`.

## Skipped passes

*None — all five passes ran.*
