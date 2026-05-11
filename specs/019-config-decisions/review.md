---
spec: 019-config-decisions
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 019-config-decisions

## Summary

Establishes `.govern.toml` as the canonical adopter-side configuration store; documents the schema in `data-model.md`; updates `govern.md` Project Configuration section and the workflow-recommendation flow; refreshes README "Pinning files" section. Pure markdown — security rules do not apply. All five passes ran; no findings. `blocking: no`.

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

`.govern.toml` contains adopter configuration only — no secrets surface. Pinning entries identify govern-shipped files the adopter wants exempted from `update` strategy; values are file paths within the adopter's repo.

### Reuse

`.govern.toml` is treated as a shared adopter-side database, not a schema owned by any single spec (per AGENTS.md Workflow note added 2026-05-10). 020 added `[review]` section, 020's review and other future specs add their own keys following the same pattern — no parallel config files.

### Quality

The schema declaration in `data-model.md` is the single source of truth; new keys added by other specs are documented in their own bodies (per the same workflow note). Per-category prompt now has three documented options (`always update` / `prompt each time` / `never update`) — explicit enum prevents ambiguous adopter responses.

### Efficiency

N/A.

### Simplicity

Single TOML file with namespaced sections (`[pinned]`, `[review]`, …) rather than per-spec config files. Adopters edit one place.
