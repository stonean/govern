# 012 — Multi-Agent Govern Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Move setup sources into per-agent directory

- [x] Create `commands/setup/` directory.
- [x] Move `commands/setup.md` → `commands/setup/claude.md` (no content change).
- [x] Move `commands/setup-auggie.md` → `commands/setup/auggie.md` (no content change).
- [x] Verify `commands/setup.md` and `commands/setup-auggie.md` no longer exist.

Done when: `commands/setup/claude.md` and `commands/setup/auggie.md` exist with the previous file contents intact, and the old flat-named files are deleted.

## 2. Refresh governance's own setup command

- [x] Re-derive `.claude/commands/gov/setup.md` from `commands/setup/claude.md` with `{cli-config-dir}` resolved to `.claude` and `{project}` resolved to `gov`.

Done when: `.claude/commands/gov/setup.md` matches `commands/setup/claude.md` with placeholders resolved (Command File Parity satisfied).

## 3. Write the unified `govern/govern.md`

- [x] Replace `govern/govern.md` with the unified content:
  - Agent registry table near the top with the five fields per agent (`key`, `name`, `config_dir`, `settings_template`, `rules_file_note`) and rows for `claude` and `auggie`.
  - Pre-flight checks (git repo, existing `specs/` warning) preserved from the existing file.
  - Agent selection logic with the precedence: `--agents=` → auto-detect → `--add-agent` / first-run prompt. Reject unknown agent keys before any scaffolding. Reject zero-agent selection in prompt paths.
  - Per-agent scaffolding loop that runs the manifest, writes the setup source from `commands/setup/{key}.md`, creates the empty session JSON, merges the agent's `settings_template` into `settings.local.json`, and installs a copy of `govern.md` into `{config_dir}/commands/govern.md`.
  - Shared-file scaffolding (constitution, templates, AGENTS.md, CLAUDE.md, .gitignore, specs/system.md, etc.) running once per `/govern` invocation outside the loop.
  - Post-write integrity check that the installed `govern.md` starts with `# Govern`, with re-fetch on failure.
  - Post-scaffolding output covering created/updated/unchanged/skipped/pinned/merged files, the `rules_file_note` for each scaffolded agent, the self-update notice when the installed `govern.md` was updated, first-run vs update-mode next steps, and migration guidance for users coming from 007's per-CLI files.

Done when: `govern/govern.md` exists in unified form, passes `npx markdownlint-cli2`, and never references `govern-auggie.md`.

## 4. Delete `govern/govern-auggie.md`

- [x] Remove `govern/govern-auggie.md`.
- [x] Confirm no other file in the repo references `govern-auggie.md` (README, specs, commands).

Done when: `govern/govern-auggie.md` is gone and the repo has no remaining references to it.

## 5. Update `README.md`

- [x] Under each per-agent curl snippet in the "Adopting in an Existing Project" section, add a single framing line stating that subsequent agents do not require a second curl — re-running `/govern` with `--add-agent` adopts additional agents from the unified file.
- [x] Confirm the README still documents both supported agents' install paths.

Done when: the README's adoption section explains the multi-agent flow without changing the existing per-agent install instructions.

## 6. Remove "Govern File Parity" from `CLAUDE.md`

- [x] Delete the "Govern File Parity" section from `CLAUDE.md`.
- [x] Leave the "Command File Parity" section in place.

Done when: `CLAUDE.md` contains only the "Command File Parity" rule under "Governance Repo Rules" and passes `npx markdownlint-cli2`.

## 7. Add signpost to `specs/007-govern-workflow/spec.md`

- [x] Add a signpost paragraph below the existing 011 signpost noting that the multi-file design is superseded by 012, with a link to `../012-multi-agent-govern/spec.md`.
- [x] Leave 007's status at `done`.

Done when: 007's spec links to 012 as the successor, status remains `done`, and the file passes `npx markdownlint-cli2`.

## 8. Lint and verify

- [x] Run `npx markdownlint-cli2` over every modified or created file: `govern/govern.md`, `commands/setup/claude.md`, `commands/setup/auggie.md`, `.claude/commands/gov/setup.md`, `README.md`, `CLAUDE.md`, `specs/007-govern-workflow/spec.md`, `specs/012-multi-agent-govern/spec.md`, `specs/012-multi-agent-govern/plan.md`, `specs/012-multi-agent-govern/data-model.md`, `specs/012-multi-agent-govern/tasks.md`.
- [x] Walk the spec's Acceptance Criteria list and confirm each is satisfied by the changes.

Done when: markdownlint reports zero errors and every acceptance criterion is verified against the implementation.

## 9. Promote spec to `done`

- [x] After the user confirms the acceptance criteria are met, set `specs/012-multi-agent-govern/spec.md` status to `done`.

Done when: the spec status is `done` and the change is committed.
