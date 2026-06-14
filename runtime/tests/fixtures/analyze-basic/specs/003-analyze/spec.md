---
status: clarified
dependencies: []
references:
  - service: api
    spec: 003-user
  - service: api
    spec: 099-ghost
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

## Cross-service references

Two cross-service references exercise the broken-reference check,
harvested into the derived `references:` index above:

- [api User model](https://github.com/acme/api/blob/main/specs/003-user/spec.md)
  — resolves to `ok` against the registered `api` checkout (the clean
  reference).
- [api Ghost spec](https://github.com/acme/api/blob/main/specs/099-ghost/spec.md)
  — `broken`: the `api` service is registered and its checkout is
  reachable, but the target spec is absent upstream, so `/gov:analyze`
  raises an Advisory broken-reference finding.

The `unregistered` / `not-checked-out` / `status-unreadable` outcomes are
informational unknowns, never findings.

## Acceptance Criteria

- [ ] Every mechanical primitive completes without operational error.
- [x] At least one rule citation is parsed.

## Open Questions

*None — all resolved.*
