---
section: "Follow-on scenarios"
---

# Cli-config-dir-per-contributor

## Context

The [commands-dir-parameterization](commands-dir-parameterization.md) work put both `cli-config-dir` and `project` in a committed `.govern.toml` `[host]` block. That is wrong for `cli-config-dir`: it names the agent's config directory (`.claude` / `.augment` / `.opencode` / `.agents`), which is a **per-contributor** choice. Teammates on one project may each use a different agent, so committing one contributor's value bakes their agent choice into shared config — every other contributor then resolves command files against the wrong directory (or `gvrn exec` fails outright). `project` has no such problem: it's the slash-command namespace, identical for every contributor, and stays committed.

The fix relocates `cli-config-dir` to the gitignored, per-contributor `.govern.session.toml` (the same file that already holds the per-user session target). `project` stays in committed `.govern.toml`. Because the session file is wholesale-rewritten on every `/{project}:target`, and `cli-config-dir` is set once at `/govern` time (before any target exists), the writer must merge rather than overwrite, and every reader must tolerate a session file that has `cli-config-dir` but no target (and vice-versa).

## Behavior

- `crate::host::Host::load` MUST resolve `cli_config_dir` in priority order: the per-contributor `.govern.session.toml`, then the legacy `.govern.toml` `[host]` value (for adopters predating the relocation), then the default `.claude`. `project` continues to load from `.govern.toml` `[host]` (then the repo-directory-basename default). A malformed `.govern.session.toml` is treated as absent so resolution falls through.
- `write-session` becomes a **merge-writer** over `.govern.session.toml`, with all on-disk keys optional:
  - A *target write* (`feature`+`path` supplied, optional `scenario`) sets the target block and a fresh `set-at`, and **preserves** any existing `cli-config-dir` unless one is supplied in the same call.
  - A *host-config write* (`cli-config-dir` supplied, no `feature`) sets `cli-config-dir` and **preserves** the existing target block verbatim. Against a fresh repo it writes a file containing only `cli-config-dir`.
  - `feature` and `path` must be supplied together; `scenario` (with `scenario-path`) requires a target write; a call supplying neither `feature` nor `cli-config-dir` is rejected.
- `dashboard`'s session reader MUST treat `feature` as optional: a session file carrying only `cli-config-dir` (no target yet) reports `session-target: null`, not a parse error.
- `/govern` (§Instructions step 6) MUST write `project` to the committed `.govern.toml` `[host]` block (dropping any legacy `cli-config-dir` the managed block previously carried) and write `cli-config-dir` to `.govern.session.toml` via a host-config write. On the markdown-only path the host performs both writes directly.
- `/{project}:target` MUST preserve `cli-config-dir` across a target switch (the merge-writer does this on the runtime path; on the markdown-only path the host reads the existing value and writes it back). `/{project}:target --clear` MUST preserve `cli-config-dir`: it rewrites the file to contain only that key when present, and deletes the file only when no `cli-config-dir` is recorded.
- Migration is self-healing, no dedicated primitive: the legacy `.govern.toml` `[host]` fallback keeps existing adopters resolving until their next `/govern`, which drops `cli-config-dir` from the committed managed block and records it in the session file.

## Edge Cases

- **Mixed-agent team.** Contributor A (Claude) and contributor B (OpenCode) share one repo. Each runs `/govern`; each writes their own `.govern.session.toml` `cli-config-dir` (`.claude` vs `.opencode`). The committed `.govern.toml` `[host]` carries only `project`, identical for both. Neither clobbers the other.
- **Legacy adopter, not yet re-run.** `.govern.toml` `[host]` still has `cli-config-dir`; no session value yet. The legacy fallback resolves it, unchanged, until the next `/govern` relocates it.
- **Target switch.** A `/{project}:target` rewrite changes feature/path/scenario/set-at and preserves `cli-config-dir`.
- **`--clear` with an agent identity recorded.** The session file is rewritten to contain only `cli-config-dir` (target cleared, agent identity kept) so `gvrn exec` keeps resolving command files; `--clear` with no recorded `cli-config-dir` deletes the file as before.
- **Host-config write on a fresh repo.** `/govern` setting `cli-config-dir` before any target writes a session file containing only that key; `dashboard` reports no target.
- **Malformed session file.** `Host::load` and the `write-session` preserve-read both treat it as absent (best-effort) rather than failing command resolution or a target switch.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

- **Setter mechanism — extend `write-session` vs. a new primitive.** Resolved: extend `write-session` into a merge-writer (option A). It is already the sole writer of `.govern.session.toml`, so it owns all the file's keys; a dedicated `set-host-config` primitive would duplicate that surface (schema, MCP, CLI, dispatch, registration) for no gain. The contract widens (`feature`/`path` optional, new `cli-config-dir`, merge semantics) but existing callers that always pass `feature`+`path` are unaffected.
- **Does `project` move too?** Resolved: no. `project` is the shared slash-command namespace — identical for every contributor — so it stays in committed `.govern.toml`. Only `cli-config-dir` is per-contributor.
