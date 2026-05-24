---
section: "Command Set"
---

# Target-clear-flag

## Context

`/gov:target` (Behavior → Command Set → Utility commands) sets the working feature for the session by writing `.govern.session.toml`. After a spec advances to `done`, the session still points at the now-completed feature (and optionally scenario), producing awkward `/gov:status` output ("Target: 000-slash-commands / done / next: done (spec is complete)") and trapping `/gov:ask` and `/gov:implement` on stale state until the user manually picks a new target.

The only reset path today is `/gov:target <other-feature>`, which mutates the session toward a different target rather than clearing it. There is no first-class "no target" state reachable through the command — even though the `dashboard` primitive already handles `session-target: null` and the status renderer already prints "No session target. Run /gov:target to select one." when that's the case (per `framework/commands/status.md` step 2).

The user-visible gap: closing out a spec leaves the session pointer dangling, and clearing it requires hand-editing `.govern.session.toml`.

## Behavior

`/gov:target` accepts a `--clear` flag (mutually exclusive with a feature argument). When set:

- Remove `.govern.session.toml` (delete the file). The `dashboard` primitive's documented "Session file absent → session-target: null" behavior is the reset state — there's no separate empty-session format to invent.
- Emit a one-line confirmation: `Session cleared. Run /gov:target to set a new target.`
- Exit 0.

Mutually exclusive with positional arguments and other flags. Invoking `--clear` alongside a feature argument halts with `/gov:target: --clear cannot be combined with a feature argument`; alongside a scenario flag (when scenario-targeting is supported), halt analogously.

## Edge Cases

- **Session file already absent.** `--clear` is a no-op delete but still emits the confirmation line and exits 0. Idempotent.
- **Session file present but malformed JSON.** `--clear` removes it cleanly; do not error on stale state.
- **Permission denied on delete.** Surface the OS error and exit non-zero — same envelope shape other session-file writes use today.
- **`--clear` combined with feature argument or scenario flag.** Halt with the mutex-violation message above; no session mutation.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
