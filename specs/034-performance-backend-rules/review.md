---
spec: 034-performance-backend-rules
reviewed-at: 2026-06-28T13:49:21Z
reviewed-against: 0f28ba44089461eca4e7f8378ae836fee6a62936
diff-base: 0f28ba44089461eca4e7f8378ae836fee6a62936
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 034-performance-backend-rules

## Summary

The change set is a new rule file (`framework/rules/performance-backend.md`, 13 rules) and a one-row addition to the `/govern` Shared Files manifest in `framework/bootstrap/govern.md` — rule-set authoring, not application code. No loaded security rule's Verification trigger fires against rule-file prose or a manifest table row, and the reuse/efficiency/simplicity passes find nothing actionable (the file cross-references `BE-PAGE` / `BE-AUTHZ` / `BE-STATUS` / `CFG-CONST-003` rather than restating them). Schema conformance, ID grammar, category disjointness, and the MUST/SHOULD severity posture were verified by the validation gate (`lint-rule-ids`, `lint-rule-filenames`, markdownlint, procedure-parseability, and both audits — all green). **0 MUST violations — not blocking; the spec may advance to `done`.**

Rule-file selection for this run: `[rules] surfaces` unset in govern's own `.govern.toml`, so step 5 fell back to detected-stack derivation; `[review] tech-stack-verified = true` skipped the alignment check.

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
