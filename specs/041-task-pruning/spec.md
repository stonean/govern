---
status: in-progress
dependencies: [022-deterministic-runtime]
review:
  last-run: 2026-07-11T12:12:24Z
  reviewed-against: c0bc8697bdef33bbb2024585fa75bd4299889fca
  must-violations: 0
  should-violations: 1
  low-confidence: 2
  blocking: false
---

# 041 — Task Pruning

A `/{project}:prune` command that reduces a feature's `tasks.md` — dropping spent, completed work and optionally resetting the file to its empty template state — so the working task list stays a view of *what is left to do* rather than an ever-growing ledger of everything ever done.

## Motivation

A feature's `tasks.md` accumulates work items across the whole life of the feature. Every reopen back-edge (a new scenario, a meaningful body edit moving `done` → `in-progress`), every `/{project}:amend`, and every `/{project}:plan`/`/{project}:implement` cycle appends more tasks. Over time the file grows large, and much of it is stale: completed checkboxes for work already merged, and sections describing behavior that later edits removed or superseded.

That accumulation has no durable value. A task's purpose is spent the moment it is complete — the durable record of *what was built* lives in the spec, its scenarios, the code, and git history, not in a checked-off checkbox. The constitution codifies this directly — §tasks-phase classifies `tasks.md` as an ephemeral work-tracking artifact: transient work whose value is spent once complete does not belong in a persistent artifact (the same durability test §bug-handling applies to chores). A bloated `tasks.md` makes the "what's left" view noisy and harder to work from during `/{project}:implement`, and it obscures the small set of genuinely pending items under a wall of finished ones.

`govern` has no command to reclaim that space. `/{project}:prune` fills the gap: a deliberate, confirmed reduction of `tasks.md` back toward a lean working set — or all the way back to the template's initial state — recovering the file's usefulness without losing anything that matters, because history and the spec already hold the record.

## Behavior

`prune` operates on the current session target's `tasks.md` (the same session-target resolution every pipeline command uses). It reads the existing task list, distinguishes spent work from pending work, and rewrites the file to a reduced form. The reduction is size-significant: a `tasks.md` full of completed sections comes back materially smaller, and in the limit resets to the template's initial state (top-level heading plus the guidance comment).

The distinction that drives the reduction is completion. A task section whose checkboxes are all checked is spent — its work is merged and recorded elsewhere, so removing it loses nothing recoverable outside git. A section with any unchecked checkbox represents pending work; dropping it would silently lose a todo, which §pipeline-boundaries ("don't backtrack silently") forbids.

Because prune destructively rewrites a working artifact, it confirms with the user before writing and never leaves a gitignored backup sidecar — recovery is git history, consistent with §text-first-artifacts (source-of-truth artifacts are plain markdown; derived/backup state is not smuggled in beside them). Whatever prune leaves behind is a valid `tasks.md`: it parses, starts with the template heading, and still satisfies the task-consistency checks `/{project}:analyze` runs for a `planned` or `in-progress` spec.

The command's scope is `tasks.md` only. It does not edit the plan, the spec, scenarios, or status. It is a maintenance/hygiene command over one artifact, not a pipeline state transition.

### Framework consistency — tasks.md is ephemeral tracking

Prune formalizes that `tasks.md` is disposable, so the framework must treat it consistently as ephemeral tracking rather than durable information. Three alignments make that true end to end:

- The constitution **canonically classifies** `tasks.md` as an ephemeral work-tracking artifact — a view of what is left to do — distinct from the durable spec, scenarios, and rules (with `plan.md` and `data-model.md` as design records). Prune and this spec cite that classification directly, not by analogy to the chore durability test.
- The shared `tasks.md` parsers **ignore HTML-comment content** (`<!-- … -->`), so a reset (template-state) file is genuinely empty to the runtime primitives just as it is to a markdown reader — `read-tasks` reports zero tasks and `append-task` numbers from 1. Without this, the reset target's commented example headings parse as phantom tasks and split the two-paths guarantee.
- `/{project}:analyze` **does not treat the scenario→task linkage as a durable index**. A `done` spec whose implemented scenario tasks have been pruned is not a drift finding: the durable record of an implemented scenario is the scenario file, the code, and git history — not a retained checkbox.

## Acceptance Criteria

<!-- Greenfield-leaning feature, but several genuine design decisions (reset-vs-prune
     default, status-gating, partial-section handling, runtime eligibility) are
     deferred to Open Questions and resolved by /{project}:clarify. The criteria
     below capture the behavior that holds regardless of how those resolve. -->

