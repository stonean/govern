# Govern

Bootstrap governance in an existing project. This command fetches templates from the governance repo, scaffolds governance files for one or more AI coding CLIs, resolves placeholders, and displays next steps.

The same `govern.md` supports every agent the framework knows about. The set of supported agents lives in the **Agent Registry** below; per-agent values are looked up by registry key during scaffolding.

## Agent Registry

The registry lists every supported agent. Per-agent paths and behaviors are derived from these rows — the rest of this file references registry values, not agent names.

| `key` | `name` | `config_dir` | `settings_template` | `rules_file_note` |
| --- | --- | --- | --- | --- |
| `claude` | Claude Code | `.claude` | `{ "permissions": { "allow": ["Bash(curl *)", "Bash(ls *)", "Bash(tar *)", "Bash(mktemp *)", "Read(/private/var/folders/**/T/govern-*/**)", "Read(//private/var/folders/**/T/govern-*/**)", "Read(/var/folders/**/T/govern-*/**)", "Read(//var/folders/**/T/govern-*/**)", "Read(/tmp/govern-*/**)", "Read(//tmp/govern-*/**)"], "deny": [] } }` | Claude Code reads `CLAUDE.md` natively. |
| `auggie` | Auggie | `.augment` | `{ "toolPermissions": [ { "toolName": "launch-process", "shellInputRegex": "^curl ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^ls ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^tar ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^mktemp ", "permission": { "type": "allow" } } ] }` | Auggie reads `CLAUDE.md` natively — no second rules file is needed. |

### Derived values

For each agent, these paths are computed by convention from the row above. They are **not** stored in the table.

| Derived value | Formula |
| --- | --- |
| Configure source path | `framework/bootstrap/configure/{key}.md` |
| Session JSON path | `{config_dir}/{project}-session.json` |
| Project commands directory | `{config_dir}/commands/{project}/` |
| Govern install path | `{config_dir}/commands/govern.md` |

### Adding a new agent

A new agent is one row above plus two satellite files:

1. Append a row with the five required fields.
2. Add `framework/bootstrap/configure/{key}.md` with the agent's full permission set in its native settings format.
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
2. Merge the agent's `settings_template` entries into the existing file: add any entries that are missing, do not deduplicate or reorder anything else, and do not overwrite entries the user or `/{project}:configure` previously added.
3. Write the file if anything was added.

This prevents repeated permission prompts during the fetch and scaffolding phases. The full permission set is applied later by `/{project}:configure`.

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

## Frontmatter Migration

If `specs/` does not exist (first run), skip this section — there is nothing to migrate.

Bring existing spec and scenario files into the YAML frontmatter format declared in `framework/constitution.md` §text-first-artifacts. Migration is idempotent: re-running on an already-migrated project produces no further metadata changes.

### Precheck

Run `git status --porcelain -- specs/` (project-relative). If the output is non-empty, refuse with:

> Migration requires a clean working tree under `specs/`. Commit or stash your changes, then re-run.

Exit before any modifications. Unrelated in-flight work outside `specs/` does not block migration.

### Walk

For each file matching one of:

- `specs/**/spec.md`
- `specs/**/spec-and-plan.md`
- `specs/**/scenarios/*.md`

Determine whether the file needs migration:

- Read the first non-blank line of the file. If it is `---`, the file already has frontmatter — skip with reason "already frontmatter."
- Otherwise, scan the first few lines after the heading for bold-prefix metadata patterns (`**Status:**`, `**Dependencies:**`, `**spec-ref:**`). If at least one is found, the file needs migration.
- If no bold-prefix lines are present and no frontmatter exists, skip with reason "no metadata to migrate."

Skip files that appear in `.governance.toml` `pinned.files` with reason "pinned." The adopter is responsible for migrating pinned files manually.

### Convert

For each file that needs migration:

**Spec files** (`spec.md`, `spec-and-plan.md`):

