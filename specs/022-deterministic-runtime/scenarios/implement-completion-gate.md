---
section: "Follow-on scenarios"
---

# Implement-completion-gate

## Context

`/gov:implement`'s numbered procedure covers the per-task loop, but the entire completion gate exists only as markdown-only prose with no numbered steps and no primitives: the all-tasks-complete tally, the all-acceptance-criteria tally, scenario-task completeness, the `review.last-run`/`review.blocking` gate reads and halt messages, the cross-spec `git diff` filter, and the final in-progress→done `set-status`. The LLM re-walks all of it on every run that reaches completion — the last minutes-long mechanical stretch on the pipeline's hottest command — and it orphans `mark-criterion`, which is fully wired and tested but referenced by zero command prose (the 2026-07-11 coverage review's top finding).

## Behavior

The completion gate becomes numbered, parseable steps in `/gov:implement`'s Instructions: `read-tasks` tallies task completion; the spec's criteria are read via `read-spec`; each criterion's verification stays semantic (the LLM judges whether it is met, marked as an extension seam in the prose) but each passing criterion is flipped with `mark-criterion`; the review gate reads `review.last-run`/`review.blocking` from the spec frontmatter and halts with the documented messages when unset/blocking; the final transition invokes `set-status` (in-progress → done) only after the user-approval gate. The markdown-only reference describes the same order with the same primitives named as fallback prose. `mark-criterion` gains its first real consumer.

## Edge Cases

- The cross-spec `git diff --stat` filter stays prose within its numbered step (no primitive owns it yet; it degrades to host judgment on both paths).
- A criterion that fails verification is left unchecked and reported — never batch-marked, matching the existing prose rule.
- When `tasks.md` still has unchecked tasks, the gate stops before criteria verification, exactly as the prose does today.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
