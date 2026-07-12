---
description: Record a raw item to the inbox.
argument-hint: "[item text]"
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

> **For agent runtimes**: the Invoke steps below call the MCP tools of the optional gvrn runtime; the host-integration contract — bare↔prefixed tool names, lazy ToolSearch schema fetch, the no-shell-utilities rule, and the two-paths guarantee — lives once in the constitution, §runtime-host-integration. With no gvrn MCP server registered, walk the same prose using the host file-reading tools (Read, Edit, Write) per the Markdown-only reference below.

<!-- audit:ignore-promotion -->
1. Capture the item. If `$ARGUMENTS` is provided, treat it as the item text. Otherwise, ask the user: "What do you want to log?" Optionally ask follow-up questions if the item is so terse it would be unrecoverable later (e.g., "broken" with no context) — one short clarification is enough; do not interrogate.

2. Invoke `append-inbox` with the item text to append `- [ ] {item text}` as a new checkbox bullet to `specs/inbox.md` (the checkbox form the inbox template and constitution §bug-handling document — inbox items clear by being done and removed). The create-if-missing semantics live in the primitive: when the file does not exist, it is created before the append (from the project inbox template when one is on disk, else with a minimal `# Inbox` heading). The item is a single line — recording stays fast and uninterpreted. The result's `item-count` field carries the new inbox total (comment/fence-aware) for the report.

3. Invoke `lint-markdown` against the modified `specs/inbox.md`.

<!-- audit:ignore-promotion -->
4. Report: the line that was added; the new total item count in the inbox (`append-inbox`'s `item-count` result); and the suggested next step: "Run `/{project}:groom` when you're ready to walk the inbox and route items to their proper homes." **Stop here.** Do not start grooming or implementation. The user invokes `/{project}:groom` explicitly.

## Markdown-only reference

With no gvrn runtime registered, the host performs the same append with its own file tools (Read, Edit, Write) — no shell-pipeline substitution (§runtime-host-integration):

1. If `specs/inbox.md` does not exist, create it with a minimal heading (`# Inbox`) followed by a blank line.
2. Append the item to the inbox list as a new checkbox bullet:

   ```markdown
   - [ ] {item text}
   ```

3. Run `npx markdownlint-cli2` on the modified file.

Either path ends with the same report: the line that was added, the new total item count in the inbox, and the `/{project}:groom` pointer — and stops there.
