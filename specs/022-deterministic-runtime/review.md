---
spec: 022-deterministic-runtime
scenario: runtime-primitive-structural-bugs
reviewed-at: 2026-05-18T00:30:00Z
reviewed-against: 7f9b121
diff-base: c7fdf585b91a1f6f7d6e4f6e1f1f5b1a8e1f7a1b
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 022-deterministic-runtime (runtime-primitive-structural-bugs scenario)

## Summary

Final review of the `runtime-primitive-structural-bugs` scenario after the four-phase autonomous run landed `gvrn 0.5.1`. Stack: govern is text-first markdown + bash with an opt-in Rust runtime under `runtime/`. Loaded rule files: `configuration-cross.md` (no CFG-* triggers fire against the Rust changes — the diff introduces no env-var lookups, operator-tunable constants, or shared cross-module values). No security rule file applies to the runtime crate at the framework level. All five passes ran; 0 findings. `blocking: no`.

**Scope.** `runtime/src/primitives/append_task.rs`, `runtime/src/primitives/read_tasks.rs`, `runtime/src/primitives/check_stuck.rs`, helper additions to `runtime/src/primitives/mod.rs` (`TasksStructure`, `detect_tasks_structure`, `iter_task_numbers_at_levels`, `iter_phase_ranges`, `PhaseRange`, `MissingArgument`, `ParentHeadingNotFound`), schema additions to `runtime/src/schema/primitives.rs` (`AppendTaskArgs.slug`, `AppendTaskArgs.parent_heading`, `Task.phase`), CHANGELOG entry, `Cargo.toml`/`Cargo.lock` version bump, plus the cross-spec fix to `framework/commands/analyze.md` and its regenerated `.claude/commands/gov/analyze.md` mirror (the spec-016 step-10 parser-regression hit during the parity test sweep for Phase 1).

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

## Pass notes

### Security

No security rule file applies. The Rust changes do not introduce HTTP, authentication, persistence, or shell-out paths beyond the existing primitives (which were reviewed in their introducing scenarios). The new `MissingArgument` and `ParentHeadingNotFound` error variants are operational errors that surface caller-side mistakes; they do not change the primitive's trust boundary.

### Reuse

Strong reuse — the shared helpers in `primitives/mod` (`TasksStructure`, `detect_tasks_structure`, `iter_task_numbers_at_levels`, `iter_phase_ranges`) are explicitly designed to be consumed by both `append-task` (Phase 2) and `read-tasks` (Phase 3). The Phase 2 commit lands the helpers; Phase 3 consumes them without duplicating detection logic. The deprecated single-purpose `iter_numbered_headings` wrapper is removed cleanly — its only callers were tests, which were migrated to invoke `iter_task_numbers_at_levels(_, &[2])` directly. `heading_starts_with_number` is duplicated between `primitives/mod` and `primitives::read_tasks` — annotated in the latter as "kept module-local to avoid widening the crate-internal surface." A future refactor could promote the helper, but the duplication is one short function and the boundary is intentional; not a finding.

### Quality

26 new unit tests across `append_task`, `read_tasks`, and `check_stuck` cover the four bug fixes and their edge cases (scenario-listed edge cases: mixed structure, alternate phase label, parent-heading-not-found, reopen, mechanical sweeps, never-reopened baseline). All atomic-write semantics preserved (tempfile-in-parent + persist). Two clippy fixes during the Phase 2 commit (collapsed `if a {} else if b {}` blocks with identical bodies into `if a || b {}`; corrected doc-comment backticks). Cross-spec regression caught: spec 016's new step 10 in `framework/commands/analyze.md` placed `check-rule-ids` inside a backtick code span, which the runtime parser interpreted as a primitive dispatch — fix landed in the same Phase 1 commit (reword to "step 5" without the code span).

### Efficiency

N/A — markdown / Rust changes only. The new phased-structure detection adds one extra line-walk over `tasks.md` content per `append-task` / `read-tasks` call (O(n) where n = lines in the file, dominated by the existing parse). No new I/O or sync points.

### Simplicity

Each bug fix is a focused commit. Phase 4 in particular was scoped down once investigation showed the implementation was already correct — only regression tests landed rather than reimplementing already-working logic. The default-phase-heading logic was extended past the strict-letter reading of Q2 to also extend an existing `Phase X — Follow-on scenarios` phase (preventing letter explosion across successive follow-ons); the deviation from the Q2 wording is documented in the Phase 2 commit message and motivated by a test that surfaced the unintended pathological behavior under the strict reading.
