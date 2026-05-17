---
spec: 016-cross-cutting-rules
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 016-cross-cutting-rules

## Summary

Generalizes the "Security rules" tier into a broader `<!-- §rules -->` constitutional section; renames the corresponding `analyze.md` section from "Security rules" to "Rules"; adds a rule-promotion check to `/gov:groom`; adds an optional `## Applicable Rules` section to the spec template; backlinks 008 as the security instance. Pure markdown — security rules do not apply. All five passes ran; no findings. `blocking: no`.

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

No security surface — this spec is the abstraction over rule files, not a rule file itself.

### Reuse

The rename from "Security rules" → "Rules" turns 008's pattern into the generalized convention later reused by 017's `configuration-cross.md` and 020's review-time loader.

### Quality

The `## Applicable Rules` section in `spec.md` is a comment-prompted optional section (consistent with §design-principles: degrading silently when omitted is fine *because* this is a planning aid, not a quality gate; the gate is the `analyze.md` loader logic, not the per-spec listing).

### Efficiency

N/A.

### Simplicity

Single generalization, no parallel rule-loading mechanisms. The signpost on 008 preserves the historic body without rewriting.
