# Constitution

The governing rules for spec-driven software development. This document defines the principles, workflow, and quality gates that apply to every project regardless of tech stack.

<!-- §principles -->

## Guiding Principles

These are evaluation criteria, not implementation instructions. Use them to identify gaps or violations, not to drive design decisions.

### Technology

- **Secure:** protect sensitive data through industry standards and best practices
- **Scalable:** design and implement to be dynamically scaled
- **Learnable:** fast onboarding through clear patterns, documentation, and accessible codebase design
- **Reliable:** graceful degradation and automatic recovery when components fail
- **Recordable:** accurate, durable data capture for business metrics, audit trails, and event tracing
- **Supportable:** simple and quick to detect, identify, and resolve issues
- **Automated:** humans only do what computers can't
- **Testable:** design for security, unit, functional, and load testing
- **Consumable:** simple and intuitive interfaces into our systems
- **Verified:** nothing reaches production without validation

### Business

- **Fast:** responsive systems, short time to market, rapid updates and fixes
- **Serviceable:** solutions exist to serve identified needs, not to justify themselves
- **Evolvable:** the business can adapt, grow, and create products and services as needs change
- **Flexible:** customers are served by products and services that fit their varied needs
- **Observable:** clear, real-time visibility into product and service performance
- **Compliant:** meet regulatory, legal, and industry requirements
- **Cost-conscious:** optimize cost across building, operating, and scaling products and services

<!-- §pipeline -->

## Development Pipeline

Every feature follows the pipeline: **spec → plan → tasks → implement**. No code is written without a spec. No implementation begins without a plan.

<!-- §spec-phase -->

### Spec Phase

Define *what* the feature does and *why*. A spec captures requirements, contracts, and constraints without prescribing implementation details.

Each feature lives in a numbered directory under `specs/`:

```text
specs/
  system.md              # Architecture, shared conventions
  events.md              # Global event catalog
  errors.md              # Error handling conventions
  {NNN-feature}/
    spec.md              # Requirements, contracts, acceptance criteria
    research.md          # (optional) Background research, prior art
    plan.md              # Implementation approach, technical decisions
    data-model.md        # (optional) Domain entities and data structures, generated during plan phase
    tasks.md             # Discrete work items derived from the plan
    scenarios/           # (optional) Scenario files elaborating spec sections
      {slug}.md          # One file per scenario
```

<!-- §spec-requirements -->

#### Spec requirements

- Every spec includes a **Status** indicator: `draft`, `clarified`, `planned`, `in-progress`, or `done`
- Every spec includes **Acceptance Criteria** — concrete, testable conditions that define "done"
- Every spec includes **Open Questions** — uncertainties and unresolved decisions
- Every spec lists **Dependencies** — other specs this feature depends on
- Open questions must be resolved before moving to the plan phase
- Specs describe behavior and contracts, not implementation

<!-- §spec-lifecycle -->

#### Spec lifecycle

| Status | Meaning |
| --- | --- |
| `draft` | Initial spec written, may have unresolved open questions |
| `clarified` | All open questions resolved, acceptance criteria are concrete and testable |
| `planned` | Plan and tasks exist, readiness check passed |
| `in-progress` | Implementation has started |
| `done` | All acceptance criteria verified, code merged |

```text
draft ──/clarify──▶ clarified ──/plan──▶ planned ──/implement──▶ in-progress ──/implement──▶ done
```

Forward edges only — `/clarify` raises status to `clarified`, `/plan` to `planned`, `/implement` to `in-progress` and then to `done`. Two back-edges exist:

- **Backward via new questions** — `planned` or `in-progress` → `clarified` when `/ask` records a new open question. The next `/clarify` resolves it and the spec advances forward again.
- **Backward via new scenario** — `done` → `in-progress` when `/elaborate` adds a scenario. The scenario's task is implemented and the spec returns to `done`.

This avoids spec proliferation; scenarios evolve the existing spec rather than spawning a new one.

#### The three cycles

