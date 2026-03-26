# 007 — Govern Workflow

**Status:** done
**Dependencies:** 003-bootstrap-automation

> **Note:** This spec was renamed from `007-adopt-workflow` to `007-govern-workflow` by [011-brownfield-process](../011-brownfield-process/spec.md). The govern command also gains a triage → inbox migration step and `/capture` in the command manifest via 011.

A self-contained slash command file that bootstraps governance in existing (brownfield) projects. Users fetch a single `.md` file into their CLI's command directory and run it — no clone of the governance repo required. The command instructs the AI agent to fetch templates from GitHub, write them into the correct locations, perform placeholder substitution, handle conflicts with existing files, and display brownfield-specific next steps.

The command supports multiple AI coding CLIs. Each CLI gets native directory paths and configuration formats — no backward-compatibility shims.

## Distribution Model

The deliverable is one markdown file per supported CLI, hosted in the governance repo. Each file is self-contained and tailored to its target CLI's conventions.

### Supported CLIs

| CLI | Command file | Install location | Invoke |
| ----- | ------------- | --------------- | -------- |
| Claude Code (default) | `govern/govern.md` | `.claude/commands/govern.md` | `/govern {project-name}` |
| Auggie | `govern/govern-auggie.md` | `.augment/commands/govern.md` | `/govern {project-name}` |

Install with one command:

```text
# Claude Code
curl -fsSL https://raw.githubusercontent.com/stonean/govern/main/govern/govern.md \
  > .claude/commands/govern.md

# Auggie
curl -fsSL https://raw.githubusercontent.com/stonean/govern/main/govern/govern-auggie.md \
  > .augment/commands/govern.md
```

No runtime, no dependencies, no build step — the "program" is a prompt. Each variant contains the same governance logic but targets its CLI's native paths and configuration formats.

## Inputs

The command collects from `$ARGUMENTS` or prompts interactively:

1. **Project name** — lowercase, alphanumeric, hyphens allowed. Used for `{project}` placeholder substitution and command directory naming.
2. **Project description** — one-line description for AGENTS.md.
3. **Primary language(s)** — comma-separated list for .gitignore language patterns.

The target CLI is implicit — determined by which `govern.md` variant the user installed. The command file itself knows its target and uses the correct paths throughout.

## Pre-flight Checks

Before scaffolding, verify:

- The current directory **is** an existing git repository.
- A `specs/` directory does **not** already exist (governance not yet adopted). If it does, stop and report: "This project already has a specs/ directory. If you want to re-run adoption, remove it first."

## File Fetching

The command contains a manifest of files to fetch from the governance repo. Each entry specifies:

