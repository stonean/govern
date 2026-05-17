---
description: Execute implementation tasks for the targeted feature.
argument-hint: "[--auto] [feature]"
parity:
  strict-fields:
    - task-checkbox-state
  strict-files:
    - "specs/{feature}/tasks.md"
  semantic-fields:
    - "code-edits[].content"
---

# Implement

Execute implementation tasks for the targeted feature.

## Purpose

Pipeline gate: planned → in-progress → done. Walks through `tasks.md` step by step, implementing each task according to the plan. This is the only command that writes application code.

## Context

Use the session target from `{cli-config-dir}/{project}-session.json`. If `$ARGUMENTS` is provided, use it to override the session target. If no session target is set and no arguments provided, stop and tell the user to run `/{project}:target` first.

### Flags

`$ARGUMENTS` may include the `--auto` flag in any position. Strip it before treating remaining text as a feature override. The flag is per-invocation and is not persisted to the session file — autonomy is an execution-time decision, not session state.

When `--auto` is set:

- Skip the per-task "prompt the user to commit and push changes" confirmation. Commit on your own and proceed to the next task.
- **Commit, do not push.** Push is hard-to-reverse and externally visible; it stays gated even with `--auto`.

The following gates **still fire and pause** even with `--auto` on:

- Pipeline gates (planned → in-progress, in-progress → done) — confirmation required per §pipeline-boundaries.
- Stuck-detection events — auto mode does not power through cycles.
- Out-of-bounds file writes — modifying a file outside the runtime boundary still requires user notification.
- Spec edits, plan edits, or new tasks discovered mid-implement.
- Risky actions per the agent's safety rules (destructive ops, secrets, force pushes, etc.).

Default is unset — without the flag, the user confirms each task as today.

## Scope Boundaries

- The runtime write boundary is derived in step 2 from git history; the plan's **Affected Files** section is a planning aid, not authoritative.
- Do NOT read or modify files belonging to other features' spec directories.
- Do NOT read source code speculatively — only read files relevant to the current task.
- Reference: §implement-phase, §pipeline-boundaries, §text-first-artifacts, plus `framework/rules/configuration-cross.md` for constants and env-vars (constitution loaded by `/{project}:target` — do not re-read).

## Instructions

