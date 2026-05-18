---
section: "Follow-on scenarios"
---

# Family-8-burndown

## Context

[`scripts/audit/introducing-drift.sh`](../../../scripts/audit/introducing-drift.sh) (Family 8) flags ~9 done specs (`000`, `011`, `014`, `017`, `020`, `021`, `022`, `023`, `024`) that retain backticked references to renamed commands — `/capture`, `/elaborate`, `/validate`, `gov-rt:` — in current-tense prose. Each rewrite is a small word-substitution from "A new `/capture` command provides..." to "`/capture` provided..." (or replacement with the new name's past-tense form where the rename is canonical).

Origin: spec 026's `/audit` v1 advisory findings, 2026-05-18. Captured via the inbox.

## Behavior

Family 8 burns down **maintainer-paced**, not as a single batch. The design intent (recorded in 026's plan and tasks §17 "Final lint sweep" notes): the audit's continuous output is the tracker, and each affected spec's body is rewritten the next time the maintainer touches it for an unrelated reason. The advisory remains advisory under `continue-on-error: true` (see [`audit-ci-hard-gate`](audit-ci-hard-gate.md)) until the count reaches zero organically.

Rationale: a batch rewrite across 9 done-spec bodies would qualify as a meaningful edit per [§spec-lifecycle](../../../framework/constitution.md#spec-lifecycle) and re-open every affected spec's review state. Letting each rewrite ride a future meaningful edit avoids manufacturing review churn for what is, conceptually, a mechanical-sweep follow-up to the rename commits that introduced the drift.

## Edge Cases

- **A maintainer touches an affected spec but skips the Family 8 rewrite.** The next `/audit` run still flags the residual references; the burndown stays visible. The advisory does not gate the unrelated edit.
- **The advisory list grows** (a new rename introduces fresh past-tense drift). Each new rename's commit should land its own past-tense sweep across the affected live-artifact bodies per the AGENTS.md "No dead references in live artifacts" rule; done-spec bodies that pre-date the rename inherit the burndown pattern documented here.

## Open Questions

*None.*

## Resolved Questions

- **Should Family 8 be a single batch rewrite or distributed?** **Distributed (maintainer-paced).** Rationale above: batch rewrite would reopen ~9 done specs for what amounts to mechanical word substitution, manufacturing review churn. The audit's recurring advisory output is a better tracker than a stale TODO list. Confirmed 2026-05-18 during the inbox-emptying groom pass.
