---
section: "Follow-on scenarios"
---

# Audit-script-refactors

## Context

Two refactor findings against `scripts/audit/*.sh` surfaced during 026's review, both bundled here because they touch the same surface:

- **REUSE-001 — shared boilerplate extraction.** Each of the nine family check scripts ([`check-zero`](../../../scripts/audit/check-zero.sh), [`cross-doc-consistency`](../../../scripts/audit/cross-doc-consistency.sh), [`manifest-parity`](../../../scripts/audit/manifest-parity.sh), [`registry-equivalence`](../../../scripts/audit/registry-equivalence.sh), [`placeholder-roundtrip`](../../../scripts/audit/placeholder-roundtrip.sh), [`template-alignment`](../../../scripts/audit/template-alignment.sh), [`ssot-invariants`](../../../scripts/audit/ssot-invariants.sh), [`sibling-coupling`](../../../scripts/audit/sibling-coupling.sh), [`introducing-drift`](../../../scripts/audit/introducing-drift.sh), [`primitive-promotion-candidates`](../../../scripts/audit/primitive-promotion-candidates.sh)) repeats ~10 lines of identical setup: `set -uo pipefail`, `ROOT="$(git rev-parse --show-toplevel)"`, `cd "$ROOT"`, `drift=0`, the `emit()` function with pipe-separated output. A shared `scripts/audit/lib.sh` exposing `audit_emit FAMILY LOCATION MESSAGE FIX` plus the ROOT/cd setup would eliminate ~90 lines of duplication.
- **QUALITY-001 — `flush_step` caller-scope mutation.** [`scripts/audit/primitive-promotion-candidates.sh`](../../../scripts/audit/primitive-promotion-candidates.sh)'s `flush_step()` function reads and mutates seven caller-scope variables (`step_start_line`, `step_buffer`, `step_has_primitive`, `step_has_llm_marker`, `step_has_ignore`, plus `emit`'s `drift`). Bash semantics make this work — functions inherit caller scope by default — but the pattern is fragile: adding a `local` keyword anywhere in the function would silently break it. Confidence 70%: correct as tested but brittle against refactor.

Origin: spec 026 review SHOULD / low-confidence findings, 2026-05-18. Captured via the inbox.

## Behavior

REUSE-001 ships as a shared `scripts/audit/lib.sh` that family check scripts source:

```bash
. "$(dirname "$0")/lib.sh"      # provides ROOT, cd, audit_emit, drift
```

`audit_emit FAMILY LOCATION MESSAGE FIX` writes the existing pipe-separated row and increments `drift`. Family check scripts shrink to their per-family check logic plus a final `exit $drift`. The orchestrator [`run-all.sh`](../../../scripts/audit/run-all.sh) is unaffected — it shells out to each family script, so the lib is internal to the family scripts.

QUALITY-001 ships as a refactor of `flush_step` to take its state as explicit arguments and return the emit count via stdout (or exit code), eliminating the caller-scope reads. The per-file loop in `primitive-promotion-candidates.sh` is restructured to build a step list first, then iterate the returned list to apply flush logic — same observable output, no fragile caller-scope dependency.

Both changes are pure refactors with no observable behavior change. Verification is `bash scripts/audit/run-all.sh` exit code parity and stdout-row diff against the pre-refactor state on a representative tree.

## Edge Cases

- **A family script wants different `emit()` semantics** (e.g., severity column). Add the column to the shared `audit_emit` signature with a sensible default; family scripts opt in.
- **Standalone invocation ergonomics under REUSE-001.** `bash scripts/audit/X.sh` continues to work because the lib is sourced via `$(dirname "$0")/lib.sh`, which resolves regardless of CWD. The documented trade-off in [`scripts/audit/README.md`](../../../scripts/audit/README.md) becomes a non-issue.
- **QUALITY-001 introduces a subshell** that breaks `drift` accumulation. Use a temp file or explicit return-by-stdout to preserve the count; verify with a fixture that has both annotated and non-annotated flagged steps.

## Open Questions

*None.*

## Resolved Questions

- **Bundle REUSE-001 and QUALITY-001 in one scenario or split?** **Bundled.** Both touch `scripts/audit/` and are pure refactors with no behavior change. A combined PR keeps the diff coherent (single review of the audit-script surface) and lets the lib.sh extraction land in the same commit window as the `flush_step` rewrite — which can use `audit_emit` directly. Confirmed 2026-05-18 during the inbox-emptying groom pass.
