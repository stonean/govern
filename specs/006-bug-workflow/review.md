---
spec: 006-bug-workflow
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 006-bug-workflow

## Summary

Scenario lifecycle: `templates/scenario.md`, `templates/triage.md` (later renamed to `inbox.md` by spec 011), `scenario` and `triage` slash-command sources, plus constitution and command-file edits to thread scenarios through the pipeline. Pure markdown — security rules do not apply. All five passes ran; no findings. `blocking: no`.

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

No security surface. Scenario files are spec elaborations consumed by AI agents.

### Reuse

The scenario file shape is reused by every subsequent spec elaborating against a section (009, 010, 011, 012, 020). The triage/inbox convention threads through `/gov:groom`, `/gov:capture`, and `/gov:log`.

### Quality

Decision-tree branches in `constitution.md` §bug-handling are exhaustive over the documented inputs (new-bug-no-section, regression-of-section, edit-existing-spec, etc.).

### Efficiency

N/A.

### Simplicity

Triage rules describe a single migration loop with explicit success/failure conditions. No accumulated complexity.
