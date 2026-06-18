---
status: in-progress
dependencies: [012-multi-agent-govern, 028-antigravity-agent, 029-bootstrap-runtime-autowire]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 031 — Agent MCP Wiring

govern wires the optional `gvrn` runtime as an MCP server so pipeline commands take
the deterministic path. The wiring is correct for Claude but **wrong for Auggie** (a
reproduced defect) and **unverified for Antigravity** (sources conflict): both agents
register MCP servers at the user/home level, not the committed repo file govern writes.
For Auggie this means the `gvrn` server never loads and every command silently degrades
to the slower markdown-only path. This spec corrects the per-agent MCP wiring and the
registry abstraction that produced the defect, and gates the Antigravity target on
live-CLI verification.

## Motivation

The Agent Registry in [012-multi-agent-govern](../012-multi-agent-govern/spec.md),
generalized by [028-antigravity-agent](../028-antigravity-agent/spec.md), derives every
per-agent path from a `layout` field (`claude-style` | `antigravity`). The
`MCP-wiring file` is one of those layout-derived values, and the
[029-bootstrap-runtime-autowire](../029-bootstrap-runtime-autowire/spec.md) State-B
auto-wire writes it. The abstraction assumes **every agent discovers a project-committed
MCP config file the way Claude does (repo-root `.mcp.json`). Only Claude does.**

The `claude-style` layout conflates two independent traits:

- markdown commands under `{config_dir}/commands/` + native `CLAUDE.md` reading — Auggie
  genuinely shares these, which is why it was filed as `claude-style`; and
- repo-root `.mcp.json` MCP discovery — Auggie does **not** share this.

Because MCP discovery rides the same `layout` axis, Auggie inherits Claude's `.mcp.json`
target (which Auggie does not read) and Antigravity gets a project-local
`.agents/mcp_config.json` whose runtime loading is disputed (see below).

### Observed behavior (official docs, blog write-ups, a reproduced upstream issue)

- **Auggie** loads MCP servers **only** from user-global `~/.augment/settings.json`. The
  `mcpServers` shape is a map keyed by server name with `command`/`args`/`env` —
  identical to Claude's `.mcp.json`. No project-local MCP file is read. Registration is
  via the `auggie mcp add <name> --command <cmd> --args "..."` / `auggie mcp add-json`
  subcommands, or per-launch `auggie --mcp-config '<inline json>'`. govern writes
  repo-root `.mcp.json`, which Auggie never reads. This defect is confirmed.
