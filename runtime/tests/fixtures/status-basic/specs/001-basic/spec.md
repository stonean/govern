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

# 001 — Basic Status Fixture

A minimal fixture used by the `/gov:status` parity test. The status is
`clarified` so that the runtime stops after `read-spec` and the host is
expected to prompt `/gov:plan` (i.e., not at the full-dashboard branch).

## Motivation

The fixture exists to exercise the `read-spec` primitive on a real
spec.md shape: frontmatter is valid YAML, the body has the canonical
sections, and the status is something other than `done` so step 2's
"stop here" branch fires.

## Acceptance Criteria

- [ ] `runtime exec status` returns exit 0.
- [ ] The output stream is exactly two envelopes: one `progress`, one
  `complete`.

## Open Questions

*None — all resolved.*
