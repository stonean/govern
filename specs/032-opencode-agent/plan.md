# 032 — OpenCode Agent Support Plan

Implements [032 — OpenCode Agent Support](spec.md).

## Overview

Add OpenCode as a fourth file-scaffold agent by introducing a third **`layout`
profile** (`opencode`) to the agent registry generalized in
[028](../028-antigravity-agent/spec.md), and a fourth per-agent MCP descriptor
([031](../031-agent-mcp-wiring/spec.md)). OpenCode sits between the two existing
profiles: its pipeline commands are **verbatim namespaced markdown files** (like
`claude-style`, not transformed skills like `antigravity`), so the self-update
and integrity branches are claude-style-like; but its config model is unique — a
single **committed root `opencode.json`** carries *both* the `mcp` server wiring
and the `permission` set as two govern-owned regions, where Claude splits them
across `.mcp.json` + `settings.local.json`. The work is almost entirely in the
markdown bootstrap plus one generator splice target, one new configure source, an
`install.sh` arm, and README — **no runtime (Rust) change ships** (the State-B MCP
write and the configure merge are host-side by definition; see Decisions 4, 5, 10).

All facts here are verified against the live `opencode 1.17.8` CLI during
`/specify` and `/clarify` (see spec Resolved Questions for provenance).

## Technical Decisions

### 1. Third `layout` profile: `opencode`

The registry gains a third `layout` value, `opencode`, alongside `claude-style`
and `antigravity`. Claude / Auggie / Antigravity rows are unchanged. OpenCode
registry row:

| field | value |
| --- | --- |
| `key` | `opencode` |
| `name` | `OpenCode` |
| `config_dir` | `.opencode` |
| `layout` | `opencode` |
| `settings_template` | `{ "$schema": "https://opencode.ai/config.json", "permission": { "bash": { "curl *": "allow", "ls *": "allow", "tar *": "allow", "mktemp *": "allow", "git status *": "allow", "git config *": "allow", "git rev-parse *": "allow", "git diff *": "allow", "git ls-files *": "allow", "chmod *": "allow", "awk *": "allow", "command -v *": "allow" } } }` (bootstrap-only seed, written into the **root `opencode.json`**) |
| `rules_file_note` | `OpenCode reads AGENTS.md natively — no second rules file.` |

§Derived values gains an `opencode` column: command path
`.opencode/command/{project}/<name>.md`; invocation `/{project}/<name>`; `govern`
install path `.opencode/command/govern.md`; settings file **root `opencode.json`**;
permission shape OpenCode's `permission` action map; native rule-loading dir — (none;
rules read from shared `specs/rules/` as in `claude-style`); native rules file
`AGENTS.md`; slash-command cleanup glob the `{project}/` subdirectory under
`command/`. §"Adding a new agent" is updated to record `opencode` as the third
layout. Detection/§Agent Selection are unchanged — they key on `config_dir`
(`.opencode/`) like every other agent.

### 2. Command scaffolding — verbatim namespaced markdown (claude-style-like)

For `layout: opencode`, §Per-Agent Scaffolding copies each
`framework/commands/<name>.md` to `.opencode/command/{project}/<name>.md`
verbatim (carry frontmatter `description`, keep the body and approval-gate
prompts, substitute `{project}` and `{cli-config-dir}` → `.opencode`, preserve
`$ARGUMENTS`). This is the `claude-style` copy with two differences: the
destination dir is `command/` (singular) under a `{project}/` subdirectory, and
invocation is `/{project}/<name>` (path namespace, verified: `command/gov/specify.md`
→ key `gov/specify`). The configure row maps to
`.opencode/command/{project}/configure.md` as for the others. **No skill
transform** — unlike `antigravity`, OpenCode reads markdown command files
directly, so the body is the procedure as-is. Slash-command cleanup prunes
`.md` files under `.opencode/command/{project}/` not in the manifest (the same
cleanup as `claude-style`, scoped to the `{project}/` subdir).

### 3. Single committed root `opencode.json` — two govern-owned regions

The decisive divergence. OpenCode reads a project-root `opencode.json`
(deep-merged over `~/.config/opencode/`), and that one file carries both the
`mcp` server map and the `permission` set. So OpenCode's **settings file and
MCP-wiring file are the same target**, and govern owns exactly two regions of it:

- `mcp.gvrn` — `{ "type": "local", "command": ["gvrn", "mcp"], "enabled": true }`
- `permission` — the bootstrap shell allows plus `"gvrn*": "allow"`

Every write into `opencode.json` is an **additive JSON-object merge** that
preserves `$schema` and all other keys (Decision 5). If the adopter keeps config
in root `opencode.jsonc`, merge into that file instead of creating a second one
(detect existing config file; default to `opencode.json` when neither exists).
The file is **committed** (team-shared wiring); `.opencode/` — the regenerated
command tree — is gitignored (Decision 7), mirroring Claude's committed root
`.mcp.json` beside a gitignored `.claude/`.