- **Antigravity** (`agy` CLI). Home-level config definitely loads —
  `~/.gemini/config/mcp_config.json` (shared) or `~/.gemini/antigravity-cli/mcp_config.json`
  (legacy). Whether the **project-local** path loads is **disputed and unverified**: some
  docs present workspace-local `.agents/mcp_config.json` as supported, while a reproduced
  upstream report (`google-antigravity/antigravity-cli` issue #60) shows project-local
  `mcp_config.json` read but never spawning servers (home-level only), referencing a
  `.antigravitycli/` path rather than `.agents/`. The web cannot settle this — it needs a
  test against the live `agy` CLI. Antigravity also has **no scriptable `agy mcp add`
  subcommand**; MCP management is the interactive in-prompt `/mcp` overlay, so registration
  is a config-file edit plus a `/mcp` reload, not a single pasted command.

### Current per-agent state

| Agent | Where MCP actually loads | Scope | What govern writes today | Works? |
| --- | --- | --- | --- | --- |
| Claude | repo-root `.mcp.json` | project-committed | `.mcp.json` | yes |
| Auggie | `~/.augment/settings.json` | user-global | `.mcp.json` | no |
| Antigravity | `~/.gemini/config/mcp_config.json` (project-local disputed) | home-level | `.agents/mcp_config.json` | unverified |

## Required behavior

- **MCP discovery is split out of the `layout` axis.** Whether an agent shares the
  command/skill layout and native-rules-file traits is independent of where it discovers
  MCP servers. The registry/derived-values gain a per-agent MCP descriptor carrying the
  correct target path **and scope** (project-committed vs. user-global/home-level), rather
  than inheriting one from `layout`.
- **govern stops writing files the agent ignores.** No repo-root `.mcp.json` for Auggie
  adoptions. For Antigravity, the `.agents/mcp_config.json` write is **retained pending
  live-CLI verification**; if project-local loading is confirmed broken, the target moves
  to home-level `~/.gemini/config/mcp_config.json` (surfaced per the posture decision).
- **§Derived values and §MCP wiring document the correct per-agent target and scope**, and
  the State-B auto-wire from
  [029-bootstrap-runtime-autowire](../029-bootstrap-runtime-autowire/spec.md) is updated
  so that, for an agent whose MCP config is user-global/home-level, it does the correct
  thing (exact write-vs-surface posture is resolved below — govern surfaces the registration instruction).
- **An adopter who follows govern's output reaches a working `gvrn` registration** in
  Auggie and in Antigravity — i.e. after adoption the `gvrn` MCP tools are loadable in
  those agents, by whatever mechanism the resolved posture prescribes.

### The home-level write problem

govern's State-B auto-wire model is "write a committed repo file and stop." For Auggie
and Antigravity the working config lives in the **user's home directory**, which cannot
be committed and is shared across all of that user's projects. This collides with the
existing model and is the crux the design fork (Open Questions) must resolve. The
`gvrn mcp` server itself is project-agnostic (it operates on the working directory), so a
single home-level registration serving every project is acceptable; the open question is
whether govern writes that home file, drives the agent's own CLI to write it, or surfaces
the instruction for the user to run.

## Acceptance Criteria

- [x] The Agent Registry / §Derived values no longer state that Auggie's MCP-wiring file
      is repo-root `.mcp.json`.
- [x] The Antigravity MCP target in the registry reflects a **verification against the
      live `agy` CLI**: home-level `~/.gemini/config/mcp_config.json` if project-local
      `.agents/mcp_config.json` is confirmed not to load servers, otherwise the
      confirmed-working project-local path. The verification outcome is recorded (in the
      plan or a scenario) so the decision is auditable rather than assumed.
- [x] A per-agent MCP descriptor records, for each of Claude / Auggie / Antigravity, the
      correct MCP config target path and its scope (project-committed vs. user-global vs.
      home-level), independent of the `layout` field.
- [x] §MCP wiring in `framework/bootstrap/govern.md` documents the correct registration
      target and mechanism for each agent, replacing the single per-layout file write.
- [x] An Auggie adoption produces no repo-root `.mcp.json`.
- [x] The [029-bootstrap-runtime-autowire](../029-bootstrap-runtime-autowire/spec.md)
      State-B path produces, for Auggie (and for Antigravity if verification moves its
      target off the committed file), an outcome that results in a loadable `gvrn`
      registration via the surfaced-instruction posture rather than a write to an ignored
      path.
- [x] govern's completion / State-B message surfaces the correct registration step: for
      Auggie, `auggie mcp add gvrn --command gvrn --args "mcp"`; for Antigravity, the
      config-file edit + `/mcp` reload — shown when `gvrn` is present but not yet
      registered.
- [x] If files already written into existing adopter projects (`.mcp.json` for Auggie,
      and `.agents/mcp_config.json` for Antigravity should verification retarget it) need
      cleanup, the change is registered in `framework/migrations.toml` so `/govern`
      reconciles them on the next run.

## Out of Scope

The same `.agents/`-is-project-local-and-read assumption that produced the MCP defect also
underpins govern's Antigravity **skills** (`.agents/skills/.../SKILL.md` — the slash
commands themselves) and **permissions** (`.agents/settings.json`). Blog sources point
Antigravity skills at home-level `~/.gemini/skills`, which would mean the Antigravity
command surface may not be discovered at all — a higher-stakes failure than the MCP gap,
since there is no markdown-prose fallback when the command itself never loads. **This spec
does not fix that.** It requires its own verification pass against the live `agy` CLI
before any change, and is tracked as a separate follow-up (see the skills/settings Open
Question below). Listing it here records the linkage without expanding this spec's surface.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Write-vs-surface posture for user-global / home-level MCP config.** govern **surfaces
  a one-line, copy-pasteable registration instruction** in the State-B / completion
  message and lets the user run it — it does **not** silently write or merge into
  `~/.augment/` or `~/.gemini/`, nor shell out to the agent's CLI on the user's behalf.
  Rationale: only this option keeps govern from mutating global state outside the repo
  (a posture change from every other write it makes), it avoids depending on each agent
  having a registration subcommand, and it satisfies the §Design-Principles
  "no dependence on human diligence" filter because the action is reduced to a single
  copy-paste. The home-level config is project-agnostic (`gvrn mcp` operates on the
  working directory), so the user runs it **once per machine**, not once per project. The
  Claude path keeps writing the committed repo `.mcp.json` as today — that automation is
  available precisely because Claude's config is a committable repo file; the asymmetry is
  inherent, not a regression.
