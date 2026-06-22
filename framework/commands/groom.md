---
description: Walk the inbox and route each item to its proper home.
---

# Groom

Walk the inbox and route each item to its proper home.

## Purpose

Backlog grooming for `specs/inbox.md`. Walks each raw item through the bug decision tree and routes it to its proper home — a rule, a new spec, a spec edit, or a scenario. Not every item belongs to a spec: a **chore** (lint or formatting cleanup, dependency cleanup, repo hygiene — project maintenance with no feature home) is not spec material and is left in the inbox to be done directly, never forced into a spec, scenario, or `tasks.md` (§bug-handling, durability test). Removes items once they are migrated or resolved. Pairs with `/{project}:log`, which records items to the inbox without interpreting them.

## Context

Use the session target from `.govern.session.toml` if set, but groom operates across all specs so a target is not required.

## Scope Boundaries

- This command grooms inbox items — it creates scenario files and appends tasks but does NOT implement fixes. Do NOT read or modify source code or test files.
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
   - If the existing spec does not cover the reported behavior clearly — recommend updating the spec directly. Offer to help update the spec section.

   **Step 4: Spec is clear — durable requirement, or a chore?**
   - The spec covers the area, so there is no rule, missing-spec, or spec-edit gap. Now decide what the item *is* (§bug-handling, durability test):
     - **A durable behavioral requirement** — a new behavior, edge case, or contract the spec covers at a high level but does not yet describe in detail. Create a scenario inline under the matching spec's `scenarios/` directory using the `specs/templates/scenario.md` template, then append a task to the spec's `tasks.md` referencing the new scenario. (`/{project}:groom` keeps the inbox flow moving; for a deeper interactive walk through a single scenario, run `/{project}:amend` against the parent spec — the classifier routes the input to the scenario branch.)
     - **A chore** — project maintenance (lint or formatting cleanup, dependency cleanup, repo hygiene, a standalone refactor) that adds no durable requirement. It is **not** spec material: do not create a rule, spec, or scenario, and do not append it to any `tasks.md` — spec tasks are feature work derived from a plan, not standalone chores. Leave the item in `specs/inbox.md` (do **not** remove it), tell the user it is general maintenance to be done directly (the project's lint/format/test workflows cover the common cases), and continue. It clears from the inbox when it is done, not by grooming.

3. After migrating an item to a spec, scenario, or rule (or otherwise resolving it), remove it from `specs/inbox.md`. A chore recognized in Step 4 is the exception — leave it in place; it is resolved by being done, not by grooming.
4. **Wait for user confirmation before moving to the next item.** Do not proceed until the user approves.

### Completion

After all items are groomed:

- Report how many items were migrated, how many specs were created, how many scenarios were added, how many items were recognized as chores (left in the inbox to be done directly), and how many items remain.
- If `specs/inbox.md` is now empty (no items left), report: "Inbox is clean."
