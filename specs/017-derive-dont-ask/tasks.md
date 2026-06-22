# 017 — Derive, Don't Ask Tasks

Tasks derived from the [plan](plan.md). Complete in order.

The work is grouped into seven phases. Each phase ends with a natural commit boundary. Tasks within a phase can usually be done in one session; cross-phase dependencies are noted at the phase level.

## Phase 1 — New rule file

### 1. Create `framework/rules/configuration-cross.md`

- [x] Author the file with the seven initial rules from `data-model.md` (`CFG-CONST-001..003`, `CFG-ENV-001..004`)
- [x] Each rule has Statement (RFC 2119), Rationale, Verification, optional Source
- [x] Lint passes

Done when: `framework/rules/configuration-cross.md` exists, `npx markdownlint-cli2` passes, and the rule format matches `data-model.md`.

## Phase 2 — Constitution and project root

### 2. Update `framework/constitution.md`

- [x] Remove `tags` row from the Spec files frontmatter table
- [x] Remove `tags` row from the Scenario files frontmatter table
- [x] Remove the entire "Starter Tag Vocabulary" subsection
- [x] Remove the `[simple]` marker bullet from §cost-levers
- [x] Replace the §constants section body with a one-line pointer: "See `framework/rules/configuration-cross.md` (CFG-CONST-NNN rules)."
- [x] Replace the §env-vars section body with a one-line pointer: "See `framework/rules/configuration-cross.md` (CFG-ENV-NNN rules)."
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

- [x] Remove the "mirror constitutions" Workflow bullet (the two-line bullet about mirroring root and framework constitutions)
- [x] Remove the "After editing any file under `framework/commands/`… run `./scripts/gen-claude-commands.sh`" bullet (the hook will handle this automatically)
- [x] Lint passes

Done when: AGENTS.md Workflow section no longer references either discipline-required step.

## Phase 3 — Templates

### 5. Strip `title:` from all spec-pipeline templates

- [x] `framework/templates/spec/spec.md` — remove `title:` and `tags:` lines from frontmatter; update body comment that mentioned tags; reframe dependencies as generated
- [x] `framework/templates/spec/plan.md` — remove entire frontmatter (only had `title`); remove "Open Questions Resolved" section; reframe Affected Files as planning aid in the comment
- [x] `framework/templates/spec/tasks.md` — remove entire frontmatter; remove `[simple]` marker documentation comment block; remove `[simple]` from the example
- [x] `framework/templates/spec/data-model.md` — remove entire frontmatter
- [x] `framework/templates/spec/research.md` — remove entire frontmatter
- [x] Lint passes on all five files

Done when: none of the five templates have a `title:` field in frontmatter; tasks.md has no `[simple]` documentation; plan.md has no "Open Questions Resolved" section.

### 6. Update `framework/templates/spec/spec-and-plan.md`

- [x] Remove `title:` line from frontmatter
- [x] Remove `tags:` line from frontmatter
- [x] Remove `track: lightweight` line from frontmatter
- [x] Remove the explanatory comment about the `track` field
- [x] Lint passes

Done when: the template has neither `title:` nor `track:` and no comment about either.

### 7. Update `framework/templates/spec/scenario.md`

- [x] Remove `title:` line from frontmatter
- [x] Replace `spec-ref:` line with `section:` (the parent feature is implicit in the file path)
- [x] Remove `tags:` line from frontmatter
- [x] Lint passes

Done when: scenario template uses `section:` only; no `title:`, `tags:`, or `spec-ref:` remain.

## Phase 4 — Generators, hooks, and CI scaffolding

### 8. Implement `scripts/gen-spec-deps.sh`