- Source path (relative to governance repo root)
- Destination path (relative to project root)
- Conflict strategy: `skip` (don't overwrite), `merge` (append), or `create` (must not exist)

Source URL pattern:

```text
https://raw.githubusercontent.com/stonean/govern/main/{source-path}
```

If a fetch fails, report the failure and continue with remaining files. The command must not abort on a single fetch error.

## Scaffolding Behavior

### CLI-agnostic files (strategy: create)

These files are identical regardless of target CLI:

- `constitution.md` — copied as-is from governance root
- `.markdownlint-cli2.jsonc` — copied as-is from governance root
- `specs/system.md` — from `templates/system.md`
- `specs/errors.md` — from `templates/errors.md`
- `specs/events.md` — from `templates/events.md`
- `specs/inbox.md` — from `templates/inbox.md`
- `specs/templates/spec.md` — from `templates/spec.md`
- `specs/templates/spec-and-plan.md` — from `templates/spec-and-plan.md` (if it exists)
- `specs/templates/plan.md` — from `templates/plan.md`
- `specs/templates/tasks.md` — from `templates/tasks.md`
- `specs/templates/data-model.md` — from `templates/data-model.md`
- `specs/templates/research.md` — from `templates/research.md`
- `specs/templates/scenario.md` — from `templates/scenario.md`

### CLI-specific files (strategy: create)

These files use native paths and formats for the target CLI:

| File | Claude Code | Auggie |
| ------ | ------------ | -------- |
| Slash commands | `.claude/commands/{project}/*.md` | `.augment/commands/{project}/*.md` |
| Session state | `.claude/gov-session.json` | `.augment/gov-session.json` |
| Rules file | `CLAUDE.md` (from `templates/claude-md.md`) | `CLAUDE.md` (Auggie reads it natively) |

Slash command templates use a `{cli-config-dir}` placeholder for CLI-specific paths (e.g., session file location). The govern command resolves this placeholder to the target CLI's native directory (`.claude` or `.augment`) during copy.

### Files with conflict handling

- **AGENTS.md** (strategy: skip) — if it exists, leave it alone. If not, copy from governance and substitute project name and description.
- **CLAUDE.md** (strategy: skip) — if it exists, leave it alone. If not, copy from `templates/claude-md.md`. Used by both Claude Code and Auggie.
- **.gitignore** (strategy: merge) — if it exists, append governance patterns and language-specific patterns below existing content, separated by a `# Governance` comment header. If not, create from `templates/gitignore` plus language patterns.

### Placeholder substitution

In every copied file, replace:

- `{project}` and `{project-name}` with the user-provided project name
- `{One-line project description.}` with the user-provided description

### What the command does NOT do

- Modify `README.md` — the project's README is its own; governance doesn't touch it
- Create feature specs — the user does that via `/{project}:specify`
- Fill in AGENTS.md content — that requires project-specific knowledge
- Fill in system.md content — that requires architectural decisions
- Make git commits — the user decides when to commit
- Run `/{project}:setup` — that happens after adoption, interactively

## Post-Scaffolding Output

After scaffolding, display:

- Summary of files created, skipped, and merged
- Any fetch failures encountered
- Brownfield-specific next steps (command names use the project's slash-command prefix):
  1. Run `/{project}:setup` to configure permissions
  2. Fill in `AGENTS.md` — tech stack, project structure, code style, testing conventions, gotchas
  3. Fill in `specs/system.md` — architecture, request lifecycle, shared infrastructure
  4. Populate `specs/inbox.md` with known issues and bugs
  5. Run `/{project}:inbox` to migrate items to specs and scenarios
  6. Create your first feature spec: `/{project}:specify {feature description}`

## Self-Maintenance

The command file remains in the CLI's command directory after execution. It is idempotent — running it again skips already-created files and only attempts files that are missing. The user may delete it after governing if desired.

## Edge Cases

- **No network access** — all fetches fail. The command reports all failures and produces no files. It does not create a partial scaffold from local-only content.
- **Partial previous run** — some files exist from a prior incomplete run. Idempotency handles this: `create` strategy skips existing files, `merge` checks for the `# Governance` marker before appending.
- **`.gitignore` merge dedup** — the command checks for the `# Governance` comment header before appending. If the marker exists, it skips the merge to avoid duplicating patterns.
- **Empty `$ARGUMENTS`** — no project name provided. The command prompts interactively for all required inputs.
- **Invalid project name** — uppercase, spaces, or special characters. The command rejects with a clear error: "Project name must be lowercase, alphanumeric, and hyphens only."
- **Command directory doesn't exist** — the CLI's command directory (e.g., `.claude/commands/`) may not exist yet. The command creates intermediate directories as needed.

## Acceptance Criteria

- [ ] One `govern.md` variant exists per supported CLI in the `govern/` directory
- [ ] Running `curl` followed by `/govern {name}` in an existing git repo produces a complete governance scaffold
- [ ] Each CLI variant scaffolds into its native directory paths (`.claude/` for Claude Code, `.augment/` for Auggie)
- [ ] Existing files (.gitignore, AGENTS.md, CLAUDE.md) are not overwritten
- [ ] `.gitignore` merge is idempotent — running twice does not duplicate governance patterns
- [ ] Fetch failures for individual files do not abort the entire process
- [ ] All generated files pass `markdownlint-cli2`
- [ ] Slash commands are installed in the CLI's native command directory with `{project}` and `{cli-config-dir}` placeholders resolved
- [ ] `specs/inbox.md` is created as the brownfield entry point
- [ ] The command is idempotent — safe to run again without duplicating content
- [ ] Post-scaffolding output displays brownfield-specific next steps
- [ ] Invalid project names are rejected with a clear error message
- [ ] Intermediate directories are created as needed
- [ ] Adding a new CLI requires only a new govern variant file — no changes to governance core

## Open Questions

<!-- All resolved. -->

- ~~Should the command also fetch `sdd-context.md`?~~ No — governance-internal only.
- ~~Should there be a `--dry-run` mode?~~ No — the command is idempotent with create/skip/merge strategies, making dry-run unnecessary.
- ~~How should the file manifest be maintained?~~ Hardcoded in each govern variant. Updated when governance templates change.
- ~~CLI-specific command variants or path variable?~~ Path variable (`{cli-config-dir}`) resolved at govern time. One set of command templates, N govern variants do the substitution.
- ~~How should `/gov:setup` work for Auggie?~~ Skipped for now — Auggie permissions are global. Deferred to future considerations in `specs/spec.md`.
