# 003 — Bootstrap Automation

**Status:** done
**Dependencies:** 000-slash-commands, 001-system-spec-templates, 002-project-scaffolding

Governance slash commands that dogfood the same pipeline commands adopting projects use (`/gov:about`, `/gov:target`, `/gov:status`, `/gov:setup`, `/gov:specify`, `/gov:clarify`, `/gov:plan`, `/gov:implement`, `/gov:validate`, `/gov:next`), plus a governance-specific `/gov:init` that scaffolds new projects from templates.

## Problem

The governance README describes a 7-step manual bootstrap process: init git, copy files, fill in AGENTS.md, create CLAUDE.md, create specs directory, add first spec, follow the pipeline. This is error-prone and tedious. A slash command can automate the mechanical steps while prompting for the decisions that require human input.

Additionally, the governance project itself has no slash commands. Agents working on governance specs operate without pipeline enforcement — no status gates, no dependency checks, no structured workflow. Governance should dogfood the same commands it provides to adopting projects.

## Behavior

### Standard pipeline commands

Copy all ten command templates from `commands/` into `.claude/commands/gov/`, replacing `{project}` with `gov`. This gives governance the same slash commands as any adopting project:

- `/gov:about` — pipeline overview
- `/gov:target` — set session target feature
- `/gov:status` — dashboard of all specs
- `/gov:setup` — configure permissions
- `/gov:specify` — create new feature spec
- `/gov:clarify` — resolve open questions (draft → clarified)
- `/gov:plan` — create plan and tasks (clarified → planned)
- `/gov:implement` — execute tasks (planned → done)
- `/gov:validate` — audit artifacts for consistency
- `/gov:next` — auto-advance to next phase

These commands enforce the same pipeline gates, dependency checks (via AGENTS.md boundaries), and conventions that adopting projects follow.

### /gov:init

### Inputs

The command collects from the user (via `$ARGUMENTS` or interactive prompts):

- **Project name** — used for directory name and placeholder substitution
- **Project path** — where to create the project (defaults to sibling of governance)
- **Project description** — one-line description for README and AGENTS.md
- **Primary language(s)** — used to fetch language-specific .gitignore patterns from github.com/github/gitignore

### Pre-flight Check

Before scaffolding, verify the target directory does not already exist. If it does, stop and report the conflict. Init is for new projects only — adding governance to an existing project should follow the manual bootstrap steps in the README.

### Scaffolding Steps

1. Create the project directory and initialize git
2. Copy governance files: `constitution.md`, `.markdownlint-cli2.jsonc`
3. Copy and customize `AGENTS.md` from template (replace `{project}` placeholder)
4. Create `CLAUDE.md` from template
5. Create `specs/` directory with system spec templates (`system.md`, `errors.md`, `events.md`)
6. Copy spec templates into `specs/templates/`
7. Copy slash command templates into `.claude/commands/{project}/`, replacing `{project}` placeholder
8. Create `.gitignore` from governance template, then fetch and append language-specific patterns from github.com/github/gitignore for each primary language
9. Create `README.md` from template, replacing `{project}` placeholder
10. Run `npx markdownlint-cli2` on all generated files
11. Display next steps: start a new Claude session in the project directory, run `/{project}:setup`, then fill in AGENTS.md and system.md

### What it does NOT do

- Fill in AGENTS.md sections (tech stack, code style, etc.) — that requires project-specific knowledge
- Write system.md content — that requires architectural decisions
- Create the first feature spec — the user does that via `/{project}:specify`
- Make any git commits — the user decides when to commit
- Run `/{project}:setup` — that runs in the new project's Claude session, not governance's

## Acceptance Criteria

### Standard pipeline commands

- [x] All ten command templates copied to `.claude/commands/gov/` with `{project}` replaced by `gov`
- [x] Commands reference `.claude/gov-session.json` for session state
- [x] Pipeline gates enforce status transitions (draft → clarified → planned → done)
- [x] Dependency checks enforced via AGENTS.md boundary rule

### /gov:init

- [x] Command exists at `.claude/commands/gov/init.md`
- [x] Command accepts project name, path, description, and primary language(s) as arguments or prompts for them
- [x] Verifies the target directory does not exist before proceeding
- [x] Creates a complete project directory with all governance files
- [x] Replaces `{project}` placeholder in all copied templates
- [x] Copies slash command templates into `.claude/commands/{project}/`
- [x] Copies spec templates into `specs/templates/`
- [x] Creates system spec stubs from templates
- [x] Creates `.gitignore` from template and appends language-specific patterns fetched from github.com/github/gitignore
- [x] Runs markdownlint on generated files
- [x] Displays next steps directing the user to a new session and `/{project}:setup`
- [x] Does not make git commits automatically
- [x] Does not fill in project-specific content (AGENTS.md sections, system.md)

## Resolved Questions

- **Running setup automatically** — no. Init runs from the governance repo; setup runs in the new project's Claude session. Tell the user to start a new session and run `/{project}:setup`. Promotes smaller, more accurate context per session.
- **Initial commit** — leave to the user. They may want to review generated files, fill in AGENTS.md, or make adjustments before committing.
- **Minimal flag** — not now. Templates are empty prompts that cost nothing to include. If a project doesn't use events, they delete `events.md`. See [specs/spec.md](../spec.md#future-considerations) for deferred rationale.
- **Existing directory** — verify it doesn't exist and stop if it does. Running in an existing directory risks overwriting files. The manual bootstrap steps in the README cover adding governance to existing projects.
- **Command location** — `.claude/commands/gov/init.md`, invoked as `/gov:init`. Consistent with the slash command pattern from spec 000.
- **Language-specific gitignore** — init asks for primary language(s) and fetches patterns from github.com/github/gitignore to append to the minimal .gitignore template.
