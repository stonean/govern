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

# 005 — Plan Fixture

A minimal fixture for the `/gov:plan` parity test. The spec is at
clarified; the procedure exercises read-spec, lint-markdown, three
writeSpecBody extension calls, a gate-confirm, and set-status to
advance the status to planned.

## Motivation

Drives the clarified → planned transition through the runtime so the
strict-fields parity check on the status transition is testable.

## Acceptance Criteria

- [ ] `runtime exec plan` walks the procedure to completion.
- [ ] The spec's status flips from clarified to planned after set-status.

## Open Questions

*None — all resolved.*
