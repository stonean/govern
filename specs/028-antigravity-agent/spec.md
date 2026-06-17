---
status: done
dependencies: [012-multi-agent-govern, 022-deterministic-runtime]
review:
  last-run: 2026-06-10T02:46:51Z
  reviewed-against: 072593cdf6334cb5ff8554f9c0db07fb38c27c79
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 028 — Antigravity Agent Support

> **Signpost (post-031):** the MCP-wiring model in this spec — Auggie via `.mcp.json`, Antigravity via `.agents/mcp_config.json`, both treated as **layout-derived** — was corrected by [031-agent-mcp-wiring](../031-agent-mcp-wiring/spec.md). MCP discovery is **not** layout-derived: Auggie registers MCP servers in user-global `~/.augment/settings.json` (not `.mcp.json`), so govern surfaces a one-line registration command instead of writing a file; Antigravity's project-local `.agents/mcp_config.json` loading is verification-gated against the live `agy` CLI. 031 moves MCP into a per-agent descriptor (`target` / `scope` / `mechanism`); the registry's other per-agent fields (command/skill location, settings file, rules location, native rules file) are unchanged.

Generalize the agent registry from "two agents that share one layout" to "N
agents across differing per-agent layouts and host conventions," then use that
generalization to add Google's **Antigravity CLI** (`agy`) as the third
supported agent. Claude Code and Auggie both adopt govern by **file-scaffolding
into a per-project config dir** (`.claude/`, `.augment/`): markdown slash
commands under `{config_dir}/commands/{project}/`, a native rules file
(`CLAUDE.md`), the gvrn runtime via `.mcp.json` + per-tool allowlists in
`{config_dir}/settings.local.json`. Antigravity uses the **same file-scaffold
model** but a **different layout**: workspace skills under `.agents/skills/`,
the gvrn runtime via `.agents/mcp_config.json`, and permissions via
`.agents/settings.json` with an `allow`/`deny`/`ask` schema. Closing that gap is
the work — once the registry models per-agent **command/skill location**,
**MCP-wiring file**, **settings file + permission format**, **rules location**,
and **native rules file**, the *next* agent becomes a true one-row append, which
is the promise [012-multi-agent-govern](../012-multi-agent-govern/spec.md) made
but only delivered for `.claude`-style agents.

> **Why Antigravity, not Gemini CLI.** This spec originally targeted Gemini CLI.
> Google is shutting down Gemini CLI on **2026-06-18** (no grace period for
> Pro/Ultra/free tiers) and migrating users to Antigravity CLI. Building support
> for a CLI that retires within days is wasted effort, so the concrete target is
> Antigravity.
>
> **Source provenance.** Antigravity's official docs are client-rendered and
> could not be fetched directly. The model below was established from (a) doc
> excerpts supplied by the maintainer, (b) **direct probing of the installed
> CLI** (`agy 1.0.7`, macOS: `agy plugin validate/install/list/uninstall`,
> `agy -p` live sessions), and (c) cross-checks between the two. One claim — that
> Antigravity reads `.claude/` — came from asking the `agy` *agent* about its own
> config; it was **empirically refuted** (a live `agy` session loaded a
> workspace skill from `.agents/skills/<name>/SKILL.md` and reported its exact
> path, while identical probes under `.claude/commands/` and `.claude/skills/`
> did not load). Self-reports from the agent are treated as unverified; observed
> behavior governs.

## Problem

[012-multi-agent-govern](../012-multi-agent-govern/spec.md) collapsed the
per-CLI `govern.md` variants into one unified file driven by an **Agent
Registry** (defined in `framework/bootstrap/govern.md`). Its closing
contract — *"Adding a new agent is a one-row addition to the registry"* plus a
`configure/{key}.md` file and a README curl snippet, with *"No other changes are
required"* — holds only because Claude and Auggie share layout assumptions baked
into the unified procedure:

1. **Commands** live at `{config_dir}/commands/{project}/<name>.md` and resolve
   to `/{project}:<name>`.
2. **The native rules file is `CLAUDE.md`**, read by both agents.
3. **The runtime is `.mcp.json`** + per-tool allowlists in
   `{config_dir}/settings.local.json`, populated by `/{project}:configure`.

