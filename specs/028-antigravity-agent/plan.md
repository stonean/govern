# 028 — Antigravity Agent Support Plan

Implements [028 — Antigravity Agent Support](spec.md).

## Overview

Add Antigravity as a third file-scaffold agent by **generalizing the agent
registry into layout profiles** rather than widening every row. Claude and
Auggie share a `claude-style` profile (commands under
`{config_dir}/commands/{project}/`, `CLAUDE.md`, `.mcp.json`,
`settings.local.json`); Antigravity gets an `antigravity` profile (dir-form
skills under `.agents/skills/`, `AGENTS.md`, `.agents/mcp_config.json`,
`.agents/settings.json`). `framework/bootstrap/govern.md`'s derived values and
§Per-Agent Scaffolding branch on the profile; everything else (the unified
procedure, manifests, session state) is unchanged. The work is almost entirely
in the markdown bootstrap + one generator + one new configure source; no
mandatory runtime (Rust) change ships in this spec (see Technical Decision 6).

## Technical Decisions

### 1. Layout profiles, not per-field registry columns

The registry gains a per-agent **`layout`** discriminator with two values:
`claude-style` (Claude, Auggie) and `antigravity`. The derived-value formulas in
govern.md §Derived values and the branch points in §Per-Agent Scaffolding select
behavior by `layout`. Claude and Auggie are unchanged (`layout: claude-style`;
they already differ only by `config_dir`). Antigravity is one new row.

Rejected: adding five columns (command location, MCP file, settings format,
rules location, rules file) to every row — it bloats the two existing rows with
identical values and scatters the divergence. A profile keeps a new
`.claude`-style agent a true one-row append (`layout: claude-style` + a
`config_dir`) — preserving 012's promise for the common case — while isolating
Antigravity's divergence in one named profile.

Antigravity registry row:

| field | value |
| --- | --- |
| `key` | `antigravity` |
| `name` | `Antigravity` |
| `config_dir` | `.agents` |
| `layout` | `antigravity` |
| `settings_template` | `{ "permissions": { "allow": [ "command(curl)", "command(ls)", "command(tar)", "command(mktemp)", "command(git status)", "command(git config)", "command(chmod)", "command(awk)" ], "deny": [], "ask": [] } }` (bootstrap-only seed) |
| `rules_file_note` | `Antigravity reads AGENTS.md natively — no second rules file.` |

### 2. Skill transform (per-agent scaffolding for `antigravity`)

For `layout: antigravity`, §Per-Agent Scaffolding does not copy command files
verbatim. Each `framework/commands/<name>.md` becomes
`.agents/skills/{project}-<name>/SKILL.md`:

- Frontmatter: set `name: {project}-<name>`, carry the source `description`.
- Body: the command procedure, with `{project}` / `{cli-config-dir}` substituted
  (`{cli-config-dir}` resolves to `.agents`).
- The `govern` installer → `.agents/skills/govern/SKILL.md` (placeholders kept
  literal, per the existing self-install rule).
- Domain rule files → `.agents/rules/<name>.md`.
- Slash-command cleanup prunes stale `.agents/skills/{project}-*/` skill dirs not
  in the manifest (the `claude-style` cleanup prunes `*.md`; the enforced glob is
  profile-derived).

Skill names are flat and project-prefixed (`/{project}-<name>`), since
Antigravity has no colon namespace.

### 3. Configure — third permission format

New `framework/bootstrap/configure/antigravity.md` writes `.agents/settings.json`
`permissions.allow/deny/ask` in Antigravity's action grammar. Canonical-set
mapping:

- gvrn runtime → a single `mcp(gvrn/*)` (replaces Claude's 27 `mcp__gvrn__*`
  lines).
- shell allows → `command(git add)`, `command(curl)`,
  `command(npx markdownlint-cli2)`, …
- denies → `command(rm -rf)`, `command(git push --force)`, …
- `read_file`/`write_file` largely omitted — workspace files are auto-allowed.

`gen-configure-mcp.sh` gains a **third splice target** emitting the Antigravity
MCP block (one `mcp(gvrn/*)` line) into `antigravity.md`, preserving the
pre-commit invariant that runtime-tool permissions stay in sync across all
agents.

### 4. MCP wiring — two files, additive

gvrn for Antigravity = `.agents/mcp_config.json` (server definition) **plus** the
`mcp(gvrn/*)` allow in `.agents/settings.json`. govern.md's MCP-registration step
branches per layout: `.mcp.json` (claude-style) vs `.agents/mcp_config.json`
(antigravity). Both writes are additive — preserve any servers/permissions the
adopter already has.

### 5. gitignore

Add `.agents/` to the framework-managed `.gitignore` block (the
`merge-managed-block` content in govern.md §Shared Files and
`framework/templates/project/gitignore`), parallel to the existing `.claude/`
line — Antigravity's scaffolded tree is adopter-local, gitignored like the
others.

### 6. Runtime `exec` command resolution — scoped out (markdown-only path ships)

