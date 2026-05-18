---
section: "Follow-on scenarios"
---

# Check-stuck-tasks-md-advancement

## Context

The `check-stuck` primitive at [`runtime/src/primitives/check_stuck.rs`](../../../runtime/src/primitives/check_stuck.rs) currently sets `stuck = count >= threshold` based purely on the number of commits touching `tasks.md` since the most-recent `in-progress` transition. The [`/gov:implement`](../../../framework/commands/implement.md) contract (and the spec 022 `runtime-primitive-structural-bugs` scenario's bug-4 description) specifies a second condition that the implementation does not enforce: `stuck: true` should only fire when **the same task is still the first incomplete one** — no checkbox has flipped to `- [x]` between the commits in the count window.

Without the second condition, once 3+ commits land on `tasks.md` — even when each flips a different subtask checkbox — `stuck: true` fires on every subsequent `/{project}:implement` run for the remainder of the feature. The warning becomes background noise; agents and operators learn to dismiss it; the warning's signal-to-noise ratio collapses to zero.

Reported 2026-05-17 from the user's anvil/017-pagination implement session as a second occurrence (the bug had been spotted earlier without being filed). Out of spec 026's scope; preserved via the inbox and routed here.

## Behavior

`check-stuck` returns `stuck: true` if AND ONLY IF both conditions hold:

1. `commit_count >= threshold` (existing behavior).
2. **The index of the first `- [ ]` group in `tasks.md` has not advanced across the walked commit window.** The check identifies the first incomplete subtask at the `since-sha` commit and at HEAD. When the indices match (no progress on the first incomplete subtask), the second condition is satisfied. When the index has advanced (a subtask got flipped between then and now), the condition is false and `stuck` stays `false` regardless of `commit_count`.

The primitive's existing result shape is preserved — only the value of the `stuck` boolean changes for the false-positive case. `commit_count` and `since_sha` still report the unchanged values they did before.

## Edge Cases

- **`tasks.md` doesn't exist at `since-sha`.** Treat as "no first-incomplete subtask at baseline"; the second condition is vacuously false (the index can't fail to advance from a state that didn't exist). `stuck` stays `false`.
- **All subtasks complete at HEAD.** The first `- [ ]` group at HEAD doesn't exist. The second condition is vacuously true (or false, depending on framing); the safe answer is `stuck: false` because completion is the opposite of stuck.
- **The first `- [ ]` group changed identity between `since-sha` and HEAD** (e.g., a task was reordered or its heading was rewritten). Treat as "advanced" — heading-content equality at the index defines "same subtask"; any divergence means progress.
- **`tasks.md` uses phased structure** (per the `read-tasks` fix in `runtime-primitive-structural-bugs`). The "first incomplete subtask" is still well-defined in phased files — walk phases in order, walk subtasks in order, find the first `- [ ]`. The check applies uniformly.
- **No commits touched `tasks.md`** in the window. `commit_count: 0`, `stuck: false` — both conditions fail. No change from current behavior.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Subtask-identity equality: heading text or position?** **Position-based equality for v1.** The check tracks the linear index of the first `- [ ]` group across the full file walk (both flat `## N.` and phased `### N.` subtasks, per the `read-tasks` semantics established by `runtime-primitive-structural-bugs`). If the index at HEAD equals the index at `since-sha`, the first incomplete subtask hasn't advanced. Rationale: matches how `/gov:implement` already walks tasks (same index ⇒ same subtask in both consumers); reordering subtasks mid-implementation is rare and breaks `/gov:implement`'s ordering contract anyway; heading-text equality is complex (normalization bikeshed) and not worth building ahead of demand. Edge case: a reorder produces a false-negative on stuck (real first-incomplete subtask is the same one but at a new index) — acceptable for v1 because a false negative beats the current false-positive flood. Heading-text equality graduates to v2 if reorder churn surfaces in real usage.
- **Version bump: patch or minor?** **Patch — `gvrn 0.5.2`.** The fix doesn't change the `CheckStuckArgs` or `CheckStuckResult` JSON schema. The change tightens (narrows) when `stuck: true` fires — strictly fewer false positives, no behavior callers were correctly depending on goes away. Matches the precedent the 0.5.1 release set for `runtime-primitive-structural-bugs` (additive primitive args, preserved JSON shape, patch bump). No new MCP tool name, no new entry in `framework/runtime-tools.txt`. A minor bump would only be appropriate if behavior changed beyond "tighten when stuck fires."
