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

- Skip the per-task "prompt the user to commit and push changes" confirmation in **Walk through tasks** step 8. Commit on your own and proceed to the next task.
- **Commit, do not push.** Push is hard-to-reverse and externally visible; it stays gated even with `--auto`. Adopters who want auto-publish can wrap `/gov:implement --auto` in a script that pushes after each session.

The following gates **still fire and pause** even with `--auto` on:

- Phase transitions (`planned`→`in-progress`, `in-progress`→`done`) — confirmation required per §pipeline-boundaries.
- Stuck-detection events (see Setup step 7) — auto mode does not power through cycles.
- Out-of-bounds file writes (see **Walk through tasks** step 4) — modifying a file not in the plan's affected files list still requires user notification.
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

- Use the plan's **Affected Files** section as the expected write boundary. If you need to modify an unlisted file, notify the user and explain why before proceeding.
- Do NOT read or modify files belonging to other features' spec directories.
- Do NOT read source code speculatively — only read files relevant to the current task.
- Reference: §implement-phase, §constants, §env-vars, §pipeline-boundaries, §text-first-artifacts (constitution loaded by `/gov:target` — do not re-read).

## Instructions

### Setup

1. Read `.claude/gov-session.json` for the session target, including optional `scenario` and `scenarioPath` fields.
2. Read `specs/{feature}/tasks.md` for the ordered task list.
3. Read `specs/{feature}/plan.md` (or the plan section of `spec-and-plan.md`) for technical decisions and affected files.
4. Read the spec file for acceptance criteria and contracts.
5. If a scenario is targeted, read the scenario file for scenario-specific context, behavior, and edge cases. The scenario scopes which part of the feature is the primary focus for this implementation session.
6. Note the plan's **Affected Files** list — this is the expected write boundary for implementation.
7. **Stuck-detection check.** If the spec's status is already `in-progress`, run `git log --oneline -- specs/{feature}/tasks.md` and count commits since the spec entered `in-progress`. Identify the first incomplete task (first `- [ ]` checkbox group) in `tasks.md`. If `git log` shows **≥ 3 commits** on `tasks.md` AND the same task is still the first incomplete one (no checkbox flipped to `- [x]` between those commits for that task), surface the cycle to the user with this message: `Task {N} ({title}) has been touched in {count} prior implement runs without completing. Consider decomposing it into smaller subtasks before continuing.` Pause and wait for user direction; do not auto-decompose. The threshold of 3 is fixed (not configurable in v1) — smallest count that distinguishes routine multi-session work from a cycle. Stuck-detection events fire even when `--auto` is set (auto mode does not power through cycles).
8. If the spec's frontmatter `status` is `planned`, ask the user to approve the transition to `in-progress` before updating the status. On confirmation, update the frontmatter `status` field to `in-progress`.

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
   - As you write or modify each file, mentally tag the edit with the acceptance criterion (or criteria) the task serves. Maintain a running map from each AC to the set of files edited in service of it. See **Code-location index** below for the mapping rules and output format.
   - If you need to modify files outside the plan's affected files list, notify the user, explain why, and add the file to the plan's **Affected Files** section with a comment explaining why it was added.
5. Verify the "done when" condition is met.
6. Mark the task as complete in `tasks.md` — update each checkbox from `- [ ]` to `- [x]`, including nested sub-item checkboxes, before proceeding.
7. Regenerate `specs/{feature}/code-locations.md` from the running map per the **Code-location index** section. Run `npx markdownlint-cli2` on the file.
8. Prompt the user to commit and push changes. With `--auto` set, skip the prompt: commit on your own, do not push.
9. Before starting the next task, assess whether sufficient context remains to complete it. If context is low, inform the user and suggest starting a new session with `/gov:implement` to continue from the next incomplete task. If context is sufficient, proceed.

### Code-location index

`/gov:implement` produces and maintains a per-spec `code-locations.md` artifact at `specs/{feature}/code-locations.md`. The artifact is a structured derived view: it ties each acceptance criterion in the spec to the source files that satisfy it. Reference: §text-first-artifacts (markdown derived views may be committed when their diffs are valuable to humans).

#### Building the map

As you walk tasks and edit files, maintain an in-memory `Map<AC, Set<file>>` for the feature:

- When you start a task, identify which acceptance criterion (or criteria) it serves. Tasks usually map to one or two ACs; if a task covers more, list them all.
- When you create or modify a file in service of the task, add the file to each associated AC's set in the map.
- When resuming a feature in a subsequent `/gov:implement` session, read the existing `code-locations.md` (if any) and seed the in-memory map from it before continuing.

#### Output format

The artifact format is:

```markdown
# {NNN} — {Feature Name} Code Locations

## AC: {first acceptance criterion text}

- `path/to/file.ext`
- `path/to/another.ext`

## AC: {second acceptance criterion text}

- `path/to/file.ext`
```

Rules:

- AC headings appear in the order they appear in the spec's Acceptance Criteria section (deterministic ordering ensures stable diffs).
- File paths within each AC are alphabetical.
- Path format is repository-relative (e.g., `framework/commands/implement.md`, not absolute or workspace-relative).
- ACs with no associated files are omitted (no empty heading, no placeholder bullet).
- The same file may appear under multiple ACs if it serves more than one.
- The file is committed to git so its diff is reviewable in PRs and so subsequent `/gov:implement` sessions can read prior state when resuming.

Idempotent — regenerating with the same map produces an identical file with no diff.

### Completion

After all tasks are done:

1. Walk through each acceptance criterion from the spec and verify it is met. Mark each passing criterion `- [x]` in the spec file at the time of verification. If a criterion fails, leave it as `- [ ]` and report the failure. Do not batch-mark — verify each individually.
2. Run the validation gate before proposing the status transition:
   - All tasks in `tasks.md` are marked `- [x]`
   - All acceptance criteria in the spec are marked `- [x]`
   - All scenario-linked tasks are complete
   - All `.md` files in the feature directory pass `npx markdownlint-cli2`
3. If any validation check fails, report the specific failures and do not propose the transition. The user fixes the issues and re-runs the command.
4. If all checks pass, present a summary and ask the user to approve the transition to `done`. Do not update the status until the user confirms.
5. On confirmation, update the spec's frontmatter `status` field from `in-progress` to `done`.
