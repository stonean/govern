---
description: Scaffold a new project with govern files, templates, and commands.
argument-hint: "[project slug]"
---

# Init

Scaffold a new project with `govern` files, templates, and slash commands.

## Purpose

Automates the manual bootstrap process from the `govern` README. Creates a complete project directory with all `govern` files, spec templates, and slash commands ready to use. This command is `govern`-specific — it does not exist in the `framework/commands/` template set.

## Inputs

Collect from `$ARGUMENTS` or prompt the user interactively. When using AskUserQuestion, every question **must** include an `options` array with 2–4 example choices (the user can always select "Other" for custom input):

1. **Project slug** — used for directory name, command prefix, and `{project}` placeholder substitution. Must be lowercase, alphanumeric, hyphens allowed. Example options: `myapp`, `my-service`.
2. **Project path** — where to create the project directory. Defaults to a sibling of the `govern` repo (i.e., `../../{slug}` relative to this repo, or the parent directory of wherever `govern` lives). Example options: the computed default path, `~/src/{slug}`.
3. **Project description** — one-line description for README and AGENTS.md. Example options: `A new microservice`, `CLI tool for X`.
4. **Tech stack** — a guided questionnaire that replaces the old "primary language(s)" question. Ask each question interactively using AskUserQuestion with 2–4 example choices plus "Other" and "Skip". The flow is:

   **4a. Project type** — Example options: `backend`, `frontend`, `fullstack`.

   **4b. Backend questions** (ask only if project type is `backend` or `fullstack`):

   - **Backend framework** — Example options: `Rails`, `FastAPI`, `Gin`, `Fastify`.
   - **Backend language** — derived automatically from the framework when the framework determines it; in that case no question and no example options are shown. Asked only as a fallback — when the framework was skipped, answered "Other"/unrecognized, or is language-ambiguous — and only then with Example options: `Ruby`, `Python`, `Go`, `TypeScript`.
   - **Database** — Example options: `PostgreSQL`, `MySQL`, `SQLite`, `MongoDB`.
   - **Messaging** — Example options: `NATS`, `Kafka`, `RabbitMQ`, `Redis Pub/Sub`.
   - **Backend test runner** — Example options: `Vite`, `pytest`, `go test`, `RSpec`.

   **4c. Frontend questions** (ask only if project type is `frontend` or `fullstack`):

   - **Frontend framework** — Example options: `Svelte`, `Vue`, `React`, `Next.js`.
   - **Frontend language** — same derivation; asked only as a fallback, and only then with Example options: `TypeScript`, `JavaScript`. Most frontend frameworks are language-ambiguous (TypeScript or JavaScript), so this one is usually asked.
   - **CSS/UI** — Example options: `Tailwind`, `SCSS`, `styled-components`.
   - **Frontend test runner** — Example options: `Vitest`, `Jest`, `Playwright`.

   **Framework-implies-language inference.** Ask the framework question before the language question in each section. When the selected framework unambiguously determines its language — e.g. Rails → Ruby, Sinatra → Ruby, Django / FastAPI / Flask → Python, Gin / Echo → Go, Laravel → PHP, Phoenix → Elixir, ASP.NET → C# — record that language automatically and present **no** language question and **no** language example options. Still write the language row into the AGENTS.md Tech Stack table, since language-triggered workflows (e.g. RuboCop and RSpec for Ruby) match on it. Show the language question — and its example options — only when the framework was skipped, answered "Other" with an unrecognized value, or is language-ambiguous (a Node framework that could be TypeScript or JavaScript, a JVM framework that could be Java or Kotlin, etc.).

   For `fullstack` projects, ask backend questions first, then frontend questions. Every question can be skipped — the user is not required to answer any category.

