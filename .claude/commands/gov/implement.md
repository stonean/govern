---
description: Execute implementation tasks for the targeted feature.
argument-hint: "[--auto] [feature]"
---

# Implement

Execute implementation tasks for the targeted feature.

## Purpose

Pipeline gate: `planned` → `in-progress` → `done`. Walks through `tasks.md` step by step, implementing each task according to the plan. This is the only command that writes application code.

## Context

Use the session target from `.claude/gov-session.json`. If `$ARGUMENTS` is provided, use it to override the session target. If no session target is set and no arguments provided, stop and tell the user to run `/gov:target` first.

### Flags

`$ARGUMENTS` may include the `--auto` flag in any position. Strip it before treating remaining text as a feature override. The flag is per-invocation and is not persisted to `.claude/gov-session.json` — autonomy is an execution-time decision, not session state.

When `--auto` is set:

- Skip the per-task "prompt the user to commit and push changes" confirmation in **Walk through tasks** step 7. Commit on your own and proceed to the next task.
- **Commit, do not push.** Push is hard-to-reverse and externally visible; it stays gated even with `--auto`. Adopters who want auto-publish can wrap `/gov:implement --auto` in a script that pushes after each session.

The following gates **still fire and pause** even with `--auto` on:

- Pipeline gates (`planned`→`in-progress`, `in-progress`→`done`) — confirmation required per §pipeline-boundaries.
- Stuck-detection events (see Setup step 8) — auto mode does not power through cycles.
- Out-of-bounds file writes (see **Walk through tasks** step 4) — modifying a file outside the runtime boundary still requires user notification.
- Spec edits, plan edits, or new tasks discovered mid-implement.
- Risky actions per the agent's safety rules (destructive ops, secrets, force pushes, etc.).

Default is unset — without the flag, the user confirms each task as today.

## Spec File Detection

Check for `spec.md` first, then `spec-and-plan.md`. Use whichever exists for reading acceptance criteria.

## Gate

Read the spec's `status` field from the YAML frontmatter at the top of the file. If `status` is not `planned` or `in-progress`, stop and report:

- `draft` → "Spec has unresolved open questions. Run `/gov:clarify` first."
- `clarified` → "No plan exists. Run `/gov:plan` first."
- `done` → "Spec is already complete."
- No tasks.md → "No task breakdown exists. Run `/gov:plan` first."

## Scope Boundaries

- The runtime write boundary is derived at Setup from `git diff` against the spec dir's first commit (see Setup step 7). The plan's **Affected Files** section is a planning aid — informational, not authoritative.
- Do NOT read or modify files belonging to other features' spec directories.
- Do NOT read source code speculatively — only read files relevant to the current task.
- Reference: §implement-phase, §pipeline-boundaries, §text-first-artifacts, plus `framework/rules/configuration.md` for constants and env-vars (constitution loaded by `/gov:target` — do not re-read).

## Instructions

### Setup

1. Read `.claude/gov-session.json` for the session target, including optional `scenario` and `scenarioPath` fields.
2. Read `specs/{feature}/tasks.md` for the ordered task list.
3. Read `specs/{feature}/plan.md` (or the plan section of `spec-and-plan.md`) for technical decisions and affected files.
4. Read the spec file for acceptance criteria and contracts.
5. If a scenario is targeted, read the scenario file for scenario-specific context, behavior, and edge cases. The scenario scopes which part of the feature is the primary focus for this implementation session.
6. **Recompute dependencies (safety net).** Run `scripts/gen-spec-deps.sh --dry-run` against the target spec. If it reports a diff, run it for real to sync `dependencies:` from body inline links. The pre-commit hook normally keeps this in sync; this step catches uncommitted body edits.
7. **Derive the runtime write boundary.** Compute `first_commit=$(git log --reverse --format=%H -- specs/{feature}/ | head -1)` and `boundary=$(git diff --name-only "$first_commit"..HEAD)`, filtered to files outside `specs/{feature}/`. This is the set of files this feature has touched so far. New writes that fall outside `boundary` ∪ the plan's Affected Files surface to the user (see **Walk through tasks** step 4). Note the boundary; do not read its contents speculatively.
8. **Stuck-detection check.** If the spec's status is already `in-progress`, run `git log --oneline -- specs/{feature}/tasks.md` and count commits since the spec entered `in-progress`. Identify the first incomplete task (first `- [ ]` checkbox group) in `tasks.md`. If `git log` shows **≥ 3 commits** on `tasks.md` AND the same task is still the first incomplete one (no checkbox flipped to `- [x]` between those commits for that task), surface the cycle to the user with this message: `Task {N} ({title}) has been touched in {count} prior implement runs without completing. Consider decomposing it into smaller subtasks before continuing.` Pause and wait for user direction; do not auto-decompose. The threshold of 3 is fixed (not configurable in v1) — smallest count that distinguishes routine multi-session work from a cycle. Stuck-detection events fire even when `--auto` is set (auto mode does not power through cycles).
9. If the spec's frontmatter `status` is `planned`, ask the user to approve the transition to `in-progress` before updating the status. On confirmation, update the frontmatter `status` field to `in-progress`.

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
   - Follow conventions in `AGENTS.md` and `specs/system.md` (§implement-phase, plus `framework/rules/configuration.md` `CFG-CONST-NNN` / `CFG-ENV-NNN` rules as applicable).
   - Respect the contracts defined in the spec.
   - If a write would land outside the runtime boundary (`boundary` ∪ the plan's Affected Files, computed at Setup step 7), notify the user, explain why, and wait for confirmation before proceeding. Once accepted, the file is part of the boundary for the rest of the session — there is no plan backfill (the plan's Affected Files section is a planning aid, not the source of truth).
5. Verify the "done when" condition is met.
6. Mark the task as complete in `tasks.md` — update each checkbox from `- [ ]` to `- [x]`, including nested sub-item checkboxes, before proceeding.
7. Prompt the user to commit and push changes. With `--auto` set, skip the prompt: commit on your own, do not push.
8. Before starting the next task, assess whether sufficient context remains to complete it. If context is low, inform the user and suggest starting a new session with `/gov:implement` to continue from the next incomplete task. If context is sufficient, proceed.

### Completion

After all tasks are done:

1. **Cross-spec impact check.** Run `git diff --stat "$first_commit"..HEAD -- specs/` (using the first-commit value from Setup step 7), filtered to paths outside `specs/{feature}/`. If any sibling spec dir shows changes, surface the list to the user and ask: "These specs were modified during the implementation window. Were the changes intentional cross-spec updates per §cross-spec-impact, or accidental?" Informational; does not block.

2. Walk through each acceptance criterion from the spec and verify it is met. Mark each passing criterion `- [x]` in the spec file at the time of verification. If a criterion fails, leave it as `- [ ]` and report the failure. Do not batch-mark — verify each individually.
3. Run the validation gate before proposing the status transition:
   - All tasks in `tasks.md` are marked `- [x]`
   - All acceptance criteria in the spec are marked `- [x]`
   - All scenario-linked tasks are complete
   - All `.md` files in the feature directory pass `npx markdownlint-cli2`
4. If any validation check fails, report the specific failures and do not propose the transition. The user fixes the issues and re-runs the command.
5. If all checks pass, present a summary and ask the user to approve the transition to `done`. Do not update the status until the user confirms.
6. On confirmation, update the spec's frontmatter `status` field from `in-progress` to `done`.
