# 000 — Slash Command Templates Plan

## Overview

Create ten generic slash command `.md` files in a `commands/` directory at the governance root. Each command is derived from anvil's working implementation but generalized: anvil-specific references are replaced with `{project}` placeholders, and anvil-specific logic (Go code style, module patterns) is removed in favor of references to the constitution and AGENTS.md.

## Technical Decisions

### Directory location

Commands live at `commands/{command}.md` in the governance root. This is the template source — adopting projects copy these to `.claude/commands/{project}/` and replace `{project}` placeholders.

Rationale: Keeping them at the governance root (not under `.claude/commands/`) avoids them being treated as active slash commands in the governance repo itself. Governance's own commands (like `/gov:init`) live separately in `.claude/commands/gov/`.

### Parameterization approach

Every command uses literal `{project}` as the placeholder. This appears in:

- Command cross-references: `/{project}:clarify`, `/{project}:plan`
- Session file path: `.claude/{project}-session.json`
- Command directory references: `.claude/commands/{project}/`

No other placeholders are needed. The bootstrap command (spec 003) handles find-and-replace during project scaffolding.

### Deriving from anvil

Each command is based on the corresponding anvil command with these transformations:

- Replace `anvil` with `{project}` in all references
- Remove anvil-specific file paths (`shared/`, `modules/`, `docker-compose.yml`)
- Remove Go-specific conventions (Querier interface, pgx patterns)
- Replace anvil-specific template paths (`specs/templates/spec-template.md`) with generic `specs/templates/spec.md`
- Keep constitution references (pipeline gates, readiness check, spec lifecycle)
- Keep AGENTS.md references (conventions, boundaries) as generic pointers

### Spec file detection

Commands that operate on a feature's spec need to handle both `spec.md` and `spec-and-plan.md`. The pattern is:

1. Check for `spec.md` first
2. If not found, check for `spec-and-plan.md`
3. If neither exists, report "no spec found"

This applies to: target, clarify, plan, implement, validate, next, status.

### Validate and markdownlint

The validate command runs `npx markdownlint-cli2` on all `.md` files in the feature directory as its final check. This is reported as a PASS/FAIL alongside the other artifact checks.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `commands/about.md` | Create | Pipeline overview, no file reads |
| `commands/target.md` | Create | Set session target |
| `commands/status.md` | Create | Dashboard of all specs |
| `commands/setup.md` | Create | Configure permissions |
| `commands/specify.md` | Create | Create new feature spec |
| `commands/clarify.md` | Create | Resolve open questions, advance to clarified |
| `commands/plan.md` | Create | Create plan and tasks, advance to planned |
| `commands/implement.md` | Create | Execute tasks, advance to done |
| `commands/validate.md` | Create | Audit artifacts for consistency |
| `commands/next.md` | Create | Auto-advance to next pipeline phase |

## Trade-offs

### Considered: single-file command reference instead of ten files

Rejected. Each command needs enough instruction detail that a single file would be unwieldy. Separate files also match how Claude Code discovers and lists slash commands — one file per command.

### Considered: using a different placeholder syntax (e.g., `{{project}}`, `$PROJECT`)

Rejected. `{project}` is simple, readable in markdown, and unlikely to conflict with other content. Curly braces are not used in the constitution or template content.

## Open Questions Resolved

All open questions were resolved during clarification. See spec.md Resolved Questions section.
