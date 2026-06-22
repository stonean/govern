# 029 — Bootstrap Runtime Auto-Detect and Wire Plan

Implements [029 — Bootstrap Runtime Auto-Detect and Wire](spec.md).

## Overview

This feature is entirely a change to the bootstrap **procedure prose** plus the per-layout **permission seeds** — there is no runtime (Rust) code to write. State B (binary present, not wired) is by definition the runtime-absent case, so every action it performs is host/markdown-side; no new MCP primitive is introduced.

The spine of the work is `framework/bootstrap/govern.md`: the existing **govern.md Self-Update Check** section is generalized into a **Pre-flight Phase** that also runs `gvrn` detection and batches both checks' restart-requiring writes into a single abort. The three Agent Registry `settings_template` seeds and the three `framework/bootstrap/configure/*.md` files gain the binary-probe permission. The README Runtime section is reframed from "manual wire" to "auto-wired on detect." Parity across the seeds and configure files is enforced by the existing audit scripts.

## Technical Decisions

### Pre-flight Phase (replaces the standalone Self-Update Check)

Generalize govern.md §govern.md Self-Update Check (currently at the section after §Permission Setup) into a **Pre-flight Phase** positioned **after the §Permission Setup seed** (so the probe is pre-authorized) and **before §Pre-run Migrations and the archive fetch**. The phase runs two checks — `gvrn` detection and the existing self-update byte-compare — accumulates every restart-requiring write (a fresh `govern.md` and/or the gvrn wiring + permission entries), and emits **one** combined "start a new session and re-run" abort if anything was written. If neither check requires a restart, the run proceeds normally.

Rationale: resolves the compounding-restart problem (spec Resolved Q1) — ordering alone can't fix it, so the two write-and-abort paths are merged. The existing self-update small-fetch stays inside pre-flight (it is not the archive), so State B still aborts before the archive fetch.

### State A detection — tool-inventory introspection (no shell, no permission)

The agent inspects its **own** available-tool inventory for any `gvrn`-namespaced MCP tool — `mcp__gvrn__*` (Claude), `mcp:gvrn:*` (Auggie/antigravity) — treating deferred/lazily-loaded tool names as present. Any match ⇒ State A ⇒ deterministic path, no message. No shell call, no permission. Rationale: spec Resolved Q2; the agent always knows its own tools, so this is portable and zero-cost.

### State B/C split — pre-granted binary probe, graceful degradation

Only when introspection finds no `gvrn` tool, a `command -v gvrn`-equivalent shell probe distinguishes installed-but-unregistered (State B) from absent (State C). The probe command is seeded into each layout's `settings_template` and each configure file (see below). If the probe **cannot run** (no shell) or is **denied**, classify as State C — a harmless false negative; detection never hard-fails.

Per-layout probe permission form:

- **Claude** (`permissions.allow`): `Bash(command -v *)`
- **Auggie** (`toolPermissions[]`): `{ "toolName": "launch-process", "shellInputRegex": "^command -v ", "permission": { "type": "allow" } }`
- **Antigravity** (`permissions.allow`, token-prefix grammar): `command(command -v)` — **open implementation detail**: antigravity's token-prefix matcher treats `command` as the leading token (a shell builtin, not a binary). Validate `command(command -v)` actually matches `command -v gvrn`; if the grammar over-broadens or fails, fall back to a `which gvrn` probe with `command(which)`. Decide at implement time and keep the probe form identical between the registry seed and the configure file for that layout.

### MCP wiring — additive, idempotent, in-place JSON merge (host-side)

Per-layout target from govern.md §Derived values: `.mcp.json` at repo root (`claude-style`), `{config_dir}/mcp_config.json` (`antigravity`). The entry:

```json
{ "mcpServers": { "gvrn": { "command": "gvrn", "args": ["mcp"] } } }
```

Merge rules (update in place, **never** replace/truncate):

- File missing → create with just the `gvrn` entry.
- File present, has `mcpServers`, no `gvrn` → add `gvrn`, preserve all other servers and top-level keys.
- File present, has `gvrn` → no-op (byte-unchanged).
- File present, no `mcpServers` key → add the key with just `gvrn`, preserve other top-level keys.
- File present, **malformed JSON** → do not touch; skip wiring, warn, degrade to markdown path (treat like State C).

No `merge-mcp-config` primitive is added — State B is runtime-absent by definition, so the write is always host-side. Rationale: spec Resolved Q5 + the user's confirmation that the write updates (not replaces) an existing config.

### gvrn tool permissions granted at wiring time (State B)

