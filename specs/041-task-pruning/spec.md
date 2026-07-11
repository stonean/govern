---
status: draft
dependencies: [022-deterministic-runtime]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 041 — Task Pruning

A `/{project}:prune` command that reduces a feature's `tasks.md` — dropping spent, completed work and optionally resetting the file to its empty template state — so the working task list stays a view of *what is left to do* rather than an ever-growing ledger of everything ever done.

## Motivation

A feature's `tasks.md` accumulates work items across the whole life of the feature. Every reopen back-edge (a new scenario, a meaningful body edit moving `done` → `in-progress`), every `/{project}:amend`, and every `/{project}:plan`/`/{project}:implement` cycle appends more tasks. Over time the file grows large, and much of it is stale: completed checkboxes for work already merged, and sections describing behavior that later edits removed or superseded.

That accumulation has no durable value. A task's purpose is spent the moment it is complete — the durable record of *what was built* lives in the spec, its scenarios, the code, and git history, not in a checked-off checkbox. This mirrors the durability test the constitution applies to chores (§bug-handling): transient work whose value is spent once complete does not belong in a persistent artifact. A bloated `tasks.md` makes the "what's left" view noisy and harder to work from during `/{project}:implement`, and it obscures the small set of genuinely pending items under a wall of finished ones.

`govern` has no command to reclaim that space. `/{project}:prune` fills the gap: a deliberate, confirmed reduction of `tasks.md` back toward a lean working set — or all the way back to the template's initial state — recovering the file's usefulness without losing anything that matters, because history and the spec already hold the record.

## Behavior

`prune` operates on the current session target's `tasks.md` (the same session-target resolution every pipeline command uses). It reads the existing task list, distinguishes spent work from pending work, and rewrites the file to a reduced form. The reduction is size-significant: a `tasks.md` full of completed sections comes back materially smaller, and in the limit resets to the template's initial state (top-level heading plus the guidance comment).

The distinction that drives the reduction is completion. A task section whose checkboxes are all checked is spent — its work is merged and recorded elsewhere, so removing it loses nothing recoverable outside git. A section with any unchecked checkbox represents pending work; dropping it would silently lose a todo, which §pipeline-boundaries ("don't backtrack silently") forbids.

Because prune destructively rewrites a working artifact, it confirms with the user before writing and never leaves a gitignored backup sidecar — recovery is git history, consistent with §text-first-artifacts (source-of-truth artifacts are plain markdown; derived/backup state is not smuggled in beside them). Whatever prune leaves behind is a valid `tasks.md`: it parses, starts with the template heading, and still satisfies the task-consistency checks `/{project}:analyze` runs for a `planned` or `in-progress` spec.

The command's scope is `tasks.md` only. It does not edit the plan, the spec, scenarios, or status. It is a maintenance/hygiene command over one artifact, not a pipeline state transition.

## Acceptance Criteria

<!-- Greenfield-leaning feature, but several genuine design decisions (reset-vs-prune
     default, status-gating, partial-section handling, runtime eligibility) are
     deferred to Open Questions and resolved by /{project}:clarify. The criteria
     below capture the behavior that holds regardless of how those resolve. -->

- [ ] A `/{project}:prune` command exists and, with no argument, operates on the current session target's `tasks.md`.
- [ ] Running prune on a `tasks.md` that contains completed task sections produces a materially smaller file.
- [ ] Prune preserves every incomplete task section — a section with any unchecked checkbox is never silently removed.
- [ ] Prune's output is a valid `tasks.md`: it starts with the template's top-level heading, passes `npx markdownlint-cli2`, and passes the `/{project}:analyze` task-consistency checks for the feature's current status.
- [ ] Prune requires explicit user confirmation before writing the reduced file; declining leaves `tasks.md` unchanged.
- [ ] A full reset restores `tasks.md` to the template's initial state (heading plus guidance comment) with no residual task entries.
- [ ] Pruned content is recoverable only from git history — prune writes no backup file or gitignored sidecar.

## Open Questions

<!-- All open questions must be resolved before moving to the plan phase. -->

- **Prune vs. reset — one mode or two?** The description spans "significantly reduce" and "reset to template state." Are these two invocations of one command (e.g., a default prune that keeps pending work vs. an explicit full reset), or does a single behavior cover both? What is the default when the user gives no flag?
- **Is reset gated on spec status?** A full reset is unambiguously safe when the spec is `done` (all tasks complete, record lives in spec + code + history). When the spec is `in-progress` with pending tasks, a full reset would drop live todos. Should reset refuse (or degrade to a keep-pending prune) unless the spec is `done`, or is the confirmation prompt sufficient protection?
- **Partial-section granularity.** When a section mixes checked and unchecked items, does prune keep the whole section untouched, strip only the completed checkboxes within it, or leave partial sections entirely alone? What is the smallest unit prune removes — a checkbox, a numbered task section, or a heading group?
- **Re-derivation contract.** After a reset, must the feature return through `/{project}:plan` to repopulate `tasks.md` before `/{project}:implement` can proceed, or is an empty template-state `tasks.md` a valid starting point mid-feature? This determines whether prune can strand a `planned`/`in-progress` spec with no runnable tasks.
- **Runtime eligibility.** Identifying and removing fully-completed sections is deterministic and currently-mechanical, which makes it a candidate primitive under the runtime boundary (§runtime-boundary, [022-deterministic-runtime](../022-deterministic-runtime/spec.md)), while deciding *whether* to prune stays a human/LLM judgment. Should prune ship a deterministic prune/reset primitive, or remain a markdown-only command? Either way the markdown-only path must reach the same result.
- **Relationship to `/{project}:groom`.** `/{project}:groom` already routes and cleans the inbox. Is task pruning conceptually adjacent enough to fold in, or is a dedicated command clearer? (Leaning dedicated — groom operates on `inbox.md`, prune on a feature's `tasks.md` — but worth confirming.)
- **Scope confirmation.** Should prune ever touch artifacts beyond `tasks.md` (e.g., plan sections that enumerate now-removed tasks), or is single-artifact scope a hard boundary?
