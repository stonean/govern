---
spec: 009-scenario-targeting
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 009-scenario-targeting

## Summary

Threads scenario-level targeting through the command set (`target`, `scenario`, `clarify`, `status`, `implement`) plus an Open Questions section on the scenario template. Pure markdown — security rules do not apply. All five passes ran; no findings. `blocking: no`.

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

Scenario-target syntax (`/gov:target NNN-slug#scenario-slug`) is the canonical addressing form reused by 014's reclarify back-edge and 020's review scoping.

### Quality

Each affected command handles both spec-target and scenario-target paths; the modifications are additive (no removal of existing behavior). The "verify" rows for `question.md` and its mirror are recorded as no-ops if the file already covers scenario paths.

### Efficiency

N/A.

### Simplicity

Scenario detection is a single anchor (`#slug`); no nested addressing.
