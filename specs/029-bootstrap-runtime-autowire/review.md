---
spec: 029-bootstrap-runtime-autowire
reviewed-at: 2026-06-11T23:43:10Z
reviewed-against: 7afcdb87de30b7fb97e62367eb4d55f16b047639
diff-base: efcb6d5a6b129131725c9a06ec3012d49e7b380f
must-violations: 0
should-violations: 0
low-confidence: 2
captured-issues: 1
skipped-passes: []
---

# Review — 029-bootstrap-runtime-autowire

## Summary

029 is a text-first feature: its entire scope is Markdown — the bootstrap procedure (`framework/bootstrap/govern.md`), the three `configure/*.md` permission files, and the README. There is no executable code, so the security/quality/efficiency passes have little code surface to bite on. Rule selection loaded `configuration-cross.md` (the only cross-cutting rule; no backend/frontend application surface is in scope), but its triggers — operator-tunable values, env vars, secrets, cross-module constants — are not exercised: the feature introduces only fixed config literals (the `gvrn` server entry, fixed `command -v` / `which` probes, fixed permission wildcards). **0 MUST and 0 SHOULD violations; not blocking.** Two low-confidence advisories record assumptions the prose itself already hedges, and one incidental issue was captured to the inbox during the work.

## MUST violations (blocking)

None.

## SHOULD violations (advisory)

None.

## Low-confidence findings

### quality — Auggie gvrn tool-permission wildcard may not be honored (confidence 55)

- **File**: `framework/bootstrap/govern.md` (§Permission Setup → gvrn runtime auto-wiring)
- **Finding**: The State-B tool-permission grant uses `{ "toolName": "mcp:gvrn:*", … }` for Auggie "if Auggie's matcher honors the wildcard, otherwise the enumerated set." Whether Auggie's `toolName` matcher supports a `*` wildcard is unverified; if it does not, the next Auggie session could prompt per gvrn tool. The prose already names the enumerated fallback, so this is a documented assumption, not a defect.
- **Auto-fixable**: no
- **Suggested fix**: Confirm Auggie wildcard support before an Auggie adopter relies on it; if unsupported, make the enumerated `mcp:gvrn:<tool>` set the primary grant for Auggie.

### quality — antigravity `command(which)` grammar match is asserted, not verified (confidence 50)

- **File**: `framework/bootstrap/govern.md` (§gvrn runtime detection → Detection mechanism; Agent Registry antigravity seed) and `framework/bootstrap/configure/antigravity.md`
- **Finding**: The antigravity probe uses `which gvrn` authorized by `command(which)`, chosen because the token-prefix matcher keys on the leading token. This is asserted to "match cleanly" but not empirically verified against a live antigravity session. The plan flags it as an implement-time validation item.
- **Auto-fixable**: no
- **Suggested fix**: Verify `command(which)` authorizes `which gvrn` without an over-broad match on a real antigravity install; adjust if the token-prefix behavior differs.

## Waived findings

None.

## Captured issues (pending /gov:groom)

- New audit family `scripts/audit/runtime-probe-parity.sh` to guard seed↔configure permission parity — deferred from this feature (Task 9) as its own spec. (Added to `specs/inbox.md` during implementation.)

## Skipped passes

None.
