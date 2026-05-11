---
status: planned
dependencies: []
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 004 — Implement Fixture

A minimal fixture for the `/gov:implement` parity test. Has one
pending task and a write boundary that admits edits inside
`runtime/**`.

## Motivation

Exercises the runtime's read-tasks → derive-boundary → check-stuck →
gate-confirm → set-status → writeCode → mark-task pipeline, including
the write-boundary check against the writeCode response payload.

## Acceptance Criteria

- [ ] `runtime exec implement` walks the procedure to completion.
- [ ] The writeCode extension's edits are validated against the write
  boundary.

## Open Questions

*None — all resolved.*
