---
spec: 005-workflows
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 005-workflows

## Summary

Workflow registry (`framework/workflows/registry.json`) and nine workflow markdown files (lint/test/format for TypeScript, Python, Go) plus init/govern.md edits to wire them in. The registry is data, not executable; workflows are markdown command surfaces consumed by adopting projects' AI agents. Security rules do not apply. All five passes ran; no findings. `blocking: no`.

Note: the directory location and naming were subsequently changed by spec 010 (renamed from `skills/` to `workflows/` and flattened). Review evaluates current state at HEAD.

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

`registry.json` carries no secrets, no executable references — only canonical names mapped to workflow file paths within the repo. Workflow files themselves document tool invocations the adopter agent runs; the framework neither executes them nor proxies them.

### Reuse

Registry shape is canonicalized in `data-model.md` and consumed by `framework/bootstrap/govern.md`. No parallel registries elsewhere.

### Quality

Mapping is keyed by stack selection (a closed enum). The registry is data-only with no consumer outside `govern.md`'s documented loader logic.

### Efficiency

N/A.

### Simplicity

Nine workflow files cover the three core stacks × three concerns (lint/test/format) — minimal cross-cut, no speculative entries.
