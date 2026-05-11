---
spec: 004-tech-stack-selection
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 004-tech-stack-selection

## Summary

Edits to `.claude/commands/gov/init.md` (tech stack questionnaire) and `AGENTS.md` (Tech Stack comment shape). Pure markdown changes; security rules do not apply. All five passes ran; no findings. `blocking: no`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._

## Pass notes

### Security

No security surface — the changes are conversational flow definitions for an AI agent.

### Reuse

Tech-stack questionnaire later feeds the workflow registry (005); the shape established here is the contract that 005 and 010 build on.

### Quality

`init.md` is the documented hand-maintained exception to the generator rule (per AGENTS.md `Gotchas`); the change touches it directly without violating the generator boundary.

### Efficiency

N/A.

### Simplicity

The question flow is linear; no speculative branches.
