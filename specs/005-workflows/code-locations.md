# 005 — Workflows Code Locations

## AC: A workflow registry exists at `framework/workflows/registry.json` in governance, using JSON format

- `framework/workflows/registry.json`

## AC: Each registry entry specifies a single-field trigger, workflow name, category, template path, and description

- `framework/workflows/registry.json`

## AC: Categories are drawn from the fixed set: Testing, Linting, Formatting, Migrations, Code Review, Deployment

- `framework/workflows/registry.json`

## AC: During init, after tech stack selection, matched workflows are presented to the user grouped by category

- `.claude/commands/gov/init.md`

## AC: The user can accept or skip each category group — no workflows are scaffolded without consent

- `.claude/commands/gov/init.md`

## AC: Accepted workflow files are copied into `.claude/commands/{slug}/workflows/` with placeholders replaced

- `.claude/commands/gov/init.md`

## AC: Skipping all workflow recommendations produces the same project as today (backwards compatible)

- `.claude/commands/gov/init.md`

## AC: If no registry entries match the user's tech stack, the workflow step is skipped silently

- `.claude/commands/gov/init.md`

## AC: Workflow files use the naming convention `{tool}.md` (revised from the original `{workflow}-{language}-{tool}.md` post-completion; see preamble Note)

- `framework/workflows/black.md`
- `framework/workflows/eslint.md`
- `framework/workflows/gofmt.md`
- `framework/workflows/golangci-lint.md`
- `framework/workflows/gotest.md`
- `framework/workflows/prettier.md`
- `framework/workflows/pytest.md`
- `framework/workflows/ruff.md`
- `framework/workflows/vitest.md`

## AC: Workflow files follow the same `.md` format and placeholder conventions as existing slash commands

- `framework/workflows/black.md`
- `framework/workflows/eslint.md`
- `framework/workflows/gofmt.md`
- `framework/workflows/golangci-lint.md`
- `framework/workflows/gotest.md`
- `framework/workflows/prettier.md`
- `framework/workflows/pytest.md`
- `framework/workflows/ruff.md`
- `framework/workflows/vitest.md`

## AC: The registry is extensible — adding a new workflow requires only a registry entry and a workflow file

- `framework/workflows/registry.json`

## AC: Init warns and continues (does not fail) if the registry file is missing or malformed

- `.claude/commands/gov/init.md`

## AC: Init warns and skips individual workflows whose file is missing

- `.claude/commands/gov/init.md`

## AC: `/{project}:govern` updates the registry and offers new, unscaffolded workflows to the user

- `framework/bootstrap/govern.md`

## AC: Govern does not overwrite workflow files that already exist in the project

- `framework/bootstrap/govern.md`

## AC: Rename internal terminology from "skills" to "workflows" to free the term "skills" for Anthropic-style context-loaded instruction packs

- `.claude/commands/gov/configure.md`
- `.claude/commands/gov/init.md`
- `README.md`
- `framework/bootstrap/configure/claude.md`
- `framework/bootstrap/govern.md`
- `framework/workflows/black.md`
- `framework/workflows/eslint.md`
- `framework/workflows/gofmt.md`
- `framework/workflows/golangci-lint.md`
- `framework/workflows/gotest.md`
- `framework/workflows/prettier.md`
- `framework/workflows/pytest.md`
- `framework/workflows/registry.json`
- `framework/workflows/ruff.md`
- `framework/workflows/vitest.md`
- `specs/005-workflows/code-locations.md`
- `specs/005-workflows/data-model.md`
- `specs/005-workflows/plan.md`
- `specs/005-workflows/spec.md`
- `specs/005-workflows/tasks.md`
- `specs/013-text-first-artifacts/plan.md`
