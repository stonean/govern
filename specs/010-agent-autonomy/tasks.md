---
title: "010-agent-autonomy — tasks"
---

# 010 — Agent Autonomy Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Add `[simple]` marker convention to the tasks template

Document the optional inline `[simple]` task header marker in `framework/templates/spec/tasks.md` so future feature tasks can use it.

- [x] Add a short paragraph (or comment block) explaining the `[simple]` marker convention: header placement (`## N. Title [simple]`), no marker = default tier, only one tier defined, marker lives on individual task headers
- [x] Include an inline example showing a `[simple]`-tagged task next to an unmarked task
- [x] File passes `npx markdownlint-cli2`

**Done when:** a future reader of `tasks.md` template understands when and how to use `[simple]`, with an example to copy from.

## 2. Update `/gov:plan` to propose `[simple]` markers

Modify `framework/commands/plan.md` to add a step that scans generated tasks and proposes `[simple]` markers on trivial ones.

- [x] In the "Create the task breakdown" section, add a sub-step after task generation: scan each task, append `[simple]` to the header if the task is trivial (single small file, no logic, no schema change, no new behavior)
- [x] Surface the marker proposals in the summary shown to the user before status transition (so the user can add, remove, or accept markers before approving)
- [x] Cross-reference the `[simple]` marker convention in the tasks template
- [x] File passes `npx markdownlint-cli2`

**Done when:** the plan command has explicit instructions for proposing `[simple]` markers and surfacing them for user review.

## 3. Update `/gov:implement` with stuck detection

Modify `framework/commands/implement.md` to add a setup-time stuck-detection check.

- [x] In the "Setup" section, add a step (before walking tasks) that runs `git log --oneline -- specs/{feature}/tasks.md` and counts commits since the spec entered `in-progress`
- [x] Detection rule: if ≥ 3 commits AND the same task remains the next incomplete one, surface the cycle to the user with a decomposition suggestion and pause — do not auto-decompose
- [x] Document that stuck-detection events fire even with `--auto` set ("auto mode does not power through cycles")
- [x] Document the threshold of 3 as a fixed constant (not configurable in v1)
- [x] File passes `npx markdownlint-cli2`

**Done when:** the implement command has explicit instructions for the stuck-detection check, including the algorithm, threshold, and behavior under `--auto`.

## 4. Update `/gov:implement` with `--auto` flag

Modify `framework/commands/implement.md` to accept and document the `--auto` flag.

- [x] In the "Context" section, document `--auto` as a recognized flag (stripped from `$ARGUMENTS` before the remaining text is treated as a feature override)
- [x] In the "Walk through tasks" section, document that `--auto` skips the per-task "prompt the user to commit and push changes" confirmation
- [x] List the gates that still fire even with `--auto` on: phase transitions (`planned`→`in-progress`, `in-progress`→`done`), stuck-detection events, out-of-bounds file writes, spec/plan edits, mid-implement task discovery, risky actions (destructive ops, secrets, force pushes)
- [x] Document the auto-mode commit policy: commit, do not push (push remains gated)
- [x] Confirm the session file is unchanged (no `autoAdvance` field)
- [x] File passes `npx markdownlint-cli2`

**Done when:** the implement command documents `--auto` as a per-invocation flag with the listed gate set and commit policy.

## 5. Add `## Skills` index section to `AGENTS.md` template

Modify `framework/templates/project/agents.md` to insert an optional `## Skills` section after `## Project Structure` and before `## Code Style`.

