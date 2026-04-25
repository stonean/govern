# 012 — Multi-Agent Govern Plan

## Overview

Replace the two per-CLI govern files (`govern/govern.md` for Claude Code, `govern/govern-auggie.md` for Auggie) with a single `govern/govern.md` that carries an agent registry. The registry is a small markdown table near the top of the file listing each supported agent and the per-agent values needed to scaffold it (`key`, `name`, `config_dir`, `settings_template`, `rules_file_note`). All other sections (pre-flight, manifest, scaffolding loop, post-scaffolding output) reference the registry rather than branching on agent name.

`/govern` resolves which agents to scaffold using a fixed precedence: `--agents=` flag, then auto-detect (silent routine update), then a sequential `AskUserQuestion` prompt (one yes/no per agent). The scaffolding loop runs once per selected agent, writing the agent's commands, settings, session JSON, setup source (from `commands/setup/{key}.md`), and a copy of `govern.md` into the agent's `commands/` directory.

The setup source files move from `commands/setup.md` and `commands/setup-auggie.md` to per-agent paths `commands/setup/{key}.md`. This makes the setup path derivable from the registry key and removes the asymmetric naming.

The `CLAUDE.md` "Govern File Parity" rule is deleted (only one govern file exists), and `specs/007-govern-workflow/spec.md` gains a signpost note pointing to this spec.

## Technical Decisions

### Agent registry as a markdown table inside `govern.md`

The registry lives at the top of `govern/govern.md` as a fenced markdown table — readable from the file itself, no external lookup, easy to extend. The agent reads the table and iterates the rows during scaffolding. This is the same pattern 007 uses for its file manifest.

Schema (five fields per row):

| Field | Description |
| --- | --- |
| `key` | Lowercase identifier used in flags (`--agents=claude`), file paths (`commands/setup/{key}.md`), and prompt text |
| `name` | Human-readable label shown in prompts and post-scaffolding summary |
| `config_dir` | Project-relative config directory (e.g., `.claude`, `.augment`) |
| `settings_template` | Bootstrap JSON for `{config_dir}/settings.local.json` in the agent's native format — only the curl/ls entries needed during `/govern` itself |
| `rules_file_note` | Short note about the agent's relationship to `CLAUDE.md`, shown in the post-scaffolding summary |

Derived values (not stored — computed by convention):

- Setup source path: `commands/setup/{key}.md`
- Session JSON path: `{config_dir}/{project}-session.json`
- Project commands directory: `{config_dir}/commands/{project}/`
- Govern install path: `{config_dir}/commands/govern.md`

Rejected alternatives:

- **Inline conditionals on agent name** — would scatter `if claude / else auggie` branches throughout the file. Rejected because adding a new agent would touch every branch. The registry centralizes per-agent values into a single addressable structure.
- **External fragments fetched per agent** — would reintroduce multi-file fetch failure modes and a bootstrap question. The whole point of unification is to avoid those.

### Setup source layout: `commands/setup/{key}.md`

Move `commands/setup.md` → `commands/setup/claude.md` and `commands/setup-auggie.md` → `commands/setup/auggie.md`. Setup source paths become derivable from the registry key (`commands/setup/{key}.md`), so the registry doesn't need an explicit `setup_source` field.

The file contents do not change — both files already use `{cli-config-dir}` for paths and contain agent-native permission entries. Only the location changes.

The Claude Code govern instance under `.claude/commands/gov/setup.md` is regenerated from `commands/setup/claude.md` to satisfy the existing "Command File Parity" rule.

### Agent selection precedence

`/govern` determines selection in this order, stopping at the first match:

