---
section: "Follow-on scenarios"
---

# Check-stuck-read-blob-reuse

## Context

The [`check-stuck`](../../../runtime/src/primitives/check_stuck.rs) primitive's `find_in_progress_commit` walker at `runtime/src/primitives/check_stuck.rs:137-141` keeps an inline blob-lookup pattern (`tree.get_path(...)?.find_blob(...)?.content().str::from_utf8(...)`) that duplicates the `read_blob_from_tree` helper introduced alongside the [`check-stuck-tasks-md-advancement`](check-stuck-tasks-md-advancement.md) fix. The bug-fix commit deliberately did not couple the refactor with the behavior change so the bug fix's diff stayed minimal and reviewable.

Origin: spec 022 [`review.md`](../review.md) finding for `check-stuck-tasks-md-advancement`, REUSE-002, 2026-05-18. Routed via the inbox.

## Behavior

`find_in_progress_commit` reads `spec.md` content from each walked commit's tree via the shared `read_blob_from_tree(repo, &tree, spec_rel)` helper instead of the inline `tree.get_path(...).find_blob(...).content()` chain. Status extraction (`extract_status` mapped over the returned `Option<String>`) stays in the caller. The walk's observable behavior is unchanged — the same `newest_in_progress` sha is returned for the same input history.

## Edge Cases

- **`spec.md` missing from a walked commit.** `read_blob_from_tree` returns `Ok(None)`; the caller treats this identically to the current `tree.get_path(...).ok()` `None` branch — status is `None`, the transition check skips that commit.
- **`spec.md` blob is non-UTF8.** `read_blob_from_tree`'s lossy decode preserves the current `from_utf8(...).unwrap_or("")` fall-through — `extract_status` runs against the decoded body and returns `None` if no `status:` line parses.

## Open Questions

*None.*

## Resolved Questions

*None.*
