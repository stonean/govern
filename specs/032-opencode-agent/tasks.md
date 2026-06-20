# 032 — OpenCode Agent Support Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Add the `opencode` registry row + derived values

- [x] Add the `opencode` row to `framework/bootstrap/govern.md` §Agent Registry (`config_dir: .opencode`, `layout: opencode`, `settings_template`, `rules_file_note`) per [data-model.md](data-model.md)
- [x] Add the `opencode` column to §Derived values (command path `.opencode/command/{project}/<name>.md`, invocation `/{project}/<name>`, `govern` install path `.opencode/command/govern.md`, settings file root `opencode.json`, permission shape OpenCode action map, native rules dir —, native rules file `AGENTS.md`, cleanup glob `{project}/` subdir under `command/`)
- [x] Update §"Adding a new agent" to record `opencode` as the third layout
- Done when: registry + derived values describe all four agents by profile; Claude/Auggie/Antigravity rows are byte-unchanged

## 2. Add the `opencode` MCP descriptor + State-B write-file branch

- [x] Add the `opencode` row to §MCP registration: target root `opencode.json` `mcp` block, scope `project-committed`, mechanism `write-file`, surfaced instruction —
- [x] Branch §MCP wiring's `write-file` subsection with an `opencode` sub-case: write the `mcp` key with OpenCode's server shape (`{ "type": "local", "command": ["gvrn", "mcp"], "enabled": true }`) into root `opencode.json`, additively (missing-file / has-key-no-gvrn / already-present no-op / no-key / invalid-JSON-skip)
- [x] Add `"gvrn*": "allow"` to the §gvrn-runtime-auto-wiring permission grant list for OpenCode
- Done when: govern.md documents OpenCode's project-committed write-file MCP wiring (root `opencode.json`, OpenCode shape) and the auto-wire grant, additively and host-side

## 3. Branch §Per-Agent Scaffolding + bootstrap flow for `opencode`

- [x] Add the `opencode` branch to the §Per-Agent Scaffolding dispatch note; copy each `framework/commands/<name>.md` → `.opencode/command/{project}/<name>.md` verbatim (carry `description`, keep body + gate prompts, substitute `{project}`/`{cli-config-dir}`→`.opencode`, preserve `$ARGUMENTS`); the configure row → `.opencode/command/{project}/configure.md`
- [x] Branch slash-command cleanup to prune stale `.md` files under `.opencode/command/{project}/` not in the manifest
- [x] Scaffold the `govern` installer to `.opencode/command/govern.md` (verbatim, placeholders kept literal)
- [x] Branch Self-Update Check + Post-Write Integrity Check claude-style-like (install path `.opencode/command/govern.md`, direct byte compare, `# govern` first-line check — no frontmatter strip)
- [x] Add `.opencode/command/{project}/` to intermediate-dir creation; `{cli-config-dir}`→`.opencode` in Placeholder Substitution
- [x] Confirm the CLAUDE.md shared-file step stays `claude-style`-only (OpenCode ships no CLAUDE.md; reads AGENTS.md); guard Workflow recommendation to skip for `opencode`
- Done when: an OpenCode bootstrap (install → self-update → scaffold) is internally consistent; the `claude-style` flow is unchanged

## 4. Branch §Permission Setup for `opencode`

- [x] Seed the root `opencode.json` `permission` from the `opencode` `settings_template` (create the file with `$schema` + `permission` if absent); note the settings file == the MCP-wiring file for OpenCode
- Done when: the fetch/scaffold phase runs without permission prompts for an OpenCode adoption, additively

## 5. Create `framework/bootstrap/configure/opencode.md`

- [x] Author the configure command writing root `opencode.json` `permission` (action map: `edit: allow`, `bash` allow/deny patterns, `webfetch`/`websearch` as needed, `"gvrn*": "allow"` ordered after any broad `*`)
- [x] Implement it as a prose-walk generic additive JSON-object merge (preserve `$schema` + adopter keys; `.jsonc` fallback), per spec Resolved Q5 — not a `merge-permissions` extension
- [x] Include the `<!-- generated:mcp-allow:start/end -->` markers for the generator
- Done when: `configure/opencode.md` exists with the canonical permission set, the merge prose, and the marker block

## 6. Emit the OpenCode MCP block from `gen-configure-mcp.sh`

- [x] Add `opencode.md` as a fourth splice target; emit a constant single `"gvrn*": "allow"` line (built outside the per-tool loop, like Antigravity's `mcp(gvrn/*)`) between the markers
- [x] Run the generator; verify `claude.md` / `auggie.md` / `antigravity.md` output is unchanged and `opencode.md` is populated
- Done when: the generator updates all four sources and the pre-commit drift invariant covers OpenCode

## 7. Add `.opencode/` to the managed `.gitignore` block

- [x] Add `.opencode/` to `framework/templates/project/gitignore.md` (ignore the regenerated command tree; leave root `opencode.json` tracked)
- [x] Update the illustrative gitignore-block references in `framework/bootstrap/govern.md` (steps that list `.claude/`, `.augment/`, `.agents/`)
- Done when: a fresh adoption gitignores `.opencode/` while keeping root `opencode.json` committed

## 8. Add the `opencode` arm to `install.sh`

- [x] Add the `opencode)` case: `dest=".opencode/command/govern.md"`, `mkdir -p .opencode/command`, seed root `opencode.json` `permission` (only if absent)
- [x] Update the usage comment and the unknown-agent error message to include `opencode`
- Done when: `curl … | sh -s -- opencode` installs a verbatim `/govern` command and a seeded `opencode.json`

## 9. Document the OpenCode bootstrap in README

- [ ] Add an OpenCode curl snippet (`… | sh -s -- opencode`) and an `### OpenCode` install section (installed as `.opencode/command/govern.md`; reads AGENTS.md)
- [ ] Add OpenCode to the supported-agents intro and the repo-layout paths summary (`.opencode/` paths)
- [ ] In Registering the runtime, note OpenCode is auto-wired (write-file to root `opencode.json`, no manual `mcp add`) and that OpenCode loads config once — a restart is required after wiring
- Done when: README documents adopting OpenCode with no second curl for additional agents

## 10. Cross-spec signpost on 028

- [ ] Add a signpost note to `specs/028-antigravity-agent/spec.md` pointing to 032 (the registry now carries a third layout, `opencode`), in a blockquote so `gen-spec-deps` derives no cycle
- Done when: 028 carries the signpost; `gen-spec-deps` derives no cycle

## 11. Tests + validation

- [ ] Extend `scripts/tests/test-gen-configure-mcp.sh` to assert the `opencode.md` `"gvrn*"` block is emitted and in sync
- [ ] Check `scripts/audit/run-all.sh` and `runtime/tests/` for any agent enumeration that must include `opencode`; fold in additions and run the audit gate green
- [ ] `npx markdownlint-cli2` clean across the feature dir and changed framework files
- [ ] Walk the spec acceptance criteria; confirm each is satisfied (including a live `opencode mcp list` → `gvrn` connected check on a scaffolded sample)
- Done when: the OpenCode generator invariant is covered, the audit gate passes, lint is clean, and all acceptance criteria are met

<!-- Out of scope (tracked, not a task here): extending gvrn `exec`'s `Host`
     command resolution to `.opencode/command/{project}/<name>.md` — a 022
     runtime change. OpenCode ships on the markdown-only path; gvrn MCP tools are
     unaffected. See plan.md Technical Decision 10. -->