- Extract `**Status:** {value}` and `**Dependencies:** {value}` from the body.
- For dependencies, parse the comma-separated slug list. The literal value `none` becomes an empty list (`[]`).
- Preserve any additional bold-prefix fields the project may have added (e.g., `**Track:** lightweight` becomes `track: lightweight` under the open-schema rule).
- Construct the YAML frontmatter block:

  ```yaml
  ---
  status: {value}
  dependencies: [{slug, slug, ...}]
  tags: []
  ---
  ```

- Remove the bold-prefix lines from the body.
- Insert the frontmatter block at the very top of the file, with one blank line separating it from the heading.

**Scenario files** (`scenarios/{slug}.md`):

- Extract `**spec-ref:** {value}` from the body.
- Construct the YAML frontmatter block:

  ```yaml
  ---
  spec-ref: "{value}"
  tags: []
  ---
  ```

  Quote the `spec-ref` value because it conventionally contains an em-dash and spaces.

- Remove the bold-prefix line from the body.
- Insert the frontmatter block at the very top of the file, with one blank line separating it from the heading.

### Edge cases

- **Partially migrated file** (frontmatter present and bold-prefix lines also present in body): the precheck above treats this as "already frontmatter" and skips. The user may run a manual cleanup pass; the migration does not attempt mixed-state recovery.
- **Malformed bold-prefix metadata** (e.g., missing `**Status:**` line, typo in field name, unparseable value): log a warning to the summary as `skipped (malformed metadata): {file path}` with a brief reason. The user repairs manually before re-running.
- **Bold-prefix metadata with custom fields**: preserved as additional frontmatter fields under the open-schema rule.

### Summary

Print a per-file summary at the end of the migration step:

- `migrated: {file path}` for converted files
- `skipped (already frontmatter): {file path}` for files that were already in the new format
- `skipped (pinned): {file path}` for files listed in `.governance.toml`
- `skipped (no metadata to migrate): {file path}` for files without recognizable metadata
- `skipped (malformed metadata): {file path} — {reason}` for files that could not be parsed

The user reviews the result via `git diff` and commits or aborts via `git restore`. No backup directory is created — git is the recovery mechanism.

## File Fetching

Files from the governance repo are sourced from a single archive download, extracted into a temp directory, and resolved as local paths for the rest of the run. Per-language `.gitignore` patterns from `github.com/github/gitignore` are **not** part of this archive — they remain separate `curl` calls (see the **.gitignore** subsection of **Shared Files** below).

### Archive fetch and extract

Issue exactly one `curl` against GitHub's repo-archive endpoint:

```text
https://github.com/stonean/govern/archive/refs/heads/main.tar.gz
```

`curl -fsSL` follows the 302 redirect to `codeload.github.com`. The archive's top-level directory is `govern-main/`; the framework files live at `govern-main/framework/...` after extraction.

After fetching:

1. Create a **new** temp directory on every run: `mktemp -d -t govern-XXXXXX`. On macOS/Linux this lands under `$TMPDIR` or `/tmp`. Never reuse a directory from a prior run, even if one is still on disk — a fresh fetch is the only way `/govern` picks up upstream changes, so the archive must be re-downloaded each invocation.
2. Extract the archive into the temp directory: `tar -xzf {archive} -C {tempdir}`.
3. Compute the framework root: `{tempdir}/govern-main/`. Treat this as the local mirror of the governance repo for the rest of the run.

If the fetch or extraction fails — non-zero exit from `curl` or `tar`, or a missing `govern-main/` directory after extract — abort the run with this error and do not continue scaffolding:

> Failed to fetch or extract the governance archive ({reason}). Re-run after checking network connectivity, or report this if it persists.

A missing archive means **every** manifest entry would be missing, so partial scaffolding is impossible — the abort is the correct behavior.

### Per-file resolution

For each manifest entry below (in **Shared Files**, **Per-Agent Scaffolding**, and the workflow-recommendation flow):

