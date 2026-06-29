---
description: Walk the inbox and route each item to its proper home.
---

# Groom

Walk the inbox and route each item to its proper home.

## Purpose

Backlog grooming for `specs/inbox.md`. Walks each raw item through the bug decision tree and routes it to its proper home — a rule, a new spec, a spec edit, or a scenario. Not every item belongs to a spec: a **chore** (lint or formatting cleanup, dependency cleanup, repo hygiene — project maintenance with no feature home) is not spec material and is left in the inbox to be done directly, never forced into a spec, scenario, or `tasks.md` (§bug-handling, durability test). Removes items once they are migrated or resolved. Pairs with `/{project}:log`, which records items to the inbox without interpreting them.

## Context

Use the session target from `.govern.session.toml` if set, but groom operates across all specs so a target is not *required* to start. When groom routes an item to an existing spec — a spec edit (Step 3) or a scenario under the matching spec (Step 4) — it **sets** the session target to that spec as part of the routing, so a follow-on `/{project}:amend` or `/{project}:implement` needs no manual `/{project}:target`. Adding a scenario to a `done` spec (Step 4) also **reopens** it `done → in-progress` so that follow-on `/{project}:implement` has an actionable target. See **Setting the session target** and **Reopening a `done` spec** below.

## Scope Boundaries

- This command grooms inbox items — it creates scenario files, appends tasks, sets the session target (`.govern.session.toml`) when routing to an existing spec, and reopens a matched spec's frontmatter status `done → in-progress` when it adds a scenario to a `done` spec (Step 4), but does NOT implement fixes. Do NOT read or modify source code or test files.
- For each item, read only the spec file of the matching feature (for decision tree evaluation) and its `tasks.md` (for appending). Do NOT read plans, data models, or source code.
- Reference: §bug-handling, §rules, §scenarios, §brownfield-inbox (constitution loaded by `/{project}:target` — do not re-read).

## Instructions

### Check for inbox file

1. Check if `specs/inbox.md` exists.
   - If it does not exist, stop and report: "No inbox file found at `specs/inbox.md`. Nothing to groom."
2. Read `specs/inbox.md`.
   - If the file has no list items (no lines beginning with `-` or `*` outside HTML comments), report: "Inbox is clean — no items to groom." Keep the file to preserve git history.

### Groom each item

Process items **one at a time**. Do not batch or pre-process multiple items. Complete the full decision tree for one item, get user confirmation, then move to the next.

For each item in the inbox list:

