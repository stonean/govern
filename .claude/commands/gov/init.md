# Init

Scaffold a new project with governance files, templates, and slash commands.

## Purpose

Automates the manual bootstrap process from the governance README. Creates a complete project directory with all governance files, spec templates, and slash commands ready to use. This command is governance-specific — it does not exist in the `commands/` template set.

## Inputs

Collect from `$ARGUMENTS` or prompt the user interactively. When using AskUserQuestion, every question **must** include an `options` array with 2–4 example choices (the user can always select "Other" for custom input):

1. **Project slug** — used for directory name, command prefix, and `{project}` placeholder substitution. Must be lowercase, alphanumeric, hyphens allowed. Example options: `myapp`, `my-service`.
2. **Project path** — where to create the project directory. Defaults to a sibling of the governance repo (i.e., `../../{slug}` relative to this repo, or the parent directory of wherever governance lives). Example options: the computed default path, `~/src/{slug}`.
3. **Project description** — one-line description for README and AGENTS.md. Example options: `A new microservice`, `CLI tool for X`.
4. **Tech stack** — a guided questionnaire that replaces the old "primary language(s)" question. Ask each question interactively using AskUserQuestion with 2–4 example choices plus "Other" and "Skip". The flow is:

   **4a. Project type** — Example options: `backend`, `frontend`, `fullstack`.

   **4b. Backend questions** (ask only if project type is `backend` or `fullstack`):

   - **Backend language** — Example options: `TypeScript`, `Python`, `Go`, `Ruby`.
   - **Backend framework** — Example options: `Fastify`, `FastAPI`, `Gin`, `Rails`.
   - **Database** — Example options: `PostgreSQL`, `MySQL`, `SQLite`, `MongoDB`.
   - **Messaging** — Example options: `NATS`, `Kafka`, `RabbitMQ`, `Redis Pub/Sub`.
   - **Backend test runner** — Example options: `Vite`, `pytest`, `go test`, `RSpec`.

   **4c. Frontend questions** (ask only if project type is `frontend` or `fullstack`):

   - **Frontend language** — Example options: `TypeScript`, `JavaScript`.
   - **Frontend framework** — Example options: `Svelte`, `Vue`, `React`, `Next.js`.
   - **CSS/UI** — Example options: `Tailwind`, `SCSS`, `styled-components`.
   - **Frontend test runner** — Example options: `Vitest`, `Jest`, `Playwright`.

   For `fullstack` projects, ask backend questions first, then frontend questions. Every question can be skipped — the user is not required to answer any category.

Validate the project slug: must be lowercase, alphanumeric, and hyphens only. If invalid, reject with: "Project slug must be lowercase, alphanumeric, and hyphens only."

## Pre-flight Check

Before scaffolding, verify:

- The target directory (`{path}/{slug}`) does **not** already exist.
- If it exists, **stop immediately** and report: "Directory already exists at {path}/{slug}. Init is for new projects only. To add governance to an existing project, follow the manual bootstrap steps in the governance README."

## Scaffolding Steps

Perform all steps in order. Use the governance repo as the source for all template files.

### 1. Create project directory and initialize git

```bash
mkdir -p {path}/{slug}
cd {path}/{slug}
git init
```

### 2. Copy governance files

Copy these files from the governance repo root into the new project root:

- `constitution.md`
- `.markdownlint-cli2.jsonc`

### 3. Copy and customize AGENTS.md

Copy `AGENTS.md` from the governance repo into the new project root. Replace every `{project-name}` placeholder with the user-provided project slug. Replace `{One-line project description.}` with the user-provided description.

Then, if the user selected any technologies in the tech stack questionnaire (step 4), replace the Tech Stack comment placeholder with an actual table. Build the table from the user's selections using this layer-to-role mapping:

| Layer | Role |
| --- | --- |
| **Language** | Application logic |
| **Backend language** | Backend application logic |
| **Frontend language** | Frontend application logic |
| **Backend framework** | HTTP framework |
| **Frontend framework** | UI framework |
| **Database** | Primary data store |
| **Messaging** | Message broker |
| **Backend test runner** | Backend test runner |
| **Frontend test runner** | Frontend test runner |
| **CSS/UI** | Styling |

For `backend` or `frontend` projects, use **Language** as the layer name. For `fullstack` projects, use **Backend language** and **Frontend language** to distinguish the two.

Only include rows for categories the user answered (not skipped). If all categories were skipped, leave the Tech Stack comment placeholder unchanged.

### 4. Create CLAUDE.md

Copy `templates/claude-md.md` from the governance repo into the new project as `CLAUDE.md`.

### 5. Create specs directory with system spec templates

Create `specs/` and copy these files from `templates/` in the governance repo:

- `templates/system.md` → `specs/system.md`
- `templates/errors.md` → `specs/errors.md`
- `templates/events.md` → `specs/events.md`

### 6. Copy spec templates

Create `specs/templates/` and copy all spec development templates from `templates/` in the governance repo:

- `spec.md`
- `spec-and-plan.md`
- `plan.md`
- `tasks.md`
- `data-model.md`
- `research.md`
- `scenario.md`

### 7. Copy slash command templates

Create `.claude/commands/{slug}/` and copy every `.md` file from the governance repo's `commands/` directory into it. In each copied file, replace every `{project}` with the user-provided project name and every `{cli-config-dir}` with `.claude`.

Additionally, copy `templates/initialize.md` from the governance repo into `.claude/commands/{slug}/initialize.md`. Replace `{project}` with the user-provided project name. This provides a stub initialize command that the project fills in with language-specific post-copy steps for use by the create command.

### 8. Create .gitignore

Copy `templates/gitignore` from the governance repo into the new project as `.gitignore`.

Then, for each language selected in the tech stack questionnaire (backend language and/or frontend language from step 4), fetch the language-specific patterns:

```text
https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore
```

Append each fetched content below the template entries, separated by a comment header:

```gitignore
# {Language}
{fetched content}
```

If a fetch fails (404 or network error), report the failure and continue with the remaining languages. The minimal template is still valid without language-specific patterns.

### 9. Create README.md

Copy `templates/project-readme.md` from the governance repo into the new project as `README.md`. Replace every `{project}` with the user-provided project name. Replace `{Brief description of what this project does.}` with the user-provided description.

### 10. Create session file

Create `.claude/{slug}-session.json` with empty content `{}`.

### 11. Run markdownlint

Run `markdownlint-cli2` on all generated `.md` files in the new project directory. Fix any issues found.

### 12. Display next steps

After scaffolding is complete, display:

---

**Project scaffolded successfully at `{path}/{slug}`.**

Next steps:

1. Start a new Claude Code session in the project directory: `cd {path}/{slug}`
2. Run `/{slug}:setup` to configure permissions
3. Fill in `AGENTS.md` — project structure, code style, testing conventions, and gotchas (Tech Stack table was populated from your selections)
4. Fill in `specs/system.md` — architecture, request lifecycle, shared infrastructure
5. Fill in `.claude/commands/{slug}/initialize.md` — language-specific post-copy steps for `/{slug}:create`
6. Create your first feature spec: `/{slug}:specify {feature description}`

---

## What This Command Does NOT Do

- Fill in AGENTS.md convention sections (code style, testing, gotchas, etc.) — that requires project-specific knowledge
- Write system.md content — that requires architectural decisions
- Create the first feature spec — the user does that via `/{slug}:specify`
- Make any git commits — the user decides when to commit
- Run `/{slug}:setup` — that runs in the new project's Claude session, not governance's
