# 028 — Antigravity Agent Support Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Generalize the registry with `layout` profiles

- [ ] Add a `layout` column to `framework/bootstrap/govern.md` §Agent Registry; set existing rows `claude` / `auggie` to `claude-style`
- [ ] Add the `antigravity` row (`config_dir: .agents`, `layout: antigravity`, `settings_template`, `rules_file_note`) per [data-model.md](data-model.md)
- [ ] Rewrite §Derived values as a profile table (command/skill path, invocation, `govern` install path, MCP file, settings file, permission shape, rules location, native rules file, cleanup glob)
- [ ] Update §"Adding a new agent" to note that a `claude-style` agent stays a one-row append
- Done when: the registry + derived-values sections describe all three agents by profile, and a hypothetical `.claude`-style agent is still a pure row append

## 2. Branch §Per-Agent Scaffolding for the `antigravity` layout

- [ ] For `layout: antigravity`, transform each `framework/commands/<name>.md` → `.agents/skills/{project}-<name>/SKILL.md` (set `name`, carry `description`, substitute `{project}`/`{cli-config-dir}`, preserve gate prompts)
- [ ] Scaffold the `govern` installer to `.agents/skills/govern/SKILL.md` (placeholders kept literal)
- [ ] Scaffold domain rule files to `.agents/rules/<name>.md`
- [ ] Branch slash-command cleanup to prune stale `.agents/skills/{project}-*/` dirs
- Done when: the scaffolding section produces the Antigravity skill/rules layout for the `antigravity` profile and the existing `claude-style` flow is unchanged

## 3. Create `framework/bootstrap/configure/antigravity.md`

- [ ] Author the configure command writing `.agents/settings.json` `permissions.allow/deny/ask` in the action grammar (shell `command(...)` allows/denies; file ops omitted)
- [ ] Include the `<!-- generated:mcp-allow:start/end -->` markers for the generator
- Done when: `configure/antigravity.md` exists with the canonical non-MCP permission set and the marker block

## 4. Emit the Antigravity MCP block from `gen-configure-mcp.sh`

- [ ] Add `antigravity.md` as a third splice target; emit a single `mcp(gvrn/*)` allow entry between the markers
- [ ] Run the generator; verify `claude.md` / `auggie.md` output is unchanged and `antigravity.md` is populated
- Done when: `scripts/gen-configure-mcp.sh` updates all three sources and the pre-commit invariant (drift fails) covers Antigravity

## 5. Branch Permission Setup + MCP registration in govern.md

- [ ] Branch §Permission Setup to seed `.agents/settings.json` from the `antigravity` `settings_template`
- [ ] Branch the MCP-registration step to write `.agents/mcp_config.json` (additive) for `antigravity`, vs `.mcp.json` for `claude-style`
- [ ] Document the additive merge for both `.agents/` files (host/markdown path)
- Done when: govern.md describes the two-file gvrn wiring and the settings seed for Antigravity, additively

## 6. Add `.agents/` to the managed `.gitignore` block

- [ ] Add `.agents/` to `framework/templates/project/gitignore`
- [ ] Add `.agents/` to the `merge-managed-block` content described in govern.md §Shared Files
- Done when: a fresh adoption gitignores `.agents/` alongside `.claude/`

## 7. Document the Antigravity bootstrap in README

- [ ] Add an Antigravity curl snippet (install the `govern` skill into `.agents/skills/govern/SKILL.md`)
- [ ] Add Antigravity to the supported-agents list / paths summary
- Done when: README documents adopting Antigravity with no second curl needed for additional agents

## 8. Cross-spec signpost on 012

- [ ] Add a signpost note to `specs/012-multi-agent-govern/spec.md` pointing to 028 (registry generalized to layout profiles), mirroring the `007 → 012` pattern
- Done when: 012 carries the signpost; `gen-spec-deps` derives no cycle

## 9. Tests

- [ ] Extend/create `scripts/tests/test-gen-configure-mcp.sh` to assert the `antigravity.md` `mcp(gvrn/*)` block is emitted and in sync
- [ ] Run the full generator + audit gate; confirm green
- Done when: the Antigravity generator invariant is covered and `scripts/audit/run-all.sh` passes

## 10. Validation

- [ ] `npx markdownlint-cli2` clean across the feature dir and changed framework files
- [ ] Walk the acceptance criteria; confirm each is satisfied
- Done when: all spec acceptance criteria are met and lint is clean

<!-- Out of scope (tracked, not a task here): extending gvrn `exec`'s `Host`
     command resolution to the `.agents/skills/<name>/SKILL.md` layout — a 022
     runtime change. Antigravity ships on the markdown-only path; gvrn MCP tools
     are unaffected. See plan.md Technical Decision 6. -->
