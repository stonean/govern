---
spec: 035-groom-session-target
reviewed-at: 2026-06-28T14:17:35Z
reviewed-against: 43e4ad0d1acbe566d9b45342809247362b630212
diff-base: 43e4ad0d1acbe566d9b45342809247362b630212
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 1
skipped-passes: []
---

# Review — 035-groom-session-target

## Summary

The change set is prose edits to one slash-command source (`framework/commands/groom.md`) and its regenerated copy — no application code. No loaded security rule's Verification trigger fires against command-source prose, and the reuse/quality/efficiency/simplicity passes find nothing actionable: the session-write description follows the same read-`cli-config-dir` / tempfile+rename pattern `specify.md` and `amend.md` already document (consistent, not duplicative logic). markdownlint, procedure-parseability, and both audits pass. **0 MUST violations — not blocking; the spec may advance to `done`.**

One incidental issue was captured to `specs/inbox.md` during implementation (below) — a pre-existing groom behavior this change makes visible, not a defect in this change.

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

- **`/gov:groom` does not reopen a `done` spec when it adds a scenario (Step 4).** Pre-existing groom behavior surfaced while implementing 035: groom Step 4 creates a scenario + task but never flips `done → in-progress` (unlike `/gov:amend`'s scenario route). With 035 now setting the session target to that spec, a follow-on `/gov:implement` against a still-`done` spec gate-fails. Recorded in `specs/inbox.md`; route via `/gov:groom`. Not a defect in 035's change (which only sets the target).

## Skipped passes

_None — all five passes ran._
