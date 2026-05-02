# 005 — Skills and Plugins Code Locations

## AC: A skill registry exists at `skills/registry.json` in governance, using JSON format

- `framework/skills/registry.json`

## AC: Each registry entry specifies a single-field trigger, skill name, category, template path, and description

- `framework/skills/registry.json`

## AC: Categories are drawn from the fixed set: Testing, Linting, Formatting, Migrations, Code Review, Deployment

- `framework/skills/registry.json`

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
