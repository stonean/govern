---
spec: 022-deterministic-runtime
scenario: mark-task-backtick-headings
reviewed-at: 2026-05-19T00:00:00Z
reviewed-against: 389b68b
diff-base: 5d04421c157c042f7d23e170fbffd9ad8797661b
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 022-deterministic-runtime (mark-task-backtick-headings scenario)

## Summary

Reviewed task #33's implementation — the `mark-task` heading-parser alignment with `read-tasks`. The work is a focused REUSE refactor: `runtime/src/primitives/mark_task.rs` now detects `tasks.md` structure (flat vs phased) via the existing `detect_tasks_structure` helper and walks the appropriate task level (2 for flat, 3 for phased), exactly the way `read_tasks.rs` already does. `parse_atx_heading` was already in use for heading parsing, so the diff is narrow: one new import (`TasksStructure`, `detect_tasks_structure`), one structure detection call before line-splitting, one new `task_level` parameter on `locate_task_range`, and one terminator-condition relaxation (`level <= task_level` instead of the hardcoded `level <= 2`).

Origin: spec 022's `mark-task-backtick-headings` scenario, routed from `specs/inbox.md` via `/gov:groom` after the bug surfaced during `/gov:implement` on spec 023 task #19. The originating symptom described "backticks in the title"; the actual cause was the structure-detection gap (level-2-only matching ignored phased `### N.` task headings). Inline-code spans in titles parse correctly via `parse_atx_heading` and were never the root cause — but the symptom only surfaced on phased files, which always have backticks-or-not headings as a function of the spec being implemented.

Scope:

- `runtime/src/primitives/mark_task.rs` — structure-aware task-level dispatch via `detect_tasks_structure`; `locate_task_range` takes a `task_level` parameter; terminator condition generalized to `level <= task_level`. Module-doc comment updated to document both file shapes.
- Test fixture `write_phased_fixture` and three new regression tests (`flips_subtask_in_phased_tasks_md`, `resolves_phased_task_with_backticks_in_heading`, `phased_task_range_terminates_at_next_phase_container`) cover the previously-broken path. The 6 existing tests still pass unchanged.

Stack: text-first markdown + Rust runtime. Loaded rule files: `configuration-cross.md`, `security-backend.md`, `api-backend.md`. None of the BE-API or BE-AUTHN/AUTHZ/etc. triggers fire — pure internal-primitive refactor.

Five-dimension review:

- **Security**: no input boundaries added; no new path/IO surface. The primitive's existing path-handling already validates feature-dir existence.
- **Reuse**: the change IS the REUSE win — `mark-task` and `read-tasks` now consume the same `detect_tasks_structure` helper, eliminating the structure-detection drift that caused this bug. Future heading-shape edge cases fix once, propagate to both primitives.
- **Quality**: terminator-condition generalization is the only behavior-altering line. Reviewed against the four existing tests (which all pass) and the three new tests; no regression. The Done-when's "regression test exercises a heading like `### N. Dedup ` + backtick + `/configure` + backtick" is satisfied by `resolves_phased_task_with_backticks_in_heading`.
- **Efficiency**: structure detection runs once before line-splitting; `O(file_size)` linear scan, same as before.
- **Simplicity**: the diff is ~10 lines of behavior change + ~50 lines of tests. The `locate_task_range` function gains one parameter and one comparison; no new control flow.

Test posture: 268 tests pass (`cargo test --release`); `cargo clippy --release --all-targets -- -D warnings` clean; `cargo fmt --check` clean.

**Result**: 0 MUST, 0 SHOULD, 0 low-confidence. `blocking: no`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._
