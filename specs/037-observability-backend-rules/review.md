---
spec: 037-observability-backend-rules
reviewed-at: 2026-06-29T02:16:18Z
reviewed-against: e375c1cffc3e2127cd3520fb61fd108d211d1c24
diff-base: e375c1cffc3e2127cd3520fb61fd108d211d1c24
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 037-observability-backend-rules

## Summary

Rule-introducing, markdown-tier change set: the new rule file
`framework/rules/observability-backend.md` (`METRIC`/`TRACE`/`HEALTH`, six rules)
and its `/govern` Shared Files manifest row in `framework/bootstrap/govern.md` —
no application code, no new surface (the `BE` surface already exists, so no
`lint-rule-ids.sh` or `data-model.md` change). No loaded security rule's
Verification trigger fires against a rule-definition file or a manifest table
row, and the reuse/efficiency/simplicity passes find nothing actionable: the
file mirrors the `performance-backend.md` schema and design-time-commitment
framing, and cites `security-backend.md` `BE-LOG-006`, `performance-backend.md`,
and `configuration-cross.md` `CFG-*` rather than restating them. The quality
pass confirms each of the six Statements uses exactly one RFC 2119 keyword; the
two MUSTs (`BE-TRACE-001` trace-context propagation, `BE-HEALTH-001` readiness
distinct from liveness) are the detection/diagnosis-blocking-regardless-of-scale
absences the spec's severity posture reserves MUST for, with that rationale
stated in each; categories are disjoint from the other backend files and
`scripts/lint-rule-ids.sh` passes; and all seven acceptance criteria are
satisfied by the implementation. **0 MUST violations — not blocking; the spec
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
