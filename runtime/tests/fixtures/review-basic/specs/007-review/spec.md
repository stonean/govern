---
status: in-progress
dependencies: []
tags: []
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 007 — Review Fixture

A minimal fixture for the `/gov:review` exec parity test. Drives the
`compute-review-scope → discover-rule-files → performReview ×5 →
process-waivers → write-review` pipeline against a single-commit repo.

## Motivation

Exercises result-threading (task 46): `compute-review-scope`'s scope and
`discover-rule-files`'s selected rule files must reach every `performReview`
request payload, and `write-review` must consume the seeded `diff-base` and
the accumulated findings.

## Acceptance Criteria

- [ ] `runtime exec review` walks all nine steps to completion.
- [ ] Each `performReview` request carries the populated scope and rules.

## Open Questions

*None — all resolved.*
