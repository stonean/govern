# 031 — Agent MCP Wiring Plan

Implements [031 — Agent MCP Wiring](spec.md).

## Overview

The defect is concentrated in one file — `framework/bootstrap/govern.md` — where MCP
discovery is a `layout`-derived value, so non-Claude agents inherit a wiring target they
never read. The fix splits MCP registration into a **per-agent descriptor** (see
[data-model.md](data-model.md)) with two mechanisms — `write-file` (Claude, today's
behavior) and `surface-instruction` (Auggie now; Antigravity pending verification) — and
rewrites the three places that consume the old layout value (§Derived values, State-B
step 1 / §MCP wiring, the Pre-flight abort message). The README's user-facing description
is corrected to match. The Antigravity target stays verification-gated against the live
`agy` CLI; everything else (the registry split + the confirmed Auggie fix + docs) lands
independently of that verification.

The work is two waves: **wave 1** (registry split, Auggie surface-instruction, README,
cross-spec signposts) does not change Antigravity behavior and is fully implementable now;
**wave 2** (Antigravity target + conditional cleanup migration) is gated on the live-CLI
verification task.

## Technical Decisions

### Split MCP discovery off the `layout` axis

Remove the `MCP-wiring file` row from the §Derived values **layout** table
(`framework/bootstrap/govern.md:71`) and add a per-agent **MCP registration** table keyed
by registry `key`, carrying `target` / `scope` / `mechanism` (schema in
[data-model.md](data-model.md)). Rationale: the original abstraction conflated two
independent traits — command/rules-file layout (which Auggie genuinely shares with Claude)
and MCP discovery (which it does not). MCP discovery is per-agent, full stop; deriving it
from `layout` is what shipped the bug.

Consequence for the registry's "adding a new agent is a one-row append" contract
(`govern.md:80-88`): a new `claude-style` agent now also needs an MCP descriptor entry.
The "Adding a new agent" note is updated to say so.

### Two registration mechanisms, keyed by scope

- **`write-file`** (`project-committed`): govern writes the repo file additively, exactly
  as today. Only **Claude** uses this in wave 1.
- **`surface-instruction`** (`user-global` / `home-level`): govern writes **no project MCP
  file**; instead the Pre-flight abort surfaces a copy-pasteable command and asks the user
  to run it and restart. Chosen over silently writing `~/.augment/` or `~/.gemini/` per the
  spec's posture decision (no mutation of state outside the repo; satisfies §Design-Principles
  "no dependence on human diligence" since it reduces to one paste).

The **permission write** (State-B step 2 — `mcp:gvrn:*` / `mcp(gvrn/*)` into the project
settings file) is **unchanged** for every agent: it targets the project-level settings file
each agent reads, independent of where the server itself is registered.

### State-B branch (`govern.md` §State B step 1, §MCP wiring, Pre-flight abort)

Replace the single "Write the per-layout MCP-wiring file additively" step with a branch on
the agent's `mechanism`:

