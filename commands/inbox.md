# Inbox

Review and migrate items from the inbox.

## Purpose

Walks each item in `specs/inbox.md` through the bug decision tree, migrates items to the appropriate spec or scenario, and removes resolved items from the inbox list.

## Context

Use the session target from `{cli-config-dir}/{project}-session.json` if set, but inbox operates across all specs so a target is not required.

## Scope Boundaries

- This command processes inbox items — it creates scenario files and appends tasks but does NOT implement fixes. Do NOT read or modify source code or test files.
- For each item, read only the spec file of the matching feature (for decision tree evaluation) and its `tasks.md` (for appending). Do NOT read plans, data models, or source code.
- Reference: §bug-handling, §scenarios, §brownfield-inbox (constitution loaded by `/{project}:target` — do not re-read).

## Instructions

### Check for inbox file

1. Check if `specs/inbox.md` exists.
   - If it does not exist, stop and report: "No inbox file found at `specs/inbox.md`. Nothing to process."
2. Read `specs/inbox.md`.
   - If the file has no items (only headings and comments), report: "Inbox is clean — no items to process." Keep the file to preserve git history.

### Process each item

Process items **one at a time**. Do not batch or pre-process multiple items. Complete the full decision tree for one item, get user confirmation, then move to the next.

For each item in the inbox list:

1. Display the item number, total remaining count, and item description.
2. Walk the bug decision tree:

   **Step 1: Does a spec exist for this behavior?**
   - Search `specs/` for a feature directory that covers this area.
   - If no spec exists — recommend creating one via `/{project}:capture`. Ask the user whether to create the spec now or skip this item.

   **Step 2: Is the spec ambiguous or incomplete?**
   - If the existing spec does not cover the reported behavior clearly — recommend updating the spec directly. Offer to help update the spec section.

   **Step 3: Is the spec clear but needs a scenario?**
   - If the spec covers the area but the specific behavior needs lower-level elaboration — create a scenario under the matching spec's `scenarios/` directory using the scenario template.
   - Append a task to the spec's `tasks.md` referencing the new scenario.

3. After migrating or resolving the item, remove it from `specs/inbox.md`.
4. **Wait for user confirmation before moving to the next item.** Do not proceed until the user approves.

### Completion

After all items are processed:

- Report how many items were migrated, how many specs were created, and how many items remain.
- If `specs/inbox.md` is now empty (no items left), report: "Inbox is clean."
