---
description: Adopt or update govern in an existing project.
argument-hint: "[project] [--agents=key1,key2,...] [--add-agent]"
parity:
  strict-files:
    - "{cli-config-dir}/commands/govern.md"
    - "{cli-config-dir}/commands/{project}/specify.md"
    - "{cli-config-dir}/commands/{project}/clarify.md"
    - "AGENTS.md"
  semantic-fields:
    - completion-message
---

# govern

Bootstrap `govern` in an existing project. This command fetches templates from the `govern` repo, scaffolds `govern` files for one or more AI coding CLIs, resolves placeholders, and displays next steps.

The same `govern.md` supports every agent the framework knows about. The set of supported agents lives in the **Agent Registry** below; per-agent values are looked up by registry key during scaffolding.

## Instructions

> **For agent runtimes**: backticked primitive names in this section (`fetch-archive`, `extract-archive`, `apply-manifest`, `merge-managed-block`, `enforce-manifest`) map to MCP tools the optional [gvrn runtime](https://crates.io/crates/gvrn) exposes under bare `<primitive>` names (e.g., `fetch-archive`). Hosts wrap them with a server-name prefix taken from `.mcp.json` (Claude: `mcp__gvrn__fetch-archive`; Auggie: `mcp:gvrn:fetch-archive`). When the server is registered for your session, **call the corresponding tool** for each step listed below — that is the deterministic path. When it is not registered, walk the markdown-only reference below (`tar -xzf`, `curl`, etc.) to produce the same result. The two paths share a contract; neither one wraps the other.

**Procedural fidelity.** Execute the steps below as written. The only confirmation prompts to issue are those the procedure specifies: project inputs (§Inputs), agent-selection prompts on `--add-agent` / first-run (§Agent Selection), the registry-driven migration prompts (§Pre-run Migrations — outer "apply N pending migrations" prompt plus any per-entry inner prompts the procedure files specify), and per-category workflow prompts (§Workflow recommendation, step 8). Do not stop to warn about uncommitted edits to update-strategy files, custom slash commands that **Slash command cleanup** is about to remove, or "data loss" from the stale → write-and-abort path. The procedure already encodes safety: `.govern.toml` `[pinned] files` is the opt-out, the stale path writes upstream and aborts cleanly (recoverable from git), and slash-command cleanup is unconditional for unpinned files. Extra prompts duplicate information the procedure already gives the user and stall routine runs.

1. The walker context carries the inputs the host has already gathered and validated: project (the destination project name), description (one-line project description), languages (comma-separated), agents (registry keys), framework-version (release tag), archive-url and sha256-url (computed from framework-version), staging-dir, substitutions-map, manifest-entries (the per-strategy list described in **Shared Files** and **Per-Agent Scaffolding**), pinned-list (from `.govern.toml`'s `[pinned] files` block), gitignore-block (the `.claude/`, `specs/.cache/`, etc. lines), host-block (the cli-config-dir and project values the runtime reads at `gvrn exec` time to resolve `{cli-config-dir}/commands/{project}/<name>.md`), enforce-directories (the slash-command directories whose top-level `*.md` files are pruned to the manifest), and the per-agent govern-install entry with `keep-literals: ["project", "cli-config-dir"]`. The host runs the markdown-only reference below to collect inputs, derive registry values, validate `.govern.toml`, and seed context; the runtime walks the procedure that follows.

2. Invoke `fetch-archive` (MCP: `fetch-archive`) to download the framework tarball. The primitive verifies the sha256 against a sidecar URL when one is supplied; without a sidecar (the live-on-main case, since GitHub's auto-generated source tarballs ship without sidecars) it returns the computed digest and `verified: false`, leaving any out-of-band verification to the host. A sidecar mismatch halts the procedure with an `error` envelope so no partial state lands in the destination tree.

3. Invoke `extract-archive` (MCP: `extract-archive`) to expand the verified tarball into the staging directory. Path-traversal protection is applied per entry; symlinks are skipped. Otherwise, follow the markdown-only path's `tar -xzf` workflow.

4. Invoke `apply-manifest` (MCP: `apply-manifest`) with the host-built manifest entries and the pinned list. The primitive walks each entry, applies the per-entry strategy (update for framework-owned files, create for adopter-seedable files, skip-if-conflict for adopter-owned templates — the three strategy values defined in **Shared Files** below), short-circuits on the pinned list, returns aggregate counts the host surfaces in the completion message. This single call replaces the per-file update / create / skip loops the markdown-only reference describes below.

5. Invoke `merge-managed-block` (MCP: `merge-managed-block`) against `.gitignore` with `marker-style: "line-prefix"` and `marker: "govern"` to install or update the framework-managed block (the `.claude/`, `specs/.cache/`, etc. lines). First-run creates the file; subsequent runs update only the region between the `# govern` preamble line and the next blank line, preserving the rest of the file byte-for-byte. Replaces the inline `grep` check the markdown-only reference describes for the `.gitignore` merge step.

6. Invoke `merge-managed-block` (MCP: `merge-managed-block`) against `.govern.toml` with `marker-style: "line-prefix"`, `marker: "govern (host)"`, and a block carrying the resolved cli-config-dir and project values (the host-block from step 1). First-run creates the file with just the managed block; subsequent runs update only the region between the `# govern (host)` preamble line and the next blank line, preserving every other `.govern.toml` section (`[pinned]`, `[workflows]`, `[migrations]`, `[review]`) byte-for-byte. The runtime reads these values at `gvrn exec` time to resolve `{cli-config-dir}/commands/{project}/<name>.md`; without the block the runtime falls back to `.claude` / repo directory basename — fine for the framework's own repo, broken for any adopter whose layout doesn't match the defaults. See §Project Configuration for the `[host]` schema.

7. Invoke `enforce-manifest` (MCP: `enforce-manifest`) once per directory in the host's enforce-directories list (typically the per-agent slash-command directory). The primitive removes files matching the glob-include arg (default `*.md`) whose relative path is neither in the expected list nor pinned. One call replaces the slash-command manifest enforcement loop the markdown-only reference describes. Adopter cleanup of historical conventions (legacy `skills/` directory, post-005 workflow filename rename, and the rest) is owned by the **Pre-run Migrations** section above and the `framework/migrations.toml` registry it drives.

8. Invoke `apply-manifest` (MCP: `apply-manifest`) a second time with a single entry for the per-agent `govern` self-install (the `{cli-config-dir}/commands/govern.md` path) and an **empty substitutions map** (`{}`). `govern.md`'s body contains prose references to every placeholder name the bulk step substitutes — `{project}`, `{cli-config-dir}`, `{project-name}`, `{One-line project description.}` — describing what those placeholders mean in *other* files. None of them are values to substitute in `govern.md` itself, so the self-install call passes no substitutions rather than relying on `keep-literals` to mask individual keys from the full map. The split from step 4 isolates the no-substitute concern from the bulk substitute step.

9. Render the completion message (host responsibility): list the agents configured, the next pipeline command (`/{project}:specify`), the optional runtime install pointer (see the README's Runtime section), and any per-agent post-install reminders from the registry rows above.

## Agent Registry

The registry lists every supported agent. Per-agent paths and behaviors are derived from these rows — the rest of this file references registry values, not agent names.

| `key` | `name` | `config_dir` | `layout` | `settings_template` | `rules_file_note` |
| --- | --- | --- | --- | --- | --- |
| `claude` | Claude Code | `.claude` | `claude-style` | `{ "permissions": { "allow": ["Bash(curl *)", "Bash(ls *)", "Bash(tar *)", "Bash(mktemp *)", "Bash(git status *)", "Bash(git config *)", "Bash(chmod *)", "Bash(awk *)", "Read(/private/var/folders/**/T/govern-*/**)", "Read(//private/var/folders/**/T/govern-*/**)", "Read(/var/folders/**/T/govern-*/**)", "Read(//var/folders/**/T/govern-*/**)", "Read(/tmp/govern-*/**)", "Read(//tmp/govern-*/**)"], "deny": [] } }` | Claude Code reads `CLAUDE.md` natively. |
| `auggie` | Auggie | `.augment` | `claude-style` | `{ "toolPermissions": [ { "toolName": "launch-process", "shellInputRegex": "^curl ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^ls ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^tar ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^mktemp ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^git status ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^git config ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^chmod ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^awk ", "permission": { "type": "allow" } } ] }` | Auggie reads `CLAUDE.md` natively — no second rules file is needed. |
| `antigravity` | Antigravity | `.agents` | `antigravity` | `{ "permissions": { "allow": [ "command(curl)", "command(ls)", "command(tar)", "command(mktemp)", "command(git status)", "command(git config)", "command(chmod)", "command(awk)" ], "deny": [], "ask": [] } }` | Antigravity reads `AGENTS.md` natively — no second rules file is needed. |

### Derived values

For each agent, these paths and behaviors are computed by convention from its row — they are **not** stored in the table. Values that are the same for every agent are layout-independent; the rest are selected by the row's `layout` field.

**Layout-independent (every agent):**

| Derived value | Formula |
| --- | --- |
| Configure source path | `framework/bootstrap/configure/{key}.md` |

**Layout-derived (selected by `layout`):**

| Derived value | `claude-style` | `antigravity` |
| --- | --- | --- |
| Command/skill path | `{config_dir}/commands/{project}/<name>.md` | `{config_dir}/skills/{project}-<name>/SKILL.md` |
| Invocation | `/{project}:<name>` | `/{project}-<name>` |
| `govern` install path | `{config_dir}/commands/govern.md` | `{config_dir}/skills/govern/SKILL.md` |
| MCP-wiring file | `.mcp.json` | `{config_dir}/mcp_config.json` |
| Settings file | `{config_dir}/settings.local.json` | `{config_dir}/settings.json` |
| Permission shape | `permissions.allow/deny` (Claude) / `toolPermissions[]` (Auggie) | `permissions.allow/deny/ask` (action grammar) |
| Native rule-loading dir | — (rules read from shared `specs/rules/`) | `{config_dir}/rules/<name>.md` |
| Native rules file | `CLAUDE.md` | `AGENTS.md` |
| Slash-command cleanup glob | `*.md` in the commands dir | `{project}-*/` skill dirs in `skills/` |

The session state file is `.govern.session.toml` at the repo root for every adopter — not a derived per-agent value. It's gitignored and host-agnostic.

### Adding a new agent

A `claude-style` agent (markdown commands under `{config_dir}/commands/{project}/`, reads `CLAUDE.md`) is still a one-row append plus two satellite files:

1. Append a row with the six fields (`layout: claude-style`).
2. Add `framework/bootstrap/configure/{key}.md` with the agent's full permission set in its native settings format.
3. Add a curl snippet for the new agent to the README's adoption section.

An agent on a **different layout** (a new value in the `layout` column) additionally needs its branch added to §Derived values and the layout-keyed steps in §Per-Agent Scaffolding, §Permission Setup, and the MCP-registration step — the work this `antigravity` layout introduced.

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

1. Read the agent's settings file — `{config_dir}/settings.local.json` for `claude-style`, or `{config_dir}/settings.json` for `antigravity` (create it if missing, with the agent's `settings_template` from the registry).
2. Merge the agent's `settings_template` entries into the existing file additively: add any entries that are missing, do not deduplicate or reorder anything else, and do not overwrite entries the user or `/{project}:configure` previously added. For `claude-style` the entries live under `permissions.allow`/`permissions.deny` (Claude) or `toolPermissions` (Auggie); for `antigravity` they live under `permissions.allow`/`permissions.deny`/`permissions.ask`.
3. Write the file if anything was added.

This prevents repeated permission prompts during the fetch and scaffolding phases. The full permission set is applied later by `/{project}:configure` (which writes the same per-layout settings file).

The gvrn runtime itself is wired separately and is **not** scaffolded by `/govern`: `claude-style` registers the server via `.mcp.json`, `antigravity` via `{config_dir}/mcp_config.json` (`{ "mcpServers": { "gvrn": { "command": "gvrn", "args": ["mcp"] } } }`, additive). Both are an optional install documented in the README's Runtime section.

## govern.md Self-Update Check

Before any other fetching, scaffolding, or migration, verify the running session's `govern.md` instructions are current. The check is its own phase — ahead of pre-run migrations and the full archive fetch — so a stale-detected abort does not leave any other write on disk and does not pay the cost of fetching the multi-hundred-KB archive on a run that is going to abort anyway.

### Small fetch

Create a fresh temp directory used by both this check and the later archive fetch:

```text
mktemp -d -t govern-XXXXXX
```

On macOS/Linux this lands under `$TMPDIR` or `/tmp`. Never reuse a directory from a prior run — a fresh fetch is the only way `/govern` picks up upstream changes.

Issue exactly one `curl` against `raw.githubusercontent.com` for the upstream bootstrap file:

```text
curl -fsSL https://raw.githubusercontent.com/stonean/govern/main/framework/bootstrap/govern.md \
  -o {tempdir}/govern.md.upstream
```

If the fetch fails — non-zero `curl` exit, network error, or a 404 — abort the run with this error and do not continue:

> Failed to fetch the govern.md self-update check ({reason}). Re-run after checking network connectivity, or report this if it persists.

### Per-agent comparison

For each selected agent, compare the upstream `{tempdir}/govern.md.upstream` against the agent's installed `govern` file and assign one status. For `claude-style` the installed file is `{config_dir}/commands/govern.md` and the comparison is a byte-compare. For `antigravity` the installed file is `{config_dir}/skills/govern/SKILL.md`, which wraps the upstream body in `name: govern` frontmatter — strip its leading frontmatter block (the first `---`-delimited region) and compare the remaining body against `{tempdir}/govern.md.upstream`:

- **`no installed copy`** — the installed file does not exist (first run for this agent). Continue.
- **`current`** — the two files are byte-identical, **or** the installed file is byte-identical to upstream and listed in `.govern.toml` `pinned.files` (the pin had nothing to suppress this run). Continue.
- **`stale`** — the two files differ and the installed file is **not** pinned. The running session is using older instructions than what is current upstream.
- **`pinned-divergent`** — the two files differ and the installed file **is** listed in `.govern.toml` `pinned.files`. The pin intentionally suppresses the update; continue, and emit a single advisory line in the post-scaffolding output.

The check is scoped to **selected agents only** — agents whose `config_dir` exists in the project but are not in this run's selection are not diffed. An unselected stale agent will trip the check on its very next `/govern` run targeting it.

### Stale → write and abort

If any selected agent is recorded as `stale`:

1. For **each stale agent**, write the freshly fetched upstream to the agent's installed `govern` file (overwrite) so the next session loads the up-to-date instructions. For `claude-style`, copy `{tempdir}/govern.md.upstream` verbatim to `{config_dir}/commands/govern.md`. For `antigravity`, write `{config_dir}/skills/govern/SKILL.md` as the transformed skill — `name: govern` frontmatter followed by the upstream body — **not** the raw `govern.md` (a raw copy is not a loadable skill). In both cases do not substitute placeholders in the body — `{project}` and `{cli-config-dir}` stay literal, per the `govern` self-install rule.
2. Run the **Post-Write Integrity Check** (see below) on each freshly written file.
3. Do not write `govern.md` for non-stale agents — their installed copies already match upstream.
4. Do not write `govern.md` for `pinned-divergent` agents — the pin opts them out of automatic updates.
5. Abort the run before any further work. Print:

> **The govern command itself has updated.** Your installed copy was behind upstream and the running session is using the older instructions. The freshly fetched copy has been written to disk for stale agents.
>
> Stale agents updated: {comma-separated names}.
>
> Start a new session and re-run `/govern` to pick up the latest version.

Everything past this point — **Pre-run Migrations**, **Project Configuration**, the **Archive fetch and extract**, **Frontmatter Migration**, **Shared Files**, **Per-Agent Scaffolding**, **Security Audit**, and **Post-Scaffolding Output** — is skipped. The only writes this run performed are the additive **Permission Setup** entries and the per-stale-agent `govern.md` overwrite.

The next `/govern` run in a new session loads the fresh `govern.md`, the self-update check sees `current` (or `no installed copy`) for every agent, and the run proceeds normally without abort.

### Pinned-divergent → continue with advisory

If a selected agent is recorded as `pinned-divergent`, the run continues normally. After scaffolding, the **Post-Scaffolding Output** includes one advisory line per divergent agent (see **Post-Scaffolding Output → Pinned govern.md advisory**). The advisory is silent on runs where every pinned agent is `current` (the pinned version happens to match upstream this run).

Pinning is an opt-out from automatic updates, not an opt-out from knowing the pin is currently active. When the pinned version actually drifts from upstream, the user usually wants to either review the upstream changes and unpin, or consciously confirm they are staying on the old version. Adopters who are deliberately and indefinitely on an old version see no recurring nag because the advisory only fires when divergence is real.

### Current / no installed copy → continue

When all selected agents are `current` or `no installed copy`, the run proceeds. The temp directory created here is reused by the **Archive fetch and extract** step below — no second `mktemp`, no leaked extra temp directory.

## Pre-run Migrations

Adopter-side cleanup for conventions that have been removed or renamed since the adopter's last `/govern` run. Driven by a machine-readable registry at `framework/migrations.toml` (one `[[migrations]]` entry per active removal); per-entry procedure bodies live at `framework/migrations/{id}.md`. Spec [027 — Bootstrap Migration Registry](../../specs/027-bootstrap-migration-registry/spec.md) defines the contract.

### Procedure

1. Read `framework/migrations.toml` from the fetched archive. If the file is missing or malformed (TOML parse error), abort with `Failed to read framework/migrations.toml; cannot run pre-run migrations.` and do not continue.
2. Read `.govern.toml`'s `[migrations].last_applied` (treat an absent `[migrations]` section as null).
3. Filter the registry to entries where both:

   - `introduced_in` is greater than `last_applied`'s `introduced_in` (SemVer comparison, lex tie-break on `id`); when `last_applied` is null, every entry qualifies.
   - Either `sunset_after` is absent, or the current gvrn release version is less than `sunset_after` (SemVer comparison).

4. If the filtered list is empty, emit nothing and proceed to the next bootstrap section.
5. Otherwise, prompt once with text of the form:

   ```text
   N framework migrations are pending since your last /govern run:
     - {id} (introduced {introduced_in})
     ...
   Apply now? (Y/n)
   ```

6. On decline, emit `warning: N migrations skipped; pipeline commands may fail on legacy artifacts until applied. Re-run /govern to apply.` and proceed without filesystem changes.
7. On confirm, for each filtered entry in order:

   1. Read `framework/migrations/{id}.md` from the fetched archive.
   2. Execute its `## Procedure` steps. The procedure file owns idempotency (step 1 of every procedure exits silently when the target artifact is absent), per-file user prompts (when applicable), and the post-scaffolding summary line.
   3. After the procedure completes successfully, update `.govern.toml`'s `[migrations].last_applied = "{id}"` atomically (tempfile + rename, matching the rest of `.govern.toml` write semantics). The update happens **per entry**, not at end of batch — an aborted batch resumes from the next-pending entry on the following `/govern` run.
   4. If a procedure aborts (rare — only via explicit user "stop everything" path inside the procedure file), halt the loop. The retained `last_applied` value points at the last-completed entry; the next run resumes.

### Stale-reference behavior

If `.govern.toml`'s `[migrations].last_applied` references an `id` that no longer exists in the active registry (the entry was sunsetted since the adopter's last run), treat the field as "before the oldest active entry" and run every active entry. Emit one warning: `last_applied was "{retired_id}" which has been retired; see CHANGELOG.md for its recipe.` Adopters far enough behind to hit a sunsetted entry apply it manually from `CHANGELOG.md`.

### Duplicate-id and reference-integrity guard

If `framework/migrations.toml` contains two entries with the same `id`, or if any entry's `procedure_file` references a path that doesn't exist in the fetched archive, abort the loop before applying anything with a clear error. `/audit`'s Family 10 (`scripts/audit/migration-coverage.sh`) catches these at maintainer time; this guard is the runtime safety net.

## Project Configuration

`.govern.toml` is the project's configuration and persisted-decisions store. If the file exists, read it before processing the file manifest. The file is optional — if it does not exist, use default behavior for every key. If the file exists but is malformed (TOML parse error), abort the run with a clear error rather than silently proceeding.

The file is a flat collection of top-level sections. There is no umbrella namespace; each section is keyed to the thing it governs. The sections that may appear in `.govern.toml`:

```toml
# govern (host)
[host]
cli-config-dir = ".claude"
project = "gov"

[pinned]
# Files listed here use 'skip' instead of 'update'.
# Use destination paths (after placeholder resolution).
files = [
  ".claude/commands/myapp/implement.md",
  "constitution.md",
]

[migrations]
# Slug of the newest pre-run migration applied. Bootstrap runs only entries
# newer than this (see §Pre-run Migrations). Absent section means "no
# migrations applied" — bootstrap runs every active entry. Maintained by
# /govern; do not edit by hand.
last_applied = "rule-files-relocate"

[workflows]
# Workflow categories the user has chosen to permanently decline at the
# per-category recommendation prompt. Match is case-insensitive against the
# registry-derived category list (Linting, Formatting, Testing, Migrations,
# Code Review, Deployment). Created lazily by /govern when the user picks
# "Skip and don't ask again" at the prompt.
declined_categories = ["Linting", "Formatting"]

# Consumed by /gov:review (not /govern itself). Excludes rule files from
# /gov:review's selection regardless of stack detection. The `reason` field
# is mandatory (trimmed length ≥ 16 Unicode codepoints) and is the audit
# trail for the override. Listed here for schema reference; uncomment and
# edit to use.
#
# [[review.disabled-rule-files]]
# file = "accessibility-frontend.md"
# reason = "Internal admin UI — WCAG AA enforcement deferred to Q3"
#
# [[review.disabled-rule-files]]
# file = "api-backend.md"
# reason = "Pre-OpenAPI; revisit after schema lands (PROJ-1234)"
```

`host.cli-config-dir` and `host.project` — the host's per-user config-dir name and the project's slash-command namespace, written by `/govern` into a managed block (`# govern (host)` line-prefix marker) on every run. The runtime reads these at `gvrn exec` time to resolve `{cli-config-dir}/commands/{project}/<name>.md`; both keys fall back to `.claude` / the repo directory basename when the block is missing. Adopters whose layout matches the defaults (this repo, anyone on Claude Code with the conventional `.claude/commands/<project>/`) never observe the difference; Auggie adopters and anyone with a non-standard layout do. The block is idempotent — re-runs update the values rather than appending duplicates.

`pinned.files` — any file listed that would normally use `update` strategy is treated as `skip` instead. Report pinned files in the post-scaffolding summary.

`migrations.last_applied` — slug of the newest pre-run migration applied to this project, written by `/govern` after each successful migration in §Pre-run Migrations. Absent section means "no migrations applied"; bootstrap runs every active entry on the next run. Adopters should not edit this field by hand — the registry in `framework/migrations.toml` and the per-entry procedure files in `framework/migrations/{id}.md` are the authoritative sources.

`workflows.declined_categories` — categories listed here suppress the per-category workflow recommendation prompt entirely (see the **Workflow recommendation** flow below). Entries that don't match any canonical category name are reported once each in the post-scaffolding summary as `unrecognized workflow decline: "{value}" (in .govern.toml)` but do not abort the run.

`review.disabled-rule-files` — array-of-tables consumed by `/gov:review` at rule-file selection time (see [`framework/commands/review.md`](../commands/review.md) §Inputs and §Behavior step 5). `/govern` does not read this key; it is documented here so adopters see the full `.govern.toml` schema in one place.

The full schema (allowed values, case-insensitive matching, empty-section behavior, future-section guidance) is declared in [`specs/019-config-decisions/data-model.md`](../../specs/019-config-decisions/data-model.md).

## File Fetching

Files from the `govern` repo are sourced from a single archive download, extracted into the temp directory established by **govern.md Self-Update Check**, and resolved as local paths for the rest of the run. Per-language `.gitignore` patterns from `github.com/github/gitignore` are **not** part of this archive — they remain separate `curl` calls (see the **.gitignore** subsection of **Shared Files** below).

This section runs only after the **govern.md Self-Update Check** passes (no stale agents). On a stale-abort, the archive is never fetched.

### Archive fetch and extract

Issue exactly one `curl` against GitHub's repo-archive endpoint, downloading into the temp directory established by the self-update check:

```text
curl -fsSL https://github.com/stonean/govern/archive/refs/heads/main.tar.gz \
  -o {tempdir}/main.tar.gz
```

`curl -fsSL` follows the 302 redirect to `codeload.github.com`. The archive's top-level directory is `govern-main/`; the framework files live at `govern-main/framework/...` after extraction.

After fetching:

1. Extract the archive into the existing temp directory: `tar -xzf {tempdir}/main.tar.gz -C {tempdir}`.
2. Compute the framework root: `{tempdir}/govern-main/`. Treat this as the local mirror of the `govern` repo for the rest of the run.

If the fetch or extraction fails — non-zero exit from `curl` or `tar`, or a missing `govern-main/` directory after extract — abort the run with this error and do not continue scaffolding:

> Failed to fetch or extract the `govern` archive ({reason}). Re-run after checking network connectivity, or report this if it persists.

A missing archive means **every** manifest entry would be missing, so partial scaffolding is impossible — the abort is the correct behavior. The self-update check has already completed by this point, so a stale `govern.md` would have already been written and the run would have aborted earlier.

### Per-file resolution

For each manifest entry below (in **Shared Files**, **Per-Agent Scaffolding**, and the workflow-recommendation flow):

1. Compute the local source path: `{tempdir}/govern-main/{source-path}`.
2. If the local source path does not exist — the file was renamed, removed upstream, or the manifest is out of sync — warn `Source not found in archive: {source-path}; skipping.` and continue with the remaining entries. This preserves the "do not abort on a single fetch error" guarantee at the per-entry level, even though the archive itself is fetched once.
3. Apply the entry's strategy (`update`, `create`, `skip`, `merge`, `pinned`) using the local file as the new content. For `update` strategy, compare the local file against the existing destination file; only overwrite and report as "updated" if the content differs. If the content is identical, report as "unchanged" (or omit from the summary). Same semantics as before — no network round-trip per file.
4. Apply placeholder substitution after reading the local source, before writing to the destination. Same rules as documented in **Placeholder Substitution** below, including the `govern.md` self-install exception that keeps `{project}` and `{cli-config-dir}` literal.

### Cleanup

`/govern` does not delete the temp directory. The path is logged in the post-scaffolding summary (and, on abort, in the error message) so the user can inspect it if needed. Both macOS (`/var/folders/.../T/`) and Linux (`/tmp` on systemd-tmpfiles distros) sweep their temp directories automatically; a few hundred KB of extracted files waiting for the next sweep is acceptable in exchange for not granting an `rm -rf` permission to the bootstrap.

The leftover directory is for inspection only — the next `/govern` run creates its own fresh temp directory via `mktemp` and never reuses a prior extract.

## Frontmatter Migration

If `specs/` does not exist (first run), skip this section — there is nothing to migrate.

Bring existing spec and scenario files into the YAML frontmatter format declared in `framework/constitution.md` §text-first-artifacts. Migration is idempotent: re-running on an already-migrated project produces no further metadata changes.

This section runs **after the govern.md Self-Update Check** so that a stale-govern abort cannot leave migration changes from old rules on the working tree. The new govern's migration logic — which may differ — is the only logic that ever writes migration changes.

### Precheck

Run `git status --porcelain -- specs/` (project-relative). If the output is non-empty, refuse with:

> Migration requires a clean working tree under `specs/`. Commit or stash your changes, then re-run.

Exit before any modifications. Unrelated in-flight work outside `specs/` does not block migration.

### Walk

For each file matching one of:

- `specs/**/spec.md`
- `specs/**/scenarios/*.md`

Determine whether the file needs migration:

- Read the first non-blank line of the file. If it is `---`, the file already has frontmatter — skip with reason "already frontmatter."
- Otherwise, scan the first few lines after the heading for bold-prefix metadata patterns (`**Status:**`, `**Dependencies:**`, `**spec-ref:**`). If at least one is found, the file needs migration.
- If no bold-prefix lines are present and no frontmatter exists, skip with reason "no metadata to migrate."

Skip files that appear in `.govern.toml` `pinned.files` with reason "pinned." The adopter is responsible for migrating pinned files manually.

### Convert

For each file that needs migration:

**Spec files** (`spec.md`):

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
- `skipped (pinned): {file path}` for files listed in `.govern.toml`
- `skipped (no metadata to migrate): {file path}` for files without recognizable metadata
- `skipped (malformed metadata): {file path} — {reason}` for files that could not be parsed

The user reviews the result via `git diff` and commits or aborts via `git restore`. No backup directory is created — git is the recovery mechanism.

## Shared Files

These files are scaffolded **once per `/govern` invocation**, regardless of how many agents are selected. They are unaffected by the agent registry.

### `govern`-owned shared files (strategy: update)

| Source Path | Destination Path |
| --- | --- |
| `framework/constitution.md` | `constitution.md` |
| `framework/rules/accessibility-frontend.md` | `specs/rules/accessibility-frontend.md` |
| `framework/rules/api-backend.md` | `specs/rules/api-backend.md` |
| `framework/rules/configuration-cross.md` | `specs/rules/configuration-cross.md` |
| `framework/rules/performance-frontend.md` | `specs/rules/performance-frontend.md` |
| `framework/rules/security-backend.md` | `specs/rules/security-backend.md` |
| `framework/rules/security-frontend.md` | `specs/rules/security-frontend.md` |
| `framework/bootstrap/hooks/govern-pre-commit` | `.githooks/govern-pre-commit` |
| `scripts/gen-spec-deps.sh` | `scripts/gen-spec-deps.sh` |
| `.markdownlint-cli2.jsonc` | `.markdownlint-cli2.jsonc` |
| `framework/templates/spec/spec.md` | `specs/templates/spec.md` |
| `framework/templates/spec/plan.md` | `specs/templates/plan.md` |
| `framework/templates/spec/tasks.md` | `specs/templates/tasks.md` |
| `framework/templates/spec/data-model.md` | `specs/templates/data-model.md` |
| `framework/templates/spec/research.md` | `specs/templates/research.md` |
| `framework/templates/spec/scenario.md` | `specs/templates/scenario.md` |
| `framework/workflows/registry.json` | `workflows/registry.json` |

### Project-specific shared files (strategy: create)

| Source Path | Destination Path |
| --- | --- |
| `framework/templates/project/system.md` | `specs/system.md` |
| `framework/templates/project/errors.md` | `specs/errors.md` |
| `framework/templates/project/events.md` | `specs/events.md` |
| `framework/templates/project/inbox.md` | `specs/inbox.md` |
| `framework/bootstrap/hooks/pre-commit` | `.githooks/pre-commit` |

### Shared files with conflict handling

**AGENTS.md** (strategy: skip) — if it exists, leave it alone. If not, fetch `framework/templates/project/agents.md` from the `govern` repo and copy it as `AGENTS.md`, substituting `{project-name}` with the project name and `{One-line project description.}` with the project description.

**CLAUDE.md** (strategy: skip) — if it exists, leave it alone. If not, fetch `framework/templates/project/claude-md.md` from the `govern` repo and copy it as `CLAUDE.md`. `claude-style` agents read `CLAUDE.md` natively (see each row's `rules_file_note`); the `antigravity` layout reads `AGENTS.md` natively instead. `CLAUDE.md` is still written once as a shared file (it is harmless for an Antigravity-only project), but it is not the rules file Antigravity loads.

**.gitignore** (strategy: merge) — install or update a framework-managed block delimited by a `# govern` line preamble, then dedup any adopter-area copies of canonical patterns. Mirrors the runtime `merge-managed-block` contract (line-prefix style, marker `govern`):

1. Fetch `framework/templates/project/gitignore` from the `govern` repo. This is the **canonical block** — including its blank-line-separated subsections.
2. If `.gitignore` does not exist, create it with `# govern\n{canonical-block}\n`. Skip to step 5 for language patterns.
3. If `.gitignore` exists and contains a `# govern` line preamble, replace the managed region (the `# govern` line through the rest of the block — note the canonical block itself contains blank lines between subsections, so do not stop at the first interior blank) with `# govern\n{canonical-block}\n`. If no `# govern` line is present, append `# govern\n{canonical-block}\n` after the existing content, separated by exactly one blank line.
4. **Dedup pass (canonical-block wins).** After the managed block is in place, scan the rest of the file (everything outside `# govern` through the canonical block's end) and remove any non-blank, non-comment line that string-equals a non-blank, non-comment line inside the canonical block. Adopter-area blank lines and comment lines are preserved untouched even when they happen to share text with a canonical pattern. This collapses duplicates that an adopter (or another command) pasted above or below the marker; the canonical copy inside `# govern` is the surviving one.
5. For each primary language provided by the user, fetch from `https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore` and append below a `# {Language}` comment header. If the file is being re-merged on a subsequent run and a `# {Language}` section is already present, leave it alone — language sections, once written, are adopter territory.

## Security Audit (brownfield)

Run a one-time security audit when the project newly receives a security rule file alongside existing feature specs. This is the brownfield-adoption hook described in `specs/008-security-rules/spec.md` — it routes findings through `specs/inbox.md` so the adopter can triage them via `/{project}:groom` at their own pace, rather than having every legacy spec immediately fail validate.

### Trigger

Run the audit only when **both** conditions hold after the **Shared Files** manifest pass has completed:

1. At least one of `specs/rules/security-backend.md` or `specs/rules/security-frontend.md` was **newly created** by the manifest pass (the destination file did not exist before this run). A file that was merely updated or unchanged does not trigger the audit.
2. The project contains at least one feature spec directory under `specs/` matching the `NNN-*` pattern (zero-padded, three-digit prefix followed by a hyphen and a slug).

If either condition fails, skip this section silently — no output, no finding, no inbox entry. This covers the two routine cases:

- **Greenfield adoption** — no `specs/NNN-*/` directories exist, so the audit has nothing to scan against.
- **Routine re-run** — the rule files were created on a prior run; the manifest pass reports them as "updated" or "unchanged" rather than "created".

### Loading rule files

For each rule file that passed the trigger:

1. Read the file from its destination path (`specs/rules/security-backend.md` or `specs/rules/security-frontend.md`).
2. Apply the same integrity checks `/{project}:analyze` uses for the security-rule check section: well-formed level-3 headings of the form `### {ID}`, the four required fields (Statement, Rationale, Verification, Source), an ID matching `{FE|BE}-{CATEGORY}-{NNN}`, and no duplicate IDs within the file.
3. If a file fails any integrity check, report `Security audit: {path} failed to load — {reason}; skipping audit for this file.` and continue with the other rule file (if applicable). Do not abort the surrounding `govern` run.

This mirrors validate's posture — partial or guessed-at parsing produces unreliable findings, so an unloadable file is treated as absent for audit purposes.

### Per-rule check

For each rule that loaded successfully:

1. Identify the artifacts in scope: `specs/NNN-*/spec.md`, `specs/NNN-*/plan.md`, and any `specs/NNN-*/scenarios/*.md`.
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

Findings the user has already groomed (lines that have been removed or rewritten) are not re-emitted — once the adopter has triaged a finding, `govern` does not resurrect it.

### Audit summary

Track the count of newly appended findings (post-deduplication). The total is reported by **Post-Scaffolding Output**; when the count is zero, the audit-summary line is omitted entirely.

## Per-Agent Scaffolding

For each selected agent (in registry row order), run these steps with `{config_dir}` resolved to the agent's value and `{key}` to the agent's key.

The steps below describe the **`claude-style`** layout. For an agent whose registry `layout` is **`antigravity`**, apply **### Antigravity layout** below in place of **### Slash commands** and **### Slash command cleanup**, and skip **### Workflow recommendation**. The `govern` self-install, the **govern.md Self-Update Check**, the **Post-Write Integrity Check**, and **Placeholder Substitution** each carry their own `layout: antigravity` branch in their own sections.

### Slash commands (strategy: update)

Fetch each command template and copy it into `{config_dir}/commands/{project}/`. In each copied file, replace `{project}` with the user-provided project name and `{cli-config-dir}` with `{config_dir}`.

| Source Path | Destination Path |
| --- | --- |
| `framework/commands/ask.md` | `{config_dir}/commands/{project}/ask.md` |
| `framework/commands/clarify.md` | `{config_dir}/commands/{project}/clarify.md` |
| `framework/commands/groom.md` | `{config_dir}/commands/{project}/groom.md` |
| `framework/commands/help.md` | `{config_dir}/commands/{project}/help.md` |
| `framework/commands/implement.md` | `{config_dir}/commands/{project}/implement.md` |
| `framework/commands/log.md` | `{config_dir}/commands/{project}/log.md` |
| `framework/commands/plan.md` | `{config_dir}/commands/{project}/plan.md` |
| `framework/commands/review.md` | `{config_dir}/commands/{project}/review.md` |
| `framework/commands/specify.md` | `{config_dir}/commands/{project}/specify.md` |
| `framework/commands/status.md` | `{config_dir}/commands/{project}/status.md` |
| `framework/commands/target.md` | `{config_dir}/commands/{project}/target.md` |
| `framework/commands/analyze.md` | `{config_dir}/commands/{project}/analyze.md` |
| `framework/bootstrap/configure/{key}.md` | `{config_dir}/commands/{project}/configure.md` |

The configure row uses the agent-specific source `framework/bootstrap/configure/{key}.md` and writes it as the canonical `configure.md` in the project's command directory.

### Slash command cleanup

After processing the slash command manifest above, list all `.md` files in `{config_dir}/commands/{project}/`. For each file that is **not** in the slash command manifest above and **not** listed in `.govern.toml` `pinned.files`:

- Delete the file.
- Report it as "removed" in the post-scaffolding summary.

Files listed in `pinned.files` are never deleted — report them as "pinned (kept)" instead.

### Antigravity layout (`layout: antigravity`)

When the agent's registry `layout` is `antigravity`, the two subsections above (**Slash commands**, **Slash command cleanup**) are replaced by the skill-based equivalents below. `{config_dir}` resolves to `.agents`; Antigravity discovers dir-form skills under `{config_dir}/skills/`.

**Skills (strategy: update).** For each row in the slash-command manifest above — the twelve `framework/commands/*.md` rows plus the `framework/bootstrap/configure/{key}.md` configure row — transform the source into a dir-form skill at `{config_dir}/skills/{project}-{name}/SKILL.md` (instead of copying to `{config_dir}/commands/{project}/{name}.md`):

1. Read the source markdown (frontmatter + body).
2. Write `{config_dir}/skills/{project}-{name}/SKILL.md` with frontmatter `name: {project}-{name}` and the `description:` carried from the source frontmatter, followed by the source body.
3. Substitute `{project}` and `{cli-config-dir}` in the body exactly as in the `claude-style` copy (`{cli-config-dir}` → `.agents`).

`{name}` is the command's base name (`specify`, `clarify`, …; the configure row's `{name}` is `configure`). The skills are invoked as `/{project}-{name}`.

**Rules (strategy: update).** Mirror each domain rule file the **Shared Files** manifest placed in `specs/rules/` into `{config_dir}/rules/{name}.md`, so Antigravity loads them natively. Both copies regenerate from `framework/rules/` on every `/govern` run — `specs/rules/` stays the pipeline-read location for every agent; `{config_dir}/rules/` is the Antigravity-native mirror. The `specs/rules/` write itself (in **Shared Files**) is layout-independent and unchanged.

**Skill cleanup (replaces Slash command cleanup).** List the skill directories under `{config_dir}/skills/` whose name matches `{project}-*`. Delete any `{config_dir}/skills/{project}-{name}/` whose `{project}-{name}` is not produced by the skills manifest above and is not listed in `.govern.toml` `pinned.files`; report removals and pinned-keeps as for the `claude-style` cleanup. Skill dirs outside the `{project}-*` namespace (and the `govern` skill) are adopter/agent territory and are never touched.

### Workflow recommendation (strategy: create per accepted workflow)

**Skip this entire section when the agent's `layout` is `antigravity`** — workflow-skill scaffolding is deferred for Antigravity (the tech-stack-gated workflow commands are not yet adapted to the skill layout); the pipeline skills above are the adoption surface. For `claude-style` agents, proceed as below.

After the slash command cleanup, offer any newly registered workflows that match the project's tech stack and have not yet been scaffolded for this agent. Adopter cleanup of legacy workflow filenames and the legacy `skills/` directory is handled by the **Pre-run Migrations** section earlier in this procedure — see `framework/migrations.toml` entries `workflow-filename-rename` and `skills-to-workflows`.

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

3. **Load recorded declines.** Read `.govern.toml` if it exists and collect entries from `[workflows] declined_categories` into a normalized lowercase set. This set is consulted at the per-category prompt step to suppress prompts for categories the user has previously chosen to permanently decline. Behavior:

   - If `.govern.toml` does not exist: the decline set is empty. Skip silently.
   - If `.govern.toml` exists without a `[workflows]` section: the decline set is empty. Skip silently.
   - If `[workflows]` exists without a `declined_categories` key, or the key is an empty array: the decline set is empty. Skip silently.
   - If the file is malformed (TOML parse error): the surrounding **Project Configuration** load already aborted the run; this step never executes on a malformed file.

   While building the set, validate each entry case-insensitively against the canonical category list (`Linting`, `Formatting`, `Testing`, `Migrations`, `Code Review`, `Deployment`). Entries that don't match any canonical name are still loaded into the set (they cannot suppress anything because no category will hash to them) and recorded for the post-scaffolding summary as one line each: `unrecognized workflow decline: "{value}" (in .govern.toml)`. Unrecognized entries do not abort the run and do not affect prompts for valid categories.

4. **Match registry entries** against the project's tech stack. For each entry, look up the project's value for `entry.trigger.field` and compare case-insensitively against `entry.trigger.value`. Collect every matching entry.

5. **Filter out already-scaffolded workflows.** For each match, check whether `{config_dir}/commands/{project}/workflows/{entry.template}` already exists. If it does, the workflow was previously scaffolded (for this agent) — drop it from the candidate list. Already-scaffolded workflow files are never overwritten, regardless of content changes upstream.

6. **Silent skip when there is nothing new to offer.** If no candidates remain, do not prompt the user and proceed to **Session state**.

7. **Group remaining candidates by category** in the order: `Linting`, `Formatting`, `Testing`, `Migrations`, `Code Review`, `Deployment`. Within each category, list each match's `name` and `description`.

8. **Per-category prompt or suppress.** Walk the grouped categories in order. For each category:

   - **Suppress branch.** If the category (lowercased) is in the decline set loaded at step 3, do not invoke `AskUserQuestion`. Skip scaffolding for this category's workflows entirely. Report `suppressed (workflow): {Category} (declined in .govern.toml)` in the post-scaffolding summary, using the category's title-case display name. Continue with the next category.
   - **Prompt branch.** Otherwise, present `AskUserQuestion`: "Scaffold these {category} workflows for {agent name}?" with the matched entries listed. Options, in order, exactly as labeled:

     1. `Yes, scaffold all in this category`
     2. `Skip this run`
     3. `Skip and don't ask again`

     The user must explicitly accept — no workflows are scaffolded without consent. Route the answer:

     - `Yes, scaffold all in this category` — proceed to step 10 with this category's matched entries marked as accepted.
     - `Skip this run` — skip scaffolding for this category's workflows. Write nothing to `.govern.toml`. The user will be asked again on the next run.
     - `Skip and don't ask again` — skip scaffolding for this category's workflows AND mark the category for persistence (consumed at step 9).

9. **Record persisted declines.** For every category whose answer at step 8 was `Skip and don't ask again`, append the category name (in title case) to `[workflows] declined_categories` in `.govern.toml`. Behavior:

    - **`.govern.toml` does not exist** — create it with exactly:

      ```toml
      [workflows]
      declined_categories = ["{Category}"]
      ```

      Report `created .govern.toml to record decline` in the post-scaffolding summary (one line, regardless of how many categories were declined this run).

    - **`.govern.toml` exists without a `[workflows]` section** — append the section at the end of the file (preceded by a blank line). Use the same shape as the create case.

    - **`[workflows]` section exists without a `declined_categories` key** — add the key inside the existing section.

    - **`declined_categories` key exists** — append the new category name to the array, deduplicating case-insensitively (do not write a duplicate if `Linting` is added when `linting` is already present).

    Preserve all existing TOML content: other sections (`[pinned]`, future sections), comments, ordering, and surrounding whitespace. Read the file, modify the `[workflows]` section in place, and write the result back. Report each newly persisted category once in the summary as `recorded decline (workflow): {Category} (in .govern.toml)`.

10. **Fetch and write accepted workflows.** For each accepted entry (categories whose step-8 answer was `Yes, scaffold all in this category`):

    - Fetch `framework/workflows/{entry.template}` from the `govern` repo using the same URL pattern as the rest of `govern`'s fetches. (Note: the workflows directory is flat — no inner `templates/` subdirectory.)
    - If the fetch fails or the file is missing, warn `Workflow file {entry.template} not found, skipping` and continue with the next accepted entry. Do not abort the surrounding scaffolding.
    - Replace every `{project}` with the user-provided project name and every `{cli-config-dir}` with the agent's `config_dir`.
    - Write the substituted content to `{config_dir}/commands/{project}/workflows/{entry.template}` (creating the `workflows/` directory if needed). Report the file as "scaffolded" in the post-scaffolding summary.

11. **Discovery note for Auggie.** Auggie's official docs document subdirectory namespacing for one level (`.augment/commands/foo/bar.md` → `/foo:bar`). Multi-level paths like `.augment/commands/{project}/workflows/lint.md` should resolve to `/{project}:workflows:lint` by the same colon-namespace convention, but a user adopting Auggie may want to confirm autocomplete the first time. Claude Code's two-level path is documented and works as expected.

### Session state

The session state file lives at `.govern.session.toml` at the repo root — host-agnostic, project-name-agnostic, gitignored. The bootstrap does not create it; it's written on the first `/{project}:target` (or its scenario sibling) invocation by the runtime's `write-session` primitive (or, on the markdown-only path, by the host's file-writing tool). Each adopted agent reads from the same file; there is no per-agent session state.

### `govern` self-installation (strategy: update)

Fetch `framework/bootstrap/govern.md` and write it to the agent's `govern` install path: `{config_dir}/commands/govern.md` for `claude-style`, or `{config_dir}/skills/govern/SKILL.md` for `antigravity`. This is the same unified file the user is currently running, installed into every selected agent so the command is invokable from that agent on subsequent runs. For `antigravity`, wrap the body in `name: govern` frontmatter (the dir-form skill); the body keeps every placeholder literal (next paragraph).

In this file (and only this file), keep **every** placeholder literal — do **not** substitute anything. `{project}` and `{cli-config-dir}` must stay literal so `govern` itself can read `$ARGUMENTS` and the per-agent config dir on each run; `{project-name}` and `{One-line project description.}` must stay literal because this file's prose *documents* those placeholders for the AGENTS.md template — substituting them would corrupt the documentation, not personalize a value.

After writing, run the **Post-Write Integrity Check** below.

## Hook Installation

After **Per-Agent Scaffolding** completes, manage the project's git pre-commit hook so generated artifacts (currently spec `dependencies:` frontmatter, future generators if added) stay in sync on every commit.

Two files participate, with different ownership models:

- **`.githooks/govern-pre-commit`** is govern-owned. Placed by the **Shared Files** manifest with `update` strategy; carries the `# managed-by: govern` sentinel on line 2; rewritten on every `/govern` run unless pinned in `.govern.toml`. Holds the generator orchestration (currently `scripts/gen-spec-deps.sh` plus output staging).
- **`.githooks/pre-commit`** is adopter-owned. Placed by the manifest with `create` strategy on first install; never overwritten thereafter. Initial content invokes `./.githooks/govern-pre-commit`; adopters add their own pre-commit checks above or below that invocation.

This section's job is to wire git up to actually run the outer hook (`git config core.hooksPath .githooks`) without clobbering whatever hook system the project already uses.

Detection runs in this order — first match wins:

1. **`core.hooksPath` already points at `.githooks`** — already wired up. The manifest passes have already written `.githooks/govern-pre-commit` (`update`) and, on first run, `.githooks/pre-commit` (`create`). Run `chmod +x .githooks/pre-commit .githooks/govern-pre-commit` to ensure both files are executable. Report `pre-commit hook already wired up`.
2. **`core.hooksPath` points at any other path** — the project uses a custom hooks dir. Skip wiring; report a warning with the manual integration snippet below.
3. **A third-party hook system is detected** — any of `.husky/`, `.pre-commit-config.yaml`, `lefthook.yml`, or `lefthook-local.yml` exists. Skip wiring; report a warning with the manual integration snippet below.
4. **No conflicts** — run `git config core.hooksPath .githooks` and `chmod +x .githooks/pre-commit .githooks/govern-pre-commit`. Report `pre-commit hook installed`.

The detection ladder no longer treats `.githooks/pre-commit` itself as a govern-managed file — under the new model the outer file is adopter-owned, so its presence is not a signal that govern installed it. Migration of pre-existing govern-installed hooks (from spec-017 adopters) is handled by the **Migration from spec-017 hook** subsection below, which runs before the detection ladder.

`scripts/gen-spec-deps.sh` ships in the **Shared Files** manifest with `update` strategy. Every `/govern` run refreshes it from upstream so adopters pick up generator fixes automatically. Adopters who have customized the script can list it in `.govern.toml` `pinned.files` to opt out of overwrites.

### Migration from spec-017 hook

Adopters who installed the pre-commit hook under spec 017 have a single govern-managed file at `.githooks/pre-commit` carrying the `# managed-by: govern` sentinel on line 2. The new layout splits that file into a govern-owned inner script and an adopter-owned outer stub at the same path. Migration runs **before** the detection ladder above and **before** the manifest passes for the two hook files, so the manifest's `update`/`create` strategies see the post-rename layout.

Trigger:

- `.githooks/pre-commit` exists, AND
- the file's line 2 is exactly `# managed-by: govern`, AND
- `.githooks/govern-pre-commit` does **not** exist.

When all three hold, perform the rename:

1. Determine whether the file is tracked: `git ls-files --error-unmatch .githooks/pre-commit` (exit code 0 = tracked).
2. If tracked: `git mv .githooks/pre-commit .githooks/govern-pre-commit`. If untracked: `mv .githooks/pre-commit .githooks/govern-pre-commit`.
3. Continue with the detection ladder and the manifest passes. The renamed inner file is byte-identical to upstream for unmodified adopters, so the `update` strategy on `.githooks/govern-pre-commit` is a no-op; the `create` strategy on `.githooks/pre-commit` writes the new outer stub since the path is now empty.
4. Append to the post-scaffolding summary: `migrated pre-commit hook: .githooks/pre-commit → .githooks/govern-pre-commit; created adopter-owned .githooks/pre-commit stub`.

Recovery branches:

- **Pre-existing `.githooks/govern-pre-commit` blocks the rename.** If the inner-file destination already exists when the trigger fires, abort the rename without renaming anything. Report `migration skipped: .githooks/govern-pre-commit already exists; resolve manually` and continue with the detection ladder and manifest passes. The `update` strategy overwrites the pre-existing inner with the shipped contents; the existing `.githooks/pre-commit` (still carrying the sentinel) is left in place but is no longer detected as govern-managed by the new ladder, so it is treated as adopter-owned going forward. The adopter resolves the duplicate manually.
- **`git mv` fails (permissions, repo locked, file in use).** Report `migration failed: could not rename .githooks/pre-commit; resolve manually` and continue with the detection ladder and manifest passes. The `update` strategy installs `.githooks/govern-pre-commit` from scratch (destination doesn't exist); the `create` strategy sees `.githooks/pre-commit` still in place and skips. The adopter ends up with both files (legacy sentinel'd outer still functional, new govern-owned inner idle) and completes the migration manually by editing the outer to call `./.githooks/govern-pre-commit`.

If any of the trigger conditions does not hold, skip the migration silently — the detection ladder handles the case.

### Manual integration snippet (for skip cases)

When detection skips installation (cases 2 and 3 above), report this message to the user:

> The `govern` pre-commit hook was not wired up because your project already uses an existing hook system. To get automatic spec-deps regeneration on every commit, add this line to your existing pre-commit chain:
>
> ```bash
> ./.githooks/govern-pre-commit
> ```
>
> The shipped hook script is idempotent and safe to call from another hook runner.

### Pinning

Both hook files are subject to `.govern.toml` `pinned.files`, but the meaning differs by ownership:

- **`.githooks/govern-pre-commit`** is the only file pinning is meaningful for. A pinned inner file uses `skip` strategy instead of `update` — `/govern` does not overwrite it across releases. Useful when an adopter has customized govern's generator orchestration and does not want it reset.
- **`.githooks/pre-commit`** is `create`-strategy and never overwritten after first run regardless of pinning. Listing it in `pinned.files` is harmless but has no effect.

The Hook Installation section above still runs and may set `core.hooksPath` regardless of pinning.

## Placeholder Substitution

In every copied file (except each selected agent's installed `govern` file — `{config_dir}/commands/govern.md` for `claude-style`, `{config_dir}/skills/govern/SKILL.md` for `antigravity` — whose body keeps `{project}` and `{cli-config-dir}` as literal placeholders), replace:

- `{project}` with the user-provided project name (used in commands, README)
- `{project-name}` with the user-provided project name (used in AGENTS.md template)
- `{One-line project description.}` with the user-provided description
- `{cli-config-dir}` with the agent's `config_dir`

## Post-Write Integrity Check

After writing the agent's installed `govern` file — whether via the **govern.md Self-Update Check** (stale-write path) or the **`govern` self-installation** manifest step — verify it is well-formed. For `claude-style` (`{config_dir}/commands/govern.md`), the file must start with `# govern`. For `antigravity` (`{config_dir}/skills/govern/SKILL.md`), the file must start with a frontmatter block whose `name:` is `govern`, and the body after that frontmatter must start with `# govern`. If the check fails, the write was corrupted — report the error and re-read the source: `{tempdir}/govern.md.upstream` for the self-update path, or `{tempdir}/govern-main/framework/bootstrap/govern.md` for the manifest path. Apply the check independently per agent.

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
- **Pinned `govern.md` in `.govern.toml`** — the manifest's `update` strategy still skips the file (no overwrite), and the **govern.md Self-Update Check** never writes pinned files even on the stale-detect path. The check byte-compares anyway: matching upstream → recorded as `current`, no output; divergent from upstream → recorded as `pinned-divergent`, the run continues, and a single advisory line is printed in the post-scaffolding output. A pinned `govern.md` will not pick up upstream changes until the pin is removed, but the user is told once when the pin is currently suppressing real divergence.
- **Self-update check sees a stale `govern` in an unselected adopted agent** — the check is scoped to selected agents only. The unselected agent's stale copy is not diffed, not written, and does not trigger the abort; it will be detected the next time the user runs `/govern` against it.
- **Self-update small fetch fails** — clean abort with the error message defined in **govern.md Self-Update Check → Small fetch**. No `govern.md` writes occur, and the archive fetch is skipped. The user re-runs after the transient failure clears.
- **Archive fetch or extract fails** — clean abort with the error message defined in **File Fetching → Archive fetch and extract**. The self-update check has already passed by this point, so no additional `govern.md` writes are pending; the user re-runs after the transient failure clears.
- **A required source file is absent from the extracted archive** — warn `Source not found in archive: {source-path}; skipping.` and continue with the remaining manifest entries. Preserves the per-entry "do not abort on a single fetch error" guarantee at the entry level even though the archive itself is fetched once.
- **First-run prompt with no detected dirs and only one supported agent** — the prompt still appears (the agent must be explicitly chosen), but the single agent is pre-selected. Confirming is one keystroke.
- **Running `govern.md` cannot infer its own install path** — fall back to no pre-selection in the first-run prompt. The user picks explicitly.

## Post-Scaffolding Output

After scaffolding, display:

- Summary of files created, updated, unchanged, skipped, pinned, merged, and removed — grouped by agent for per-agent files, with shared files in their own group
- For each scaffolded agent, the agent's `rules_file_note` from the registry
- Hook installation status — one line: `pre-commit hook installed`, `pre-commit hook already wired up`, or `pre-commit hook skipped — existing {husky|lefthook|pre-commit-py|core.hooksPath} detected; see manual integration snippet above`. When the spec-017 → spec-018 migration ran, append the migration summary line described in §Hook Installation > Migration from spec-017 hook (or the relevant recovery-branch warning if the rename was skipped or failed).
- Any fetch failures encountered
- Pinned `govern.md` advisory (if applicable — see below)
- Security audit summary (if applicable — see below)
- Next steps (varies by mode):

### Pinned `govern.md` advisory

If the **govern.md Self-Update Check** recorded any selected agent as `pinned-divergent` (the installed `govern` file (`{config_dir}/commands/govern.md`, or `{config_dir}/skills/govern/SKILL.md` for `antigravity`) is listed in `.govern.toml` `pinned.files` and differs from upstream), append one advisory line per divergent agent after the file summary and before next steps:

> {agent}: govern.md pinned, upstream has changed.

The advisory is omitted when no agent is `pinned-divergent` — adopters whose pinned version still matches upstream see nothing; adopters with no pin see nothing. The check's `stale` path aborts before this output is ever produced, so the advisory is only ever about pinned files.

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

**govern adopted successfully.**

Adopted agents: {comma-separated `name` of selected agents}.

Next steps:

1. Run `/{project}:configure` in each adopted agent to apply the full permission set.
2. Fill in `AGENTS.md` — tech stack, project structure, code style, testing conventions, gotchas.
3. Fill in `specs/system.md` — architecture, request lifecycle, shared infrastructure.
4. Use `/{project}:log` to record any known issues or bugs into `specs/inbox.md`.
5. Run `/{project}:groom` to walk the inbox and route each item to its proper spec or scenario.
6. Create your first feature spec: `/{project}:specify {feature description}`.
7. Optional: install the deterministic runtime for faster slash commands — see [Runtime](https://github.com/stonean/govern#runtime) in the govern README.

To adopt an additional agent later, re-run `/govern --add-agent`.

Tip: `specs/` is plain markdown and works in any PKM tool (Obsidian, Logseq, Foam) or as a published site (Quartz, MkDocs). Pick whichever fits your workflow, or none.

---

### Update mode (existing `specs/` directory detected)

---

**govern updated successfully.**

Updated agents: {comma-separated `name` of selected agents}.

Review changes to updated files and commit when ready. To adopt an additional agent, re-run `/govern --add-agent`.

Tip: `specs/` is plain markdown and works in any PKM tool (Obsidian, Logseq, Foam) or as a published site (Quartz, MkDocs). Optional: install the deterministic runtime for faster slash commands — see [Runtime](https://github.com/stonean/govern#runtime) in the govern README.

---

## Idempotency

This command is safe to run again. Files with `update` strategy are always overwritten with the latest `govern` version — unless pinned in `.govern.toml`, in which case they are skipped. Files with `create` strategy skip existing files. The `.gitignore` merge checks for the `# govern` marker before appending. `skip` strategy files are never overwritten.

Re-runs are additive across agents — adopting a new agent leaves existing agents' files untouched.

## Directory Creation

Create intermediate directories as needed (e.g., `specs/`, `specs/templates/`, and — by layout — `{config_dir}/commands/{project}/` for `claude-style`, or `{config_dir}/skills/` and `{config_dir}/rules/` for `antigravity`).
