# {NNN} — {Feature Name} Plan

## Overview

<!-- Brief summary of the implementation approach and key technical decisions. -->

## Technical Decisions

<!-- List decisions made during planning and their rationale. Example:

### Session storage

Sessions are stored in PostgreSQL for durability and cached in Valkey for speed.
Alternative considered: Valkey-only — rejected because sessions would be lost on cache eviction.

-->

## Affected Files

<!-- List files that will be created or modified. Example:

| File | Action | Purpose |
| --- | --- | --- |
| `src/auth/session` | Create | Session management logic |
| `src/middleware/auth` | Create | Auth middleware |
| `migrations/20250228_create_sessions` | Create | Sessions table |

-->

## Open Questions Resolved

<!-- Reference open questions from the spec and record their resolution. Example:

- **Rate limit scope**: Per-tenant, using Valkey sliding window counters.

-->
