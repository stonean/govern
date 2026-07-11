---
section: "Follow-on scenarios"
---

# Primitive-robustness-hardening

## Context

The 2026-07-11 runtime review surfaced eight SHOULD-tier robustness gaps across the primitive library, logged to the inbox and groomed here as one hardening pass:

- Path validation is inconsistent: `validate_no_traversal` guards some path-taking primitives but not the destructive one — `enforce-manifest` accepts an arbitrary absolute `directory` plus `glob`/`recursive` with an empty `expected` list (a filesystem-wide delete loop) — nor `apply-manifest`'s manifest-entry `source`/`dest` (fetched-artifact class; a `..` entry writes outside the target root), nor `merge-managed-block`/`merge-permissions` path args.
- `check-stuck` assumes linear first-parent history: a transition commit on a merged side branch is never reached (inflated counts, false `stuck`), `find_in_progress_commit` tracks `previous_status` across a topological walk (phantom transitions), and its hand-rolled `extract_status` misses CRLF close fences.
- `write-review` writes `review.md` before parsing spec frontmatter, so a malformed spec halts between the two writes.
- `substitute-templates` lands outputs at the 0600 tempfile mode, discarding source modes.
- `create-scenario` does not YAML-escape `"`/`\` in the `section` argument; `append-task` interpolates `title`/`done-when`/`body` without newline sanitization (structure injection).
- `dashboard` hard-fails the whole render when the targeted scenario file has missing/malformed frontmatter (one bad scenario bricks `/gov:status`), and its open-question count doc contradicts behavior.
- `check-rule-ids` marks an ID deprecated when `**DEPRECATED` appears within 256 bytes after any occurrence anywhere in the file — a live rule near a deprecated neighbor false-flags.
- `set-status` writes any string, including values outside the constitution's lifecycle set.

## Behavior

- Every primitive accepting a repo path from the host or LLM applies `validate_no_traversal` (or an equivalent containment check) before filesystem operations: `enforce-manifest` rejects directories outside the repo root, `apply-manifest` validates each manifest entry's `source` and `dest`, `merge-managed-block` and `merge-permissions` validate their `path`.
- `check-stuck` reads history in a branch-shape-tolerant way: the transition commit is found even when it lands via a merge, phantom transitions are not reported across topologically adjacent commits from different branches, and frontmatter splitting reuses the shared CRLF-aware helper.
- `write-review` reads and validates all inputs (including spec frontmatter) before its first write, so a halt never leaves `review.md` and the spec `review:` block inconsistent.
- `substitute-templates` mirrors each source file's permissions onto the written output, matching `apply-manifest`/`extract-archive`.
- `create-scenario` emits valid YAML for any `section` string; `append-task` rejects or flattens embedded newlines in `title`, `done-when`, and `body` items.
- `dashboard` degrades to a detail-less target on a malformed targeted scenario (parse failure behaves like the missing-file case) and its doc matches the counting behavior.
- `check-rule-ids` scopes the deprecation scan to the rule's own section (heading to next heading).
- `set-status` rejects `to`/`from` values outside `draft|clarified|planned|in-progress|done`; transition-edge legality stays with procedures.

## Edge Cases

- `enforce-manifest` continues to work for legitimate absolute directories inside the repo (the resolved path must sit under the repo root).
- A waiver of the traversal rule is not provided: primitives that genuinely need absolute out-of-repo paths do not exist today; if one appears it gets its own scenario.
- `check-stuck` on a repo with no merge commits behaves exactly as before.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
