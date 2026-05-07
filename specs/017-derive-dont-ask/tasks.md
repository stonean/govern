---
title: "017-derive-dont-ask — tasks"
---

# 017 — Derive, Don't Ask Tasks

Tasks derived from the [plan](plan.md). Complete in order.

The work is grouped into seven phases. Each phase ends with a natural commit boundary. Tasks within a phase can usually be done in one session; cross-phase dependencies are noted at the phase level.

## Phase 1 — New rule file

### 1. Create `framework/rules/configuration.md`

- [x] Author the file with the seven initial rules from `data-model.md` (`CFG-CONST-001..003`, `CFG-ENV-001..004`)
- [x] Each rule has Statement (RFC 2119), Rationale, Verification, optional Source
- [x] Lint passes

Done when: `framework/rules/configuration.md` exists, `npx markdownlint-cli2` passes, and the rule format matches `data-model.md`.

## Phase 2 — Constitution and project root

### 2. Update `framework/constitution.md`

- [x] Remove `tags` row from the Spec files frontmatter table
- [x] Remove `tags` row from the Scenario files frontmatter table
- [x] Remove the entire "Starter Tag Vocabulary" subsection
- [x] Remove the `[simple]` marker bullet from §cost-levers
- [x] Replace the §constants section body with a one-line pointer: "See `framework/rules/configuration.md` (CFG-CONST-NNN rules)."
- [x] Replace the §env-vars section body with a one-line pointer: "See `framework/rules/configuration.md` (CFG-ENV-NNN rules)."
- [x] Update §text-first-artifacts to declare `section` as the scenario hard-fail field (replacing `spec-ref`); add the canonical-source row for configuration rules to the §drift-prevention table; do not list `tags` as advisory anymore
- [x] Lint passes

Done when: constitution body reflects the schema and rule changes; no references to `tags` (as a frontmatter field), `[simple]`, or `spec-ref` remain; `npx markdownlint-cli2` passes.

### 3. Delete root `constitution.md` and update root pointers

