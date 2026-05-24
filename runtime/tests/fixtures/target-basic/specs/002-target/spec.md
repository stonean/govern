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

# 002 — Target Fixture

A minimal fixture for the `/gov:target` parity test. The status is
`planned` so the host-side render after the runtime returns will route
to `/gov:implement` as the next pipeline step.

## Motivation

Exercises the runtime's `read-spec` primitive when /gov:target is
invoked with a feature already resolved to `002-target`. The fixture
sets the session file to point at this feature so the parity check on
`.govern.session.toml` can compare host-written values across the
LLM-driven and runtime-driven paths.

## Acceptance Criteria

- [x] Frontmatter is valid YAML with a status field.
- [ ] `runtime exec target feature=002-target` returns exit 0 with the
  expected envelope stream.

## Open Questions

*None — all resolved.*
