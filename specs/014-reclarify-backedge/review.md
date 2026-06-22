---
spec: 014-reclarify-backedge
reviewed-at: 2026-05-24T21:40:00Z
reviewed-against: 36461bdd3456c3cd666546aeeedf21452c072640
diff-base: 1aca7680000000000000000000000000000000000
must-violations: 0
should-violations: 1
low-confidence: 0
skipped-passes: []
---

# Review — 014-reclarify-backedge

## Summary

Re-review after Task 8 (Options A + B for the reopen-after-informal-edits scenario) landed in `36461bd`. Scope: `framework/commands/amend.md` (new "Re-open precondition" section), `.claude/commands/gov/amend.md` (regenerated mirror), `AGENTS.md` (Workflow entry for the agent-side `set-status` shortcut), and the new scenario file. Pure markdown; the only rule file that applies to a markdown scope is `configuration-cross.md`, which has no triggers in prose. One SHOULD finding caught and fixed in this same review: the decline branch of the new precondition contradicted the scenario's "Delta exists but the user intends to add a new scenario" edge case. `blocking: no`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

### SHOULD: SCENARIO-CONTRACT — decline branch contradicted edge case (fixed in-review)

- **File**: `framework/commands/amend.md:66`
- **Rule**: Scenario-documented contract: `specs/014-reclarify-backedge/scenarios/reopen-after-informal-edits.md` Edge Cases — _"Delta exists but the user intends to add a new scenario — Option B's prompt is offered before classification, so the user can decline the re-open prompt and continue into the scenario branch with the new input."_
- **Finding**: The initial Option B implementation wrote step 5 as "On **decline**, exit without modifying any file." That terminates `/amend` entirely, which contradicts the scenario's edge case requiring the user to be able to decline the re-open and still route a new scenario input. The contradiction is internal to the same section — the trailing paragraph already promised the opt-out-and-continue behavior.
- **Auto-fixable**: yes
- **Suggested fix**: replace "On **decline**, exit without modifying any file" with "On **decline**, continue to **Gather the input** without modifying any file" plus the disambiguating sentence about empty vs. new input.
- **Status**: **fixed in this review run** (no separate commit yet — fix is in the working tree alongside this report).

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._

## Pass notes

### Security

Markdown command-spec edits, no executable surface. The new `git status --porcelain` invocation is scoped to specific paths inside the feature directory; no user-controlled input flows into the command line.

### Reuse

The "detect on-disk delta, prompt user, mutate status" shape echoes `/gov:clarify`'s recovery path from Tasks 1–2 (status + open-question-count → prompt → revert). No shared helper is warranted in a markdown-spec framework, but the pattern is now applied symmetrically across `/amend` and `/clarify` — operators can predict it from one to the other.

### Quality

Found the one SHOULD finding above and fixed it. The status-mutation table picked up a new row for the precondition path; the row's placement after the existing `done | scenario` row is acceptable because each row is independently descriptive. The `set-status` primitive used by the runtime path matches the markdown-only path's direct frontmatter edit — same observable outcome.

### Efficiency

`git status --porcelain` is the canonical machine-parseable interface, scoped to three paths. O(small) on every `done`-spec `/amend` invocation; the cost is justified by the user-visible re-open shortcut.

### Simplicity

Option A is a single AGENTS.md Workflow entry — no new primitive, no new command flag. Option B adds one section, one table row, and one scope-boundary note to `framework/commands/amend.md`. No premature abstraction.
