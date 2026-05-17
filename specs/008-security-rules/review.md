---
spec: 008-security-rules
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 008-security-rules

## Summary

The rule files themselves (`framework/rules/security-backend.md`, `framework/rules/security-frontend.md`) plus `govern.md`, `analyze.md`, and `constitution.md` edits that wire them into the pipeline. These are the rules that `/gov:review` loads — meta-review applies, but the rule prose is what gets loaded, not interpreted code. All five passes ran; no findings. `blocking: no`.

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

The artifact under review *is* the security rule set. Each rule has Statement / Rationale / Verification / Source per the data-model schema. Rules use stable `SEC-{CATEGORY}-{NNN}` IDs (permanent per the data-model decision documented here and reaffirmed in 020).

### Reuse

Rule entry shape is reused unchanged by `framework/rules/configuration-cross.md` (017) and by any future domain rule files. The 7 edge cases codified in `analyze.md`'s "Security rules" check are the canonical loader logic, later generalized by 016 to "Rules".

### Quality

`framework/bootstrap/govern.md` Shared Files mapping uses the `update` strategy (so adopter additions survive), per the canonical-sources discipline. The Security audit (brownfield) section orders correctly between Shared Files and Per-Agent Scaffolding.

### Efficiency

N/A.

### Simplicity

Rule format is minimal — four fields plus the body statement. No tunable thresholds, no optional metadata.
