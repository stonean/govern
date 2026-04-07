# 002 — Project Scaffolding Templates Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create templates/project-readme.md

- [x] Create the template with sections: project name/description, Quick Start, Getting Started (referencing setup and status commands), Documentation, Feature Specs table, Development Pipeline, Slash Commands table, Working on Existing Specs
- [x] Use `{project}` placeholder in all command references and project name locations
- [x] Include an empty feature table with correct column headers (Spec, Status, Dependencies, Description)
- [x] Include the pipeline diagram with `/{project}:*` command references

Done when: `templates/project-readme.md` exists with all sections, uses `{project}` placeholders, includes getting started with setup/status references, passes markdownlint.

## 2. Create templates/gitignore

- [x] Create the template with sections: secrets (`.env`, `.env.*`), Claude settings (`.claude/*`, `!.claude/commands/`), IDE files (`.vscode/`, `.idea/`, `*.swp`, `*.swo`, `*~`), OS files (`.DS_Store`, `Thumbs.db`)
- [x] Name the file `gitignore` (no dot) to avoid being treated as active gitignore

Done when: `templates/gitignore` exists with minimal entries, preserves `.claude/commands/`, no language-specific patterns.

## 3. Create templates/claude-md.md

- [x] Create the template with `@import constitution.md` and `@import AGENTS.md`

Done when: `templates/claude-md.md` exists with both import directives, passes markdownlint.

## 4. Final review and lint

- [x] Run `npx markdownlint-cli2` on markdown templates (`project-readme.md`, `claude-md.md`)
- [x] Verify `{project}` placeholder is used consistently across all templates
- [x] Compare against anvil's README, .gitignore, and CLAUDE.md to ensure no important sections are missing
- [x] Update spec status to `done`

Done when: all templates pass lint (where applicable), placeholders are consistent, and no major gaps compared to anvil.
