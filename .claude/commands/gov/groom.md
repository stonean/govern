---
description: Walk the inbox and route each item to its proper home.
---

# Groom

Walk the inbox and route each item to its proper home.

## Purpose

Backlog grooming for `specs/inbox.md`. Walks each raw item through the bug decision tree and routes it to its proper home — a rule, a new spec, a spec edit, or a scenario. Not every item belongs to a spec: a **chore** (lint or formatting cleanup, dependency cleanup, repo hygiene — project maintenance with no feature home) is not spec material and is left in the inbox to be done directly, never forced into a spec, scenario, or `tasks.md` (§bug-handling, durability test). Removes items once they are migrated or resolved. Pairs with `/gov:log`, which records items to the inbox without interpreting them.

## Context

Use the session target from `.govern/session.toml` if set, but groom operates across all specs so a target is not *required* to start. When groom routes an item to an existing spec — a spec edit or a scenario under the matching spec — it **sets** the session target to that spec as part of the routing, so a follow-on `/gov:amend` or `/gov:implement` needs no manual `/gov:target`. Adding a scenario to a `done` spec also **reopens** it `done → in-progress` so that follow-on `/gov:implement` has an actionable target. See **Setting the session target** and **Reopening a `done` spec** in the Markdown-only reference below.

## Scope Boundaries

- This command grooms inbox items — it creates scenario files, appends tasks, edits a matched spec's body when the operator confirms the spec-edit route (Step 3), sets the session target (`.govern/session.toml`) when routing to an existing spec, and reopens a matched spec's frontmatter status `done → in-progress` when it adds a scenario to a `done` spec, but does NOT implement fixes. Do NOT read or modify source code or test files.
- For each item, read only the spec file of the matching feature (for decision tree evaluation) and its `tasks.md` (for appending). Do NOT read plans, data models, or source code.
- Reference: §bug-handling, §rules, §scenarios, §brownfield-inbox (constitution loaded by `/gov:target` — do not re-read).

## Instructions

> **For agent runtimes**: the Invoke steps below call the MCP tools of the optional gvrn runtime; the host-integration contract — bare↔prefixed tool names, lazy ToolSearch schema fetch, the no-shell-utilities rule, and the two-paths guarantee — lives once in the constitution, §runtime-host-integration. With no gvrn MCP server registered, walk the same prose using the host file-reading tools (Read, Edit, Write) per the Markdown-only reference below.

Process items **one at a time** — do not batch or pre-process multiple items. Complete steps 2–8 for one item (decision, confirmation, writes, removal), then repeat from step 2 for the next; step 9 runs once after all items. The decision-tree detail, prompt wording, and write shapes live under the Markdown-only reference below.

