---
spec: 039-reliability-backend-rules
reviewed-at: 2026-06-29T02:41:53Z
reviewed-against: 38ef20a463fe2162ccf0888bcc56954a9b6e9c9d
diff-base: 38ef20a463fe2162ccf0888bcc56954a9b6e9c9d
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 039-reliability-backend-rules

## Summary

Rule-introducing, markdown-tier change set: the new rule file
`framework/rules/reliability-backend.md` (`TIMEOUT`/`RETRY`/`BREAKER`/`DRAIN`/`BULK`,
eight rules) and its `/govern` Shared Files manifest row in
`framework/bootstrap/govern.md` — no application code, no new surface (the `BE`
surface already exists, so no `lint-rule-ids.sh` or `data-model.md` change). No
loaded security rule's Verification trigger fires against a rule-definition file
or a manifest row, and the reuse/efficiency/simplicity passes find nothing
actionable: the file mirrors the established backend rule-file schema and
design-time-commitment framing, and cites `api-backend.md` `BE-IDEMP`,
`observability-backend.md` `BE-HEALTH-001`, `performance-backend.md`
`BE-ASYNC`/`BE-POOL-002`, and `configuration-cross.md` `CFG-*` rather than
restating them. The quality pass confirms each of the eight Statements uses
exactly one RFC 2119 keyword; the three MUSTs (`BE-TIMEOUT-001` unbounded wait,
`BE-RETRY-001` retry storm, `BE-DRAIN-001` no graceful drain) are each
availability or cascading-failure hazards that occur regardless of scale — the
bar the spec's severity posture reserves MUST for, stated in each rationale —
while breaker adoption, deadline propagation, retry budgeting, and bulkheading
stay SHOULD; categories are disjoint from the other backend files and
`scripts/lint-rule-ids.sh` passes; and all eight acceptance criteria are
satisfied, including AC #8 — this file resolves 034's forward-reference by
landing the deferred deadlines, timeouts, retries, and circuit breakers. **0 MUST
violations — not blocking; the spec may advance to `done`.**

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
