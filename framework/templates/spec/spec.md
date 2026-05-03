---
status: draft
dependencies: []
tags: []
---

# {NNN} — {Feature Name}

{Brief description of what this feature does and why it exists.}

## {Section}

<!-- Organize the spec into sections that describe behavior, contracts, and constraints.
     Use headings that make sense for this feature — there is no fixed set of required sections
     beyond Acceptance Criteria and Open Questions.

     Metadata (status, dependencies, tags) lives in the YAML frontmatter block above —
     not in the body. Run /{project}:clarify, /{project}:plan, and /{project}:implement
     to advance status; the commands update the frontmatter for you.

     Scenarios: when a spec section needs lower-level elaboration (edge cases, bug fixes,
     detailed behavior), run /{project}:elaborate to create a scenario file under
     specs/{NNN-feature-name}/scenarios/.
-->

## Acceptance Criteria

At least one concrete, testable criterion is required before `/{project}:clarify` will advance the spec.

<!-- Concrete, testable conditions that define "done". Each criterion should be verifiable
     through a test or observable behavior. Replace this comment block with real `- [ ]`
     checkbox items. Example:

- [ ] Health endpoint returns 200 when all dependencies are reachable
- [ ] Health endpoint returns 503 with a JSON body listing failures when any check fails
- [ ] Auth middleware rejects requests without a valid session or token with 401

-->

## Applicable Rules

<!-- Optional. Cite rule IDs (from rule files like specs/security-backend.md) that
     constrain the surface this spec touches. Citing rules here makes the cross-
     cutting requirements this spec depends on visible to reviewers and to
     /{project}:validate, which checks every cited ID against the loaded rule
     files. See §rules in the constitution for when a concern belongs in a rule
     vs an acceptance criterion vs a scenario.

     Replace this comment block with a list of rule references when applicable:

- `BE-AUTHN-001` — memory-hard password hashing
- `FE-XSS-002` — output encoding strategy
- `BE-INPUT-001` — server-side input validation

     Delete this section entirely if no rules apply to the area this spec covers.
-->

## Open Questions

<!-- Uncertainties, unresolved decisions, and areas needing investigation.
     All open questions must be resolved before moving to the plan phase.

     To surface questions: assume this feature shipped and failed — what went wrong? Example:

- Should rate limits be configurable per tenant or fixed globally?
- What is the retention policy for audit log entries?
- What happens when the sessions table grows unbounded?

     When a question is resolved, move it to a "Resolved Questions" section with its answer.
-->