- [x] Delete root `constitution.md` (collapse per spec Q2)
- [x] Update root `CLAUDE.md`: change `@import constitution.md` to `@import framework/constitution.md`
- [x] Update root `README.md`: every link to `constitution.md` already targets `framework/constitution.md`; no change needed (bare references in lines 141/162/187/258/276 are adopter-context documentation where the file lives at the adopter's root)
- [x] Lint passes on README.md and CLAUDE.md

Done when: root `constitution.md` no longer exists; all references resolve to `framework/constitution.md`.

### 4. Update root `AGENTS.md`

- [ ] Remove the "mirror constitutions" Workflow bullet (the two-line bullet about mirroring root and framework constitutions)
- [ ] Remove the "After editing any file under `framework/commands/`… run `./scripts/gen-claude-commands.sh`" bullet (the hook will handle this automatically)
- [ ] Lint passes

Done when: AGENTS.md Workflow section no longer references either discipline-required step.

## Phase 3 — Templates

### 5. Strip `title:` from all spec-pipeline templates

- [ ] `framework/templates/spec/spec.md` — remove `title:` line from frontmatter
- [ ] `framework/templates/spec/plan.md` — remove `title:` line; remove the entire "Open Questions Resolved" section
- [ ] `framework/templates/spec/tasks.md` — remove `title:` line; remove the `[simple]` marker documentation comment block
- [ ] `framework/templates/spec/data-model.md` — remove `title:` line
- [ ] `framework/templates/spec/research.md` — remove `title:` line
- [ ] Lint passes on all five files

Done when: none of the five templates have a `title:` field in frontmatter; tasks.md has no `[simple]` documentation; plan.md has no "Open Questions Resolved" section.

### 6. Update `framework/templates/spec/spec-and-plan.md`

- [ ] Remove `title:` line from frontmatter
- [ ] Remove `track: lightweight` line from frontmatter
- [ ] Remove the explanatory comment about the `track` field
- [ ] Lint passes

Done when: the template has neither `title:` nor `track:` and no comment about either.

### 7. Update `framework/templates/spec/scenario.md`

- [ ] Remove `title:` line from frontmatter
- [ ] Replace `spec-ref:` line with `section:` (the parent feature is implicit in the file path)
- [ ] Update the explanatory comment to reflect the rename
- [ ] Lint passes

Done when: scenario template uses `section:` only; no `title:` or `spec-ref:` remain.

## Phase 4 — Generators, hooks, and CI scaffolding

### 8. Implement `scripts/gen-spec-deps.sh`

- [ ] Walk every file matching `specs/*/spec.md` and `specs/*/spec-and-plan.md`
- [ ] Parse markdown body, ignoring fenced code blocks
- [ ] Find inline links matching sibling-spec patterns (`../NNN-feature/...` or `(specs/NNN-feature/...)`)
- [ ] Compute the union of slugs and rewrite the frontmatter `dependencies` list (sorted; empty list rendered as `[]`)
- [ ] Idempotent: running twice produces the same output
- [ ] Bash-portable (no GNU-isms in macOS BSD utilities)
- [ ] Exit non-zero with a clear error if any spec file's frontmatter is malformed

Done when: running `./scripts/gen-spec-deps.sh` against the current repo produces a valid result; second run produces no diff; the script is executable.

### 9. Implement `scripts/gen-readme-table.sh`

- [ ] Find marker comments in `README.md`: `<!-- generated:feature-specs:start -->` and `<!-- generated:feature-specs:end -->`
- [ ] For each `specs/*/spec*.md`, parse frontmatter (`status`, `dependencies`) and the first body paragraph (description)
- [ ] Emit a markdown table sorted by feature number between the markers
- [ ] Exit non-zero if markers are absent
- [ ] Idempotent

Done when: running the script populates the README table from spec frontmatter; second run produces no diff.

### 10. Implement `scripts/gen-help-tables.sh`

- [ ] Find marker comments in `framework/commands/help.md` for each of the five command groups (`commands-pipeline`, `commands-elaborate`, `commands-brownfield`, `commands-orient`, `commands-bootstrap`)
- [ ] For each group, list the commands belonging to that group (hardcoded grouping in the generator) and read each command file's frontmatter `description:`
- [ ] Emit a markdown table per group between its markers
- [ ] Exit non-zero if any expected marker pair is absent or any referenced command file is missing
- [ ] Idempotent

Done when: running the script populates all five tables in `help.md` from command frontmatter; second run produces no diff.

### 11. Add marker comments to `README.md` and `framework/commands/help.md`

- [ ] Insert `<!-- generated:feature-specs:start -->` and `<!-- generated:feature-specs:end -->` around the existing Feature Specs table in `README.md`
- [ ] Insert the five marker pairs in `help.md` around the corresponding tables (see `data-model.md` § Marker comment names)
- [ ] Run the new generators to confirm they recognize the markers and produce correct output
- [ ] Lint passes (HTML comments are allowed per project markdownlint config)

Done when: both files have valid marker pairs; running the generators produces no diff.

### 12. Implement `.githooks/pre-commit` and `scripts/install-hooks.sh`

- [ ] Create `.githooks/` directory and `pre-commit` script
- [ ] Hook calls all four generators in order: `gen-claude-commands.sh`, `gen-readme-table.sh`, `gen-help-tables.sh`, `gen-spec-deps.sh`
- [ ] After generators run, `git add -A` the staged files (only files that were already part of the commit, plus any generator outputs)
- [ ] Hook is executable (`chmod +x`)
- [ ] Create `scripts/install-hooks.sh` that runs `git config core.hooksPath .githooks` (idempotent)
- [ ] Both scripts begin with `#!/usr/bin/env bash` and `set -euo pipefail`

Done when: running `./scripts/install-hooks.sh` configures `core.hooksPath` correctly; making a no-op commit does not produce errors; modifying a command source produces an updated `.claude/commands/gov/*.md` automatically on commit.

### 13. Implement `framework/bootstrap/hooks/pre-commit` and `framework/bootstrap/hooks/install.sh`

- [ ] Create the shipped adopter hook: calls `scripts/gen-spec-deps.sh` only
- [ ] First line after shebang is the sentinel comment: `# managed-by: govern`
- [ ] Create the shipped install script: idempotent `git config core.hooksPath .githooks`
- [ ] Both scripts use `#!/usr/bin/env bash` and `set -euo pipefail`
- [ ] Both are executable

Done when: the shipped scripts exist and would correctly install in an adopter project.

### 14. Add CI workflow for govern repo

- [ ] Create `.github/workflows/generators.yml`
- [ ] On `pull_request`, run all four generators and `git diff --exit-code`
- [ ] Fail the build on non-empty diff
- [ ] Use ubuntu-latest with bash

Done when: the workflow runs locally via `act` (or equivalent) and fails on a synthetic stale-commit; passes on a clean working tree after generators ran.

### 15. Ship adopter CI template

- [ ] Create `framework/templates/ci/adopter-generators.yml`
- [ ] Document in the file header that adopters copy this into their own `.github/workflows/`
- [ ] Reference `scripts/gen-spec-deps.sh` (the shipped generator name)
- [ ] Add a one-paragraph note to the adopter README pointing at the template

Done when: the template exists and documents adopter usage.

## Phase 5 — Commands

### 16. Update `framework/commands/specify.md`

- [ ] Remove the entire "Tag prompt" step (step 4)
- [ ] Renumber remaining steps
- [ ] Remove the `title` placeholder fill-in instruction; replace with: "no `title:` field is written"
- [ ] Remove all references to `tags` in the frontmatter writing instructions
- [ ] Update the lightweight-track detection step to not reference `track:` (filename detection is the source of truth)
- [ ] Lint passes

Done when: `/specify` writes a spec with only `status` and `dependencies` (initially empty list) in frontmatter.

### 17. Update `framework/commands/capture.md`

- [ ] Remove the `title` placeholder fill-in instruction
- [ ] Remove the `tags` field reference (currently "Leave frontmatter `tags` as `[]`…")
- [ ] Lint passes

Done when: `/capture` writes a spec with only `status` and `dependencies` in frontmatter.

### 18. Update `framework/commands/clarify.md`

- [ ] Remove the `title` check from the validation gate
- [ ] Remove the `tags` advisory from the validation gate
- [ ] Add a new "Cross-spec scan" step (after question resolution, before the validation gate): scan body for inline markdown links to sibling specs not in the current `dependencies`; surface the list as informational ("These will be added to dependencies on commit by gen-spec-deps.sh")
- [ ] Add a "Recompute dependencies" step at the very start of Setup: run `gen-spec-deps.sh` against the target spec for the safety-net path; treat as idempotent if the hook already synced
- [ ] Lint passes

Done when: `/clarify` performs cross-spec scan and dep recompute; no `title` or `tags` checks remain.

### 19. Update `framework/commands/plan.md`

- [ ] Remove the `title` placeholder fill-in instructions for plan.md and data-model.md
- [ ] Remove the "Open Questions Resolved" reference from the plan body fill-in step
- [ ] Remove step 3 of "Create the task breakdown" (the `[simple]` marker proposal)
- [ ] Reframe the Affected Files section instructions: it is a planning aid, not the authoritative implement-time write boundary
- [ ] Add a "Cross-spec scan" step after creating the plan: surface inline body links not in dependencies (informational)
- [ ] Add a "Recompute dependencies" step at Setup: same as `/clarify`
- [ ] Update the Finalize step to not list `[simple]` markers in the summary
- [ ] Lint passes

Done when: `/plan` does none of the discipline-trap steps; the cross-spec scan and dep recompute are present.

### 20. Update `framework/commands/implement.md`

- [ ] Remove the `[simple]` marker reading and surfacing logic (in Setup or Walk through tasks)
- [ ] Replace the Affected Files write-boundary section: the boundary is derived at Setup from `git log specs/{feature}/ | tail -1` (first commit on spec dir) and `git diff --name-only {first-commit}..HEAD` (filtered to files outside `specs/{feature}/`). The plan's Affected Files section is read for the *initial* expected set; the runtime boundary is the union of the two
- [ ] Replace the existing "If you need to modify files outside the plan's affected files list, notify the user, explain why, and add the file to the plan's Affected Files section" with: "If a write would land outside the runtime boundary (plan-expected ∪ git-derived), notify the user. The user accepts (the file enters the boundary for the rest of the session) or rejects (revert)."
- [ ] Remove the "add to plan's Affected Files section" backfill step
- [ ] Add a Cross-spec scan step at Completion: `git diff --name-only {first-commit}..HEAD` filtered to files in `specs/` outside the target dir; prompt user to confirm whether changes should be recorded as new acceptance criteria or scenarios in the affected spec
- [ ] Add a "Recompute dependencies" step at Setup
- [ ] Lint passes

Done when: `/implement` derives the boundary from git, no longer asks the author to backfill the plan, and surfaces cross-spec changes at completion.

### 21. Update `framework/commands/elaborate.md`

- [ ] Remove the `title` placeholder fill-in instruction (currently step 2 of "Create the scenario file")
- [ ] Change the `spec-ref` writing instruction to `section:` — write only the section name, not the parent-feature-prefixed string
- [ ] Add "Recompute dependencies" step at the start of Confirm target
- [ ] Lint passes

Done when: `/elaborate` writes a scenario with `section:` only and no `title`; deps recompute at entry.

### 22. Update `framework/commands/groom.md`

- [ ] Remove the `[promote-to-rule]` prefix instruction in Step 1 of the bug decision tree
- [ ] Replace with: "If the item qualifies for promotion to a rule but no rule file covers the domain, leave it in the inbox unmodified — every subsequent groom pass walks every unmigrated item, including this one."
- [ ] Lint passes

Done when: groom no longer asks the agent to mark items with a discipline-required prefix.

### 23. Update `framework/commands/validate.md`

- [ ] Remove the entire "PKM title field (advisory)" section
- [ ] Remove the `tags` advisory from the "Frontmatter schema (advisory)" section (the section becomes empty — remove it as well)
- [ ] Rename the `spec-ref` check in "Frontmatter schema (hard fail)" to `section`
- [ ] Update help-equivalence check to be a dry-run-of-generator check: run `gen-help-tables.sh --dry-run` (or equivalent) and report if it would produce a diff
- [ ] Add a new advisory entry: when `gen-spec-deps.sh --dry-run` would change the target spec's frontmatter, emit "Body links and frontmatter dependencies are out of sync; the next commit will resolve."
- [ ] Add `framework/rules/configuration.md` to the rule-file list (the section that loads `specs/security-backend.md`, etc.)
- [ ] Remove the entire "Fix Mode" section
- [ ] Remove the `--fix` flag from the file's frontmatter `argument-hint` and from the Context section's flag parsing
- [ ] Update Scope Boundaries to remove the `--fix` write permission
- [ ] Lint passes

Done when: `/validate` no longer offers `--fix`; PKM title and tags advisories are gone; `section` replaces `spec-ref`; the configuration rule file is loaded; help-equivalence is structural.

### 24. Update `framework/commands/ask.md`

- [ ] Add a "Recompute dependencies" step in Confirm target (run `gen-spec-deps.sh --dry-run` against the target spec; if it would change anything, run it for real)
- [ ] Lint passes

Done when: `/ask` recomputes deps on entry.

### 25. Update `framework/commands/target.md`

- [ ] Remove the `tags` parse from step 4 of "With arguments — set target"
- [ ] Add a "Recompute dependencies" step in step 4 (between frontmatter parse and artifact-existence check)
- [ ] Lint passes

Done when: `/target` no longer reads `tags`; deps recompute happens at entry.

### 26. Add marker comments to `framework/commands/help.md`

- [ ] Insert the five marker pairs around the five command tables (Pipeline, Elaborate, Brownfield, Orient, Bootstrap)
- [ ] Run `gen-help-tables.sh` and verify the file matches generator output
- [ ] Lint passes

Done when: help.md has all five marker pairs and matches generator output.

## Phase 6 — Bootstrap installer

### 27. Update `framework/bootstrap/govern.md` for adopter hook installation

- [ ] Add a new top-level section "Hook Installation" between "Per-Agent Scaffolding" and "Post-Scaffolding Output"
- [ ] Document the four detection states and actions (per plan's "Adopter hook surface" table)
- [ ] Sentinel-based detection: if `.githooks/pre-commit` exists, check for `# managed-by: govern` to distinguish managed from hand-rolled
- [ ] Husky detection: presence of `.husky/` directory
- [ ] Lefthook detection: presence of `lefthook.yml` or `lefthook-local.yml`
- [ ] Pre-commit-py detection: presence of `.pre-commit-config.yaml`
- [ ] Add `framework/rules/configuration.md` → `specs/configuration.md` to the "govern-owned shared files (strategy: update)" manifest
- [ ] Add `framework/bootstrap/hooks/pre-commit` → `.githooks/pre-commit` to the manifest with `update` strategy (subject to sentinel detection)
- [ ] Add `framework/bootstrap/hooks/install.sh` → `.githooks/install.sh` with `create` strategy
- [ ] Add `scripts/gen-spec-deps.sh` → `scripts/gen-spec-deps.sh` with `create` strategy
- [ ] Update Post-Scaffolding Output to report hook install/update/skip status per agent (or per-project once for shared files)
- [ ] Update the "What This Command Does NOT Do" section if behavior changed
- [ ] Lint passes

Done when: `/govern` ships the new files and manages the adopter hook with the documented detection logic.

### 28. Update permission files

- [ ] `framework/bootstrap/configure/claude.md` — add `Bash(git config *)`, `Bash(.githooks/*)`, `Bash(scripts/gen-*)` to the permission set
- [ ] `framework/bootstrap/configure/auggie.md` — same permissions in Auggie's format
- [ ] Run `./scripts/gen-claude-commands.sh` to regenerate `.claude/commands/gov/configure.md` (the generator handles the substitution)
- [ ] Lint passes

Done when: adopters scaffolding via `/govern` get the necessary Bash permissions for hook operations without prompts.

## Phase 7 — Migration, regeneration, and self-cleanup

### 29. Migrate existing dogfood specs' body inline links

- [ ] For each spec at `specs/000-016/spec.md` (and `spec-and-plan.md`), read the frontmatter `dependencies` list
- [ ] Scan the body for inline markdown links to each declared dependency (`../NNN-feature/...` or `(specs/NNN-feature/...)`) outside fenced code blocks
- [ ] For each dependency without an inline link, append a "References" section at the end of the body (or the end of an existing References section if present) with a bullet linking the dep
- [ ] Confirm `gen-spec-deps.sh` against each migrated spec produces no change to its frontmatter `dependencies` list
- [ ] Lint passes on every modified file

Done when: every existing dependency in every spec is reflected in the body via an inline link, AND running `gen-spec-deps.sh` against the entire repo produces no frontmatter changes for specs 000–016.

### 30. Run all generators and commit derived state

- [ ] Run `./scripts/install-hooks.sh` to set `core.hooksPath`
- [ ] Run all four generators by hand: `./scripts/gen-claude-commands.sh && ./scripts/gen-readme-table.sh && ./scripts/gen-help-tables.sh && ./scripts/gen-spec-deps.sh`
- [ ] Verify each generator exits 0
- [ ] `git diff` — review the changes (mostly README table, help.md tables, possibly some `.claude/commands/gov/*.md`)
- [ ] Run `npx markdownlint-cli2` across the repo
- [ ] Commit the derived state

Done when: working tree is clean after running every generator a second time.

### 31. Verify `/validate` runs cleanly on every existing spec

- [ ] Run `/gov:validate --all` against the repo
- [ ] Confirm no new findings introduced by schema changes for specs 000–016 (stale fields ignored per open-schema rule)
- [ ] Confirm 017 itself produces only expected findings (e.g., its own `tags` and `title`, which the final task removes)
- [ ] Address any unexpected findings before proceeding

Done when: validate runs cleanly against the whole repo modulo expected pre-final-task findings on 017.

### 32. Strip `title:` and `tags:` from this spec's own artifacts [simple]

- [ ] `specs/017-derive-dont-ask/spec.md` — remove `title:` and `tags:` lines from frontmatter
- [ ] `specs/017-derive-dont-ask/plan.md` — remove `title:` line
- [ ] `specs/017-derive-dont-ask/tasks.md` — remove `title:` line
- [ ] `specs/017-derive-dont-ask/data-model.md` — remove `title:` line
- [ ] Re-run `/gov:validate` to confirm no findings

Done when: this spec's artifacts have no `title:` or `tags:` frontmatter; validate runs clean.
