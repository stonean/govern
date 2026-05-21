# 005 — Plan Fixture

A partially-filled plan exercising `writeSpecBody`'s re-run state. The
`Technical Decisions` section already has content on disk; the runtime
inlines it into the request as `existing-content` so the LLM can edit
rather than overwrite. The other two sections are empty headings; their
requests carry no `existing-content` (re-run state absent).

## Technical Decisions

Prior decision recorded on the previous `/gov:plan` run: use the
standard library and avoid third-party dependencies for this fixture.

## Affected Files

## Trade-offs
