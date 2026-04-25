# Govern

Bootstrap governance in an existing project. This command fetches templates from the governance repo, scaffolds governance files for one or more AI coding CLIs, resolves placeholders, and displays next steps.

The same `govern.md` supports every agent the framework knows about. The set of supported agents lives in the **Agent Registry** below; per-agent values are looked up by registry key during scaffolding.

## Agent Registry

The registry lists every supported agent. Per-agent paths and behaviors are derived from these rows — the rest of this file references registry values, not agent names.

| `key` | `name` | `config_dir` | `settings_template` | `rules_file_note` |
| --- | --- | --- | --- | --- |
| `claude` | Claude Code | `.claude` | `{ "permissions": { "allow": ["Bash(curl *)", "Bash(ls *)"], "deny": [] } }` | Claude Code reads `CLAUDE.md` natively. |
| `auggie` | Auggie | `.augment` | `{ "toolPermissions": [ { "toolName": "launch-process", "shellInputRegex": "^curl ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^ls ", "permission": { "type": "allow" } } ] }` | Auggie reads `CLAUDE.md` natively — no second rules file is needed. |

### Derived values

For each agent, these paths are computed by convention from the row above. They are **not** stored in the table.

| Derived value | Formula |
| --- | --- |
| Setup source path | `commands/setup/{key}.md` |
| Session JSON path | `{config_dir}/{project}-session.json` |
| Project commands directory | `{config_dir}/commands/{project}/` |
| Govern install path | `{config_dir}/commands/govern.md` |

### Adding a new agent

A new agent is one row above plus two satellite files:

1. Append a row with the five required fields.
2. Add `commands/setup/{key}.md` with the agent's full permission set in its native settings format.
3. Add a curl snippet for the new agent to the README's adoption section.

No other changes are required.

## Inputs

Collect from `$ARGUMENTS` or prompt the user interactively. When using AskUserQuestion, every question **must** include an `options` array with 2–4 example choices (the user can always select "Other" for custom input):

1. **Project name** — lowercase, alphanumeric, hyphens allowed. Used for `{project}` placeholder substitution and command directory naming. If `$ARGUMENTS` contains a single non-flag word, use it as the project name and prompt for the remaining inputs. Example options: the current directory name, `my-service`.
2. **Project description** — one-line description for AGENTS.md. Example options: `A new microservice`, `CLI tool for X`.
3. **Primary language(s)** — comma-separated list for .gitignore language patterns. Example options: `Go`, `Python`, `Node`, `Go, Python`.

Validate the project name: must be lowercase, alphanumeric, and hyphens only. If invalid, reject with: "Project name must be lowercase, alphanumeric, and hyphens only."

Recognized flags in `$ARGUMENTS`:

- `--agents=key1,key2,...` — explicit list of agent keys to scaffold. Bypasses any prompt. Reject unknown keys.
- `--add-agent` — force the agent-selection prompt even when agents are already detected.

Flags may appear in any order alongside the project name.

## Pre-flight Checks

Before any scaffolding, verify:

- The current directory **is** an existing git repository. If not, stop and report: "This is not a git repository. Run `git init` first."
- If a `specs/` directory already exists, this is a re-run. Report: "Existing specs/ directory found — running in update mode." Proceed normally; `update` strategy files will be overwritten, `create` strategy files will be skipped, `skip` strategy files will be left alone.

## Agent Selection

Determine which agents to scaffold using the first matching rule:

1. **Explicit list (`--agents=`)** — parse the comma-separated keys. For each key, look up the registry row. If any key is not present in the registry, stop before any scaffolding and report: "Unknown agent key: `{key}`. Valid keys: {comma-separated registry keys}." Do not partially scaffold. If the list is non-empty and all keys are valid, scaffold exactly those agents — no prompt.

2. **Auto-detect (default — routine update path)** — when neither `--agents=` nor `--add-agent` is present, list registry entries whose `config_dir` exists in the project. If at least one is detected, scaffold those silently with no prompt. This is the path that runs on every routine `/govern` re-run.

3. **Add-agent / first-run prompt** — triggered when `--add-agent` is present, OR when no agent dirs are detected (first run after the curl install). Iterate the registry in row order and ask one yes/no `AskUserQuestion` per agent. Pre-select "Yes" when:
   - the agent's `config_dir` exists in the project, OR
   - this is first run (no detected dirs) AND the agent's `config_dir` is the parent directory of the running `govern.md` file (i.e., the agent the user just curled into).

   If the running command cannot infer its own install path, fall back to no pre-selection — the user picks explicitly. This is acceptable on first run because the user just installed the file and knows which agent they're in.

   If the user confirms with zero agents selected, reject with: "At least one agent must be selected." Do not partially scaffold.

