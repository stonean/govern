# 001 — System Spec Templates

**Status:** done
**Dependencies:** none

Templates for the cross-cutting system specs that the constitution references but does not provide: `system.md`, `errors.md`, and `events.md`.

## Problem

The constitution's spec phase defines a directory structure that includes `system.md`, `errors.md`, and `events.md` under `specs/`. The README tells adopters to "write `specs/system.md` describing your architecture" but provides no template or guidance on what sections to include. Projects like anvil have built these from scratch, establishing patterns that should be reusable.

## Behavior

Governance provides three new templates in the `templates/` directory. Each template has placeholder sections with comments explaining what to fill in, following the same pattern as existing templates (spec.md, plan.md, etc.).

### system.md template

Prompts adopters for architectural patterns that feature specs reference. Sections are prompts, not prescriptions — adopters include what applies and remove what doesn't:

- Configuration approach (environment variables, config files, etc.)
- Application lifecycle (startup sequence, initialization order)
- Request or message lifecycle (middleware chain, handler pattern)
- Multi-tenancy or scoping model (if applicable)
- Shared infrastructure packages/modules
- Module or component pattern (isolation rules, dependency injection)

### errors.md template

Covers error handling conventions:

- Error response format (JSON structure, fields)
- Error code naming convention
- HTTP status code mapping (or equivalent for non-HTTP)
- Validation error format (per-field details)
- Logging conventions for errors (severity mapping)
- Internal vs external error exposure rules

### events.md template

An event catalog — a registry of event types populated as features are built. Includes:

- Event catalog structure (how to document each event type)
- Event envelope or message format
- Subject or topic naming convention
- Publisher and subscriber documentation pattern
- A comment suggesting that projects consider specifying retry policy and dead-letter handling as dedicated feature specs

## Acceptance Criteria

- [x] `templates/system.md` exists with placeholder sections for configuration, lifecycle, request flow, shared infrastructure, and module pattern
- [x] `templates/errors.md` exists with placeholder sections for error format, code convention, status mapping, validation errors, and logging
- [x] `templates/events.md` exists with placeholder sections for event catalog, envelope format, naming convention, and a comment suggesting retry/dead-letter as feature specs
- [x] Each template uses HTML comments with commented-out example content, consistent with existing template style
- [x] Templates are technology-agnostic — no language-specific code or framework references
- [x] Each template starts with a top-level heading and passes markdownlint

## Resolved Questions

- **Graceful shutdown in system.md** — too implementation-specific for a template. System.md sections are prompts for useful architectural patterns, not prescriptions. Shutdown behavior belongs in a feature spec if needed.
- **Retry policy and dead-letter in events.md** — keep in feature specs, not the catalog. Events.md is a registry. Include a comment suggesting projects consider specifying retry policy and dead-letter handling as dedicated feature specs.
- **Example content style** — use commented-out examples matching existing template style (spec.md, plan.md, etc.). Examples make templates self-documenting.
