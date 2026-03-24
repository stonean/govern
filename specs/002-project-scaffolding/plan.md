# 002 — Project Scaffolding Templates Plan

## Overview

Create three new template files in the existing `templates/` directory: `project-readme.md`, `gitignore`, and `claude-md.md`. These are project-level files that the init command (spec 003) copies into new projects. They use `{project}` placeholders that get replaced during bootstrap.

## Technical Decisions

### README template derived from anvil

The project README template is based on anvil's `README.md` structure, generalized:

- Anvil-specific content (Go, PostgreSQL, NATS, Docker commands) replaced with `{project}` placeholders and generic sections
- Feature table format preserved — it's the standard from the constitution's numbering convention
- Slash command references use `/{project}:*` pattern
- Getting Started section references `/{project}:setup` and `/{project}:status` as decided during clarification

### Gitignore stays minimal

The `.gitignore` template contains only governance-universal entries. No language-specific patterns. The file is named `gitignore` (no dot) in `templates/` to avoid it being treated as an active gitignore by git. The init command renames it to `.gitignore` during copy.

### CLAUDE.md is two lines

The template is deliberately minimal — just the two `@import` directives. Projects may add more configuration later, but the starting point is always the same.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `templates/project-readme.md` | Create | Project README with quick start, getting started, docs, feature table, pipeline, slash commands |
| `templates/gitignore` | Create | Minimal .gitignore for secrets, claude settings, IDE, OS files |
| `templates/claude-md.md` | Create | CLAUDE.md with @import directives |

## Trade-offs

### Considered: naming the README template `readme.md`

Rejected. Using `project-readme.md` avoids confusion — `readme.md` could be mistaken for the governance project's own README. The `project-` prefix makes it clear this is a template for adopting projects.

### Considered: including `{description}` placeholder in templates

Rejected. Only `{project}` is used as a placeholder. The project description appears in specific locations (README header, AGENTS.md intro) and the init command fills those in directly from user input rather than using a second placeholder.

## Open Questions Resolved

All open questions were resolved during clarification. See spec.md Resolved Questions section.
