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

> **For agent runtimes**: backticked primitive names in this section (`fetch-archive`, `extract-archive`, `apply-manifest`, `merge-managed-block`, `enforce-manifest`) map to MCP tools the optional [gvrn runtime](https://crates.io/crates/gvrn) exposes under bare `<primitive>` names (e.g., `fetch-archive`). Hosts wrap them with a server-name prefix taken from the agent's MCP registration (Claude: `mcp__gvrn__fetch-archive`; Auggie: `mcp:gvrn:fetch-archive`). When the server is registered for your session, **call the corresponding tool** for each step listed below — that is the deterministic path. When it is not registered, walk the markdown-only reference below (`tar -xzf`, `curl`, etc.) to produce the same result. The two paths share a contract; neither one wraps the other.

**Procedural fidelity.** Execute the steps below as written. The only confirmation prompts to issue are those the procedure specifies: project inputs (§Inputs), agent-selection prompts on `--add-agent` / first-run (§Agent Selection), the registry-driven migration prompts (§Pre-run Migrations — outer "apply N pending migrations" prompt plus any per-entry inner prompts the procedure files specify), and per-category workflow prompts (§Workflow recommendation, step 8). Do not stop to warn about uncommitted edits to update-strategy files, custom slash commands that **Slash command cleanup** is about to remove, or "data loss" from the stale → write-and-abort path. The procedure already encodes safety: `.govern.toml` `[pinned] files` is the opt-out, the stale path writes upstream and aborts cleanly (recoverable from git), and slash-command cleanup is unconditional for unpinned files. Extra prompts duplicate information the procedure already gives the user and stall routine runs.

1. The walker context carries the inputs the host has already gathered and validated: project (the destination project name), description (one-line project description), languages (comma-separated), agents (registry keys), framework-version (release tag), archive-url and sha256-url (computed from framework-version), staging-dir, substitutions-map, manifest-entries (the per-strategy list described in **Shared Files** and **Per-Agent Scaffolding**), pinned-list (from `.govern.toml`'s `[pinned] files` block), gitignore-block (the `.claude/`, `.augment/`, `.agents/`, `.opencode/`, `specs/.cache/`, etc. lines), host-block (the `project` value — the team-shared slash-command namespace — written to committed `.govern.toml`, plus the per-contributor `cli-config-dir` written to the gitignored `.govern.session.toml` since teammates may use different agents; the runtime reads both at `gvrn exec` time to resolve `{cli-config-dir}/commands/{project}/<name>.md`), enforce-directories (the slash-command directories whose top-level `*.md` files are pruned to the manifest), and the per-agent govern-install entry with `keep-literals: ["project", "cli-config-dir"]`. The host runs the markdown-only reference below to collect inputs, derive registry values, validate `.govern.toml`, and seed context; the runtime walks the procedure that follows.

2. Invoke `fetch-archive` (MCP: `fetch-archive`) to download the framework tarball. The primitive verifies the sha256 against a sidecar URL when one is supplied; without a sidecar (the live-on-main case, since GitHub's auto-generated source tarballs ship without sidecars) it returns the computed digest and `verified: false`, leaving any out-of-band verification to the host. A sidecar mismatch halts the procedure with an `error` envelope so no partial state lands in the destination tree.

3. Invoke `extract-archive` (MCP: `extract-archive`) to expand the verified tarball into the staging directory. Path-traversal protection is applied per entry; symlinks are skipped. Otherwise, follow the markdown-only path's `tar -xzf` workflow.

4. Invoke `apply-manifest` (MCP: `apply-manifest`) with the host-built manifest entries and the pinned list. The primitive walks each entry, applies the per-entry strategy (update for framework-owned files, create for adopter-seedable files, skip-if-conflict for adopter-owned templates — the three strategy values defined in **Shared Files** below), short-circuits on the pinned list, returns aggregate counts the host surfaces in the completion message. This single call replaces the per-file update / create / skip loops the markdown-only reference describes below.

5. Invoke `merge-managed-block` (MCP: `merge-managed-block`) against `.gitignore` with `marker-style: "line-prefix"` and `marker: "govern"` to install or update the framework-managed block (the `.claude/`, `.augment/`, `.agents/`, `.opencode/`, `specs/.cache/`, etc. lines). First-run creates the file; subsequent runs update only the region between the `# govern` preamble line and the next blank line, preserving the rest of the file byte-for-byte. Replaces the inline `grep` check the markdown-only reference describes for the `.gitignore` merge step.

6. Establish the host configuration, split by audience. (a) Invoke `merge-managed-block` (MCP: `merge-managed-block`) against `.govern.toml` with `marker-style: "line-prefix"`, `marker: "govern (host)"`, and a block carrying **only** the resolved `project` value (the team-shared slash-command namespace). First-run creates the file with just the managed block; subsequent runs update only the region between the `# govern (host)` preamble line and the next blank line, preserving every other `.govern.toml` section (`[pinned]`, `[workflows]`, `[migrations]`, `[review]`) byte-for-byte — and dropping any legacy `cli-config-dir` key a prior version wrote into the managed block. (b) Invoke `write-session` (MCP: `write-session`) with `cli-config-dir` set to the agent's resolved config-dir and **no** target fields — a host-config write that records the per-contributor agent identity in the gitignored `.govern.session.toml` (preserving any existing target), never in committed config, because teammates on one project may each use a different agent. The runtime reads `project` from `.govern.toml` and `cli-config-dir` from the session file at `gvrn exec` time to resolve `{cli-config-dir}/commands/{project}/<name>.md`; absent either, it falls back to `.claude` / repo directory basename — fine for the framework's own repo, broken for any adopter whose layout doesn't match the defaults. On the markdown-only path, the host writes both the `.govern.toml` `[host]` block and the session-file `cli-config-dir` key with its file-writing tool. See §Project Configuration for the `[host]` schema.

7. Invoke `enforce-manifest` (MCP: `enforce-manifest`) once per directory in the host's enforce-directories list (typically the per-agent slash-command directory). The primitive removes files matching the glob-include arg (default `*.md`) whose relative path is neither in the expected list nor pinned. One call replaces the slash-command manifest enforcement loop the markdown-only reference describes. Adopter cleanup of historical conventions (legacy `skills/` directory, post-005 workflow filename rename, and the rest) is owned by the **Pre-run Migrations** section above and the `framework/migrations.toml` registry it drives.

8. Invoke `apply-manifest` (MCP: `apply-manifest`) a second time with a single entry for the per-agent `govern` self-install (the `{cli-config-dir}/commands/govern.md` path) and an **empty substitutions map** (`{}`). `govern.md`'s body contains prose references to every placeholder name the bulk step substitutes — `{project}`, `{cli-config-dir}`, `{project-name}`, `{One-line project description.}` — describing what those placeholders mean in *other* files. None of them are values to substitute in `govern.md` itself, so the self-install call passes no substitutions rather than relying on `keep-literals` to mask individual keys from the full map. The split from step 4 isolates the no-substitute concern from the bulk substitute step.

9. Render the completion message (host responsibility): list the agents configured, the next pipeline command (`/{project}:specify`), the optional runtime install pointer (see the README's Runtime section), and any per-agent post-install reminders from the registry rows above.

## Agent Registry

The registry lists every supported agent. Per-agent paths and behaviors are derived from these rows — the rest of this file references registry values, not agent names.

| `key` | `name` | `config_dir` | `layout` | `settings_template` | `rules_file_note` |
| --- | --- | --- | --- | --- | --- |
| `claude` | Claude Code | `.claude` | `claude-style` | `{ "permissions": { "allow": ["Bash(curl *)", "Bash(ls *)", "Bash(tar *)", "Bash(mktemp *)", "Bash(git status *)", "Bash(git config *)", "Bash(git rev-parse *)", "Bash(git diff *)", "Bash(git ls-files *)", "Bash(chmod *)", "Bash(awk *)", "Bash(command -v *)", "Read(/private/var/folders/**/T/govern-*/**)", "Read(//private/var/folders/**/T/govern-*/**)", "Read(/var/folders/**/T/govern-*/**)", "Read(//var/folders/**/T/govern-*/**)", "Read(/tmp/govern-*/**)", "Read(//tmp/govern-*/**)"], "deny": [] } }` | Claude Code reads `CLAUDE.md` natively. |
| `auggie` | Auggie | `.augment` | `claude-style` | `{ "toolPermissions": [ { "toolName": "launch-process", "shellInputRegex": "^curl ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^ls ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^tar ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^mktemp ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^git status ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^git config ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^git rev-parse ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^git diff ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^git ls-files ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^chmod ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^awk ", "permission": { "type": "allow" } }, { "toolName": "launch-process", "shellInputRegex": "^command -v ", "permission": { "type": "allow" } } ] }` | Auggie reads `CLAUDE.md` natively — no second rules file is needed. |
| `antigravity` | Antigravity | `.agents` | `antigravity` | `{ "permissions": { "allow": [ "command(curl)", "command(ls)", "command(tar)", "command(mktemp)", "command(git status)", "command(git config)", "command(git rev-parse)", "command(git diff)", "command(git ls-files)", "command(chmod)", "command(awk)", "command(which)" ], "deny": [], "ask": [] } }` | Antigravity reads `AGENTS.md` natively — no second rules file is needed. |
| `opencode` | OpenCode | `.opencode` | `opencode` | `{ "$schema": "https://opencode.ai/config.json", "permission": { "bash": { "curl *": "allow", "ls *": "allow", "tar *": "allow", "mktemp *": "allow", "git status *": "allow", "git config *": "allow", "git rev-parse *": "allow", "git diff *": "allow", "git ls-files *": "allow", "chmod *": "allow", "awk *": "allow", "command -v *": "allow" } } }` | OpenCode reads `AGENTS.md` natively — no second rules file is needed. |

### Derived values

For each agent, these paths and behaviors are computed by convention from its row — they are **not** stored in the table. Values that are the same for every agent are layout-independent; the rest are selected by the row's `layout` field.

**Layout-independent (every agent):**

| Derived value | Formula |
| --- | --- |
| Configure source path | `framework/bootstrap/configure/{key}.md` |

**Layout-derived (selected by `layout`):**

| Derived value | `claude-style` | `antigravity` | `opencode` |
| --- | --- | --- | --- |
| Command/skill path | `{config_dir}/commands/{project}/<name>.md` | `{config_dir}/skills/{project}-<name>/SKILL.md` | `{config_dir}/command/{project}/<name>.md` |
| Invocation | `/{project}:<name>` | `/{project}-<name>` | `/{project}/<name>` |
| `govern` install path | `{config_dir}/commands/govern.md` | `{config_dir}/skills/govern/SKILL.md` | `{config_dir}/command/govern.md` |
| Settings file | `{config_dir}/settings.local.json` | `{config_dir}/settings.json` | `opencode.json` (repo root; same file as MCP wiring) |
| Permission shape | `permissions.allow/deny` (Claude) / `toolPermissions[]` (Auggie) | `permissions.allow/deny/ask` (action grammar) | `permission` action map (`allow`/`ask`/`deny`) |
| Native rule-loading dir | — (rules read from shared `specs/rules/`) | `{config_dir}/rules/<name>.md` | — (rules read from shared `specs/rules/`) |
| Native rules file | `CLAUDE.md` | `AGENTS.md` | `AGENTS.md` |
| Slash-command cleanup glob | `*.md` in the commands dir | `{project}-*/` skill dirs in `skills/` | `*.md` in `command/{project}/` |

The session state file is `.govern.session.toml` at the repo root for every adopter — not a derived per-agent path (the path is uniform across agents). It's gitignored, and it additionally records the per-contributor `cli-config-dir` (see §Session state).

### MCP registration (per-agent)

MCP discovery is **not** layout-derived — it is a per-agent property. A host can share Claude's command/skill layout and native `CLAUDE.md` reading (Auggie does) yet register MCP servers somewhere entirely different. Each agent therefore declares its own MCP registration descriptor; the State-B auto-wire (§gvrn runtime detection) and §MCP wiring branch on the `mechanism` column.

| `key` | MCP target | scope | mechanism | surfaced instruction (when `surface-instruction`) |
| --- | --- | --- | --- | --- |
| `claude` | `.mcp.json` (repo root) | `project-committed` | `write-file` | — |
| `auggie` | `~/.augment/settings.json` | `user-global` | `surface-instruction` | `auggie mcp add gvrn --command gvrn --args "mcp"` |
| `antigravity` | `~/.gemini/config/mcp_config.json` | `home-level` | `surface-instruction` | edit `~/.gemini/config/mcp_config.json`, then `/mcp` reload |
| `opencode` | `opencode.json` (repo root) `mcp` block | `project-committed` | `write-file` | — |

- **`write-file`** — govern writes `target` additively at State-B wire time (the additive merge in §MCP wiring). Only `project-committed` agents use it.
- **`surface-instruction`** — govern writes **no** MCP file; State B surfaces the instruction in the Pre-flight abort and the user runs it once per machine, then restarts. Required for `user-global` / `home-level` agents, whose MCP config lives outside the repo and which govern must not silently mutate.
- **Antigravity** loads MCP servers only from home-level `~/.gemini/config/mcp_config.json`; project-local `.agents/mcp_config.json` is **ignored** (verified against the live `agy` CLI). There is no scriptable `agy mcp add`, so registration is a config-file edit plus a `/mcp` reload.

### Adding a new agent

A `claude-style` agent (markdown commands under `{config_dir}/commands/{project}/`, reads `CLAUDE.md`) is a one-row registry append plus an MCP registration entry plus two satellite files:

1. Append a row with the six fields (`layout: claude-style`).
2. Add a row to §MCP registration (per-agent). MCP discovery is per-agent, not layout-derived, so even a `claude-style` agent must declare its own `target` / `scope` / `mechanism` — it is **not** inherited from the layout.
3. Add `framework/bootstrap/configure/{key}.md` with the agent's full permission set in its native settings format.
4. Add a curl snippet for the new agent to the README's adoption section.

An agent on a **different layout** (a new value in the `layout` column) additionally needs its branch added to §Derived values and the layout-keyed steps in §Per-Agent Scaffolding and §Permission Setup — the work the `antigravity` and `opencode` layouts each introduced. (MCP registration is per-agent regardless of layout, covered by step 2 above.)

## Inputs

The project inputs are the **project name**, a one-line **description**, and the primary **language(s)**. From `$ARGUMENTS`, extract the project name now (a single non-flag word, if present) and recognize the flags below. **Do not prompt for any missing project input here** — interactive collection is deferred to **§Collect Project Inputs**, which runs *after* the **Pre-flight Phase**. Collecting them earlier means a pre-flight abort (a stale `govern.md` or a freshly-wired gvrn) discards the user's freshly-typed answers and forces them to re-enter everything on the restart. Nothing before §Collect Project Inputs — the pre-flight checks, agent selection, permission seeding, and the Pre-flight Phase itself — needs the interactive inputs; they need only `$ARGUMENTS` and the project's on-disk layout.

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

1. Read the agent's settings file — `{config_dir}/settings.local.json` for `claude-style`, `{config_dir}/settings.json` for `antigravity`, or the **repo-root `opencode.json`** for `opencode` (the same file as OpenCode's MCP-wiring target — settings and MCP wiring share one file; create it if missing, with the agent's `settings_template` from the registry; for `opencode`, merge into the adopter's existing `opencode.jsonc` instead if that is where their config lives).
2. Merge the agent's `settings_template` entries into the existing file additively: add any entries that are missing, do not deduplicate or reorder anything else, and do not overwrite entries the user or `/{project}:configure` previously added. For `claude-style` the entries live under `permissions.allow`/`permissions.deny` (Claude) or `toolPermissions` (Auggie); for `antigravity` they live under `permissions.allow`/`permissions.deny`/`permissions.ask`; for `opencode` they live under the `permission` action map (preserving `$schema` and every other top-level key).
3. Write the file if anything was added.

This prevents repeated permission prompts during the fetch and scaffolding phases. The full permission set is applied later by `/{project}:configure` (which writes the same per-layout settings file). The seed also includes the gvrn **binary probe** (`command -v gvrn` for `claude-style`, Auggie, and `opencode`, `which gvrn` for `antigravity`) so the **Pre-flight Phase**'s State B/State C probe does not prompt on routine runs.

### gvrn runtime auto-wiring

`/govern` wires the optional gvrn runtime automatically when its binary is detected on the session's `PATH` but not yet registered as an MCP server — the **Pre-flight Phase → State B** path. (This replaces the previous model where the runtime was a separate, hand-wired install.) Wiring depends on the agent's MCP registration `mechanism` (§MCP registration): a `write-file` agent gets its MCP file written; a `surface-instruction` agent gets a one-line registration command surfaced for the user to run (govern never writes the user's home config) — see **gvrn runtime detection → MCP wiring** for the per-mechanism rules. In the same pass, either way, `/govern` adds the **gvrn tool permissions** to the settings file so the next session calls the runtime without a per-tool prompt:

- **Claude** (`permissions.allow`): `mcp__gvrn__*`
- **Antigravity** (`permissions.allow`): `mcp(gvrn/*)`
- **Auggie** (`toolPermissions`): `{ "toolName": "mcp:gvrn:*", "permission": { "type": "allow" } }` if Auggie's matcher honors the wildcard, otherwise the enumerated `mcp:gvrn:<tool>` set `/{project}:configure` already installs.
- **OpenCode** (`permission`): `"gvrn*": "allow"` (a single glob in the root `opencode.json` `permission` map).

The wildcard is the minimal bootstrap grant; the enumerated per-tool set stays owned by the generated block in `/{project}:configure`'s permission file and coexists harmlessly (exact-match dedup leaves both). Both the wiring write and this permission write are additive and idempotent and follow the same merge rules as the seed above — no existing entry is removed, reordered, or overwritten. There is **no new confirmation prompt**: the wiring is disclosed by the **Pre-flight abort** message, which names every file written — consistent with the §Procedural-fidelity rule the silent seed writes already follow. The runtime remains an optional install (the binary is still installed out of band; see the README's Runtime section) — `/govern` automates only the MCP registration once the binary is present.

## Pre-flight Phase

Run a single pre-flight phase after the **Permission Setup** seed (so the gvrn binary probe is pre-authorized) and before **Pre-run Migrations** and the full archive fetch. The phase owns two restart-requiring checks — **gvrn runtime detection** and the **govern.md self-update check** — that can each force the session to restart: gvrn detection to load a newly-wired MCP server, the self-update check to load a fresh `govern.md`. Neither pays the cost of the multi-hundred-KB archive; both run on a small fetch or no fetch, so a restart-triggering abort never leaves archive work on disk.

The phase runs both checks, accumulates every restart-requiring write into a **pending-restart set**, and at the end emits a **single combined abort** if that set is non-empty (see **Pre-flight abort**). If neither check needs a restart, the run proceeds to **Pre-run Migrations**. Running both checks before the single abort is what collapses the worst case — a stale `govern.md` on an adopter who has never wired gvrn — into one restart instead of two.

### gvrn runtime detection

Detect whether the optional gvrn runtime is available and, when its binary is installed but not yet wired into this project, register it so the next session can run the deterministic path. Detection resolves to one of three states — A (runtime live this session), B (binary present, not wired), C (absent); the **Detection mechanism** and per-state behavior follow in the subsections below.

#### Detection mechanism

Two independent probes resolve the state:

- **Tool-inventory introspection (State A).** Inspect your own available-tool inventory for any `gvrn`-namespaced MCP tool — `mcp__gvrn__*` on Claude Code, `mcp:gvrn:*` on Auggie and antigravity — counting deferred or lazily-loaded tool names as present (a host that lists tool names before exposing their schemas still has the runtime registered). Any match ⇒ **State A**. This needs no shell and no permission; you always know your own tools.
- **Binary probe (State B vs. State C).** Only when introspection finds no `gvrn` tool, run a binary probe — `command -v gvrn` on `claude-style`, Auggie, and `opencode`, `which gvrn` on `antigravity` (whose token-prefix permission grammar matches `which` cleanly). The probe is pre-authorized by the **Permission Setup** seed, so it does not prompt on routine runs. There is no non-shell way to detect an installed-but-unregistered binary — anything a tool could answer would already be **State A**. If the probe cannot run (no shell granted) or is denied, classify the run as **State C** — a harmless false negative; detection never hard-fails.

#### State A — runtime live this session

A `gvrn`-namespaced tool is available to this session, so the runtime is live and the rest of the run takes the **deterministic primitive path**. gvrn contributes nothing to the **pending-restart set**, and detection emits no message.

State A is a **binding execution contract, not a preference.** Detecting the runtime and then walking the prose `curl`/`tar`/`python3` path anyway is the exact failure 029 exists to prevent — it spends the markdown path's tokens despite a cheaper path being live, and it is what makes the State-B wire-and-restart pointless. For the rest of this run:

- **Every step that names a backticked primitive** — a bare name (`fetch-archive`, `extract-archive`, `apply-manifest`, `merge-managed-block`, `enforce-manifest`, `substitute-templates`, `merge-permissions`, `merge-claude-md`, `run-generator`, …) that matches a `gvrn` tool in your inventory — **MUST be performed by calling that MCP tool** (`mcp__gvrn__<primitive>` on Claude, `mcp:gvrn:<primitive>` on Auggie/antigravity; mapping per §Instructions).
- **The shell commands shown under those steps** (`curl`, `tar -xzf`, `python3`, `awk`, byte-compares, hand-authored scaffold loops) are the **State-B/C fallback specification.** In State A they document the contract each tool fulfills; they are **not instructions to execute.** Do not run them. If you are about to run `curl`/`tar`/`python3` for a step that names a primitive, stop — that is the fallback path leaking into a State-A run; call the tool instead.
- **Steps with no backticked primitive run as shown in every state** — the per-language `.gitignore` `curl` against `github.com/github/gitignore`, `git config core.hooksPath`, `chmod`, the git repo / tracked-file checks, and the §Collect Project Inputs prompts have no tool equivalent.
- **If a primitive call errors** — e.g., a too-old wired `gvrn` surfaces a parse error per [spec 022 §Versioning enforcement](../022-deterministic-runtime/spec.md) — fall back to **that step's** shell specification for that one step and continue; do not abandon the deterministic path for the whole run.

#### State B — binary present, not wired

The binary probe succeeded but no `gvrn` tool is available to this session. In order:

1. Register the `gvrn` server per the agent's MCP registration `mechanism` (§MCP registration; details in **MCP wiring**): for `write-file`, write the MCP file additively; for `surface-instruction`, write **no** MCP file — the registration command is surfaced in the abort (step 3) for the user to run once per machine.
2. Add the permission entries needed to call the `gvrn` tools (see **Permission Setup**), so the next session calls them without a prompt. This write is the same for every agent regardless of `mechanism` — it targets the project-level settings file, not the MCP-server location.
3. Add the wiring (and the permission write) to the **pending-restart set** and contribute this notice to the combined **Pre-flight abort**, naming every file written:

> **gvrn runtime detected.** The `gvrn` binary is installed but was not registered for this project, so `/govern` could not use the faster deterministic runtime this run.

The abort takes the form matching the selected agent's `mechanism`:

- **`write-file` agent** (e.g. Claude): "It has now been wired in so the next session runs through the runtime, which uses far fewer tokens. Files written: {comma-separated paths — the wiring file, and the settings file when permission entries were added}."
- **`surface-instruction` agent** (e.g. Auggie): "{Agent} registers MCP servers in your user-level config, which `/govern` does not write. To enable the faster runtime, run this once, then start a fresh session: `{the agent's surfaced instruction from §MCP registration}`. Files written: {the settings file, when permission entries were added}."

State B issues **no separate consent prompt** — any file writes are additive and idempotent, matching the silent **Permission Setup** writes; the abort's file list (and, for a `surface-instruction` agent, the one-line command) is the disclosure. There is no opt-out flag for auto-wiring.

#### State C — binary absent

The binary probe failed, could not run, or was denied. Proceed on the markdown path exactly as today; gvrn contributes nothing to the **pending-restart set**. After scaffolding, the **Post-Scaffolding Output** emits one tip line noting that installing gvrn reduces token use.

#### MCP wiring

How State B registers `gvrn` depends on the agent's MCP registration `mechanism` (§MCP registration). For the `mcpServers`-shaped agents (Claude/Auggie/Antigravity) the server entry is a `mcpServers` map keyed by name; only the **location** (and whether govern writes it) differs:

```json
{ "mcpServers": { "gvrn": { "command": "gvrn", "args": ["mcp"] } } }
```

OpenCode uses a different shape — an `mcp` key with a typed local-server entry — written into the committed root `opencode.json` (the OpenCode sub-case below):

```json
{ "mcp": { "gvrn": { "type": "local", "command": ["gvrn", "mcp"], "enabled": true } } }
```

**`write-file` agents** (scope `project-committed` — Claude and OpenCode). govern writes the agent's `target` MCP file from §MCP registration, using that agent's server-entry shape — **Claude:** `.mcp.json` at the repo root, the `mcpServers` map, `{ "command": "gvrn", "args": ["mcp"] }`; **OpenCode:** the committed root `opencode.json` (or the adopter's existing `opencode.jsonc`), the `mcp` map, `{ "type": "local", "command": ["gvrn", "mcp"], "enabled": true }`. The write **updates the file in place — it never replaces or truncates it.** Apply the matching case (read `{servers-key}` as `mcpServers` for Claude, `mcp` for OpenCode):

- **Missing file** — create it containing only the `gvrn` entry (for OpenCode, include `"$schema": "https://opencode.ai/config.json"`).
- **Has `{servers-key}`, no `gvrn`** — add the `gvrn` entry; preserve every other server and every other top-level key (including OpenCode's `$schema` and `permission`).
- **Already has a `gvrn` entry** — no-op; leave the file byte-unchanged (idempotent re-run).
- **No `{servers-key}` key** — add the key with just the `gvrn` entry; preserve all other top-level keys.
- **Not valid JSON** — do **not** touch the file. Skip wiring, warn the user to repair it, and degrade to the markdown path for this run (treat as **State C**). A hand-maintained config is never clobbered.

There is no `gvrn` runtime primitive for this merge: State B is the runtime-absent case by definition, so the write is always host-side.

**`surface-instruction` agents** (scope `user-global` / `home-level` — Auggie and Antigravity). The agent reads MCP servers from a file in the user's **home** directory, shared across all their projects, which govern must **not** write. govern writes no MCP file; instead the **Pre-flight abort** surfaces the agent's registration instruction for the user to run once per machine, then restart:

- **Auggie** — `auggie mcp add gvrn --command gvrn --args "mcp"` (the documented, schema-stable subcommand; it writes `~/.augment/settings.json`).
- **Antigravity** — add the `gvrn` block above to `~/.gemini/config/mcp_config.json`, then reload via the in-prompt `/mcp` overlay (there is no scriptable `agy mcp add`; project-local `.agents/mcp_config.json` is ignored).

The permission write (State B step 2) still happens for these agents — it targets the project-level settings file the agent reads, independent of the home-level MCP-server location.

### Self-update check

Verify the running session's `govern.md` instructions are current.

#### Small fetch

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

#### Per-agent comparison

For each selected agent, compare the upstream `{tempdir}/govern.md.upstream` against the agent's installed `govern` file and assign one status. For `claude-style` the installed file is `{config_dir}/commands/govern.md` and for `opencode` it is `{config_dir}/command/govern.md` — both installed verbatim (frontmatter included), so the comparison is a direct byte-compare against `{tempdir}/govern.md.upstream`. For `antigravity` the installed file is `{config_dir}/skills/govern/SKILL.md`, which wraps **only the upstream body** in `name: govern` frontmatter — the installer drops `govern.md`'s own frontmatter when wrapping. So compare **bodies on both sides**: strip the leading frontmatter block (the first `---`-delimited region) from the installed `SKILL.md` **and** from `{tempdir}/govern.md.upstream`, then byte-compare what remains. Stripping only the `SKILL.md` side leaves `govern.md`'s frontmatter on the upstream side, which never matches — a false `stale` on every run. The statuses below are assigned from this body-vs-body (antigravity) or file-vs-file (`claude-style` / `opencode`) comparison:

- **`no installed copy`** — the installed file does not exist (first run for this agent). Continue.
- **`current`** — the two files are byte-identical, **or** the installed file is byte-identical to upstream and listed in `.govern.toml` `pinned.files` (the pin had nothing to suppress this run). Continue.
- **`stale`** — the two files differ and the installed file is **not** pinned. The running session is using older instructions than what is current upstream.
- **`pinned-divergent`** — the two files differ and the installed file **is** listed in `.govern.toml` `pinned.files`. The pin intentionally suppresses the update; continue, and emit a single advisory line in the post-scaffolding output.

The check is scoped to **selected agents only** — agents whose `config_dir` exists in the project but are not in this run's selection are not diffed. An unselected stale agent will trip the check on its very next `/govern` run targeting it.

#### Stale → defer to pre-flight abort

If any selected agent is recorded as `stale`:

1. For **each stale agent**, write the freshly fetched upstream to the agent's installed `govern` file (overwrite) so the next session loads the up-to-date instructions. For `claude-style`, copy `{tempdir}/govern.md.upstream` verbatim to `{config_dir}/commands/govern.md`; for `opencode`, copy it verbatim to `{config_dir}/command/govern.md`. For `antigravity`, write `{config_dir}/skills/govern/SKILL.md` as the transformed skill — `name: govern` frontmatter followed by the upstream body — **not** the raw `govern.md` (a raw copy is not a loadable skill). In both cases do not substitute placeholders in the body — `{project}` and `{cli-config-dir}` stay literal, per the `govern` self-install rule.
2. Run the **Post-Write Integrity Check** (see below) on each freshly written file.
3. Do not write `govern.md` for non-stale agents — their installed copies already match upstream.
4. Do not write `govern.md` for `pinned-divergent` agents — the pin opts them out of automatic updates.
5. Add each stale agent's overwrite to the **pending-restart set** and contribute this notice to the combined **Pre-flight abort** — do **not** abort here:

> **The govern command itself has updated.** Your installed copy was behind upstream and the running session is using the older instructions. The freshly fetched copy has been written to disk for stale agents.
>
> Stale agents updated: {comma-separated names}.

The shared "start a new session and re-run" line and the skip of every later section are owned by **Pre-flight abort**, so a stale `govern.md` and a freshly-wired gvrn surface in one abort and one restart rather than two.

#### Pinned-divergent → continue with advisory

If a selected agent is recorded as `pinned-divergent`, the run continues normally. After scaffolding, the **Post-Scaffolding Output** includes one advisory line per divergent agent (see **Post-Scaffolding Output → Pinned govern.md advisory**). The advisory is silent on runs where every pinned agent is `current` (the pinned version happens to match upstream this run).

Pinning is an opt-out from automatic updates, not an opt-out from knowing the pin is currently active. When the pinned version actually drifts from upstream, the user usually wants to either review the upstream changes and unpin, or consciously confirm they are staying on the old version. Adopters who are deliberately and indefinitely on an old version see no recurring nag because the advisory only fires when divergence is real.

#### Current / no installed copy → continue

When all selected agents are `current` or `no installed copy`, the self-update check contributes nothing to the **pending-restart set**. The temp directory created here is reused by the **Archive fetch and extract** step below — no second `mktemp`, no leaked extra temp directory. Whether the run proceeds is decided by **Pre-flight abort** once gvrn detection has also run.

### Pre-flight abort

After both checks have run, inspect the **pending-restart set**:

- **Empty** — no restart is needed. Proceed to **Pre-run Migrations**. (gvrn detection resolved to State A or State C, and the self-update check saw `current` / `no installed copy` / `pinned-divergent` for every selected agent.)
- **Non-empty** — emit one combined abort and stop before any further work. The message includes every contributed notice and names every file written during this phase:
  - the gvrn-wiring notice (State B), when gvrn was wired this run — see **gvrn runtime detection → State B**;
  - the stale-update notice, when any selected agent was `stale` — see **Self-update check → Stale → defer to pre-flight abort**;
  - a single shared closing line: **Start a new session and re-run `/govern` to pick up the changes.**

Everything past the pre-flight phase — **Collect Project Inputs**, **Pre-run Migrations**, **Project Configuration**, the **Archive fetch and extract**, **Frontmatter Migration**, **Shared Files**, **Per-Agent Scaffolding**, **Security Audit**, and **Post-Scaffolding Output** — is skipped. The only writes performed are the additive **Permission Setup** entries, any per-stale-agent `govern.md` overwrite, and any gvrn wiring plus its permission entries. Because input collection now lives past this point, an aborted run never prompts the user for the project name, description, or languages — they are asked exactly once, in the session that proceeds to scaffold. The next `/govern` run in a new session sees gvrn live (or absent) and every selected agent `current` (or `no installed copy`), and proceeds normally without abort.

## Collect Project Inputs

The Pre-flight Phase has passed (nothing in the pending-restart set), so this run will proceed to scaffold. **Only now** — never before the Pre-flight Phase — resolve the project inputs, so an abort can never discard answers the user just typed.

`.govern.toml` is the persistent home for these answers. Resolve each input from the first available source and **prompt only for what is still missing**:

1. **Project name** — from `$ARGUMENTS` (a single non-flag word, per §Inputs), else `[project] name` in `.govern.toml` (else `[host] project` for configs predating the `[project]` table), else prompt. Used for `{project}` substitution and command directory naming.
2. **Project description** — from `[project] description` in `.govern.toml`, else prompt. Used for AGENTS.md.
3. **Primary language(s)** — from `[project] languages` in `.govern.toml`, else prompt. Used for .gitignore language patterns.
4. **Rule surfaces** — from `[rules] surfaces` in `.govern.toml`, else prompt ("Which rule surfaces does this project need? backend / frontend / both"). Recorded as a list with members in `{backend, frontend}` ("both" records both). Selects which rule files `/govern` installs (§Shared Files) and which `/gov:review` enforces (`review.md` §Behavior step 5). When the recorded surfaces exclude a surface that `[project] languages` implies (e.g., a frontend language is listed but `surfaces` omits `frontend`), emit one advisory line and honor the explicit value. **Validate a present value before using it** (degenerate configs fail fast per `CFG-ENV-003`, never silently ignored): the **empty list** (`surfaces = []`) is valid and means cross-only — install only `*-cross.md`, no surface-suffixed files — and is distinct from the key being unset (which derives/installs all); an **unrecognized member** outside `{backend, frontend}` (a typo, or `"cross"` — cross-cutting files are not a selectable surface) halts with `/govern: invalid [rules] surfaces member "<value>" — accepted members are "backend" and "frontend" (use [] for cross-only; -cross.md files always apply)`, and a list mixing valid and invalid members fails on the invalid one; a **non-list value** (a bare string) halts with `/govern: [rules] surfaces must be a list of strings, got <type>`.

On a routine re-run (update mode) `.govern.toml` already carries all four, so this step prompts for nothing. On a first scaffold it prompts for whatever is missing, then **persists the three project inputs into `.govern.toml`'s `[project]` table** (`name`, `description`, `languages`) **and the rule surfaces into the `[rules]` table** (`surfaces`; see §Project Configuration), preserving every other section, so the next run — and the session after any State B / stale-`govern.md` restart — reads them back instead of re-asking. `host.project` continues to be written from `project.name` as the runtime's slash-command namespace.

When prompting (AskUserQuestion), every question **must** include an `options` array with 2–4 example choices (the user can always select "Other" for custom input):

- **Project name** — example options: the current directory name, `my-service`.
- **Project description** — example options: `A new microservice`, `CLI tool for X`.
- **Primary language(s)** — comma-separated list. Example options: `Go`, `Python`, `Node`, `Go, Python`.

Validate the project name: must be lowercase, alphanumeric, and hyphens only. If invalid, reject with: "Project name must be lowercase, alphanumeric, and hyphens only."

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
# `project` only — the team-shared slash-command namespace. The per-contributor
# `cli-config-dir` lives in the gitignored `.govern.session.toml` (teammates may
# use different agents), never here.
project = "gov"

[project]
# The inputs /govern collects (§Collect Project Inputs), persisted so re-runs
# and post-restart sessions read them back instead of re-prompting. This table
# is the source of truth for the answers; host.project below is the derived
# slash-command namespace, written from project.name.
name = "my-service"
description = "A new microservice"
languages = ["Go", "Python"]

[rules]
# Which rule surfaces /gov:review enforces and /govern installs. A list with
# members in {"backend", "frontend"}; full-stack lists both. "cross" is not a
# member — cross-cutting (-cross.md) rule files always apply. The empty list
# ([]) is valid and means cross-only (only -cross.md), distinct from the key
# being unset. Unset means "derive": /gov:review falls back to stack detection
# and /govern installs every rule file (pre-033 behavior). An unrecognized
# member or a non-list value fails fast. Collected by /govern (§Collect
# Project Inputs); read by /gov:review (§Behavior step 5).
surfaces = ["backend"]

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

`host.project` — the project's slash-command namespace, written by `/govern` into a managed block (`# govern (host)` line-prefix marker) in committed `.govern.toml` on every run (idempotent — re-runs update rather than append). The per-contributor `cli-config-dir` (the agent's config-dir name) is **not** committed: teammates on one project may each use a different agent, so `/govern` writes it to the gitignored `.govern.session.toml` instead (§Instructions step 6). The runtime reads `project` from `.govern.toml` and `cli-config-dir` from the session file at `gvrn exec` time to resolve `{cli-config-dir}/commands/{project}/<name>.md`; both fall back to `.claude` / the repo directory basename when absent. Adopters whose layout matches the defaults (this repo, anyone on Claude Code with the conventional `.claude/commands/<project>/`) never observe the difference; Auggie / OpenCode adopters and anyone with a non-standard layout do.

`project.name`, `project.description`, and `project.languages` — the project inputs collected at §Collect Project Inputs (name; one-line description for AGENTS.md; primary languages for .gitignore patterns), written into the `[project]` table additively (preserving every other section) and read back on every subsequent run so the inputs are asked at most once. `[project]` is the source of truth for the answers; `host.project` is written from `project.name` as the runtime's slash-command namespace (the derived runtime view of the same value), so the two cannot diverge. Editing a `[project]` value re-runs the corresponding scaffold step with the new value on the next `/govern` — the documented way to rename a project or change its languages. The table is host-side state (the host gathers inputs before the runtime walks per §Instructions step 1), so it is written on every adoption path without a runtime primitive.

`rules.surfaces` — the rule surfaces the project enforces and installs (§Collect Project Inputs, item 4). A list with members in `{backend, frontend}`; `-cross.md` rule files are unconditional and not selectable members. When unset, `/govern` installs every rule file and `/gov:review` derives the surface from stack detection (pre-033 behavior). When set, the **Shared Files** manifest pass installs only the rule files whose suffix matches a listed surface plus every `*-cross.md`, and `/gov:review` enforces only those (`review.md` §Behavior step 5). The **empty list** (`surfaces = []`) is a valid set value meaning **cross-only** — only `*-cross.md` is installed/enforced — and is distinct from the key being unset (the empty list declares "no surface rules"; unset means "derive"). A **degenerate value** fails fast per `CFG-ENV-003` rather than being silently ignored: an unrecognized member outside `{backend, frontend}` (a typo, or `"cross"`) and a non-list value both halt the command that reads the setting (`/govern` here, `/gov:review` in `review.md` §Behavior step 5), naming the offending value or type. Editing `surfaces` takes effect on the next `/govern`: newly-listed surfaces are installed, and rule files for a removed surface are **left in place** (not deleted — they are not in `enforce-directories`), they simply stop receiving updates.

`pinned.files` — any file listed that would normally use `update` strategy is treated as `skip` instead. Report pinned files in the post-scaffolding summary.

`migrations.last_applied` — slug of the newest pre-run migration applied to this project, written by `/govern` after each successful migration in §Pre-run Migrations. Absent section means "no migrations applied"; bootstrap runs every active entry on the next run. Adopters should not edit this field by hand — the registry in `framework/migrations.toml` and the per-entry procedure files in `framework/migrations/{id}.md` are the authoritative sources.

`workflows.declined_categories` — categories listed here suppress the per-category workflow recommendation prompt entirely (see the **Workflow recommendation** flow below). Entries that don't match any canonical category name are reported once each in the post-scaffolding summary as `unrecognized workflow decline: "{value}" (in .govern.toml)` but do not abort the run.

`review.disabled-rule-files` — array-of-tables consumed by `/gov:review` at rule-file selection time (see [`framework/commands/review.md`](../commands/review.md) §Inputs and §Behavior step 5). `/govern` does not read this key; it is documented here so adopters see the full `.govern.toml` schema in one place.

The full schema (allowed values, case-insensitive matching, empty-section behavior, future-section guidance) is declared in [`specs/019-config-decisions/data-model.md`](../../specs/019-config-decisions/data-model.md).

## File Fetching

Files from the `govern` repo are sourced from a single archive download, extracted into the temp directory established during the **Pre-flight Phase**, and resolved as local paths for the rest of the run. Per-language `.gitignore` patterns from `github.com/github/gitignore` are **not** part of this archive — they remain separate `curl` calls (see the **.gitignore** subsection of **Shared Files** below).

This section runs only after the **Pre-flight Phase** passes (no pending restart — no stale `govern.md` and no freshly-wired gvrn). On a pre-flight abort, the archive is never fetched.

**State A reminder:** the archive fetch/extract and the manifest passes below are primitive-backed. In a State-A run (gvrn live), call the `fetch-archive`, `extract-archive`, `apply-manifest`, and `enforce-manifest` tools — the `curl`/`tar` blocks shown are their State-B/C fallback spec, not commands to execute (see **§Pre-flight Phase → State A — runtime live this session**). The per-language `.gitignore` `curl` is *not* primitive-backed and runs as shown in every state.

### Archive fetch and extract

Issue exactly one `curl` against GitHub's archive host, downloading into the temp directory established during the pre-flight phase:

```text
curl -fsSL https://codeload.github.com/stonean/govern/tar.gz/refs/heads/main \
  -o {tempdir}/main.tar.gz
```

This is the direct `codeload.github.com` endpoint — the target that `https://github.com/stonean/govern/archive/refs/heads/main.tar.gz` 302-redirects to. Fetch it directly: the redirect form lands the command on a **new host mid-flight**, which some hosts (e.g. Antigravity) gate with a permission prompt even when a `curl` allow is pre-granted, because the grant matched the original host, not the redirect target. The direct URL has no redirect, so the bootstrap seed's `curl` pre-grant (`command(curl)` / `Bash(curl *)` / the Auggie `^curl` regex matcher) actually covers it. The archive's top-level directory is `govern-main/`; the framework files live at `govern-main/framework/...` after extraction.

After fetching:

1. Extract the archive into the existing temp directory: `tar -xzf {tempdir}/main.tar.gz -C {tempdir}`.
2. Compute the framework root: `{tempdir}/govern-main/`. Treat this as the local mirror of the `govern` repo for the rest of the run.

If the fetch or extraction fails — non-zero exit from `curl` or `tar`, or a missing `govern-main/` directory after extract — abort the run with this error and do not continue scaffolding:

> Failed to fetch or extract the `govern` archive ({reason}). Re-run after checking network connectivity, or report this if it persists.

A missing archive means **every** manifest entry would be missing, so partial scaffolding is impossible — the abort is the correct behavior. The pre-flight phase has already completed by this point, so a stale `govern.md` or a freshly-wired gvrn would have already triggered the pre-flight abort earlier.

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

This section runs **after the Pre-flight Phase** so that a stale-govern abort cannot leave migration changes from old rules on the working tree. The new govern's migration logic — which may differ — is the only logic that ever writes migration changes.

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

**Rule-file surface filter.** The `framework/rules/*.md → specs/rules/*.md` entries below are filtered by `[rules] surfaces` (§Project Configuration) before the manifest is applied: an entry is kept when its suffix matches a configured surface (`*-backend.md` for `backend`, `*-frontend.md` for `frontend`), and every `*-cross.md` entry is kept unconditionally. When `surfaces` is the **empty list** (`[]`), no surface-suffixed entry matches, so only the `*-cross.md` entries are kept (cross-only). When `surfaces` is unset, all rule files are kept (pre-033 behavior). (A degenerate `surfaces` value — an unrecognized member or a non-list — has already halted the run at §Collect Project Inputs item 4 before this filter runs.) Entries the filter omits are simply not applied — never pruned — so a rule file already on disk for a now-unconfigured surface is left in place (rule files are not in `enforce-directories`); it just stops receiving updates.

### `govern`-owned shared files (strategy: update)

| Source Path | Destination Path |
| --- | --- |
| `framework/constitution.md` | `constitution.md` |
| `framework/rules/accessibility-frontend.md` | `specs/rules/accessibility-frontend.md` |
| `framework/rules/api-backend.md` | `specs/rules/api-backend.md` |
| `framework/rules/concurrency-backend.md` | `specs/rules/concurrency-backend.md` |
| `framework/rules/configuration-cross.md` | `specs/rules/configuration-cross.md` |
| `framework/rules/observability-backend.md` | `specs/rules/observability-backend.md` |
| `framework/rules/performance-backend.md` | `specs/rules/performance-backend.md` |
| `framework/rules/performance-frontend.md` | `specs/rules/performance-frontend.md` |
| `framework/rules/quality-cross.md` | `specs/rules/quality-cross.md` |
| `framework/rules/security-backend.md` | `specs/rules/security-backend.md` |
| `framework/rules/security-frontend.md` | `specs/rules/security-frontend.md` |
| `framework/bootstrap/hooks/govern-pre-commit` | `.githooks/govern-pre-commit` |
| `scripts/gen-spec-deps.sh` | `scripts/gen-spec-deps.sh` |
| `scripts/gen-cross-service-refs.sh` | `scripts/gen-cross-service-refs.sh` |
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

**CLAUDE.md** (strategy: skip, `claude-style` only) — written only when at least one selected agent is `claude-style`. If it exists, leave it alone. Otherwise, when a `claude-style` agent is selected, fetch `framework/templates/project/claude-md.md` from the `govern` repo and copy it as `CLAUDE.md`. `claude-style` agents read `CLAUDE.md` natively (see each row's `rules_file_note`); the `antigravity` and `opencode` layouts read `AGENTS.md` natively and do not need `CLAUDE.md`, so an **Antigravity-only** or **OpenCode-only** adoption ships no `CLAUDE.md`. (`AGENTS.md` is still written for every adoption, as below.)

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

The steps below describe the **`claude-style`** layout. For an agent whose registry `layout` is **`antigravity`**, apply **### Antigravity layout** below in place of **### Slash commands** and **### Slash command cleanup**, and skip **### Workflow recommendation**. The `govern` self-install, the **Pre-flight Phase**, the **Post-Write Integrity Check**, and **Placeholder Substitution** each carry their own `layout: antigravity` branch in their own sections.

For an agent whose `layout` is **`opencode`**, apply **### OpenCode layout** below in place of **### Slash commands** and **### Slash command cleanup**, and skip **### Workflow recommendation**. OpenCode's installer is a **verbatim markdown file** (no skill wrapper), so the `govern` self-install, **Self-update check**, **Post-Write Integrity Check**, and **Placeholder Substitution** follow the **`claude-style`** path — with the command directory `command/` (singular) and `{cli-config-dir}` resolving to `.opencode`.

### Slash commands (strategy: update)

Fetch each command template and copy it into `{config_dir}/commands/{project}/`. In each copied file, replace `{project}` with the user-provided project name and `{cli-config-dir}` with `{config_dir}`.

| Source Path | Destination Path |
| --- | --- |
| `framework/commands/amend.md` | `{config_dir}/commands/{project}/amend.md` |
| `framework/commands/clarify.md` | `{config_dir}/commands/{project}/clarify.md` |
| `framework/commands/groom.md` | `{config_dir}/commands/{project}/groom.md` |
| `framework/commands/help.md` | `{config_dir}/commands/{project}/help.md` |
| `framework/commands/implement.md` | `{config_dir}/commands/{project}/implement.md` |
| `framework/commands/link.md` | `{config_dir}/commands/{project}/link.md` |
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

**Skills (strategy: update).** For each row in the slash-command manifest above — the thirteen `framework/commands/*.md` rows plus the `framework/bootstrap/configure/{key}.md` configure row — transform the source into a dir-form skill at `{config_dir}/skills/{project}-{name}/SKILL.md` (instead of copying to `{config_dir}/commands/{project}/{name}.md`):

1. Read the source markdown (frontmatter + body).
2. Write `{config_dir}/skills/{project}-{name}/SKILL.md` with frontmatter `name: {project}-{name}` and the `description:` carried from the source frontmatter, followed by the source body.
3. Substitute `{project}` and `{cli-config-dir}` in the body exactly as in the `claude-style` copy (`{cli-config-dir}` → `.agents`).

`{name}` is the command's base name (`specify`, `clarify`, …; the configure row's `{name}` is `configure`). The skills are invoked as `/{project}-{name}`.

**Rules (strategy: update).** Mirror each domain rule file the **Shared Files** manifest placed in `specs/rules/` into `{config_dir}/rules/{name}.md`, so Antigravity loads them natively. Both copies regenerate from `framework/rules/` on every `/govern` run — `specs/rules/` stays the pipeline-read location for every agent; `{config_dir}/rules/` is the Antigravity-native mirror. The `specs/rules/` write itself (in **Shared Files**) is layout-independent and unchanged.

**Skill cleanup (replaces Slash command cleanup).** List the skill directories under `{config_dir}/skills/` whose name matches `{project}-*`. Delete any `{config_dir}/skills/{project}-{name}/` whose `{project}-{name}` is not produced by the skills manifest above and is not listed in `.govern.toml` `pinned.files`; report removals and pinned-keeps as for the `claude-style` cleanup. Skill dirs outside the `{project}-*` namespace (and the `govern` skill) are adopter/agent territory and are never touched.

### OpenCode layout (`layout: opencode`)

When the agent's registry `layout` is `opencode`, the two subsections above (**Slash commands**, **Slash command cleanup**) are replaced by the equivalents below. `{config_dir}` resolves to `.opencode`; OpenCode discovers markdown commands under `{config_dir}/command/` (singular), namespaced by subdirectory.

**Commands (strategy: update).** For each row in the slash-command manifest above — the thirteen `framework/commands/*.md` rows plus the `framework/bootstrap/configure/{key}.md` configure row — copy the source **verbatim** (frontmatter + body, no skill transform) to `{config_dir}/command/{project}/{name}.md` (instead of `{config_dir}/commands/{project}/{name}.md`). Substitute `{project}` and `{cli-config-dir}` (→ `.opencode`) in the body exactly as in the `claude-style` copy, and carry the `description` frontmatter as-is. `{name}` is the command's base name (the configure row's `{name}` is `configure`). The commands are invoked `/{project}/{name}` — OpenCode namespaces by subdirectory (verified: `command/gov/specify.md` registers as command key `gov/specify`).

**Command cleanup (replaces Slash command cleanup).** List the `.md` files under `{config_dir}/command/{project}/`. Delete any whose base name is not produced by the manifest above and is not listed in `.govern.toml` `pinned.files`; report removals and pinned-keeps as for the `claude-style` cleanup. Files outside the `{project}/` subdirectory are adopter/agent territory and are never touched.

**Rules.** OpenCode reads `AGENTS.md` natively (via its `instructions` resolution) and the pipeline reads the shared `specs/rules/` directly — there is **no** native rules-dir mirror (unlike `antigravity`). Nothing extra to scaffold.

**MCP + permissions.** Both the `gvrn` `mcp` block and the `permission` set live in the committed root `opencode.json` — seeded by §Permission Setup, wired by §gvrn runtime detection (State-B `write-file`), and completed by `/{project}:configure`. See §Derived values and §MCP registration.

### Workflow recommendation (strategy: create per accepted workflow)

**Skip this entire section when the agent's `layout` is `antigravity` or `opencode`** — workflow scaffolding is deferred for those layouts (the tech-stack-gated workflow commands are not yet adapted); the pipeline commands above are the adoption surface. For `claude-style` agents, proceed as below.

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

The session state file lives at `.govern.session.toml` at the repo root — a single uniform path for every adopter, project-name-agnostic, gitignored, and **per-contributor**. It carries two things: the session target (feature, optional scenario, `set-at`), written on each `/{project}:target` (or its scenario sibling) invocation; and the contributor's `cli-config-dir`, written by `/govern` at adoption (§Instructions step 6) — the one place an agent-specific value belongs, since teammates on one project may use different agents. Both are written by the runtime's `write-session` primitive (a target write preserves `cli-config-dir`; a host-config write preserves the target), or on the markdown-only path by the host's file-writing tool. There is no per-agent session state beyond this one file.

### `govern` self-installation (strategy: update)

Fetch `framework/bootstrap/govern.md` and write it to the agent's `govern` install path: `{config_dir}/commands/govern.md` for `claude-style`, `{config_dir}/command/govern.md` for `opencode`, or `{config_dir}/skills/govern/SKILL.md` for `antigravity`. This is the same unified file the user is currently running, installed into every selected agent so the command is invokable from that agent on subsequent runs. For `antigravity`, wrap the body in `name: govern` frontmatter (the dir-form skill); for `claude-style` and `opencode` the file is the verbatim `govern.md`. The body keeps every placeholder literal (next paragraph).

In this file (and only this file), keep **every** placeholder literal — do **not** substitute anything. `{project}` and `{cli-config-dir}` must stay literal so `govern` itself can read `$ARGUMENTS` and the per-agent config dir on each run; `{project-name}` and `{One-line project description.}` must stay literal because this file's prose *documents* those placeholders for the AGENTS.md template — substituting them would corrupt the documentation, not personalize a value.

After writing, run the **Post-Write Integrity Check** below.

## Hook Installation

After **Per-Agent Scaffolding** completes, manage the project's git pre-commit hook so generated artifacts (currently spec `dependencies:` and `references:` frontmatter, future generators if added) stay in sync on every commit.

Two files participate, with different ownership models:

- **`.githooks/govern-pre-commit`** is govern-owned. Placed by the **Shared Files** manifest with `update` strategy; carries the `# managed-by: govern` sentinel on line 2; rewritten on every `/govern` run unless pinned in `.govern.toml`. Holds the generator orchestration (currently `scripts/gen-spec-deps.sh --staged` and `scripts/gen-cross-service-refs.sh --staged` plus output staging). Both run with `--staged` so a commit only rewrites the specs it touches, never unrelated ones.
- **`.githooks/pre-commit`** is adopter-owned. Placed by the manifest with `create` strategy on first install; never overwritten thereafter. Initial content invokes `./.githooks/govern-pre-commit`; adopters add their own pre-commit checks above or below that invocation.

This section's job is to wire git up to actually run the outer hook (`git config core.hooksPath .githooks`) without clobbering whatever hook system the project already uses.

Detection runs in this order — first match wins:

1. **`core.hooksPath` already points at `.githooks`** — already wired up. The manifest passes have already written `.githooks/govern-pre-commit` (`update`) and, on first run, `.githooks/pre-commit` (`create`). Run `chmod +x .githooks/pre-commit .githooks/govern-pre-commit` to ensure both files are executable. Report `pre-commit hook already wired up`.
2. **`core.hooksPath` points at any other path** — the project uses a custom hooks dir. Skip wiring; report a warning with the manual integration snippet below.
3. **A third-party hook system is detected** — any of `.husky/`, `.pre-commit-config.yaml`, `lefthook.yml`, or `lefthook-local.yml` exists. Skip wiring; report a warning with the manual integration snippet below.
4. **No conflicts** — run `git config core.hooksPath .githooks` and `chmod +x .githooks/pre-commit .githooks/govern-pre-commit`. Report `pre-commit hook installed`.

The detection ladder no longer treats `.githooks/pre-commit` itself as a govern-managed file — under the new model the outer file is adopter-owned, so its presence is not a signal that govern installed it. Migration of pre-existing govern-installed hooks (from spec-017 adopters) is handled by the **Migration from spec-017 hook** subsection below, which runs before the detection ladder.

`scripts/gen-spec-deps.sh` and `scripts/gen-cross-service-refs.sh` ship in the **Shared Files** manifest with `update` strategy. Every `/govern` run refreshes them from upstream so adopters pick up generator fixes automatically. Adopters who have customized a script can list it in `.govern.toml` `pinned.files` to opt out of overwrites.

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

In every copied file (except each selected agent's installed `govern` file — `{config_dir}/commands/govern.md` for `claude-style`, `{config_dir}/command/govern.md` for `opencode`, `{config_dir}/skills/govern/SKILL.md` for `antigravity` — whose body keeps `{project}` and `{cli-config-dir}` as literal placeholders), replace:

- `{project}` with the user-provided project name (used in commands, README)
- `{project-name}` with the user-provided project name (used in AGENTS.md template)
- `{One-line project description.}` with the user-provided description
- `{cli-config-dir}` with the agent's `config_dir`

## Post-Write Integrity Check

After writing the agent's installed `govern` file — whether via the **Pre-flight Phase** (stale-write path) or the **`govern` self-installation** manifest step — verify it is well-formed. For `claude-style` (`{config_dir}/commands/govern.md`) and `opencode` (`{config_dir}/command/govern.md`), the file must start with `# govern`. For `antigravity` (`{config_dir}/skills/govern/SKILL.md`), the file must start with a frontmatter block whose `name:` is `govern`, and the body after that frontmatter must start with `# govern`. If the check fails, the write was corrupted — report the error and re-read the source: `{tempdir}/govern.md.upstream` for the self-update path, or `{tempdir}/govern-main/framework/bootstrap/govern.md` for the manifest path. Apply the check independently per agent.

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
- **Pinned `govern.md` in `.govern.toml`** — the manifest's `update` strategy still skips the file (no overwrite), and the **Pre-flight Phase**'s self-update check never writes pinned files even on the stale-detect path. The check byte-compares anyway: matching upstream → recorded as `current`, no output; divergent from upstream → recorded as `pinned-divergent`, the run continues, and a single advisory line is printed in the post-scaffolding output. A pinned `govern.md` will not pick up upstream changes until the pin is removed, but the user is told once when the pin is currently suppressing real divergence.
- **Self-update check sees a stale `govern` in an unselected adopted agent** — the check is scoped to selected agents only. The unselected agent's stale copy is not diffed, not written, and does not trigger the abort; it will be detected the next time the user runs `/govern` against it.
- **Self-update small fetch fails** — clean abort with the error message defined in **Pre-flight Phase → Self-update check → Small fetch**. No `govern.md` writes occur, and the archive fetch is skipped. The user re-runs after the transient failure clears.
- **Archive fetch or extract fails** — clean abort with the error message defined in **File Fetching → Archive fetch and extract**. The pre-flight phase has already passed by this point, so no additional `govern.md` or gvrn-wiring writes are pending; the user re-runs after the transient failure clears.
- **A required source file is absent from the extracted archive** — warn `Source not found in archive: {source-path}; skipping.` and continue with the remaining manifest entries. Preserves the per-entry "do not abort on a single fetch error" guarantee at the entry level even though the archive itself is fetched once.
- **First-run prompt with no detected dirs and only one supported agent** — the prompt still appears (the agent must be explicitly chosen), but the single agent is pre-selected. Confirming is one keystroke.
- **Running `govern.md` cannot infer its own install path** — fall back to no pre-selection in the first-run prompt. The user picks explicitly.
- **gvrn binary present but unwired (State B)** — the **Pre-flight Phase** registers the runtime per the agent's `mechanism` (writes the MCP config for a `write-file` agent, or surfaces the registration command for a `surface-instruction` agent) plus the gvrn tool permissions, then stops as part of the single combined pre-flight abort. No archive is fetched; the user starts a new session and re-runs. See **gvrn runtime detection → State B**.
- **gvrn wiring file is malformed JSON** — the wiring write does not touch the file. `/govern` skips wiring, warns the user to repair it, and continues on the markdown path for this run (treated as State C). A hand-maintained MCP config is never clobbered.
- **gvrn binary probe cannot run or is denied** — the run is classified as State C (binary absent): the markdown path proceeds and the post-scaffolding tip fires. Detection never hard-fails on a host without shell.
- **Stale `govern.md` on an adopter who has never wired gvrn** — both pre-flight checks contribute writes (a fresh `govern.md` and the gvrn wiring), but the **Pre-flight abort** emits one combined message and the user restarts once, not twice.

## Post-Scaffolding Output

After scaffolding, display:

- Summary of files created, updated, unchanged, skipped, pinned, merged, and removed — grouped by agent for per-agent files, with shared files in their own group
- For each scaffolded agent, the agent's `rules_file_note` from the registry
- Hook installation status — one line: `pre-commit hook installed`, `pre-commit hook already wired up`, or `pre-commit hook skipped — existing {husky|lefthook|pre-commit-py|core.hooksPath} detected; see manual integration snippet above`. When the spec-017 → spec-018 migration ran, append the migration summary line described in §Hook Installation > Migration from spec-017 hook (or the relevant recovery-branch warning if the rename was skipped or failed).
- Any fetch failures encountered
- Pinned `govern.md` advisory (if applicable — see below)
- Security audit summary (if applicable — see below)
- gvrn runtime tip (State C only — see below)
- Next steps (varies by mode):

### gvrn runtime tip

When the **Pre-flight Phase** resolved to **State C** (no `gvrn` binary detected), append one line after the file summary:

> Tip: this run used the markdown path. Installing the `gvrn` runtime makes `/govern` and the pipeline commands much cheaper in tokens — see [Runtime](https://github.com/stonean/govern#runtime). Once it's on your `PATH`, `/govern` wires it in automatically.

Omit the tip in **State A** (the runtime is already live) and **State B** (the run aborted in pre-flight before this output). State B's file disclosure rides the **Pre-flight abort** message, not this output.

### Pinned `govern.md` advisory

If the **Pre-flight Phase** recorded any selected agent as `pinned-divergent` (the installed `govern` file (`{config_dir}/commands/govern.md`, or `{config_dir}/skills/govern/SKILL.md` for `antigravity`) is listed in `.govern.toml` `pinned.files` and differs from upstream), append one advisory line per divergent agent after the file summary and before next steps:

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

Create intermediate directories as needed (e.g., `specs/`, `specs/templates/`, and — by layout — `{config_dir}/commands/{project}/` for `claude-style`, `{config_dir}/command/{project}/` for `opencode`, or `{config_dir}/skills/` and `{config_dir}/rules/` for `antigravity`).
