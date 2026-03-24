# 004 — Tech Stack Selection Plan

## Overview

Modify the `/gov:init` command to replace the single "primary language(s)" question with a multi-step tech stack questionnaire. The flow starts with project type (backend, frontend, fullstack), then asks relevant technology questions per category. Selections populate the AGENTS.md Tech Stack table and drive `.gitignore` fetching.

The change is entirely within `init.md` (the command definition) and `AGENTS.md` (the template). No new files or infrastructure needed.

## Technical Decisions

### Project type gates category visibility

A static mapping determines which question categories apply to each project type:

- **backend** → language, framework, database, messaging, test runner
- **frontend** → language, framework, CSS/UI, test runner
- **fullstack** → all categories, backend first then frontend

This avoids any dynamic discovery mechanism. Adding a new category means editing `init.md`.

### Tech Stack table replaces comment placeholder

The AGENTS.md template currently has a comment block showing a Tech Stack table example. When technologies are selected, the comment is replaced with an actual table. When all categories are skipped, the comment remains unchanged (backwards compatible).

Each selection maps to a table row:

| Layer | Technology | Role |
| --- | --- | --- |
| **Language** | Go | Application logic |
| **Framework** | Gin | HTTP framework |
| **Database** | PostgreSQL | Primary data store |

The layer name comes from the question category. The role is a standard label per category (e.g., "Application logic" for language, "HTTP framework" for framework, "Primary data store" for database).

### Fullstack projects get two language rows

For fullstack projects, backend and frontend languages are separate rows:

| Layer | Technology | Role |
| --- | --- | --- |
| **Backend language** | Go | Backend application logic |
| **Frontend language** | TypeScript | Frontend application logic |

### .gitignore fetching uses selected languages

The current step 8 in init fetches `.gitignore` patterns based on the "primary language(s)" input. This input no longer exists — instead, collect languages from the backend and/or frontend language selections and fetch patterns for each.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `.claude/commands/gov/init.md` | Modify | Replace language question with tech stack flow |
| `AGENTS.md` | Modify | Adjust Tech Stack comment to support replacement |

## Open Questions Resolved

- **Snippet files**: Not needed. Governance populates only the Tech Stack table. Conventions are the dev project's responsibility.
- **Composability**: Not applicable — each selection is an independent table row, no merging needed.
- **User-contributed snippets**: Out of scope — no snippet mechanism exists.
- **Category filtering**: Static mapping in `init.md`, not dynamic discovery.
