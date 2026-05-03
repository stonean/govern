---
status: done
dependencies: [000-slash-commands, 009-scenario-targeting, 013-text-first-artifacts]
tags: [pipeline, commands]
---

# 014 — Re-clarify Back-Edge

Wire up `/ask` to own the `clarified` / `planned` / `in-progress` → `draft` back-edge so questions surfacing mid-pipeline are captured and the spec's lifecycle invariant is maintained automatically. When a question surfaces on a `clarified`, `planned`, or `in-progress` spec, `/ask` records the question and reverts status to `draft` — the only state that tolerates open questions per the constitution. The next `/clarify` resolves the question and the spec advances forward again. No flag, no manual frontmatter editing, no inconsistent intermediate state.

## Problem

Constitution §spec-lifecycle (`framework/constitution.md` lines 96–99) defines two back-edges:

1. `done → in-progress` via `/elaborate` adding a scenario.
2. `planned/in-progress → clarified` via `/ask` recording a new open question.

Back-edge 1 is implemented in `framework/commands/elaborate.md` step 7. Back-edge 2 is not implemented anywhere:

- `framework/commands/ask.md` "Status warning" only warns the user and explicitly states "Do not change the status — this is informational only."
- `framework/commands/clarify.md` Gate refuses any non-`draft` status with "Spec is already clarified. Run `/{project}:plan`…".

A user who runs `/ask` on an `in-progress` spec gets a question recorded plus a hint to re-run `/clarify` — but `/clarify` then refuses to proceed. The documented back-edge has no working entry point. Worse, the spec ends up in an internally inconsistent state: `clarified` (or later) status, but with unresolved open questions in the body.

The behavior is not academic. The most common moment a question surfaces is during `/implement`, when the abstract spec collides with concrete code. Adopters need a way to capture the question, see the implications, and resume work — without bookkeeping or workarounds.

## Behavior

### Ownership

`/ask` becomes the owner of the `planned/in-progress → clarified` back-edge. When `/ask` records a new open question on a `clarified`, `planned`, or `in-progress` spec, it reverts status to `draft` as part of the same write — capturing the question and fixing the lifecycle invariant in one action.

This mirrors `/elaborate`, which owns the `done → in-progress` back-edge. The pattern is consistent: the command that introduces work or uncertainty incompatible with the current status is also the command that updates the status to match. The user's explicit invocation of `/ask` (and acceptance of the refined question) is the consent for the mutation; no separate confirmation prompt is required.

`/clarify` becomes the resolver, not the back-edge entry point. Its hot path is unchanged: walk open questions on a `draft` spec, advance to `clarified`. A recovery branch handles hand-edited specs that arrive at `/clarify` with a non-`draft` status and unresolved questions in the body — a state that should not occur via normal usage but might from manual frontmatter edits.

### `/ask` status mutation

`/ask`'s behavior depends on the targeted spec's current status:

| Status | Behavior |
| --- | --- |
| `draft` | Refine question; append to `## Open Questions`. No status change. (Existing behavior.) |
| `clarified` / `planned` / `in-progress` | Refine question; append to `## Open Questions`; revert status to `draft`. Display impact: prior status, plan artifacts that exist (with timestamps), scenario files. |
| `done` | Refuse. Report: "Spec is `done`. Run `/{project}:elaborate` to capture this as a scenario instead." A question on a `done` spec means either the behavior needs lower-level elaboration (a scenario) or the spec is wrong (manual revision); `/ask`'s back-edge does not cover either. |

When `/ask` mutates status, it does so after the user accepts the refined question — that acceptance is the explicit consent for the mutation. A separate yes/no prompt at status-change time would be redundant friction.

The post-question hint always points at `/{project}:clarify`. The spec is now at `draft` (regardless of where it started), so the next step is the same in every non-`done` case: "Question recorded. Run `/{project}:clarify` to resolve it." When `/ask` redirects to `/elaborate` on a `done` spec, the hint becomes that redirect message instead.

When `/ask` targets a scenario (per spec 009), it appends to the scenario's `## Open Questions`. Scenarios have no status field — there is nothing to mutate. The scenario back-edge mechanism is unaffected by this spec.

### `/clarify` gate behavior

With `/ask` owning the back-edge, `/clarify`'s gate becomes simple:

| Status | Has open questions? | Behavior |
| --- | --- | --- |
| `draft` | yes | Walk questions; advance to `clarified` (existing) |
| `draft` | no | Verify acceptance criteria; advance to `clarified` (existing) |
| `clarified` / `planned` / `in-progress` | yes | **Recovery path** — see below |
| `clarified` / `planned` / `in-progress` | no | Stop with "Spec is already `{status}`. Run `/{project}:plan`/`/{project}:implement` to advance." (existing message, lightly tightened) |
| `done` | (any) | Stop with "Spec is `done`. Run `/{project}:elaborate` to capture this as a scenario instead." |

