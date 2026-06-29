---
spec: 035-groom-session-target
scenario: confirmation-names-reopen
reviewed-at: 2026-06-29T01:23:45Z
reviewed-against: c97c5b9d64f6ce110ff8d9a447d0df8e21240dda
diff-base: c97c5b9d64f6ce110ff8d9a447d0df8e21240dda
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 035-groom-session-target

## Summary

Scenario-targeted run (`confirmation-names-reopen`). Markdown-tier change set:
prose edits to one slash-command source (`framework/commands/groom.md`) and its
regenerated `.claude/commands/gov/groom.md` copy, plus the new scenario file — no
application code. Task 6 closes the transparency gap the prior review
(`reviewed-against: acffa85`) raised as low-confidence: when groom's Step 4
routes a scenario to a `done` spec, the per-item routing confirmation now names
the `done → in-progress` reopen, so the operator consents to the status change
before it happens — matching `/gov:amend`'s practice. No loaded rule's
Verification trigger fires against command-source prose, and the
reuse/efficiency/simplicity passes find nothing actionable: the confirmation
extends the existing single-prompt pattern and the reworded bullet cross-references
the Step 4 example rather than duplicating it. The quality pass confirms the
implementation is consistent across the item-4 confirmation, the "No separate
prompt, but the reopen is named" bullet, the Completion line, and the scenario;
the cross-doc-consistency audit passes. **0 MUST violations — not blocking; the
spec may advance to `done`.**

Rule-file selection for this run: `[rules] surfaces` unset in govern's own
`.govern.toml`, so step 5 fell back to detected-stack derivation;
`[review] tech-stack-verified = true` skipped the alignment check.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None. The prior review's low-confidence finding (done-spec reopen not named in
the routing confirmation) is resolved: the Step 4 confirmation now names the
reopen for `done` specs and is unchanged for non-`done` specs._

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no inbox additions since diff-base._

## Skipped passes

_None — all five passes ran._