Every spec moves through one of three cycles depending on where it starts and whether new behavior surfaces:

1. **Greenfield** — `/specify` → `/clarify` → `/plan` → `/implement` → `done`. A new feature designed from scratch.
2. **Brownfield** — `/capture` (sketch spec) → real work touches the area → `/elaborate` to add a scenario, or `/clarify` to resolve open questions, or both → `/implement` → `done`. Existing reality being absorbed into specs incrementally.
3. **Reopen** — a `done` spec is revisited because a bug, edge case, or change request surfaces. `/elaborate` adds a scenario, the spec moves back to `in-progress`, and the next pipeline command resumes from there.

All three converge on the same pipeline; what differs is where the spec enters and how precision accumulates.

<!-- §plan-phase -->

### Plan Phase

Define *how* the feature will be implemented. A plan makes technical decisions, identifies affected files, and considers trade-offs.

#### Plan requirements

- References the spec it implements
- Lists technical decisions and their rationale
- Identifies affected files and packages
- Addresses all open questions from the spec
- Produces a data model if the feature introduces or modifies domain entities or data structures

<!-- §tasks-phase -->

### Tasks Phase

Break the plan into discrete, ordered work items. Each task is small enough to implement and verify independently.

#### Task requirements

- Tasks are derived from the plan, not invented independently
- Each task has a clear definition of done
- Tasks are ordered to respect dependencies
- A task can be completed in a single working session

<!-- §readiness-check -->

### Readiness Check

Before implementation begins, verify the feature is ready to build. This is a quick pass/fail gate, not a ceremony.

- [ ] Spec status is `planned`
- [ ] Acceptance criteria are concrete and testable — no empty placeholders
- [ ] All open questions are resolved
- [ ] Data model exists if the feature introduces or modifies domain entities or data structures
- [ ] Plan does not conflict with `system.md` or other feature specs
- [ ] Tasks are ordered and each has a clear definition of done

If any item fails, fix the gap before writing code.

<!-- §implement-phase -->

### Implement Phase

Write code, tests, and migrations. Implementation follows the tasks list.

#### Implementation requirements

- Code matches the contracts defined in the spec
- Tests verify the acceptance criteria
- No work happens outside the tasks list — if new work is discovered, add it as a task first
- Refactoring that preserves existing behavior and contracts does not require a spec or scenario update. If a refactor reveals a missing requirement or changes documented behavior, update the spec or add a scenario to capture the new expectation before proceeding.

<!-- §constants -->

#### Constants and configuration

Values that an operator or deployer might need to tune — such as timeouts, retry counts, batch sizes, thresholds, and rate limits — must never appear as bare literals in the code.

- **Configurable values** — any value that determines system behavior (expiry times, retry counts, batch sizes, thresholds, rate limits, etc.) must be backed by an environment variable, following the rules in the section below.
- **Configurable ranges** — when a configurable value has meaningful bounds (e.g., minimum retries and maximum retries), expose each bound as its own environment variable so operators can tune them without code changes.
- **Fixed constants** — values that are fixed by design and never change across deployments (protocol versions, well-known header names, media types, format strings) must be named constants, not bare literals repeated across the codebase.

Ordinary literals used for local logic — loop indices, string formatting within a function, intermediate calculations — do not need to be extracted.

Organize constants into two tiers:

- **Shared constants** — values used across multiple modules live in a centralized location (e.g., `shared/constants/`). This makes cross-cutting defaults easy to find and audit.
- **Module-local constants** — values used only within a single module live in that module's own constants file. This keeps the module self-contained and avoids coupling unrelated modules through a shared import.

<!-- §env-vars -->

#### Environment variables

When a feature introduces environment variables, follow these rules:

