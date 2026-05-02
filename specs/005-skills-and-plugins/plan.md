# 005 — Skills and Plugins Plan

## Overview

Add a tech-stack-driven skill scaffolding bundle to the governance framework. The bundle is a registry (`framework/skills/registry.json`) plus a templates directory (`framework/skills/templates/`) of standalone `.md` skill files. Init recommends and scaffolds skills after the tech stack questionnaire. `govern.md` syncs the registry to adopted projects and offers any newly registered skills on subsequent runs. The feature is entirely prompt-and-data — no application code, only markdown commands, JSON, and template files.

## Technical Decisions

### Bundle lives at `framework/skills/`

A new top-level area under `framework/` keeps the registry and templates together because they are meaningless apart. The directory follows the framework's "place by purpose, not by file kind" rule: skills are a self-contained scaffolding bundle, distinct from constitution/rules/commands/templates/bootstrap.

```text
framework/skills/
  registry.json
  templates/
    lint-typescript-eslint.md
    lint-python-ruff.md
    ...
```

Alternative considered: split registry into `framework/registry/skills.json` and templates into `framework/templates/skills/`. Rejected — they always ship and evolve together; co-locating them makes the bundle obvious and avoids cross-directory hunting.

### Registry is a JSON array of entry objects

Per resolved question #1. Each entry has the schema fixed in `data-model.md`:

```json
{
  "name": "ESLint",
  "category": "Linting",
  "trigger": { "field": "backend_language", "value": "TypeScript" },
  "template": "lint-typescript-eslint.md",
  "description": "Run the ESLint linter for TypeScript code"
}
```

`template` is a path relative to `framework/skills/templates/`. `category` is drawn from the fixed set (Testing, Linting, Formatting, Migrations, Code Review, Deployment). `trigger.field` matches one of the tech stack questionnaire keys captured in init step 4 (`project_type`, `backend_language`, `backend_framework`, `database`, `messaging`, `backend_test_runner`, `frontend_language`, `frontend_framework`, `css_ui`, `frontend_test_runner`).

### One template file per language-tool combination

Per resolved question #6. Templates follow the `{workflow}-{language}-{tool}.md` naming convention. Each template uses the same `{project}` and `{cli-config-dir}` placeholders as existing slash commands so the same substitution pass works on them. A template is a small prompt file (a few dozen lines) describing how the agent should perform the workflow in that stack.

### Scaffold destination is `{config_dir}/commands/{project}/skills/`

A per-project `skills/` subdirectory under the existing project commands directory. This:

- Survives the slash command cleanup loop in `govern.md` unchanged — that loop only walks top-level `.md` files in `{config_dir}/commands/{project}/`, so files inside `skills/` are untouched.
- Discovers naturally as namespaced slash commands (`/{project}:skills:{template-stem}`) under Claude Code's standard subdirectory-as-namespace rule. Discovery for non-Claude agents may vary; v1 ships the file in the same conventional path and leaves agent-specific discovery rules to the agent.
- Keeps scaffolded skills clearly separated from pipeline commands so `/{project}:configure`, `/{project}:plan`, etc. remain visually distinct.

Alternative considered: a flat `{config_dir}/skills/` outside `commands/`. Rejected — `commands/` is the directory the agents already register; placing skills under it avoids needing additional discovery wiring per agent.

Alternative considered: same directory as pipeline commands with a `skill-` filename prefix. Rejected — would require expanding the slash command cleanup logic to whitelist a prefix, and the resulting names (`/{project}:skill-lint-typescript-eslint`) read worse than the namespaced form.

### Single-field, case-insensitive trigger matching

Per resolved question #3. Each registry entry has exactly one `trigger.field` and one `trigger.value`. Matching compares the trigger value against the corresponding tech stack selection case-insensitively. Multiple entries can share a trigger to recommend several skills for one selection.

The match source differs by entry point:

- **Init** matches against the in-memory selections from step 4 of the questionnaire.
- **Govern** matches against the existing project's AGENTS.md Tech Stack table (the same source 004 populates). Govern reads the table and maps each row's layer/technology back to the registry's trigger fields.

### Init scaffolds for Claude only; govern loops per selected agent

Init is governance-specific to Claude Code (per `CLAUDE.md` — `init.md` has no source counterpart and is hand-maintained). It scaffolds directly into `.claude/commands/{slug}/skills/`.

Govern operates over the agent registry and may scaffold for one or more agents. Skill scaffolding is performed inside the existing per-agent loop, with `{config_dir}` resolved per agent.

### Govern reads the local registry copy after sync

The skill recommendation step in `govern.md` runs **after** the manifest has copied `framework/skills/registry.json` into the project at `skills/registry.json`. Recommendation reads the just-synced local copy rather than re-fetching from upstream. This avoids a redundant fetch and matches the pattern already used by other shared files (e.g., `specs/templates/`).

Templates are fetched on demand from upstream at scaffold time using the same URL pattern as the rest of govern's fetches. They are not synced into the project tree by default — the project only carries the registry as a manifest of what is available.

