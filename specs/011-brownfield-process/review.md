---
spec: 011-brownfield-process
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 011-brownfield-process

## Summary

`triage` → `inbox` rename across template, command source, and `.claude/commands/gov/` mirror; new `capture` command for brownfield spec sketches; constitution, README, and `AGENTS.md` updates threading the renaming through. Pure markdown — security rules do not apply. All five passes ran; no findings. `blocking: no`.

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

No security surface.

### Reuse

The capture command shape reuses the standard `framework/commands/*.md` shape; the inbox template format is consistent with the broader scenario/spec markdown convention.

### Quality

Rename is exhaustive across the affected-files table — every callsite of `triage` migrated. The "not modified (historical specs — self-contained at time of writing)" list explicitly identifies which prior-spec internals to leave alone, supporting the "frozen archaeology" pattern later codified in 017's plan.

### Efficiency

N/A.

### Simplicity

Single rename + one new command. No abstraction layer between triage and inbox concepts — they're the same thing under a clearer name.