- [x] Walk every file matching `specs/NNN-*/spec.md` and `specs/NNN-*/spec-and-plan.md` (excludes `specs/templates/` and other non-feature dirs by design)
- [x] Parse markdown body, ignoring fenced code blocks
- [x] Find inline links matching sibling-spec patterns (`../NNN-feature/...` or `(specs/NNN-feature/...)`)
- [x] Compute the union of slugs and rewrite the frontmatter `dependencies` list (sorted; empty list rendered as `[]`)
- [x] Idempotent: running twice produces the same output
- [x] Bash-portable (awk-based parsing; works with macOS BSD utilities)
- [x] `--dry-run` flag exits 1 if any spec needs updating; useful for CI and validate
- [x] `--help` prints usage

Done when: running `./scripts/gen-spec-deps.sh` against the current repo produces a valid result; second run produces no diff; the script is executable.

### 9. Implement `scripts/gen-readme-table.sh`

- [x] Find marker comments in `README.md`: `<!-- generated:feature-specs:start -->` and `<!-- generated:feature-specs:end -->`
- [x] For each `specs/NNN-*/spec*.md`, parse frontmatter (`status`, `dependencies`) and the first body paragraph (description)
- [x] Skip leading blockquote signposts/notes; truncate description to first sentence
- [x] Reduce dependencies to the NNN prefix only (matches original compact README style)
- [x] Emit a markdown table sorted by feature number between the markers
- [x] Exit 3 if markers are absent
- [x] Idempotent; supports `--dry-run` (exit 1 on diff)

Done when: running the script populates the README table from spec frontmatter; second run produces no diff. ✓

### 10. Implement `scripts/gen-help-tables.sh`

- [x] Find marker comments in `framework/commands/help.md` for each of the five command groups (`commands-pipeline`, `commands-elaborate`, `commands-brownfield`, `commands-orient`, `commands-bootstrap`)
- [x] For each group, list the commands belonging to that group (hardcoded grouping in the generator) and read each command file's frontmatter `description:`
- [x] Pipeline group includes the extra "Pipeline Gate" column (gate values hardcoded — they are static pipeline facts)
- [x] Bootstrap group sources `/govern` from `framework/bootstrap/govern.md` and `/configure` from `framework/bootstrap/configure/claude.md`
- [x] Emit a markdown table per group between its markers
- [x] Exit 5 if any expected marker pair is absent; exit 4 if any referenced command file is missing
- [x] Idempotent; supports `--dry-run`

Done when: running the script populates all five tables in `help.md` from command frontmatter; second run produces no diff. ✓

### 11. Add marker comments to `README.md` and `framework/commands/help.md`

- [x] Insert `<!-- generated:feature-specs:start -->` and `<!-- generated:feature-specs:end -->` around the existing Feature Specs table in `README.md`
- [x] Insert the five marker pairs in `help.md` around the corresponding tables
- [x] Run the new generators to confirm they recognize the markers and produce correct output
- [x] Lint passes (HTML comments are allowed per project markdownlint config)

Done when: both files have valid marker pairs; running the generators produces no diff. ✓

### 12. Implement `.githooks/pre-commit` and `scripts/install-hooks.sh`

- [x] Create `.githooks/` directory and `pre-commit` script
- [x] Hook calls all four generators in order: `gen-claude-commands.sh`, `gen-readme-table.sh`, `gen-help-tables.sh`, `gen-spec-deps.sh`
- [x] After generators run, stage outputs at known paths (`.claude/commands/gov/`, `README.md`, `framework/commands/help.md`, `specs/NNN-*/spec*.md`)
- [x] Hook is executable
- [x] Create `scripts/install-hooks.sh` that runs `git config core.hooksPath .githooks` (idempotent; warns if existing different value)
- [x] Both scripts begin with `#!/usr/bin/env bash` and `set -euo pipefail`

Done when: running `./scripts/install-hooks.sh` configures `core.hooksPath` correctly; making a no-op commit does not produce errors; modifying a command source produces an updated `.claude/commands/gov/*.md` automatically on commit. ✓

### 13. Implement `framework/bootstrap/hooks/pre-commit` and `framework/bootstrap/hooks/install.sh`

