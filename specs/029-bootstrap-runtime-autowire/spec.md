---
status: done
dependencies: [003-bootstrap-automation, 021-runtime-boundary, 022-deterministic-runtime, 028-antigravity-agent]
review:
  last-run: 2026-06-11T23:43:10Z
  reviewed-against: 7afcdb87de30b7fb97e62367eb4d55f16b047639
  must-violations: 0
  should-violations: 0
  low-confidence: 2
  blocking: false
---

# 029 — Bootstrap Runtime Auto-Detect and Wire

`/govern` should detect the `gvrn` runtime at the start of a bootstrap run, wire its MCP server into the project when the binary is present but unregistered, and tell the user that doing so reduces token use. The goal is to stop first-time adoptions from exhausting their token budget on the markdown-only reference path when a deterministic path was actually available.

## Motivation

On a first-time adoption, the bootstrap walks the markdown-only reference path — `curl`, `tar`, byte-compares, and a hand-authored scaffold script — because the [gvrn runtime](../022-deterministic-runtime/spec.md) is not registered for the agent session. The model interprets a ~20k-token procedure step by step and can run out of tokens before the install finishes. This was observed with the [antigravity layout](../028-antigravity-agent/spec.md).

The failure is not that `gvrn` is missing — the binary may already be installed globally and on `PATH`. The failure is that [`/govern`](../003-bootstrap-automation/spec.md) deliberately does **not** scaffold the MCP wiring (it is documented as a separate, manual install), so the agent session starts with no `gvrn` MCP tools and has no deterministic path to take even though one was one config file away.

MCP servers are loaded at session start. Writing the wiring file mid-run therefore cannot help the running session — only the next one. So the fix is not "use the runtime now"; it is "wire the runtime, tell the user, and let the next session run cheaply." This is the same shape as the existing **govern.md Self-Update Check**, which already writes a fresh file and aborts with a "start a new session and re-run" instruction. Both are restart-requiring pre-flight concerns, so this feature unifies them into a single pre-flight phase (see **Pre-flight Phase**) — the user restarts at most once even when both fire.

The markdown-only path remains first-class per [§runtime-boundary](../021-runtime-boundary/spec.md): when `gvrn` is genuinely absent, the agent still walks the prose. This feature only changes what happens when the binary exists.

## Pre-flight Phase

`gvrn` detection and the existing **govern.md Self-Update Check** run as a single pre-flight phase — after the cheap **Permission Setup** seed (so the binary probe is pre-authorized) and before pre-run migrations and the archive fetch. Both checks can independently require the session to restart: the self-update check to load a fresh `govern.md`, gvrn detection to load the MCP server. The pre-flight phase runs both checks, accumulates every restart-requiring write (a fresh `govern.md` and/or the gvrn wiring and permission entries), and aborts **once** with a single combined message if anything was written. If neither check requires a restart, the run proceeds past pre-flight normally. This guarantees the user restarts at most once regardless of how many pre-flight concerns fired.

Within pre-flight, `gvrn` detection resolves to exactly one of three states.

### Detection mechanism

Two independent probes resolve the state:

- **Tool-inventory introspection (State A).** The agent inspects its own available-tool inventory for any `gvrn`-namespaced MCP tool — `mcp__gvrn__*` on Claude Code, `mcp:gvrn:*` on Auggie and antigravity — counting deferred or lazily-loaded tool names as present. This needs no shell and no permission; the agent always knows its own tools. Any match is State A.
- **Binary probe (State B vs. State C).** Only when introspection finds no `gvrn` tool, a `command -v gvrn`-equivalent shell probe distinguishes an installed-but-unregistered binary (State B) from an absent one (State C). There is no non-shell way to detect this — a tool that could answer "is gvrn installed" would itself put the session in State A. The probe command is seeded into the **Permission Setup** pre-grant so routine runs do not prompt for it.

If the host cannot run the binary probe, or the user denies it, the run is classified as State C. A false State C is harmless — it costs nothing beyond today's behavior (no auto-wire; manual wiring stays documented) — so the gate never hard-fails on a locked-down host.

### State A — runtime live this session

The `gvrn` MCP tools are callable in the current session. The bootstrap takes the deterministic primitive path for the rest of the run and emits no detection message.

A host that lists MCP tool schemas lazily (e.g., Claude Code surfaces tool names in a deferred-tool reminder before exposing their schemas) is still in State A — the runtime is registered. The gate must treat lazily-listed `gvrn` tools as live, not absent, and fetch the schema through the host's mechanism rather than falling through to a lower state.

### State B — binary present, not wired

The `gvrn` binary is discoverable (a `command -v gvrn`-equivalent probe succeeds) but no `gvrn` MCP tools are available to the session. The bootstrap, in order:

