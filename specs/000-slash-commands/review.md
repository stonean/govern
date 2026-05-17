---
spec: 000-slash-commands
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 000-slash-commands

## Summary

Pipeline slash-command templates (about, target, status, setup, specify, clarify, plan, implement, validate, next). Text-first artifacts — agent-interpreted markdown only, no runtime. Security rules (`security-backend.md`, `security-frontend.md`) are loaded conceptually but do not apply: no HTTP surface, no DOM, no auth flow. All five passes ran; no findings. `blocking: no`.

Note: many of these files have since been superseded by later specs (013 frontmatter migration, 014 reclarify back-edges, 020 review gate). The review evaluates the current state of each file as of HEAD `3d7c50b`, per `/gov:review`'s idempotency invariant.

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

No security-sensitive code introduced. Pure markdown command templates interpreted by an AI agent at invocation time. The shipped security rules describe HTTP, persistence, auth, and DOM patterns — none of which apply to slash-command sources.

### Reuse

Templates establish the reusable command shape that subsequent specs follow (description frontmatter, Inputs / Behavior / Output sections). No duplication introduced; later command files reuse the structure.

### Quality

Command behaviors are described as instructions to an AI agent; correctness is a property of those instructions, not of executable code. The pipeline state transitions described here are cross-checked by `/gov:analyze` against actual spec frontmatter; no contract violations.

### Efficiency

N/A — no executable code. Markdown is interpreted once per invocation.

### Simplicity

The command set is intentionally minimal: one command per pipeline phase plus a small set of orient/elaborate commands. No dead branches, no unused fields.
