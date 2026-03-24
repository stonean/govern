# {NNN} — {Feature Name}

**Status:** draft
**Dependencies:** none
**Track:** lightweight

{Brief description of what this feature does and why it exists.}

## {Section}

<!-- Organize the spec into sections that describe behavior, contracts, and constraints.
     Use headings that make sense for this feature — there is no fixed set of required sections
     beyond Acceptance Criteria.

     Lightweight track: this document combines spec and plan. Use when ALL of these are true:
     - The feature touches a single module or package
     - There are no open questions — the approach is obvious
     - The change is small (roughly <50 lines of production code)
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

<!-- Concrete, testable conditions that define "done". Each criterion should be verifiable
     through a test or observable behavior. Example:

- [ ] Session refresh extends expiry by the configured duration
- [ ] Expired sessions return 401

-->