### `/clarify` recovery path

A `clarified`/`planned`/`in-progress` spec with open questions should not exist via normal usage — `/ask` reverts to `draft` whenever it records on such a spec. The state can still arise from a manual frontmatter edit or from a spec migrated from another tool.

When `/clarify` encounters this state, it offers a recovery prompt before mutating:

- Display the current status, count and titles of open questions, plan-artifact list with timestamps, and scenario-file list
- Prompt: "Spec is `{status}` but has {N} unresolved open questions in the body — this state usually arises from a manual frontmatter edit. Revert status to `draft` and walk the questions?"
- **Confirm:** revert to `draft`, run the standard clarify walk, advance to `clarified` after resolution.
- **Decline:** stop with no changes. The spec retains its inconsistent state; the user can re-run `/clarify` later or fix manually.

Previously-resolved questions in `## Resolved Questions` are never re-walked — `/clarify` only processes items in `## Open Questions`.

### Validation impact

When `/ask` reverts status to `draft`, it does not delete or rewrite downstream artifacts (`plan.md`, `tasks.md`, `data-model.md`, scenario files). Existing checkboxes and content are preserved. The impact display surfaces these artifacts so the user knows what may need re-review after the question is resolved.

After `/clarify` resolves the question and advances back to `clarified`, `/plan` is the natural next step. Per **Plan re-run safety** below, `/plan` detects existing artifacts and prompts the user to keep or replace them — it does not silently overwrite work the back-edge cycle is trying to preserve.

### Plan re-run safety

Today, `/plan` unconditionally copies `framework/templates/spec/plan.md` over any existing `plan.md` (and the same for `tasks.md` and `data-model.md`). On a fresh `clarified` spec this is correct; on a re-clarified spec — or any spec where the user re-runs `/plan` — it destroys committed work.

`/plan` is updated to detect existing plan artifacts and prompt before overwriting:

- If `plan.md`, `tasks.md`, or `data-model.md` already exist when `/plan` runs:
  - List the existing artifacts with their last-modified timestamps.
  - Prompt: "Plan artifacts exist from a prior `/plan` run. Keep them and run the readiness check, or replace with fresh templates?"
  - **Keep** (default): skip template copy. Run the existing readiness check on the kept artifacts. Advance status to `planned` only if all checks pass; report failures otherwise so the user can edit and retry.
  - **Replace:** copy fresh templates over the existing files. The user is responsible for re-applying any kept content. Then proceed with the standard plan flow.
- If no plan artifacts exist, the existing behavior is unchanged: copy templates and proceed.

This protection applies to every `/plan` run, not only those triggered after a back-edge cycle. It also covers users who manually edit frontmatter back to `clarified`, who rerun `/plan` for any reason, or who lose confidence in the existing plan.

## Constitution Updates

`framework/constitution.md` §spec-lifecycle (lines 96–99) needs revision so it matches what's wired up. The original wording is close to correct; only the destination state and the mechanism need clarification:

- Back-edge 1 stays as written (delivered by `/elaborate`).
- Back-edge 2 changes from "`planned` or `in-progress` → `clarified` when `/ask` records a new open question" to "`clarified` / `planned` / `in-progress` → `draft` when `/ask` records a new open question; the next `/clarify` resolves the question and the spec advances forward again." The destination is `draft` (the only state that tolerates open questions), not `clarified`.

Both back-edges then read as command-owned, status-mutating actions triggered by the introduction of new work or uncertainty — consistent.

## Acceptance Criteria

### `/ask` back-edge

- [x] `framework/commands/ask.md` reverts spec status to `draft` after appending an open question to a spec at `clarified`, `planned`, or `in-progress`
- [x] On a `draft` spec, `/ask` records the question without status mutation (existing behavior preserved)
- [x] On a `done` spec, `/ask` refuses and reports: "Spec is `done`. Run `/{project}:elaborate` to capture this as a scenario instead." No question is recorded; no status mutation occurs
- [x] When `/ask` mutates status, it displays the prior status, plan artifacts that exist (with last-modified timestamps), and scenario files — so the user can see what may need re-review
- [x] `/ask` does not prompt for separate yes/no confirmation before mutating status — the user's acceptance of the refined question is the consent
- [x] When `/ask` targets a scenario (per spec 009), it appends to the scenario's `## Open Questions` and does not mutate any spec or scenario status (scenarios have no status field)
- [x] `/ask`'s post-question hint is "Question recorded. Run `/{project}:clarify` to resolve it." in every case where a question is recorded