gvrn `exec` resolves a command file at `{cli-config-dir}/commands/{project}/
<name>.md` via [022](../022-deterministic-runtime/spec.md)'s `Host`. For
Antigravity the procedure lives at `.agents/skills/{project}-<name>/SKILL.md`, so
`gvrn exec` would not find it without extending `Host` to the skill layout — a
Rust change.

This spec **scopes that out**: (a) Antigravity adoption works on the
markdown-only path immediately — the agent reads `SKILL.md` directly; (b) gvrn's
MCP tools (`read-spec`, `mark-task`, `lint-markdown`, …) operate on `specs/`
independent of agent layout, so the runtime's value-add is available regardless.
Extending `Host` to resolve the skill path is recorded as a follow-up dependent
on 022 (a new scenario on 022 or a successor spec). Rationale: don't gate
Antigravity adoption on a runtime change; ship file-scaffold + MCP tools now.

### 7. Merge ownership — host/markdown now, primitive later

`.agents/mcp_config.json` (JSON-object merge: add `gvrn` if absent) and
`.agents/settings.json` (`permissions` allow/deny/ask merge) are performed
host-side per govern.md prose. `merge-permissions` /`merge-managed-block`
extension to these shapes is a 022 follow-up (parallel to the existing Auggie
`toolPermissions` gap already noted in 022). govern.md documents the additive
merge so the markdown-only path is faithful.

### 8. The `antigravity` layout touches the whole bootstrap, not just scaffolding

Discovered mid-implement (Task 2): the layout branch is not confined to
§Per-Agent Scaffolding. Because the Antigravity `govern` installer is a
*transformed* skill (`.agents/skills/govern/SKILL.md`, frontmatter `name:
govern` plus the body) rather than a verbatim copy of `govern.md`, several other
bootstrap
sections that assume the `claude-style` `commands/govern.md` install must also
branch on `layout`:

- **govern.md Self-Update Check** — byte-compares the installed file against
  upstream; for `antigravity` it must strip the added frontmatter and compare the
  body, and the stale-write path must write the transformed skill, not raw
  `govern.md` (otherwise every run reports stale and installs a broken skill).
- **Post-Write Integrity Check** — asserts a `# govern` first line; for
  `antigravity` it checks the `SKILL.md` frontmatter + body.
- **Placeholder-substitution exception**, **intermediate-dir creation**,
  **`parity.strict-files` frontmatter**, the **CLAUDE.md shared-file step**
  (`claude-style` only), and **Workflow recommendation** (deferred for
  `antigravity`) likewise branch.

All are folded into Task 2; the Self-Update and Integrity branches are required
for Antigravity to bootstrap at all.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/bootstrap/govern.md` | Modify | `layout` profiles + Antigravity registry row; §Derived values + §Per-Agent Scaffolding branch (skill transform, rules, `mcp_config.json`); §Permission Setup branch (`.agents/settings.json`); MCP-registration branch; `.gitignore` block adds `.agents/`; §Agent Selection/detection unchanged (keys on `config_dir`) |
| `framework/bootstrap/configure/antigravity.md` | Create | Antigravity permission set → `.agents/settings.json` (`allow/deny/ask`, action grammar, `mcp(gvrn/*)`) |
| `scripts/gen-configure-mcp.sh` | Modify | Third splice target: emit `mcp(gvrn/*)` Antigravity block; keep the cross-agent in-sync invariant |
| `framework/templates/project/gitignore` | Modify | Add `.agents/` to the managed block |
| `README.md` | Modify | Antigravity curl-bootstrap snippet + add Antigravity to the supported-agents list |
| `specs/012-multi-agent-govern/spec.md` | Modify | Signpost note pointing to 028 (cross-spec impact; mirrors `007 → 012`) |
| `scripts/tests/test-gen-configure-mcp.sh` (or extend) | Create/Modify | Assert the Antigravity `mcp(gvrn/*)` block is emitted and stays in sync |
| `specs/028-antigravity-agent/data-model.md` | Create | Registry schema + Antigravity artifact schemas (this plan) |

## Trade-offs

- **Layout profiles vs per-field columns** — profiles chosen (keeps the common
  case a one-row append; isolates divergence). Cost: a third profile would mean a
  new branch set rather than new column values; acceptable until a fourth layout
  appears.
- **Runtime `exec` deferred** — Antigravity ships on the markdown-only path now;
  adopters with gvrn installed do not get the deterministic command-walk speedup
  for Antigravity until 022's `Host` is extended. gvrn MCP tools are unaffected.
  Limitation documented; tracked as a 022 follow-up.
- **Host-side merge vs runtime primitive** — host/prose now; primitive extension
  deferred to 022 (same posture as Auggie's `toolPermissions`).
- **Project-prefixed flat skill names** (`/{project}-specify`) — avoids
  collisions and matches Antigravity's flat namespace; loses the `:` visual
  grouping of `/{project}:specify`.
- **Global plugin + marketplace deferred** — adoption is workspace `.agents/`
  only; a one-command global install is future work.

## Data Model

See [data-model.md](data-model.md) — the generalized registry schema (with the
`layout` field and profile-derived values) and the Antigravity artifact schemas
(`.agents/skills/<name>/SKILL.md`, `.agents/mcp_config.json`,
`.agents/settings.json`).
