# 005 — Skills and Plugins Code Locations

## AC: A skill registry exists at `skills/registry.json` in governance, using JSON format

- `framework/skills/registry.json`

## AC: Each registry entry specifies a single-field trigger, skill name, category, template path, and description

- `framework/skills/registry.json`

## AC: Categories are drawn from the fixed set: Testing, Linting, Formatting, Migrations, Code Review, Deployment

- `framework/skills/registry.json`

## AC: During init, after tech stack selection, matched skills are presented to the user grouped by category

- `.claude/commands/gov/init.md`

## AC: The user can accept or skip each category group — no skills are scaffolded without consent

- `.claude/commands/gov/init.md`

## AC: Accepted skill templates are copied into `.claude/commands/{slug}/` with placeholders replaced

- `.claude/commands/gov/init.md`

## AC: Skipping all skill recommendations produces the same project as today (backwards compatible)

- `.claude/commands/gov/init.md`

## AC: If no registry entries match the user's tech stack, the skill step is skipped silently

- `.claude/commands/gov/init.md`

## AC: Skill templates use the naming convention `{workflow}-{language}-{tool}.md`

- `framework/skills/templates/format-go-gofmt.md`
- `framework/skills/templates/format-python-black.md`
- `framework/skills/templates/format-typescript-prettier.md`
- `framework/skills/templates/lint-go-golangci-lint.md`
- `framework/skills/templates/lint-python-ruff.md`
- `framework/skills/templates/lint-typescript-eslint.md`
- `framework/skills/templates/test-go-gotest.md`
- `framework/skills/templates/test-python-pytest.md`
- `framework/skills/templates/test-typescript-vitest.md`

## AC: Skill templates follow the same `.md` format and placeholder conventions as existing slash commands

- `framework/skills/templates/format-go-gofmt.md`
- `framework/skills/templates/format-python-black.md`
- `framework/skills/templates/format-typescript-prettier.md`
- `framework/skills/templates/lint-go-golangci-lint.md`
- `framework/skills/templates/lint-python-ruff.md`
- `framework/skills/templates/lint-typescript-eslint.md`
- `framework/skills/templates/test-go-gotest.md`
- `framework/skills/templates/test-python-pytest.md`
- `framework/skills/templates/test-typescript-vitest.md`

## AC: The registry is extensible — adding a new skill requires only a registry entry and a template file

- `framework/skills/registry.json`

## AC: Init warns and continues (does not fail) if the registry file is missing or malformed

- `.claude/commands/gov/init.md`

## AC: Init warns and skips individual skills whose template file is missing

- `.claude/commands/gov/init.md`

## AC: `/{project}:govern` updates the registry and offers new, unscaffolded skills to the user

- `framework/bootstrap/govern.md`

## AC: Govern does not overwrite skill files that already exist in the project

- `framework/bootstrap/govern.md`
