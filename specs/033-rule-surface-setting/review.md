---
spec: 033-rule-surface-setting
reviewed-at: 2026-06-29T01:00:19Z
reviewed-against: cbc5117e7d5fa009bcf7fec0a4c7c0fd48bd4d13
diff-base: 98f859520f2672b58830911d891f6f9eeb14a98e
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 033-rule-surface-setting

## Summary

Markdown-tier change set: slash-command source and documentation prose
(`framework/commands/review.md` → generated `.claude/commands/gov/review.md`,
`framework/bootstrap/govern.md`) — no application code. This run covers the
work window reopened at `98f8595` (groom added the `degenerate-surfaces-config`
scenario and Task 8); the prior review (`reviewed-against: fafb52b`) predates
that work. No loaded rule's Verification trigger fires against command-source
procedures, and the reuse/efficiency/simplicity passes find nothing actionable.
The quality pass confirms Task 8 **resolves both low-confidence findings the
prior review raised** — degenerate `surfaces` configurations are now fully
specified, internally consistent across `§Inputs` ↔ `§Behavior step 5` ↔
`govern.md` ↔ the scenario, and the cross-doc-consistency audit passes.
**0 MUST violations — not blocking; the spec may advance to `done`.**

Rule-file selection for this run: `[rules] surfaces` unset in govern's own
`.govern.toml`, so step 5 fell back to detected-stack derivation;
`[review] tech-stack-verified = true` skipped the alignment check.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None. The two low-confidence findings from the prior review are resolved:_

- _**empty `surfaces` list is unspecified** (prior confidence 60) — resolved.
  `surfaces = []` is now specified as **cross-only**, explicitly distinct from
  the key being unset, in `review.md` §Behavior step 5, `review.md` §Inputs,
  `govern.md` §Collect Project Inputs item 4 / §Project Configuration /
  §Shared Files, and the `degenerate-surfaces-config` scenario._
- _**invalid `surfaces` member handling is unspecified** (prior confidence 55)
  — resolved. An unrecognized member (including `"cross"`), a list mixing valid
  and invalid members, and a non-list value all **fail fast** with a named
  error, consistent with `CFG-ENV-003`'s fail-fast posture, in both
  `/gov:review` and `/govern`._

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no inbox additions since diff-base._

## Skipped passes

_None — all five passes ran._