- **`.env.example`** — add every new variable with a descriptive comment and a safe placeholder value. This file is the single source of truth for what the application expects.
- **Defaults** — every environment variable must have a default fallback defined as a named constant. Never scatter bare literals across the codebase. Read the variable once at startup and fall back to the constant when unset.
- **Validation** — validate that every required environment variable resolves to a usable value (either from the environment or its default) at startup. Fail fast with a clear error message naming any variable that cannot be resolved.
- **Time values** — include the unit in the variable name (`_MS`, `_SECONDS`, `_MINUTES`). The corresponding constant must also make the unit explicit (e.g., `DEFAULT_SHUTDOWN_TIMEOUT_SECONDS = 30`).

<!-- §lightweight-track -->

### Lightweight Track

Not every feature needs the full pipeline. Small, well-understood changes with no open questions and no cross-module impact can use a combined `spec-and-plan.md` that merges the spec and plan phases into a single document, then move directly to tasks.

Use the lightweight track when **all** of the following are true:

- The feature touches a single module or package
- There are no open questions — the approach is obvious
- The data model change is trivial or nonexistent
- The spec fits in under 50 lines

If any of these conditions are not met, use the full pipeline.

<!-- §bug-handling -->

## Bug Handling

Bugs are unwritten scenarios. Rather than tracking defects in a separate system, every bug is evidence that a spec is missing, ambiguous, or violated.

### Bug Decision Tree

When a bug is reported, follow this decision tree in order:

1. **No spec exists for the behavior** — the bug is a gap. Write the spec first, then fix the code.
2. **Spec exists but is ambiguous or incomplete** — the bug is a spec deficiency. Correct or enhance the spec, then fix the implementation.
3. **Spec is clear but implementation is wrong** — add a scenario capturing the correct behavior, then fix the code.

In all three cases, the spec becomes more precise. The scenario or spec update is the primary artifact, not a bug report.

<!-- §scenarios -->

### Scenarios

A scenario is a spec at a lower level of abstraction — same format, same discipline, narrower scope. Scenarios live in a `scenarios/` subdirectory alongside the spec they elaborate.

Each scenario file contains:

- **spec-ref** — a reference to the parent spec and section the scenario elaborates
- **Context** — the specific situation or precondition
- **Behavior** — what the system does in that situation
- **Edge Cases** — boundary conditions and exceptions (optional)

Scenarios use plain language. Given/When/Then syntax is not required.

#### Scenario lifecycle

Scenarios do not have their own status field. A scenario is either written (merged) or not. When a scenario is created, a task is appended to the parent spec's `tasks.md` referencing the scenario. The task carries the completion status — the scenario itself is a permanent requirement document.

- The parent spec's status remains `in-progress` while scenario tasks are being worked
- When the task is complete, the scenario stays as documentation of the expected behavior
- If a scenario becomes obsolete, it is deleted — not marked with a status

#### When to create a scenario

- A bug surfaces that the spec covers at a high level but does not describe in sufficient detail
- An edge case is discovered during implementation or review
- A spec section is growing too large and needs to be decomposed

#### When a scenario is not needed

- The spec itself was missing or ambiguous — fix the spec directly
- The behavior is already captured by an existing scenario — update the existing file

<!-- §scenario-promotion -->

#### Scenario promotion

In brownfield projects, scenarios serve a dual purpose: they elaborate edge cases (as in greenfield) and they decompose broad features into distinct workflows. When a scenario grows complex enough, it signals that the behavior warrants its own feature spec.

Indicators that a scenario should be promoted:

- The scenario has more than three edge cases
- The scenario's behavior section is longer than the parent spec's
- The scenario has open questions unrelated to the parent spec's domain
- Multiple scenarios in the same feature share overlapping concerns that would be better unified in their own spec

To promote: the user runs `/specify` (for new behavior) or `/capture` (for another existing feature) to create the new spec, then replaces the original scenario with a dependency reference in the parent spec.

Promotion is a user decision, not automated. The framework provides the pattern; the user recognizes when decomposition is needed.

<!-- §brownfield-inbox -->

### Brownfield Inbox

For projects adopting governance incrementally, a `specs/inbox.md` file serves as a temporary inbox for known issues not yet assigned to a feature spec. Items are recorded with `/log` and groomed into their proper home with `/groom`.

