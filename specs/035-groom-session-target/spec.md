---
status: clarified
dependencies: [006-bug-workflow, 017-derive-dont-ask, 023-govern-refinement]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 035 — Groom sets the session target from the routed item

`/gov:groom` sets `.govern.session.toml` to the spec it routes an inbox item to, so a follow-on `/gov:amend` or `/gov:implement` operates on the right target without a manual `/gov:target`.

## Motivation

`/gov:groom` walks the inbox and, for each item, finds the matching feature by searching `specs/` (the [006-bug-workflow](../006-bug-workflow/spec.md) decision tree, §bug-handling) and routes it — a spec edit (Step 3) or a scenario under the matching spec (Step 4). But groom never writes the session target: its Context says a target "is not required" because it operates across all specs. The consequence is friction at exactly the moment the spec is known — groom has *just identified* the right feature, yet the operator must remember it and run `/gov:target NNN` by hand before the follow-on `/gov:amend` or `/gov:implement`.

This is the same "don't make the operator remember session state" gap that self-contained inbox items only half-close: the item names its target, but nothing carries that target into the session. This spec carries it the rest of the way — groom sets the target as part of the routing it already performs.

## Behavior

- When groom routes an item to an **existing spec** — a spec edit (Step 3) or a scenario created under the matching spec (Step 4, durable-requirement branch) — it sets `.govern.session.toml` to that feature as part of the routing action. The target is the feature the decision tree matched in Step 2 (reinforced by, but not dependent on, any `specs/NNN-*/` link in the item text).
- The per-item routing confirmation groom already requires before acting now **names the target it will set** — e.g., *"Create a scenario under `033-rule-surface-setting` and set it as the session target? (Y/n)"*. That single confirmation is the consent for both the routing and the target write; no separate target prompt is added (consistent with the procedural-fidelity / don't-add-prompts stance). The operator sees and confirms the target without having to recall it.
- **New-spec items** (Step 2, no spec exists → `/gov:specify`) are unchanged: `/gov:specify` already targets the spec it creates.
- **Rule items** (Step 1, amend a rule file) and **chores** (Step 4 chore, left in the inbox) set no target — neither has a single spec home.
- Across a multi-item run, the session target **follows the current item**: each spec-routed item sets it, so when the run ends the target points at the most recently groomed spec (the one the operator is most likely to act on next).
- The session write **preserves any existing `cli-config-dir`** (the per-contributor agent identity), using the same `write-session` target-write semantics from [023-govern-refinement](../023-govern-refinement/spec.md); it must not be dropped.
- The completion summary names the resulting session target (or states it is unchanged when no item set one).

The change is confined to `framework/commands/groom.md` (and its generated `.claude/commands/gov/groom.md` copy).

## Acceptance Criteria

- [ ] When groom routes an item to a spec edit (Step 3) or a scenario under the matching spec (Step 4 durable branch), it sets `.govern.session.toml` to that feature.
- [ ] The per-item routing confirmation names the target it will set; groom adds no separate "set the target?" prompt.
- [ ] New-spec items, rule-file items (Step 1), and chores (Step 4 chore) do **not** set a session target via groom.
- [ ] Across a multi-item run, the session target follows the current item (the last spec-routed item is the final target).
- [ ] The session-target write preserves any existing `cli-config-dir` value.
- [ ] The completion summary names the resulting session target (or states it is unchanged when no item set one).
- [ ] `framework/commands/groom.md` documents the behavior, and its generated `.claude/commands/gov/groom.md` copy regenerates cleanly.

## Resolved Questions

- **Auto-set vs. explicit prompt.** Resolved: **no separate prompt.** The per-item routing confirmation groom already requires now names the target it will set, so that one confirmation is the consent for both the routing and the target write. The operator is still shown and confirms the target (satisfying "prompt for the session target") without a redundant second prompt — consistent with [017-derive-dont-ask](../017-derive-dont-ask/spec.md) and the procedural-fidelity stance.
- **Multi-item groom run.** Resolved: the target **follows the current item** — each spec-routed item sets it, so the run ends with the target pointing at the most recently groomed spec (the most likely next action). Not set-once.
- **Step 3 spec-edit vs. Step 4 scenario.** Resolved: **both.** Any routing to an existing spec sets the target; the operator's next command applies equally to a spec edit or a newly-created scenario.
- **Completion-summary detail.** Resolved: the summary names the **final** session target (the most recently set), and states "session target unchanged" when no groomed item set one. A per-item trail is unnecessary noise.