**Antigravity keeps the file-scaffold model but diverges on every one of those
specifics.** Its invocable units are **skills** (`.agents/skills/<name>/SKILL.md`,
frontmatter `name` + `description`, invoked `/<name>`), not
`commands/{project}/`. It reads **`AGENTS.md`** natively, not `CLAUDE.md`. The
runtime is wired in **`.agents/mcp_config.json`**, not `.mcp.json`. And
permissions live in **`.agents/settings.json`** under a `permissions` object
with **`allow`/`deny`/`ask`** arrays whose entries use an action grammar
(`command(...)`, `read_file(...)`, `mcp(server/tool)`, `unsandboxed(...)`, …) —
a *third* format alongside Claude's `permissions.allow/deny` and Auggie's
`toolPermissions[]`. Appending an Antigravity row to today's registry would
scaffold commands into a directory Antigravity never scans and a runtime it
cannot reach. This spec is therefore a **framework change** — generalizing the
registry's layout and host-convention assumptions — not a row append.

### The structural divergences

| Dimension | Claude / Auggie | Antigravity CLI |
| --- | --- | --- |
| Adoption | file-scaffold into config dir | file-scaffold into config dir (same model) |
| Config dir | `.claude/` / `.augment/` | `.agents/` |
| Invocable unit | markdown command, `commands/{project}/<name>.md` | **skill**, `.agents/skills/<name>/SKILL.md` (dir form; `name`+`description` frontmatter) |
| Invocation | `/{project}:<name>` (colon namespace) | `/<name>` (flat; govern prefixes, e.g. `/{project}-<name>`) |
| Native rules file | `CLAUDE.md` | `AGENTS.md` (already shipped) |
| Domain rule files | filesystem `specs/rules/` | `.agents/rules/<name>.md` |
| MCP wiring | `.mcp.json` (server) + per-tool allowlist | `.agents/mcp_config.json` (server) + `mcp(gvrn/*)` allow |
| Permissions | `settings.local.json` (`permissions.allow/deny` or `toolPermissions[]`) | `.agents/settings.json` (`permissions.allow/deny/ask`, action grammar) |

(Antigravity facts verified against `agy 1.0.7` and maintainer-supplied docs;
see Resolved Questions for the per-fact provenance.)

## Verified Antigravity Layout

govern adopts Antigravity by scaffolding into the project-local **`.agents/`**
directory (the workspace scope — verified: a live `agy` session loaded a
workspace skill at `<repo>/.agents/skills/<name>/SKILL.md`):

| govern artifact | Antigravity destination | Notes |
| --- | --- | --- |
| Pipeline commands (`framework/commands/*.md`) | `.agents/skills/{project}-<name>/SKILL.md` | dir-form skill; body = the command procedure (gates preserved as in-body prompts); `name` + `description` frontmatter; invoked `/{project}-<name>` |
| `govern` installer | `.agents/skills/govern/SKILL.md` | curl-scaffolded for bootstrap |
| Domain rule files | `.agents/rules/<name>.md` | native Antigravity rule loading |
| gvrn runtime | `.agents/mcp_config.json` | `{ "mcpServers": { "gvrn": { "command": "gvrn", "args": ["mcp"] } } }` |
| Permissions | `.agents/settings.json` | `permissions.allow/deny/ask`; gvrn covered by a single `mcp(gvrn/*)` |
| Project context | `AGENTS.md` (already shipped) | read natively; no `CLAUDE.md`, no new context file |

A **global plugin** form also exists (`agy plugin install` →
`~/.gemini/config/plugins/<name>/` with `plugin.json` + `skills/` + `rules/` +
`mcp_config.json`), but it is per-user/global and not how a per-project tool
adopts. It is recorded as an **optional, deferred** distribution channel (see
Resolved Questions / marketplace), not govern's adoption path.

## Registry Generalization

The Agent Registry stops assuming the `.claude`-style layout. These become
per-agent registry-derived values rather than hard-coded constants:

- **Command/skill location & invocation** — `{config_dir}/commands/{project}/`
  with `/{project}:<name>` vs `.agents/skills/<name>/SKILL.md` with `/<name>`.
- **MCP-wiring file** — `.mcp.json` vs `.agents/mcp_config.json`.
- **Settings file & permission format** — `settings.local.json`
  (`permissions.allow/deny` / `toolPermissions[]`) vs `.agents/settings.json`
  (`permissions.allow/deny/ask`, action grammar).
- **Rules location** — filesystem `specs/rules/` vs `.agents/rules/`.
- **Native rules file** — `CLAUDE.md` vs `AGENTS.md`.

A future `.claude`-style agent that reads `CLAUDE.md` remains a pure one-row
append — the generalization must not tax the common case.

## Per-Agent Adoption for Antigravity

For Antigravity, `/govern` scaffolds the project-local `.agents/` tree:

- **Skills.** Transform each `framework/commands/*.md` into
  `.agents/skills/{project}-<name>/SKILL.md`: keep the body (the procedure, with
  its approval-gate prompts) unchanged, carry the `description` frontmatter, add
  the `name` field. Placeholder substitution (`{project}`, `{cli-config-dir}`)
  still applies. The argument-token equivalent of `$ARGUMENTS` is a plan-phase
  detail to confirm.