- `write-file` → write `target` additively (the existing five-case merge logic in §MCP
  wiring: missing file, has `mcpServers`, already has `gvrn`, no `mcpServers` key, malformed).
  Add the file to the pending-restart set; the abort lists files written (today's behavior).
- `surface-instruction` → write nothing to the project for MCP; the abort instead carries
  the per-agent registration command + "run this, then start a fresh session." The
  permission write still happens and is still disclosed. The pending-restart set still
  fires (the user must restart after registering).

§MCP wiring is rewritten from "the wiring file is the per-layout path…" to a per-mechanism
description, and the abort message template (`govern.md:182-184`) gains the
surface-instruction variant.

### Per-agent surfaced instructions

- **Auggie**: `auggie mcp add gvrn --command gvrn --args "mcp"` (documented subcommand,
  flag form — paste-safe; no embedded JSON to mangle).
- **Antigravity**: edit `~/.gemini/config/mcp_config.json` to add the `gvrn` block, then
  `/mcp` reload (no scriptable `agy mcp add`; management is the interactive `/mcp` overlay).
  Surfaced only if verification routes Antigravity to `surface-instruction`.

### Antigravity target is verification-gated

A dedicated task tests the live `agy` CLI: does a `gvrn` entry in project-local
`.agents/mcp_config.json` actually spawn the server, or is it read-but-ignored (issue #60)?
The outcome is recorded in a scenario file (`scenarios/antigravity-mcp-verification.md`) so
the decision is auditable. Branch per [data-model.md](data-model.md):

- **loads** → Antigravity is `write-file` / `project-committed` on `.agents/mcp_config.json`;
  govern's current behavior is correct, descriptor recorded, no migration.
- **ignored** → Antigravity is `surface-instruction` / `home-level` on
  `~/.gemini/config/mcp_config.json`; add a conditional cleanup migration (below).

Until resolved, the descriptor records the current behavior annotated "unverified"; if the
verification cannot be run at all, the safe default is the home-level / surface-instruction
branch (home-level loading is confirmed).

### Conditional cleanup migration (wave 2, Antigravity only)

If verification confirms project-local is ignored, add a `framework/migrations.toml` entry
that removes a stale `.agents/mcp_config.json` from adopter repos on the next `/govern` run.
This **never touches `.mcp.json`** — that file is Claude's legitimate config and may be in
active use in a mixed-agent repo; a stale Auggie-only `.mcp.json` is harmless and is left
in place. Per the AGENTS.md gotcha, a new migration wires into `framework/migrations.toml`
and is picked up by the existing Pre-run Migrations registry.

### README correction

`README.md:186` currently says govern "writes the per-agent MCP config (`.mcp.json` for
Claude-style agents, `.agents/mcp_config.json` for Antigravity)" — now inaccurate for
Auggie (claude-style but home-level) and over-committed for Antigravity. Rewrite to: govern
writes `.mcp.json` for Claude, and surfaces a one-line registration command for
home-level agents (Auggie now; Antigravity per verification).

### Preamble phrasing (secondary, mechanical)

The shared command-preamble line "server-name prefix taken from `.mcp.json`"
(`govern.md:22` and `framework/commands/{target,status,analyze,implement,audit,specify,plan,ask}.md`)
is Claude-specific in a host-generic sentence. Generalize to "taken from the agent's MCP
registration." Low-risk uniform sweep; grouped as the last task and not a blocker for the
behavior fix.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `specs/031-agent-mcp-wiring/data-model.md` | Create | Per-agent MCP registration descriptor schema + values |
| `specs/031-agent-mcp-wiring/scenarios/antigravity-mcp-verification.md` | Create | Record the live-`agy` verification outcome and the resulting Antigravity descriptor |
| `framework/bootstrap/govern.md` | Modify | Remove MCP row from §Derived values layout table; add per-agent MCP registration table; branch State-B step 1 + §MCP wiring + Pre-flight abort by `mechanism`; update "Adding a new agent" note |
| `README.md` | Modify | Correct the runtime/MCP wiring description (line ~186) to per-agent reality |
| `framework/migrations.toml` | Modify (wave 2, conditional) | Remove stale `.agents/mcp_config.json` — only if verification confirms project-local ignored |
| `framework/commands/{target,status,analyze,implement,audit,specify,plan,ask}.md` | Modify (optional sweep) | Generalize "prefix taken from `.mcp.json`" preamble phrasing |
| `specs/028-antigravity-agent/spec.md` | Modify (cross-spec) | Back-linked signpost: 031 supersedes the `.agents/mcp_config.json` layout-MCP decision |
| `specs/029-bootstrap-runtime-autowire/spec.md` | Modify (cross-spec) | Back-linked signpost: 031 changes State-B for home-level agents |

## Trade-offs

- **Asymmetric UX (Claude auto-wires; others need a paste).** Accepted — inherent to where
  each agent stores MCP config. Rejected alternatives: forcing all agents into a project
  file (doesn't work — they don't read it); silently writing the user's home files
  (violates the disclose-every-write posture and risks clobbering hand-maintained config).
- **Antigravity verification deferred to implement, not resolved at spec time.** Accepted —
  it can't be settled from the web (docs vs. reproduced issue #60 conflict); it needs the
  live `agy` CLI. Risk: if no machine with `agy` is available, the Antigravity branch can't
  be finalized. Mitigation: wave 1 (registry split + Auggie fix + docs) ships regardless,
  and the safe home-level default applies if verification is impossible.
- **`.mcp.json` is never cleaned up.** Accepted — it's Claude's file; deleting it would
  break Claude wiring in mixed-agent repos. Limitation: an Auggie-only adopter keeps a
  harmless, ignored `.mcp.json`.
- **govern can't confirm the user ran the surfaced command.** Accepted — the next session's
  State-A tool-inventory introspection is the implicit check (tools present ⇒ it worked);
  no extra verification machinery is added.
- **Cross-spec signposts touch `done` specs 028/029.** This may trip the `done → in-progress`
  back-edge; the recording mechanism (signpost vs. `/gov:ask`) is settled at implement time
  to keep the reopen, if any, intentional rather than incidental.
