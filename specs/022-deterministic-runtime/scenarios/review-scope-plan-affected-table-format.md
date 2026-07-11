---
section: "Follow-on scenarios"
---

# Review-scope plan-affected — parse the canonical `## Affected Files` table

## Context

The [review-runtime-acceleration](review-runtime-acceleration.md) scenario's
`compute-review-scope` resolves the review file scope as "the plan's Affected
Files unioned with the files modified since `diff-base`, larger set wins." Its
`read_plan_affected` helper (`runtime/src/primitives/compute_review_scope.rs`)
parses the `## Affected Files` section as a **bullet list** (dash + backticked
path). But the canonical plan format — the `/gov:plan` writeSpecBody template
and every real `specs/*/plan.md` — is a **Markdown table**
(`| File | Action | Purpose |`), which the pre-existing
`payload.rs::parse_affected_files` (the writeCode plan reader) already parses
correctly.

The two Affected-Files parsers disagree on format, so for real (table-format)
plans `read_plan_affected` returns empty: the `plan_affected` half of the review
scope is silently dropped and scope collapses to `modified-since` alone,
defeating the "unioned, larger wins" contract. Surfaced 2026-07-06 during task
46b (the `review-basic` exec fixture's table-format plan yielded an empty scope)
and recorded as a low-confidence `QUAL-STUB-001` finding in the 022 review.

## Behavior

- **Unify on the table parser.** Promote `payload::parse_affected_files` to a
  shared `pub(crate)` helper and call it from `read_plan_affected` in place of
  the bullet parser, so `compute-review-scope` and the writeCode plan reader
  agree on one canonical format. For a real table-format plan, `plan_affected`
  is non-empty and the "larger set wins" branch is reachable in production
  again.
- **Fixture realignment.** The `review-basic` exec fixture's `plan.md` switches
  from the bullet list back to the canonical table (dropping the interim
  bullet-form workaround note added in 46b). The fixture's parity golden is
  unchanged — the resolved scope is still `src/reviewed.rs` — so no re-bless is
  needed.

## Edge Cases

- **Bullet-form back-compat.** No real plan uses the bullet form, so the
  bullet parser is dropped, not kept alongside the table parser — a single
  canonical format avoids a second divergence. (If a caller ever needs the old
  form, that is a separate, explicitly-scoped change.)
- **Absent / empty section.** A plan with no `## Affected Files` section, or an
  empty one, still yields an empty `plan_affected` (unchanged) and scope falls
  back to `modified-since`.
- **Tie-break unchanged.** On equal set sizes, `modified-since` still wins
  (git-authoritative for what the work actually touched).

## Tests

- Add a table-format case to `compute_review_scope::tests` asserting
  `plan_affected` parses a `| File | Action | Purpose |` table (mirroring
  `plan_affected_wins_when_it_is_the_larger_set` in table form).
- Switch the `plan_affected_wins_when_it_is_the_larger_set` fixture and the
  `review-basic` fixture `plan.md` to the table form; confirm the `review-basic`
  parity golden needs no re-bless.

## Open Questions

*None — captured during grooming.*

## Resolved Questions

*None yet.*
