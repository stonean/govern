---
section: "Follow-on scenarios"
---

# Parser-nested-list-continuation

## Context

The 2026-07-11 review found a parser silent-failure class not addressed in the 0.19.0 hardening (deferred as too entangled with list-item finalization to fix without destabilizing the parse goldens): a backtick-quoted primitive named in a step's continuation text *after* a nested ordered list closes is dropped.

When a step opens a nested ordered sub-list (`1.` → `1.1`, `1.2`), each sub-item finalizes the current step builder, so by the time the nested list ends the walker's `current_step` is `None`. Any `Event::Code` (a primitive name) or trailing prose after the nested list is then ignored — it is not attributed to the parent step, not tracked as a suspicious span, and does not raise a conflict. On the exec path the dispatch is silently skipped; the walk still reports a clean `complete`. No shipped `framework/commands/*.md` file triggers it, but adopter-authored command files are not constrained, and a typo'd primitive name is already a hard error — this same-severity authoring mistake should not pass silently.

## Behavior

A primitive named in a step's continuation content following a nested ordered list is not silently dropped. Either it is attributed to the enclosing step (so the dispatch fires where the author intended), or — when attribution is genuinely ambiguous — the parse fails with a diagnostic naming the step and the orphaned primitive, consistent with the two-primitive hard-error already in place.

## Edge Cases

- A nested **unordered** list (bullets) already keeps the parent step open; only the ordered-nested case is affected — the fix must not change the unordered behavior.
- Content after a nested list that names no primitive (plain prose) stays part of the step's prose as today.
- Deeply nested ordered lists (three or more levels) resolve to the nearest enclosing step.
- The parser's numbered-step self-check (parsed numbering equals literal numbering) still holds after the fix.

## Open Questions

- Attribute-to-parent versus hard-error: which is the less surprising default for an adopter, given the existing two-primitive hard-error precedent?

## Resolved Questions

*None yet.*
