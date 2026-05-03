---
status: done
dependencies: [004-tech-stack-selection]
tags: [bootstrap, templates]
---

# 005 — Workflows

Based on project tech stack, recommend and scaffold relevant development workflow files during bootstrap.

## Problem

Common development workflows — lint, test, format, migrate — are tech-stack-specific but not project-specific. Currently, init produces a project with only pipeline commands and no stack-aware workflow definitions. Users must discover and install workflow tooling manually, unaware of what's available or what fits their stack.

## Behavior

### Workflow registry

Governance maintains a workflow registry at `framework/workflows/registry.json` — a JSON file that maps tech stack selections to recommended workflow files. Each entry in the registry contains:

- **Trigger** — a single tech stack field and value that activates this recommendation (e.g., `{"field": "backend_language", "value": "TypeScript"}`)
- **Workflow name** — human-readable name (e.g., "ESLint", "pytest")
- **Category** — one of the fixed categories: `Testing`, `Linting`, `Formatting`, `Migrations`, `Code Review`, `Deployment`
- **Template** — path to the workflow file in governance, relative to `framework/workflows/` (e.g., `lint-typescript-eslint.md`). Field name is `template` because the file contains placeholders that get substituted at scaffold time.
- **Description** — one-line explanation of what the workflow does

Each trigger matches a single tech stack field. A workflow is recommended when the user's selection for that field matches the trigger value. Multiple entries can share the same trigger to recommend several workflows for one selection.

### Recommendation flow

During `/gov:init`, after the tech stack questionnaire (step 4 from 004), the system:

1. **Matches** — scans the registry for entries whose trigger field and value match any of the user's tech stack selections. If no entries match, skip the workflow step entirely — do not prompt the user.
2. **Presents** — displays matched workflows grouped by category with name and description. The user can accept or skip each category group.
3. **Scaffolds** — for accepted workflows, copies the workflow file from `framework/workflows/` in governance into `.claude/commands/{slug}/workflows/` in the new project, replacing `{project}` and other standard placeholders.

### Workflow files

Each workflow file is a standalone `.md` file in `framework/workflows/`, one file per language-tool combination. Files follow the same format as existing slash commands and are parameterized with `{project}` and other standard placeholders. The workflows directory is flat — registry and workflow files sit at the same level.

Naming convention: `{workflow}-{language}-{tool}.md` (e.g., `lint-typescript-eslint.md`, `test-python-pytest.md`, `format-go-gofmt.md`).

Workflow files cover common development workflows that are tech-stack-specific but not project-specific. Examples:

- **Lint** — run the stack-appropriate linter (e.g., `eslint` for TypeScript, `ruff` for Python, `golangci-lint` for Go)
- **Test** — run the stack-appropriate test runner with conventional options
- **Migrate** — run database migrations for the selected database/ORM
- **Format** — run the stack-appropriate code formatter

These are starting points — projects customize them after scaffolding.

### Govern integration

When `/{project}:govern` syncs governance files, it also updates the workflow registry file in the project (using the same `update` strategy as other governance files). After updating, govern scans for new workflow recommendations that were not previously scaffolded and offers them to the user, following the same present-and-accept flow as init.

Workflows already scaffolded in `.claude/commands/{slug}/workflows/` are not overwritten — they may have been customized. Only new, unscaffolded workflows are offered.

### No-match behavior

If the user's tech stack selections match no registry entries (e.g., all categories skipped, or an uncommon stack), skip the recommendation step entirely. Do not prompt the user about workflows. The project gets only the standard pipeline commands, same as today.

### Edge cases

- **Registry file missing or malformed** — init warns ("Workflow registry not found or invalid, skipping workflow recommendations") and continues without the workflow step. Init must not fail due to registry issues.
- **Workflow file missing** — if a registry entry references a file that does not exist under `framework/workflows/`, warn ("Workflow file {name} not found, skipping") and skip that individual workflow. Continue with remaining workflows.
- **Duplicate triggers** — multiple registry entries can match the same tech stack selection. All matched entries are presented. This is expected (e.g., TypeScript triggers both a lint workflow and a format workflow).
- **Govern with customized workflows** — govern detects existing workflow files by checking if a file with the expected name already exists in `.claude/commands/{slug}/workflows/`. If it exists, the workflow is treated as already scaffolded and is not offered again, regardless of content changes.

## Acceptance Criteria

- [x] A workflow registry exists at `framework/workflows/registry.json` in governance, using JSON format
- [x] Each registry entry specifies a single-field trigger, workflow name, category, template path, and description
- [x] Categories are drawn from the fixed set: Testing, Linting, Formatting, Migrations, Code Review, Deployment
- [x] During init, after tech stack selection, matched workflows are presented to the user grouped by category
- [x] The user can accept or skip each category group — no workflows are scaffolded without consent
- [x] Accepted workflow files are copied into `.claude/commands/{slug}/workflows/` with placeholders replaced
- [x] Skipping all workflow recommendations produces the same project as today (backwards compatible)
- [x] If no registry entries match the user's tech stack, the workflow step is skipped silently
- [x] Workflow files use the naming convention `{workflow}-{language}-{tool}.md`
- [x] Workflow files follow the same `.md` format and placeholder conventions as existing slash commands
- [x] The registry is extensible — adding a new workflow requires only a registry entry and a workflow file
- [x] Init warns and continues (does not fail) if the registry file is missing or malformed
- [x] Init warns and skips individual workflows whose file is missing
- [x] `/{project}:govern` updates the registry and offers new, unscaffolded workflows to the user
- [x] Govern does not overwrite workflow files that already exist in the project
- [x] Rename internal terminology from "skills" to "workflows" to free the term "skills" for Anthropic-style context-loaded instruction packs (signpost: driven by [010-agent-autonomy](../010-agent-autonomy/spec.md); the rename also flattens the framework directory — workflow files now sit directly under `framework/workflows/` instead of an inner `templates/` subdirectory)

## Resolved Questions

1. **Registry format** — JSON. Consistent with `gov-session.json`, `settings.local.json`, and other structured files in the project. No new format dependency.
2. **Plugin ecosystem maturity** — v1 focuses exclusively on workflow files that governance fully controls. Plugin/marketplace support is deferred to a future spec when the ecosystem stabilizes.
3. **Trigger complexity** — single-value matching only. Each trigger matches one tech stack field to one value. Compound logic (AND/OR) is deferred — single triggers cover the common cases and keep the registry simple.
4. **Workflow categories** — fixed set: Testing, Linting, Formatting, Migrations, Code Review, Deployment. Adding a new category requires a governance update. This ensures consistent grouping in the UI.
5. **Update mechanism** — `/{project}:govern` updates the registry file and offers new workflow recommendations. This integrates naturally with the existing govern flow. A standalone `/{project}:workflows` command is not needed for v1 since govern covers the use case.
6. **File granularity** — one file per language-tool combination (e.g., `lint-typescript-eslint.md`). Explicit, easy to maintain. Minimal duplication since each workflow file is small.
