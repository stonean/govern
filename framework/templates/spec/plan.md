# {NNN} — {Feature Name} Plan

Implements [{NNN} — {Feature Name}](spec.md).

## Overview

<!-- Brief summary of the implementation approach and key technical decisions. -->

## Technical Decisions

<!-- List decisions made during planning and their rationale. Example:

### Session storage

Sessions are stored in PostgreSQL for durability and cached in Valkey for speed.
Alternative considered: Valkey-only — rejected because sessions would be lost on cache eviction.

-->

## Affected Files

<!-- Planning aid only — `/{project}:implement` derives the runtime write boundary
     from `git diff` against the spec dir's first commit. List files you expect
     to create or modify so reviewers can sanity-check scope. The list does not
     need to be exhaustive; implement-time additions surface naturally. Example:

| File | Action | Purpose |
| --- | --- | --- |
| `src/auth/session` | Create | Session management logic |
| `src/middleware/auth` | Create | Auth middleware |
| `migrations/20250228_create_sessions` | Create | Sessions table |

-->
