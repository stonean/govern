# 031 — Agent MCP Wiring Tasks

Tasks derived from the [plan](plan.md). Complete in order. Planning produced
[data-model.md](data-model.md); these are the implementation edits.

Wave 1 (tasks 1–4) does not change Antigravity behavior and is implementable now. Wave 2
(tasks 5–7) is gated on the live-`agy` verification in task 5.

## 1. Split MCP discovery off the `layout` axis in the registry

- [x] In `framework/bootstrap/govern.md`, remove the `MCP-wiring file` row from the
      §Derived values **layout** table (line ~71).
- [x] Add a per-agent **MCP registration** table (keyed by registry `key`) with
      `target` / `scope` / `mechanism` columns for `claude` / `auggie` / `antigravity`,
      matching [data-model.md](data-model.md). Antigravity's row is marked provisional /
      unverified pending task 5.
- [x] Update the "Adding a new agent" note (line ~80–88): a new `claude-style` agent now
      also needs an MCP registration descriptor entry (MCP no longer rides `layout`).
- [x] **Done when:** §Derived values no longer lists an MCP file; the per-agent table
      exists; the "Adding a new agent" contract mentions the descriptor; `lint-markdown`
      clean.

## 2. Branch State-B wiring by `mechanism` in `govern.md`

- [x] Rewrite §State B step 1 (line ~178) to branch on the agent's `mechanism`:
      `write-file` writes `target` additively (existing five-case merge); `surface-instruction`
      writes no project MCP file.
- [x] Rewrite §MCP wiring (line ~192–208) from "the per-layout path" to a per-mechanism
      description; keep the additive five-case merge for the `write-file` branch.
- [x] Add a `surface-instruction` variant to the Pre-flight abort message (line ~182–184)
      that carries the per-agent registration command and "run this, then start a fresh
      session." Auggie command: `auggie mcp add gvrn --command gvrn --args "mcp"`.
- [x] Confirm the State-B **permission write** (step 2) stays unchanged for all agents
      (project-level settings, independent of MCP server location).
- [x] **Done when:** govern.md describes both mechanisms; Auggie is no longer wired via
      `.mcp.json`; the abort covers both variants; Antigravity retains current behavior
      (provisional); `lint-markdown` clean.

## 3. Correct the README wiring description

- [x] Edit `README.md` (~line 186): replace "writes the per-agent MCP config (`.mcp.json`
      for Claude-style agents, `.agents/mcp_config.json` for Antigravity)" with the
      per-agent reality — writes `.mcp.json` for Claude; surfaces a one-line registration
      command for home-level agents (Auggie now; Antigravity per verification).
- [x] **Done when:** the README no longer claims Auggie uses a committed MCP file
      (Antigravity stays documented as `write-file`/`.agents/mcp_config.json` pending the
      task-6 verification); `lint-markdown` clean.

## 4. Record cross-spec impact on 028 and 029

- [x] Add a back-linked "Signpost (post-031)" note to `specs/028-antigravity-agent/spec.md`:
      031 supersedes the `.agents/mcp_config.json` layout-derived MCP-wiring decision.
- [x] Add a back-linked note to `specs/029-bootstrap-runtime-autowire/spec.md`: 031 changes
      State-B for home-level agents (surface-instruction, not file write).
- [x] Decide and apply the recording mechanism (signpost vs. `/gov:ask`): **Option B** —
      non-reopening navigational signposts, 028/029 stay `done` (per the existing 012
      `Signpost (post-028)` precedent). User-approved deviation from `constitution.md:543`;
      flagged for a possible constitution clarification follow-up.
- [x] **Done when:** both specs carry a back-link to 031 and the back-edge decision is
      applied.

## 5. Verify Antigravity project-local MCP loading against the live `agy` CLI

- [ ] On a machine with `agy` installed, place a `gvrn` entry in a project's
      `.agents/mcp_config.json` and confirm whether the server actually spawns (vs.
      read-but-ignored per `google-antigravity/antigravity-cli` issue #60).
- [ ] Create `specs/031-agent-mcp-wiring/scenarios/antigravity-mcp-verification.md`
      recording the `agy` version tested, the observed behavior, and the resulting
      descriptor branch (`write-file`/`project-committed` vs. `surface-instruction`/`home-level`).
- [ ] **Done when:** the scenario file records a definitive outcome (or, if `agy` is
      unavailable, records that and selects the safe home-level default).

## 6. Finalize the Antigravity descriptor and State-B branch (gated on task 5)

- [ ] Set Antigravity's registry MCP descriptor (task 1 table) to the verified values.
- [ ] If `surface-instruction`: add the Antigravity abort instruction (edit
      `~/.gemini/config/mcp_config.json`, then `/mcp` reload) and stop writing
      `.agents/mcp_config.json`. If `write-file`: keep current behavior, descriptor now
      explicit.
- [ ] **Done when:** govern.md's Antigravity MCP descriptor and State-B branch match the
      task-5 outcome; `lint-markdown` clean.

## 7. Conditional cleanup migration (gated on task 5; only if project-local is ignored)

- [ ] If verification confirmed project-local ignored: add a `framework/migrations.toml`
      entry removing a stale `.agents/mcp_config.json` from adopter repos on the next
      `/govern` run. Verify the matcher targets only that file and **never** `.mcp.json`.
- [ ] **Done when:** the migration entry exists and is antigravity-file-scoped — or the
      task is closed N/A because project-local loads.

## 8. Generalize the command-preamble MCP-prefix phrasing (optional sweep)

- [ ] Replace "server-name prefix taken from `.mcp.json`" with "taken from the agent's MCP
      registration" in `framework/bootstrap/govern.md` (line ~22) and
      `framework/commands/{target,status,analyze,implement,audit,specify,plan,ask}.md`.
- [ ] **Done when:** the phrase is host-generic everywhere; `lint-markdown` clean.

## 9. Final lint and dead-reference sweep

- [ ] Run `lint-markdown` across the feature directory and every changed framework file.
- [ ] Grep the framework + README for `.mcp.json` / `mcp_config.json` and confirm no
      remaining reference implies Auggie (or, per task 5, Antigravity) reads a committed
      MCP file.
- [ ] **Done when:** lint clean and no dead per-agent MCP-file references remain.
