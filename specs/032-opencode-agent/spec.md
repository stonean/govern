---
status: in-progress
dependencies: [012-multi-agent-govern, 022-deterministic-runtime, 028-antigravity-agent, 031-agent-mcp-wiring]
review:
  last-run: 2026-06-20T16:55:57Z
  reviewed-against: a65c021bbcdf3bdd96dc970b486d51b454ddac85
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 032 — OpenCode Agent Support

Add **OpenCode** (the open-source terminal coding agent, `opencode`) as the
fourth agent `govern` scaffolds into, alongside Claude Code, Auggie, and
Antigravity. The work expresses OpenCode's layout, MCP discovery, native rules
file, and permission format as registry-derived values so `/govern` produces a
working OpenCode adoption — on **verified** conventions, not assumed ones.

> **Source provenance.** Unlike the conflicting/unverified state
> [031-agent-mcp-wiring](../031-agent-mcp-wiring/spec.md) faced, OpenCode's model
> here is **verified against the live CLI** — `opencode 1.17.8`, macOS, installed
> at `~/.opencode/bin/opencode`. Methods: `opencode debug paths` (resolved global
> paths), `opencode debug config` (resolved config from a project dir), `opencode
> debug skill` (the built-in `customize-opencode` skill, which is OpenCode's own
> authoritative config reference), and a project-config probe — a throwaway
> `/tmp` project with an `opencode.json` declaring the `gvrn` MCP server, a
> `.opencode/command/hello.md`, and an `AGENTS.md`. The probe loaded all three:
> the resolved config merged the project `mcp` / `permission` / `command` blocks,
> and `opencode mcp list` reported **`✓ gvrn connected`** — a live wiring proof.
> Following the [028-antigravity-agent](../028-antigravity-agent/spec.md)
> discipline — observed behavior governs — the model below is what the CLI did,
> not what docs claim. The narrow items still open are **design choices**, not
> unknowns about OpenCode.

## Motivation

The Agent Registry from [012-multi-agent-govern](../012-multi-agent-govern/spec.md),
generalized by [028-antigravity-agent](../028-antigravity-agent/spec.md), makes
adding an agent a registry row plus satellite files — a one-row append when the
agent shares an existing `layout`, or a bounded framework change adding a new
`layout` branch when it does not.
[031-agent-mcp-wiring](../031-agent-mcp-wiring/spec.md) then split MCP discovery
out of the `layout` axis into a per-agent descriptor (`target` / `scope` /
`mechanism`).

OpenCode is the next agent. Adopters driving their work through OpenCode get none
of the `govern` pipeline today — the slash commands are never scaffolded where
OpenCode scans, and the `gvrn` runtime is never wired where OpenCode discovers MCP
servers. This spec slots OpenCode into the same registry-driven machinery the
other three agents use.

## Verified OpenCode Layout

`govern` adopts OpenCode by file-scaffolding into the project-local **`.opencode/`**
directory and a committed **`opencode.json`** at the project root. All facts below
are verified (see provenance):

| Dimension | Verified value |
| --- | --- |
| Config dir | `.opencode/` (project); global is `~/.config/opencode/` (the install dir `~/.opencode/` is **not** the config dir) |
| Config file | `./opencode.json`, `./opencode.jsonc`, or `.opencode/opencode.json`; global `~/.config/opencode/opencode.json`. Deep-merged, **project overrides global**. `$schema: https://opencode.ai/config.json`. Unknown top-level keys are rejected with `ConfigInvalidError` |
| Invocable unit | markdown **command** at `.opencode/command/{project}/<name>.md`: `description` frontmatter, body becomes the command prompt/`template`, `$ARGUMENTS` is the argument token. Commands namespace by subdirectory — verified: `command/gov/specify.md` registers as key `gov/specify` |
| Invocation | `/{project}/<name>` (e.g. `/gov/specify`) — path-style namespace via the `{project}/` subdirectory, the OpenCode analog of Claude's colon `/{project}:<name>` |
| Native rules file | `AGENTS.md` (already shipped), read via OpenCode's `instructions` resolution — no `CLAUDE.md`, no new context file |
| MCP wiring | `mcp` block in the **project-committed** `opencode.json`: `{ "type": "local", "command": ["gvrn", "mcp"], "enabled": true }`. Project config is read and merged (probe: `✓ gvrn connected`). A scriptable `opencode mcp add` subcommand also exists |
| Permissions | `permission` block in the **same** `opencode.json`: actions `allow` / `ask` / `deny`; per-tool string or `{ pattern: action }` (last match wins); keys include `read, edit, bash, task, webfetch, …` |

