---
status: draft
dependencies: []
tags: [foundation]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 000 — Blocker Fixture

A draft spec that blocks downstream consumers — `002-blocked` depends on
it. Exists to exercise `dashboard`'s `blocked-by` computation: a dep at
status `draft` is below `clarified` and is therefore listed in the
dependent's `blocked-by` array.

## Open Questions

- One unresolved item to give this spec a non-zero open-question count.