- **Rules.** Scaffold govern's domain rule files to `.agents/rules/<name>.md`.
- **MCP.** Write `.agents/mcp_config.json` wiring gvrn as a local stdio server,
  additively if the file already exists.
- **Permissions (`configure`).** A new `framework/bootstrap/configure/{key}.md`
  for Antigravity writes `.agents/settings.json` `permissions` in Antigravity's
  action grammar: `mcp(gvrn/*)` for the runtime, `command(...)` allows/denies for
  shell, with workspace files auto-allowed by default. `gen-configure-mcp.sh`
  emits the Antigravity MCP block (a single `mcp(gvrn/*)` line).
- **Context.** `AGENTS.md` (already shipped) is read natively — nothing new.

## Bootstrap

Because Antigravity skills are markdown and discovered from `.agents/`, the
existing curl-scaffold bootstrap works — only the destination changes: the
README documents installing the `govern` skill into
`.agents/skills/govern/SKILL.md`, after which the user runs `/govern`, which
scaffolds the rest. The govern self-install rule (keep every placeholder literal)
carries over. No plugin build, no `agy plugin install`, no chicken-and-egg.

## Update Story

`/govern` re-runs re-scaffold the `.agents/` files (live-on-main), exactly like
the other file-scaffold agents — no install/registration step. Pinning via
`.govern.toml` and the manifest `update`/`create`/`skip` strategies apply
unchanged.

## Out of Scope

- **Gemini CLI** — dropped given its 2026-06-18 shutdown.
- **Global plugin + marketplace** (`agy plugin install govern@…`) — an optional,
  separate distribution channel, deferred to future work.