Two structural facts drive the design:

- **MCP discovery is project-committed.** OpenCode reads MCP servers from the
  committed `opencode.json`, so `govern` can **write the file** (the Claude
  `write-file` posture) — it does **not** need the surface-instruction posture
  Auggie and Antigravity require ([031](../031-agent-mcp-wiring/spec.md)). OpenCode
  is only the second agent (after Claude) whose `gvrn` wiring is fully automatable.
- **One committed file carries both MCP and permissions.** `mcp` and `permission`
  are sibling keys in `opencode.json`. The settings file and the MCP-wiring file
  collapse to a single target with **two `govern`-owned regions**, so the additive
  merge must preserve `$schema` and adopter keys while owning only `mcp` and
  `permission`.

## Registry Generalization

OpenCode diverges from both existing layouts — a single committed JSON config
spanning MCP + permissions, namespaced `command/{project}/<name>.md` commands, AGENTS.md native
reading, project-committed MCP — so it is **not** a `claude-style` one-row append.
It introduces a new `layout` value (`opencode`) with its own branches in §Derived
values, §Per-Agent Scaffolding, and §Permission Setup, plus a per-agent MCP
descriptor row. These become registry-derived values:

- **Command location & invocation** — `.opencode/command/{project}/<name>.md`
  (subdirectory namespacing), invoked `/{project}/<name>`; the slash-command
  cleanup glob is the `{project}/` subdirectory under `command/`.