1. Writes the per-layout MCP-wiring file additively (see **MCP Wiring**).
2. Adds the permission entries the agent needs to call the `gvrn` tools (see **Permission Setup**).
3. Contributes to the single pre-flight abort: the combined message tells the user that `gvrn` was detected and wired, that it reduces token use, and that they should start a new session and re-run `/govern`, and it **names every file it wrote** (the wiring file and the settings file) — alongside any stale-`govern.md` notice the self-update check produced in the same phase.
4. No archive is fetched and no scaffolding runs this session — the pre-flight abort happens before the archive fetch.

The only writes State B performs are the additive MCP-wiring entry and the additive permission entries; these join any fresh-`govern.md` write from the self-update check in the one pre-flight abort, so State B never triggers a restart of its own beyond that shared one.

State B issues **no separate consent prompt** before writing. The writes are additive and idempotent, matching the existing silent **Permission Setup** writes and the procedure's no-extra-prompts rule; the abort message's file list is the disclosure, and the user reverses any change through git (for tracked files) or by deleting the additive `gvrn` entry. There is no opt-out flag for auto-wiring — it is deliberately out of scope for this spec.

### State C — binary absent

The `command -v gvrn`-equivalent probe fails — or cannot run because the host grants no shell or the user denies it. The bootstrap proceeds on the markdown-only path exactly as it does today, and emits one tip line noting that installing `gvrn` reduces token use, pointing at the README Runtime section.

## MCP Wiring

The wiring file is the per-layout path already named in the bootstrap's §Derived values: `.mcp.json` at the repo root for the `claude-style` layout, `{config_dir}/mcp_config.json` for the `antigravity` layout. The entry registers the server as `gvrn` running the runtime's MCP subcommand.

The write **updates an existing configuration in place — it never replaces or truncates the file.** It is additive and idempotent:

- An existing wiring file with other `mcpServers` entries keeps every one of them, and every other top-level key; only a missing `gvrn` entry is added under `mcpServers`.
- An existing wiring file that already contains a `gvrn` entry is left unchanged — re-running does not duplicate or overwrite it.
- An existing wiring file that has no `mcpServers` key gets the key added with just the `gvrn` entry, preserving all other top-level keys.
- A missing wiring file is created containing just the `gvrn` entry.
- An existing wiring file that is **not valid JSON is left untouched** — the gate skips wiring, warns the user to repair it, and degrades to the markdown path for this run. A hand-maintained config is never clobbered. This is consistent with the "detection never hard-fails" principle: an unwritable config is treated like State C.

Reversal of the wiring follows the file's tracking status per layout: `claude-style`'s repo-root `.mcp.json` is conventionally tracked, so `git restore` applies; `antigravity`'s `{config_dir}/mcp_config.json` lives under the gitignored `.agents/`, so the user reverses it by deleting the additive `gvrn` entry.

Because the write is idempotent, State B is self-limiting: once it has run, the next session is either State A (the entry took effect) or — if the binary was removed between runs — State C.

Wiring is triggered by the binary's presence alone; the gate performs **no version-compatibility check**. This follows [spec 022 §Versioning enforcement](../022-deterministic-runtime/spec.md), which deliberately rejects startup version comparison and refuse-on-mismatch in favor of lockstep tagged releases plus the runtime's own parse-error path. A too-old wired `gvrn` surfaces through that runtime parse error on the next session, and any failing primitive falls back to the markdown path — so the gate has no reason to second-guess the binary's version.

## Permission Setup

When the wiring file is written, the agent's settings must also grant permission to call the `gvrn` MCP tools, so the next session can use the deterministic path without a permission prompt per tool. These entries are added to the same per-layout settings file the existing **Permission Setup** section manages, using the same additive rules: add only missing entries, never remove, reorder, or overwrite entries the user or `/{project}:configure` previously added.

Separately, the binary probe used by pre-flight detection (a `command -v gvrn`-equivalent) is added to the bootstrap's existing pre-grant seed of bootstrap shell commands, so the State B/State C probe does not prompt on routine runs. This seed runs before pre-flight (see **Pre-flight Phase**).

## Documentation

The README Runtime section, which currently frames the runtime as an entirely separate manual install that `/govern` does not touch, is updated to state that `/govern` auto-wires `gvrn` into the project when the binary is detected. The manual binary-install instructions remain (the binary itself is still installed out of band); only the MCP-registration step becomes automatic.

## Acceptance Criteria

