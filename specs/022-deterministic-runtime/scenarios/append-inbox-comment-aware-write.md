---
section: "Follow-on scenarios"
---

# Append-inbox-comment-aware-write

## Context

`append-inbox`'s `append_bullet` (`runtime/src/primitives/append_inbox.rs`) writes the new `- [ ] {text}` bullet **comment-blind** — it appends at the file's end without regard for HTML comments or code fences — while the inbox *read* side counts and dedups **comment-aware** (`iter_bullets` / `count_inbox_bullets` in `primitives/mod.rs`, the shared inbox grammar that ignores bullets inside `<!-- … -->`). The write and read disagree about what counts as inbox content.

If `inbox.md` ends inside an *unclosed* `<!--` HTML comment (no closing `-->` — malformed input), the appended bullet lands inside the comment region, and `count_inbox_bullets` then skips it: the item is both invisible to the reader and undercounted. The shipped template closes its comment before the trailing bullet, so this requires malformed input and is low priority — the latent smell is the asymmetry itself (comment-aware read, comment-blind write). Surfaced 2026-07-12 during the command-runtime alignment review (gvrn 0.20.0).

## Behavior

- `append_bullet` appends the new bullet at a position the read side will count: the last position outside any open HTML comment or code fence, mirroring the comment/fence awareness of `iter_bullets` / `count_inbox_bullets`. An appended item is always visible to the reader and included in the count — write and read share one notion of inbox content.
- A well-formed `inbox.md` (every comment and fence closed) appends exactly as before — the bullet lands after the final closed region.

## Edge Cases

- `inbox.md` ending inside an unclosed `<!--` comment: the bullet lands in a position `count_inbox_bullets` counts (before the unterminated region), never inside the comment where it would be skipped.
- A trailing unterminated code fence is handled the same way as an unterminated comment.
- An empty or comment-only inbox still receives the appended bullet in a counted position.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
