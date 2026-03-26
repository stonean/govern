# SDD Context Document

> This document captures the Spec Driven Development framework — principles,
> pipeline, spec and scenario conventions, bug handling, brownfield adoption,
> operational commands, and quality standards.

## Core Principle

The specification is the primary artifact and source of truth. Code is derived from
the spec, not the other way around. A spec describes *what the system does*, not
*how it does it*.

### What a Spec Is Not

- Not a PRD — specs are living documents that stay accurate after code is written
- Not Agile or Scrum — SDD is about what the source of truth is, not how teams organize work
- Not vibe coding — SDD is the discipline layer that makes AI coding reliable
- Not just using AI agents — agents are tools, SDD is what makes them consistent

## Development Pipeline

Every feature follows the pipeline: **spec → plan → tasks → implement**. No code
is written without a spec. No implementation begins without a plan.

| Phase | Produces | Key rule |
| --- | --- | --- |
| **Spec** | What the feature does, requirements, contracts, constraints | All open questions resolved before planning |
| **Plan** | How the feature will be implemented, technical decisions, affected files | References the spec, addresses all open questions, produces data model if needed |
| **Tasks** | Discrete, ordered work items derived from the plan | Each task has a definition of done and can be completed in a single session |
| **Implement** | Code, tests, and migrations following the tasks list | Follows the tasks in order, updates spec status as work progresses |

### Pipeline Boundaries

- Never implement without a spec
- Never plan without resolving open questions
- Never skip phases — each phase produces artifacts the next phase consumes
- Never transition a spec to the next status without explicit user approval — present the work done and wait for the user to confirm before updating the status field
- Specs and plans are living documents — update them when decisions change, but don't backtrack silently

## Spec Lifecycle

Every spec carries a status that tracks its progress through the pipeline.

| Status | Meaning |
| --- | --- |
| `draft` | Initial spec written, may have unresolved open questions |
| `clarified` | All open questions resolved, acceptance criteria are concrete and testable |
| `planned` | Plan and tasks exist, readiness check passed |
| `in-progress` | Implementation has started |
| `done` | All acceptance criteria verified, code merged |

A spec advances forward through these states. Moving backward (e.g., `planned` →
`clarified`) is allowed when new questions surface during implementation. A `done`
spec is reopened by adding a new scenario — the spec status moves to `in-progress`.

## Repository Structure

```text
specs/
  system.md                # Architecture, shared conventions, request flow
  events.md                # Global event catalog (grows with features)
  errors.md                # Error handling conventions
  inbox.md                # Temporary inbox for brownfield adoption
  000-user-auth/
    spec.md                # Requirements, contracts, acceptance criteria
    plan.md                # Implementation approach, technical decisions
    tasks.md               # Discrete work items derived from the plan
    data-model.md          # (optional) Database schema
    research.md            # (optional) Background research, prior art
    scenarios/
      token-expiry.md
      invalid-credentials.md
  001-checkout/
    spec.md
    plan.md
    tasks.md
    scenarios/
      rounding-error.md
  002-security/
    spec.md                # Cross-cutting concerns are first class specs
```

**Rules:**

- Feature directories use numbered prefixes (`NNN-feature-name`) for ordering and uniqueness
- `spec.md` stays high level and stable — if scenarios are bloating it, they need their own files
- Cross-cutting concerns (security, data integrity) are first class feature specs
- Bugs do not get a top level directory — they belong to the feature they violate
- Status metadata lives inside files, not in folder structure

## Spec Format

Specs use flexible section headings — there is no fixed set of required sections
beyond the mandatory ones. Use headings that make sense for the feature.

```markdown
# {NNN} — {Feature Name}

**Status:** draft
**Dependencies:** none

{Brief description of what this feature does and why it exists.}

## {Sections}

<!-- Organize into sections that describe behavior, contracts, and constraints.
     Use headings that make sense for this feature. -->

## Acceptance Criteria

- [ ] Concrete, testable conditions that define "done"

## Open Questions

- Uncertainties, unresolved decisions, and areas needing investigation
```

**Mandatory elements:**

- **Status** — one of: draft, clarified, planned, in-progress, done
- **Dependencies** — other specs this feature depends on
- **Acceptance Criteria** — concrete, testable conditions that define "done"
- **Open Questions** — uncertainties and unresolved decisions (all must be resolved before planning)

### Lightweight Track

Small, well-understood changes can use a combined `spec-and-plan.md` that merges
the spec and plan into a single document, then move directly to tasks.

