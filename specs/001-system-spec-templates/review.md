---
spec: 001-system-spec-templates
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 001-system-spec-templates

## Summary

Three project-doc templates (`system.md`, `errors.md`, `events.md`) consumed once at adoption. Pure markdown — no runtime, no code, no schemas with executable semantics. Security rules do not apply. All five passes ran; no findings. `blocking: no`.

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

No HTTP, persistence, auth, or DOM patterns in scope — the templates document conventions adopting projects will fill in. Security rules cannot meaningfully flag template prose.

### Reuse

Templates establish the canonical structure for `system.md`, `errors.md`, and `events.md` across adopted projects; the framework itself references these shapes from `framework/commands/validate.md` audit logic.

### Quality

Templates expose sections rather than baking in defaults — correctness is delegated to the adopter at fill-in time. No off-by-one or contract-violation risk applies.

### Efficiency

N/A.

### Simplicity

Templates are intentionally section-only with prompt comments. No premature configurability.
