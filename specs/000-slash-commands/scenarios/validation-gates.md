# Validation Gates

**spec-ref:** 000-slash-commands — Command Set

## Context

Specs contain acceptance criteria as checkbox lists (`- [ ]`). Task files contain task items as checkbox lists. When a pipeline command verifies that a criterion or task is complete, the checkbox should be updated to `- [x]` in the file. Currently checkboxes are inconsistently updated across the project.

Additionally, the `validate` command exists for on-demand auditing but its checks are not integrated into the pipeline. Validation should run automatically before every status transition, not rely on the user remembering to invoke it separately.

## Behavior

### Checkbox marking (implement)

- When marking a task as complete in `tasks.md`, update the checkbox from `- [ ]` to `- [x]`.
- Nested checkboxes within a task (sub-items) should each be marked individually as they are completed.
- During the completion phase, verify each acceptance criterion in the spec file (`spec.md` or `spec-and-plan.md`). Mark each passing criterion `- [x]` in the file.
- Checkboxes are updated at the time of verification, not deferred to the end.
- Only mark a checkbox as complete when the item has been explicitly verified — do not batch-mark items without verification.

### Validation before status transitions (clarify, plan, implement)

Every pipeline command that proposes a status transition must run the relevant validate checks as a gate before asking the user to approve the transition. Failures block the transition.

- **clarify** (`draft` → `clarified`): open questions resolved, acceptance criteria are concrete and testable, dependencies checked.
- **plan** (`clarified` → `planned`): plan and tasks exist, readiness check passes, no conflicts with `system.md`.
- **implement** (`in-progress` → `done`): all tasks marked `- [x]`, all acceptance criteria marked `- [x]`, scenario-linked tasks complete, markdownlint passes.

The `/validate` slash command remains available for on-demand auditing outside the pipeline.

## Edge Cases

- If an acceptance criterion was already marked `- [x]` from a prior session, leave it as-is and do not re-verify unless the user requests it.
- If a criterion fails verification, leave it as `- [ ]` and report the failure.
- If validation fails, report the specific failures and do not propose the status transition. The user fixes the issues and re-runs the command.