### 4. MCP wiring — State-B write-file, OpenCode shape

OpenCode's MCP descriptor is `target`: root `opencode.json` `mcp` block;
`scope`: `project-committed`; `mechanism`: `write-file` — the Claude posture, **no
surfaced instruction** (verified: gvrn loads from the committed file, `opencode mcp
list` → `✓ gvrn connected`). The §MCP wiring `write-file` branch currently writes
Claude's `.mcp.json` shape (`mcpServers` map, `{command, args}`); it gains an
`opencode` sub-case writing the **`mcp` key** with OpenCode's server shape
(`{type, command:[…], enabled}`) into root `opencode.json`, additively (same
five cases: missing file, has-key-no-gvrn, already-present no-op, no-key, invalid
JSON → skip). The State-B auto-wire permission grant (§gvrn runtime auto-wiring)
adds `"gvrn*": "allow"` to `opencode.json` `permission` for OpenCode (alongside
Claude's `mcp__gvrn__*`, Antigravity's `mcp(gvrn/*)`, Auggie's `mcp:gvrn:*`). Per
the existing rule, this State-B write is host-side — there is no runtime primitive
(State B is the runtime-absent case by definition).

### 5. Configure — fourth permission format, prose-walk generic JSON merge

New `framework/bootstrap/configure/opencode.md` writes the root `opencode.json`
`permission` block in OpenCode's action map (`allow` / `ask` / `deny`,
per-tool string or `{ pattern: action }`, last-match-wins). Canonical mapping:

- gvrn runtime → a single `"gvrn*": "allow"` (one glob, like Antigravity's
  `mcp(gvrn/*)`; verified accepted, no `mcp` permission key exists). Ordered
  **after** any broad `"*"` rule so it is not shadowed.
- `edit` → `"allow"` (govern edits specs); `bash` → `{ "<allow patterns>":
  "allow", "rm -rf *": "deny", "*": "ask" }`; `webfetch` / `websearch` as needed.
- exact allow/deny patterns finalized at implement against the published schema.

Per spec Resolved Question 5 (option A), the merge is a **generic additive
JSON-object merge** over the `permission` (and `mcp`) regions — *not* an
extension of `merge-permissions` to a fourth grammar (OpenCode's `permission` is
plain `{ tool → action }` JSON needing key-preserving merge, not allow/deny
reconciliation). `configure/opencode.md` therefore **walks the prose** host-side
(like `antigravity.md`), preserving `$schema` and adopter keys; a generic
JSON-region merge primitive is left as a 022 follow-up (runtime-eligibility
analysis), not built here.

### 6. `gen-configure-mcp.sh` — fourth splice target

`scripts/gen-configure-mcp.sh` gains `opencode.md` as a fourth splice target.
Like Antigravity's constant `mcp(gvrn/*)` line, OpenCode uses a single
`"gvrn*": "allow"` glob (not per-tool enumeration), so its block is a constant
single line built outside the per-tool loop and spliced between
`<!-- generated:mcp-allow:start/end -->` markers added to `configure/opencode.md`.
This keeps the cross-agent in-sync invariant (drift fails the pre-commit hook).

### 7. gitignore `.opencode/`, not root `opencode.json`

Add `.opencode/` to the framework-managed `.gitignore` block (the
`merge-managed-block` content referenced in govern.md and
`framework/templates/project/gitignore.md`), parallel to `.claude/` / `.augment/`
/ `.agents/`. The **root `opencode.json` is deliberately not gitignored** — it
carries the committed, team-shared gvrn wiring (Decision 3). Like the Claude
`commands` carve-out, ensure the regenerated command tree is ignored while the
committed config stays tracked.

### 8. `install.sh` — `opencode` arm

Add an `opencode)` case: `dest=".opencode/command/govern.md"`,
`mkdir -p .opencode/command`, and seed the root `opencode.json` `permission`
(only if absent) from the `opencode` `settings_template`. Update the usage
comment and the unknown-agent error to include `opencode`. The installed
`govern` command is a **verbatim** markdown file at `.opencode/command/govern.md`
(invoked `/govern`), placeholders kept literal — claude-style, not a wrapped skill.

### 9. Bootstrap-wide branches (mostly claude-style-like)

Because OpenCode's `govern` installer is a verbatim markdown file (not a
transformed skill), the bootstrap branches OpenCode needs are simpler than
Antigravity's:

- **§Per-Agent Scaffolding dispatch note** — add the `opencode` branch alongside
  the `antigravity` note (commands → `.opencode/command/{project}/`, no skill
  transform, cleanup scoped to the `{project}/` subdir).
- **govern self-installation / Self-Update Check / Post-Write Integrity Check** —
  claude-style-like: install path `.opencode/command/govern.md`, direct byte
  compare against upstream `govern.md`, `# govern`-first-line integrity check. No
  frontmatter strip (unlike antigravity).
- **§Permission Setup** — seed root `opencode.json` `permission` from the
  `settings_template`; the settings file == the MCP-wiring file for OpenCode.