1. Compute the local source path: `{tempdir}/govern-main/{source-path}`.
2. If the local source path does not exist — the file was renamed, removed upstream, or the manifest is out of sync — warn `Source not found in archive: {source-path}; skipping.` and continue with the remaining entries. This preserves the "do not abort on a single fetch error" guarantee at the per-entry level, even though the archive itself is fetched once.
3. Apply the entry's strategy (`update`, `create`, `skip`, `merge`, `pinned`) using the local file as the new content. For `update` strategy, compare the local file against the existing destination file; only overwrite and report as "updated" if the content differs. If the content is identical, report as "unchanged" (or omit from the summary). Same semantics as before — no network round-trip per file.
4. Apply placeholder substitution after reading the local source, before writing to the destination. Same rules as documented in **Placeholder Substitution** below, including the `govern.md` self-install exception that keeps `{project}` and `{cli-config-dir}` literal.

### Cleanup

`/govern` does not delete the temp directory. The path is logged in the post-scaffolding summary (and, on abort, in the error message) so the user can inspect it if needed. Both macOS (`/var/folders/.../T/`) and Linux (`/tmp` on systemd-tmpfiles distros) sweep their temp directories automatically; a few hundred KB of extracted files waiting for the next sweep is acceptable in exchange for not granting an `rm -rf` permission to the bootstrap.

The leftover directory is for inspection only — the next `/govern` run creates its own fresh temp directory via `mktemp` and never reuses a prior extract.

## Shared Files

These files are scaffolded **once per `/govern` invocation**, regardless of how many agents are selected. They are unaffected by the agent registry.

### Governance-owned shared files (strategy: update)

| Source Path | Destination Path |
| --- | --- |
| `framework/constitution.md` | `constitution.md` |
| `framework/rules/security-backend.md` | `specs/security-backend.md` |
| `framework/rules/security-frontend.md` | `specs/security-frontend.md` |
| `.markdownlint-cli2.jsonc` | `.markdownlint-cli2.jsonc` |
| `framework/templates/spec/spec.md` | `specs/templates/spec.md` |
| `framework/templates/spec/plan.md` | `specs/templates/plan.md` |
| `framework/templates/spec/tasks.md` | `specs/templates/tasks.md` |
| `framework/templates/spec/data-model.md` | `specs/templates/data-model.md` |
| `framework/templates/spec/research.md` | `specs/templates/research.md` |
| `framework/templates/spec/scenario.md` | `specs/templates/scenario.md` |
| `framework/templates/spec/spec-and-plan.md` | `specs/templates/spec-and-plan.md` |
| `framework/workflows/registry.json` | `workflows/registry.json` |

### Project-specific shared files (strategy: create)

| Source Path | Destination Path |
| --- | --- |
| `framework/templates/project/system.md` | `specs/system.md` |
| `framework/templates/project/errors.md` | `specs/errors.md` |
| `framework/templates/project/events.md` | `specs/events.md` |
| `framework/templates/project/inbox.md` | `specs/inbox.md` |

### Shared files with conflict handling

**AGENTS.md** (strategy: skip) — if it exists, leave it alone. If not, fetch `framework/templates/project/agents.md` from the governance repo and copy it as `AGENTS.md`, substituting `{project-name}` with the project name and `{One-line project description.}` with the project description.

