# 005 — Skills and Plugins Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create skill registry

Create `skills/registry.json` with entries for the common tech stack selections from the 004 questionnaire. Each entry contains trigger, skill name, category, template path, and description. Cover the primary languages (TypeScript, Python, Go) across available categories (Linting, Testing, Formatting at minimum).

- [ ] Create `skills/` directory
- [ ] Create `skills/registry.json` with initial entries
- [ ] Validate JSON is well-formed

**Done when:** `skills/registry.json` exists with entries covering TypeScript, Python, and Go for at least Linting, Testing, and Formatting categories.

## 2. Create skill templates

Create `.md` template files in `skills/templates/` for each registry entry. Each template follows the same format as existing slash commands and uses `{project}` and other standard placeholders.

- [ ] Create `skills/templates/` directory
- [ ] Create template files matching every registry entry's template path
- [ ] Templates use `{project}` placeholder and follow slash command format

**Done when:** Every template path referenced in `skills/registry.json` has a corresponding file in `skills/templates/`. All templates pass markdownlint.

## 3. Update init command with skill recommendation step

Add a skill recommendation step to `commands/init.md` after the tech stack questionnaire (current step 4). The step reads the registry, matches against selections, groups by category, presents to user, and scaffolds accepted templates.

- [ ] Add skill recommendation step to `commands/init.md`
- [ ] Include match logic, category grouping, present-and-accept flow
- [ ] Include error handling: warn and skip if registry missing/malformed, warn and skip individual missing templates
- [ ] Re-derive `.claude/commands/gov/init.md` from updated template

**Done when:** Init command includes skill step. Both `commands/init.md` and `.claude/commands/gov/init.md` are updated and pass markdownlint.

## 4. Update govern commands with skill sync

Add `skills/registry.json` to the file manifest in both `govern/govern.md` and `govern/govern-auggie.md` with `update` strategy. Add a skill recommendation step after file sync that scans for new, unscaffolded skills and offers them to the user.

- [ ] Add registry to manifest in `govern/govern.md`
- [ ] Add skill recommendation step after sync in `govern/govern.md`
- [ ] Add registry to manifest in `govern/govern-auggie.md`
- [ ] Add skill recommendation step after sync in `govern/govern-auggie.md`

**Done when:** Both govern files include the registry in their manifest and offer new skills after sync. Both pass markdownlint.

## 5. Validate and finalize

Run markdownlint on all new and modified files. Verify registry entries and template files are consistent.

- [ ] Run `markdownlint-cli2` on all new and modified files
- [ ] Verify every registry entry points to an existing template
- [ ] Verify spec acceptance criteria are met

**Done when:** All files pass markdownlint, registry and templates are consistent, acceptance criteria are satisfied.
