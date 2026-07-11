---
section: "Follow-on scenarios"
---

# Write-boundary-path-normalization

## Context

`validate_write_code_boundary` matches each `writeCode` edit path against the boundary patterns with raw string prefix comparison and no path normalization. An edit path containing `..` segments — e.g. `runtime/../framework/constitution.md` — satisfies the pattern `runtime/**` because the string starts with `runtime/`, so a hallucinated or hostile LLM response can write outside the boundary the runtime exists to enforce. The data model states the runtime rejects out-of-boundary edits; the canonicalized-containment discipline already exists in `payload.rs::classify_contained` but is not applied here. Surfaced in the 2026-07-11 runtime review.

## Behavior

Edit paths in a `writeCode` response are validated before boundary matching: absolute paths and paths containing `.` or `..` segments are rejected outright (or resolved and re-checked for containment), so no edit path can escape the declared write boundary via traversal segments. The rejection is a domain outcome reported to the host, consistent with the existing out-of-boundary rejection semantics.

## Edge Cases

- A path that is lexically inside the boundary but contains a redundant `./` segment is rejected rather than normalized silently — the LLM is asked for clean repo-relative paths.
- Boundary patterns themselves are trusted (they come from `derive-boundary`), only response paths are suspect.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
