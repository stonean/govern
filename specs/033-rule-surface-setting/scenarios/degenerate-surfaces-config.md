---
section: "Setting"
---

# Degenerate-surfaces-config

## Context

An operator sets `[rules] surfaces` in `.govern.toml` to a degenerate value — either an explicitly empty list (`surfaces = []`) or a list containing a member that is not a recognized surface (e.g. `"fullstack"`, a typo). The Setting section defines the valid members (`"backend"`, `"frontend"`) and rejects `"cross"`, but does not define behavior for an empty list or an unrecognized member. Both cases are distinct from the key being *unset* (absent entirely), which falls back to 024-rule-loader's stack derivation.

## Behavior

- **`surfaces = []` (explicitly empty) is valid and means "cross-only".** No surface-suffixed rule files (`-backend.md`, `-frontend.md`) are selected; only `-cross.md` files apply. This is distinct from the key being *unset*, which falls back to 024 stack derivation — the empty list is the operator explicitly declaring "this project needs no surface rules, only cross-cutting ones."
- **An unrecognized member fails fast.** When `surfaces` contains a value outside the accepted set (`"backend"`, `"frontend"`), the command that reads the setting (`/govern`, `/gov:review`) errors immediately, naming the offending value and listing the accepted members, consistent with `CFG-ENV-003`'s fail-fast-on-invalid-configuration posture. The setting is never silently ignored, and the unknown member is never warn-and-continued.

## Edge Cases

- A list mixing valid and invalid members (`["backend", "fullstack"]`) fails fast on the invalid member; the presence of a valid member does not rescue the config.
- A non-list value (`surfaces = "backend"`, a bare string) is malformed configuration and fails fast the same way, naming the type mismatch.
- `"cross"` as a member remains rejected as the Setting section already specifies (cross-cutting files are unconditional, not a surface); its rejection follows the same fail-fast path.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
