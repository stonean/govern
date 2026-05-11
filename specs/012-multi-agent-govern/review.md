---
spec: 012-multi-agent-govern
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 012-multi-agent-govern

## Summary

Unified `/govern` installer (replacing per-CLI `govern.md` / `govern-auggie.md`), agent registry, and `commands/setup/{claude,auggie}.md` split. Pure markdown — security rules do not apply at the framework-source level. The bootstrap installer itself is interpreted by an AI agent at adoption time; fetch semantics audited in spec 015. All five passes ran; no findings. `blocking: no`.

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

The unified `govern.md` operates within the constitutional §text-first invariant. Per-agent settings_template values are hardcoded paths within the framework tarball (no operator input). The Agent Registry is data, not code.

### Reuse

The single-installer model collapses two parallel installer files into one source of truth. `data-model.md` documents the registry schema as the canonical reference.

### Quality

The signpost added to 007's spec correctly points readers to 012 for the current installer shape; the resolved-note refresh in `specs/spec.md` keeps the cross-cutting decisions doc consistent.

### Efficiency

N/A.

### Simplicity

Unification reduces duplicate maintenance surface and was a precondition for later per-agent additions to live in one place.
