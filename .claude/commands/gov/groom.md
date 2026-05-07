---
description: Walk the inbox and route each item to its proper home.
---

# Groom

Walk the inbox and route each item to its proper home.

## Purpose

Backlog grooming for `specs/inbox.md`. Walks each raw item through the bug decision tree and migrates it to the appropriate spec, scenario, or spec edit. Removes resolved items from the inbox. Pairs with `/gov:log`, which records items to the inbox without interpreting them.

## Context

Use the session target from `.claude/gov-session.json` if set, but groom operates across all specs so a target is not required.

## Scope Boundaries

- This command grooms inbox items — it creates scenario files and appends tasks but does NOT implement fixes. Do NOT read or modify source code or test files.
- For each item, read only the spec file of the matching feature (for decision tree evaluation) and its `tasks.md` (for appending). Do NOT read plans, data models, or source code.
- Reference: §bug-handling, §rules, §scenarios, §brownfield-inbox (constitution loaded by `/gov:target` — do not re-read).

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
   - If a loaded rule file already covers the domain (e.g., `specs/security-backend.md` for an authentication concern, `specs/configuration.md` for an env-var concern), recommend the user amend the relevant rule file directly — note that local edits to rule files are overwritten by `/govern` unless the file is pinned in `.govern.toml`, so amendments belong upstream in the framework rather than in adopting projects.
   - If no rule file covers the domain, creating a new rule file is its own feature spec (out of `/gov:groom`'s scope). Leave the item in the inbox unmodified — every subsequent groom pass walks every unmigrated item, including this one, so it stays surfaced. Ask the user whether to skip and continue.
   - If the item is feature-specific rather than cross-cutting, fall through to Step 2.

   **Step 2: Does a spec exist for this behavior?**
   - Search `specs/` for a feature directory that covers this area.
   - If no spec exists — recommend creating one via `/gov:capture`. Ask the user whether to create the spec now or skip this item.

   **Step 3: Is the spec ambiguous or incomplete?**
   - If the existing spec does not cover the reported behavior clearly — recommend updating the spec directly. Offer to help update the spec section.

   **Step 4: Is the spec clear but needs a scenario?**
   - If the spec covers the area but the specific behavior needs lower-level elaboration — create a scenario inline under the matching spec's `scenarios/` directory using the `specs/templates/scenario.md` template, then append a task to the spec's `tasks.md` referencing the new scenario. (`/gov:groom` keeps the inbox flow moving; for a deeper interactive walk through a single scenario, run `/gov:elaborate` separately.)

3. After migrating or resolving the item, remove it from `specs/inbox.md`.
4. **Wait for user confirmation before moving to the next item.** Do not proceed until the user approves.

### Completion

After all items are groomed:

- Report how many items were migrated, how many specs were created, and how many items remain.
- If `specs/inbox.md` is now empty (no items left), report: "Inbox is clean."
