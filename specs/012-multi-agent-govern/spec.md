# 012 — Multi-Agent Govern

**Status:** done
**Dependencies:** 007-govern-workflow

A single `govern.md` command that supports adopting governance for multiple AI coding CLIs in the same project, with the target agent(s) selected at run time rather than baked into the file. Re-runs are additive — a project initialized for one agent can later adopt another by re-running `/govern` and selecting the new agent.

## Problem

[007-govern-workflow](../007-govern-workflow/spec.md) ships a separate `govern.md` per CLI (`govern/govern.md` for Claude Code, `govern/govern-auggie.md` for Auggie). The target CLI is implicit — determined by which file the user installed. This worked for single-agent projects but creates two failure modes when a project uses more than one agent:

1. **Half-installed projects** — when only one variant has been run, the second CLI has no commands and no settings. Adding the second agent requires the user to know about the curl bootstrap and pick the right URL.
2. **Format leakage between agents** — the two variants are 95% identical, and divergent details (settings file format, setup command source) drift unless the parity rule in `CLAUDE.md` is followed strictly. Drift has already produced a `permissions` key (Claude format) inside `.augment/settings.local.json`, which Auggie cannot parse.

The two-file model also makes the parity rule itself a recurring maintenance cost: any change to shared logic must be applied in both files.

## Distribution Model

The deliverable is a single `govern/govern.md` file in the governance repo. The file is self-contained and supports every agent the framework knows about. Each supported agent contributes a small set of per-agent values (config directory name, settings format, setup command source) consumed by the same shared logic.

### Initial install

The user installs `govern.md` into one CLI's command directory using a single curl command. The README documents the install path for each supported agent. After the first install, the user does not need to curl again to add additional agents — the unified command self-installs into every selected agent's command directory during scaffolding.

### Supported agents

The set of agents the unified command supports is intrinsic to the file. Adding a new agent requires only contributing per-agent values to the unified command, not creating a new top-level govern file.

The framework currently supports:

| Agent | Config directory | Settings format | Setup source |
| --- | --- | --- | --- |
| Claude Code | `.claude/` | `permissions.allow` / `permissions.deny` | `commands/setup.md` |
| Auggie | `.augment/` | `toolPermissions` array | `commands/setup-auggie.md` |

## Agent Selection

When `/govern` runs, it determines which agents to scaffold using this order:

1. **Explicit `$ARGUMENTS`** — if `$ARGUMENTS` contains `--agents=...`, the listed agents are scaffolded and no prompt is shown.
2. **Auto-detect (default — routine update)** — if any supported agent's config directory exists in the project (e.g., `.augment/`), those agents are scaffolded silently with no prompt. This is the path that runs on every routine `/govern` re-run.
3. **Add-agent mode** — if `$ARGUMENTS` contains `--add-agent`, or no agent dirs are detected (first run), a prompt is shown with detected agents pre-selected. The user picks the additional agents to adopt.

The user must end up with at least one selected agent. In add-agent or first-run mode, selecting zero is rejected with a clear message. The auto-detect path always has at least one agent (the dir the running `govern.md` lives in is detected).

Removing an adopted agent's tree is not part of `/govern`'s scope — see the **Re-Run Behavior** section.

## Per-Agent Scaffolding

For each selected agent, the unified command performs the same steps with per-agent values substituted:

- Resolve `{cli-config-dir}` to the agent's config directory (e.g., `.claude`, `.augment`).
- Scaffold slash commands into `{cli-config-dir}/commands/{project}/`.
- Scaffold the setup command from the agent's setup source into `{cli-config-dir}/commands/{project}/setup.md`.
- Create `{cli-config-dir}/{project}-session.json` (empty `{}`) if it does not exist.
- Write a settings file at `{cli-config-dir}/settings.local.json` in the agent's native format with the entries needed for govern's curl/ls fetches to run without prompts.
- Install a copy of the unified `govern.md` at `{cli-config-dir}/commands/govern.md` so the command is invokable from that agent on subsequent runs.