- [x] A `/{project}:prune` command exists and, with no argument, operates on the current session target's `tasks.md`.
- [x] Running prune on a `tasks.md` that contains completed task sections produces a materially smaller file.
- [x] Prune preserves every incomplete task section — a section with any unchecked checkbox is never silently removed.
- [x] Prune's output is a valid `tasks.md`: it starts with the template's top-level heading, passes `npx markdownlint-cli2`, and passes the `/{project}:analyze` task-consistency checks for the feature's current status.
- [x] Prune requires explicit user confirmation before writing the reduced file; declining leaves `tasks.md` unchanged.
- [x] A full reset restores `tasks.md` to the template's initial state (heading plus guidance comment) with no residual task entries.
- [x] Pruned content is recoverable only from git history — prune writes no backup file or gitignored sidecar.
- [x] `/{project}:prune --reset` on a non-`done` spec refuses — naming the current status and pointing at the keep-pending default — and writes nothing unless `--force` is also supplied.
- [x] Running prune when no task section is spent makes no write and reports that there is nothing to prune.
- [x] Prune stops without writing when there is no session target (directs to `/{project}:target`) or the target has no `tasks.md` (directs to `/{project}:plan`).
- [x] The reduction is produced and written by the deterministic runtime prune primitive, with the file body never round-tripped through agent context; the markdown-only fallback reaches identical bytes.
- [x] The constitution canonically classifies `tasks.md` as an ephemeral work-tracking artifact, distinct from the durable spec / scenarios / rules — stated directly, not only by analogy to the chore durability test.
- [x] The shared `tasks.md` parsers ignore content inside HTML comments (`<!-- … -->`): a reset (template-state) `tasks.md` parses to zero tasks via `read-tasks` and yields task number 1 from `append-task`, matching what a markdown reader sees.
- [x] `/{project}:analyze` does not report a scenario-consistency inconsistency when a `done` spec's scenario-linked tasks have been pruned; the scenario→task linkage is not required to persist after the scenario is implemented.

## Edge Cases

- **Already lean / template-state.** When keep-pending finds no spent section, prune makes no write and reports nothing to prune; `--reset` on an already template-state file is an idempotent no-op.
- **Missing `tasks.md`.** Prune stops and directs the user to `/{project}:plan`; it never creates a task list from nothing.
- **No session target.** Prune stops and directs the user to `/{project}:target`.
- **Sections with no checkboxes.** Structural or prose blocks (including the top-level heading and the template guidance comment) carry no checkbox, so they are never classified spent — spent requires ≥1 checkbox, all checked — and are always preserved.
- **Section fully checked with no other content.** Classified spent and removed in full by keep-pending.
- **Malformed `tasks.md`.** If the file does not parse or lacks the template's top-level heading, the primitive errors and writes nothing rather than emit a corrupt file.
- **`done` spec with hand-edited unchecked boxes.** `--reset` is gated on status alone, so it proceeds and drops them — the user's explicit reset intent on a `done` spec; the safe default keep-pending prune still preserves them.
- **Concurrent invocations.** Atomic tempfile + rename yields last-writer-wins with no partial file left behind.

## Open Questions

<!-- All open questions must be resolved before moving to the plan phase. -->

*None — all resolved.*

## Resolved Questions