- [x] When `gvrn` MCP tools are callable in the session (State A), `/govern` takes the deterministic primitive path and emits no detection message.
- [x] `gvrn` tools exposed lazily by the host (deferred schemas, not-yet-loaded) are classified as State A, not as absent.
- [x] State A is reached by introspecting the agent's own tool inventory (per-host `gvrn` tool prefix) — no shell probe and no added permission are required to detect State A.
- [x] When the binary probe cannot run (no shell granted) or is denied, the run is classified as State C — detection never hard-fails.
- [x] The `command -v gvrn`-equivalent probe command is part of the bootstrap's pre-granted shell-command seed, so it does not prompt on routine runs.
- [x] When the `gvrn` binary is discoverable but no `gvrn` MCP tools are available (State B), `/govern` writes the per-layout MCP-wiring file (`.mcp.json` for `claude-style`, `{config_dir}/mcp_config.json` for `antigravity`).
- [x] In State B, the write preserves every pre-existing `mcpServers` entry and is idempotent — a wiring file that already contains a `gvrn` entry is left byte-unchanged.
- [x] The gate wires `gvrn` on presence of the binary alone and performs no version-compatibility check (deferred to the runtime per spec 022 §Versioning enforcement).
- [x] An existing wiring file with no `mcpServers` key gains the key with just the `gvrn` entry, preserving all other top-level keys.
- [x] An existing wiring file that is not valid JSON is left untouched — the gate skips wiring, warns the user, and degrades to the markdown path (no clobber).
- [x] In State B, `/govern` aborts before the archive fetch and instructs the user to start a new session and re-run, stating that `gvrn` reduces token use and naming every file it wrote. No archive is fetched and no scaffolding runs.
- [x] State B performs no separate consent prompt before writing the wiring and permission entries (additive, idempotent); disclosure is the abort message's file list.
- [x] In State B, the agent's per-layout settings file gains the permission entries needed to call `gvrn` tools, added additively (no existing entry removed or reordered).
- [x] When the `gvrn` binary is not discoverable (State C), `/govern` proceeds on the markdown-only path and emits exactly one tip line pointing at the README Runtime section.
- [x] `gvrn` detection and the govern.md self-update check run in one pre-flight phase before the archive fetch; their restart-requiring writes are batched into a single abort so the user restarts at most once even when both fire.
- [x] Both the `claude-style` and `antigravity` layouts are covered by the above.
- [x] The README Runtime section states that `/govern` auto-wires `gvrn` when the binary is detected.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Compounding restarts.** *Resolved:* unify `gvrn` detection and the govern.md self-update check into a single **pre-flight phase** that batches every restart-requiring write (a fresh `govern.md` and/or the gvrn wiring + permissions) into one combined abort. Ordering alone cannot fix the compounding — a stale + unwired adopter hits two independent write-and-abort points regardless of order — so the paths are merged, not reordered. Worst case drops from three sessions (wire → update → run) to two (pre-flight writes both → run). On a genuine first run the just-curled `govern.md` is already current, so self-update does not abort and only the gvrn restart fires; the compounding only ever affected an existing adopter on a stale `govern.md` who had never wired gvrn.
- **Consent to modify agent config.** *Resolved:* no separate consent prompt. State B writes the wiring and permission entries additively and idempotently — consistent with the existing silent **Permission Setup** writes and govern.md's "no extra prompts" procedural-fidelity rule — and the pre-flight abort message names every file written as the disclosure. The user reverses any change via git or by deleting the additive `gvrn` entry. An opt-out flag for auto-wiring is explicitly out of scope for this spec.
- **claude-style `.mcp.json` location.** *Resolved:* repo-root `.mcp.json` is correct — it is Claude Code's documented project-level MCP config and already the value in govern.md §Derived values (line 71); this spec activates it rather than inventing a path. Coexistence with an existing config is the additive/idempotent in-place update: preserve all existing servers and top-level keys, add only a missing `gvrn` entry, add the `mcpServers` key if absent, leave an existing `gvrn` entry unchanged, and never clobber a malformed-JSON file (skip + warn + degrade to markdown). Reversal is per layout — `git restore` for the tracked `.mcp.json` (claude-style), manual entry-deletion for the gitignored `.agents/mcp_config.json` (antigravity). The JSON-merge implementation (host-side vs. a future primitive — none exists today) is left to `/gov:plan`.
- **Version compatibility.** *Resolved:* presence of the binary is sufficient to wire — the gate performs no version check. `runtime-tools.txt` carries only tool names (no version floor), and spec 022 §Versioning enforcement already rejected startup version comparison and refuse-on-mismatch in favor of lockstep tagged releases plus the runtime's parse-error path. A too-old wired `gvrn` is surfaced by that runtime parse error and degrades to the markdown fallback, so the bootstrap gate is the wrong layer to enforce a version floor.
- **Detection mechanism across hosts.** *Resolved:* two independent probes. State A is reached by the agent introspecting its own tool inventory for a `gvrn`-namespaced MCP tool (`mcp__gvrn__*` on Claude, `mcp:gvrn:*` on Auggie/antigravity), counting deferred/lazy names as present — no shell, no permission. The State B vs. State C split needs a `command -v gvrn`-equivalent shell probe (the only way to detect an installed-but-unregistered binary; anything queryable via a tool would already be State A), reached only when introspection finds no tool. The probe command is seeded into the existing Permission Setup pre-grant so it does not prompt. If the probe cannot run or is denied, the run is classified as State C — a harmless false negative (no auto-wire; manual wiring stays documented), so detection never hard-fails on a locked-down host.
