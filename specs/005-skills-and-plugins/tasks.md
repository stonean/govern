# 005 — Skills and Plugins Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create the skill registry

Create `framework/skills/registry.json` with the v1 starter set of nine entries (TypeScript / Python / Go × Linting / Testing / Formatting). Validate against the schema in `data-model.md`.

- [x] Create `framework/skills/` directory
- [x] Create `framework/skills/registry.json` with the nine starter entries
- [x] Each entry includes `name`, `category`, `trigger.field`, `trigger.value`, `template`, `description`
- [x] All `category` values are drawn from the fixed set
- [x] All `trigger.field` values are drawn from the recognized tech stack keys
- [x] All `template` paths end in `.md`
- [x] File is valid JSON

**Done when:** `framework/skills/registry.json` exists, parses as JSON, and contains nine entries that validate against `data-model.md`.

## 2. Create the v1 skill templates

Create one `.md` template file under `framework/skills/templates/` for each registry entry. Each template follows the slash-command prompt format and uses the standard placeholders.

- [ ] Create `framework/skills/templates/` directory
- [ ] `lint-typescript-eslint.md`
- [ ] `test-typescript-vitest.md`
- [ ] `format-typescript-prettier.md`
- [ ] `lint-python-ruff.md`
- [ ] `test-python-pytest.md`
- [ ] `format-python-black.md`
- [ ] `lint-go-golangci-lint.md`
- [ ] `test-go-gotest.md`
- [ ] `format-go-gofmt.md`
- [ ] Every registry entry's `template` path resolves to an existing file
- [ ] All templates use `{project}` and `{cli-config-dir}` consistently with existing slash commands
- [ ] All templates pass `npx markdownlint-cli2`

**Done when:** every registry entry has a corresponding template file, and all templates pass markdownlint.

## 3. Add the skill recommendation step to init

Modify `.claude/commands/gov/init.md` to insert the skill recommendation step after the tech stack questionnaire. This is a hand-maintained, governance-specific command (no source counterpart).

- [ ] Insert a new "Recommend and scaffold skills" step between current step 4 (tech stack questionnaire) and current step 5 (Create CLAUDE.md)
- [ ] Renumber steps 5–12 to 6–13
- [ ] The new step reads `framework/skills/registry.json` from the governance repo, matches entries case-insensitively against the in-memory tech stack selections, groups matches by category, presents per-category accept/skip prompts, and copies accepted templates to `.claude/commands/{slug}/skills/{template-stem}.md` with `{project}` and `{cli-config-dir}` substituted
- [ ] Step warns and continues if registry is missing or malformed (`Skill registry not found or invalid, skipping skill recommendations`)
- [ ] Step warns and skips individual templates whose file is missing
- [ ] Step is silently skipped if no entries match the user's selections
- [ ] All cross-references to step numbers elsewhere in `init.md` reflect the new numbering
- [ ] `.claude/commands/gov/init.md` passes `npx markdownlint-cli2`

**Done when:** init's step list includes the recommendation step, all step numbers and cross-references are consistent, and the file passes markdownlint.

## 4. Add registry sync and skill recommendation to govern

Modify `framework/bootstrap/govern.md` to ship the registry to adopted projects and offer new skills on subsequent runs.

- [ ] Add a new row to **Governance-owned shared files (strategy: update)** mapping `framework/skills/registry.json` → `skills/registry.json`
- [ ] Add a new "Skill Recommendation" step in the per-agent scaffolding flow, after **Slash command cleanup** and before **Session state**
- [ ] The step reads `skills/registry.json` (the just-synced local copy), matches entries case-insensitively against the AGENTS.md Tech Stack table, filters out entries whose target file already exists at `{config_dir}/commands/{project}/skills/{template-stem}.md`, groups remaining matches by category, presents per-category accept/skip prompts, fetches accepted templates from upstream (`framework/skills/templates/{template-stem}.md`), and writes them with `{project}` and `{cli-config-dir}` substituted
- [ ] Step is silently skipped if no AGENTS.md exists, no Tech Stack table is found, no entries match, or all matches are already scaffolded
- [ ] Step warns and continues if registry is missing or malformed
- [ ] Step warns and skips individual templates whose upstream fetch fails
- [ ] Edge case noted: scaffolded skill files in `{config_dir}/commands/{project}/skills/` are not affected by the existing slash command cleanup (the cleanup only walks top-level `.md` files in the project commands directory)
- [ ] `framework/bootstrap/govern.md` passes `npx markdownlint-cli2`

**Done when:** govern syncs the registry as an `update`-strategy file, offers new skills after sync, never overwrites already-scaffolded skill files, and the file passes markdownlint.

## 5. Validate end-to-end and run readiness checks

Run all markdownlint and structural checks, and verify the spec's acceptance criteria are satisfied by the produced artifacts.

- [ ] Every `template` path in `framework/skills/registry.json` points to an existing file under `framework/skills/templates/`
- [ ] Every category value in the registry is in the fixed set
- [ ] Every `trigger.field` value in the registry is in the recognized set
- [ ] `npx markdownlint-cli2` passes on all created/modified `.md` files (skill templates, init, govern, plan, tasks, data-model)
- [ ] `python -m json.tool framework/skills/registry.json` (or equivalent JSON validator) succeeds
- [ ] Each acceptance criterion in `spec.md` is checked individually against the produced artifacts and marked `- [x]` only if satisfied

**Done when:** all checks pass, the registry and templates are mutually consistent, and every acceptance criterion is satisfied.