- **Skill-vs-slash-command form refinement** — probing showed dir-form
  `skills/<name>/SKILL.md` loads as an agent-context skill while a flat
  `.agents/skills/<name>.md` did not load in `agy -p` (print mode reported "no
  custom slash commands"), hinting at a skill (agent-context) vs slash-command
  (TUI-only) distinction. govern targets the verified dir-form skill; pinning the
  exact form and the `$ARGUMENTS` equivalent is a plan-phase detail.
- Agents beyond Antigravity (Cursor, Copilot, …).
- Removing an adopted agent (unchanged from 012 — manual).

## Acceptance Criteria

- [x] The Agent Registry expresses per-agent **command/skill location**,
      **invocation**, **MCP-wiring file**, **settings file + permission format**,
      **rules location**, and **native rules file** as derived values;
      Claude/Auggie keep their `.claude`/`.augment` layout, Antigravity is
      `config_dir = .agents` with the skill/`mcp_config.json`/`settings.json`/
      `rules` layout, and the unified procedure branches on registry values, not
      the agent name
- [x] A `.claude`-style file-scaffold agent that reads `CLAUDE.md` can still be
      added by a single registry-row append + `configure/{key}.md` + README
      snippet (verified by a documented "add a hypothetical agent" checklist)
- [x] Scaffolding Antigravity writes each pipeline command as
      `.agents/skills/{project}-<name>/SKILL.md` (dir-form skill, body = the
      command procedure with gates intact, `name` + `description` frontmatter),
      invocable as `/{project}-<name>`
- [x] govern's domain rule files scaffold to `.agents/rules/<name>.md`
- [x] gvrn is wired via `.agents/mcp_config.json` (local stdio server, additive)
      **and** `.agents/settings.json` allows `mcp(gvrn/*)`
- [x] A `framework/bootstrap/configure/{key}.md` for Antigravity writes
      `.agents/settings.json` `permissions` in Antigravity's action grammar;
      `gen-configure-mcp.sh` emits the Antigravity MCP block
- [x] Adopting Antigravity ships **no `CLAUDE.md`** and no new context file;
      `AGENTS.md` (already shipped) carries the context
- [x] Each agent receives only its own permission format — Antigravity never
      receives Claude's `permissions`/`Bash(...)` or Auggie's `toolPermissions`,
      and vice versa (the 012 format-leakage failure mode does not recur)
- [x] Auto-detection recognizes an existing `.agents/` adoption and re-scaffolds
      on routine re-runs, consistent with the 012 detect path
- [x] The README documents the Antigravity bootstrap (curl the `govern` skill
      into `.agents/skills/govern/SKILL.md`, then run `/govern`)
- [x] All shipped markdown passes `npx markdownlint-cli2`; the emitted
      `.agents/mcp_config.json` and `.agents/settings.json` are valid JSON

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Unit + mapping (was "command→skill namespacing").** govern targets
  Antigravity's documented surface — **skills** (`.agents/skills/<name>/SKILL.md`)
  for pipeline steps, **`rules/`** for govern's rule files, **`mcp_config.json`**
  for gvrn — not a `commands/` directory. The `commands/*.md → skills` conversion
  observed via `agy plugin validate`/`install` is an undocumented
  import-compatibility shim (for `agy plugin import claude|gemini`); govern does
  not rely on it. Skill names are **flat** — no `/{project}:<name>` colon
  namespace — so govern prefixes names (`/{project}-<name>`). govern's rule files
  map 1:1 onto `rules/`.
- **Skill invocation / UX (and the `.claude/` refutation).** Skills are
  user-invoked: `.agents/skills/<name>/SKILL.md` → `/<name>` (maintainer-confirmed
  for `.agents/skills/format-tests.md` → `/format-tests`). govern's gated,
  user-driven pipeline therefore **ports directly** — the user types the slash
  command and the approval gates remain in-body prompts. A live `agy` session
  loaded a workspace skill from `<repo>/.agents/skills/probe-ws/SKILL.md`
  (reported with its exact path), while identical probes under `.claude/commands/`
  and `.claude/skills/` did **not** load — empirically refuting the agent's
  self-report that Antigravity uses `.claude/`. The **dir form**
  `<name>/SKILL.md` is what loaded; a flat `.agents/skills/<name>.md` did not load
  in print mode. The flat-vs-dir / skill-vs-slash-command nuance and the
  `$ARGUMENTS` equivalent are plan-phase details.
- **Install model + update flow (was "install: copy vs reference") — scoped to
  the optional plugin path.** `agy plugin install <dir>` snapshot-copies into
  `~/.gemini/config/plugins/<name>/` and is idempotent (re-install overwrites;
  verified VERSION_ONE → VERSION_TWO). This applies only to the deferred global
  plugin distribution; **govern's adoption is file-scaffold into `.agents/`**, so
  its update flow is re-scaffolding files, not re-installing.
- **Rules file + context (was "CLAUDE.md readership").** Confirmed: Antigravity
  reads `AGENTS.md` natively. govern ships **no `CLAUDE.md`** and no new context
  file; `AGENTS.md` (already shipped) carries the context; rule files map to the
  `.agents/rules/` dir. Where the constitution content lands (inline in
  `AGENTS.md` vs a rule) is a plan-phase mapping detail.
- **Adoption model + detection (was "detection key").** Antigravity has a
  per-workspace scope — `.agents/skills/`, `.agents/mcp_config.json`,
  `.agents/settings.json` (workspace-skill loading empirically verified). govern
  adopts by **file-scaffolding into `.agents/`** per project, exactly as for
  Claude/Auggie — not by building/installing a global plugin. Registry row:
  `config_dir = .agents`. **Detection** keys on `.agents/` existing in the
  project. **Update** is re-scaffolding (live-on-main).
- **Permissions / `configure` (corrected from "N/A").** Antigravity **does** have
  a permissions file — `.agents/settings.json` (workspace; global form
  `~/.gemini/antigravity-cli/settings.json`), **not** `.claude/settings.local.json`
  (refuted with the `.claude/` test). Schema: `{ "permissions": { "allow": [],
  "deny": [], "ask": [] } }`, structurally close to Claude's `permissions.allow/
  deny` plus an `ask` tier, with an action grammar (`command(...)`,
  `read_file(...)`, `mcp(server/tool)`, `unsandboxed(...)`, `*`). govern therefore
  **gains a `configure` step** for Antigravity (a third permission format): a new
  `framework/bootstrap/configure/{key}.md` writes the allow/deny/ask set, with
  gvrn covered by a single `mcp(gvrn/*)` and shell allows/denies as
  `command(...)`; workspace files are auto-allowed so `read_file`/`write_file`
  are largely unneeded.
- **MCP wiring + merge ownership.** gvrn spans two `.agents/` files —
  `mcp_config.json` (server definition) and `settings.json` (`mcp(gvrn/*)`
  permission) — mirroring Claude's `.mcp.json` + settings split. Both installs are
  additive (preserve adopter entries). `mcp_config.json` is a JSON-object merge;
  `settings.json` `permissions` is close enough to Claude's shape that
  `merge-permissions`
  ([022-deterministic-runtime](../022-deterministic-runtime/spec.md)) is a
  candidate to extend to a third format. Both are host-owned for now; whether a
  runtime primitive grows to own them is a plan-phase call (parallel to the
  existing Auggie `merge-permissions` gap).
- **Marketplace / global-plugin distribution — deferred.** Given the workspace
  `.agents/` adoption model, publishing a global plugin
  (`agy plugin install govern@marketplace`) is an optional, separate distribution
  channel, out of scope for this spec. Revisit as future work if there is demand
  for a one-command global install.