### Init's recommendation step is inserted after step 4 (tech stack), renumbering 5–12 to 6–13

The current `.claude/commands/gov/init.md` has 12 ordered steps. The skill recommendation step needs the full set of tech stack selections and must run before AGENTS.md is finalized so any future "skills installed" annotation can be added if desired. The natural slot is immediately after step 4. Existing steps 5–12 shift down by one.

The "Display next steps" final step already enumerates configure/AGENTS/system/initialize/specify follow-ups. No additional next-step item is required for skills — they are scaffolded inside the new step itself.

### Govern's recommendation step is inserted after manifest processing and before post-scaffolding output

Govern's flow is: pre-flight → agent selection → permission setup → project configuration → frontmatter migration → file fetching (manifest) → per-agent scaffolding → post-scaffolding output. The skill recommendation step belongs at the end of per-agent scaffolding, after slash command cleanup and after the registry has been written to the project. It iterates over selected agents and offers any newly matched, not-yet-scaffolded skills.

### Initial registry coverage

Three languages × three categories = nine entries in v1, matching the most common picks from the 004 questionnaire:

| Trigger field | Trigger value | Category | Template |
| --- | --- | --- | --- |
| `backend_language` | TypeScript | Linting | `lint-typescript-eslint.md` |
| `backend_language` | TypeScript | Testing | `test-typescript-vitest.md` |
| `backend_language` | TypeScript | Formatting | `format-typescript-prettier.md` |
| `backend_language` | Python | Linting | `lint-python-ruff.md` |
| `backend_language` | Python | Testing | `test-python-pytest.md` |
| `backend_language` | Python | Formatting | `format-python-black.md` |
| `backend_language` | Go | Linting | `lint-go-golangci-lint.md` |
| `backend_language` | Go | Testing | `test-go-gotest.md` |
| `backend_language` | Go | Formatting | `format-go-gofmt.md` |

Frontend coverage is intentionally out of scope for v1 — most frontend languages overlap with backend (TypeScript) and the registry is designed for easy extension.

### No JSON Schema for the registry in v1

Per resolved question equivalent in the original plan. Validation is done at read time by init/govern: well-formed JSON, required fields present, category in the fixed set, template path exists. Failures emit warnings and skip entries; init/govern do not abort.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/skills/registry.json` | Create | Skill registry mapping tech stack to templates |
| `framework/skills/templates/lint-typescript-eslint.md` | Create | TypeScript ESLint skill |
| `framework/skills/templates/test-typescript-vitest.md` | Create | TypeScript Vitest skill |
| `framework/skills/templates/format-typescript-prettier.md` | Create | TypeScript Prettier skill |
| `framework/skills/templates/lint-python-ruff.md` | Create | Python Ruff lint skill |
| `framework/skills/templates/test-python-pytest.md` | Create | Python pytest skill |
| `framework/skills/templates/format-python-black.md` | Create | Python Black formatter skill |
| `framework/skills/templates/lint-go-golangci-lint.md` | Create | Go golangci-lint skill |
| `framework/skills/templates/test-go-gotest.md` | Create | Go test skill |
| `framework/skills/templates/format-go-gofmt.md` | Create | Go gofmt skill |
| `.claude/commands/gov/init.md` | Modify | Insert skill recommendation step after tech stack questionnaire; renumber 5–12 → 6–13 |
| `framework/bootstrap/govern.md` | Modify | Add registry to manifest with `update` strategy; add skill recommendation step in per-agent scaffolding |
| `specs/005-skills-and-plugins/data-model.md` | Create | Schema for registry entries |

## Trade-offs

### Starter set vs. comprehensive coverage

V1 ships nine templates covering TypeScript / Python / Go × Lint / Test / Format. Less common stacks (Ruby, frontend frameworks, databases, messaging) match no entries and silently fall through. Acceptable because the registry is designed for easy extension — adding a skill is one registry entry plus one template file.

### Category-level accept vs. per-skill accept

Grouping the present-and-accept flow by category reduces interaction cost but means the user can't cherry-pick within a category. With v1's coverage (typically 1 template per workflow per language), this is rarely felt.

### Skill discoverability across agents

Scaffolding to `{config_dir}/commands/{project}/skills/` works cleanly for Claude Code (subdirectory namespacing). For Auggie or future agents whose command discovery rules differ, the user may need to move or re-link the files. V1 ships in the conventional path; agent-specific discovery wiring is deferred.

### Templates not synced to project

Only the registry is shipped to adopted projects; templates are fetched on demand from upstream during scaffolding. Reduces project surface area but adds a network dependency at scaffold time. Govern already depends on the network for the rest of its file fetching, so this introduces no new failure mode.

### Single-field triggers

Compound triggers (AND/OR across fields) are deferred. A skill that depends on language *and* framework (e.g., a Rails-specific lint config) is expressed as multiple entries with the same template path, each keyed to a different field. Acceptable for v1; compound triggers can be added without changing the existing entry shape.

## Open Questions Resolved

All open questions were resolved during clarification. See `spec.md` Resolved Questions section.
