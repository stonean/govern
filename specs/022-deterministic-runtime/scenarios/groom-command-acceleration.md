---
section: "Follow-on scenarios"
---

# Groom-command-acceleration

## Context

The spec's third deferred follow-on, never shipped: `/gov:groom` remains legacy prose with zero primitive references, and — worse than mere non-acceleration — its prose re-specifies long-hand, in its own words, four writes that already exist as primitives and that `/gov:amend` routes through them: scenario creation from template (`create-scenario`), task append (`append-task`), the done→in-progress reopen with concurrent-edit guard (`set-status`), and the session-target write with cli-config-dir preservation (`write-session`). Two divergent specifications of the same writes is exactly the drift §drift-prevention exists to prevent. The routing decision per inbox item is the semantic core and maps to the `routeInboxItem` extension point (typed builder ships in extension-request-hygiene).

## Behavior

`/gov:groom`'s Instructions are rewritten to the parseable conventions: the inbox walk reads `specs/inbox.md`, each item's routing decision is the `routeInboxItem` extension seam (spec / scenario / rule / chore-stays / discard, per the decision tree), and the mechanical consequences invoke the same primitives amend uses — `create-scenario` + `append-task` for a scenario route, `set-status` for a done-spec reopen, `write-session` where the target changes — plus the item's removal from the inbox and the completion-count summary. The file parses cleanly, leaves `legacy-prose-commands.txt`, and the markdown-only reference names the same operations as fallback prose (one contract, two paths).

## Edge Cases

- A chore item is left in place (no write, no route) exactly as the constitution's inbox rules require.
- An item routing to a spec that does not exist directs the user to `/gov:specify` and moves on, as today.
- Inbox item removal remains a host edit (`Edit` on `specs/inbox.md`) until the scaffolding-primitives scenario's `append-inbox`/inbox tooling covers removal — prose stays the owner of that line-level edit on both paths.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
