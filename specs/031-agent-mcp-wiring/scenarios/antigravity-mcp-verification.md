---
section: "Required behavior"
---

# Antigravity MCP loading — live `agy` CLI verification

## Context

031 left the Antigravity MCP `target` / `scope` / `mechanism` verification-gated: docs
present project-local `.agents/mcp_config.json` as workspace-local, while
`google-antigravity/antigravity-cli` issue #60 reports project-local config is
read-but-ignored (home-level only). The question can only be settled against a live `agy`
CLI. This scenario records that test (run 2026-06-17 against `agy` at `~/.local/bin/agy`,
product `antigravity-cli`).

## Behavior

**Method (model-independent).** A throwaway MCP "probe" server whose `command` simply
`touch`es a sentinel file was placed in each candidate config location; `agy -p` was run
once per location and the sentinel checked. A spawned probe ⇒ agy loaded that config. The
`agy --log-file` output was also grepped for MCP/server references. (agy's account was
quota-exhausted — `RESOURCE_EXHAUSTED 429` — so the model could not enumerate tools
directly; the sentinel spawn happens at MCP-init, before the model call, so the result is
unaffected.)

**Result.**

| Config location | Probe spawned? | MCP refs in log |
| --- | --- | --- |
| Project-local `.agents/mcp_config.json` (two runs) | **NO** | 0 |
| Home-level `~/.gemini/config/mcp_config.json` (control) | **YES** | 19 |

The home-level positive control proves `agy -p` **does** load MCP servers and reads them
from the home-level file; project-local `.agents/mcp_config.json` produced no spawn and no
log references across two runs. **Conclusion: Antigravity loads MCP only from home-level
`~/.gemini/config/mcp_config.json`; project-local `.agents/mcp_config.json` is ignored.**
Issue #60 confirmed.

**Resulting descriptor (resolves the provisional row in [data-model.md](../data-model.md)):**

- `target`: `~/.gemini/config/mcp_config.json`
- `scope`: `home-level`
- `mechanism`: `surface-instruction` (no scriptable `agy mcp add`; edit the config file, then `/mcp` reload)

This justifies the Task-7 cleanup migration: govern's previously-written
`.agents/mcp_config.json` is inert cruft and should be retired from adopter repos.

## Edge Cases

- **`agy -p` headless hang.** The print-mode session can hang past `--print-timeout`
  (a known agy bug); the verification bounds it with a poll-and-exit loop and relies on the
  sentinel file (written at MCP-init) rather than the model's response.
- **Quota exhaustion.** A `429 RESOURCE_EXHAUSTED` does not affect the result — MCP
  initialization (and the probe spawn) precedes the model call.
- **Reproduction.** Place a probe server
  `{ "mcpServers": { "probe": { "command": "sh", "args": ["-c", "touch <marker>"] } } }`
  in each config location in turn (back up and restore `~/.gemini/config/mcp_config.json`
  around the home-level run), run `agy -p` in a clean workspace, and check whether
  `<marker>` was created.

## Resolved Questions

- **Does project-local `.agents/mcp_config.json` load servers in Antigravity?** No — only
  home-level `~/.gemini/config/mcp_config.json` loads. Verified 2026-06-17 (method above).