<!-- audit:ignore-promotion -->
1. Check if `specs/inbox.md` exists. If it does not exist, stop and report: "No inbox file found at `specs/inbox.md`. Nothing to groom." Read `specs/inbox.md`; if the file has no list items (no `-` bullets outside HTML comments — the shared inbox grammar the inbox primitives use: `-` list items, checkbox or plain, with any lines inside `<!-- … -->` comments ignored, such as the template's `<!-- Rules: … -->` guidance), report: "Inbox is clean — no items to groom." and stop. Keep the file to preserve git history.

2. <!-- llm:routeInboxItem --> For each item in the inbox list, one routing round trip: display the item number, total remaining count, and item description, then walk the bug decision tree (§bug-handling; the **Groom decision tree** reference below) to choose exactly one of the five routes, naming the matched feature when the route targets an existing spec:
   - `rule` — a cross-cutting concern (four-indicator promotion checklist, §rules): recommend amending the covering rule file, or — when no rule file covers the domain — note that a new rule file is its own feature spec (out of scope), leave the item in the inbox, and ask whether to skip and continue. No write, no target.
   - `spec` — the item routes to a spec rather than a scenario. With no covering spec, recommend creating one via `/gov:specify` and ask whether to create it now or skip this item, as today. With a matched spec that is ambiguous or incomplete for the reported behavior, recommend updating the spec directly and offer to help edit the spec section — a confirmed spec edit sets the session target (step 7).
   - `scenario` — a durable behavioral requirement under a matched, clear spec: steps 4–7 perform the writes.
   - `chore` — project maintenance with no feature home: not spec material; left in place per step 8's exception.
   - `discard` — not actionable or already resolved: removed from the inbox (step 8) with no other write.

3. Invoke `gate-confirm` to confirm the routing with the user before any write, and do not proceed until the user approves — the single confirmation is the consent for everything the route performs. When the item routes to an existing spec, the confirmation names the target it will set — e.g. *"Create a scenario under `NNN-slug` and set it as the session target? (Y/n)"* — so no separate target prompt is added. When the matched spec is `done` (a scenario route, which reopens it), the confirmation also names the reopen — e.g. *"Create a scenario under `NNN-slug`, reopen it to `in-progress`, and set it as the session target? (Y/n)"* — so the operator consents to the `done → in-progress` change before it happens. When the matched spec is not `done`, there is no reopen to name and the confirmation is unchanged.

4. **Scenario route (scenario file)** — invoke `create-scenario` to write `specs/{feature}/scenarios/{slug}.md`. The primitive frames it with the `section:` frontmatter, the H1-from-slug, and the Open / Resolved Questions scaffolding (this framing is compiled into the primitive, mirroring `framework/templates/spec/scenario.md`; it does not read the template file — so an adopter who customizes `specs/templates/scenario.md` sees the compiled framing on the runtime path and the on-disk template only on the markdown-only path). (`/gov:groom` keeps the inbox flow moving; for a deeper interactive walk through a single scenario, run `/gov:amend` against the parent spec — the classifier routes the input to the scenario branch.)

5. **Scenario route (task)** — invoke `append-task` to append a task to the spec's `tasks.md` referencing the scenario written in step 4 — together the two writes match `/gov:amend`'s scenario route.

6. **Reopen a `done` spec** — when the scenario route matched a spec whose status is `done`, invoke `set-status` with `from: done`, `to: in-progress` as part of the same routing action (the documented back-edge §spec-lifecycle "Backward via new scenario" defines, the same one `/gov:amend`'s scenario route performs). The `from: done` guard against a concurrent edit and the only-from-`done` rule are detailed in **Reopening a `done` spec** below; the spec-edit route is out of scope for this reopen.

7. **Set the session target** — when the item routed to an existing spec, invoke `write-session`: for a confirmed spec edit, pass the matched feature and its repo-relative `specs/{feature}` directory; for a scenario route, additionally pass the new scenario slug and `specs/{feature}/scenarios/{slug}.md`. Rule items, missing-spec items (routed to `/gov:specify`), chores, and discards set no target. The atomic-write shape and the `cli-config-dir` preservation rule are detailed in **Setting the session target** below. Across a multi-item run each spec-routed item performs this write, so the session target follows the current item and ends pointing at the most recently groomed spec.

8. After migrating an item to a spec, scenario, or rule (or otherwise resolving it — a `discard` is resolved by removal alone), invoke `remove-inbox-item` with the item's bullet text to remove it from `specs/inbox.md` (atomic write; a no-match is a clean outcome, and the result reports the remaining item count). A chore recognized in step 2 is the exception — leave it in place and tell the user it is general maintenance to be done directly (the project's lint/format/test tooling covers the common cases); it clears from the inbox when it is done, not by grooming. On the markdown-only path (no gvrn runtime registered), remove the line with a host `Edit`. Then return to step 2 for the next item.

<!-- audit:ignore-promotion -->
9. After all items are groomed, report the completion summary: how many items were migrated, how many specs were created, how many scenarios were added, how many items were recognized as chores (left in the inbox to be done directly), and how many items remain. Name any spec reopened `done → in-progress` during the run, so the status change is surfaced rather than silent. Report the resulting session target — the spec (and scenario, if any) set by the most recently groomed spec-routed item, or "session target unchanged" when no groomed item set one. If `specs/inbox.md` is now empty (no items left), report: "Inbox is clean."

## Markdown-only reference

With no gvrn runtime registered, the host performs the same walk and the same writes with its own file tools (Read, Edit, Write) — no shell-pipeline substitution — one contract, two paths (§runtime-host-integration). The routing decision per item stays the same decision tree; the sections below carry the full detail both paths follow.

### Groom decision tree

For each item, walk the steps in order; the first matching step names the route.

**Step 1: Is this a cross-cutting concern with no covering rule?** (route: `rule`)

- Apply the four-indicator promotion checklist (§rules in `.govern/constitution.md`): cross-cutting, citable, governance-recognized category, generalizable wording. If the item qualifies, recommend promoting it to a rule.
- If a loaded rule file already covers the domain (e.g., `specs/rules/security-backend.md` for an authentication concern, `specs/rules/configuration-cross.md` for an env-var concern), recommend the user amend the relevant rule file directly — note that local edits to rule files are overwritten by `/govern` unless the file is pinned in `.govern/config.toml`, so amendments belong upstream in the framework rather than in adopting projects.
- If no rule file covers the domain, creating a new rule file is its own feature spec (out of `/gov:groom`'s scope). Leave the item in the inbox unmodified — every subsequent groom pass walks every unmigrated item, including this one, so it stays surfaced. Ask the user whether to skip and continue.
- If the item is feature-specific rather than cross-cutting, fall through to Step 2.

**Step 2: Does a spec exist for this behavior?** (route: `spec`, no matched feature)

- Search `specs/` for a feature directory that covers this area.
- If no spec exists — recommend creating one via `/gov:specify`. Ask the user whether to create the spec now or skip this item. (If the item is a chore rather than a feature gap — see Step 4 — it never needs a spec; leave it in the inbox.)

**Step 3: Is the spec ambiguous or incomplete?** (route: `spec`, matched feature)

- If the existing spec does not cover the reported behavior clearly — recommend updating the spec directly. Offer to help update the spec section. When the spec edit is confirmed, **set the session target** to this feature (see **Setting the session target**).

**Step 4: Spec is clear — durable requirement, or a chore?** (route: `scenario` or `chore`)

- The spec covers the area, so there is no rule, missing-spec, or spec-edit gap. Now decide what the item *is* (§bug-handling, durability test):
  - **A durable behavioral requirement** (route: `scenario`) — a new behavior, edge case, or contract the spec covers at a high level but does not yet describe in detail. Create a scenario inline under the matching spec's `scenarios/` directory using the `specs/templates/scenario.md` template, then append a task to the spec's `tasks.md` referencing the new scenario. **If the matched spec's status is `done`, reopen it `done → in-progress`** as part of the same routing action (see **Reopening a `done` spec**), so a follow-on `/gov:implement` finds an actionable target instead of gate-failing on a `done` spec; a spec already in `draft`, `clarified`, `planned`, or `in-progress` is left unchanged. After the scenario and task are written and any reopen is applied, **set the session target** to this feature plus the new scenario (see **Setting the session target**).
  - **A chore** (route: `chore`) — project maintenance (lint or formatting cleanup, dependency cleanup, repo hygiene, a standalone refactor) that adds no durable requirement. It is **not** spec material: do not create a rule, spec, or scenario, and do not append it to any `tasks.md` — spec tasks are feature work derived from a plan, not standalone chores. Leave the item in `specs/inbox.md` (do **not** remove it), tell the user it is general maintenance to be done directly (the project's lint/format/test tooling covers the common cases), and continue. It clears from the inbox when it is done, not by grooming.

An item that is not actionable or already resolved takes the `discard` route: remove it from `specs/inbox.md` with no other write.

### Setting the session target

When an item routes to an existing spec, set the session target to that spec as part of the routing, so the operator's next command (`/gov:amend`, `/gov:implement`) needs no manual `/gov:target`:

- **Spec edit (Step 3)** — target the matched feature: write `feature` and `path` (the repo-relative `specs/{feature}` directory).
- **Scenario creation (Step 4)** — target the matched feature **plus the new scenario**: write `feature`, `path`, `scenario` (the new scenario slug), and `scenario-path` (`specs/{feature}/scenarios/{slug}.md`) — the same target shape `/gov:amend`'s scenario route writes.
- **Rule items (Step 1), new-spec items (Step 2 → `/gov:specify`), chores, and discards** set **no** target — a rule file and a chore have no single spec home, and `/gov:specify` already targets the spec it creates.

Write the file the way every session-target write does: first read any existing `.govern/session.toml` to capture its `cli-config-dir` (the per-contributor agent identity written by `/govern`) and carry it forward, then rewrite the file atomically (tempfile + rename) with the target keys plus `set-at` (ISO 8601 UTC). Dropping `cli-config-dir` would strip the agent identity, so it is always preserved. On the runtime path the `write-session` primitive performs exactly this write.

Across a multi-item run, each spec-routed item performs this write, so the session target follows the current item and ends pointing at the most recently groomed spec.

### Reopening a `done` spec

When a Step 4 scenario is created under a matched spec whose status is `done`, reopen it `done → in-progress` as part of the same routing action — the documented back-edge §spec-lifecycle ("Backward via new scenario") defines, the same one `/gov:amend`'s scenario route performs. The new scenario carries an unimplemented task, so leaving the spec `done` would make the follow-on `/gov:implement` gate-fail; the reopen keeps status and on-disk content consistent.

- **Only from `done`.** A matched spec in `draft`, `clarified`, `planned`, or `in-progress` is left unchanged — groom never advances or otherwise moves a non-`done` status.
- **Guarded.** Confirm the on-disk status is still `done` immediately before writing; if it has changed since the match (a concurrent edit), surface the discrepancy rather than blindly overwriting — guarding the reopen against a lost concurrent update the way every status back-edge does. On the runtime path the `set-status` primitive's `from: done` argument enforces this guard.
- **No separate prompt, but the reopen is named.** When the matched spec is `done`, the per-item routing confirmation names the reopen alongside the scenario and target (see step 3 of the Instructions), so that single confirmation is the consent for the reopen and the operator approves the `done → in-progress` change before it happens — no separate prompt is added. The completion summary also names any spec reopened, so the status change is surfaced both before (at the confirmation) and after (in the summary).
- **The spec-edit route is out of scope.** It has its own back-edge (§spec-lifecycle, "Backward via meaningful body edit") and this scenario-reopen does not apply to it.
