---
section: "Follow-on scenarios"
---

# Mark-task-backtick-headings

## Context

Spec 022's `mark-task` and `read-tasks` primitives both parse `tasks.md` heading lines to locate a task by number. A heading containing backtick-quoted inline code — e.g., `### 19. Dedup ` + backtick + `/configure` + backtick + ` permission entries via new gvrn primitive` — parses correctly via `read-tasks` (returns the task with `"number":"19"`) but fails via `mark-task` with `task '19' not found`. The inconsistency is between the two primitives' heading parsers; `mark-task` does not yet route through the shared `primitives::mod::parse_atx_heading` helper that `read-tasks` uses.

Origin: observed 2026-05-19 during `/gov:implement` on spec 023 task #19. The work was unblocked by editing the checkbox directly, but the bug remains and any task title with backticks (a common shape for slash-command names, primitive names, and file paths) will hit the same wall. This belongs in the `runtime-primitive-structural-bugs` family on spec 022.

## Behavior

- `mark-task` and `read-tasks` MUST recognize the same set of task headings on a given `tasks.md`. For every `(file, task-number)` pair, `read-tasks` returning a task with that number implies `mark-task` resolves the same task — and vice versa.
- Heading parsing routes through a single shared helper. The existing `primitives::mod::parse_atx_heading` is the canonical surface; `mark-task` consumes it the same way `read-tasks` already does. Any heading-shape edge case fixed once propagates to both primitives.
- The fix is REUSE-only — observable behavior of `read-tasks` is unchanged; `mark-task` gains the same recognition surface.

## Edge Cases

- **Multiple backtick spans in one heading** (e.g., `### 5. Wire ` + backtick + `read-tasks` + backtick + ` to ` + backtick + `mark-task` + backtick): both primitives resolve the same task number `5`.
- **ATX-closed heading form** (`### 5. Title ###`): the shared helper already strips trailing `#` runs; `mark-task` picks up the same normalization.
- **Backtick at start of title** (`### 7. ` + backtick + `gvrn` + backtick + ` self-test`): the dot-separator (`.`) is the parser's discriminator, not the leading character of the title — backtick at position 0 of the title is benign once heading parsing uses the shared helper.
- **Empty backtick span** (e.g., a heading text fragment with `` `` `` mid-string): technically valid markdown; both primitives should treat the span as zero-length and not lose the surrounding characters.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