**CLAUDE.md** (strategy: skip) — if it exists, leave it alone. If not, fetch `framework/templates/project/claude-md.md` from the governance repo and copy it as `CLAUDE.md`. Both supported agents read `CLAUDE.md` natively (see each row's `rules_file_note`).

**.gitignore** (strategy: merge) — if it exists, check for a `# Governance` comment header. If the header exists, skip (already merged). If no header, append governance patterns below existing content:

1. Fetch `framework/templates/project/gitignore` from the governance repo.
2. Append its content below a `# Governance` comment header.
3. For each primary language provided by the user, fetch from `https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore` and append below a `# {Language}` comment header.

If `.gitignore` does not exist, create it from `framework/templates/project/gitignore` plus language patterns.

## Security Audit (brownfield)

Run a one-time security audit when the project newly receives a security rule file alongside existing feature specs. This is the brownfield-adoption hook described in `specs/008-security-rules/spec.md` — it routes findings through `specs/inbox.md` so the adopter can triage them via `/{project}:groom` at their own pace, rather than having every legacy spec immediately fail validate.

### Trigger

Run the audit only when **both** conditions hold after the **Shared Files** manifest pass has completed:

1. At least one of `specs/security-backend.md` or `specs/security-frontend.md` was **newly created** by the manifest pass (the destination file did not exist before this run). A file that was merely updated or unchanged does not trigger the audit.
2. The project contains at least one feature spec directory under `specs/` matching the `NNN-*` pattern (zero-padded, three-digit prefix followed by a hyphen and a slug).

If either condition fails, skip this section silently — no output, no finding, no inbox entry. This covers the two routine cases:

- **Greenfield adoption** — no `specs/NNN-*/` directories exist, so the audit has nothing to scan against.
- **Routine re-run** — the rule files were created on a prior run; the manifest pass reports them as "updated" or "unchanged" rather than "created".

### Loading rule files

For each rule file that passed the trigger:

1. Read the file from its destination path (`specs/security-backend.md` or `specs/security-frontend.md`).
2. Apply the same integrity checks `/{project}:validate` uses for the security-rule check section: well-formed level-3 headings of the form `### {ID}`, the four required fields (Statement, Rationale, Verification, Source), an ID matching `{FE|BE}-{CATEGORY}-{NNN}`, and no duplicate IDs within the file.
3. If a file fails any integrity check, report `Security audit: {path} failed to load — {reason}; skipping audit for this file.` and continue with the other rule file (if applicable). Do not abort the surrounding govern run.

This mirrors validate's posture — partial or guessed-at parsing produces unreliable findings, so an unloadable file is treated as absent for audit purposes.

### Per-rule check

For each rule that loaded successfully:

1. Identify the artifacts in scope: `specs/NNN-*/spec.md`, `specs/NNN-*/spec-and-plan.md`, `specs/NNN-*/plan.md`, and any `specs/NNN-*/scenarios/*.md`.
2. Read the rule's **Verification** field. The field describes the trigger — what makes the rule applicable to a given artifact — and the commitment the artifact must include when triggered.
3. For each artifact whose content fires the rule's trigger but does not include the required commitment, produce one finding.

Rules whose Verification trigger does not fire for any artifact produce no finding (the contextual-application property — silently inert when no spec exercises the rule's surface).

### Writing findings to the inbox

Each finding is one line appended to `specs/inbox.md`:

```text
- [ ] {Rule ID}: {affected artifact path} does not address — {one-line summary}
```

The `{one-line summary}` describes the gap concretely (e.g., `does not name a memory-hard password hashing algorithm`, `does not specify an output encoding strategy`). Prefixing each line with the rule ID makes related findings group naturally during `/{project}:groom` and gives the adopter a stable handle for cross-referencing.

### Deduplication

Before appending each finding, scan the existing `specs/inbox.md` (if it exists) for any line beginning with `- [ ] {Rule ID}: {affected artifact path}` — the prefix up to the first em-dash. If a matching line is already present, skip the new finding. This makes the audit safe to re-trigger after a user deletes and re-installs a rule file.

Findings the user has already groomed (lines that have been removed or rewritten) are not re-emitted — once the adopter has triaged a finding, govern does not resurrect it.

### Audit summary

Track the count of newly appended findings (post-deduplication). The total is reported by **Post-Scaffolding Output**; when the count is zero, the audit-summary line is omitted entirely.

## Per-Agent Scaffolding

For each selected agent (in registry row order), run these steps with `{config_dir}` resolved to the agent's value and `{key}` to the agent's key.

### Project-specific files (strategy: create)

Created on first run, skipped on re-run.

| Source Path | Destination Path |
| --- | --- |
| `framework/templates/project/initialize.md` | `{config_dir}/commands/{project}/initialize.md` |

### Slash commands (strategy: update)

Fetch each command template and copy it into `{config_dir}/commands/{project}/`. In each copied file, replace `{project}` with the user-provided project name and `{cli-config-dir}` with `{config_dir}`.

| Source Path | Destination Path |
| --- | --- |
| `framework/commands/ask.md` | `{config_dir}/commands/{project}/ask.md` |
| `framework/commands/capture.md` | `{config_dir}/commands/{project}/capture.md` |
| `framework/commands/clarify.md` | `{config_dir}/commands/{project}/clarify.md` |
| `framework/commands/elaborate.md` | `{config_dir}/commands/{project}/elaborate.md` |
| `framework/commands/groom.md` | `{config_dir}/commands/{project}/groom.md` |
| `framework/commands/help.md` | `{config_dir}/commands/{project}/help.md` |
| `framework/commands/implement.md` | `{config_dir}/commands/{project}/implement.md` |
| `framework/commands/log.md` | `{config_dir}/commands/{project}/log.md` |
| `framework/commands/plan.md` | `{config_dir}/commands/{project}/plan.md` |
| `framework/commands/spawn.md` | `{config_dir}/commands/{project}/spawn.md` |
| `framework/commands/specify.md` | `{config_dir}/commands/{project}/specify.md` |
| `framework/commands/status.md` | `{config_dir}/commands/{project}/status.md` |
| `framework/commands/target.md` | `{config_dir}/commands/{project}/target.md` |
| `framework/commands/validate.md` | `{config_dir}/commands/{project}/validate.md` |
| `framework/bootstrap/configure/{key}.md` | `{config_dir}/commands/{project}/configure.md` |

The configure row uses the agent-specific source `framework/bootstrap/configure/{key}.md` and writes it as the canonical `configure.md` in the project's command directory.

### Slash command cleanup

After processing the slash command manifest above, list all `.md` files in `{config_dir}/commands/{project}/`. For each file that is **not** in the slash command manifest above, **not** the `initialize.md` file, and **not** listed in `.governance.toml` `pinned.files`:

- Delete the file.
- Report it as "removed" in the post-scaffolding summary.

Files listed in `pinned.files` are never deleted — report them as "pinned (kept)" instead.

### Workflow recommendation (strategy: create per accepted workflow)

After the slash command cleanup, offer any newly registered workflows that match the project's tech stack and have not yet been scaffolded for this agent.

1. **Read the synced registry** at `workflows/registry.json` (the project-local copy written by the manifest above). If the file is missing or not valid JSON, warn `Workflow registry not found or invalid, skipping workflow recommendations` and skip the rest of this section. Validate each entry against the schema in `specs/005-workflows/data-model.md`; drop invalid entries with a per-entry warning.

2. **Read the project's tech stack** from `AGENTS.md`. Locate the **Tech Stack** table and parse each row's `Layer` column to recover the canonical key:

   - `Language` → `backend_language` for backend-only projects, `frontend_language` for frontend-only projects (use the project context from the rest of AGENTS.md to disambiguate; if unclear, treat the row as both)
   - `Backend language` → `backend_language`
   - `Frontend language` → `frontend_language`
   - `Backend framework` → `backend_framework`
   - `Frontend framework` → `frontend_framework`
   - `Database` → `database`
   - `Messaging` → `messaging`
   - `Backend test runner` → `backend_test_runner`
   - `Frontend test runner` → `frontend_test_runner`
   - `CSS/UI` → `css_ui`

   If `AGENTS.md` is missing, has no Tech Stack table, or the table is empty (still the comment placeholder), skip the rest of this section silently — there is nothing to match against.

3. **Match registry entries** against the project's tech stack. For each entry, look up the project's value for `entry.trigger.field` and compare case-insensitively against `entry.trigger.value`. Collect every matching entry.

4. **Filter out already-scaffolded workflows.** For each match, check whether `{config_dir}/commands/{project}/workflows/{entry.template}` already exists. If it does, the workflow was previously scaffolded (for this agent) — drop it from the candidate list. Already-scaffolded workflow files are never overwritten, regardless of content changes upstream.

5. **Silent skip when there is nothing new to offer.** If no candidates remain, do not prompt the user and proceed to **Session state**.

6. **Group remaining candidates by category** in the order: `Linting`, `Formatting`, `Testing`, `Migrations`, `Code Review`, `Deployment`. Within each category, list each match's `name` and `description`.

7. **Present per-category accept/skip prompts** via `AskUserQuestion`: "Scaffold these {category} workflows for {agent name}?" with the matched entries listed. Options: `Yes, scaffold all in this category`, `No, skip this category`. The user must explicitly accept — no workflows are scaffolded without consent.

8. **Fetch and write accepted workflows.** For each accepted entry:

   - Fetch `framework/workflows/{entry.template}` from the governance repo using the same URL pattern as the rest of govern's fetches. (Note: the workflows directory is flat — no inner `templates/` subdirectory.)
   - If the fetch fails or the file is missing, warn `Workflow file {entry.template} not found, skipping` and continue with the next accepted entry. Do not abort the surrounding scaffolding.
   - Replace every `{project}` with the user-provided project name and every `{cli-config-dir}` with the agent's `config_dir`.
   - Write the substituted content to `{config_dir}/commands/{project}/workflows/{entry.template}` (creating the `workflows/` directory if needed). Report the file as "scaffolded" in the post-scaffolding summary.

9. **Discovery note for Auggie.** Auggie's official docs document subdirectory namespacing for one level (`.augment/commands/foo/bar.md` → `/foo:bar`). Multi-level paths like `.augment/commands/{project}/workflows/lint.md` should resolve to `/{project}:workflows:lint` by the same colon-namespace convention, but a user adopting Auggie may want to confirm autocomplete the first time. Claude Code's two-level path is documented and works as expected.

10. **Adopter migration note.** Adopters who already ran `/{project}:govern` before this rename will have a `skills/` directory in their project. After re-running govern, they should manually delete the legacy `skills/` directory — workflow files have been re-created under `workflows/`, and the old directory is no longer referenced.

### Session state (strategy: create)

Create `{config_dir}/{project}-session.json` with empty content `{}` only if it does not already exist.

### Govern self-installation (strategy: update)

Fetch `framework/bootstrap/govern.md` and write it to `{config_dir}/commands/govern.md`. This is the same unified file the user is currently running, copied into every selected agent's command directory so the command is invokable from that agent on subsequent runs.

In this file (and only this file), keep `{project}` and `{cli-config-dir}` as literal placeholders — do **not** substitute. Govern itself reads `$ARGUMENTS` for the project name on each run.

After writing, run the **Post-Write Integrity Check** below.

## Placeholder Substitution

In every copied file (except `{config_dir}/commands/govern.md` for each selected agent — those keep `{project}` and `{cli-config-dir}` as literal placeholders), replace:

- `{project}` with the user-provided project name (used in commands, README)
- `{project-name}` with the user-provided project name (used in AGENTS.md template)
- `{One-line project description.}` with the user-provided description
- `{cli-config-dir}` with the agent's `config_dir`

## Post-Write Integrity Check

After writing `{config_dir}/commands/govern.md` for each selected agent, verify the file starts with `# Govern`. If it does not, the write was corrupted — report the error and re-read the file from the extracted archive. Apply the check independently per agent.

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
- Run `/{project}:configure` — that happens after adoption, interactively
- Delete an agent's adopted tree — manual cleanup

## Edge Cases

- **Unknown agent key in `--agents=`** — stop before scaffolding; report the unknown key with the list of valid keys.
- **All supported agents already adopted with `--add-agent`** — show the prompt with all agents pre-selected; if the user confirms with no additions, treat it as a routine update and continue silently.
- **`settings.local.json` already has entries beyond the bootstrap** — only add the curl/ls bootstrap entries if missing. Do not overwrite, deduplicate, or reorder entries added by `/{project}:configure` or by the user.
- **`govern.md` content already matches the version on disk** — when the manifest's `update` strategy compares fetched content to the installed file, identical content reports as "unchanged" and avoids a redundant write. Same rule applies to per-project `configure.md` and other update-strategy files.
- **Pinned `govern.md` in `.governance.toml`** — pinned files are skipped, including `govern.md` itself. A pinned `govern.md` will not pick up upstream changes until the pin is removed.
- **Archive fetch or extract fails** — clean abort with the error message defined in **File Fetching → Archive fetch and extract**. No partial scaffolding: a missing archive means every manifest entry would be missing, and the user re-runs after the transient failure clears.
- **A required source file is absent from the extracted archive** — warn `Source not found in archive: {source-path}; skipping.` and continue with the remaining manifest entries. Preserves the per-entry "do not abort on a single fetch error" guarantee at the entry level even though the archive itself is fetched once.
- **First-run prompt with no detected dirs and only one supported agent** — the prompt still appears (the agent must be explicitly chosen), but the single agent is pre-selected. Confirming is one keystroke.
- **Running `govern.md` cannot infer its own install path** — fall back to no pre-selection in the first-run prompt. The user picks explicitly.

## Post-Scaffolding Output

After scaffolding, display:

- Summary of files created, updated, unchanged, skipped, pinned, merged, and removed — grouped by agent for per-agent files, with shared files in their own group
- For each scaffolded agent, the agent's `rules_file_note` from the registry
- Any fetch failures encountered
- Self-update notice (if applicable — see below)
- Security audit summary (if applicable — see below)
- Next steps (varies by mode):

### Self-update notice

If any selected agent's `{config_dir}/commands/govern.md` was reported as "updated" (i.e., the fetched version differs from the previously installed version), append this notice after the file summary and before next steps:

> **The govern command itself was updated.** Start a new session and re-run `/govern` to apply the latest changes.

This notice is not shown on first run (the file is new, not updated) or when the govern command was unchanged across all agents.

### Security audit summary

If the **Security Audit (brownfield)** section ran and appended one or more new findings to `specs/inbox.md`, append this single line to the file summary:

> {N} security audit items added to `specs/inbox.md`. Run `/{project}:groom` to triage.

Where `{N}` is the count of newly appended findings (after deduplication). Omit this line when:

- The audit did not run (trigger conditions did not fire — greenfield run, or routine re-run with rule files already present), OR
- The audit ran but every finding was already in the inbox (`N == 0`), OR
- The audit ran but produced no findings (no rule's Verification trigger fired against any existing artifact).

This summary complements `/{project}:groom`, which is the user's path to working through the inbox at their own pace.

### First run (no existing `specs/` directory)

---

**Governance adopted successfully.**

Adopted agents: {comma-separated `name` of selected agents}.

Next steps:

1. Run `/{project}:configure` in each adopted agent to apply the full permission set.
2. Fill in `AGENTS.md` — tech stack, project structure, code style, testing conventions, gotchas.
3. Fill in `specs/system.md` — architecture, request lifecycle, shared infrastructure.
4. Use `/{project}:log` to record any known issues or bugs into `specs/inbox.md`.
5. Run `/{project}:groom` to walk the inbox and route each item to its proper spec or scenario.
6. Create your first feature spec: `/{project}:specify {feature description}`.

To adopt an additional agent later, re-run `/govern --add-agent`.

Tip: `npx quartz specs/` renders your specs as a navigable graph view in the browser. Other PKM tools (Obsidian, Logseq, MkDocs) work unchanged — pick whichever fits your workflow, or none.

---

### Update mode (existing `specs/` directory detected)

---

**Governance updated successfully.**

Updated agents: {comma-separated `name` of selected agents}.

Review changes to updated files and commit when ready. To adopt an additional agent, re-run `/govern --add-agent`.

Tip: `npx quartz specs/` renders your specs as a navigable graph view in the browser. Other PKM tools (Obsidian, Logseq, MkDocs) work unchanged.

---

## Idempotency

This command is safe to run again. Files with `update` strategy are always overwritten with the latest governance version — unless pinned in `.governance.toml`, in which case they are skipped. Files with `create` strategy skip existing files. The `.gitignore` merge checks for the `# Governance` marker before appending. `skip` strategy files are never overwritten.

Re-runs are additive across agents — adopting a new agent leaves existing agents' files untouched.

## Directory Creation

Create intermediate directories as needed (e.g., `specs/`, `specs/templates/`, `{config_dir}/commands/{project}/`).