### `/clarify` gate

- [x] `framework/commands/clarify.md` Gate branches on the spec's open-question count, not on a flag
- [x] On a `draft` spec (with or without open questions), the existing behavior is preserved — walk questions if present, verify ACs, advance to `clarified`
- [x] On a `clarified` / `planned` / `in-progress` spec with no open questions, the command stops with the existing "Spec is already `{status}`" message (lightly tightened to mention the next pipeline command)
- [x] On a `done` spec (any open-question count), the command stops with "Spec is `done`. Run `/{project}:elaborate` to capture this as a scenario instead." and exits without mutation
- [x] No downstream artifacts (`plan.md`, `tasks.md`, `data-model.md`, scenario files) are deleted or rewritten by `/clarify`

### `/clarify` recovery path

- [x] On a `clarified` / `planned` / `in-progress` spec with one or more open questions (an inconsistent state usually arising from manual frontmatter edit), the command displays the current status, open-question titles, plan-artifact list with timestamps, and scenario-file list, then prompts: "Spec is `{status}` but has {N} unresolved open questions — revert status to `draft` and walk the questions?"
- [x] On confirm, the spec's frontmatter `status` field is updated to `draft` before the standard clarify walk proceeds
- [x] On decline, no files are modified — the spec retains its inconsistent state and open questions remain in `## Open Questions`
- [x] Previously-resolved questions in `## Resolved Questions` are never re-walked — `/clarify` only processes items in `## Open Questions`

### `/plan` re-run safety

- [x] `framework/commands/plan.md` detects whether `plan.md`, `tasks.md`, or `data-model.md` already exist in the feature directory before generating
- [x] If any plan artifact exists, `/plan` lists the existing files with their last-modified timestamps and prompts the user to keep or replace
- [x] On "keep" (default), `/plan` skips template copy, runs the existing readiness check on the kept artifacts, and advances status to `planned` only if all checks pass
- [x] On "replace", `/plan` copies fresh templates over the existing files, then proceeds with the standard plan flow
- [x] If no plan artifacts exist, `/plan` behavior is unchanged
- [x] The protection applies to every `/plan` run, not only those triggered after a back-edge cycle

### Cross-spec deliverables

- [x] `framework/constitution.md` §spec-lifecycle back-edge bullet for `/ask` is rewritten per the **Constitution Updates** section above (named `/ask` as the entry point, `draft` as the destination)
- [x] `specs/000-slash-commands/spec.md` gains a signpost noting that `/ask` becomes the back-edge owner in 014 (mutating status to `draft` on non-`draft` specs), `/clarify` gains the open-questions-on-non-`draft`-spec recovery path in 014, and `/plan` gains overwrite-protection on existing artifacts in 014
- [x] `.claude/commands/gov/ask.md`, `.claude/commands/gov/clarify.md`, and `.claude/commands/gov/plan.md` are regenerated via `scripts/gen-claude-commands.sh`
- [x] All modified `.md` files pass `npx markdownlint-cli2`

## Edge Cases

### `/ask`

- **`/ask` on a `clarified` spec where the question is identical to one already in `## Open Questions`** — the refinement loop is the natural place to detect duplicates. If the refined question matches an existing one, `/ask` reports "An equivalent question is already recorded: '{existing}'. Skip or refine further?" before any mutation. On skip, no question is added and no status mutation occurs.
- **`/ask` on a feature whose `spec.md` does not exist** — same gate message the existing command emits ("Spec does not exist. Run `/{project}:specify` first."). The new back-edge logic does not bypass the spec-existence check.
- **`/ask` on a `clarified` spec that is a dependency of an `in-progress` spec** — reverting the dependency to `draft` may temporarily block the dependent spec's pipeline gates (since `/clarify` and `/plan` both check that dependencies are at `clarified` or later). This is the correct behavior — a question on a dependency is information the dependent spec's author should know about. `/ask`'s impact display includes a one-line note: "Note: this spec is a dependency of {dependent specs}; their pipeline checks will block until this spec returns to `clarified`."
- **User aborts the refinement loop** — `/ask` exits without recording a question and without mutating status, regardless of the spec's starting status. The status mutation only fires when a question is actually recorded.

### `/clarify` recovery path