- [x] Create the shipped adopter hook: calls `scripts/gen-spec-deps.sh` only
- [x] Sentinel comment `# managed-by: govern` on the second line (after the shebang) — this is what `/govern` looks for to distinguish managed from hand-rolled hooks
- [x] Create the shipped install script: idempotent `git config core.hooksPath .githooks`; refuses to overwrite an existing non-`.githooks` value (deferred to /govern's detection of existing hook systems)
- [x] Both scripts use `#!/usr/bin/env bash` and `set -euo pipefail`
- [x] Both are executable

Done when: the shipped scripts exist and would correctly install in an adopter project. ✓

### 14. Add CI workflow for govern repo

- [x] Create `.github/workflows/generators.yml`
- [x] On `pull_request` (path-filtered to relevant files) and on `push` to main, run all four generators and `git diff --exit-code`
- [x] Fail the build on non-empty diff with a clear `::error::` annotation
- [x] Use ubuntu-latest with bash; checkout via actions/checkout@v4

Done when: the workflow file exists and is valid GHA YAML. Will run on next PR.

### 15. Ship adopter CI template

- [x] Create `framework/templates/ci/adopter-generators.yml`
- [x] Document in the file header that adopters copy this into their own `.github/workflows/govern-generators.yml`
- [x] Reference `scripts/gen-spec-deps.sh` (the shipped generator name)
- [x] Add a one-paragraph note to govern's README ("Optional CI enforcement" section) pointing at the template

Done when: the template exists and documents adopter usage. ✓

## Phase 5 — Commands

### 16. Update `framework/commands/specify.md`

- [x] Remove the "Tag prompt" step entirely
- [x] Renumber remaining steps
- [x] Remove the `title` placeholder fill-in instruction
- [x] Remove all references to `tags` in the frontmatter writing instructions
- [x] Reframe lightweight-track detection step around filename-as-source-of-truth
- [x] Remove the "Add the new feature to the table in README.md" step (regenerated by hook)
- [x] Reframe `dependencies` as generator-managed
- [x] Lint passes

### 17. Update `framework/commands/capture.md`

- [x] Remove the `title` placeholder fill-in instruction
- [x] Remove the `tags` field reference
- [x] Remove the "Add the new feature to the table in README.md" step
- [x] Reframe `dependencies` as generator-managed
- [x] Lint passes

### 18. Update `framework/commands/clarify.md`

- [x] Remove the `tags` advisory from the validation gate
- [x] Add cross-spec impact check step (after deps readiness)
- [x] Add "Recompute dependencies" step at the very start of Hot Path
- [x] Lint passes

### 19. Update `framework/commands/plan.md`

- [x] Remove the `title` placeholder fill-in instructions for plan.md and data-model.md
- [x] Remove the "Open Questions Resolved" reference from the plan body fill-in
- [x] Remove the `[simple]` marker proposal step
- [x] Reframe Affected Files as planning aid (not authoritative)
- [x] Add cross-spec impact check step before Finalize
- [x] Add "Recompute dependencies" step at Setup
- [x] Update Finalize to not list `[simple]` markers
- [x] Lint passes

### 20. Update `framework/commands/implement.md`

- [x] Remove the `[simple]` marker reading
- [x] Replace Affected Files write-boundary with git-derived boundary at Setup step 7
- [x] Update Walk through tasks step 4 to use new boundary; remove plan-backfill instruction
- [x] Add cross-spec scan step at Completion (step 1)
- [x] Add "Recompute dependencies" step at Setup (step 6)
- [x] Renumber Setup steps; fix --auto references to step numbers
- [x] Replace constants/env-vars constitution refs with `framework/rules/configuration-cross.md`
- [x] Lint passes

### 21. Update `framework/commands/elaborate.md`

- [x] Remove the `title` placeholder fill-in instruction
- [x] Change `spec-ref` writing to `section:` (parent feature implicit in path)
- [x] Add "Recompute dependencies" step in Confirm target
- [x] Lint passes

### 22. Update `framework/commands/groom.md`

- [x] Remove the `[promote-to-rule]` prefix instruction
- [x] Replace with "leave in inbox unmodified — next groom pass re-walks every unmigrated item"
- [x] Add `specs/rules/configuration-cross.md` to the rule-file examples
- [x] Lint passes

### 23. Update `framework/commands/analyze.md`

- [x] Remove the entire "PKM title field (advisory)" section
- [x] Remove the entire "Frontmatter schema (advisory)" section (only contained tags)
- [x] Rename the `spec-ref` check to `section`
- [x] Add new "Generator drift (advisory)" section running gen-spec-deps/gen-readme-table/gen-help-tables in dry-run
- [x] Add `framework/rules/configuration-cross.md` to the rule-file list; cite 017's data-model alongside 008's
- [x] Remove the entire "Fix Mode" section
- [x] Remove `--fix` from frontmatter `argument-hint` and Context flag parsing
- [x] Update Scope Boundaries to remove `--fix` write permission and add dry-run script invocation permission
- [x] Replace per-row help-equivalence check with structural generator-drift check
- [x] Lint passes

### 24. Update `framework/commands/amend.md`

- [x] Add "Recompute dependencies" step in Confirm target (skip on scenario targets)
- [x] Lint passes

### 25. Update `framework/commands/target.md`

- [x] Remove `tags` from frontmatter parse
- [x] Add "Recompute dependencies" step in step 4 (between spec-file detection and frontmatter parse)
- [x] Lint passes

### 26. Add marker comments to `framework/commands/help.md`

- [x] Done as part of task 11 (all five marker pairs added; generator produces no diff). ✓

## Phase 6 — Bootstrap installer

### 27. Update `framework/bootstrap/govern.md` for adopter hook installation

- [x] Add new top-level section "Hook Installation" before "Placeholder Substitution"
- [x] Document seven detection states + actions (already-wired, custom hooksPath, husky, pre-commit-py, lefthook, govern-managed, none)
- [x] Sentinel-based detection of govern-managed hooks via `# managed-by: govern` comment
- [x] Add `framework/rules/configuration-cross.md` → `specs/rules/configuration-cross.md` to update-strategy manifest
- [x] Add `framework/bootstrap/hooks/pre-commit` → `.githooks/pre-commit` to update-strategy manifest (subject to detection)
- [x] Add `scripts/gen-spec-deps.sh` → `scripts/gen-spec-deps.sh` to update-strategy manifest (pinnable via `.govern.toml`)
- [x] Document manual integration snippet for adopters with existing hook systems
- [x] Document `.govern.toml` pinning for the hook file
- [x] Update Post-Scaffolding Output to include hook installation status line
- [x] Lint passes

Done when: `/govern` ships the new files and manages the adopter hook with the documented detection logic. ✓

### 28. Update permission files

- [x] `framework/bootstrap/configure/claude.md` — added `chmod +x`, `git config core.hooksPath` (set/get/unset), `./.githooks/pre-commit`, `scripts/gen-*.sh`, `scripts/install-hooks.sh` permissions
- [x] `framework/bootstrap/configure/auggie.md` — same permissions in Auggie's regex format
- [x] Lint passes

Done when: adopters scaffolding via `/govern` get the necessary Bash permissions for hook operations without prompts. ✓

## Phase 7 — Migration, regeneration, and self-cleanup

### 29. Migrate existing dogfood specs' body inline links

- [x] For each spec at `specs/000-016/spec.md`, read the frontmatter `dependencies` list
- [x] Scan the body for inline markdown links to each declared dependency outside fenced code blocks
- [x] For each dependency without an inline link, append a "References" section at the end of the body with a bullet linking the dep (13 specs got References sections)
- [x] Run `gen-spec-deps.sh` against the entire repo — second run produces no diff (idempotent)
- [x] Lint passes on every modified file

Done when: every existing declared dependency is reflected in the body via an inline link; `gen-spec-deps.sh` is idempotent on the migrated specs. ✓

Note: gen-spec-deps additionally added body-derived deps where the body had inline links to specs not in the original frontmatter (e.g., spec 000 gained `[012, 014]` from cross-references in its body). This is the principle in action — body is authoritative, frontmatter reflects all real cross-references. Six specs (000, 003, 005, 006, 007, 008) saw frontmatter additions of this kind; their original declared deps are preserved alongside the additions.

### 30. Run all generators and commit derived state

- [x] Run all four generators by hand: gen-claude-commands, gen-readme-table, gen-help-tables, gen-spec-deps
- [x] Verify each generator exits 0
- [x] `git diff` — reviewed: 11 .claude/commands/gov/*.md (Phase 5 propagation), README.md (deps now show derived numbers), 15 specs/*/spec.md (migration + derivation)
- [x] Run `npx markdownlint-cli2` across modified files — clean
- [x] Re-run `./scripts/install-hooks.sh` to reactivate the pre-commit hook (deactivated at task 12 to avoid early gen-spec-deps trigger)
- [x] Commit the derived state

Done when: working tree is clean after running every generator a second time. ✓

### 31. Verify `/validate` runs cleanly on every existing spec

- [x] Confirmed via manual scripted sanity check (proxy for `/gov:analyze --all`):
  - Every scenario has either `section` (new) or `spec-ref` (legacy) — no hard-fails
  - Every spec has required `status` and `dependencies` fields
  - `gen-spec-deps.sh --dry-run` reports no drift across all 18 specs
  - `npx markdownlint-cli2` passes on all 115 .md files (specs, framework, README, AGENTS)
- [x] Validate logic + constitution updated to accept `spec-ref` as a legacy alternative to `section` — required to honor the frozen-archaeology rule for 6 existing scenarios in done specs (000, 007)
- [x] No new findings introduced by schema changes; stale `title`/`tags`/`spec-ref`/`track` fields in done specs are silently ignored per open-schema rule

Done when: validate runs cleanly against the whole repo modulo expected pre-final-task findings on 017. ✓ (no expected findings remain — title/tags on 017 are silently ignored per open-schema rule, just like in done specs).

### 32. Strip `title:` and `tags:` from this spec's own artifacts

- [x] `specs/017-derive-dont-ask/spec.md` — removed `title:` and `tags:` lines from frontmatter (frontmatter now: status, dependencies)
- [x] `specs/017-derive-dont-ask/plan.md` — removed entire frontmatter (only had `title:`)
- [x] `specs/017-derive-dont-ask/tasks.md` — removed entire frontmatter (only had `title:`)
- [x] `specs/017-derive-dont-ask/data-model.md` — removed entire frontmatter (only had `title:`)
- [x] Lint passes on all four files

Done when: this spec's artifacts have no `title:` or `tags:` frontmatter; validate runs clean. ✓

## Phase 8 — Post-done scenarios

### 33. Implement scenario: skip-prose-cross-references

- [x] Implement the behavior described in `scenarios/skip-prose-cross-references.md`

- **Done when**: the scenario's described behavior is correctly implemented and tested. `gen-spec-deps.sh` recognizes the chosen opt-out form (resolved via `/gov:clarify` Q1) so navigational cross-references can stay rich without inducing dep edges; fixtures cover the opt-out region, an unmarked link still producing an edge, and the existing code-fence exclusion regression; constitution / AGENTS.md / 017's §Generators and Hooks gain the one-line carve-out describing the chosen form.

### 34. Implement scenario: detect-dependency-cycles

- [x] Implement the behavior described in `scenarios/detect-dependency-cycles.md`

- **Done when**: the scenario's described behavior is correctly implemented and tested. `gen-spec-deps.sh` exits non-zero and names the SCC(s) on stderr when the generated graph contains a cycle; the pre-commit hooks (`.githooks/govern-pre-commit` and shipped `framework/bootstrap/hooks/govern-pre-commit`) propagate the failure and block the commit; fixtures cover 2-cycle, 3-cycle (single SCC), mixed acyclic+cyclic, self-cycle, and the acyclic happy path.
