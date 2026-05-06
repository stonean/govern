---
title: "005-workflows — tasks"
---

# 005 — Workflows Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create the workflow registry

Create `framework/workflows/registry.json` with the v1 starter set of nine entries (TypeScript / Python / Go × Linting / Testing / Formatting). Validate against the schema in `data-model.md`.

- [x] Create `framework/workflows/` directory
- [x] Create `framework/workflows/registry.json` with the nine starter entries
- [x] Each entry includes `name`, `category`, `trigger.field`, `trigger.value`, `template`, `description`
- [x] All `category` values are drawn from the fixed set
- [x] All `trigger.field` values are drawn from the recognized tech stack keys
- [x] All `template` paths end in `.md`
- [x] File is valid JSON

**Done when:** `framework/workflows/registry.json` exists, parses as JSON, and contains nine entries that validate against `data-model.md`.

## 2. Create the v1 workflow files

Create one `.md` workflow file directly under `framework/workflows/` for each registry entry. Each file follows the slash-command prompt format and uses the standard placeholders. The directory is flat — workflow files sit alongside the registry, with no inner `templates/` subdirectory.

- [x] `lint-typescript-eslint.md`
- [x] `test-typescript-vitest.md`
- [x] `format-typescript-prettier.md`
- [x] `lint-python-ruff.md`
- [x] `test-python-pytest.md`
- [x] `format-python-black.md`
- [x] `lint-go-golangci-lint.md`
- [x] `test-go-gotest.md`
- [x] `format-go-gofmt.md`
- [x] Every registry entry's `template` path resolves to an existing file
- [x] All workflow files use `{project}` and `{cli-config-dir}` consistently with existing slash commands
- [x] All workflow files pass `npx markdownlint-cli2`

**Done when:** every registry entry has a corresponding workflow file, and all files pass markdownlint.

## 3. Add the workflow recommendation step to init

Modify `.claude/commands/gov/init.md` to insert the workflow recommendation step after the slash command templates are scaffolded (so `.claude/commands/{slug}/` exists). This is a hand-maintained, governance-specific command (no source counterpart).

- [x] Insert a new "Recommend and scaffold workflows" step as scaffolding step 8, after step 7 ("Copy slash command templates"), so the project commands directory exists before workflow files are written into it
- [x] Renumber steps 8–12 to 9–13 to make room
- [x] The new step reads `framework/workflows/registry.json` from the governance repo, matches entries case-insensitively against the in-memory tech stack selections, groups matches by category, presents per-category accept/skip prompts, and copies accepted workflow files to `.claude/commands/{slug}/workflows/{file-stem}.md` with `{project}` and `{cli-config-dir}` substituted
- [x] Step warns and continues if registry is missing or malformed (`Workflow registry not found or invalid, skipping workflow recommendations`)
- [x] Step warns and skips individual workflow files whose file is missing
- [x] Step is silently skipped if no entries match the user's selections
- [x] All cross-references to step numbers elsewhere in `init.md` reflect the new numbering
- [x] `.claude/commands/gov/init.md` passes `npx markdownlint-cli2`

**Done when:** init's step list includes the recommendation step, all step numbers and cross-references are consistent, and the file passes markdownlint.

## 4. Add registry sync and workflow recommendation to govern

Modify `framework/bootstrap/govern.md` to ship the registry to adopted projects and offer new workflows on subsequent runs.

- [x] Add a new row to **Governance-owned shared files (strategy: update)** mapping `framework/workflows/registry.json` → `workflows/registry.json`
- [x] Add a new "Workflow Recommendation" step in the per-agent scaffolding flow, after **Slash command cleanup** and before **Session state**
- [x] The step reads `workflows/registry.json` (the just-synced local copy), matches entries case-insensitively against the AGENTS.md Tech Stack table, filters out entries whose target file already exists at `{config_dir}/commands/{project}/workflows/{file-stem}.md`, groups remaining matches by category, presents per-category accept/skip prompts, fetches accepted workflow files from upstream (`framework/workflows/{file-stem}.md`), and writes them with `{project}` and `{cli-config-dir}` substituted
- [x] Step is silently skipped if no AGENTS.md exists, no Tech Stack table is found, no entries match, or all matches are already scaffolded
- [x] Step warns and continues if registry is missing or malformed
- [x] Step warns and skips individual workflow files whose upstream fetch fails
- [x] Edge case noted: scaffolded workflow files in `{config_dir}/commands/{project}/workflows/` are not affected by the existing slash command cleanup (the cleanup only walks top-level `.md` files in the project commands directory)
- [x] `framework/bootstrap/govern.md` passes `npx markdownlint-cli2`

**Done when:** govern syncs the registry as an `update`-strategy file, offers new workflows after sync, never overwrites already-scaffolded workflow files, and the file passes markdownlint.

## 5. Validate end-to-end and run readiness checks

Run all markdownlint and structural checks, and verify the spec's acceptance criteria are satisfied by the produced artifacts.

- [x] Every `template` path in `framework/workflows/registry.json` points to an existing file under `framework/workflows/`
- [x] Every category value in the registry is in the fixed set
- [x] Every `trigger.field` value in the registry is in the recognized set
- [x] `npx markdownlint-cli2` passes on all created/modified `.md` files (workflow files, init, govern, plan, tasks, data-model)
- [x] `python -m json.tool framework/workflows/registry.json` (or equivalent JSON validator) succeeds
- [x] Each acceptance criterion in `spec.md` is checked individually against the produced artifacts and marked `- [x]` only if satisfied

**Done when:** all checks pass, the registry and workflow files are mutually consistent, and every acceptance criterion is satisfied.

## 6. Cross-spec rename: "skills" → "workflows"

Driven by [010-agent-autonomy](../010-agent-autonomy/spec.md). 010's "skills" capability adopts Anthropic/Claude Code terminology for context-loaded instruction packs, which conflicts with 005's prior use of "skills" for tech-stack-conditional development workflows (lint, test, format, migrate). To free the term, rename 005's internal concept to "workflows" throughout governance code and prose, and flatten the framework directory (the inner `templates/` becomes redundant once the parent already says "workflows"). Implementation is performed by 010's `/gov:implement` pass; this task tracks completion from 005's side.

- [x] `framework/skills/` renamed to `framework/workflows/` and flattened (registry + nine workflow files at the same level, no inner `templates/`)
- [x] `specs/005-skills-and-plugins/` renamed to `specs/005-workflows/`
- [x] `framework/bootstrap/govern.md` manifest, recommendation step, and prose updated to use "workflows" / `workflows/` paths
- [x] `.claude/commands/gov/init.md` recommendation step paths and prose updated (hand-maintained, generator skips)
- [x] `framework/bootstrap/configure/claude.md` "Bash commands used by skills" comment label updated to "workflows"
- [x] 005's own artifacts (`spec.md`, `plan.md`, `tasks.md`, `data-model.md`) updated for terminology and renamed paths
- [x] `specs/013-text-first-artifacts/plan.md` one-row migration entry updated to the new spec dir path
- [x] `README.md` references to 005's "skills" feature updated to "workflows"
- [x] `.claude/commands/gov/configure.md` regenerated from updated source via `./scripts/gen-claude-commands.sh`
- [x] `npx markdownlint-cli2` passes on all modified `.md` files
- [x] After 010's implementation completes, advance 005 from `in-progress` back to `done` via a separate `/gov:implement` pass

**Done when:** the rename is complete and consistent across governance code and 005's artifacts, the new acceptance criterion in `spec.md` is verifiable, and 005 is ready for re-advancement to `done`.
