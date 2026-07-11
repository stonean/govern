---
spec: 022-deterministic-runtime
scenario: review-scope-plan-affected-table-format
reviewed-at: 2026-07-11T02:26:43Z
reviewed-against: 26688e5
diff-base: 6d42d52
must-violations: 0
should-violations: 0
low-confidence: 2
captured-issues: 0
skipped-passes: []
---

# Review — 022-deterministic-runtime

## Summary

Re-review after the spec was reopened for task 47. It covers the changes since
the reopen (`6d42d52`) — task 47's Affected-Files parser unification plus the
incidental clippy `?`-rewrites and the toolchain/dependency maintenance — and
re-verifies the findings from the 2026-07-07 review against the current code
(HEAD `26688e5`). All three code findings that had a fix path are now resolved,
each with a test: the `performReview` scope-file path traversal (`BE-INPUT-004`)
and the silently-dropped waiver threading (`QUAL-STUB-001`) were fixed in
`416f780`, and the `read_plan_affected` bullet-vs-table divergence
(`QUAL-STUB-001`) was fixed by task 47 — `compute-review-scope` and the writeCode
plan reader now share one canonical table parser (`primitives::parse_affected_files`),
which this run exercised (the primitive now resolves a non-empty plan-affected
scope where it previously returned empty). The changed code is a pure refactor
plus behavior-preserving lint rewrites — no new violations. 11 rule files loaded;
no frontend code in scope; no waivers. **No MUST violations — not blocking.**

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

Both carry forward from the prior review in code untouched by this reopen's
work; each remains non-blocking.

### LOW · BE-INPUT-004 — write-review builds its write path from an unvalidated `feature`

- **File**: `runtime/src/primitives/write_review.rs:46-91`
- **Finding**: `run()` writes `review.md` / rewrites `spec.md` under
  `repo.join(&root).join(&args.feature)` with no traversal check on `feature`
  (the sibling `create_scenario` validates its feature path). Low confidence:
  `feature` is a semi-trusted session routing key validated to an existing specs
  dir by `/gov:target`, and writes are gated by a pre-existing `spec.md` at the
  target. **Auto-fixable**: no. **Suggested fix**: mirror `create_scenario`'s
  `validate_no_traversal` on `feature`.

### LOW · QUAL-STUB-001 — process-waivers `fired` is never populated on the exec path

- **File**: `runtime/src/schema/primitives.rs:116-119`
- **Finding**: on `runtime exec review`, `process-waivers` (step 3) runs before
  the `performReview` passes that produce findings, so its `fired` input is
  always empty and every waiver is classified expired. Low confidence: the root
  cause is the linear-walker step ordering, a known exec-path limitation noted in
  the `review-exec-wiring` scenario's Edge Cases (the applied/expired threading
  itself is now wired, per the `416f780` fix). **Auto-fixable**: no — needs
  walker-level step-ordering/conditional support (a separate follow-on).

## Waived findings

*None.*

## Captured issues (pending /gov:groom)

*None* — the `read_plan_affected` issue logged during earlier work was groomed
into task 47 (now implemented), so the inbox has no additions in this window.

## Resolved this run

Findings from the 2026-07-07 review, confirmed resolved against the current code:

- **BE-INPUT-004 — performReview scope reader path traversal** — fixed in
  `416f780` via the shared `classify_contained` containment check in
  `load_scope_files`. Test: `payload::tests::load_scope_files_confines_reads_to_the_repo_root`.
- **QUAL-STUB-001 — waiver results silently dropped on the exec path** — fixed in
  `416f780` via serde aliases so `write-review` reads `process-waivers`' bare
  result keys. Test: `walker::applied_waiver_threads_from_process_waivers_into_write_review`.
- **QUAL-STUB-001 — read_plan_affected parsed a bullet list, not the canonical
  table** — fixed by **task 47**: `parse_affected_files` promoted to a shared
  `primitives` helper called by both readers. Tests:
  `primitives::tests::parse_affected_files_*` and
  `compute_review_scope::tests::plan_affected_wins_when_it_is_the_larger_set`
  (now table-form).

## Skipped passes

*None — all five passes ran.*
