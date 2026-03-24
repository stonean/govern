# 006 — Bug Workflow Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create scenario and triage templates

- [ ] Create `templates/scenario.md` with spec-ref, Context, Behavior, and Edge Cases sections
- [ ] Create `templates/triage.md` with flat inbox format and migration rules
- [ ] Update `templates/spec.md` to reference the `scenarios/` directory convention

Done when: all three template files exist with correct structure, and `spec.md` template mentions scenarios.

## 2. Update constitution

- [ ] Add bug handling section with the decision tree (three branches: no spec, ambiguous spec, clear spec)
- [ ] Add scenario lifecycle documentation (scenarios as first-class artifacts, directory convention, when to create vs. not)
- [ ] Update the spec phase file structure to include `scenarios/` subdirectory

Done when: `constitution.md` includes bug handling, scenario lifecycle, and updated file structure showing `scenarios/`.

## 3. Create `/gov:scenario` command

- [ ] Create `commands/scenario.md` template: requires active session target, confirms target, walks decision tree, creates scenario file in `scenarios/`, appends task to `tasks.md`
- [ ] Handle edge cases: no session target, no `tasks.md`, duplicate scenario name, parent spec is `done`
- [ ] Create `.claude/commands/gov/scenario.md` by copying template and replacing `{project}` with `gov`

Done when: both command files exist, `/gov:scenario` creates scenario files under the correct feature's `scenarios/` directory and appends linked tasks to `tasks.md`.

## 4. Create `/gov:triage` command

- [ ] Create `commands/triage.md` template: reads `specs/triage.md`, walks each item through the decision tree, migrates items to specs or scenarios, removes resolved items
- [ ] Handle edge cases: `triage.md` does not exist, `triage.md` is empty
- [ ] Create `.claude/commands/gov/triage.md` by copying template and replacing `{project}` with `gov`

Done when: both command files exist, `/gov:triage` processes triage items and migrates them.

## 5. Update existing command templates

- [ ] Update `commands/about.md` to document `/scenario`, `/triage`, scenario conventions, and bug workflow
- [ ] Update `commands/status.md` to display scenario counts per spec
- [ ] Update `commands/next.md` to suggest `/scenario` as a next action when appropriate
- [ ] Update `commands/validate.md` to check that scenario-linked tasks are complete

Done when: all four command templates include the new functionality.

## 6. Re-derive governance command copies

- [ ] Re-derive `.claude/commands/gov/about.md` from updated `commands/about.md` (replace `{project}` with `gov`)
- [ ] Re-derive `.claude/commands/gov/status.md` from updated `commands/status.md`
- [ ] Re-derive `.claude/commands/gov/next.md` from updated `commands/next.md`
- [ ] Re-derive `.claude/commands/gov/validate.md` from updated `commands/validate.md`

Done when: all four governance copies match their templates with `{project}` replaced by `gov`.

## 7. Update README

- [ ] Add bug workflow and scenario conventions documentation to `README.md`
- [ ] Update the feature specs table with correct status for 006-bug-workflow

Done when: `README.md` documents the bug workflow and the feature table reflects current statuses.

## 8. Final lint and verification

- [ ] Run `markdownlint-cli2` on all new and modified files
- [ ] Verify all acceptance criteria from the spec are addressed by the tasks above

Done when: all files pass lint and every acceptance criterion maps to a completed task.
