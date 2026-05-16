---
status: clarified
dependencies: []
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 003 — Analyze Fixture

A minimal fixture for the `/gov:analyze` parity test. Exercises every
mechanical primitive plus the assessSpecQuality extension point.

<!-- §spec-phase -->

## Motivation

Self-contained anchor target: this section is referenced as §spec-phase
above. The resolve-anchor primitive should report the reference as
resolved.

Rule citations exercised: CFG-CONST-001 (defined in
framework/rules/configuration.md within the same fixture).

## Acceptance Criteria

- [ ] Every mechanical primitive completes without operational error.
- [x] At least one rule citation is parsed.

## Open Questions

*None — all resolved.*
