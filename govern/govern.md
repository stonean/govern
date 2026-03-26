# Govern — Claude Code

Bootstrap governance in an existing project. This command fetches templates from the governance repo, scaffolds governance files, resolves placeholders, and displays next steps.

## Inputs

Collect from `$ARGUMENTS` or prompt the user interactively. When using AskUserQuestion, every question **must** include an `options` array with 2–4 example choices (the user can always select "Other" for custom input):

1. **Project name** — lowercase, alphanumeric, hyphens allowed. Used for `{project}` placeholder substitution and command directory naming. If `$ARGUMENTS` contains a single word, use it as the project name and prompt for the remaining inputs. Example options: the current directory name, `my-service`.
2. **Project description** — one-line description for AGENTS.md. Example options: `A new microservice`, `CLI tool for X`.
3. **Primary language(s)** — comma-separated list for .gitignore language patterns. Example options: `Go`, `Python`, `Node`, `Go, Python`.

Validate the project name: must be lowercase, alphanumeric, and hyphens only. If invalid, reject with: "Project name must be lowercase, alphanumeric, and hyphens only."

## Pre-flight Checks

Before scaffolding, verify:

- The current directory **is** an existing git repository. If not, stop and report: "This is not a git repository. Run `git init` first."
- If a `specs/` directory already exists, this is a re-run. Report: "Existing specs/ directory found — running in update mode." Proceed normally; `update` strategy files will be overwritten, `create` strategy files will be skipped, `skip` strategy files will be left alone.

## Permission Setup

Before fetching any files, read `.claude/settings.local.json` (create it if missing, with `{"permissions":{"allow":[],"deny":[]}}`). Ensure `permissions.allow` contains `Bash(curl *)` and `Bash(ls *)`. If missing, add them and write the file. This prevents repeated permission prompts during the fetch and scaffolding phases.

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

## Migration: triage → inbox

Before processing the file manifest, check for artifacts from the pre-rename `triage` naming:

1. If `specs/triage.md` exists and `specs/inbox.md` does not — rename `specs/triage.md` to `specs/inbox.md`.
2. If `specs/triage.md` exists and `specs/inbox.md` also exists — merge items from `triage.md` into `inbox.md`, then delete `triage.md`.
3. If `.claude/commands/{project}/triage.md` exists — delete it.

Report any migration actions in the post-scaffolding summary.

## File Manifest

Fetch each file from the governance repo and copy it to the destination path. The source URL pattern is:

```text
https://raw.githubusercontent.com/stonean/govern/main/{source-path}
```

If a fetch fails, report the failure and continue with remaining files. Do not abort on a single fetch error.

For `update` strategy files, compare fetched content against the existing file. Only overwrite and report as "updated" if the content differs. If the content is identical, report as "unchanged" (or omit from the summary).

### Governance-owned files (strategy: update)

These files are owned by governance and always overwritten with the latest version on re-run.

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
| `govern/govern.md` | `.claude/commands/govern.md` |

### Project-specific files (strategy: create)

These files are filled in by the user with project-specific content. Created on first run, skipped on re-run.

| Source Path | Destination Path |
| --- | --- |
| `templates/system.md` | `specs/system.md` |
| `templates/errors.md` | `specs/errors.md` |
| `templates/events.md` | `specs/events.md` |
| `templates/inbox.md` | `specs/inbox.md` |
| `templates/initialize.md` | `.claude/commands/{project}/initialize.md` |

### Slash commands (strategy: update)

Fetch each command template and copy it into `.claude/commands/{project}/`. In each copied file, replace `{project}` with the user-provided project name and `{cli-config-dir}` with `.claude`.

| Source Path | Destination Path |
| --- | --- |
| `commands/about.md` | `.claude/commands/{project}/about.md` |
| `commands/clarify.md` | `.claude/commands/{project}/clarify.md` |
| `commands/implement.md` | `.claude/commands/{project}/implement.md` |
| `commands/plan.md` | `.claude/commands/{project}/plan.md` |
| `commands/question.md` | `.claude/commands/{project}/question.md` |
| `commands/scenario.md` | `.claude/commands/{project}/scenario.md` |
| `commands/setup.md` | `.claude/commands/{project}/setup.md` |
| `commands/specify.md` | `.claude/commands/{project}/specify.md` |
| `commands/status.md` | `.claude/commands/{project}/status.md` |
| `commands/target.md` | `.claude/commands/{project}/target.md` |
| `commands/inbox.md` | `.claude/commands/{project}/inbox.md` |
| `commands/validate.md` | `.claude/commands/{project}/validate.md` |
| `commands/capture.md` | `.claude/commands/{project}/capture.md` |
| `commands/create.md` | `.claude/commands/{project}/create.md` |