- **Prune vs. reset — one mode or two?** → One command, two modes, safe default. `/{project}:prune` with no flag performs a **keep-pending prune**: it drops every fully-completed task section and preserves any section holding at least one unchecked checkbox. `/{project}:prune --reset` performs a **full reset** to the template's initial state (top-level heading + guidance comment) regardless of pending work. Keep-pending is the default because the zero-flag invocation must never silently drop a live todo (§pipeline-boundaries "don't backtrack silently"); the destructive full reset requires explicit `--reset` intent. Reset is the limit case of prune, so both live in one command rather than two. Both modes are deterministic runtime operations (see the runtime-eligibility resolution).
- **Is reset gated on spec status?** → Yes, deterministically, with an explicit override. `--reset` is permitted when the spec status is `done`. On a `draft` / `clarified` / `planned` / `in-progress` spec, `--reset` **refuses** — it does not silently degrade to a keep-pending prune — and stops with a message that names the current status, points at the default keep-pending `/{project}:prune` as the likely intent, and documents `--reset --force` as the explicit escape hatch for the deliberate non-`done` reset. The confirmation prompt alone is not sufficient: a prompt is reflexively accepted, whereas the status gate is a deterministic guardrail the runtime enforces on every invocation. The gate decision is computed purely from frontmatter `status`, so it lives in the runtime primitive (allow / block-needs-override), and the command surfaces it.
- **Partial-section granularity.** → The **task section** is the atomic unit; partial sections are left entirely untouched. The smallest unit prune removes is a heading-delimited task section (a task heading and everything under it up to the next heading of equal-or-higher level), never an individual checkbox. Per section: all checkboxes checked (and at least one present) → the section is spent and removed in full; any unchecked checkbox → the section is pending and kept verbatim, including its already-completed checkboxes. Prune never reaches inside a pending section to strip checked items — "all boxes checked → drop the section; else keep byte-for-byte" is a judgment-free rule the runtime computes directly, whereas rewriting a live section's interior (renumbering, intra-section references) would drag fragility and judgment into a mechanical operation. The completed items inside a half-done section also carry working context worth keeping. The exact section-boundary grammar (which heading level delimits a task section in the `tasks.md` template) is pinned down in `plan.md` / `data-model.md`; this resolution fixes the unit and the rule.
- **Re-derivation contract.** → An empty template-state `tasks.md` is valid *at rest* but not *runnable*. (1) At rest it is well-formed: it passes `markdownlint` and is treated as vacuously consistent by `/{project}:analyze` — no tasks means nothing can conflict with the spec or its status — which keeps the "output passes the analyze task-consistency checks" acceptance criterion true for the reset case. (2) It is not a valid input to `/{project}:implement`: implement's existing gate finds no unchecked tasks and directs the user back to `/{project}:plan` to repopulate. Prune does not auto-invoke plan and does not change status; re-derivation is an explicit, separate `/{project}:plan` step. (3) No silent stranding: because `--reset` is gated to `done` specs, the only path to an empty `tasks.md` on a live `planned`/`in-progress` spec is the deliberate `--reset --force`, a conscious choice to discard and re-plan. This separates concerns cleanly — `analyze` checks consistency (empty is consistent), `implement` checks runnability (empty routes to plan) — and preserves the `plan → tasks → implement` invariant without prune knowing anything about status transitions.
- **Runtime eligibility.** → Prune ships a deterministic runtime primitive that also performs the write — the runtime exists to do the deterministic work *and* to keep bulky content out of model context (token reduction), so a preview-only primitive that returned the proposed body to the model (round-tripping the whole file twice) is rejected. The primitive — tentatively `prune-tasks` (feature + mode `keep-pending`|`reset` + an `apply` flag), building on [022-deterministic-runtime](../022-deterministic-runtime/spec.md) — is fully deterministic in two modes. `apply: false` (preview) parses `tasks.md` into task sections, classifies each as spent (≥1 checkbox, all checked) or pending (any unchecked), computes the `--reset` status-gate decision, and returns **only a compact summary** (current status, gate decision `allowed`/`blocked-needs-force`, per-section classification, removed/kept counts, size before→after) — never the file body. `apply: true` (commit) re-reads, recomputes, and **writes** the reduced `tasks.md` directly via atomic tempfile + rename, returning only a small write-confirmation; the reduced content is produced and written entirely inside the runtime and never enters model context. The Q1 confirmation gate is preserved without the token cost: the command calls preview, shows the compact summary, takes the yes/no, then calls apply. The command layer keeps only judgment — *whether* to prune and relaying the confirmation; everything mechanical (parse, classify, gate, rewrite, write) is in the primitive. The markdown-only fallback reaches the same bytes by hand (§runtime-host-integration; neither path wraps the other).
- **Relationship to `/{project}:groom`.** → Dedicated `/{project}:prune` command, not folded into groom. The two differ in artifact and scope (groom operates on the repo-level `inbox.md`; prune on a specific feature's `tasks.md`, resolved through the session target — repo-scoped vs. target-scoped, different files) and in verb (groom *routes/classifies* inbox items to their proper homes, judgment-heavy LLM work; prune *reduces* one file via a deterministic, confirmed runtime write). Folding a destructive target-scoped reduction into groom's inbox routing would overload one command with two unrelated mental models; a dedicated command keeps each single-purpose, matching how `/{project}:target`, `/{project}:plan`, and `/{project}:implement` each own one job and leaving groom's "walk the inbox, route each item" contract intact.
- **Scope confirmation.** → Single-artifact scope is a hard boundary: prune touches `tasks.md` and nothing else — never `plan.md`, `spec.md`, `data-model.md`, scenario files, or frontmatter `status`. This promotes the Behavior section's assertion (the command's scope is `tasks.md` only) from description to invariant. A single-file atomic write is trivially reviewable in git, whereas reaching into `plan.md` to reconcile now-removed tasks requires interpretive judgment a mechanical hygiene command must not do and would bloat the primitive's write surface. Dangling plan references are acceptable — `plan.md` is a design/approach record, not a live checklist — and genuine plan↔tasks drift is already surfaced by `/{project}:analyze` as an advisory finding; prune deliberately does not chase cross-artifact edits. A one-file write surface keeps `prune-tasks` and its markdown-only twin easy to match byte-for-byte.
