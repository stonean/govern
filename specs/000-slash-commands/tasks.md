# 000 — Slash Command Templates Tasks

Tasks derived from the [plan](plan.md). Complete in order.

> **Note:** these tasks were written against the original layout and command names. Command sources now live in `framework/commands/`; several names were later renamed (`about → help`, `setup → configure`, `next` retired). See `spec.md` for the full rename history.

## 1. Create the commands directory and about command

- [x] Create `commands/` directory at governance root
- [x] Write `commands/about.md` — static guide with pipeline stages, command table, typical session flow, key concepts
- [x] Verify `{project}` placeholder is used for all command references

Done when: `about.md` exists and prints a self-contained guide without referencing any files to read.

## 2. Create utility commands (target, status, setup)

- [x] Write `commands/target.md` — accept feature identifier, resolve to directory, write session file, display status
- [x] Write `commands/status.md` — scan `specs/` for `NNN-*` directories, extract status from each, display dashboard table
- [x] Write `commands/setup.md` — configure `.claude/settings.local.json` with git, lint, and file read permissions

Done when: all three utility commands exist, use `{project}` placeholders, and reference `.claude/{project}-session.json` for session state.

## 3. Create the specify command

- [x] Write `commands/specify.md` — accept description, determine next feature number, prompt lightweight track qualifying questions, create spec directory, copy appropriate template, set session target, update README
- [x] Include lightweight track detection logic (four qualifying questions)
- [x] Include instructions for both `spec.md` and `spec-and-plan.md` creation paths

Done when: specify command handles both standard and lightweight track, creates the correct template, and updates README.

## 4. Create the clarify command

- [x] Write `commands/clarify.md` — read session target, enforce draft gate, resolve open questions, enumerate edge cases, verify acceptance criteria, check dependencies, update status
- [x] Include spec file detection (check `spec.md` then `spec-and-plan.md`)

Done when: clarify command enforces the draft gate and handles both spec file types.

## 5. Create the plan command

- [x] Write `commands/plan.md` — read session target, enforce clarified gate, create plan.md and tasks.md from templates, run readiness check, update status
- [x] Include lightweight track adaptation (skip plan creation if `spec-and-plan.md` exists with plan section, still create tasks.md)
- [x] Include readiness check matching constitution's seven-point gate

Done when: plan command enforces the clarified gate, creates both plan and tasks artifacts, runs readiness check, and handles lightweight track.

## 6. Create the implement command

- [x] Write `commands/implement.md` — read session target, enforce planned/in-progress gate, walk through tasks, verify acceptance criteria, update status
- [x] Include spec file detection for acceptance criteria verification

Done when: implement command enforces the gate, walks tasks in order, and verifies acceptance criteria before marking done.

## 7. Create the validate and next commands

- [x] Write `commands/validate.md` — read session target, run all checks (spec integrity, artifact completeness, plan consistency, task consistency, dependencies, cross-spec references, markdownlint)
- [x] Write `commands/next.md` — read session target, determine current status, run the appropriate pipeline command

Done when: validate reports PASS/FAIL for all checks including markdownlint, and next correctly maps status to the right command.

## 8. Final review and lint

- [x] Run `npx markdownlint-cli2` on all files in `commands/`
- [x] Verify every command uses `{project}` consistently (no hardcoded project names)
- [x] Verify all cross-references between commands are correct
- [x] Update spec status to `planned`

Done when: all ten commands pass lint, use consistent placeholders, and cross-reference correctly.

## 9. Implement scenario: target-argument-parsing

- [x] Implement the behavior described in `scenarios/target-argument-parsing.md`

Done when: the scenario's described behavior is correctly implemented and tested.

## 10. Implement scenario: clarify-one-at-a-time

- [x] Implement the behavior described in `scenarios/clarify-one-at-a-time.md`

Done when: the scenario's described behavior is correctly implemented and tested.

## 11. Implement scenario: validation-gates

- [x] Implement the behavior described in `scenarios/validation-gates.md`

Done when: the scenario's described behavior is correctly implemented and tested.

## 12. Implement scenario: validate-fix-mode

- [x] Implement the behavior described in `scenarios/validate-fix-mode.md`

Done when: the scenario's described behavior is correctly implemented and tested.

## 13. Implement scenario: code-location-index

- [x] Implement the behavior described in `scenarios/code-location-index.md`

Done when: the scenario's described behavior is correctly implemented and tested. (Producer logic added to `framework/commands/implement.md` as a new "Code-location index" section + walk-tasks step 7. Will be exercised on the next `/gov:implement` run; the prose is mechanical — maintain a map, write deterministic markdown — so no fixture simulation is warranted at this time.)

## 14. Implement scenario: command-autocomplete-summary

- [x] Verified Claude Code's frontmatter field for autocomplete summaries is `description:` (confirmed by single-file experiment on `init.md` — the description surfaced in the session skills list while siblings without it rendered as bare names).
- [x] Added `description:` frontmatter to all 14 sources under `framework/commands/*.md` and to `framework/bootstrap/configure/claude.md`. Each description is a one-line concrete summary. `framework/bootstrap/configure/auggie.md` left alone — different CLI with its own conventions.
- [x] Re-ran `scripts/gen-claude-commands.sh` to flow descriptions into `.claude/commands/gov/*.md`. Manually added the same field to the hand-maintained `.claude/commands/gov/init.md`.
- [x] Lint clean (`npx markdownlint-cli2 "**/*.md" "!CLAUDE.md"` — 121 files, 0 errors).

Done when: the scenario's described behavior is correctly implemented; every gov command shows a summary in the Claude Code autocomplete window; the repo lints clean.
