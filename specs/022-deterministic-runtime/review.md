---
spec: 022-deterministic-runtime
scenario: ask-consolidation
reviewed-at: 2026-05-16T14:15:00Z
reviewed-against: c4cbebd4983bf736f148e99fed1bf0ed8fde1d16
diff-base: 7283e2ca3af69039f08643fec77e6b3c4b6a93b4
must-violations: 0
should-violations: 0
low-confidence: 1
skipped-passes: []
---

# Review — 022-deterministic-runtime (ask-consolidation scenario)

## Summary

Re-review after the four SHOULD findings from the initial 0.4.0 pass were addressed in `gvrn 0.4.1`:

- **BE-INPUT-004 defense-in-depth** — `create-scenario` now validates `slug` (rejects path separators, dot-prefixes, empties) and `feature_path` (rejects parent components, absolute paths, empties) before any filesystem operation. `append-task` validates `feature_path` likewise. New helpers `validate_slug` and `validate_no_traversal` in `primitives/mod.rs` are reusable across future primitives that accept caller-supplied path components.
- **REUSE** — `next_task_number` collapses to a one-line consumer of the shared `iter_numbered_headings(content)` iterator in `primitives/mod.rs`. The iterator yields ATX-2 numbered headings while skipping fenced code blocks; available to any future primitive that walks `tasks.md`.
- **QUALITY** — `append-task`'s newly-created `tasks.md` conditionally emits the `[plan](plan.md)` link only when `plan.md` exists at creation time. Closes the dangling-link case.

19 new unit tests cover the validators (4 + 4), the shared heading-iterator (4), and the conditional intro (2 across the two primitives). Test suite grows 203 → 222 lib (256 total); `cargo fmt --check`, `clippy --all-targets --all-features -- -D warnings`, `lint-procedure-parseability`, `lint-tool-coverage`, `lint-frontmatter`, and `markdownlint-cli2` all clean.

Blocking: no. One low-confidence finding from the prior pass remains (advisory only; see below).

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None — all four from the 0.4.0 pass resolved in `gvrn 0.4.1`.*

## Low-confidence findings

### QUALITY (confidence 70): `derive_tasks_heading` may produce a heading that includes the entire H1 verbatim

- **File**: `runtime/src/primitives/append_task.rs` (`derive_tasks_heading`)
- **Finding**: `derive_tasks_heading` reads the feature's `spec.md`, finds the first ATX-1 heading, and emits `# {text} Tasks`. For a spec whose H1 is `"042 — Foo Bar"`, the derived heading becomes `"# 042 — Foo Bar Tasks"`. That matches the existing tasks.md convention (verified against `specs/022-deterministic-runtime/tasks.md` and similar). If a spec author wrote a verbose H1 like `"042 — Foo Bar (deprecated; superseded by 043)"`, the tasks-heading inherits the parenthetical noise. Low confidence because the convention has held in practice across 22 prior specs; flagged only because a single counter-example would surface as markdownlint MD024 (duplicate-heading) if "Tasks" already appeared in the H1 by coincidence. Not blocking.

## Waived findings

*None.*

## Skipped passes

*None.*
