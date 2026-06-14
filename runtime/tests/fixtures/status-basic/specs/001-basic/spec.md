---
status: clarified
dependencies: []
tags: [test, pipeline]
references:
  - service: api
    spec: 003-user
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 001 — Basic Status Fixture

A minimal fixture used by the `/gov:status` parity test. Status
`clarified` with no dependencies — the dashboard reports it as unblocked
and renders its next action as `/gov:plan`.

## Motivation

The fixture exercises the `dashboard` primitive on a real spec.md shape:
frontmatter is valid YAML, the body has the canonical sections, and the
spec carries `tags` so the `tags-union` fold produces a non-empty
result.

It also declares one cross-service reference — the [api User model](https://github.com/acme/api/blob/main/specs/003-user/spec.md)
— harvested into the derived `references:` index above. The registered
`api` service (see `.govern.toml`) resolves it to `ok` against the
`checkouts/api` checkout, exercising the reference readout without
perturbing the `dashboard` stream.

## Acceptance Criteria

- [ ] `runtime exec status` returns exit 0.
- [ ] The output stream is exactly two envelopes: one `progress`, one
  `complete`.

## Open Questions

*None — all resolved.*