Use the lightweight track when **all** of the following are true:

- The feature touches a single module or package
- There are no open questions — the approach is obvious
- The data model change is trivial or nonexistent
- The spec fits in under 50 lines

The combined document adds `**Track:** lightweight`, `Technical Decisions`, and
`Affected Files` sections alongside the spec content. If any qualifying condition
is not met, use the full pipeline.

## Plan Phase

A plan references the spec it implements and contains:

- Technical decisions and their rationale
- Affected files and packages
- Resolution of all open questions from the spec
- A data model if the feature involves persistence

## Tasks Phase

Tasks are derived from the plan, not invented independently. Each task:

- Has a clear definition of done
- Is ordered to respect dependencies
- Can be completed in a single working session

## Readiness Check

Before implementation begins, all gates must pass:

- [ ] Spec status is `planned`
- [ ] Acceptance criteria are concrete and testable — no empty placeholders
- [ ] All open questions are resolved
- [ ] Data model exists if the feature involves persistence
- [ ] Plan does not conflict with `system.md` or other feature specs
- [ ] Tasks are ordered and each has a clear definition of done

## Scenario Format

A scenario is a spec at a lower level of abstraction. Same format, same discipline,
just narrower in scope. Anchored to the spec it elaborates on via `spec-ref`.

```markdown
# Scenario: Token Expiry Enforced on Request

**spec-ref:** 000-user-auth — Token Lifecycle

## Context
A user has an active session token that has been inactive for 24 hours.

## Behavior
When the user makes a request with the expired token the system rejects
it with a 401 and clears the token from the session store.

## Edge Cases
- Token that expires mid-request completes the current request but
  rejects the next
- Clock skew of up to 30 seconds is tolerated before expiry is enforced

## Open Questions
<!-- Questions captured via the question command. Resolved during clarify. -->

## Resolved Questions
<!-- Answers to previously open questions, preserved for context. -->
```

No Given/When/Then syntax required. Plain language is equally valid and often
more readable for complex scenarios.

### Scenario Lifecycle

Scenarios do not have their own status field. A scenario is either written (merged)
or not. When a scenario is created, a task is appended to the parent spec's
`tasks.md` referencing the scenario. The task carries the completion status — the
scenario itself is a permanent requirement document.

- If the parent spec was `done`, its status reverts to `in-progress`
- The task in `tasks.md` shows what is being worked on and links to the scenario
- When the task is complete, the scenario stays as documentation of the expected behavior
- If a scenario becomes obsolete, it is deleted — not marked with a status

### Scenario Targeting

Individual scenarios can be targeted in the session for focused work. The session
file supports an optional `scenario` and `scenarioPath` field alongside the feature
target.

- `target {feature}` — targets the feature, clears any scenario
- `target {feature}/{scenario-slug}` — targets the feature and a specific scenario

When a scenario is targeted, scenario-aware commands (`question`, `clarify`,
`status`, `implement`) operate on the scenario file instead of the parent spec.
Feature-only commands (`specify`, `plan`, `validate`) always operate at the feature
level regardless of scenario targeting. The `/scenario` command automatically sets
the newly created scenario as the session target.

## Bug Handling

A bug is just an unwritten scenario. Most bugs exist because a situation was never
formally described. The fix is not a bug report — it is a scenario added to the spec
it belongs to.

**Decision tree — in order:**

1. Does a spec exist for this behavior? If not, write it first
2. Is the spec ambiguous or incomplete? Correct or enhance it
3. Is the spec clear but the implementation wrong? Add the missing scenario, then fix the code

There is no bug file. The scenario captures the correct behavior. The git history
on that file records when and why it was added. A descriptive commit message covers
the rest.

## Brownfield Adoption

- Do not frontfill bugs you aren't actively working on
- Write specs for areas you are actively touching — let adoption spread naturally
- Use `inbox.md` as a temporary inbox for known issues not yet assigned to a feature spec
- As specs get written, items migrate from inbox into their proper home
- The goal is for `inbox.md` to eventually disappear
- SDD adoption in a brownfield project is incremental by feature area, not a big-bang effort

### Brownfield Process

The `/capture` command initializes a skeleton spec from freeform user input — no pressure to be comprehensive. Start broad; decompose through scenarios over time.

