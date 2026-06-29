---
spec: 038-concurrency-backend-rules
reviewed-at: 2026-06-29T02:32:22Z
reviewed-against: 40885b822de1d6ba2196affa71324c4bb1ed2054
diff-base: 40885b822de1d6ba2196affa71324c4bb1ed2054
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 038-concurrency-backend-rules

## Summary

Rule-introducing, markdown-tier change set: the new rule file
`framework/rules/concurrency-backend.md` (`RACE`/`LOCK`/`TXN`/`COORD`, eight
rules) and its `/govern` Shared Files manifest row in
`framework/bootstrap/govern.md` — no application code, no new surface (the `BE`
surface already exists, so no `lint-rule-ids.sh` or `data-model.md` change). No
loaded security rule's Verification trigger fires against a rule-definition file
or a manifest row, and the reuse/efficiency/simplicity passes find nothing
actionable: the file mirrors the `performance-backend.md` /
`observability-backend.md` schema and design-time-commitment framing, and cites
`api-backend.md` `BE-IDEMP`, `performance-backend.md` `BE-POOL-*`, and
`configuration-cross.md` `CFG-*` rather than restating them (idempotency is not
duplicated as a colliding category). The quality pass confirms each of the eight
Statements uses exactly one RFC 2119 keyword; the four MUSTs (`BE-RACE-001` data
race, `BE-TXN-002` lost update, `BE-COORD-001` missing fencing token,
`BE-COORD-002` non-idempotent retry) are each silent-corruption hazards that two
concurrent actors suffice to trigger — the scale-independent corruption bar the
spec's severity posture reserves MUST for, stated in each rationale — while the
contextual choices (optimistic vs. pessimistic locking, isolation-level
selection, race-surface reduction) stay SHOULD; categories are disjoint from the
other backend files and `scripts/lint-rule-ids.sh` passes; and all seven
acceptance criteria are satisfied. **0 MUST violations — not blocking; the spec
may advance to `done`.**

Rule-file selection for this run: `[rules] surfaces` unset in govern's own
`.govern.toml`, so step 5 fell back to detected-stack derivation;
`[review] tech-stack-verified = true` skipped the alignment check.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no inbox additions since diff-base._

## Skipped passes

_None — all five passes ran._
