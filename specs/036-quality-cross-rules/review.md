---
spec: 036-quality-cross-rules
reviewed-at: 2026-06-29T02:01:56Z
reviewed-against: 7615f7fe656b26db674656e8c54068e6d892ae7c
diff-base: 7615f7fe656b26db674656e8c54068e6d892ae7c
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 036-quality-cross-rules

## Summary

Rule-introducing, markdown-tier change set: the new rule file
`framework/rules/quality-cross.md` (inaugural `QUAL-STUB-001`), the `QUAL`-surface
registration in `scripts/lint-rule-ids.sh`, the surface registry
`specs/036-quality-cross-rules/data-model.md`, and the `/govern` Shared Files
manifest wiring in `framework/bootstrap/govern.md` — no application code. No
loaded security rule's Verification trigger fires against a rule-definition file
or a static regex edit, and the reuse/efficiency/simplicity passes find nothing
actionable: the rule file mirrors the `configuration-cross.md` /
`performance-backend.md` schema, cites `api-backend.md` `BE-SCHEMA-002` rather
than restating it, and the §Shared Files note was made count-free (removing a
hand-maintained number that had silently drifted, rather than re-introducing
one). The quality pass confirms the implementation is internally consistent
across the rule file, `data-model.md`, the seven spec acceptance criteria, and
the manifest; `QUAL-STUB-001`'s Statement uses exactly one RFC 2119 keyword
(MUST) and its Verification is the review-time three-part discriminator with the
approved exemption list. `scripts/lint-rule-ids.sh` accepts `QUAL-STUB-001` and
still rejects malformed IDs. **0 MUST violations — not blocking; the spec may
advance to `done`.**

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

_None — no inbox additions since diff-base. (An incidental count-drift in the
§Shared Files note, found mid-implement, was inside the editing task's scope and
fixed in place — the note is now count-free — rather than logged.)_

## Skipped passes

_None — all five passes ran._
