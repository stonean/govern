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

# 001 — Basic Sample Feature

A fixture spec used by the runtime primitive tests. The shape mirrors a
real govern spec: numbered acceptance criteria, an Open Questions
section, and at least one rule citation (`BE-AUTHN-001`).

## Motivation

Establish a deterministic input the read-only primitive tests can pin
their expectations against. Per §spec-phase, every fixture should look
like a real spec.

## Acceptance Criteria

- [ ] First criterion: parser surfaces both checked and unchecked items.
- [x] Second criterion: this one is checked.
- [ ] Third criterion: cites rule `BE-AUTHN-001`.

## Open Questions

- Should fixtures embed binary assets? (placeholder — kept open here so
  the parser exercises the question-extraction path.)

## Resolved Questions

- **Encoding** — UTF-8 only.