Inbox rules:

- Do not frontfill bugs that are not being actively worked on
- Write specs for areas being actively touched — let adoption spread naturally
- As specs are written, `/groom` migrates items from the inbox into their proper home
- The goal is for `inbox.md` to eventually be empty and deleted

<!-- §brownfield-process -->

### Brownfield Process

Brownfield projects adopt governance incrementally. The `/capture` command initializes a skeleton spec from freeform user input — no pressure to be comprehensive. Start broad; decompose through scenarios over time.

#### Capture → incremental growth → promotion

1. **Capture** — the user describes an existing feature in their own words. `/capture` drafts a skeleton spec at `draft` status with whatever behavior is known. Sparse acceptance criteria are expected and valid.
2. **Incremental growth** — every subsequent touch on the feature adds precision:
   - A **bug fix** reveals missing behavior → adds an acceptance criterion or scenario
   - An **enhancement** adds new behavior → follows the normal pipeline (spec change before implementation)
   - A **clarification** resolves an open question → narrows ambiguity
3. **Promotion** — when a scenario outgrows its parent spec, the user promotes it to its own feature spec (see [Scenario promotion](#scenario-promotion))

Over time the spec converges on a complete description of the feature — not from a documentation effort, but as a side effect of doing work.

#### Inbox integration

When a `/groom` pass encounters an item that does not map to any existing spec, `/groom` directs the user to run `/capture` to initialize a spec first, then return to process the item. The commands stay decoupled — `/log` records, `/groom` routes, `/capture` creates specs.

<!-- §text-first-artifacts -->

## Text-First Artifacts

Governance treats every artifact — constitution, specs, plans, tasks, scenarios, rules — as plain markdown the agent can edit with `Edit`. This is load-bearing: the agent's write path stays simple, PRs review glanceably, merge conflicts stay rare and human-resolvable, and adopting governance requires no bootstrap tooling beyond the AI agent itself.

### Principles

- All governance artifacts are markdown by default. The agent reads and writes them with the same `Edit` flow used for code.
- Structured metadata lives in YAML frontmatter at the top of each markdown file; the document body remains markdown prose.
- Cross-artifact references use standard relative markdown links (`[label](../path.md)`), not wiki-links — this keeps PRs reviewable on GitHub and viewers like Quartz/Obsidian still resolve them.
- Source-of-truth artifacts are markdown. Structured derived views (SQLite caches, generated graph data, JSON indexes) are permitted only as gitignored build artifacts that consumers regenerate on demand. They never become the canonical record.
- Exceptions to text-first source-of-truth require an explicit constitutional amendment with stated rationale.

### Frontmatter Schema

The frontmatter schema applies to **spec files** (`spec.md`, `spec-and-plan.md`) and **scenario files** (`scenarios/{slug}.md`). Other governance artifacts (`system.md`, `errors.md`, `events.md`, `inbox.md`, plan files, tasks files, rule files, README files) MAY include frontmatter when a specific consumer benefits, but are not required to.

#### Spec files

| Field | Required | Type | Allowed values | Description |
| --- | --- | --- | --- | --- |
| `status` | yes | string | `draft`, `clarified`, `planned`, `in-progress`, `done` | Spec lifecycle state |
| `dependencies` | yes | list of strings | spec slugs (e.g., `002-events`); empty list permitted | Specs this feature depends on |
| `tags` | no | list of strings | free-form; see starter vocabulary below | Cross-cutting categories used by graph-view consumers |

#### Scenario files

| Field | Required | Type | Allowed values | Description |
| --- | --- | --- | --- | --- |
| `spec-ref` | yes | string | parent spec ref, conventionally `{NNN-feature} — {Section}` | Identifies the parent spec and section the scenario elaborates |
| `tags` | no | list of strings | free-form | Scenario-level cross-cutting tags |

#### Open-schema rule

Additional fields beyond those listed above are permitted and ignored by uninterested consumers. Examples adopters or future governance work might add: `owner`, `target_release`, `created_at`, `description`, `aliases`. Consumers MUST NOT error on the presence of unknown fields. `/gov:validate` reports unknown fields as informational findings (not errors).

### Validation Severity

`/gov:validate` checks frontmatter against this schema with the following severity:

- **Hard fail** — frontmatter block missing on a spec or scenario file; frontmatter YAML malformed; `status` missing or not in the allowed set; `dependencies` missing or not a list; `spec-ref` missing on a scenario.
- **Advisory** — `tags` missing or empty; existing checkbox/cross-reference checks.
- **Informational** — unknown fields present.

Hard fails block the validation pass. Advisory and informational findings are reported but do not block.

### Starter Tag Vocabulary

Published as guidance, not enforcement. Adopters and future specs MAY introduce new tags as needed; `/gov:specify` surfaces existing tags from sibling specs as autocomplete to drive convergence by reuse rather than ceremony.

| Tag | Suggested use |
| --- | --- |
| `cli` | Specs about slash commands or command-line interactions |
| `bootstrap` | Specs about adopting governance, project scaffolding, or initialization |
| `process` | Specs about workflow, lifecycle, or pipeline behavior |
| `templates` | Specs about template files (spec, plan, scenario, project-readme, etc.) |
| `security` | Specs about security rules, authentication, authorization |
| `agent` | Specs about AI-agent behavior, capabilities, or coordination |
| `format` | Specs about artifact formats, schemas, or serialization conventions |
| `pipeline` | Specs about the spec → plan → tasks → implement flow |
| `migration` | Specs that convert existing artifacts to a new format or convention |

<!-- §pipeline-boundaries -->

## Pipeline Boundaries

- Never implement without a spec
- Never plan without resolving open questions
- Never skip phases — each phase produces artifacts the next phase consumes
- Never transition a spec to the next status without explicit user approval — present the work done and wait for the user to confirm before updating the status field
- Specs and plans are living documents — update them when decisions change, but don't backtrack silently

<!-- §cross-spec-impact -->

### Cross-Spec Impact

Specs are self-contained. When work on one spec identifies changes that affect another spec, those changes are recorded in the affected spec — not left as a note in the originating spec. The affected spec is the source of truth for its own behavior.

This applies when:

- A feature renames or supersedes an artifact from a prior spec
- Work on spec A reveals that spec B needs a new acceptance criterion or scenario
- A scenario in spec A exposes an edge case that belongs to spec B
- An implementation decision in spec A's plan creates a constraint for spec B

In each case:

- The change is recorded in the affected spec as a new acceptance criterion, scenario, or signpost note
- The signpost references the originating spec so the reader understands why the change was made
- If the affected spec is `done`, adding the change reopens it to `in-progress` per the normal lifecycle

The originating spec's acceptance criteria include delivering the cross-spec update. This ensures the change is tracked as part of the work that discovered it.

<!-- §numbering -->

## Numbering Convention

Feature directories use three-digit zero-padded numbers: `000-skeleton`, `001-observability`, `002-events`. Numbers establish creation order and suggest a natural implementation sequence, but dependencies between features determine the actual build order.

<!-- §markdown-standards -->

## Markdown Standards

All `.md` files must pass `npx markdownlint-cli2` using the project config in `.markdownlint-cli2.jsonc`.

Key rules:

- Every fenced code block must specify a language — **MD040**
- Files must start with a top-level heading — **MD041**
- No trailing spaces or missing blank lines around headings, lists, and fenced code blocks
- ATX-style headings only (`#`, `##`, etc.)
- Heading levels increment by one — **MD001**
- No duplicate headings at the same level within the same parent — **MD024** (siblings\_only)
- Link fragments must reference valid heading anchors — **MD051**
- Ordered lists use sequential numbering — **MD029**
- Tables use compact style: `| text |` — **MD060**
- Line length is not enforced (MD013 disabled)
- Inline HTML is allowed (MD033 disabled)
