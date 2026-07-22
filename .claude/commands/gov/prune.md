---
description: Prune a feature's tasks.md — drop spent task sections, or reset to template state.
argument-hint: "[--reset] [--force]"
---

# Prune

Reduce the session target's `tasks.md` so the working list stays a view of *what is left to do* — dropping spent, completed task sections, or resetting the file to its template initial state.

## Purpose

A feature's `tasks.md` accumulates completed work across the whole life of the feature; a task's value is spent the moment it is complete (the durable record lives in the spec, the code, and git history). `/gov:prune` reclaims that space — a deliberate, confirmed reduction of `tasks.md` back toward a lean working set, or all the way back to the template's initial state. It is a maintenance command over one artifact, not a pipeline state transition.

## Scope Boundaries

- The only file written is the session target's `tasks.md`. Do NOT edit the plan, the spec, scenarios, `data-model.md`, or the frontmatter `status` — single-artifact scope is a hard boundary.
- Recovery is git history: prune writes no backup file and no gitignored sidecar.
- Prune never changes pipeline status and never advances or reverts the lifecycle.
- Reference: §tasks-phase (`tasks.md` is an ephemeral work-tracking artifact, safe to prune — not a durable source of truth), §pipeline-boundaries ("don't backtrack silently"), §text-first-artifacts, §runtime-boundary, plus [041 — Task Pruning](../../specs/041-task-pruning/spec.md) for the reduction semantics and [data-model](../../specs/041-task-pruning/data-model.md) for the segmentation and classification.

## Instructions

> **For agent runtimes**: the Invoke steps below call the MCP tools of the optional gvrn runtime; the host-integration contract — bare↔prefixed tool names, lazy ToolSearch schema fetch, the no-shell-utilities rule, and the two-paths guarantee — lives once in the constitution, §runtime-host-integration. With no gvrn MCP server registered, walk the same prose using the host file-reading tools (Read, Edit, Write) per the markdown-only reference below.

<!-- audit:ignore-promotion -->
1. Resolve the session target from `.govern/session.toml`. If no target is set, stop and tell the user to run `/gov:target` first. Parse the invocation flags: `--reset` selects a full reset (default is a keep-pending prune); `--force` overrides the reset status gate on a non-`done` spec. `--force` without `--reset` is ignored (it only gates reset).

2. Invoke `prune-tasks` against the target feature in preview mode (`apply: false`), passing the `reset` and `force` flags. The result is a compact summary — mode, the `--reset` gate outcome, the per-section classification (`spent` / `pending` / `no-checkbox`), the removed/kept counts, and the size before/after — and never carries the file body. When the target has no `tasks.md`, stop and direct the user to run `/gov:plan` (there is no task list to prune yet — the MCP surface returns this as a `tasks.md not found: …` error); when the feature directory does not exist, direct the user to `/gov:target` (a `feature directory not found: …` error). These are operational errors carrying a Display message — the `tasks-file-missing` / `feature-not-found` names label the error variants, they are not literal tokens in the payload.

<!-- audit:ignore-promotion -->
3. Render the preview for the user from the summary: the mode, the size before → after, the removed/kept counts, and one line per task section with its classification and action. Two early exits, neither of which writes: when `nothing-to-prune` is true, report that there is nothing to prune and stop; when the gate is `blocked-needs-force` (a `--reset` on a non-`done` spec), name the current status, point at the default keep-pending `/gov:prune` as the likely intent, note `--reset --force` as the explicit escape hatch, and stop.

4. Invoke `gate-confirm` with a prompt that names the destructive write (the mode and how much the file shrinks). Prune rewrites a working artifact, so it confirms before writing; on a declined gate, leave `tasks.md` unchanged and stop.

5. On confirmation, invoke `prune-tasks` again with `apply: true` (same `reset` / `force` flags) to perform the atomic write. Report the outcome — the file's new size and the sections removed — from the returned summary.

<!-- audit:ignore-promotion -->
6. Confirm the single-artifact result: only `tasks.md` changed. A plan that still enumerates now-removed tasks is not reconciled here; genuine plan↔tasks drift is surfaced by `/gov:analyze` as an advisory finding, not by prune.

## Markdown-only reference

With no gvrn runtime registered, the host reaches the same result with its own file tools — no shell-pipeline substitution — producing byte-for-byte the output the `prune-tasks` primitive would write (the two-paths guarantee, §runtime-host-integration).

Segment `tasks.md` with the same grammar every tasks command uses (see [data-model](../../specs/041-task-pruning/data-model.md)): detect flat (`## N.`) versus phased (`### N.` under `## …` containers), then split the file into its preamble, phase containers, and task sections. Classify each task section by its checkboxes — **spent** (≥ 1 checkbox, all checked), **pending** (any unchecked), or **no-checkbox** (zero checkboxes) — counting only real task-list checkboxes (a `- **Done when**:` line is not one).

- **keep-pending** (default): preserve the preamble and every pending / no-checkbox section verbatim; drop every spent section; in a phased file drop a phase container left with no surviving task section. When nothing is spent, leave the file byte-for-byte unchanged. Normalize seams to a single blank line with one trailing newline.
- **reset** (`--reset`): preserve the file's existing `# …` heading and replace everything below it with the template's initial tasks body (the intro line plus the guidance comment). On the runtime path this body is **compiled into** `prune-tasks`, pinned to `framework/templates/spec/tasks.md`, so an adopter who customizes `specs/templates/tasks.md` gets the framework body on the runtime path and their own template only on this markdown-only path. Refuse unless the spec status is `done` or `--force` is supplied; a file with no `# …` heading is malformed and is left untouched.

Confirm with the user before writing, write atomically (a temp file then rename), and leave no backup — recovery is git history.