1. **Explicit list** — `$ARGUMENTS` contains `--agents=claude,auggie`. Use exactly those keys, no prompt. Reject unknown keys before any scaffolding.
2. **Auto-detect (no `--add-agent`)** — list registry entries whose `config_dir` exists in the project. If at least one is found, scaffold those silently (routine-update path). This is the common case on re-runs.
3. **Add-agent prompt** — triggered when `$ARGUMENTS` contains `--add-agent`, OR when no detected dirs exist (first run). Iterate the registry and ask one yes/no `AskUserQuestion` per agent. Pre-select "Yes" when the agent's `config_dir` is detected, OR when this is first run AND the agent's key matches the dir holding the running `govern.md` (detected via the running command's install path).

Rationale: routine `/govern` re-runs (the common case) must be silent — adding a prompt every run would tax users for the rare multi-agent bootstrap. Sequential per-agent yes/no questions are universally supported in both CLIs and scale to N agents without per-CLI rendering branches.

If the user selects zero agents in any prompt path, reject with: "At least one agent must be selected." Do not partially scaffold.

### First-run "current agent" detection

On first run with no detected `config_dir` (no `.claude/`, no `.augment/`), the prompt still appears so the user explicitly chooses. To pre-select the right agent, the running `govern.md` infers its own location: it is invoked from `{config_dir}/commands/govern.md`, so the agent whose `config_dir` matches the running file's parent path is the one to pre-select.

If the agent cannot infer its own path (CLI doesn't expose it), the prompt iterates with no pre-selection — the user picks explicitly. This is acceptable on first run because the user just curled the file and knows which agent they're in.

### Per-agent scaffolding loop

For each selected agent, run these steps with registry values substituted:

1. Resolve `{cli-config-dir}` to the agent's `config_dir`.
2. Process the slash-command manifest into `{config_dir}/commands/{project}/`. Substitute `{project}` and `{cli-config-dir}` in each copied file.
3. Fetch `commands/setup/{key}.md` from the governance repo and write it as `{config_dir}/commands/{project}/setup.md` (strategy: `update`).
4. Create `{config_dir}/{project}-session.json` with `{}` if missing (strategy: `create`).
5. Read or create `{config_dir}/settings.local.json` and merge in entries from the agent's `settings_template` if missing. Do not reorder, deduplicate, or overwrite entries beyond the bootstrap set.
6. Fetch `govern/govern.md` and write it to `{config_dir}/commands/govern.md` (strategy: `update`). Run the post-write integrity check: file must start with `# Govern`. If not, re-fetch.

Shared files (`constitution.md`, `.markdownlint-cli2.jsonc`, `AGENTS.md`, `CLAUDE.md`, `.gitignore`, `specs/system.md`, `specs/errors.md`, `specs/events.md`, `specs/inbox.md`, `specs/templates/*`) are scaffolded **once per run** outside the loop, regardless of how many agents are selected.

### `settings_template` content

The registry's `settings_template` is the **fully populated** bootstrap JSON in each agent's native syntax. Only the bootstrap entries (curl, ls) — enough for `/govern` to run without permission prompts. The full permission set lives in `commands/setup/{key}.md` and is applied later via `/{project}:setup`.

Claude Code (`permissions.allow`):

```json
{ "permissions": { "allow": ["Bash(curl *)", "Bash(ls *)"], "deny": [] } }
```

Auggie (`toolPermissions`):

```json
{
  "toolPermissions": [
    { "toolName": "launch-process", "shellInputRegex": "^curl ", "permission": { "type": "allow" } },
    { "toolName": "launch-process", "shellInputRegex": "^ls ", "permission": { "type": "allow" } }
  ]
}
```

Merge rules: read existing file, append any missing entries from `settings_template`, write back. Preserve all entries the user or `/{project}:setup` has already added — only the bootstrap set is touched.

### Migration: 007's per-CLI files → unified file

No backward-compatibility alias at `govern/govern-auggie.md`. Migration is three manual steps documented in the spec's "Migration from 007's Multi-File Model" section:

1. Curl the unified `govern.md` into the agent's command dir, overwriting the old per-CLI file.
2. Run `/govern` in a new session — detection pre-selects the agent whose dir exists.
3. Run `/{project}:setup` — for Auggie, this strips any legacy `permissions` key written under the pre-`9eb9a2f` bug.

A backward-compat alias was rejected because it would reintroduce the file the unification is meant to remove.

The pre-`9eb9a2f` Auggie bug is already handled by the existing migration block in `commands/setup-auggie.md` (now `commands/setup/auggie.md`). No new code needed for that path.

### "Govern File Parity" rule removed; "Command File Parity" rule kept

The CLAUDE.md "Govern File Parity" rule disappears — there is only one govern file. The "Command File Parity" rule (between `commands/` and `.claude/commands/gov/`) is unrelated and stays in place.

### 007's signpost note

`specs/007-govern-workflow/spec.md` already has a signpost from 011 at the top. Add a second signpost paragraph below it pointing to 012, marking the multi-file design as superseded. 007 keeps its `done` status — its work shipped, and 012 carries forward the new design.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `govern/govern.md` | Modify | Replace with unified content: agent registry table, selection logic, per-agent scaffolding loop |
| `govern/govern-auggie.md` | Delete | Superseded by unified `govern.md` |
| `commands/setup/claude.md` | Create | Move from `commands/setup.md` (no content change) |
| `commands/setup/auggie.md` | Create | Move from `commands/setup-auggie.md` (no content change) |
| `commands/setup.md` | Delete | Replaced by `commands/setup/claude.md` |
| `commands/setup-auggie.md` | Delete | Replaced by `commands/setup/auggie.md` |
| `.claude/commands/gov/setup.md` | Modify | Re-derive from `commands/setup/claude.md` (Command File Parity) — no content change expected |
| `README.md` | Modify | Add framing line under each curl snippet noting the multi-agent flow is available on re-run |
| `CLAUDE.md` | Modify | Remove the "Govern File Parity" section |
| `specs/007-govern-workflow/spec.md` | Modify | Add signpost paragraph pointing to 012 |
| `specs/spec.md` | Modify | Refresh the "Auggie permissions setup" resolved-note to reference `commands/setup/auggie.md` and the unified `govern.md` (added during implementation — original plan missed this stale note) |
| `specs/012-multi-agent-govern/data-model.md` | Create | Document the agent registry schema |
| `specs/012-multi-agent-govern/spec.md` | Modify | Status to `done` after acceptance criteria are verified (final task) |

## Trade-offs

### Considered: keep two govern files, just deduplicate via shared fragments

Each per-CLI file would `curl` shared fragments at govern time. Rejected — fragments reintroduce a fetch-failure mode (the bootstrap pulls more files), make the system harder to reason about offline, and re-create the parity burden the spec is trying to eliminate. The registry approach keeps one file with the per-agent surface area boxed into a single table.

### Considered: separate `/govern-add-agent` command

A second slash command for the multi-agent path. Rejected because it doubles the discovery surface (two commands to remember, two files to scaffold into every project), with no upside the `--add-agent` flag doesn't already provide.

### Considered: detection-only auto-add of new agents

If a new agent's `config_dir` is detected on a routine re-run, scaffold it silently. Rejected — adopting a new agent should be an explicit user decision, not a side effect of a directory existing. The `--add-agent` flag is the explicit path.

### Considered: removal of an adopted agent via `/govern`

Add a `--remove=auggie` flag that deletes `.augment/` and its scaffolding. Rejected — destructive, rarely needed, and the user can `rm -rf .augment/` themselves. `/govern` stays purely additive/idempotent.

### Known limitation: pinned `govern.md` blocks migration

A project that lists `.claude/commands/govern.md` or `.augment/commands/govern.md` in `.governance.toml` `pinned.files` will not migrate to the unified file via `/govern` — the pin overrides the `update` strategy. Documented as a footgun, not solved programmatically. Removing the pin is a one-line edit.

### Known limitation: agent self-path inference

The "first-run pre-select the agent whose dir holds the running `govern.md`" logic depends on the running CLI exposing the running command's install path to the prompt. If a future CLI doesn't, the prompt falls back to no pre-selection. The user types one extra keystroke; nothing breaks.

## Open Questions Resolved

All eight open questions are resolved in the spec's "Resolved Questions" section. No new questions surfaced during planning.

- **Agent selection UI** — sequential per-agent `AskUserQuestion` yes/no, with `--agents=` bypass.
- **Per-agent values: inline conditionals vs. fragments** — registry table inside `govern.md`.
- **Setup source layout** — `commands/setup/{key}.md`, derived from registry key.
- **Bootstrap install path** — keep one curl per supported agent in the README, with a framing line about the multi-agent flow on re-run.
- **Add-an-agent contract** — five fields per agent (`key`, `name`, `config_dir`, `settings_template`, `rules_file_note`); other paths derived by convention.
- **Stale per-CLI govern files on migration** — manifest's `update` strategy overwrites the installed `govern.md`; no special detection.
- **First-run default agent selection** — pre-select the agent whose `config_dir` holds the running `govern.md`.
- **Routine update vs. agent selection — single command or two?** — single `/govern`, with `--add-agent` triggering the prompt and `--agents=` bypassing it.
- **Deselecting an adopted agent** — manual `rm -rf {config_dir}` outside `/govern`'s scope.
