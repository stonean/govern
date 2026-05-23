---
spec: 000-slash-commands
reviewed-at: 2026-05-23T00:00:00Z
reviewed-against: badbc8039ae3fc273a1c8a8e9d50dbd1ba404121
diff-base: badbc8039ae3fc273a1c8a8e9d50dbd1ba404121
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 000-slash-commands

## Summary

Pipeline slash-command templates (about, target, status, setup, specify, clarify, plan, implement, validate, next), refreshed against HEAD after spec 000 was reopened to add the `dashboard-dependencies-column` scenario. The diff since this in-progress cycle began (`badbc80`) consists of a one-line prose edit to step 3 of `framework/commands/status.md` (and its regenerated copy under `.claude/commands/gov/`), a new scenario file, a checkbox flip in `tasks.md`, and the frontmatter status flip — all text-first artifacts interpreted by an AI agent, no source code introduced. Security rules (`security-backend.md`, `security-frontend.md`) and the other shipped rule sets are loaded conceptually but do not apply for the same reason recorded in the prior review at `3d7c50b`: no HTTP surface, no DOM, no auth flow, no executable code path under review. All five passes ran; no findings. `blocking: no`.

Note: the prior review's qualifier still applies — many of the original 2025-era command files have since been superseded by later specs (013 frontmatter migration, 014 reclarify back-edges, 020 review gate). The review evaluates the current state of each file as of HEAD `badbc80`, per `/gov:review`'s idempotency invariant.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._ All five passes ran.

## Pass notes

### Security

No security-sensitive code introduced. The `dashboard-dependencies-column` change modifies one line of agent-interpreted prose describing a table column and a marker convention; no input parsing, no HTTP, no persistence, no DOM. The shipped security rules describe HTTP, persistence, auth, and DOM patterns — none of which apply to slash-command sources.

### Reuse

The Dependencies column is sourced directly from `specs[].dependencies` already returned by the `dashboard` primitive (per spec 022's `dashboard-primitive` scenario) — the renderer reuses an existing field rather than introducing a parallel data path. The bold-marker replacement removes a one-off convention without introducing new abstractions.

### Quality

The rendering change is described as instructions to an AI agent; correctness is a property of those instructions, not of executable code. The new prose specifies cell content ("comma-separated three-digit NNN prefixes from `specs[].dependencies`, sorted ascending — `—` when the array is empty") and the no-target case ("when session-target is null, no row is bolded") explicitly, including the edge cases the scenario file enumerates (empty deps, drift, no target, stale target).

### Efficiency

N/A — no executable code. The renderer change adds one column derived from already-loaded data; no extra primitive call.

### Simplicity

The change removes a fragile convention (`>>` leading the first cell, which some renderers strip) and replaces it with bold (`**slug**`), a well-supported inline span. The Dependencies column reuses the existing `—` empty-cell convention already used by Data-model. No new abstractions, no premature configuration, no dead branches.
