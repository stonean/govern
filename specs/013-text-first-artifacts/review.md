---
spec: 013-text-first-artifacts
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 013-text-first-artifacts

## Summary

Migration from bold-prefix metadata to YAML frontmatter across every spec template, command source, and `.claude/commands/gov/` mirror — plus the self-migration of specs 000–013. `§text-first-artifacts` constitutional principle. Pure markdown — security rules do not apply. All five passes ran; no findings. `blocking: no`.

Note: 013 introduced `tags: []` in frontmatter; spec 017 (derive-don't-ask) later removed `tags` and `title` from active templates. The `tags` field persists in the older specs reviewed here (`frozen archaeology` per 017's plan).

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._

## Pass notes

### Security

No security surface.

### Reuse

The frontmatter schema codified here is the contract that every later spec (014–021) reads and writes. The migration loop is the canonical "walk every spec, transform in place" pattern reused at scale by 017's migration tasks.

### Quality

Self-migration of 013 last (after 000–012) was a deliberate exercise — running the migration against a known-good source surfaces edge cases before they reach the operator. The audit/modify rows for `groom.md`, `elaborate.md`, `amend.md`, `log.md`, `help.md` correctly call out which need touch and which don't.

### Efficiency

N/A.

### Simplicity

One canonical metadata location (frontmatter), not two (frontmatter + body bold-prefix). Reduces ambiguity for both human and AI readers.