Per-agent native conventions are preserved:

- **Claude Code** uses `permissions.allow`/`permissions.deny` arrays. `CLAUDE.md` is written from `templates/claude-md.md` if missing.
- **Auggie** uses a `toolPermissions` array with `toolName`/`shellInputRegex` entries. `CLAUDE.md` is written but Auggie reads it natively — no second rules file is needed.

## Shared Files

These files are scaffolded once per `/govern` run regardless of how many agents are selected:

- `constitution.md`
- `.markdownlint-cli2.jsonc`
- `AGENTS.md` (skip if exists)
- `CLAUDE.md` (skip if exists)
- `.gitignore` (merge with `# Governance` marker; skip if marker present)
- `specs/system.md`, `specs/errors.md`, `specs/events.md`, `specs/inbox.md` (create if missing)
- `specs/templates/*.md` (update strategy)

Shared files are unaffected by which agents are selected on re-runs.

## Re-Run Behavior

`/govern` is idempotent and additive across agents:

- **Re-run with the same selection** — applies the manifest's `update` strategy to the agent's slash commands and refreshes shared files. Project-specific files (strategy `create`) are skipped if present.
- **Re-run adding a new agent** — scaffolds the new agent's tree from scratch alongside the existing one. The existing agent's command dir, settings, and session file are not touched.
- **Re-run removing an agent** — the unified command does not delete an agent's tree on its own. If the user wants to remove an adopted agent, that is a manual cleanup operation outside this command's scope.

## Migration from 007's Multi-File Model

Existing projects that were scaffolded under the per-CLI model (i.e., `.claude/commands/govern.md` or `.augment/commands/govern.md` is the old per-CLI variant) migrate in three steps:

1. **Manual curl to replace the installed `govern.md`.** The old per-CLI `govern.md` cannot self-update to the unified version because its own fetch manifest points at the old per-CLI source path (e.g., `govern/govern-auggie.md`), which is removed from the repo. Users curl the unified file directly into the agent's command dir:

   ```bash
   curl -fsSL https://raw.githubusercontent.com/stonean/govern/main/govern/govern.md \
     > {cli-config-dir}/commands/govern.md
   ```

