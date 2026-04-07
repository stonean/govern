# 005 — Skills and Plugins

**Status:** planned
**Dependencies:** 004-tech-stack-selection

Based on project tech stack, recommend and scaffold relevant Claude Code skill templates during bootstrap.

## Problem

Claude Code has a rich plugin ecosystem with production-ready skills for common workflows (code review, commit automation, frontend design, etc.) and the ability to define custom project-specific skills. Currently, init produces a project with only pipeline commands — no development workflow skills. Users must discover and install plugins manually, unaware of what's available or what fits their stack.

## Behavior

### Skill registry

Governance maintains a skill registry at `skills/registry.json` — a JSON file that maps tech stack selections to recommended skill templates. Each entry in the registry contains:

- **Trigger** — a single tech stack field and value that activates this recommendation (e.g., `{"field": "backend_language", "value": "TypeScript"}`)
- **Skill name** — human-readable name (e.g., "TypeScript Linter", "Database Migration")
- **Category** — one of the fixed categories: `Testing`, `Linting`, `Formatting`, `Migrations`, `Code Review`, `Deployment`
- **Template** — path to the template file in governance, relative to `skills/templates/` (e.g., `lint-typescript-eslint.md`)
- **Description** — one-line explanation of what the skill does

Each trigger matches a single tech stack field. A skill is recommended when the user's selection for that field matches the trigger value. Multiple entries can share the same trigger to recommend several skills for one selection.

### Recommendation flow

During `/gov:init`, after the tech stack questionnaire (step 4 from 004), the system:

1. **Matches** — scans the registry for entries whose trigger field and value match any of the user's tech stack selections. If no entries match, skip the skill step entirely — do not prompt the user.
2. **Presents** — displays matched skills grouped by category with name and description. The user can accept or skip each category group.
3. **Scaffolds** — for accepted skills, copies the skill template from `skills/templates/` in governance into `.claude/commands/{slug}/` in the new project, replacing `{project}` and other standard placeholders.

### Skill templates

Each skill template is a standalone `.md` file in `skills/templates/`, one file per language-tool combination. Templates follow the same format as existing slash commands and are parameterized with `{project}` and other standard placeholders.

Naming convention: `{workflow}-{language}-{tool}.md` (e.g., `lint-typescript-eslint.md`, `test-python-pytest.md`, `format-go-gofmt.md`).

Skill templates cover common development workflows that are tech-stack-specific but not project-specific. Examples:

- **Lint** — run the stack-appropriate linter (e.g., `eslint` for TypeScript, `ruff` for Python, `golangci-lint` for Go)
- **Test** — run the stack-appropriate test runner with conventional options
- **Migrate** — run database migrations for the selected database/ORM
- **Format** — run the stack-appropriate code formatter

These are starting points — projects customize them after scaffolding.

### Govern integration

When `/{project}:govern` syncs governance files, it also updates the skill registry file in the project (using the same `update` strategy as other governance files). After updating, govern scans for new skill recommendations that were not previously scaffolded and offers them to the user, following the same present-and-accept flow as init.

Skills already scaffolded in `.claude/commands/{slug}/` are not overwritten — they may have been customized. Only new, unscaffolded skills are offered.

### No-match behavior

If the user's tech stack selections match no registry entries (e.g., all categories skipped, or an uncommon stack), skip the recommendation step entirely. Do not prompt the user about skills. The project gets only the standard pipeline commands, same as today.

### Edge cases

- **Registry file missing or malformed** — init warns ("Skill registry not found or invalid, skipping skill recommendations") and continues without the skill step. Init must not fail due to registry issues.
- **Template file missing** — if a registry entry references a template that does not exist in `skills/templates/`, warn ("Skill template {name} not found, skipping") and skip that individual skill. Continue with remaining skills.
- **Duplicate triggers** — multiple registry entries can match the same tech stack selection. All matched entries are presented. This is expected (e.g., TypeScript triggers both a lint skill and a format skill).
- **Govern with customized skills** — govern detects existing skill files by checking if a file with the expected name already exists in `.claude/commands/{slug}/`. If it exists, the skill is treated as already scaffolded and is not offered again, regardless of content changes.

## Acceptance Criteria

- [ ] A skill registry exists at `skills/registry.json` in governance, using JSON format
- [ ] Each registry entry specifies a single-field trigger, skill name, category, template path, and description
- [ ] Categories are drawn from the fixed set: Testing, Linting, Formatting, Migrations, Code Review, Deployment
- [ ] During init, after tech stack selection, matched skills are presented to the user grouped by category
- [ ] The user can accept or skip each category group — no skills are scaffolded without consent
- [ ] Accepted skill templates are copied into `.claude/commands/{slug}/` with placeholders replaced
- [ ] Skipping all skill recommendations produces the same project as today (backwards compatible)
- [ ] If no registry entries match the user's tech stack, the skill step is skipped silently
- [ ] Skill templates use the naming convention `{workflow}-{language}-{tool}.md`
- [ ] Skill templates follow the same `.md` format and placeholder conventions as existing slash commands
- [ ] The registry is extensible — adding a new skill requires only a registry entry and a template file
- [ ] Init warns and continues (does not fail) if the registry file is missing or malformed
- [ ] Init warns and skips individual skills whose template file is missing
- [ ] `/{project}:govern` updates the registry and offers new, unscaffolded skills to the user
- [ ] Govern does not overwrite skill files that already exist in the project

## Resolved Questions

1. **Registry format** — JSON. Consistent with `gov-session.json`, `settings.local.json`, and other structured files in the project. No new format dependency.
2. **Plugin ecosystem maturity** — v1 focuses exclusively on template skills that governance fully controls. Plugin/marketplace support is deferred to a future spec when the ecosystem stabilizes.
3. **Trigger complexity** — single-value matching only. Each trigger matches one tech stack field to one value. Compound logic (AND/OR) is deferred — single triggers cover the common cases and keep the registry simple.
4. **Skill categories** — fixed set: Testing, Linting, Formatting, Migrations, Code Review, Deployment. Adding a new category requires a governance update. This ensures consistent grouping in the UI.
5. **Update mechanism** — `/{project}:govern` updates the registry file and offers new skill recommendations. This integrates naturally with the existing govern flow. A standalone `/{project}:skills` command is not needed for v1 since govern covers the use case.
6. **Template granularity** — one file per language-tool combination (e.g., `lint-typescript-eslint.md`). Explicit, easy to maintain. Minimal duplication since each template is small.
