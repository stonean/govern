# 005 — Skills and Plugins Plan

## Overview

Create a skill registry (`skills/registry.json`) and skill templates (`skills/templates/`) in governance. Update the init command to recommend and scaffold skills after the tech stack questionnaire. Update adopt to sync the registry and offer new skills. The feature is entirely prompt-and-data — no application code, only markdown commands, JSON, and template files.

## Technical Decisions

### Registry lives at `skills/registry.json`

A top-level `skills/` directory keeps registry and templates together, separate from specs and command templates. The JSON file is an array of entry objects — flat, grep-able, and consistent with other structured files in the project (`gov-session.json`, `settings.local.json`).

Alternative considered: YAML for readability. Rejected — the project has no YAML dependency and JSON is already the structured-data format used everywhere else.

### One template file per language-tool combination

Each skill template is a standalone `.md` file following the naming convention `{workflow}-{language}-{tool}.md` (e.g., `lint-typescript-eslint.md`). This is explicit and avoids conditional logic inside templates. Duplication is minimal since each template is small (a few dozen lines of prompt instructions).

Alternative considered: parameterized templates with tool/language variables. Rejected — adds complexity for marginal savings. Each template is small enough that explicit files are easier to maintain and review.

### Skill recommendation inserted as step 5 in init (after tech stack, before CLAUDE.md)

The init command currently has 12 steps. The skill recommendation slots in after step 4 (tech stack questionnaire) because that's when all technology selections are available. Steps 5–12 shift to 6–13. The skill step:

1. Reads `skills/registry.json` from governance
2. Matches entries against collected tech stack selections
3. Groups matched skills by category and presents them
4. Scaffolds accepted templates into `{cli-config-dir}/commands/{slug}/`

If no entries match or the registry is missing, the step is skipped silently.

### Govern syncs registry and offers new skills

The govern file manifest gains one entry: `skills/registry.json` with `update` strategy (always overwritten, governance-owned). After syncing, govern scans the registry for matches against the project's AGENTS.md Tech Stack table, filters out skills whose template file already exists in `{cli-config-dir}/commands/{slug}/`, and offers new ones using the same present-and-accept flow as init.

Alternative considered: a separate `/{project}:skills` command. Rejected per resolved question #5 — adopt covers the use case for v1.

### Matching is single-field, case-insensitive

Each registry entry has a trigger with one field and one value. Matching compares the trigger value against the corresponding tech stack selection, case-insensitively. This keeps the registry simple and avoids compound logic. Multiple entries can share the same trigger to recommend several skills for one selection.

### Category grouping for user presentation

Skills are presented grouped by the six fixed categories: Testing, Linting, Formatting, Migrations, Code Review, Deployment. The user accepts or skips each category group as a whole. This reduces prompt fatigue compared to per-skill confirmation while still giving control.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `skills/registry.json` | Create | Skill registry mapping tech stack to templates |
| `skills/templates/*.md` | Create | Skill template files (one per language-tool combo) |
| `commands/init.md` | Modify | Add skill recommendation step after tech stack questionnaire |
| `.claude/commands/gov/init.md` | Modify | Re-derive from updated template with placeholders resolved |
| `govern/govern.md` | Modify | Add registry to manifest, add skill recommendation after sync |
| `govern/govern-auggie.md` | Modify | Same registry and skill changes as `govern.md` |
| `specs/005-skills-and-plugins/spec.md` | Modify | Update status to `planned` |

## Trade-offs

### Starter set of templates vs. comprehensive coverage

The initial registry will include templates for the most common tech stack selections from the 004 questionnaire (TypeScript, Python, Go — lint, test, format). Less common stacks get no recommendations until templates are added. This is acceptable because the registry is designed for easy extension — adding a skill requires only a registry entry and a template file.

### Category-level accept vs. per-skill accept

Grouping by category reduces prompt interactions but means the user can't cherry-pick within a category. Acceptable for v1 since categories are narrow (usually 1–2 skills per category per stack). If users request finer control, per-skill selection can be added later without changing the registry format.

### No validation schema for registry.json

The registry is validated at read time by the init/govern commands rather than by a separate schema file. If the file is malformed, the command warns and skips. This avoids adding a JSON Schema dependency for a single file.

## Open Questions Resolved

All open questions were resolved during clarification. See spec.md Resolved Questions section.