2. **Run `/govern`** in a new session so the unified file is loaded. Detection pre-selects whichever agent's directory exists; the user can add additional agents during the same run. Scaffolding overwrites the project's per-agent `setup.md` with the correct content from `commands/setup/{key}.md` (manifest's `update` strategy).

3. **Run `/{project}:setup`** in each adopted agent. For Auggie projects affected by the pre-`9eb9a2f` bug, the setup command's migration block strips the legacy `permissions` key from `.augment/settings.local.json` and writes proper `toolPermissions`.

No backward-compatibility alias is left at `govern/govern-auggie.md` — the manual curl is the migration path. Adding an alias would reintroduce the very file the unification is meant to remove.

## Pre-flight Checks

Before any scaffolding, the unified command verifies:

- The current directory **is** an existing git repository. If not, stop and report: "This is not a git repository. Run `git init` first."
- If a `specs/` directory already exists, this is a re-run. Report: "Existing specs/ directory found — running in update mode."

Pre-flight is identical to 007's checks; the unified command preserves them.

## Edge Cases

- **Unknown agent in `--agents=`** — if `$ARGUMENTS` lists an agent key not present in the registry, stop before scaffolding and report the unknown key with the list of valid keys. Do not partially scaffold.
- **All supported agents already adopted with `--add-agent`** — show the prompt with all agents pre-selected; if the user confirms with no additions, treat it as a routine update and continue silently. Do not error.
- **`settings.local.json` already has entries beyond the bootstrap** — the unified command only adds the curl/ls bootstrap entries if missing. It must not overwrite, deduplicate, or reorder entries added by `/{project}:setup` or by the user.
- **`govern.md` content already matches the unified version** — when the manifest's `update` strategy compares fetched content to the installed file, identical content reports as "unchanged" and avoids a redundant write. Same rule applies to per-project `setup.md` and other update-strategy files.
- **Pinned `govern.md` in `.governance.toml`** — pinned files are skipped, including `govern.md` itself. A project that pins its installed `govern.md` will not migrate from 007's multi-file model until the pin is removed. Documented as a footgun, not solved programmatically.
- **Curl fails on a single file in the manifest** — report the failure and continue with remaining files. Do not abort the entire scaffolding pass.
- **First-run prompt with no detected dirs and only one supported agent** — the prompt still appears (the agent must be explicitly chosen), but the single agent is pre-selected. Confirming is one keystroke.

## Cross-Spec Impact

[007-govern-workflow](../007-govern-workflow/spec.md) is superseded on the points where this spec inverts its decisions:

- 007 specifies "one markdown file per supported CLI" and "the target CLI is implicit." This spec replaces both with a single file and explicit runtime selection.
- 007's resolved question "CLI-specific command variants or path variable?" remains correct — the per-agent values are still substituted at govern time. Only the file count changes.

The signpost note added to 007 is delivered by this spec. 007 keeps its `done` status — its work shipped, and this spec carries forward the new design.

## Acceptance Criteria

- [x] A single `govern/govern.md` file replaces both `govern/govern.md` and `govern/govern-auggie.md` — only one govern source exists in `govern/`
- [x] Running `/govern` in a project with at least one detected agent dir (`.claude/` or `.augment/`) updates the detected agents silently — no agent-selection prompt is shown (routine-update path)
- [x] Running `/govern --add-agent` shows the agent-selection prompt with detected agents pre-selected, allowing the user to add additional agents
- [x] Running `/govern` for the first time in a project with no detected agent dirs (e.g., immediately after the curl install) shows the agent-selection prompt with the agent whose dir holds the running `govern.md` pre-selected
- [x] An explicit `--agents=...` list in `$ARGUMENTS` selects exactly those agents and bypasses the prompt
- [x] Selecting zero agents in any prompt path (add-agent or first-run) is rejected with a clear message
- [x] For each selected agent, scaffolding produces: project commands at `{cli-config-dir}/commands/{project}/`, the agent's setup source written as `setup.md`, an empty session JSON, a settings file in the agent's native format, and a copy of `govern.md` at `{cli-config-dir}/commands/govern.md`
- [x] Claude Code's `settings.local.json` uses `permissions.allow`/`permissions.deny`; Auggie's uses a `toolPermissions` array — neither agent receives the other's format
- [x] Re-running `/govern` and adding a previously unselected agent scaffolds that agent's tree alongside the existing one without modifying the existing agent's command dir, settings, or session JSON
- [x] Re-running `/govern` with the same selection updates the agent's slash commands per the manifest's `update` strategy and leaves `create`-strategy files alone
- [x] Shared files (`constitution.md`, `specs/templates/*`, `.markdownlint-cli2.jsonc`, `AGENTS.md`, `CLAUDE.md`, `.gitignore`, `specs/system.md`, `specs/errors.md`, `specs/events.md`, `specs/inbox.md`) are scaffolded once per run and untouched by which agents are selected
- [x] Adding support for a new agent requires only adding the agent's per-agent values (config dir, settings format, setup source) inside the unified command — no new top-level govern file is created
- [x] Existing projects scaffolded under 007's multi-file model migrate to the unified `/govern` via the documented one-time curl, after which subsequent re-runs handle updates without further manual intervention (see [Migration from 007's Multi-File Model](#migration-from-007s-multi-file-model))
- [x] The README documents the install path for each supported agent and states that subsequent agents do not require a second curl
- [x] 007's spec gains a signpost note pointing to 012, marking the multi-file design as superseded
- [x] The `CLAUDE.md` "Govern File Parity" rule is removed — there is only one govern file
- [x] An unknown agent key in `--agents=` is rejected with a clear error before any scaffolding occurs
- [x] `settings.local.json` entries added by `/{project}:setup` or the user are preserved across `/govern` re-runs — only the bootstrap curl/ls entries are added if missing
- [x] All shipped markdown files pass `npx markdownlint-cli2`

## Open Questions

None — all resolved.

## Resolved Questions

- **Agent selection UI** — sequential per-agent yes/no questions via `AskUserQuestion`, one question per supported agent. Each pre-selects "Yes" when the agent's config dir is detected. Universally supported in both CLIs, scales to N agents, and avoids per-CLI rendering branches. An explicit `--agents=` list in `$ARGUMENTS` skips the prompt entirely.
- **Per-agent values: inline conditionals vs. fragments** — inline, structured as an "agent registry" table near the top of `govern.md`. The rest of the file references registry values (config dir, settings format, setup source, rules-file note) rather than branching on agent name. Adding a new agent is a one-row addition to the registry. Fragments were rejected because they reintroduce multi-file maintenance, fetch-failure modes, and a bootstrap question — the exact burdens this spec exists to remove.
- **Setup source layout** — move to per-agent directory: `commands/setup/{agent}.md` (e.g., `commands/setup/claude.md`, `commands/setup/auggie.md`). The setup source path becomes derivable from the agent key, removing the asymmetric `setup.md` / `setup-auggie.md` naming. Migration is a pair of file moves; only consumer is `govern.md` itself. The agent registry doesn't need an explicit `setup_source` column — it's `commands/setup/{agent}.md` by convention.
- **Bootstrap install path** — keep one curl snippet per supported agent in the README, with a short framing line that the multi-agent flow is available on re-run. A canonical single-path install would tax every single-agent user (the common case) to simplify the rare multi-agent bootstrap; the framing line addresses discoverability without that tax.
- **Add-an-agent contract** — five fields per agent: `key`, `name`, `config_dir`, `settings_template`, `rules_file_note`. The setup source path, README install snippet, session JSON path, and project commands dir are all derived from these. `settings_template` stores the **fully populated** bootstrap JSON in each agent's native syntax — Claude uses `Bash(...)` allow strings under `permissions.allow`, Auggie uses `launch-process` entries with `shellInputRegex` under `toolPermissions`. The registry only seeds the curl/ls entries needed during `/govern` itself; the full permission set lives in `commands/setup/{key}.md` and is applied later via `/{project}:setup`. Adding a new agent is three edits: registry row, setup source file, README curl snippet.
- **Stale per-CLI govern files on migration** — replacing the installed `govern.md` is sufficient; no special detection or cleanup logic. The manifest's `update` strategy covers `govern.md` and per-project `setup.md`, which is enough to overwrite all on-disk legacy artifacts. No version stamping or sentinel comments. Migration is a manual curl + `/govern` + `/{project}:setup` (see "Migration from 007's Multi-File Model" section). Pinning `govern.md` in `.governance.toml` is documented as a footgun — pinned files won't migrate, and that's acceptable.
- **First-run default agent selection** — dissolved by the revised agent-selection rules. Routine `/govern` runs are silent (auto-detect path), so first-run is the only case that prompts. On first run, detection has nothing to lock onto except the agent whose dir holds the running `govern.md` — that agent is pre-selected. No separate "no detection" branch is needed.
- **Routine update vs. agent selection — single command or two?** — single command, with default behavior driven by detection. `/govern` with no flags performs a silent update of detected agents (the common case). `/govern --add-agent` triggers the agent-selection prompt with detected agents pre-selected. `/govern --agents=...` bypasses the prompt entirely. A separate `/govern-add-agent` command was rejected because it adds a second entry point to discover and a second file to scaffold into every agent's commands dir, with no upside the flag doesn't already provide.
- **Deselecting an adopted agent** — stays manual. `/govern` is purely additive/idempotent; it never deletes an adopted agent's tree. `--agents=` has scope-limiting effect (scaffolds only the listed agents), not removal effect — agents not in the list are left untouched. Removing an adopted agent is a manual `rm -rf {config_dir}` operation outside `/govern`'s scope, justified by the destructive nature of the action and the rarity of the use case.