- **Recovery prompt declined** — spec retains its inconsistent state. The next `/clarify` invocation will offer the same prompt. This is intentional — the system surfaces the inconsistency on every clarify attempt rather than silently advancing.
- **Recovery on a spec whose dependencies are not at `clarified` or later** — the dependency check fires after the status revert, same as a normal clarify run on a `draft` spec. Reverting first matches the principle that the user's explicit confirmation takes precedence over read-only checks.
- **Scenario-targeted `/clarify` on a parent spec in the inconsistent state** — scenario clarify is independent of spec clarify (per spec 009). The scenario walk handles its own open questions; it does not check or mutate the parent spec's status. If the user wants to walk parent-spec questions, they re-target the parent.

### `/plan` re-run safety

- **`/plan` re-run on a spec where the user genuinely wants a clean slate** — the user picks "replace" at the prompt. Existing artifacts are overwritten with templates; the user re-applies any content they want to keep. This is the same behavior as the original `/plan` command, just gated behind explicit consent.
- **`/plan` re-run after a back-edge cycle where the resolved question invalidates parts of the kept plan** — picking "keep" surfaces structural conflicts via the existing readiness check (plan contradicts spec, missing data-model, etc.), but cannot detect every semantic invalidation. The user must read the kept plan against the new resolution and pick "replace" if a clean rewrite is needed. The prompt's explicit framing ("Plan artifacts exist from a prior `/plan` run") signals this is a decision worth making, not a default to skip past.
- **`/plan` "keep" run where the readiness check fails** — `/plan` reports the failures and does not advance status. The kept artifacts remain on disk; the user edits them (or re-runs with "replace") and tries again. This matches the existing failure path for `/plan`.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Symmetric vs asymmetric back-edge.** Symmetric: every status mutation is owned by the command whose action makes it necessary. `/elaborate` owns `done → in-progress` because adding a scenario creates incomplete work. `/ask` owns `clarified+ → draft` because adding an open question creates unresolved uncertainty (and the constitution defines `clarified` as "open questions resolved" — the new question violates that invariant). The asymmetric alternative (trim the constitution, leave `/ask` as a pure flag) was considered and rejected — symmetric matches the constitution's intent and prevents the spec from sitting in an inconsistent state.
- **Naming: `--reopen` vs `--reclarify` vs `--revert`.** Moot — no flag exists in the final design. The trigger is the data, not a user-supplied flag.
- **Clear `plan.md`/`tasks.md` checkboxes on back-edge?** No — leave checkboxes as-is and surface stale artifacts in the impact display when `/ask` mutates status. Most back-edges are narrow (one new question); clearing destroys signal about what was actually completed pre-revert. Matches `/elaborate`'s precedent (adding a scenario to a `done` spec does not clear acceptance-criteria checkboxes; it adds a new task). Stale-checkbox risk is mitigated by the impact display plus the `/plan` re-run safety added by this spec.
- **Revert to `draft` or to `clarified`?** Revert to `draft`. The constitution defines `clarified` as "open questions resolved" — once a new open question exists, the spec no longer satisfies that definition, so `clarified` is internally inconsistent as a destination. `draft` is the only status that tolerates open questions. The "wholesale rethink" concern (re-walking previously resolved questions) does not apply: `/clarify` only walks items in `## Open Questions`, leaving `## Resolved Questions` untouched. After the new question is resolved, the spec advances back through `clarified` → `planned` (now safe per the `/plan` re-run protection added by this spec) → `in-progress` → `done`.
- **Flag-driven (`--reopen`) or implicit (branch on data)?** Implicit. The trigger isn't user intent expressed via flag — it's the data: does the spec have open questions? Adding a flag would require the user to express the same intent twice (running the command AND passing `--reopen`). Mirrors `/elaborate` — that command doesn't have a flag for "reopen done spec"; it just does the right thing.
- **Should `/ask` mutate status, or should `/clarify` own the mutation?** `/ask` mutates. Same logic: a `clarified+` spec with an open question is internally inconsistent. The command that *creates* the inconsistency is the natural place to fix it — same way `/elaborate` reverts `done → in-progress` immediately on adding a scenario. Putting the mutation in `/clarify` instead would let the inconsistent state persist between the two commands and would require either a flag or pre-revert prompt, both of which the user critique correctly identified as redundant. `/clarify`'s gate keeps a recovery branch for the rare hand-edit case but not for the normal flow.
- **`/ask` post-question hint posture.** Active suggestion in soft language. `/ask` is invoked because the user already cares about the question; naming the next obvious step (`/clarify`) is helpful, not pushy. Soft framing — "Question recorded. Run `/{project}:clarify` to resolve it." — is a recommendation, not a directive.
- **Should `/ask` confirm before mutating status?** No. The user's acceptance of the refined question (the existing refinement loop) is the consent. `/elaborate` doesn't confirm before reverting `done → in-progress`; `/ask` shouldn't either. The impact display surfaces what's about to happen; the actual mutation follows the user's commitment to the refined question.