5. **Spec-root directory** — the top-level directory that will hold every `govern` artifact (feature dirs, `inbox.md`, `rules/`, shared docs). Defaults to `specs`; accept the default unless a different name is needed to avoid colliding with an existing directory (e.g. RSpec's `spec/`). Example options: `specs` (the default), `governance`, `design`. Validate the entered value: empty, a path separator (`/` or `\`), `..`, or a leading slash is rejected with "Spec-root must be a single directory name (no separators, no '..', no leading slash)." When a non-`specs` name is chosen, init records it in the new project's `.govern.toml` so every command and the runtime resolve it (spec 040). Referred to below as `{spec-root}`.

Validate the project slug: must be lowercase, alphanumeric, and hyphens only. If invalid, reject with: "Project slug must be lowercase, alphanumeric, and hyphens only."

## Pre-flight Check

Before scaffolding, verify:

- The target directory (`{path}/{slug}`) does **not** already exist.
- If it exists, **stop immediately** and report: "Directory already exists at {path}/{slug}. Init is for new projects only. To add `govern` to an existing project, follow the manual bootstrap steps in the `govern` README."

## Scaffolding Steps

Perform all steps in order. Use the `govern` repo as the source for all template files.

### 1. Create project directory and initialize git

```bash
mkdir -p {path}/{slug}
cd {path}/{slug}
git init
```

### 2. Copy `govern` files

Copy these files from the `govern` repo into the new project root:

- `framework/constitution.md` → `constitution.md`
- `.markdownlint-cli2.jsonc` → `.markdownlint-cli2.jsonc`

### 3. Copy and customize AGENTS.md

Copy `framework/templates/project/agents.md` from the `govern` repo into the new project root as `AGENTS.md`. Replace every `{project-name}` placeholder with the user-provided project slug. Replace `{One-line project description.}` with the user-provided description.

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

Copy `framework/templates/project/claude-md.md` from the `govern` repo into the new project as `CLAUDE.md`.

### 5. Create the spec-root directory with system spec templates

`{spec-root}` is the directory chosen in input 5 (default `specs`). When it is **not** `specs`, first write the new project's `.govern.toml` so every command and the runtime resolve the rename:

```toml
[paths]
specs-root = "{spec-root}"
```

Create `{spec-root}/` and copy these files from `framework/templates/project/` in the `govern` repo:

- `framework/templates/project/system.md` → `{spec-root}/system.md`
- `framework/templates/project/errors.md` → `{spec-root}/errors.md`
- `framework/templates/project/events.md` → `{spec-root}/events.md`
- `framework/templates/project/inbox.md` → `{spec-root}/inbox.md`

Also create `{spec-root}/rules/` and copy the shipped rule files from `framework/rules/` into it so `/{slug}:review` and `/{slug}:analyze` have rules to load on day one:

- `framework/rules/accessibility-frontend.md` → `{spec-root}/rules/accessibility-frontend.md`
- `framework/rules/api-backend.md` → `{spec-root}/rules/api-backend.md`
- `framework/rules/configuration-cross.md` → `{spec-root}/rules/configuration-cross.md`
- `framework/rules/performance-frontend.md` → `{spec-root}/rules/performance-frontend.md`
- `framework/rules/security-backend.md` → `{spec-root}/rules/security-backend.md`
- `framework/rules/security-frontend.md` → `{spec-root}/rules/security-frontend.md`

### 6. Copy spec templates

Create `{spec-root}/templates/` and copy all spec-pipeline templates from `framework/templates/spec/` in the `govern` repo (the destination is flat — no `spec/` subdirectory):

- `spec.md`
- `spec-and-plan.md`
- `plan.md`
- `tasks.md`
- `data-model.md`
- `research.md`
- `scenario.md`

### 7. Copy slash command templates

Create `.claude/commands/{slug}/` and copy every `.md` file from the `govern` repo's `framework/commands/` directory into it. In each copied file, replace every `{project}` with the user-provided project name and every `{cli-config-dir}` with `.claude`.

Additionally, copy `framework/bootstrap/configure/claude.md` into `.claude/commands/{slug}/configure.md` (renaming it), substituting placeholders the same way.

### 8. Recommend and scaffold workflows

Match the user's tech stack selections from input step 4 against the workflow registry and offer matching workflows, grouped by category.

1. **Read the registry** at `framework/workflows/registry.json` from the `govern` repo.

   - If the file is missing or not valid JSON, warn `Workflow registry not found or invalid, skipping workflow recommendations` and skip the rest of this step entirely.
   - Validate each entry against the schema in `specs/005-workflows/data-model.md`: required fields, `category` in the fixed set, `trigger.field` in the recognized set, `template` ending in `.md`. Drop invalid entries with a per-entry warning.

2. **Match entries against selections.** For each registry entry, look up the user's selection for `entry.trigger.field` from the in-memory tech stack questionnaire results. Compare case-insensitively against `entry.trigger.value`. Collect every matching entry.

3. **Silent skip when there is nothing to offer.** If no entries match, do not prompt the user and proceed to step 9. The project still gets the standard pipeline commands.

4. **Group matches by category** in the order: `Linting`, `Formatting`, `Testing`, `Migrations`, `Code Review`, `Deployment`. Within each category, list each match's `name` and `description`.

5. **Present per-category accept/skip prompts.** For each non-empty category, ask the user via `AskUserQuestion`: "Scaffold these {category} workflows?" with the matched entries listed. Options: `Yes, scaffold all in this category`, `No, skip this category`. The user must explicitly accept — no workflows are scaffolded without consent.

6. **Scaffold accepted workflows.** For each accepted entry:

   - Read `framework/workflows/{entry.template}` from the `govern` repo. (The workflows directory is flat — no inner `templates/` subdirectory.)
   - If the workflow file is missing, warn `Workflow file {entry.template} not found, skipping` and continue with the next accepted entry. Do not abort.
   - Replace every `{project}` with the user-provided project slug and every `{cli-config-dir}` with `.claude`.
   - Write the substituted content to `.claude/commands/{slug}/workflows/{entry.template}` (creating the `workflows/` directory if needed).

7. **Summarize.** Display a one-line summary: `Scaffolded N workflows under .claude/commands/{slug}/workflows/.` If zero workflows were scaffolded (all categories skipped), say `No workflows scaffolded.`

### 9. Create .gitignore

Copy `framework/templates/project/gitignore` from the `govern` repo into the new project as `.gitignore`.

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

### 10. Create README.md

Copy `framework/templates/project/project-readme.md` from the `govern` repo into the new project as `README.md`. Replace every `{project}` with the user-provided project name. Replace `{Brief description of what this project does.}` with the user-provided description.

### 11. Create session file

Create `.claude/{slug}-session.json` with empty content `{}`.

### 12. Run markdownlint

Run `npx markdownlint-cli2` on all generated `.md` files in the new project directory. Fix any issues found.

### 13. Display next steps

After scaffolding is complete, display:

---

**Project scaffolded successfully at `{path}/{slug}`.**

Next steps:

1. Start a new Claude Code session in the project directory: `cd {path}/{slug}`
2. Run `/{slug}:configure` to apply the full permission set
3. Fill in `AGENTS.md` — project structure, code style, testing conventions, and gotchas (Tech Stack table was populated from your selections)
4. Fill in `{spec-root}/system.md` — architecture, request lifecycle, shared infrastructure
5. Create your first feature spec: `/{slug}:specify {feature description}`

---

## What This Command Does NOT Do

- Fill in AGENTS.md convention sections (code style, testing, gotchas, etc.) — that requires project-specific knowledge
- Write system.md content — that requires architectural decisions
- Create the first feature spec — the user does that via `/{slug}:specify`
- Make any git commits — the user decides when to commit
- Run `/{slug}:configure` — that runs in the new project's Claude session, not `govern`'s
