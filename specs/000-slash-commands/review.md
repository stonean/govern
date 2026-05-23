---
spec: 000-slash-commands
reviewed-at: 2026-05-23T00:00:00Z
reviewed-against: f63446b
diff-base: f63446b
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 000-slash-commands

## Summary

Pipeline slash-command templates, refreshed against HEAD after spec 000 was reopened a second time to add the `target-clear-flag` scenario. The diff under review consists of: a new prose step in `framework/commands/target.md` describing a `--clear` invocation that removes the session JSON, supporting prose tweaks (the argument-hint frontmatter and a renumbering of steps 2–8 → 3–9 with one internal cross-reference update from "step 4" → "step 5"), the regenerated `.claude/commands/gov/target.md`, the new scenario file, and a checkbox flip plus task append in `tasks.md`. All artifacts are agent-interpreted markdown; no source code is introduced.

Scope note: this review evaluates the working tree (HEAD `f63446b` plus the uncommitted task-15 diff). The bundled commit will land at a new SHA; `reviewed-against` records `f63446b` because that's the most recent committed state and the new step's behavior is what the review actually evaluates.

The shipped security/quality/efficiency rules apply conceptually but find no surface in the diff for the same reasons recorded in the prior review at `badbc80`: no HTTP, no DOM, no auth, no executable code path. The new `--clear` semantics reuse an existing convention already documented in spec 022's `dashboard-primitive` scenario (session file absent → `session-target: null`) rather than inventing a parallel "empty-session" file format. All five passes ran; no findings. `blocking: no`.

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

The `--clear` step deletes `.claude/gov-session.json`, a session-state pointer that contains a feature slug and an optional scenario slug — no secrets, no PII, no auth tokens. The deletion is operator-invoked and path-fixed; there is no input-driven path traversal surface. The shipped security rules describe HTTP, persistence, auth, and DOM patterns — none of which apply to a single file unlink in agent-interpreted prose.

### Reuse

The `--clear` mechanism reuses the existing "session file absent → null" semantic that the `dashboard` primitive already implements (per spec 022's `dashboard-primitive` scenario). No new file-state convention is introduced — the reset state is the documented null path, not a new "empty-session" sentinel.

### Quality

The new step's behavior is fully specified: the mutex against feature arguments / scenario suffixes is explicit with a concrete error message; the idempotent semantics on an already-absent file are explicit (no-op delete, confirmation still emitted); permission-denied falls through to the same OS-error envelope used elsewhere. The cross-reference update in step 9 ("step 4" → "step 5") is consistent with the renumbering of the gen-spec-deps step.

### Efficiency

N/A — a single `unlink` on a file path known at parse time. No loops, no extra I/O, no allocation pressure.

### Simplicity

The change is minimal: one new prose step, one argument-hint update, and a mechanical renumbering. No new flags beyond `--clear`, no new args, no new config keys, no new state file shape. Reusing the documented "session absent → null" path avoids inventing parallel reset semantics — consistent with the **Design Principles** rule's "prefer derived over disciplined" framing.