1. **Capture** — describe an existing feature in your own words. `/capture` drafts a skeleton spec at `draft` status with whatever behavior is known. Sparse acceptance criteria are expected and valid.
2. **Incremental growth** — every subsequent touch adds precision: bug fixes add acceptance criteria or scenarios, enhancements follow the normal pipeline, clarifications resolve open questions.
3. **Scenario promotion** — when a scenario outgrows its parent spec (more than three edge cases, longer behavior section, unrelated open questions), the user promotes it to its own feature spec via `/specify` or `/capture`, then replaces the original scenario with a dependency reference.

### Cross-Spec Impact

When work on one spec identifies changes that affect another spec, those changes are recorded in the affected spec — not left as a note in the originating spec. A signpost references the originating spec so the reader understands why the change was made. If the affected spec is `done`, the change reopens it to `in-progress`.

Governance can be adopted in a single command. Install the govern command for your
agent (Claude Code or Auggie), run `/govern {project-name}`, and it fetches
governance files, scaffolds the spec directory, installs slash commands, and displays
next steps. The command is idempotent — safe to run again to pick up updates.

## Slash Commands

The governance framework is operationalized through slash commands installed during
adoption. All commands are session-aware — run `/target` to set the working feature,
then use pipeline commands in context.

| Command | Purpose |
| --- | --- |
| `/target` | Set the working feature or scenario for the session |
| `/status` | Dashboard of all features' progress, or focused view of current target |
| `/about` | Project overview, constitution summary, governance version |
| `/specify` | Create a new feature spec (detects lightweight vs standard track) |
| `/clarify` | Resolve open questions, advance to `clarified` |
| `/plan` | Create plan with technical decisions, affected files, resolved questions |
| `/implement` | Work through tasks, update status to `in-progress` then `done` |
| `/validate` | Audit spec, plan, tasks, and scenarios for completeness and consistency |
| `/question` | Ask a question about the current feature or scenario |
| `/scenario` | Create a scenario for a bug fix, edge case, or behavior clarification |
| `/inbox` | Walk inbox.md items through the bug decision tree |
| `/capture` | Initialize a skeleton spec from freeform description of an existing feature |
| `/setup` | Configure agent permissions for governance commands |
| `/create` | Create a new spec artifact (plan, tasks, data model, scenario) |

### Validate

The validate command is a read-only consistency check. It reports issues as
**blocking** (must fix to advance) or **advisory** (should fix):

- Spec integrity — status, dependencies, acceptance criteria
- Artifact completeness — required files for each status level
- Plan consistency — references spec, has decisions with rationale, lists affected files
- Task consistency — references plan, has "done when" conditions, proper ordering
- Scenario consistency — every scenario has a corresponding task
- Dependencies — all declared dependencies exist and are at least `clarified`
- Cross-spec alignment — event types match events.md, error codes follow errors.md
- Markdown lint — all files pass markdownlint-cli2

## Security Rules

The framework includes enforceable security rules for backend and frontend code,
distributed via adopt. Rules use RFC 2119 language: **MUST/MUST NOT** are blocking
violations, **SHOULD/SHOULD NOT** are advisory warnings.

- **Backend rules** — authentication, authorization, input validation, data protection, API security, logging, dependency management, error handling
- **Frontend rules** — XSS prevention, CSRF protection, secure storage, authentication handling, content security, dependency management

## Constants and Configuration

**Configurable values:** Any value that determines system behavior (expiry times,
retry counts, batch sizes, thresholds, rate limits) must be backed by an environment
variable.

**Fixed constants:** Values that are fixed by design and never change across
deployments (protocol versions, header names, media types) must be named constants,
not bare literals.

**Environment variables:**

- `.env.example` is the single source of truth for what the application expects
- Every variable must have a default fallback defined as a named constant
- Validate all required variables at startup — fail fast with a clear error
- Include the unit in time variable names (`_MS`, `_SECONDS`, `_MINUTES`)

**Organization:**

- Shared constants — values used across modules live in a centralized location
- Module-local constants — values used within a single module live in that module

## Markdown Standards

All `.md` files must pass `markdownlint-cli2` using the project config. Key rules:

- ATX-style headings only, incrementing by one
- Every fenced code block specifies a language
- Files start with a top-level heading
- Tables use compact style
- Ordered lists use sequential numbering
- Link fragments reference valid heading anchors

## Key Mindset Shifts

- A bug is just an unwritten scenario
- Scenarios are specs at a lower level of abstraction — same format, same discipline
- The spec absorbs knowledge that Jira buries in closed tickets
- No work begins without a spec or scenario to satisfy
- A pull request that changes behavior without updating the spec is incomplete
