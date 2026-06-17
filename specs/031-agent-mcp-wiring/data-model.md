# 031 — Agent MCP Wiring Data Model

This feature modifies the **Agent Registry** schema established in
[012-multi-agent-govern](../012-multi-agent-govern/spec.md) and
[028-antigravity-agent](../028-antigravity-agent/spec.md). It removes MCP wiring from
the layout-derived value set and introduces a **per-agent MCP registration descriptor**.

## Per-agent MCP registration descriptor

MCP discovery is no longer derived from `layout`. Each registry agent gains a descriptor
with three fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `target` | string (path) | Where the agent reads MCP server definitions. May be a repo-relative path or a `~`-rooted home path. |
| `scope` | enum `project-committed` \| `user-global` \| `home-level` | Whether `target` lives in (and travels with) the repo, or in the user's home and is shared across all their projects. |
| `mechanism` | enum `write-file` \| `surface-instruction` | How govern's State-B auto-wire registers the server: write the file directly, or surface a copy-pasteable instruction the user runs. |

`scope` and `mechanism` are correlated but distinct: `project-committed` ⇒ `write-file`
(govern owns a repo file); `user-global` / `home-level` ⇒ `surface-instruction` (govern
must not mutate the user's home, per the spec's posture decision).

## Per-agent values

| key | `target` | `scope` | `mechanism` | Surfaced instruction (when `surface-instruction`) |
| --- | --- | --- | --- | --- |
| `claude` | `.mcp.json` (repo root) | `project-committed` | `write-file` | — |
| `auggie` | `~/.augment/settings.json` | `user-global` | `surface-instruction` | `auggie mcp add gvrn --command gvrn --args "mcp"` |
| `antigravity` | **verification-gated** (see below) | **verification-gated** | **verification-gated** | edit `~/.gemini/config/mcp_config.json` (add the `gvrn` block), then `/mcp` reload |

### Antigravity: verification-gated descriptor

Until tested against the live `agy` CLI, Antigravity's descriptor is **provisional**. Two
outcomes:

- **Project-local `.agents/mcp_config.json` loads servers** ⇒ `target: .agents/mcp_config.json`,
  `scope: project-committed`, `mechanism: write-file` — govern's current behavior is
  correct and only the descriptor is recorded explicitly.
- **Project-local is read-but-ignored** (the `google-antigravity/antigravity-cli` issue #60
  reading) ⇒ `target: ~/.gemini/config/mcp_config.json`, `scope: home-level`,
  `mechanism: surface-instruction`, and a `framework/migrations.toml` entry removes the
  stale `.agents/mcp_config.json` from adopter repos.

Until verification resolves it, the provisional value is the current behavior
(`.agents/mcp_config.json`, `write-file`) annotated "unverified" — the home-level default
is the safe fallback (home-level definitely loads) if verification cannot be performed.

## Server entry shape (unchanged across all agents)

Every target — repo file or home file — uses the same `mcpServers` map keyed by server
name. Only the file location differs.

```json
{ "mcpServers": { "gvrn": { "command": "gvrn", "args": ["mcp"] } } }
```

## Relationship to the existing registry

- The `MCP-wiring file` row is **removed** from the §Derived values *layout* table
  (`framework/bootstrap/govern.md`) — it was the source of the conflation.
- The descriptor above is added as a **per-agent** table (keyed by registry `key`, like
  `config_dir`), not a layout-derived value.
- Adding a new `claude-style` agent is therefore no longer a pure one-row append: it also
  needs an MCP registration descriptor entry, because MCP no longer rides `layout`.

## Notes

- The permission grant (`mcp__gvrn__*` / `mcp:gvrn:*` / `mcp(gvrn/*)`) is **independent**
  of this descriptor and unchanged — it lives in the project-level settings file every
  agent reads, regardless of where the server itself is registered.
- The `gvrn mcp` server is project-agnostic (operates on the working directory), so a
  single `user-global` / `home-level` registration serves every project — the user runs
  the surfaced instruction once per machine.
