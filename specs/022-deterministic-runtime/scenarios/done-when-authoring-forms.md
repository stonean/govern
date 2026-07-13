---
section: "Follow-on scenarios"
---

# Done-when-authoring-forms

## Context

Spec 022's `read-tasks` records each task's completion condition as `done_when`, and `check-artifacts`' task-consistency family flags any task whose `done_when` is absent (reference `analyze.md` §"Task consistency", whose sole requirement is *"Each task has a 'done when' condition"* — a semantic condition, no marker syntax). The `read-tasks` parser recognized only the canonical bold form `- **Done when**: …` that the `append-task` primitive emits.

But `append-task` is not the primary writer of a task breakdown. `/gov:plan` fills the `tasks.md` template directly (its step reads "the runtime provides no primitive for the breakdown itself"), and the template documented no done-when line for the LLM to copy — so the plan step's LLM invents the syntax. In practice it authors a checkbox-nested clause (`- [x] Done when: …`) or, historically, a bulletless `Done when: …`. Neither was recognized: the checkbox form was swallowed as the task's last **subtask** (`done_when` stayed unset), and the bulletless form matched nothing. The result: `check-artifacts` reported *every* task in such a spec as "has no Done when clause," and `/gov:review`'s done-gate would too — a 100%-false-positive block.

Origin: observed 2026-07-12 during `/gov:analyze` on an adopter project (`nookwit/magpie`) whose specs were authored end-to-end by `/gov:plan` in the checkbox form; all 24 tasks across two specs flagged. Govern's own corpus carried the same latent split — 001/002/003/004/013/014/… in the bulletless form, 015/024/025 in the checkbox form (024 colon-less). The writer and the reader inside the framework disagreed on format. This belongs in the `runtime-primitive-structural-bugs` family on spec 022.

## Behavior

- `read-tasks` MUST recognize a task's "Done when" clause in every form the writers and the corpus produce, and record its body as `done_when`:
  - `- **Done when**: <body>` — the canonical form `append-task` emits and the `tasks.md` template documents.
  - `- [x] Done when: <body>` / `- [ ] Done when: <body>` — the checkbox-nested form `/gov:plan`'s task breakdown tends to author.
  - `Done when: <body>` — the bulletless form.
- Recognition routes through a single shared helper (`primitives::mod::parse_done_when`) that `read-tasks` and `mark-task` both consume, so a checkbox-form clause is treated as a clause by **both** sides — never as an addressable subtask. This preserves the read/mark index contract: `read-tasks` excluding the clause from its subtask list and `mark-task` excluding it from the subtask index space are the same decision made once.
- The leading list bullet, an optional task-list checkbox, the `**` emphasis around the label, and the `:` separator are all optional; the `Done when` label is matched case-insensitively.
- To avoid reading an ordinary subtask that merely opens with a longer word (`Done whenever …`) as a clause, the label MUST land on a word boundary — the character after it is the `:` separator, a closing `**`, whitespace, or end of line.
- The fix does not weaken the check: a task carrying no "Done when" line in any form is still flagged. `check-artifacts` observable behavior changes only in that the previously-unrecognized forms now satisfy the requirement they always semantically met.
- Prevention (framework side, outside the runtime): the `tasks.md` template carries a `- **Done when**: …` example line and `/gov:plan`'s task-breakdown reference names the exact marker, so the primary writer converges on the canonical form the reader prefers.

## Edge Cases

- **Colon-less checkbox form** (`- [x] Done when \`framework/rules/\` lists the file`, spec 024's shape): recognized; the body is everything after the label, whitespace-trimmed.
- **Emphasis wrapping the colon** (`- **Done when:** <body>` as well as `- **Done when**: <body>`): both reduce to the same body; the separator strip tolerates the `**` on either side of the `:`.
- **Body that itself opens with emphasis** (`- **Done when**: **all** tests pass`): the separator strip removes only the single leading `**`/`:` decoration, never the body's own `**all**`.
- **Checkbox-form clause is not an addressable subtask**: for a task whose only checkboxes are two real subtasks followed by `- [x] Done when: …`, `read-tasks` reports two subtasks and `mark-task --subtask-index 2` is out of range (total 2), not a flip of the done-when line.
- **Word-boundary guard**: `- [ ] Done whenever the cache warms` is an ordinary subtask, not a `Done when` clause; `done_when` stays unset.
- **Genuine omission is still flagged**: a task with acceptance sub-bullets but no "Done when" line in any form (spec 027's shape) continues to produce a `task-consistency` finding — the widening tolerates real clauses, it does not invent them.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