- **Auggie registration mechanism.** The instruction govern surfaces for Auggie is the
  documented `auggie mcp add` subcommand in **flag form** —
  `auggie mcp add gvrn --command gvrn --args "mcp"` — as the primary path. It is the
  forward-compatible, schema-stable mechanism (Auggie owns writing its own
  `~/.augment/settings.json`), and the flag form is the most paste-safe (no embedded JSON
  for a shell to mangle, unlike `auggie mcp add-json`). `add-json` may be mentioned as an
  alternative; a manual edit of `~/.augment/settings.json` is a fallback only for users
  without the binary on PATH.
- **Antigravity registration mechanism.** Antigravity has **no scriptable `agy mcp add`
  subcommand** — MCP management is the interactive in-prompt `/mcp` overlay (status,
  reload, logs). So the instruction govern surfaces for Antigravity is a **config-file
  edit plus a `/mcp` reload**, not a single pasted command like Auggie's. Separately, the
  Antigravity MCP **target/scope is left verification-gated**: sources conflict on whether
  project-local `.agents/mcp_config.json` actually loads servers (documented as
  workspace-local, but `google-antigravity/antigravity-cli` issue #60 reports project-local
  read-but-ignored, home-level only). This cannot be settled from the web; plan/implement
  must test against the live `agy` CLI and set the target to home-level
  `~/.gemini/config/mcp_config.json` if project-local loading is confirmed broken, or to
  the verified project-local path otherwise. The earlier framing that Antigravity was
  "definitely broken, worse than Auggie" is **downgraded to unverified** accordingly.
- **Cleanup of files already written into adopter repos.** Asymmetric by file. Auggie's
  repo-root `.mcp.json` is **never migrate-deleted**: it is Claude's legitimate config
  file, govern supports multiple agents per repo, and deleting it would break Claude's
  gvrn wiring; a stale Auggie-only copy is harmless (Auggie ignores it), so govern merely
  stops writing it. Antigravity's `.agents/mcp_config.json` lives under Antigravity's own
  config dir (not shared), so a `framework/migrations.toml` entry may remove it — **but
  only if** the Q3 live-CLI verification confirms project-local does not load; if it does
  load, the file is correct and stays. The cleanup migration is therefore decided at
  implement-time, gated on the verification outcome, and never touches `.mcp.json`.
- **Scope of the Antigravity skills/settings verification.** It is a **separate spec**;
  031 stays scoped to MCP wiring. Skills/command discovery (no markdown fallback — the
  command either loads or it does not) and MCP wiring (a runtime fast-path that degrades
  gracefully) are independent surfaces with different failure modes; coupling them would
  block 031's confirmed Auggie fix on an unrelated investigation, and the constitution
  favors narrow single-concern specs. The shared live-`agy`-CLI verification session is a
  convenience, not a reason to widen scope — whoever verifies 031's Antigravity MCP target
  can check the skills/settings paths in the same sitting and feed the result into the new
  spec. The Out of Scope section records the linkage.
