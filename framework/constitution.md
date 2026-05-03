# Constitution

The governing rules for spec-driven software development. This document defines the principles, workflow, and quality gates that apply to every project regardless of tech stack.

<!-- §principles -->

## Guiding Principles

These are evaluation criteria, not implementation instructions. Use them to identify gaps or violations, not to drive design decisions.

### Technology

- **Secure:** protect sensitive data through industry standards and best practices. See `specs/security-backend.md` and `specs/security-frontend.md` for enforceable rules.
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

<!-- §cost-levers -->

### Cost levers

Per-task token tracking and budget ceilings require a runtime `govern` does not have — that work belongs to the AI platform. `govern` contributes by offering cost-aware patterns the user can opt into. The current levers: the [Lightweight Track](#lightweight-track) skips the plan phase for small features; the optional `[simple]` marker on tasks signals to the agent (and the user) that a trivial task should be routed to a cheaper model per the adopter's platform mapping; the stuck-detection step in `/{project}:implement` catches runaway loops before they compound spend; default-off autonomy keeps the human in the loop unless `--auto` is explicitly passed. For runtime cost controls, point the adopter at the platform's tooling — Claude Code's `/cost`, the Anthropic usage dashboard, Cursor's request limits, and equivalents.

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

- **Backward via new questions** — `clarified` / `planned` / `in-progress` → `draft` when `/ask` records a new open question; the next `/clarify` resolves the question and the spec advances forward again. `draft` is the only status that tolerates open questions, so it is the destination; `/ask` performs the status mutation in the same write that records the question.
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

<!-- §rules -->

### Rules

A rule is an enforceable, citable requirement that applies across multiple features. Rules are the third artifact tier — alongside specs (feature-wide) and scenarios (situational), rules cover **cross-cutting** concerns the framework has opinions about regardless of which feature is being built (security, performance, concurrency, observability, accessibility, audit/compliance, data handling).

Rule files ship under `specs/{rule-set}.md` and are referenced from feature specs by ID. The canonical example is `specs/security-backend.md`, whose rules (e.g., `BE-AUTHN-001`) any spec touching authentication can cite. `/{project}:validate` enforces rules — it loads each rule file, runs each rule's Verification step against feature artifacts, and reports gaps.

#### Rule format (summary)

Every rule has four required fields:

- **ID** — a permanent identifier (e.g., `BE-AUTHN-001`) cited from feature specs.
- **Statement** — one sentence using RFC 2119 keywords (MUST, MUST NOT, SHOULD, SHOULD NOT). MUST/MUST NOT rules are blocking; SHOULD/SHOULD NOT are advisory.
- **Rationale** — the threat or risk the rule mitigates.
- **Verification** — instruction to the validate agent on how to check compliance against feature artifacts.

The full schema, ID stability invariants, category abbreviations, and Verification phrasing rules are canonically declared in `specs/008-security-rules/data-model.md`. New rule files follow the same schema.

#### When to write a rule

A new (or amended) rule is justified when **all four** of these hold:

1. **Cross-cutting** — the concern applies to multiple existing or anticipated features, not a single feature's domain.
2. **Citable** — the concern's verification can be expressed as a step a reviewer or `/{project}:validate` can check (a code-pattern check, a documentation-commitment check, or both).
3. **Governance-recognized category** — the concern belongs to a class the framework treats as foundational (security, performance, concurrency, observability, accessibility, audit/compliance, data handling, etc.) rather than feature-specific behavior.
4. **Generalizable wording** — the rule statement would make sense in any spec that touches the area, not only the spec that motivated it.

Indicators are evaluative, not mechanical. The same judgment discipline applies to rule promotion as to scenario promotion ([§scenario-promotion](#scenario-promotion)) — the framework provides the pattern; the user recognizes when promotion is warranted.

#### When a rule is not needed

- The concern is **situational** (specific condition, concrete behavior) → write a scenario under the affected spec.
- The concern is **feature-wide** (one feature, broad property) → add an acceptance criterion or section to that spec.
- An existing rule already covers the concern → cite the existing rule from the spec rather than creating a new one.

#### Lifecycle

- Rule IDs are permanent. Once assigned, an ID is never renumbered, even if the rule moves within the file or is edited.
- Rules are deprecated with a `**DEPRECATED in {version}:**` label and a removal target version, then removed only after the deprecation window has passed.
- New rule files are introduced via their own feature spec (the same way 008 introduced `security-backend.md` and `security-frontend.md`).

See `specs/008-security-rules/data-model.md` for the full ID-stability invariants and deprecation rules.

#### Three tiers, selected by scope

| Tier | Scope | Artifact |
| --- | --- | --- |
| **Rule** | Cross-cutting (applies across many features) | A rule file under `specs/{rule-set}.md`, cited by ID from the specs that depend on it |
| **Spec / acceptance criterion** | Feature-wide (one feature, broad property) | A section or AC in the feature's `spec.md` |
| **Scenario** | Situational (a specific condition with concrete behavior) | A file in the feature's `scenarios/` directory |

Bugs route to the tier that matches the *scope* of the missing or violated requirement (see [Bug Decision Tree](#bug-decision-tree) above). A perf bug that affects every API endpoint promotes to a rule; a perf bug specific to one feature becomes an acceptance criterion; a perf bug that only manifests under a specific concurrency condition becomes a scenario.

<!-- §brownfield-inbox -->

### Brownfield Inbox

For projects adopting `govern` incrementally, a `specs/inbox.md` file serves as a temporary inbox for known issues not yet assigned to a feature spec. Items are recorded with `/log` and groomed into their proper home with `/groom`.

Inbox rules:

- Do not frontfill bugs that are not being actively worked on
- Write specs for areas being actively touched — let adoption spread naturally
- As specs are written, `/groom` migrates items from the inbox into their proper home
- The goal is for `inbox.md` to eventually be empty and deleted

<!-- §brownfield-process -->

### Brownfield Process

Brownfield projects adopt `govern` incrementally. The `/capture` command initializes a skeleton spec from freeform user input — no pressure to be comprehensive. Start broad; decompose through scenarios over time.

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

`govern` treats every artifact — constitution, specs, plans, tasks, scenarios, rules — as plain markdown the agent can edit with `Edit`. This is load-bearing: the agent's write path stays simple, PRs review glanceably, merge conflicts stay rare and human-resolvable, and adopting `govern` requires no bootstrap tooling beyond the AI agent itself.

### Principles

- All `govern` artifacts are markdown by default. The agent reads and writes them with the same `Edit` flow used for code.
- Structured metadata lives in YAML frontmatter at the top of each markdown file; the document body remains markdown prose.
- Cross-artifact references use standard relative markdown links (`[label](../path.md)`), not wiki-links — this keeps PRs reviewable on GitHub and viewers like Quartz/Obsidian still resolve them.
- Source-of-truth artifacts are markdown. Structured derived views are regenerated from canonical sources and never become the canonical record.
- **Non-markdown derived views** (SQLite caches, JSON indexes, generated graph data, binary artifacts) MUST be gitignored and regenerated on demand by their consumers.
- **Markdown derived views** (e.g., a per-spec `code-locations.md`) MAY be committed when their diffs are valuable to humans — for PR review, onboarding, refactoring impact analysis, or session-resumption context for the agent. Adopters MAY gitignore a particular markdown derived view if they prefer; commit is permitted, not required.
- Exceptions to text-first source-of-truth require an explicit constitutional amendment with stated rationale.

### Frontmatter Schema

The frontmatter schema applies to **spec files** (`spec.md`, `spec-and-plan.md`) and **scenario files** (`scenarios/{slug}.md`). Other `govern` artifacts (`system.md`, `errors.md`, `events.md`, `inbox.md`, plan files, tasks files, rule files, README files) MAY include frontmatter when a specific consumer benefits, but are not required to.

#### Spec files

| Field | Required | Type | Allowed values | Description |
| --- | --- | --- | --- | --- |
| `status` | yes | string | `draft`, `clarified`, `planned`, `in-progress`, `done` | Spec lifecycle state |
| `dependencies` | yes | list of strings | spec slugs (e.g., `002-events`); empty list permitted | Specs this feature depends on |
| `tags` | no | list of strings | free-form; see starter vocabulary below | Cross-cutting categories used by graph-view consumers |

#### Scenario files

| Field | Required | Type | Allowed values | Description |
| --- | --- | --- | --- | --- |
| `spec-ref` | yes | string | parent spec ref, conventionally `"{NNN-feature-name} — {Section}"` (quoted because the value commonly contains an em-dash and slash) | Identifies the parent spec and section the scenario elaborates |
| `tags` | no | list of strings | free-form | Scenario-level cross-cutting tags |

#### Open-schema rule

Additional fields beyond those listed above are permitted and ignored by uninterested consumers. Examples adopters or future `govern` work might add: `owner`, `target_release`, `created_at`, `description`, `aliases`. The `spec-and-plan.md` template uses the open-schema rule to carry `track: lightweight` — a human-readable marker, not a pipeline-consumed field. Consumers MUST NOT error on the presence of unknown fields. `/gov:validate` reports unknown fields as informational findings (not errors).

### Validation Severity

`/gov:validate` checks frontmatter against this schema with the following severity:

- **Hard fail** — frontmatter block missing on a spec or scenario file; frontmatter YAML malformed; `status` missing or not in the allowed set; `dependencies` missing or not a list; `spec-ref` missing on a scenario.
- **Advisory** — `tags` missing or empty; existing checkbox/cross-reference checks.
- **Informational** — unknown fields present.

Hard fails block the validation pass. Advisory and informational findings are reported but do not block.

For non-frontmatter checks (spec integrity, artifact completeness, plan/task consistency, dependencies, security rules), `/gov:validate` adds a fourth tier — **Blocking** — between Hard fail and Advisory. Blocking findings are structural or content issues that must be fixed before the next pipeline gate fires (e.g., missing `plan.md` on a `planned` spec, an unknown rule ID referenced in a spec). Hard fail and Blocking both prevent pipeline advancement; the distinction is that Hard fail says "the spec file itself is malformed," while Blocking says "the artifact set is incomplete or inconsistent." See `framework/commands/validate.md` for the full per-check severity assignment.

### Starter Tag Vocabulary

Published as guidance, not enforcement. Adopters and future specs MAY introduce new tags as needed; `/gov:specify` surfaces existing tags from sibling specs as autocomplete to drive convergence by reuse rather than ceremony.

| Tag | Suggested use |
| --- | --- |
| `cli` | Specs about slash commands or command-line interactions |
| `commands` | Specs that introduce, rename, or significantly change slash commands |
| `bootstrap` | Specs about adopting `govern`, project scaffolding, or initialization |
| `process` | Specs about workflow, lifecycle, or pipeline behavior |
| `templates` | Specs about template files (spec, plan, scenario, project-readme, etc.) |
| `security` | Specs about security rules, authentication, authorization |
| `agent` | Specs about AI-agent behavior, capabilities, or coordination |
| `format` | Specs about artifact formats, schemas, or serialization conventions |
| `pipeline` | Specs about the spec → plan → tasks → implement flow |
| `migration` | Specs that convert existing artifacts to a new format or convention |
| `scenarios` | Specs about scenario semantics, scenario targeting, or scenario tooling |
| `brownfield` | Specs about brownfield adoption, capture, inbox grooming, or incremental spec growth |

<!-- §drift-prevention -->

## Drift Prevention

These principles keep facts consistent as the framework evolves. They apply both to `govern` itself and to projects that adopt it. Drift is a class of bug; preventing it is part of the framework's design, not an afterthought.

### Canonical sources

For every kind of fact described in multiple places, one location is authoritative. Other documents that describe the fact MUST reference the canonical source rather than restate it.

| Fact | Canonical source |
| --- | --- |
| Spec lifecycle states and back-edges | `framework/constitution.md` §spec-lifecycle |
| Pipeline command behavior | each command's source under `framework/commands/*.md` (or `framework/bootstrap/configure/{key}.md`) |
| Frontmatter schema for specs and scenarios | `framework/constitution.md` §text-first-artifacts |
| Validation severity tiers | `framework/constitution.md` §text-first-artifacts (Validation Severity subsection) |
| Workflow registry | `framework/workflows/registry.json` |
| Per-agent permission set | `framework/bootstrap/configure/{key}.md` |
| Constitution section anchors | `<!-- §<anchor> -->` markers in `framework/constitution.md` |
| Command frontmatter (description, argument-hint) | each command's own frontmatter block |
| Rules artifact tier definition | `framework/constitution.md` §rules |
| Rule file format and ID conventions | `specs/008-security-rules/data-model.md` |

When adding a new kind of fact that may be referenced from multiple documents, name its canonical source explicitly here.

### Cross-document references

When document B describes content authored in document A, B includes a back-link to A — relative markdown link, anchor reference (`§anchor`), or section name. Two consequences follow:

- Changing A includes auditing every back-link to A. The audit is structured wherever it can be machine-checked (anchor resolution, help-table descriptions, registry-frontmatter equivalence), and a manual sweep otherwise.
- Adding a fact that conceptually belongs in A but landing it in B is drift. Either move the fact to A and back-link, or extend A's scope explicitly.

### Template-rule alignment

Every blocking check in `/{project}:validate` has a corresponding scaffolding element in the template that produces a passing artifact by default. The contract runs in both directions:

- Adding a new blocking check requires a template update so a freshly-copied artifact passes the check without manual editing.
- Adding template structure requires a corresponding rule (validate check, constitution rule, or both). Sections that don't trace back to a rule are dead weight.

Templates and validate evolve together. A diff that touches one without the other is incomplete.

### Manifest discipline

When multiple commands distribute or reference the same set of files (e.g., `/govern` and `/{project}:init` both scaffold a project; `/{project}:configure` and the bootstrap install both apply permission sets), the file list lives in one place:

- Either as a shared section the commands include by reference, or
- As a registry both commands read.

Two commands that copy-paste the same manifest into their own bodies are guaranteed to drift over time. Consolidate or accept that drift is the rule, not the exception.

### Done specs are frozen archaeology

`done` specs reflect the world at merge time. The framework will continue to evolve; done specs will not be rewritten to match. Drift between a done spec's body and the current framework is expected — handle it with **signposts at the top of the spec**, not by rewriting the body. Plan and tasks files in done-spec directories follow the same rule.

A signpost names what changed and points readers at the current source of truth. It does not edit history.

<!-- §pipeline-boundaries -->

## Pipeline Boundaries

- Never implement without a spec
- Never plan without resolving open questions
- Never skip phases — each phase produces artifacts the next phase consumes
- Never transition a spec to the next status without explicit user approval — present the work done and wait for the user to confirm before updating the status field
- Specs and plans are living documents — update them when decisions change, but don't backtrack silently

<!-- §concurrent-features -->

### Concurrent Features

The session state file (`{cli-config-dir}/{project}-session.json`) holds a single target by design. The pipeline is serial within a feature, and concurrent work on independent features uses two independent sessions in two terminals — not multi-target session state. Isolation is provided by the platform layer: `git worktree` keeps the working trees separate, and AI-agent platforms typically expose isolation primitives (Claude Code's `isolation: "worktree"` agent parameter, Cursor's worktree integration, etc.). Reach for those rather than asking `govern` to track multiple targets at once.

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