- **CLAUDE.md shared-file step** — already `claude-style`-only; OpenCode is
  excluded automatically and ships **no CLAUDE.md** (reads AGENTS.md, already
  written for every adoption).
- **Workflow recommendation** — defer for `opencode` (match the antigravity
  deferral; the pipeline commands are the adoption surface). Revisit later.
- **Placeholder Substitution / intermediate-dir creation** — `{cli-config-dir}`
  → `.opencode`; create `.opencode/command/{project}/`.

### 10. No runtime primitive, no migration (scope)

- **No Rust change.** The State-B MCP write (Decision 4) and the configure merge
  (Decision 5) are host-side by the existing runtime boundary. `gvrn`'s MCP tools
  operate on `specs/` independent of agent layout, so the runtime's value is
  available to OpenCode adopters regardless. Extending gvrn `exec`'s `Host`
  command resolution to `.opencode/command/{project}/<name>.md` is a 022 follow-up
  (parallel to the antigravity skill-path follow-up), not gated here.
- **No `framework/migrations.toml` entry.** OpenCode is a brand-new agent — there
  is no pre-existing adopter file to clean up (unlike 031's `.mcp.json` retarget).

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/bootstrap/govern.md` | Modify | `opencode` registry row + §Derived-values column; §MCP-registration row (write-file, project-committed); §MCP-wiring write-file `opencode` sub-case (root `opencode.json` `mcp` shape); §gvrn-auto-wiring `"gvrn*"` grant; §Per-Agent-Scaffolding `opencode` branch (verbatim namespaced commands, cleanup); §Permission-Setup seed; self-install / self-update / integrity / placeholder / dir-creation branches; §"Adding a new agent" note; `.gitignore` block adds `.opencode/` |
| `framework/bootstrap/configure/opencode.md` | Create | OpenCode permission set → root `opencode.json` `permission` (action map; `edit`/`bash` allows, denies, `"gvrn*": "allow"`); prose-walk generic JSON merge; `generated:mcp-allow` markers |
| `scripts/gen-configure-mcp.sh` | Modify | Fourth splice target: emit constant `"gvrn*": "allow"` block into `opencode.md`; keep the cross-agent in-sync invariant |
| `framework/templates/project/gitignore.md` | Modify | Add `.opencode/` to the managed block (root `opencode.json` stays tracked) |
| `install.sh` | Modify | `opencode)` arm: dest `.opencode/command/govern.md`, seed root `opencode.json`, usage + error text |
| `README.md` | Modify | OpenCode install snippet (`sh -s -- opencode` arm); add OpenCode to supported-agents intro + paths summary; Registering-the-runtime note (auto-wired via root `opencode.json`, restart required — config loads once) |
| `scripts/tests/test-gen-configure-mcp.sh` | Modify | Assert the `opencode.md` `"gvrn*"` block is emitted and stays in sync |
| `specs/032-opencode-agent/data-model.md` | Create | The `opencode` layout's derived values + the two owned `opencode.json` regions (merge contract) |
| `specs/028-antigravity-agent/spec.md` | Modify | Signpost note → 032 (registry now carries a third layout, `opencode`); blockquote so `gen-spec-deps` derives no cycle |

(Authoritative write boundary is derived from git history at implement time; this
table is a planning aid. Audit/parity coverage — `scripts/audit/run-all.sh`,
`runtime/tests/` agent enumerations — is checked during Task "Tests" and any
opencode additions folded in.)

## Trade-offs

- **Third layout profile vs. a new abstraction** — a third `layout` value keeps
  Claude/Auggie/Antigravity untouched and isolates OpenCode's divergence; the
  cost is a third branch set in the layout-keyed sections (acceptable; the
  profile model was built for exactly this).
- **One committed `opencode.json` for both regions** — simpler for the adopter
  (their own config file) and enables the fully-automated `write-file` posture,
  but means the permission set is committed/team-shared (vs Claude's per-user
  `settings.local.json`). Acceptable: it is the non-secret bootstrap allow-list,
  and OpenCode's design co-locates the regions.
- **Host-side generic JSON merge vs. a runtime primitive** — host/prose now (Q5
  option A); a generic JSON-region merge primitive is a deferred 022 follow-up,
  not a fourth `merge-permissions` grammar.
- **`write-file` automation asymmetry** — OpenCode (like Claude) gets fully
  automated wiring because its config is committable; Auggie/Antigravity still
  surface an instruction. Inherent, not a regression.
- **Runtime `exec` command resolution deferred** — OpenCode ships on the
  markdown-only path now; gvrn MCP tools are unaffected. Tracked as a 022 follow-up.
- **Workflow-recommendation deferred for `opencode`** — matches the antigravity
  deferral; the pipeline commands are the adoption surface.

## Data Model

See [data-model.md](data-model.md) — the `opencode` layout profile's
derived-value formulas and the two govern-owned `opencode.json` regions (`mcp`,
`permission`) with the additive-merge contract.
