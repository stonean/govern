---
section: "Gate Enforcement"
---

# Implement-skips-planned-prompt

## Context

The Gate Enforcement section defines the implement gate (`planned` or
`in-progress`). The current command source `framework/commands/implement.md`
(step 4) asks the user to approve the `planned → in-progress` transition before
any code changes, then flips status (step 5). The `--auto` carve-out (Flags
section) also lists `planned → in-progress` among the pipeline gates that "still
fire and pause" even under `--auto`, citing §pipeline-boundaries.

That prompt is redundant. Invoking `/gov:implement` is itself the user's
deliberate decision to start work and enter `in-progress`; a yes/no prompt fired
immediately after invocation — before any work — asks the user to re-confirm what
they just requested. §pipeline-boundaries' "present the work done and wait for
the user to confirm" clause targets advancement *after* work exists (the
`in-progress → done` edge); at `planned → in-progress` there is nothing to
present, so the command invocation is the explicit approval the rule calls for.

## Behavior

- `/gov:implement` no longer prompts to confirm `planned → in-progress`.
  Invoking the command against a `planned` spec flips status to `in-progress`
  (after the read-tasks / derive-boundary / check-stuck setup) and proceeds
  directly to the first task. No yes/no gate, with or without `--auto`.
- The `in-progress → done` transition is unchanged: it still requires explicit
  confirmation (work has been done and is presented for review per
  §pipeline-boundaries; the pre-done review gate also still applies).
- Cross-spec: the `--auto` carve-out in `implement.md` (spec 010-agent-autonomy)
  drops `planned → in-progress` from its "still fires and pauses" list;
  `in-progress → done` remains. Recorded on 010 per §cross-spec-impact.

## Edge Cases

- **Spec already `in-progress` (resuming).** No transition prompt either way
  today; unchanged — the command continues with the next task.
- **Spec at `clarified` or earlier.** The gate still rejects and directs the user
  to `/gov:plan` first. Only the *confirmation* is removed, not the status check.
- **Denial escape hatch removed.** Step 4 previously let the user deny and exit
  without mutating the spec. With no prompt there is no denial branch — a user
  who doesn't want to start simply doesn't invoke the command.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
