---
section: "Follow-on scenarios"
---

# Merge-managed-block-trailing-append

## Context

`merge-managed-block`'s group-alignment walk (`walk_body_extent`) mis-attributes adopter content when the new canonical block appends subsection(s) at the end. With an on-disk managed block containing groups `{Env, IDE}` followed by an adopter-authored tail section (e.g. `# Rust` / `/target`), and a new canonical block `{Env, IDE, OS}`, the walk consumes the adopter's `# Rust` section as a "full rewrite" of the unmatched trailing canonical group — the adopter's content is deleted from the merged result. Probe-verified in the 2026-07-11 runtime review. A framework release that adds a new subsection at the bottom of the shipped `.gitignore` template would destroy the first adopter-authored section following the managed block on every adopter's next `/govern` run. Existing tests cover mid-block insertion but not trailing append.

## Behavior

When the canonical block gains subsections beyond those present on disk, the alignment walk treats every unmatched trailing canonical group as a pure insertion at the end of the managed block. Adopter content following the managed block is never consumed by group alignment — it is preserved verbatim after the merged block, regardless of how many canonical groups were appended.

## Edge Cases

- Multiple canonical groups appended at once are all inserted; the adopter tail still survives.
- An adopter tail section whose heading happens to equal a newly appended canonical heading is still adopter content — the canonical group is inserted inside the managed block markers and the adopter's copy outside the markers is left untouched (dedup of the resulting duplication is the adopter's judgment, not the merge's).

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
