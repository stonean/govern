---
spec: 033-rule-surface-setting
reviewed-at: 2026-06-28T13:29:53Z
reviewed-against: fafb52b1841f45e8d14d6d13f43a31e3d85b9ae6
diff-base: fafb52b1841f45e8d14d6d13f43a31e3d85b9ae6
must-violations: 0
should-violations: 0
low-confidence: 2
captured-issues: 0
skipped-passes: []
---

# Review — 033-rule-surface-setting

## Summary

Markdown-tier feature: the change set is slash-command source and documentation prose (`framework/bootstrap/govern.md`, `framework/commands/review.md`, `framework/commands/analyze.md`, `README.md`) — no application code. No loaded security rule's Verification trigger fires against command-source procedures, and the reuse/efficiency/simplicity passes find nothing actionable. The quality pass surfaces two low-confidence spec-completeness gaps (degenerate `surfaces` configurations), both non-blocking. **0 MUST violations — not blocking; the spec may advance to `done`.**

Rule-file selection for this run: `[rules] surfaces` unset in govern's own `.govern.toml`, so step 5 fell back to detected-stack derivation; `[review] tech-stack-verified = true` skipped the alignment check.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

### quality — empty `surfaces` list is unspecified (confidence 60)

- **File**: `framework/bootstrap/govern.md`, `framework/commands/review.md`
- **Finding**: The procedures distinguish `surfaces` **set** from **unset** (unset → derive). An explicitly **empty** list (`surfaces = []`) is a third state — by the literal selection rule ("keep rule files whose surface is listed in `surfaces`") it would select only `*-cross.md` files. That may be a legitimate "cross-cutting rules only" intent, but it is not documented as such, and it is easy to confuse with unset.
- **Auto-fixable**: no
- **Suggested fix**: Add one sentence stating whether `surfaces = []` means "cross-only" (distinct from unset = derive) or is rejected as invalid. Worth a follow-up `/gov:amend` if the team wants it pinned down; not blocking.

### quality — invalid `surfaces` member handling is unspecified (confidence 55)

- **File**: `framework/bootstrap/govern.md`, `framework/commands/review.md`
- **Finding**: Member values are documented as `{backend, frontend}` with `"cross"` rejected, but the procedures do not state the behavior on an out-of-set member (e.g., `"fullstack"`, a typo) — error and abort, ignore-and-warn, or silent drop.
- **Auto-fixable**: no
- **Suggested fix**: Specify the validation behavior for unknown members (recommend: fail-fast with a named error, consistent with `CFG-ENV-003` fail-fast posture). Follow-up `/gov:amend`; not blocking.

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no inbox additions since diff-base._

## Skipped passes

_None — all five passes ran._