Alongside the wiring write, State B adds the `gvrn` tool permission to the layout's settings file (additively) so the **next** session calls the tools prompt-free. Use the per-layout wildcard — `mcp__gvrn__*` (Claude), `mcp:gvrn:*` (Auggie), `mcp(gvrn/*)` (Antigravity) — the minimal bootstrap grant. The enumerated per-tool set already lives in the generated `<!-- generated:mcp-allow -->` blocks of the configure files and is applied later by `/{project}:configure`; the wildcard at wiring time coexists harmlessly (exact-match dedup leaves both). Rationale: spec §Permission Setup + Resolved Q3.

### Reverse the "wired separately, not scaffolded" decision

The govern.md §Permission Setup paragraph that currently states the gvrn runtime is "wired separately and **not** scaffolded by `/govern`" is rewritten to describe the auto-wire-on-detect behavior. **No new confirmation prompt** is introduced — the §Procedural-fidelity allowed-prompts list is left unchanged; disclosure rides the existing pre-flight abort message, which must name every file written. Rationale: spec Resolved Q3; this reverses the line-142 decision that originated in specs 003/028.

### No version check; no data-model.md

Wire on presence of the binary alone — no version-compatibility gate (spec Resolved Q4, per 022 §Versioning enforcement). The feature introduces no domain entities or persisted data structures; the three-state enum and the JSON/permission snippets live in this plan, so **no `data-model.md` is created**.

### Parity enforcement

The probe seed entry must stay in sync across the three Agent Registry `settings_template` blobs and the three configure files. Extend/verify `scripts/audit/installer-registry-parity.sh` (and `registry-equivalence.sh`) so a future drift between a registry seed and its configure file is caught at maintainer time, matching the existing Family-14 parity discipline.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/bootstrap/govern.md` | Modify | Fold §govern.md Self-Update Check into a **Pre-flight Phase**; add three-state `gvrn` detection, an **MCP Wiring** subsection, the State-B tool-perm grant; reverse the §Permission Setup "not scaffolded" paragraph; add the probe to each Agent Registry `settings_template`; add new §Edge Cases and the State-C tip in §Post-Scaffolding Output |
| `framework/bootstrap/configure/claude.md` | Modify | Add `Bash(command -v *)` probe to the canonical allow set (gvrn tool perms already present via generated block) |
| `framework/bootstrap/configure/auggie.md` | Modify | Add the `"^command -v "` probe entry to `toolPermissions` |
| `framework/bootstrap/configure/antigravity.md` | Modify | Add the probe (`command(command -v)` or `command(which)`) to `permissions.allow` |
| `README.md` | Modify | Runtime section: reframe to "`/govern` auto-wires gvrn when the binary is detected"; keep the binary-install steps |
| `scripts/audit/installer-registry-parity.sh` | Modify | Extend parity coverage to the new probe seed entry across registry + configure (verify `registry-equivalence.sh` too) |
| `specs/029-bootstrap-runtime-autowire/plan.md` | Create | This plan |
| `specs/029-bootstrap-runtime-autowire/tasks.md` | Create | Task breakdown below |

Cross-spec (informational, not in this scope): a back-linked scenario on **003** and/or **028** noting 029 reverses the "not scaffolded" decision — record via `/gov:amend` after planning.

## Trade-offs

- **Merge vs. order the two pre-flight aborts.** Chose merge (single pre-flight). Ordering alone leaves a stale + unwired adopter with two restarts; only batching the writes guarantees one. (Resolved Q1.)
- **Probe in the always-applied seed vs. only at State B.** The *binary probe* permission goes in the always-applied seed (it must be authorized before the state is known); the *gvrn tool* permissions go in at State-B wiring time (they only matter once wired). Considered seeding the tool perms too — rejected as noise for State-C projects that never wire.
- **No `merge-mcp-config` runtime primitive.** Rejected: State B is runtime-absent by definition, so a primitive could never run there. Revisit only if a future "re-assert wiring while runtime is live" need appears.
- **Wildcard vs. enumerated gvrn tool grant at wiring time.** Chose the per-layout wildcard for the bootstrap grant (minimal, robust); the enumerated set stays owned by the generated configure blocks applied by `/{project}:configure`.
- **Antigravity probe grammar is unresolved at the prose level.** `command(command -v)` may over-broaden or mis-match under token-prefix matching; the fallback is a `which gvrn` probe. Flagged as an implement-time decision rather than guessed here.
- **Known limitation — mid-session settings reload.** Seeding the probe permission only avoids a prompt if the host re-reads its settings file mid-session (the same assumption the existing §Permission Setup already relies on). On a host that doesn't, the probe prompts once on first run; the graceful-degradation path (deny ⇒ State C) keeps that from being fatal.
- **Known limitation — no automated test for agent-driven detection.** The three-state logic is prose the agent executes; verification is the audit parity check (static parts) plus a manual walk-through of each state. No runtime unit test is added.
