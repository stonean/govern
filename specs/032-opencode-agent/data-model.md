# 032 — OpenCode Agent Support Data Model

Structures introduced by [032 — OpenCode Agent Support](spec.md). The agent
registry schema and the `layout`-profile model are owned by
[028](../028-antigravity-agent/spec.md); this document adds the `opencode`
profile and the two govern-owned regions of OpenCode's config file. All shapes
are verified against `opencode 1.17.8` (see spec Resolved Questions).

## `opencode` registry row

| field | value |
| --- | --- |
| `key` | `opencode` |
| `name` | `OpenCode` |
| `config_dir` | `.opencode` |
| `layout` | `opencode` |
| `settings_template` | bootstrap-seed `permission` block (below), written into the **root `opencode.json`** |
| `rules_file_note` | `OpenCode reads AGENTS.md natively — no second rules file.` |

## `opencode` layout-derived values

Selected by `layout: opencode` in `framework/bootstrap/govern.md` §Derived values.

| Derived value | `opencode` formula |
| --- | --- |
| Command path | `.opencode/command/{project}/<name>.md` (verbatim markdown; `description` frontmatter; body = prompt; `$ARGUMENTS` token) |
| Invocation | `/{project}/<name>` (path namespace via the `{project}/` subdirectory) |
| `govern` install path | `.opencode/command/govern.md` (verbatim; placeholders kept literal) |
| Settings file | root `opencode.json` (**same file as the MCP-wiring file**) |
| Permission shape | OpenCode `permission` action map (`allow` / `ask` / `deny`; per-tool string or `{ pattern: action }`, last match wins) |
| Native rule-loading dir | — (rules read from shared `specs/rules/`, as `claude-style`) |
| Native rules file | `AGENTS.md` (read via OpenCode's `instructions` resolution) |
| Slash-command cleanup glob | `.md` files under the `{project}/` subdirectory of `command/` |

## `opencode` MCP descriptor

Per [031](../031-agent-mcp-wiring/spec.md) §MCP registration (per-agent;
independent of `layout`).

| field | value |
| --- | --- |
| target | root `opencode.json` `mcp` block |
| scope | `project-committed` |
| mechanism | `write-file` |
| surfaced instruction | — (none; the committed file is read directly) |

## govern-owned regions of `opencode.json`

OpenCode reads a root `opencode.json` (`./opencode.json`, `./opencode.jsonc`, or
`.opencode/opencode.json`), deep-merged over `~/.config/opencode/opencode.json`,
project overriding global, validated against `https://opencode.ai/config.json`
(unknown top-level keys hard-fail with `ConfigInvalidError`). govern writes the
project-root file and owns exactly two keys; every other key (`$schema`, `model`,
`provider`, `agent`, `command`, adopter `mcp`/`permission` entries) is preserved.

### Region 1 — `mcp.gvrn` (local stdio server)

```json
{
  "mcp": {
    "gvrn": { "type": "local", "command": ["gvrn", "mcp"], "enabled": true }
  }
}
```

`type` is required; `command` is an array. Written by the State-B `write-file`
auto-wire (host-side). Idempotent: an existing `mcp.gvrn` entry is a no-op.

### Region 2 — `permission`

Bootstrap seed (`settings_template`, written by `install.sh` and §Permission
Setup so the fetch/scaffold phase does not prompt):

```json
{
  "$schema": "https://opencode.ai/config.json",
  "permission": {
    "bash": {
      "curl *": "allow", "ls *": "allow", "tar *": "allow",
      "mktemp *": "allow", "git status *": "allow", "git config *": "allow",
      "git rev-parse *": "allow", "git diff *": "allow",
      "git ls-files *": "allow", "chmod *": "allow", "awk *": "allow",
      "command -v *": "allow"
    }
  }
}
```

Full set (written by `framework/bootstrap/configure/opencode.md`):

```json
{
  "permission": {
    "edit": "allow",
    "webfetch": "allow",
    "websearch": "allow",
    "bash": { "<allow patterns>": "allow", "rm -rf *": "deny", "*": "ask" },
    "gvrn*": "allow"
  }
}
```

Notes (verified):

- `"gvrn*": "allow"` pre-allows every gvrn MCP tool with one glob — there is **no**
  dedicated `mcp` permission key; MCP tools are matched by tool-name patterns
  (`gvrn*` / `gvrn_*` both accepted with no `ConfigInvalidError`).
- OpenCode evaluates the **last** matching rule, so `gvrn*` (and other narrow
  allows) must be ordered after any broad `"*"` rule.
- Exact `bash` allow/deny patterns are finalized at implement against the
  published schema; the shape above is the contract.

## Merge contract

Every write into `opencode.json` (the State-B `mcp` write and the configure
`permission` write) is a **generic additive JSON-object merge** (spec Resolved
Q5, option A — *not* a `merge-permissions` extension):

- Preserve `$schema` and every key govern does not own, byte-for-byte where
  possible.
- Merge into `mcp` and `permission` object keys only; add govern's entries,
  preserve the adopter's.
- If a root `opencode.jsonc` already exists, merge into that file rather than
  creating a second `opencode.json`; default to `opencode.json` when neither
  exists.
- Invalid JSON → do not touch the file (skip wiring, warn, degrade to the
  markdown path) — a hand-maintained config is never clobbered.
- The merge is host-side (markdown-only path faithful); a generic JSON-region
  merge runtime primitive is a deferred 022 follow-up.