- **Config / settings / MCP-wiring file** — the project-root committed
  `opencode.json` (or the adopter's existing `opencode.jsonc`), with `mcp` and
  `permission` as the two owned regions. `.opencode/` (the regenerated command
  tree) is gitignored; root `opencode.json` stays committed — the Claude split.
- **Native rules file** — `AGENTS.md` (already shipped).
- **MCP descriptor** — `target`: project `opencode.json` `mcp` block; `scope`:
  `project-committed`; `mechanism`: `write-file`.
- **Permission format** — OpenCode's `permission` action map (a fourth format
  alongside Claude's `permissions.allow/deny`, Auggie's `toolPermissions[]`, and
  Antigravity's action grammar).

The Claude / Auggie / Antigravity rows are unchanged; a future `CLAUDE.md`-reading
`claude-style` agent stays a one-row append.

## Per-Agent Adoption for OpenCode

For OpenCode, `/govern` scaffolds:

- **Commands.** Transform each `framework/commands/*.md` into
  `.opencode/command/{project}/<name>.md` — keep the body (procedure + approval-gate
  prompts), carry the `description` frontmatter, preserve the `$ARGUMENTS` token.
  Placeholder substitution (`{project}`, `{cli-config-dir}`) still applies.
- **MCP.** Write the `gvrn` local-stdio server into `opencode.json`'s `mcp` block
  additively (`write-file`), preserving other keys.
- **Permissions (`configure`).** A new `framework/bootstrap/configure/opencode.md`
  writes the `permission` block in OpenCode's native format.
- **Context.** `AGENTS.md` (already shipped) is read natively — nothing new.

## Bootstrap

The existing curl-scaffold bootstrap model carries over — only the destination
changes: the README documents installing the `govern` installer into OpenCode's
command location, after which the user runs `/govern`, which scaffolds the rest.
Because OpenCode loads config **once at startup** (no hot reload), the completion
message must tell the user to restart OpenCode for newly-wired MCP servers and
commands to take effect.

## Update Story

`/govern` re-runs re-scaffold the `.opencode/` files and re-merge `opencode.json`
(live-on-main), like the other file-scaffold agents — no install/registration
step. Pinning via `.govern.toml` and the manifest strategies apply unchanged.

## Out of Scope

- **Any agent beyond OpenCode** (Cursor, Copilot, …) — each is its own spec.
- **Removing an adopted agent** — unchanged from 012 (manual).
- **OpenCode skills and agents as `govern` surfaces.** OpenCode also supports
  model-invoked **skills** (`.opencode/skill(s)/<name>/SKILL.md`) and **agents**
  (`.opencode/agent/<name>.md`); `govern`'s pipeline maps to user-invoked
  **commands**. Reusing skills/agents for any `govern` surface is deferred.
- **OpenCode's external-skill auto-load** of `~/.claude/skills/` and
  `~/.agents/skills/` (it scans those) — interaction with a co-installed Claude or
  Antigravity `govern` adoption is noted, not addressed here.
- **Extending `merge-permissions`
  ([022-deterministic-runtime](../022-deterministic-runtime/spec.md)) to
  OpenCode's format** — a plan-phase call.

## Acceptance Criteria

- [x] The Agent Registry gains an `opencode` row, and OpenCode's divergent axes
      (single `opencode.json` config, namespaced `.opencode/command/{project}/<name>.md` commands,
      `AGENTS.md` native reading, project-committed MCP, OpenCode permission
      format) are expressed as registry-derived values via a new `opencode`
      `layout` branch — not branched on the agent name; the Claude / Auggie /
      Antigravity rows are unchanged
- [x] Scaffolding OpenCode writes each pipeline command to
      `.opencode/command/{project}/<name>.md` (markdown, `description` frontmatter,
      body = prompt with `$ARGUMENTS` preserved, approval-gate prompts intact),
      invocable as `/{project}/<name>` (subdirectory namespacing)
- [x] The per-agent MCP descriptor records OpenCode's target as the
      project-committed `opencode.json` `mcp` block, `scope: project-committed`,
      `mechanism: write-file`; `/govern` writes the `gvrn` local-stdio server
      (`{ "type": "local", "command": ["gvrn", "mcp"], "enabled": true }`)
      additively, preserving other config keys
- [x] An OpenCode adoption reaches a loadable `gvrn` registration with **no
      surfaced manual instruction** (OpenCode reads the committed file) —
      verifiable by `opencode mcp list` reporting `gvrn` connected, as observed
      during this spec's verification
- [x] A `framework/bootstrap/configure/opencode.md` writes OpenCode's `permission`
      block (allow/ask/deny) in OpenCode's native format — the framework's bootstrap
      shell allows plus `"gvrn*": "allow"` to pre-allow gvrn's MCP tools without
      prompts; OpenCode never receives another agent's permission format
- [x] OpenCode's `mcp` and `permission` blocks coexist in one committed root
      `opencode.json` (or the adopter's existing `opencode.jsonc`); `govern`'s
      additive merge preserves `$schema` and adopter keys and touches only the two
      regions it owns (no `ConfigInvalidError`)
- [x] `/govern` gitignores `.opencode/` (the regenerated `command/{project}/`
      tree) but leaves the root `opencode.json` committed, so the `gvrn` wiring is
      team-shared — the same split as Claude's gitignored `.claude/` and committed
      root `.mcp.json`
- [x] Adopting OpenCode ships no `CLAUDE.md` and no new context file — the
      already-shipped `AGENTS.md` is read natively
- [x] Auto-detection recognizes an existing OpenCode adoption (its `config_dir` /
      `opencode.json`) and re-scaffolds on routine `/govern` re-runs, consistent
      with the 012/028 detect path
- [x] The README documents the OpenCode bootstrap and notes that OpenCode loads
      config once — a restart is required after MCP/command changes
- [x] All shipped markdown passes `npx markdownlint-cli2`; the emitted
      `opencode.json` is valid JSON and passes OpenCode's strict config validation

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Config dir, file, and validation.** Verified: global config is
  `~/.config/opencode/opencode.json`; project config is `opencode.json` /
  `opencode.jsonc` / `.opencode/opencode.json` (OpenCode walks up to the worktree
  root). Scopes deep-merge, project over global. Every config declares
  `$schema: https://opencode.ai/config.json`; unknown top-level keys hard-fail with
  `ConfigInvalidError`. `~/.opencode/` is only the install bin dir, not config.
  (Source: `opencode debug paths`, `opencode debug config`, the built-in
  `customize-opencode` skill.)
- **Command format, invocation & namespacing.** Verified: a
  `.opencode/command/hello.md` with `description` frontmatter and a
  `Hello $ARGUMENTS` body resolved to
  `command.hello = { description, template: "Hello $ARGUMENTS" }` — markdown
  commands under `.opencode/command/<name>.md`, body becomes the prompt,
  `$ARGUMENTS` is the argument token. OpenCode **namespaces by subdirectory**: a
  `command/gov/specify.md` registers as command key `gov/specify`, so `govern`
  scaffolds to `.opencode/command/{project}/<name>.md` invoked `/{project}/<name>`
  (e.g. `/gov/specify`) — the OpenCode analog of Claude's colon `/{project}:<name>`,
  not Antigravity's flat-prefix workaround. The `opencode` layout's slash-command
  cleanup glob is the `{project}/` subdirectory under `command/`. `govern`'s gated
  pipeline ports directly (approval gates stay in-body).
- **Native rules file.** Verified: OpenCode reads `AGENTS.md` via its
  `instructions` resolution. `govern` ships no `CLAUDE.md` and no new context file.
- **MCP discovery — target, scope, mechanism.** Verified: OpenCode reads the
  project-committed `opencode.json` `mcp` block and **connected to `gvrn`** in the
  probe (`opencode mcp list` → `✓ gvrn connected`). Local-server shape is
  `{ "type": "local", "command": [...], "enabled": true }` (`type` required,
  `command` an array). Descriptor: `target` = project `opencode.json` `mcp`;
  `scope` = `project-committed`; `mechanism` = `write-file`. A scriptable
  `opencode mcp add` also exists. Config is loaded once at startup (no hot reload),
  so a restart is required after wiring.
- **Permission schema.** Verified: a `permission` block in the same
  `opencode.json`, actions `allow` / `ask` / `deny`, per-tool value a string or
  `{ pattern: action }` (last matching rule wins). A fourth permission format,
  warranting its own `configure/opencode.md` and `settings_template`.
- **Layout decision.** Verified divergence from both existing layouts → a new
  `opencode` `layout` value, not a `claude-style` row append. The single committed
  `opencode.json` spanning MCP + permissions is the decisive difference;
  markdown-command files and AGENTS.md reading echo Antigravity but the config
  model matches neither.
- **Config-file target & gitignore.** Resolved: `govern` writes the project-root
  `opencode.json` (the conventional, committed, adopter-authored location), merging
  additively into the `mcp` and `permission` keys and preserving `$schema` and all
  other keys; if the adopter already keeps config in root `opencode.jsonc`, `govern`
  merges into that file rather than creating a second one (plan-phase detection).
  `.opencode/` — the regenerated `command/{project}/` tree — is gitignored, while
  root `opencode.json` stays committed, mirroring Claude's split (gitignored
  `.claude/`, committed root `.mcp.json`). Committing the file is what makes the
  project-committed MCP posture work; `.opencode/opencode.json` was rejected because
  it would fall under the gitignored `.opencode/` rule and split config from where
  adopters keep it.
- **MCP posture.** Resolved: `write-file` is the sole mechanism — `govern` writes
  the gvrn `mcp` block into the committed root `opencode.json` and surfaces **no**
  registration instruction (descriptor stays `mechanism: write-file`,
  `scope: project-committed`). The only post-adoption user action is restarting
  opencode (config loads once at startup). `opencode mcp add` is **not** part of the
  flow: it exposes no `--command` flag for a local stdio server (only `--url` /
  `--env` / `--header`), so it prompts interactively rather than accepting a clean
  one-liner like Auggie's — surfacing it would reintroduce a manual step the file
  write makes unnecessary. It may earn at most a README footnote as a manual
  alternative.
- **Permitting the `gvrn` MCP tools without prompts.** Resolved: `govern`'s
  `configure/opencode.md` pre-allows gvrn's tools with a single glob key
  `"gvrn*": "allow"` in the `permission` block. Verified: opencode accepts
  MCP-server-scoped permission keys (`gvrn*` / `gvrn_*` survived `debug config` with
  no `ConfigInvalidError`); there is **no** dedicated `mcp` permission key, so MCP
  tools are matched by tool-name patterns. The `gvrn*` prefix glob covers the whole
  gvrn tool namespace regardless of the exact `<server>_<tool>` separator and as
  gvrn adds primitives. opencode evaluates the **last** matching rule, so the allow
  must be ordered after any broad `*` rule (an implement-time placement detail). The
  exact separator is reconfirmed at implement time; the prefix glob is
  forward-robust.
- **`merge-permissions` (022) extension vs. generic JSON merge.** Resolved (option
  A): `govern` implements the `opencode.json` write as a **generic additive
  JSON-object merge** over the two owned regions (`mcp`, `permission`), preserving
  `$schema` and all adopter keys — **not** an extension of `merge-permissions` to a
  fourth grammar. OpenCode's `permission` is plain `{ tool-name → action }` JSON
  needing key-preserving object merge, not the allow/deny grammar reconciliation
  `merge-permissions` encodes for Claude/Auggie, and one JSON merge covers both
  regions uniformly. Per §runtime-boundary the markdown-only path performs this
  merge by hand regardless; which runtime primitive accelerates it (a new generic
  JSON-region merge vs. reusing an existing one) is a plan deliverable decided
  against the runtime-eligibility criteria.
