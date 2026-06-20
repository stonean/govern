---
section: "Follow-on scenarios"
---

# Opencode-command-resolution

## Context

The [commands-dir-parameterization](commands-dir-parameterization.md) work made `gvrn exec` resolve an installed command file via the `.govern.toml` `[host]` block — `{cli-config-dir}/commands/{project}/<name>.md` — instead of a hardcoded `.claude/commands/gov/` path. That parameterized the config-dir name (`.claude` / `.augment`) and the project namespace, but left the `commands/` path segment a literal at both callsites (`main::run_exec` and `interpreter::payload::locate_command_file`).

[032-opencode-agent](../../032-opencode-agent/spec.md) adds OpenCode, whose command files live under a **singular** `command/` directory — `.opencode/command/{project}/<name>.md` (verified against the live CLI; commands namespace by subdirectory, so `command/gov/specify.md` registers as `gov/specify`). The literal `commands/` segment therefore never resolves for an OpenCode adopter: `gvrn exec <name>` builds `.opencode/commands/{project}/<name>.md` and fails with "command file not found." Spec 032 Decision 10 accepted this for the MVP — OpenCode ships on the markdown-only path, where the host reads the command file directly — and deferred runtime `exec` resolution to this 022 follow-up. The MCP tools are layout-independent and already work for OpenCode adopters; only `gvrn exec` is affected.

The host config carries only the agent's `cli-config-dir` and the `project` namespace; neither distinguishes the plural and singular layouts, and adding a layout signal would require new framework wiring. The runtime can instead resolve both known flat-namespaced layouts directly: the config-dir names are agent-specific (`.claude` / `.augment` for claude-style, `.opencode` for opencode), so the plural and singular candidates are mutually exclusive per adopter and trying both is unambiguous. (Where `cli-config-dir` is *read from* — the per-contributor `.govern.session.toml`, a legacy `.govern.toml` `[host]` fallback, or the default — is orthogonal to this layout detection; see [cli-config-dir-per-contributor](cli-config-dir-per-contributor.md).)

## Behavior

- A new method `Host::command_file_candidates(command_name)` (`runtime/src/host.rs`) MUST return the repo-relative installed command-file paths for both flat-namespaced layouts, in resolution order: plural `{cli-config-dir}/commands/{project}/<name>.md` (claude-style — Claude, Auggie) first, then singular `{cli-config-dir}/command/{project}/<name>.md` (opencode). The plural-first order keeps claude-style resolution byte-for-byte unchanged.
- Both resolution callsites — `main::run_exec` and `interpreter::payload::locate_command_file` — MUST build their candidate list through this method rather than formatting the installed path inline, so the layout set lives in one place and the two callsites cannot drift. The surrounding candidates are unchanged: `framework/commands/<name>.md` is tried first (the framework's own repo), the installed candidates next, and `framework/bootstrap/<name>.md` last (bootstrap procedures invoked before any framework files exist).
- `gvrn exec <name>` against an OpenCode adopter (resolved `cli_config_dir = ".opencode"`, `project = <name>`) MUST resolve `.opencode/command/{project}/<name>.md` and drive its procedure, exactly as it does for a claude-style adopter's `commands/` file.
- Command resolution needs no new host-config field for layout detection, and no framework or schema change — the layouts are told apart by trying both candidates. The change is additive and backward-compatible: a new public method on `Host` plus the two callsites' use of it.

## Edge Cases

- **Claude-style adopter (the common path)** — `cli-config-dir` is `.claude` or `.augment`; the plural candidate `{dir}/commands/{project}/<name>.md` exists and is tried first, so resolution is identical to pre-change behavior. The singular candidate is never reached.
- **OpenCode adopter** — `cli-config-dir` is `.opencode`; the plural candidate is absent (OpenCode never creates `commands/`), so the singular `{dir}/command/{project}/<name>.md` candidate resolves. The two directories are mutually exclusive per adopter, so there is no ambiguity about which file wins.
- **Both directories present for one adopter** — not a real layout (an agent installs into exactly one), but if it occurred the plural form wins by candidate order. Documented, not guarded.
- **Command absent from every candidate** — unchanged: `run_exec` prints "command file not found (tried …)" listing every candidate (now including the singular path) and exits non-zero; `locate_command_file` returns `None`.
- **Framework's own repo / bootstrap** — unaffected: `framework/commands/<name>.md` is still tried before the installed candidates and `framework/bootstrap/<name>.md` after them, so this repo's own `gvrn exec` and the `/govern` bootstrap procedure resolve as before.
- **Antigravity layout** — out of scope. Antigravity installs skills (`{dir}/skills/{project}-<name>/SKILL.md`), a dir-per-skill structure unlike the flat `<name>.md` namespacing both claude-style and opencode use; `gvrn exec` does not resolve it today and this change does not add it.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

- **Host-block field vs. try-both resolution.** Resolved: the runtime tries both the plural and singular candidates rather than adding a `command-subdir` field to the `[host]` block. The directories are agent-specific and mutually exclusive, so trying both is unambiguous, and it keeps the change runtime-only — no coordination with the framework-side spec-032 work (separate session) and no `[host]` schema growth. A field would only be warranted if a future layout reused an existing config-dir name with a different command subdir, which no current agent does.