### Slash command cleanup

After processing the manifest above, list all `.md` files in `.claude/commands/{project}/`. For each file that is **not** in the slash command manifest above **and** is **not** listed in `.governance.toml` `pinned.files`:

- Delete the file.
- Report it as "removed" in the post-scaffolding summary.

Files listed in `pinned.files` are never deleted — report them as "pinned (kept)" instead. The `initialize.md` command is project-specific (strategy: create) and must not be deleted.

### Session state (strategy: create)

Create `.claude/{project}-session.json` with empty content `{}` only if it does not already exist.

### Files with conflict handling

**AGENTS.md** (strategy: skip) — if it exists, leave it alone. If not, fetch `AGENTS.md` from the governance repo root and substitute `{project-name}` with the project name and `{One-line project description.}` with the project description.

**CLAUDE.md** (strategy: skip) — if it exists, leave it alone. If not, fetch `templates/claude-md.md` from the governance repo and copy it as `CLAUDE.md`.

**.gitignore** (strategy: merge) — if it exists, check for a `# Governance` comment header. If the header exists, skip (already merged). If no header, append governance patterns below existing content:

1. Fetch `templates/gitignore` from the governance repo.
2. Append its content below a `# Governance` comment header.
3. For each primary language provided by the user, fetch from `https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore` and append below a `# {Language}` comment header.

If .gitignore does not exist, create it from `templates/gitignore` plus language patterns.

## Placeholder Substitution

In every copied file (except `.claude/commands/govern.md` — the govern command must keep `{project}` and `{cli-config-dir}` as literal placeholders), replace:

- `{project}` with the user-provided project name (used in commands, README)
- `{project-name}` with the user-provided project name (used in AGENTS.md template)
- `{One-line project description.}` with the user-provided description
- `{cli-config-dir}` with `.claude`

## Post-Write Integrity Check

After writing `.claude/commands/govern.md`, verify the file starts with `# Govern`. If it does not, the write was corrupted — report the error and re-fetch the file.

## What This Command Does NOT Do

- Modify `README.md` — the project's README is its own
- Create feature specs — the user does that via `/{project}:specify`
- Fill in AGENTS.md content — that requires project-specific knowledge
- Fill in system.md content — that requires architectural decisions
- Make git commits — the user decides when to commit
- Run `/{project}:setup` — that happens after adoption, interactively

## Post-Scaffolding Output

After scaffolding, display:

- Summary of files created, updated, unchanged, skipped, pinned, and merged
- Any fetch failures encountered
- Self-update notice (if applicable — see below)
- Next steps (varies by mode):

### Self-update notice

If `.claude/commands/govern.md` was reported as "updated" (i.e., the fetched version differs from the previously installed version), append this notice after the file summary and before next steps:

> **The govern command itself was updated.** Start a new session and re-run `/govern` to apply the latest changes.

This notice is not shown on first run (the file is new, not updated) or when the govern command was unchanged.

### First run (no existing `specs/` directory)

---

**Governance adopted successfully.**

Next steps:

1. Run `/{project}:setup` to configure permissions
2. Fill in `AGENTS.md` — tech stack, project structure, code style, testing conventions, gotchas
3. Fill in `specs/system.md` — architecture, request lifecycle, shared infrastructure
4. Populate `specs/inbox.md` with known issues and bugs
5. Run `/{project}:inbox` to migrate items to specs and scenarios
6. Create your first feature spec: `/{project}:specify {feature description}`

---

### Update mode (existing `specs/` directory detected)

---

**Governance updated successfully.**

Review changes to updated files and commit when ready.

---

## Idempotency

This command is safe to run again. Files with `update` strategy are always overwritten with the latest governance version — unless pinned in `.governance.toml`, in which case they are skipped. Files with `create` strategy skip existing files. The `.gitignore` merge checks for the `# Governance` marker before appending. `skip` strategy files are never overwritten.

## Directory Creation

Create intermediate directories as needed (e.g., `specs/`, `specs/templates/`, `.claude/commands/{project}/`).
