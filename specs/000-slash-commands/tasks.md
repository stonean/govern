# 000 — Slash Command Templates Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create the commands directory and about command

- [ ] Create `commands/` directory at governance root
- [ ] Write `commands/about.md` — static guide with pipeline stages, command table, typical session flow, key concepts
- [ ] Verify `{project}` placeholder is used for all command references

Done when: `about.md` exists and prints a self-contained guide without referencing any files to read.

## 2. Create utility commands (target, status, setup)

- [ ] Write `commands/target.md` — accept feature identifier, resolve to directory, write session file, display status
- [ ] Write `commands/status.md` — scan `specs/` for `NNN-*` directories, extract status from each, display dashboard table
- [ ] Write `commands/setup.md` — configure `.claude/settings.local.json` with git, lint, and file read permissions

Done when: all three utility commands exist, use `{project}` placeholders, and reference `.claude/{project}-session.json` for session state.

## 3. Create the specify command

- [ ] Write `commands/specify.md` — accept description, determine next feature number, prompt lightweight track qualifying questions, create spec directory, copy appropriate template, set session target, update README
- [ ] Include lightweight track detection logic (four qualifying questions)
- [ ] Include instructions for both `spec.md` and `spec-and-plan.md` creation paths

Done when: specify command handles both standard and lightweight track, creates the correct template, and updates README.

## 4. Create the clarify command

- [ ] Write `commands/clarify.md` — read session target, enforce draft gate, resolve open questions, enumerate edge cases, verify acceptance criteria, check dependencies, update status
- [ ] Include spec file detection (check `spec.md` then `spec-and-plan.md`)

Done when: clarify command enforces the draft gate and handles both spec file types.

## 5. Create the plan command

- [ ] Write `commands/plan.md` — read session target, enforce clarified gate, create plan.md and tasks.md from templates, run readiness check, update status
- [ ] Include lightweight track adaptation (skip plan creation if `spec-and-plan.md` exists with plan section, still create tasks.md)
- [ ] Include readiness check matching constitution's seven-point gate

Done when: plan command enforces the clarified gate, creates both plan and tasks artifacts, runs readiness check, and handles lightweight track.

## 6. Create the implement command

- [ ] Write `commands/implement.md` — read session target, enforce planned/in-progress gate, walk through tasks, verify acceptance criteria, update status
- [ ] Include spec file detection for acceptance criteria verification

Done when: implement command enforces the gate, walks tasks in order, and verifies acceptance criteria before marking done.

## 7. Create the validate and next commands

- [ ] Write `commands/validate.md` — read session target, run all checks (spec integrity, artifact completeness, plan consistency, task consistency, dependencies, cross-spec references, markdownlint)
- [ ] Write `commands/next.md` — read session target, determine current status, run the appropriate pipeline command

Done when: validate reports PASS/FAIL for all checks including markdownlint, and next correctly maps status to the right command.

## 8. Final review and lint

- [ ] Run `markdownlint-cli2` on all files in `commands/`
- [ ] Verify every command uses `{project}` consistently (no hardcoded project names)
- [ ] Verify all cross-references between commands are correct
- [ ] Update spec status to `planned`

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