- [x] Insert empty `## Skills` section with an HTML-comment guide explaining what skills are (Anthropic/Claude Code "skills" — context-loaded instruction packs), when to populate the section, and a copy-friendly example table of skill files and their activation conditions
- [x] Make explicit that per-platform mapping (Claude Code skills, Cursor rules, etc.) is the adopter's call — governance defines the index pattern, not the location
- [x] Section is empty by default (backwards-compatible for projects that don't decompose)
- [x] File passes `npx markdownlint-cli2`

**Done when:** the template has the new section in the correct location, with the in-place guide, and adopters can populate it without re-reading governance docs.

## 6. Add `### Cost levers` subsection to the constitution

Modify `framework/constitution.md` to add a new subsection immediately after the `### Business` principles list.

- [x] Insert `### Cost levers` heading after the `### Business` list
- [x] Paragraph names governance's existing cost levers: lightweight track (§lightweight-track), `[simple]` marker (010), stuck detection (010), default-off autonomy (010)
- [x] Paragraph points at the adopter's platform tooling for runtime cost controls (Claude Code's `/cost`, Anthropic usage dashboard, Cursor's request limits — examples, not commitments)
- [x] Paragraph stays short (4–6 sentences); reads as guidance, not a manual
- [x] File passes `npx markdownlint-cli2`

**Done when:** the constitution has the new subsection in the correct location, listing all four levers and pointing at platform tooling.

## 7. Add `### Concurrent Features` subsection to the constitution

Modify `framework/constitution.md` to add a new subsection under `## Pipeline Boundaries`.

- [x] Insert `### Concurrent Features` heading under `## Pipeline Boundaries`
- [x] Paragraph states the session file holds a single target by design
- [x] Paragraph directs users to `git worktree` and platform isolation features (Claude Code's `isolation: "worktree"` agent parameter, Cursor's worktree integration) for concurrent feature work
- [x] Paragraph stays short (3–5 sentences)
- [x] File passes `npx markdownlint-cli2`

**Done when:** the constitution has the new subsection in the correct location, communicating the single-target invariant and the platform/git workaround for concurrent work.

## 8. Reopen 005 and record the cross-spec rename

Update `specs/005-workflows/spec.md` (the spec dir is renamed in task 9; for this task it is still at `specs/005-skills-and-plugins/`) to reopen the spec and record the new acceptance criterion before performing any rename work.

- [x] Set 005's frontmatter `status` from `done` to `in-progress`
- [x] Add a new acceptance criterion to 005's spec: "Rename internal terminology from 'skills' to 'workflows' to free the term 'skills' for Anthropic-style context-loaded instruction packs (signpost: driven by 010-agent-autonomy)."
- [x] Add a new task to 005's tasks.md for the cross-spec rename, marked as carried out by 010's implementation
- [x] 005 spec.md and tasks.md both pass `npx markdownlint-cli2`

**Done when:** 005 is reopened to `in-progress` with the new acceptance criterion and a corresponding task entry, before any rename is applied.

## 9. Rename `framework/skills/` to `framework/workflows/` (flatten) and `specs/005-skills-and-plugins/` to `specs/005-workflows/`

Perform both directory moves. The framework directory is also flattened: the inner `templates/` directory is eliminated and the registry + nine workflow files sit at the same level.

- [x] `mv framework/skills framework/workflows` (note: `git mv` is denied by configure permissions; use plain `mv` and let `git add` detect the rename)
- [x] Move the nine `.md` files out of `framework/workflows/templates/` up to `framework/workflows/` and remove the inner `templates/` directory
- [x] `mv specs/005-skills-and-plugins specs/005-workflows`
- [x] Verify `framework/workflows/registry.json` exists and the nine workflow files sit alongside it
- [x] Confirm no path strings inside the workflow files reference the old `framework/skills/` path (files use `{project}` and `{cli-config-dir}` placeholders only)
- [x] All workflow `.md` files still pass `npx markdownlint-cli2` after the move

**Done when:** both directories are renamed, the framework workflows directory is flattened, and no internal paths still point to the old names.

## 10. Update `framework/bootstrap/govern.md` for the rename

Update the manifest, recommendation step, and all prose in `framework/bootstrap/govern.md` from "skills" / "skill" to "workflows" / "workflow" wherever the term refers to 005's concept.

- [x] Update the manifest row: `framework/skills/registry.json` → `framework/workflows/registry.json` and `skills/registry.json` → `workflows/registry.json` (project-side)
- [x] Update the recommendation-step path references to read from `workflows/registry.json` and fetch from `framework/workflows/{entry.template}` (note: flattened — no inner `templates/`)
- [x] Update scaffold destination from `{config_dir}/commands/{project}/skills/{entry.template}` to `{config_dir}/commands/{project}/workflows/{entry.template}`
- [x] Update the section heading from "Skill recommendation" to "Workflow recommendation" and update prose throughout (warning messages, summary lines, discovery note for Auggie)
- [x] Update the slash-command-cleanup edge-case note to reference the new directory name (no explicit edge-case note found in govern.md; cleanup walks top-level files only, so subdirectory immunity is implicit and documented in the recommendation step itself)
- [x] Update the schema reference from `specs/005-skills-and-plugins/data-model.md` to `specs/005-workflows/data-model.md`
- [x] Add a one-line migration note: adopters who already ran `/gov:govern` should manually delete the old `skills/` directory after re-running govern
- [x] File passes `npx markdownlint-cli2`

**Done when:** govern.md has no references to "skills" as 005's concept; all paths use `workflows`; markdownlint passes.

## 11. Update `.claude/commands/gov/init.md` for the rename

Hand-edit `init.md` (the generator skips it). Update the recommendation step's paths and prose.

- [x] Update path references: `framework/skills/registry.json` → `framework/workflows/registry.json`, `framework/skills/templates/{entry.template}` → `framework/workflows/{entry.template}` (flattened)
- [x] Update scaffold destination from `.claude/commands/{slug}/skills/` → `.claude/commands/{slug}/workflows/`
- [x] Update the step heading from "Recommend and scaffold skills" to "Recommend and scaffold workflows"
- [x] Update prose (warning messages, summary lines) throughout
- [x] Update the schema reference from `specs/005-skills-and-plugins/data-model.md` to `specs/005-workflows/data-model.md`
- [x] File passes `npx markdownlint-cli2`

**Done when:** init.md has no references to "skills" as 005's concept; all paths use `workflows`; markdownlint passes.

## 12. Update `framework/bootstrap/configure/claude.md` for the rename

Replace the "skills" term in the configure source.

- [x] Change the comment label `**Bash commands used by skills (read-only shell operations):**` to `**Bash commands used by workflows (read-only shell operations):**`
- [x] File passes `npx markdownlint-cli2`

**Done when:** configure source no longer uses "skills" for 005's concept.

## 13. Update 005's spec, plan, tasks, and data-model prose

Update prose and titles in 005's artifacts to reflect the new terminology and renamed paths.

- [x] `specs/005-workflows/spec.md`: title `005 — Skills and Plugins` → `005 — Workflows`; replace "skill" / "skills" terminology in body, acceptance criteria, and resolved questions where it refers to 005's concept; preserve the AC added in task 8
- [x] `specs/005-workflows/plan.md`: title rename; prose updates; affected-files table updated to use `framework/workflows/` (flattened) paths; trade-off section terminology updated
- [x] `specs/005-workflows/tasks.md`: title rename; prose updates; checkbox state preserved; the task added in task 8 is preserved
- [x] `specs/005-workflows/data-model.md`: terminology updated in schema description (entry name fields, examples)
- [x] `specs/013-text-first-artifacts/plan.md`: update the one-row migration entry that references `specs/005-skills-and-plugins/spec.md` to the new path
- [x] All five files pass `npx markdownlint-cli2`

**Done when:** 005's artifacts use "workflow" / "workflows" consistently for 005's concept, with renamed paths reflected in tables; 013 references the new spec dir; markdownlint passes.

## 14. Update `README.md` references

Update any references to 005's "skills" feature in the top-level `README.md`.

- [x] Search `README.md` for "skill" references (case-insensitive); update those that refer to 005's concept to "workflows"; leave alone any references to Anthropic/Claude Code "skills" in the 010 sense (if any exist)
- [x] File passes `npx markdownlint-cli2`

**Done when:** README.md no longer uses "skills" for 005's concept.

## 15. Regenerate `.claude/commands/gov/*.md` from sources

Run the generator after all source-side edits to `framework/commands/` and `framework/bootstrap/configure/claude.md` are complete.

- [x] Run `./scripts/gen-claude-commands.sh`
- [x] Verify `.claude/commands/gov/plan.md` reflects the `[simple]` marker step
- [x] Verify `.claude/commands/gov/implement.md` reflects stuck detection and `--auto` flag
- [x] Verify `.claude/commands/gov/configure.md` reflects the renamed comment label
- [x] Verify `.claude/commands/gov/init.md` is untouched (hand-maintained)
- [x] All regenerated files pass `npx markdownlint-cli2`

**Done when:** the regenerated files match the sources, init.md is preserved, and markdownlint passes.

## 16. Validate end-to-end and verify acceptance criteria

Run all checks and walk through 010's acceptance criteria.

- [x] `npx markdownlint-cli2` passes on all created/modified `.md` files in this feature
- [x] `python -m json.tool framework/workflows/registry.json` (or equivalent JSON validator) succeeds — registry still parses after the rename
- [x] Every `template` path in the registry resolves to an existing file at `framework/workflows/{template}` (flattened)
- [x] No remaining references to `framework/skills/` exist anywhere except in:
  - The 005-rename signpost AC (referencing the old term as historical context)
  - Any prose specifically discussing the rename itself
- [x] No remaining references to `specs/005-skills-and-plugins/` exist anywhere except in commit messages or prose specifically discussing the rename
- [x] `grep -rn "skill" --include="*.md" .` produces no hits referring to 005's concept (only Anthropic-style skills references in `AGENTS.md` template, constitution Cost levers paragraph, and 010's spec/plan/tasks)
- [x] Each acceptance criterion in 010's `spec.md` is checked individually against the produced artifacts and marked `- [x]` only if satisfied
- [x] Cross-spec deliverable AC reflects 005 reopened to `in-progress` (005 is left at `in-progress` for a separate `/gov:implement` to advance back to `done`)

**Done when:** all checks pass, the rename is complete and consistent, and every 010 acceptance criterion is verified.
