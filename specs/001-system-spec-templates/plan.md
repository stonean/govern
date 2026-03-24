# 001 — System Spec Templates Plan

## Overview

Create three new template files in the existing `templates/` directory: `system.md`, `errors.md`, and `events.md`. Each follows the established template pattern — a top-level heading, placeholder sections with HTML comments containing commented-out examples, and no technology-specific content.

## Technical Decisions

### Template style consistency

All existing templates (`spec.md`, `plan.md`, `tasks.md`, `data-model.md`, `research.md`) use the same pattern:

- Top-level heading with `{NNN}` or descriptive title
- Sections with `<!-- -->` HTML comments explaining what to fill in
- Commented-out examples showing the expected format
- No mandatory content — everything is guidance

The three new templates follow this exact pattern. The difference is that system spec templates are placed directly in `specs/` (not in a numbered feature directory), so their headings use a descriptive title rather than `{NNN} — {Feature Name}`.

### Section selection based on anvil analysis

Sections were chosen by analyzing what anvil's `system.md`, `errors.md`, and `events.md` contain, then generalizing. Anvil-specific content (Go patterns, pgx, NATS subjects) is stripped and replaced with technology-agnostic prompts.

### All three are living documents

All three system specs grow as features are built, at different rates:

- **system.md** — established early with the core architecture, then updated minimally as shared infrastructure evolves
- **errors.md** — grows as modules add error codes and new response patterns emerge
- **events.md** — grows the most, with new entries added as each feature publishes or subscribes to events

The templates should reflect this: each starts with conventions and structure up front, plus a catalog or registry section that expands over time. `events.md` starts the most empty since it's purely a catalog populated by feature work.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `templates/system.md` | Create | Architecture overview template with sections for configuration, lifecycle, request flow, shared infrastructure, module pattern |
| `templates/errors.md` | Create | Error handling conventions template with sections for response format, codes, status mapping, validation, logging |
| `templates/events.md` | Create | Event catalog template with structure for documenting event types, envelope format, naming convention |

## Trade-offs

### Considered: putting system spec templates in a separate directory (e.g., `templates/system/`)

Rejected. The existing `templates/` directory is flat with one file per template. Adding a subdirectory breaks the pattern without adding value. Three more files in the same directory is straightforward.

### Considered: including a `spec-and-plan.md` combined template for lightweight track

Deferred. The lightweight track template is referenced in spec 000 (slash commands) and will be needed when the `specify` command creates lightweight track features. It should be added alongside the slash command implementation, not here, since it's a variant of the existing `spec.md` and `plan.md` templates rather than a system spec.

## Open Questions Resolved

All open questions were resolved during clarification. See spec.md Resolved Questions section.