1. Display the item number, total remaining count, and item description.
2. Walk the bug decision tree:

   **Step 1: Is this a cross-cutting concern with no covering rule?**
   - Apply the four-indicator promotion checklist (§rules in `constitution.md`): cross-cutting, citable, governance-recognized category, generalizable wording. If the item qualifies, recommend promoting it to a rule.
   - If a loaded rule file already covers the domain (e.g., `specs/rules/security-backend.md` for an authentication concern, `specs/rules/configuration-cross.md` for an env-var concern), recommend the user amend the relevant rule file directly — note that local edits to rule files are overwritten by `/govern` unless the file is pinned in `.govern.toml`, so amendments belong upstream in the framework rather than in adopting projects.
   - If no rule file covers the domain, creating a new rule file is its own feature spec (out of `/{project}:groom`'s scope). Leave the item in the inbox unmodified — every subsequent groom pass walks every unmigrated item, including this one, so it stays surfaced. Ask the user whether to skip and continue.
   - If the item is feature-specific rather than cross-cutting, fall through to Step 2.

   **Step 2: Does a spec exist for this behavior?**
   - Search `specs/` for a feature directory that covers this area.
   - If no spec exists — recommend creating one via `/{project}:specify`. Ask the user whether to create the spec now or skip this item. (If the item is a chore rather than a feature gap — see Step 4 — it never needs a spec; leave it in the inbox.)

   **Step 3: Is the spec ambiguous or incomplete?**
   - If the existing spec does not cover the reported behavior clearly — recommend updating the spec directly. Offer to help update the spec section. When the spec edit is confirmed, **set the session target** to this feature (see **Setting the session target**).

   **Step 4: Spec is clear — durable requirement, or a chore?**
   - The spec covers the area, so there is no rule, missing-spec, or spec-edit gap. Now decide what the item *is* (§bug-handling, durability test):
     - **A durable behavioral requirement** — a new behavior, edge case, or contract the spec covers at a high level but does not yet describe in detail. Create a scenario inline under the matching spec's `scenarios/` directory using the `specs/templates/scenario.md` template, then append a task to the spec's `tasks.md` referencing the new scenario. **If the matched spec's status is `done`, reopen it `done → in-progress`** as part of the same routing action (see **Reopening a `done` spec**), so a follow-on `/{project}:implement` finds an actionable target instead of gate-failing on a `done` spec; a spec already in `draft`, `clarified`, `planned`, or `in-progress` is left unchanged. After the scenario and task are written and any reopen is applied, **set the session target** to this feature plus the new scenario (see **Setting the session target**). (`/{project}:groom` keeps the inbox flow moving; for a deeper interactive walk through a single scenario, run `/{project}:amend` against the parent spec — the classifier routes the input to the scenario branch.)
     - **A chore** — project maintenance (lint or formatting cleanup, dependency cleanup, repo hygiene, a standalone refactor) that adds no durable requirement. It is **not** spec material: do not create a rule, spec, or scenario, and do not append it to any `tasks.md` — spec tasks are feature work derived from a plan, not standalone chores. Leave the item in `specs/inbox.md` (do **not** remove it), tell the user it is general maintenance to be done directly (the project's lint/format/test workflows cover the common cases), and continue. It clears from the inbox when it is done, not by grooming.

3. After migrating an item to a spec, scenario, or rule (or otherwise resolving it), remove it from `specs/inbox.md`. A chore recognized in Step 4 is the exception — leave it in place; it is resolved by being done, not by grooming.
4. **Wait for user confirmation before moving to the next item.** When the item routes to an existing spec, this confirmation names the target it will set — e.g. *"Create a scenario under `NNN-slug` and set it as the session target? (Y/n)"* — so the single confirmation is the consent for both the routing and the target write; no separate target prompt is added. **When the matched spec is `done`** (a Step 4 scenario route, which reopens it), the confirmation also names the reopen — e.g. *"Create a scenario under `NNN-slug`, reopen it to `in-progress`, and set it as the session target? (Y/n)"* — so the operator consents to the `done → in-progress` change before it happens (see **Reopening a `done` spec**). When the matched spec is not `done`, there is no reopen to name and the confirmation is unchanged. Do not proceed until the user approves.

### Setting the session target

When an item routes to an existing spec, set the session target to that spec as part of the routing, so the operator's next command (`/{project}:amend`, `/{project}:implement`) needs no manual `/{project}:target`:

- **Step 3 (spec edit)** — target the matched feature: write `feature` and `path` (the repo-relative `specs/{feature}` directory).
- **Step 4 (scenario creation)** — target the matched feature **plus the new scenario**: write `feature`, `path`, `scenario` (the new scenario slug), and `scenario-path` (`specs/{feature}/scenarios/{slug}.md`) — the same target shape `/{project}:amend`'s scenario route writes.
- **Step 1 (rule item), Step 2 (new spec → `/{project}:specify`), and the Step 4 chore branch** set **no** target — a rule file and a chore have no single spec home, and `/{project}:specify` already targets the spec it creates.

Write the file the way every session-target write does: first read any existing `.govern.session.toml` to capture its `cli-config-dir` (the per-contributor agent identity written by `/govern`) and carry it forward, then rewrite the file atomically (tempfile + rename) with the target keys plus `set-at` (ISO 8601 UTC). Dropping `cli-config-dir` would strip the agent identity, so it is always preserved.

Across a multi-item run, each spec-routed item performs this write, so the session target follows the current item and ends pointing at the most recently groomed spec.

### Reopening a `done` spec

When a Step 4 scenario is created under a matched spec whose status is `done`, reopen it `done → in-progress` as part of the same routing action — the documented back-edge §spec-lifecycle ("Backward via new scenario") defines, the same one `/{project}:amend`'s scenario route performs. The new scenario carries an unimplemented task, so leaving the spec `done` would make the follow-on `/{project}:implement` gate-fail; the reopen keeps status and on-disk content consistent.

- **Only from `done`.** A matched spec in `draft`, `clarified`, `planned`, or `in-progress` is left unchanged — groom never advances or otherwise moves a non-`done` status.
- **Guarded.** Confirm the on-disk status is still `done` immediately before writing; if it has changed since the match (a concurrent edit), surface the discrepancy rather than blindly overwriting — guarding the reopen against a lost concurrent update the way every status back-edge does.
- **No separate prompt, but the reopen is named.** When the matched spec is `done`, the per-item routing confirmation names the reopen alongside the scenario and target (see the Step 4 confirmation in **Groom each item**), so that single confirmation is the consent for the reopen and the operator approves the `done → in-progress` change before it happens — no separate prompt is added. The Completion summary also names any spec reopened, so the status change is surfaced both before (at the confirmation) and after (in the summary).
- **Step 3 is out of scope.** The spec-edit route has its own back-edge (§spec-lifecycle, "Backward via meaningful body edit") and this scenario-reopen does not apply to it.

### Completion

After all items are groomed:

- Report how many items were migrated, how many specs were created, how many scenarios were added, how many items were recognized as chores (left in the inbox to be done directly), and how many items remain.
- Name any spec reopened `done → in-progress` during the run (per **Reopening a `done` spec**), so the status change is surfaced rather than silent.
- Report the resulting session target — the spec (and scenario, if any) set by the most recently groomed spec-routed item, or "session target unchanged" when no groomed item set one.
- If `specs/inbox.md` is now empty (no items left), report: "Inbox is clean."
