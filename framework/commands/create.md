# Create

Scaffold a new project from this one. Copies specs, commands, configuration, and — if present — implementation code to a new directory with the new project name.

## Purpose

A spec project defines the tech stack, architecture, and foundational features for a product. When the spec project also contains implementation code, the created project inherits a working foundation and can jump straight to business-specific features. It is the greenfield counterpart to `govern`.

## Scope Boundaries

- This command copies files and renames references. Do NOT generate new code, specs, or plans.
- Read only what is needed: the source project's file structure, `AGENTS.md` (for display name detection), and `initialize.md` (if present, for additional inputs and copy guidance). Do NOT read spec contents, source code logic, or plan details.
- All operations target the new project directory. Do NOT modify the source project.

## Inputs

Collect from `$ARGUMENTS` or prompt the user interactively. When using AskUserQuestion, every question **must** include an `options` array with 2-4 example choices (the user can always select "Other" for custom input):

1. **Project slug** — lowercase, alphanumeric, hyphens allowed. Used for directory name, command prefix, and code references. If `$ARGUMENTS` contains a single word, use it as the slug and prompt for the remaining inputs. Example options: `myapp`, `my-service`.
2. **Project display name** — human-readable name used in README headings, AGENTS.md title, and documentation. Example options: `My App`, `My Service`.
3. **Git remote URL** — the remote repository URL. Used to set `git remote add origin` and passed to the initialize command if present. Example options: `https://github.com/stonean/{slug}`.
4. **Project path** — where to create the project directory. Defaults to a sibling of the current project (i.e., `../{slug}` relative to the current project root). Example options: the computed default path, `~/src/{slug}`.

If an initialize command exists at `.claude/commands/{source}/initialize.md`, read it before collecting inputs — it may define additional inputs to collect.

Validate the project slug: must be lowercase, alphanumeric, and hyphens only. If invalid, reject with: "Project slug must be lowercase, alphanumeric, and hyphens only."

## Pre-flight Check

Before scaffolding, verify:

- The target directory (`{path}/{slug}`) does **not** already exist.
- If it exists, **stop immediately** and report: "Directory already exists at {path}/{slug}. Choose a different name or remove the existing directory."

## Scaffolding Steps

Perform all steps in order.

### 1. Create project directory and initialize git

```bash
mkdir -p {path}/{slug}
cd {path}/{slug}
git init
git remote add origin {git-remote-url}
```

### 2. Copy project files

Copy these files from the current project root into the new project root:

- `constitution.md`
- `.markdownlint-cli2.jsonc`
- `.gitignore`
- `AGENTS.md`
- `CLAUDE.md`
- `README.md`

### 3. Copy specs directory

Copy the entire `specs/` directory (including all feature specs, templates, system.md, errors.md, events.md, and any scenarios) into the new project.

### 4. Copy implementation files

Copy all implementation directories and files that exist in the source project. Skip any that do not exist — a spec-only project will have none of these.

Copy any directories and files at the project root that are clearly part of the application. Do **not** copy `.git/`, `.claude/`, or IDE-specific settings. The initialize command (if present) may list specific directories and files to copy — follow its guidance.

### 5. Copy slash commands

Create `.claude/commands/{slug}/` and copy every `.md` file from `.claude/commands/{source}/` in the current project into `.claude/commands/{slug}/`.

### 6. Copy govern command

If `.claude/commands/govern.md` exists in the current project, copy it to `.claude/commands/govern.md` in the new project.

### 7. Create session file

Create `.claude/{slug}-session.json` with empty content `{}`.

### 8. Create settings

Create `.claude/settings.local.json` with default content `{"permissions":{"allow":[],"deny":[]}}`. Do **not** copy from the source project — it contains absolute paths specific to the source.

### 9. Rename project references

The source project name is derived from the command prefix (e.g., if this command is `/{source}:create` then the source name is `{source}`).

Replace the source project name with the new slug **in all case variants**, and replace the source display name with the new display name:

| Source form | Replacement form | Example |
| --- | --- | --- |
| `{source}` (lowercase) | `{slug}` (lowercase) | `anvil` → `myapp` |
| `{Source}` (capitalized) | `{Slug}` (capitalized) | `Anvil` → `Myapp` |
| `{SOURCE}` (uppercase) | `{SLUG}` (uppercase) | `ANVIL` → `MYAPP` |

Additionally, replace the source project's display name (found in `AGENTS.md` heading, e.g., `# Project: Anvil`) with the new display name (e.g., `# Project: My App`).

**Files to scan and replace in:**

- `AGENTS.md`
- `README.md`
- `CLAUDE.md`
- All files in `.claude/commands/{slug}/`
- All `.md` files under `specs/` (including `specs/templates/`)

Do **not** replace inside:

- `constitution.md` (governance-owned, no project references)
- `.gitignore` and `.markdownlint-cli2.jsonc` (no project name references)
- Implementation files (handled by the initialize command in step 10, if present)

### 10. Run initialize command (if present)

If `.claude/commands/{slug}/initialize.md` exists in the new project directory (copied in step 5), read it and execute its instructions. The initialize command receives all collected inputs (slug, display name, git remote URL, path, and any additional inputs it defined).

The initialize command handles all language-specific and project-specific post-copy work, such as:

- Renaming module paths and import statements in source code
- Updating build configuration files with the new project name
- Regenerating lock files or checksums
- Any other transformations specific to the project's tech stack

If no initialize command exists, skip this step.

### 11. Run markdownlint

Run `npx markdownlint-cli2` on all generated `.md` files in the new project directory. Fix any issues found.

### 12. Display next steps

After scaffolding is complete, display:

---

**Project created successfully at `{path}/{slug}`.**

Created from `{source}`.

Next steps:

1. Start a new Claude Code session in the project directory: `cd {path}/{slug}`
2. Run `/{slug}:setup` to configure permissions
3. Review `AGENTS.md` and update any project-specific details
4. Run `/{slug}:status` to see all features and their progress
5. Add your first business feature: `/{slug}:specify`

---

## What This Command Does NOT Do

- Generate code — it copies the source project as-is, including any implementation
- Make any git commits — the user decides when to commit
- Run `/{slug}:setup` — that runs in the new project's Claude session
- Remove or modify the source project in any way