The user must end up with at least one selected agent in every path. Removing an adopted agent's tree is not part of this command's scope — see **Re-Run Behavior**.

## Permission Setup

For each selected agent, before fetching any files:

1. Read `{config_dir}/settings.local.json` (create it if missing, with the agent's `settings_template` from the registry).
2. Merge the agent's `settings_template` entries into the existing file: add any entries that are missing, do not deduplicate or reorder anything else, and do not overwrite entries the user or `/{project}:setup` previously added.
3. Write the file if anything was added.

This prevents repeated permission prompts during the fetch and scaffolding phases. The full permission set is applied later by `/{project}:setup`.

## Project Configuration

If `.governance.toml` exists, read it before processing the file manifest. This file is optional — if it does not exist, use default behavior for all files.

```toml
[pinned]
# Files listed here use 'skip' instead of 'update'.
# Use destination paths (after placeholder resolution).
files = [
  ".claude/commands/myapp/implement.md",
  "constitution.md",
]
```

Any file listed in `pinned.files` that would normally use `update` strategy is treated as `skip` instead. Report pinned files in the post-scaffolding summary.

A project that pins its installed `govern.md` (e.g., `.claude/commands/govern.md`) will not migrate from 007's multi-file model until the pin is removed. This is documented as a footgun, not solved programmatically.

## Migration: triage → inbox

Before processing the file manifest, check for artifacts from the pre-rename `triage` naming. Run these checks once per `/govern` invocation, regardless of how many agents are selected:

1. If `specs/triage.md` exists and `specs/inbox.md` does not — rename `specs/triage.md` to `specs/inbox.md`.
2. If `specs/triage.md` exists and `specs/inbox.md` also exists — merge items from `triage.md` into `inbox.md`, then delete `triage.md`.
3. For each selected agent, if `{config_dir}/commands/{project}/triage.md` exists — delete it.

Report any migration actions in the post-scaffolding summary.

## File Fetching

Fetch each file from the governance repo and copy it to the destination path. The source URL pattern is:

```text
https://raw.githubusercontent.com/stonean/govern/main/{source-path}
```

If a fetch fails, report the failure and continue with remaining files. Do not abort on a single fetch error.

For `update` strategy files, compare fetched content against the existing file. Only overwrite and report as "updated" if the content differs. If the content is identical, report as "unchanged" (or omit from the summary).

## Shared Files

These files are scaffolded **once per `/govern` invocation**, regardless of how many agents are selected. They are unaffected by the agent registry.

### Governance-owned shared files (strategy: update)

| Source Path | Destination Path |
| --- | --- |
| `constitution.md` | `constitution.md` |
| `.markdownlint-cli2.jsonc` | `.markdownlint-cli2.jsonc` |
| `templates/spec.md` | `specs/templates/spec.md` |
| `templates/plan.md` | `specs/templates/plan.md` |
| `templates/tasks.md` | `specs/templates/tasks.md` |
| `templates/data-model.md` | `specs/templates/data-model.md` |
| `templates/research.md` | `specs/templates/research.md` |
| `templates/scenario.md` | `specs/templates/scenario.md` |
| `templates/spec-and-plan.md` | `specs/templates/spec-and-plan.md` |

### Project-specific shared files (strategy: create)

| Source Path | Destination Path |
| --- | --- |
| `templates/system.md` | `specs/system.md` |
| `templates/errors.md` | `specs/errors.md` |
| `templates/events.md` | `specs/events.md` |
| `templates/inbox.md` | `specs/inbox.md` |

### Shared files with conflict handling

**AGENTS.md** (strategy: skip) — if it exists, leave it alone. If not, fetch `AGENTS.md` from the governance repo root and substitute `{project-name}` with the project name and `{One-line project description.}` with the project description.

**CLAUDE.md** (strategy: skip) — if it exists, leave it alone. If not, fetch `templates/claude-md.md` from the governance repo and copy it as `CLAUDE.md`. Both supported agents read `CLAUDE.md` natively (see each row's `rules_file_note`).

**.gitignore** (strategy: merge) — if it exists, check for a `# Governance` comment header. If the header exists, skip (already merged). If no header, append governance patterns below existing content:

1. Fetch `templates/gitignore` from the governance repo.
2. Append its content below a `# Governance` comment header.
3. For each primary language provided by the user, fetch from `https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore` and append below a `# {Language}` comment header.

If `.gitignore` does not exist, create it from `templates/gitignore` plus language patterns.

## Per-Agent Scaffolding

For each selected agent (in registry row order), run these steps with `{config_dir}` resolved to the agent's value and `{key}` to the agent's key.

### Project-specific files (strategy: create)

Created on first run, skipped on re-run.

| Source Path | Destination Path |
| --- | --- |
| `templates/initialize.md` | `{config_dir}/commands/{project}/initialize.md` |

### Slash commands (strategy: update)

Fetch each command template and copy it into `{config_dir}/commands/{project}/`. In each copied file, replace `{project}` with the user-provided project name and `{cli-config-dir}` with `{config_dir}`.

| Source Path | Destination Path |
| --- | --- |
| `commands/about.md` | `{config_dir}/commands/{project}/about.md` |
| `commands/clarify.md` | `{config_dir}/commands/{project}/clarify.md` |
| `commands/implement.md` | `{config_dir}/commands/{project}/implement.md` |
| `commands/plan.md` | `{config_dir}/commands/{project}/plan.md` |
| `commands/question.md` | `{config_dir}/commands/{project}/question.md` |
| `commands/scenario.md` | `{config_dir}/commands/{project}/scenario.md` |
| `commands/setup/{key}.md` | `{config_dir}/commands/{project}/setup.md` |
| `commands/specify.md` | `{config_dir}/commands/{project}/specify.md` |
| `commands/status.md` | `{config_dir}/commands/{project}/status.md` |
| `commands/target.md` | `{config_dir}/commands/{project}/target.md` |
| `commands/inbox.md` | `{config_dir}/commands/{project}/inbox.md` |
| `commands/validate.md` | `{config_dir}/commands/{project}/validate.md` |
| `commands/capture.md` | `{config_dir}/commands/{project}/capture.md` |
| `commands/create.md` | `{config_dir}/commands/{project}/create.md` |

The setup row uses the agent-specific source `commands/setup/{key}.md` and writes it as the canonical `setup.md` in the project's command directory.

### Slash command cleanup

After processing the slash command manifest above, list all `.md` files in `{config_dir}/commands/{project}/`. For each file that is **not** in the slash command manifest above, **not** the `initialize.md` file, and **not** listed in `.governance.toml` `pinned.files`:

- Delete the file.
- Report it as "removed" in the post-scaffolding summary.

Files listed in `pinned.files` are never deleted — report them as "pinned (kept)" instead.

### Session state (strategy: create)

Create `{config_dir}/{project}-session.json` with empty content `{}` only if it does not already exist.

### Govern self-installation (strategy: update)

Fetch `govern/govern.md` and write it to `{config_dir}/commands/govern.md`. This is the same unified file the user is currently running, copied into every selected agent's command directory so the command is invokable from that agent on subsequent runs.

In this file (and only this file), keep `{project}` and `{cli-config-dir}` as literal placeholders — do **not** substitute. Govern itself reads `$ARGUMENTS` for the project name on each run.

After writing, run the **Post-Write Integrity Check** below.

## Placeholder Substitution

In every copied file (except `{config_dir}/commands/govern.md` for each selected agent — those keep `{project}` and `{cli-config-dir}` as literal placeholders), replace:

- `{project}` with the user-provided project name (used in commands, README)
- `{project-name}` with the user-provided project name (used in AGENTS.md template)
- `{One-line project description.}` with the user-provided description
- `{cli-config-dir}` with the agent's `config_dir`

## Post-Write Integrity Check

After writing `{config_dir}/commands/govern.md` for each selected agent, verify the file starts with `# Govern`. If it does not, the write was corrupted — report the error and re-fetch the file. Apply the check independently per agent.

## Re-Run Behavior

`/govern` is idempotent and additive across agents:

- **Re-run with the same selection** — applies the manifest's `update` strategy to the agent's slash commands and refreshes shared files. `create`-strategy files are skipped if present. `skip`-strategy files are never overwritten.
- **Re-run adding a new agent** — scaffolds the new agent's tree from scratch alongside the existing one. The existing agent's command dir, settings, and session JSON are not touched.
- **Re-run removing an agent** — this command does not delete an agent's tree on its own. Removing an adopted agent is a manual `rm -rf {config_dir}` operation outside `/govern`'s scope.

## What This Command Does NOT Do

- Modify `README.md` — the project's README is its own
- Create feature specs — the user does that via `/{project}:specify`
- Fill in AGENTS.md content — that requires project-specific knowledge
- Fill in system.md content — that requires architectural decisions
- Make git commits — the user decides when to commit
- Run `/{project}:setup` — that happens after adoption, interactively
- Delete an agent's adopted tree — manual cleanup

## Edge Cases

- **Unknown agent key in `--agents=`** — stop before scaffolding; report the unknown key with the list of valid keys.
- **All supported agents already adopted with `--add-agent`** — show the prompt with all agents pre-selected; if the user confirms with no additions, treat it as a routine update and continue silently.
- **`settings.local.json` already has entries beyond the bootstrap** — only add the curl/ls bootstrap entries if missing. Do not overwrite, deduplicate, or reorder entries added by `/{project}:setup` or by the user.
- **`govern.md` content already matches the unified version** — when the manifest's `update` strategy compares fetched content to the installed file, identical content reports as "unchanged" and avoids a redundant write. Same rule applies to per-project `setup.md` and other update-strategy files.
- **Pinned `govern.md` in `.governance.toml`** — pinned files are skipped, including `govern.md` itself. A project that pins its installed `govern.md` will not migrate from 007's multi-file model until the pin is removed.
- **Curl fails on a single file in the manifest** — report the failure and continue with remaining files. Do not abort the entire scaffolding pass.
- **First-run prompt with no detected dirs and only one supported agent** — the prompt still appears (the agent must be explicitly chosen), but the single agent is pre-selected. Confirming is one keystroke.
- **Running `govern.md` cannot infer its own install path** — fall back to no pre-selection in the first-run prompt. The user picks explicitly.

## Post-Scaffolding Output

After scaffolding, display:

- Summary of files created, updated, unchanged, skipped, pinned, merged, and removed — grouped by agent for per-agent files, with shared files in their own group
- For each scaffolded agent, the agent's `rules_file_note` from the registry
- Any fetch failures encountered
- Self-update notice (if applicable — see below)
- Migration guidance (if applicable — see below)
- Next steps (varies by mode):

### Self-update notice

If any selected agent's `{config_dir}/commands/govern.md` was reported as "updated" (i.e., the fetched version differs from the previously installed version), append this notice after the file summary and before next steps:

> **The govern command itself was updated.** Start a new session and re-run `/govern` to apply the latest changes.

This notice is not shown on first run (the file is new, not updated) or when the govern command was unchanged across all agents.

### Migration from 007's multi-file model

If the previously installed file at `{config_dir}/commands/govern.md` was the old per-CLI variant (i.e., the unified file replaced it on this run), include this notice in the summary:

> **Migrated from per-CLI govern files.** Run `/{project}:setup` in each adopted agent to apply the full permission set. For Auggie, this also strips any legacy `permissions` key written by older versions.

The unified file's `update` strategy is sufficient to overwrite both old per-CLI variants. No version stamping or sentinel comment is needed.

### First run (no existing `specs/` directory)

---

**Governance adopted successfully.**

Adopted agents: {comma-separated `name` of selected agents}.

Next steps:

1. Run `/{project}:setup` in each adopted agent to configure the full permission set.
2. Fill in `AGENTS.md` — tech stack, project structure, code style, testing conventions, gotchas.
3. Fill in `specs/system.md` — architecture, request lifecycle, shared infrastructure.
4. Populate `specs/inbox.md` with known issues and bugs.
5. Run `/{project}:inbox` to migrate items to specs and scenarios.
6. Create your first feature spec: `/{project}:specify {feature description}`.

To adopt an additional agent later, re-run `/govern --add-agent`.

---

### Update mode (existing `specs/` directory detected)

---

**Governance updated successfully.**

Updated agents: {comma-separated `name` of selected agents}.

Review changes to updated files and commit when ready. To adopt an additional agent, re-run `/govern --add-agent`.

---

## Idempotency

This command is safe to run again. Files with `update` strategy are always overwritten with the latest governance version — unless pinned in `.governance.toml`, in which case they are skipped. Files with `create` strategy skip existing files. The `.gitignore` merge checks for the `# Governance` marker before appending. `skip` strategy files are never overwritten.

Re-runs are additive across agents — adopting a new agent leaves existing agents' files untouched.

## Directory Creation

Create intermediate directories as needed (e.g., `specs/`, `specs/templates/`, `{config_dir}/commands/{project}/`).
