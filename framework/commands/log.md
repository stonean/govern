---
description: Record a raw item to the inbox.
---

# Log

Record a raw item to the inbox.

## Purpose

Append an item to `specs/inbox.md` for later grooming. Use this when a bug, observation, or open issue surfaces and you want to capture it without breaking flow. The item stays raw until `/{project}:groom` walks it through the bug decision tree and routes it to a spec, scenario, or spec edit.

## Context

This command does not require a session target — items in the inbox span the whole project. If `$ARGUMENTS` is provided, use it as the item text. If empty, ask the user what to log.

## Scope Boundaries

- This command only appends a single line to `specs/inbox.md`. Do NOT modify any other file. Do NOT read or write source code, test files, specs, plans, or scenarios.
- Do NOT walk the decision tree, classify the item, or suggest a spec — that is `/{project}:groom`'s job. Keep the recording step fast and uninterpreted.
- Reference: §brownfield-inbox (constitution loaded by `/{project}:target` — do not re-read).

## Instructions

### Capture the item

1. If `$ARGUMENTS` is provided, treat it as the item text. Otherwise, ask the user: "What do you want to log?"
2. Optionally ask follow-up questions if the item is so terse it would be unrecoverable later (e.g., "broken" with no context). One short clarification is enough — do not interrogate.

### Append to inbox.md

1. If `specs/inbox.md` does not exist, create it with a minimal heading (`# Inbox`) followed by a blank line.
2. Append the item to the inbox list as a new bullet:

   ```markdown
   - {item text}
   ```

3. Run `npx markdownlint-cli2` on the modified file.

### Report

Display:

- The line that was added.
- The new total item count in the inbox.
- Suggested next step: "Run `/{project}:groom` when you're ready to walk the inbox and route items to their proper homes."
- **Stop here.** Do not start grooming or implementation. The user invokes `/{project}:groom` explicitly.
