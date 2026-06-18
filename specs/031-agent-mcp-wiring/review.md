---
spec: 031-agent-mcp-wiring
reviewed-at: 2026-06-18T23:30:38Z
reviewed-against: ef1aaccca59ac6e982fdda8f0b14f2cd78daf5c8
diff-base: 0f8e334aed51ab0993359c245acb81f72a688926
must-violations: 0
should-violations: 1
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 031-agent-mcp-wiring

## Summary

031 is an entirely text-first change: per-agent MCP registration prose in
`framework/bootstrap/govern.md`, a README correction, a Claude-specific phrasing sweep
across eight command sources + `runtime-tools.txt`, two non-reopening signposts in
028/029, and the 031 spec artifacts. **No application code, no backend/frontend surface,
no constants or env vars** are introduced — so the security, efficiency, and
configuration (`configuration-cross.md`) rule surfaces do not apply. The quality pass
focused on internal consistency of the modified bootstrap procedure (the State-B branch
by `mechanism`, the abort-message variants, the per-agent registration table, and the
permission-write semantics) and found it coherent. **0 MUST violations — not blocking.**
One advisory simplicity observation on the descriptor schema. The Antigravity descriptor
is backed by a recorded live-`agy` verification (`scenarios/antigravity-mcp-verification.md`),
not an assumption.

## MUST violations (blocking)

None.

## SHOULD violations (advisory)

### SHOULD: SIMPLICITY — `scope` field is descriptive metadata, not behaviorally load-bearing

- **File**: `framework/bootstrap/govern.md` (§MCP registration table) and `specs/031-agent-mcp-wiring/data-model.md`
- **Rule**: AGENTS.md §Design Principles / simplicity pass — avoid fields that are not load-bearing.
- **Finding**: The per-agent descriptor carries `scope` (`project-committed` / `user-global` / `home-level`) alongside `mechanism` (`write-file` / `surface-instruction`). Only `mechanism` drives State-B branching, and the exact location is already given by `target`. `scope` is therefore derivable/descriptive — `user-global` (Auggie) and `home-level` (Antigravity) both map to `surface-instruction` and differ only in *which* home location `target` already names.
- **Auto-fixable**: no
- **Suggested fix**: Keep as-is (recommended). `scope` documents a real conceptual distinction readers care about — committed-in-repo vs. user-config-dir vs. home-global — and the three-line table costs nothing. Recorded as advisory so the trade-off is visible; not worth a schema change.

## Low-confidence findings

None.

## Waived findings

None.

## Captured issues (pending /gov:groom)

None — no issues were appended to `specs/inbox.md` during the work window.

## Skipped passes

None — all five passes ran. Security / efficiency / configuration found no applicable
surface (text-first change, no code); this is recorded as 0 findings, not skipped.
