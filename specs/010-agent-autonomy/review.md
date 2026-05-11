---
spec: 010-agent-autonomy
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 010-agent-autonomy

## Summary

`[simple]` task marker, `--auto` flag on `/gov:implement`, stuck-detection step, plus the `skills/` → `workflows/` rename (and the cross-spec migration into 005). Pure markdown across command sources, templates, constitution, and adopter docs. The directory rename is byte-preserving for templates. Security rules do not apply. All five passes ran; no findings. `blocking: no`.

Note: the `[simple]` task marker introduced here was later removed by spec 017 (derive-don't-ask) along with the related plan.md proposal step. The autonomy mechanism itself (the `--auto` flag and gates) remains.

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

`--auto` gates are documented in `framework/commands/implement.md` (e.g., halt on missing review, halt on validate failure). The gates compose with the §runtime-boundary's text-first invariant — no autonomy decision crosses into runtime.

### Reuse

Stuck-detection reads `tasks.md` commits (not affected-files commits), reusing the canonical task list as the progress signal.

### Quality

The `[simple]` marker was a discipline-dependent input — by 017 it was removed in favor of git-derived signal, consistent with the §design-principles "never depend on human diligence" rule.

### Efficiency

N/A.

### Simplicity

The directory rename and flatten collapsed an unnecessary nested `templates/` directory — a simplification.
