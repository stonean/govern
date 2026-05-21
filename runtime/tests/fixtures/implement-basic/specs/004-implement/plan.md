# 004 — Implement Fixture Plan

A tiny plan exercising the `writeCode` cache-anchor bundling: the table
below feeds `plan-relevant-files` and the canonical `implement.md`
file's `Reference:` line feeds `constitution-excerpts`.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `specs/004-implement/spec.md` | Edit | Update acceptance criteria as tasks complete. |
| `runtime/src/foo.rs` | Create | Stub module produced by the writeCode step. |

The runtime omits `runtime/src/foo.rs` from `plan-relevant-files`
because the file doesn't exist yet (planned-new file). The first row's
target — `specs/004-implement/spec.md` — does exist and is inlined.
