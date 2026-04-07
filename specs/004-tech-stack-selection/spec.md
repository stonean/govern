# 004 — Tech Stack Selection

**Status:** done
**Dependencies:** 003-bootstrap-automation

Interactive tech stack selection during `/gov:init` that collects richer project metadata beyond primary language(s). From those selections, populate the AGENTS.md Tech Stack table so the project's technology context is captured at creation time.

## Problem

Init currently asks only for primary language(s). Real projects have databases, messaging systems, frameworks, CSS preprocessors, test runners, and other infrastructure that shape how the project is built. Without this context, the AGENTS.md Tech Stack table is left blank — the user fills it manually every time. Tech-stack-specific conventions (Code Style, Testing, Gotchas) are the responsibility of the dev project, not governance, but the Tech Stack table itself should be populated at init time.

## Behavior

During `/gov:init`, after collecting the project slug, path, and description, replace the single "primary language(s)" question with a guided tech stack questionnaire. Selections populate the AGENTS.md Tech Stack table.

### Flow

1. **Project type** — ask: backend, frontend, or fullstack
2. **Backend questions** (if backend or fullstack):
   - Primary language (e.g., TypeScript, Python, Go, Ruby)
   - Framework (e.g., Fastify, FastAPI, Gin, Rails)
   - Database (e.g., PostgreSQL, MySQL, SQLite, MongoDB)
   - Messaging (e.g., NATS, Kafka, RabbitMQ, Redis Pub/Sub)
   - Test runner (e.g., Vite, pytest, go test, RSpec)
3. **Frontend questions** (if frontend or fullstack):
   - Primary language (e.g., TypeScript, JavaScript)
   - Framework (e.g., Svelte, Vue, React, Next.js)
   - CSS/UI (e.g., Tailwind, SCSS, styled-components)
   - Test runner (e.g., Vitest, Jest, Playwright)

For fullstack projects, backend questions are asked first, then frontend.

Each category is optional — the user can skip any. Every question offers 2–4 common choices plus "Other" and "Skip", following the existing init question format.

### AGENTS.md population

For each selected technology, add a row to the AGENTS.md Tech Stack table with the layer, technology name, and its role. Skipping all categories produces the same blank-table AGENTS.md as today. Code Style, Testing, and Gotchas sections remain as comment placeholders — populating those is the dev project's responsibility.

### .gitignore

Language-specific `.gitignore` patterns are fetched based on the language(s) selected in this flow, replacing the current standalone language question.

## Acceptance Criteria

- [x] Init asks project type (backend, frontend, fullstack) before any language question
- [x] Backend-only projects are not asked CSS/UI or frontend framework questions
- [x] Frontend-only projects are not asked database or messaging questions
- [x] Fullstack projects are asked backend questions first, then frontend questions
- [x] Each category offers 2–4 common choices plus "Other" and "Skip"
- [x] Selected technologies populate the AGENTS.md Tech Stack table with layer, technology, and role
- [x] Skipping all categories produces the same AGENTS.md as today (backwards compatible)
- [x] The single "primary language(s)" question is replaced by this flow — no duplicate language prompts
- [x] `.gitignore` patterns are fetched for all languages selected during the flow

## Open Questions

None — all resolved during clarification.
