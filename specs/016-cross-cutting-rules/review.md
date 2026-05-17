---
spec: 016-cross-cutting-rules
reviewed-at: 2026-05-17T23:25:00Z
reviewed-against: ef96450
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 016-cross-cutting-rules

## Summary

Generalizes the "Security rules" tier into a broader `<!-- §rules -->` constitutional section; renames the corresponding `analyze.md` section to "Rules"; adds a rule-promotion check to `/gov:groom`; adds an optional `## Applicable Rules` section to the spec template; backlinks 008 as the security instance. The post-task-9 work adds the inverse rule-citation check to `/gov:analyze`: for every rule ID cited under a spec's `## Applicable Rules` whose Verification trigger does not fire against the spec, emit an advisory finding. Severity resolved to advisory in v1 with a promotion criterion (5 stale citations in `/gov:analyze --all`, two consecutive runs). Tech-stack alignment skipped via `.govern.toml`. Loaded rule files: `configuration-cross.md` (text-first project — no backend/frontend code). All five passes ran; no findings. `blocking: no`.

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

No security surface — this spec is the abstraction over rule files, and the new step adds inert procedural text to a slash-command source. No rule loaded by stack detection (`configuration-cross.md` covers constants/env-vars, neither of which are introduced).

### Reuse

The new step explicitly reuses the existing rule-loading machinery (suffix-based discovery, `loaded rule files: ...` notice) and the "fired set" produced by steps 8 and 9. The markdown-only reference subsection names both directions (rule-fires-not-cited, cited-rule-does-not-fire) in one place so future readers see the rule-citation audit as a single concern with two faces. No parallel mechanism introduced.

### Quality

Step numbering verified end-to-end: step 9 unchanged, new step 10 inserted, render step renumbered 10 → 11. Cross-reference inside the new step (`handled by step 5 (check-rule-ids)`) still points at the correct existing step. Skip condition explicit ("Skip this step when the spec has no `## Applicable Rules` section"), which matches the scenario's first edge case. The promotion criterion (5 stale citations across `/gov:analyze --all`, two consecutive runs) is recorded in the body so future maintainers can act on it without re-deriving the threshold. `.claude/commands/gov/analyze.md` regenerated and in sync.

### Efficiency

N/A — markdown procedure text. The runtime cost of the new step is bounded by the size of `## Applicable Rules` (typically 0–5 IDs per spec), and it reuses the already-computed fired set rather than re-evaluating triggers.

### Simplicity

Single new step, single new subsection in the markdown-only reference, single regeneration. No new primitive added (the scenario explicitly noted no runtime primitive is required — the comparison is host-side and the inputs are already available). The advisory severity is intentionally the simplest landing; the promotion criterion deferred to operational evidence.