> **For agent runtimes**: backticked primitive names in this section map to MCP tools the optional [gvrn runtime](https://crates.io/crates/gvrn) exposes under bare `<primitive>` names (e.g., `read-tasks`). Hosts wrap them with a server-name prefix taken from `.mcp.json` (Claude: `mcp__gvrn__read-tasks`; Auggie: `mcp:gvrn:read-tasks`). When the server is registered for your session, **call the corresponding tool** for each step listed below — that is the deterministic path. When the server is not registered, walk the prose to produce the same result. The two paths share a contract; neither one wraps the other.

1. Invoke `read-tasks` (MCP: `read-tasks`) against the targeted feature to load the ordered task list and the per-task "done when" conditions. The walker also seeds the session target's feature, scenario fields, and writeCode arguments (task-number, subtask-index, checked, write-boundary, threshold) from the runtime context.

2. Invoke `derive-boundary` (MCP: `derive-boundary`) against the feature to compute the runtime write boundary from `git diff` against the spec dir's first commit; the result emits as a progress envelope and the host stores the boundary in context for the writeCode validator below. Otherwise, follow the markdown-only path: compute the same diff with the host's shell.

3. Invoke `check-stuck` (MCP: `check-stuck`) against the feature with a threshold of 3 to detect stuck cycles before starting work. When the result reports stuck, surface the cycle to the user and pause for direction before proceeding — auto mode does not power through cycles. Otherwise, follow the markdown-only path: count commits on `tasks.md` since the spec entered in-progress.

4. Ask the user to approve the transition from planned to in-progress before any code changes. On confirmation, continue to step 5; on denial, the walker exits cleanly without modifying the spec.

5. Invoke `set-status` (MCP: `set-status`) to flip the spec frontmatter's status from planned to in-progress; the primitive guards against a stale "from" value so concurrent edits surface as an operational error rather than a silent overwrite.

6. <!-- llm:writeCode --> Implement the first incomplete task. The host receives the task description, plan-relevant files, the derived write boundary, and constitution excerpts; it returns an edits array plus a one-line summary. The walker validates every edit's path against the write boundary and emits an `out-of-boundary-edit` error envelope (halting the procedure) when any edit escapes the boundary. Otherwise, follow the markdown-only path: read the plan, write code, run tests.

7. Invoke `mark-task` (MCP: `mark-task`) to flip the first incomplete subtask's checkbox from unchecked to checked in `tasks.md` (atomic write via tempfile + rename). The primitive returns the previous and current states; a previous value of `true` surfaces as a no-op result.

8. Render the completion summary (host responsibility): list the task processed, surface the cross-spec impact diff (any changes outside `specs/{feature}/`), remind the user to commit, and prompt for the next pipeline gate. The in-progress → done transition is its own invocation — re-run `/{project}:implement` after every task has been marked complete and review is clean.

## Markdown-only reference

The full setup, walk-through, completion gate, and stuck-detection details are documented below for the markdown-only path. The numbered steps above invoke the mechanical primitives that automate each phase; the host applies the same procedure against the markdown-only path when the runtime is unavailable.

### Setup details

- Read `{cli-config-dir}/{project}-session.json` for the session target, including optional `scenario` and `scenarioPath` fields.
- Read `specs/{feature}/tasks.md` for the ordered task list (primitive: `read-tasks`).
- Read `specs/{feature}/plan.md` for technical decisions and affected files.
- Read the spec file for acceptance criteria and contracts.
- If a scenario is targeted, read the scenario file for scenario-specific context, behavior, and edge cases. The scenario scopes which part of the feature is the primary focus for this implementation session.
- **Recompute dependencies (safety net).** Run `scripts/gen-spec-deps.sh --dry-run` against the target spec; if it reports a diff, run it for real to sync `dependencies:` from body inline links.

### Stuck-detection details

If the spec's status is already in-progress, run `git log --oneline -- specs/{feature}/tasks.md` and count commits since the spec entered in-progress. Identify the first incomplete task (first `- [ ]` checkbox group) in `tasks.md`. If `git log` shows ≥ 3 commits on `tasks.md` AND the same task is still the first incomplete one (no checkbox flipped to `- [x]` between those commits for that task), surface the cycle to the user with this message: `Task {N} ({title}) has been touched in {count} prior implement runs without completing. Consider decomposing it into smaller subtasks before continuing.` Pause and wait for user direction; do not auto-decompose. The threshold of 3 is fixed (not configurable in v1) — smallest count that distinguishes routine multi-session work from a cycle.

### Progressive context loading

- **At setup:** read only the spec, plan, tasks, and scenario file (if targeted). Do NOT read `system.md`, `events.md`, `errors.md`, or source code yet.
- **Per task:** read only the source files relevant to that task from the plan's affected files list.
- **At completion:** re-read acceptance criteria from the spec to verify. Do NOT re-read the full plan or tasks.

### Walk through tasks (per task, in order)

1. Display the task number, description, and "done when" condition.
2. Read the relevant technical decisions from the plan.
3. Read only the existing code files relevant to this task from the plan's affected files.
4. Implement the task: write code, tests, and migrations as needed. Follow conventions in `AGENTS.md` and `specs/system.md`; respect the contracts defined in the spec. If a write would land outside the runtime boundary, notify the user, explain why, and wait for confirmation before proceeding. Once accepted, the file is part of the boundary for the rest of the session.
5. Verify the "done when" condition is met.
6. Mark the task as complete in `tasks.md` — update each checkbox to `- [x]`, including nested sub-item checkboxes, before proceeding.
7. Prompt the user to commit and push changes. With `--auto` set, skip the prompt: commit on your own, do not push.
8. Before starting the next task, assess whether sufficient context remains to complete it. If context is low, suggest starting a new session.

### Completion gate (after all tasks)

1. **Cross-spec impact check.** Run `git diff --stat <first-commit>..HEAD -- specs/`, filtered to paths outside `specs/{feature}/`. If any sibling spec dir shows changes, surface the list to the user and ask whether the changes were intentional cross-spec updates per §cross-spec-impact. Informational; does not block.
2. Walk through each acceptance criterion from the spec and verify it is met. Mark each passing criterion `- [x]` in the spec file at the time of verification. If a criterion fails, leave it unchecked and report the failure. Do not batch-mark — verify each individually.
3. Run the validation gate before proposing the status transition:
   - All tasks in `tasks.md` are marked complete.
   - All acceptance criteria in the spec are marked complete.
   - All scenario-linked tasks are complete.
   - All `.md` files in the feature directory pass `npx markdownlint-cli2`.
4. If any validation check fails, report the specific failures and do not propose the transition. The user fixes the issues and re-runs the command.
5. **Pre-done review gate.** Read the target spec's frontmatter `review:` block before asking for the done transition. If `review.last-run` is missing, null, or the review block is absent, halt with: `blocked: spec has not been reviewed — run /gov:review before completing`. If `review.blocking: true`, halt with: `blocked: spec has {must-violations} MUST violation(s) — see specs/NNN-feature/review.md` followed by guidance to either resolve the violations and re-run `/gov:review`, or run `/gov:review --waive <rule-id> --reason "..."` for each waivable finding. Otherwise, proceed.
6. If all checks pass, present a summary and ask the user to approve the transition to done. Do not update the status until the user confirms.
7. On confirmation, update the spec's frontmatter status from in-progress to done.
