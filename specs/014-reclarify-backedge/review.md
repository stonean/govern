---
spec: 014-reclarify-backedge
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 014-reclarify-backedge

## Summary

Adds a status-mutation back-edge to `/gov:ask` and reworks `/gov:clarify` and `/gov:plan` to handle re-entry (existing artifact detection, keep/replace prompt, duplicate-question detection, `done` refusal). Constitution §spec-lifecycle bullet rewrite. Pure markdown — security rules do not apply. All five passes ran; no findings. `blocking: no`.

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

Status-mutation branching is the same shape later reused by `/gov:review`'s pre-`done` gate (020).

### Quality

The `done` refusal path is explicit: once a spec is `done`, neither `/gov:ask` nor `/gov:clarify` can mutate status without an explicit reopen. This composes correctly with 020's blocking gate (a `done` spec with a review violation has to round-trip back to `in-progress` via `/gov:analyze --fix`).

### Efficiency

N/A.

### Simplicity

Back-edge is intentionally narrow: only `/gov:ask` mutates status backward; `/gov:plan` and `/gov:clarify` detect existing artifacts but do not silently overwrite.
