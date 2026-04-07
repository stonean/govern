# Implement

Execute implementation tasks for the targeted feature.

## Purpose

Pipeline gate: `planned` → `in-progress` → `done`. Walks through `tasks.md` step by step, implementing each task according to the plan. This is the only command that writes application code.

## Context

Use the session target from `{cli-config-dir}/{project}-session.json`. If `$ARGUMENTS` is provided, use it to override the session target. If no session target is set and no arguments provided, stop and tell the user to run `/{project}:target` first.

## Spec File Detection

Check for `spec.md` first, then `spec-and-plan.md`. Use whichever exists for reading acceptance criteria.

## Gate

Read the spec status. If the status is not `planned` or `in-progress`, stop and report:

- `draft` → "Spec has unresolved open questions. Run `/{project}:clarify` first."
- `clarified` → "No plan exists. Run `/{project}:plan` first."
- `done` → "Feature is already complete."
- No tasks.md → "No task breakdown exists. Run `/{project}:plan` first."

## Scope Boundaries

- Use the plan's **Affected Files** section as the expected write boundary. If you need to modify an unlisted file, notify the user and explain why before proceeding.
- Do NOT read or modify files belonging to other features' spec directories.
- Do NOT read source code speculatively — only read files relevant to the current task.
- Reference: §implement-phase, §constants, §env-vars, §pipeline-boundaries (constitution loaded by `/{project}:target` — do not re-read).

## Instructions

### Setup

1. Read `{cli-config-dir}/{project}-session.json` for the session target, including optional `scenario` and `scenarioPath` fields.
2. Read `specs/{feature}/tasks.md` for the ordered task list.
3. Read `specs/{feature}/plan.md` (or the plan section of `spec-and-plan.md`) for technical decisions and affected files.
4. Read the spec file for acceptance criteria and contracts.
5. If a scenario is targeted, read the scenario file for scenario-specific context, behavior, and edge cases. The scenario scopes which part of the feature is the primary focus for this implementation session.
6. Note the plan's **Affected Files** list — this is the expected write boundary for implementation.
7. If spec status is `planned`, ask the user to approve the transition to `in-progress` before updating the status.

### Progressive context loading

Load context incrementally to stay focused:

- **At setup:** Read only the spec, plan, tasks, and scenario file (if targeted). Do NOT read `system.md`, `events.md`, `errors.md`, or source code yet.
- **Per task:** Read only the source files relevant to that task from the plan's affected files list. When a scenario is targeted, prioritize tasks related to the scenario's behavior. Read `AGENTS.md` conventions and `specs/system.md` sections only when the task involves patterns they govern (e.g., read error conventions only when implementing error handling).
- **At completion:** Re-read acceptance criteria from the spec to verify. Do NOT re-read the full plan or tasks.

### Walk through tasks

For each task in order:

1. Display the task number, description, and "done when" condition.
2. Read the relevant technical decisions from the plan.
3. Read only the existing code files relevant to this task from the plan's affected files.
4. Implement the task:
   - Write code, tests, and migrations as needed.
   - Follow conventions in `AGENTS.md` and `specs/system.md` (§implement-phase, §constants, §env-vars as applicable).
   - Respect the contracts defined in the spec.
   - If you need to modify files outside the plan's affected files list, notify the user, explain why, and add the file to the plan's **Affected Files** section with a comment explaining why it was added.
5. Verify the "done when" condition is met.
6. Mark the task as complete in `tasks.md` — update each checkbox from `- [ ]` to `- [x]`, including nested sub-item checkboxes, before proceeding.
7. Prompt the user to commit and push changes.
8. Before starting the next task, assess whether sufficient context remains to complete it. If context is low, inform the user and suggest starting a new session with `/{project}:implement` to continue from the next incomplete task. If context is sufficient, proceed.

### Completion

After all tasks are done:

1. Walk through each acceptance criterion from the spec and verify it is met. Mark each passing criterion `- [x]` in the spec file at the time of verification. If a criterion fails, leave it as `- [ ]` and report the failure. Do not batch-mark — verify each individually.
2. Run the validation gate before proposing the status transition:
   - All tasks in `tasks.md` are marked `- [x]`
   - All acceptance criteria in the spec are marked `- [x]`
   - All scenario-linked tasks are complete
   - All `.md` files in the feature directory pass `markdownlint-cli2`
3. If any validation check fails, report the specific failures and do not propose the transition. The user fixes the issues and re-runs the command.
4. If all checks pass, present a summary and ask the user to approve the transition to `done`. Do not update the status until the user confirms.
5. Update spec status to `done`.
