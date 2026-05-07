---
status: draft
dependencies: []
---

# {NNN} — {Feature Name}

{Brief description of what this feature does and why it exists.}

## {Section}

<!-- Organize the spec into sections that describe behavior, contracts, and constraints.
     Use headings that make sense for this feature — there is no fixed set of required sections
     beyond Acceptance Criteria and Open Questions.

     Lightweight track: this document combines spec and plan. Use when ALL of these are true:
     - The feature touches a single module or package
     - There are no open questions — the approach is obvious
     - The change is small (roughly <50 lines of production code)

     If a question surfaces later, capture it in Open Questions below and run
     /{project}:clarify before continuing.
-->

## Technical Decisions

<!-- Brief notes on the implementation approach. Example:

### Storage

Using the existing sessions table with an added `last_active` column.
Alternative considered: separate table — rejected because session data is already co-located.

-->

## Affected Files

<!-- List files that will be created or modified. Example:

| File | Action | Purpose |
| --- | --- | --- |
| `src/auth/handlers` | Modify | Add session refresh endpoint |
| `migrations/20250301_add_last_active` | Create | Add column to sessions table |

-->

## Acceptance Criteria

At least one concrete, testable criterion is required before `/{project}:clarify` will advance the spec.

<!-- Concrete, testable conditions that define "done". Each criterion should be verifiable
     through a test or observable behavior. Replace this comment block with real `- [ ]`
     checkbox items. Example:

- [ ] Session refresh extends expiry by the configured duration
- [ ] Expired sessions return 401

-->

## Open Questions

<!-- Lightweight specs are expected to have none at creation time — that is the qualifying
     condition for this track. If one surfaces later (during plan or implement), capture it
     here and run /{project}:clarify to resolve before continuing.
-->
