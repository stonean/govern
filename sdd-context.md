# SDD Context Document
> This document captures a fully developed Spec Driven Development framework including
> conventions for bugs, scenarios, repository structure, brownfield adoption, and a
> blog series in progress. Use this as context for continuing the work.

---

## 1. The Framework

### Core Principle
The specification is the primary artifact and source of truth. Code is derived from
the spec, not the other way around. A spec describes *what the system does*, not
*how it does it*.

### What a Spec Is Not
- Not a PRD — specs are living documents that stay accurate after code is written
- Not Agile or Scrum — SDD is about what the source of truth is, not how teams organize work
- Not vibe coding — SDD is the discipline layer that makes AI coding reliable
- Not just using AI agents — agents are tools, SDD is what makes them consistent

---

### Repository Structure

```
specs/
  features/
    user-auth/
      spec.md             ← high level behavior, stable, core scenarios only
      scenarios/
        token-expiry.md
        invalid-credentials.md
    checkout/
      spec.md
      scenarios/
        rounding-error.md
    security/
      spec.md             ← cross-cutting concerns are first class specs
    data-integrity/
      spec.md
  triage.md               ← temporary inbox for brownfield adoption, should shrink to nothing
```

**Rules:**
- `spec.md` stays high level and stable — if scenarios are bloating it, they need their own files
- Cross-cutting concerns (security, data integrity) are first class feature specs, not a floating `invariants.md`
- Bugs do not get a top level directory — they belong to the feature they violate
- Status metadata lives inside files, not in folder structure (no `open/` or `resolved/` directories)

---

### Spec Format

```markdown
# Feature Name

## Overview
One or two sentences describing what this feature does at a high level.

## Behavior
- Concrete behavioral statements written in plain language
- Each statement describes what the system does in a specific situation
- No implementation detail

## Constraints
- Rules that must always hold
- Rate limits, validation rules, invariants specific to this feature
```

---

### Scenario Format
A scenario is a spec at a lower level of abstraction. Same format, same discipline,
just narrower in scope. Anchored to the spec it elaborates on via `spec-ref`.

```markdown
# Scenario: Token Expiry Enforced on Request

spec-ref: spec.md#token-lifecycle

## Context
A user has an active session token that has been inactive for 24 hours.

## Behavior
When the user makes a request with the expired token the system rejects
it with a 401 and clears the token from the session store.

## Edge Cases
- Token that expires mid-request completes the current request but
  rejects the next
- Clock skew of up to 30 seconds is tolerated before expiry is enforced
```

No Given/When/Then syntax required. Plain language is equally valid and often
more readable for complex scenarios.

---

### Bug Handling

**The core insight:** A bug is just an unwritten scenario. Most bugs exist because
a situation was never formally described. The fix is not a bug report — it is a
scenario added to the spec it belongs to.

**Decision tree — in order:**
1. Does a spec exist for this behavior? If not, write it first
2. Is the spec ambiguous or incomplete? Correct or enhance it
3. Is the spec clear but the implementation wrong? Add the missing scenario, then fix the code

**A bug file is almost never needed.** The scenario captures the correct behavior.
The git history on that file records when and why it was added. A descriptive commit
message covers the rest.

**A bug file is only justified when:**
- The root cause is complex enough that losing it would be costly
- Reproduction requires context that doesn't belong in the spec or scenario
- A workaround needs to be documented while a fix is deferred

**The rule:** A bug file should never be the first artifact created. The spec always comes first.

---

### Brownfield Adoption

- Do not frontfill bugs you aren't actively working on
- Write specs for areas you are actively touching — let adoption spread naturally
- Use `triage.md` as a temporary inbox for known issues not yet assigned to a feature spec
- As specs get written, items migrate from triage into their proper home
- The goal is for `triage.md` to eventually disappear
- SDD adoption in a brownfield project is incremental by feature area, not a big-bang effort

---

### Key Mindset Shifts
- A bug is just an unwritten scenario
- Scenarios are specs at a lower level of abstraction — same format, same discipline
- The spec absorbs knowledge that Jira buries in closed tickets
- No work begins without a spec or scenario to satisfy
- A pull request that changes behavior without updating the spec is incomplete

---

## 2. Blog Series

### Series Structure
Foundational posts read in order. Deep dives are standalone and can be read in any order.

| # | Title | Type | Status |
|---|-------|------|--------|
| 1 | What Spec Driven Development Actually Is | Foundation | Drafted |
| 2 | A Bug Is Just An Unwritten Scenario | Foundation | Drafted |
| 3 | How to Structure a Spec Repository | Deep dive | Outline only |
| 4 | Introducing SDD to a Brownfield Project | Deep dive | Outline only |
| 5 | The Anatomy of a Good Spec | Deep dive | Outline only |
| 6 | SDD and AI — Why Specs Make AI Coding Agents Actually Useful | Deep dive | Outline only |

**Target audience:** Developers, engineering leads, CTOs, architects, general tech audience
**Tone:** Thought leadership
**Length:** ~1000 words per post

---

### Post Outlines (not yet drafted)

**Post 3 — How to Structure a Spec Repository**
- Folder structure and the reasoning behind it
- What belongs in spec.md vs scenarios/
- Cross-cutting concerns as first class specs
- How the structure scales as the system grows
- Naming conventions and spec versioning

**Post 4 — Introducing SDD to a Brownfield Project**
- Why big-bang spec adoption fails
- The triage.md pattern as a temporary inbox
- Starting with the areas you're actively touching
- How to write a spec for code that already exists
- What done looks like — when triage.md disappears

**Post 5 — The Anatomy of a Good Spec**
- What makes a spec too vague vs too detailed
- The right level of abstraction for spec.md
- When to split a spec into multiple specs
- How scenarios extend the spec without bloating it
- Common mistakes and how to avoid them

**Post 6 — SDD and AI**
- Why vague prompts produce vague code
- How a well-structured spec repository becomes context for an AI agent
- Scenarios as executable validation gates
- The risk of AI-generated code without specs — architectural drift, security gaps
- SDD as the discipline layer that makes AI coding reliable

---

## 3. Drafted Posts

### Post 1: What Spec Driven Development Actually Is

Most software teams share a common dysfunction. Requirements live in Jira tickets.
Architecture decisions live in Confluence pages nobody reads. Business logic lives
in the heads of the two engineers who have been around long enough to remember why
things work the way they do. And the code — the only artifact anyone actually trusts
— drifts further from every other document with every passing sprint.

Spec Driven Development is a response to that dysfunction. But to understand what it
is, it helps to first be precise about what it is not.

**What SDD Is Not**

It is not a PRD. Product Requirements Documents describe intent from a business
perspective. They are written before development begins and rarely updated once work
starts. By the time a feature ships, the PRD typically bears little resemblance to
what was built. SDD specifications are living documents — they evolve with the system
and remain accurate after the code is written, not just before.

It is not Agile or Scrum. Agile is a philosophy about how teams collaborate and
iterate. Scrum is a framework for organizing that work into sprints, ceremonies, and
roles. Neither says anything meaningful about what artifacts teams should produce or
how those artifacts should relate to the code. SDD is not a replacement for how your
team organizes work — it is a discipline about what your source of truth is.

It is not vibe coding. Vibe coding — using AI to generate code from loose natural
language prompts — optimizes for speed of initial output at the expense of
consistency, maintainability, and predictability. It works for prototypes. It breaks
down for production systems where multiple people, and multiple AI agents, need to
work on the same codebase over time without introducing drift.

It is not just using AI coding agents. AI coding agents are tools. SDD is the
discipline that makes those tools reliable. An AI agent given a well-structured spec
produces consistent, predictable output. An AI agent given a vague prompt produces
code that may work today and silently break something tomorrow.

**What SDD Actually Is**

Spec Driven Development is a methodology in which the specification is the primary
artifact — the source of truth from which everything else is derived. Code is an
output of the spec, not the other way around.

This is a meaningful inversion. In most teams, the code is what's real. Documentation
is written to describe code that already exists, which means it is always slightly out
of date, always slightly wrong, and rarely trusted. In SDD, the spec is what's real.
Code that diverges from the spec is wrong by definition.

A spec is not a long document. It is a precise, readable description of what a feature
does — written at a level of abstraction that a developer, a product manager, and an
engineer three years from now can all understand. It describes behavior, not
implementation. It answers the question *what should the system do* without
prescribing *how the system should do it*.

A minimal spec looks like this:

```markdown
# User Authentication

## Overview
Users authenticate with an email address and password. Sessions persist
for 24 hours of activity before requiring re-authentication.

## Behavior
- A user who provides valid credentials receives a session token
- A user who provides invalid credentials receives a 401 with no
  indication of which field was wrong
- A session token that has been inactive for 24 hours is invalidated
  on the next request
- A user may explicitly sign out, immediately invalidating their token

## Constraints
- Passwords must be a minimum of 12 characters
- Authentication attempts are rate limited to 10 per minute per IP address
```

That is the whole spec. Not a hundred acceptance criteria. Not a sequence diagram.
Not a Jira epic with fourteen sub-tasks. A clear, version-controlled document that
anyone on the team can read in two minutes and trust completely.

**The Spec as Source of Truth**

The power of this approach compounds over time. When a new engineer joins the team,
they read the specs — not the tickets, not the wiki, not the code comments. When a
bug surfaces, the first question is whether the spec covers the scenario — not who
filed the ticket or which sprint it belongs to. When an AI agent is asked to implement
a feature or fix an issue, it is given the spec as context — not a vague description
typed into a chat window.

Specs live in version control alongside the code. Changes to behavior require changes
to the spec. A pull request that modifies behavior without updating the spec is
incomplete by definition. This is how the spec stays alive rather than rotting into
documentation debt.

**Where This Series Goes**

This post establishes the foundation. What follows builds on it.

The next post tackles the question every team hits immediately after adopting SDD:
where do bugs go? The answer reframes what a bug actually is — and leads to a workflow
that is leaner than anything a Jira-based process can offer.

Subsequent posts go deeper: how to structure a spec repository as a system grows, how
to introduce SDD to a codebase that already exists, what makes a spec genuinely good
versus superficially correct, and how a well-structured spec repository transforms
what AI coding agents can do.

The throughline across all of it is simple. The teams that build reliable software —
whether with human engineers, AI agents, or both — are the ones who agree on what the
system should do before they talk about how it works. Spec Driven Development is the
discipline that makes that agreement stick.

---

### Post 2: A Bug Is Just An Unwritten Scenario

Every team that adopts Spec Driven Development hits the same wall. The methodology
is compelling for new features — write the spec, derive the implementation, keep the
two in sync. But then a bug surfaces, and the process breaks down. Nobody agrees
where it goes. Does it get a ticket in Jira? A note in Slack? A comment in the code?
The spec-driven workflow, which felt so clean moments ago, has no obvious answer.

The reason is that most SDD material is written with greenfield features in mind.
Bugs are treated as an afterthought — a separate category managed by a separate tool.
But this is a false separation, and resolving it leads to a cleaner system than most
teams are running today.

**The Reframe: A Bug Is a Spec Violation**

The insight that changes everything is simple: a bug is not a standalone event. It
is evidence that the system's behavior diverges from what was specified. That means
every bug is one of two things — either the spec is wrong, or the implementation is
wrong. In both cases, the resolution flows back into the spec.

This reframe has a practical consequence. Before reaching for Jira, the first question
should always be: does a spec exist for this behavior? The answer determines everything
that follows.

- No spec exists — the bug is actually a gap. Write the spec first, describing the
  correct behavior. The implementation fix follows from that.
- The spec exists but is ambiguous — the bug is a spec deficiency. Correct or enhance
  the spec, then fix the implementation to match.
- The spec is clear and the implementation is wrong — add a scenario to the spec that
  captures the correct behavior explicitly, then fix the code to satisfy it.

In all three cases, the spec becomes more precise. This is the compounding value that
issue trackers never deliver: closed tickets bury knowledge, while an evolving spec
encodes it permanently.

**A Bug Is Just An Unwritten Scenario**

Following this decision tree to its conclusion leads to a more specific claim: most
bugs exist because a scenario was never specified. The system behaved unexpectedly in
a situation that was never formally described.

The fix, then, is not a bug report. It is a scenario — a concrete description of what
the system should do in that specific situation, added to the spec it belongs to.

```
specs/
  features/
    user-auth/
      spec.md
      scenarios/
        token-expiry.md
        invalid-credentials.md
    checkout/
      spec.md
      scenarios/
        rounding-error.md
```

`spec.md` stays high level and stable — the authoritative description of what a
feature does. Scenarios live alongside it, each one capturing a specific context,
behavior, and edge cases. The scenario file for a bug that was just fixed tells the
whole story: what situation triggered it, what the correct behavior is, and what edge
cases surround it.

A scenario follows the same format as a spec, just narrower in scope:

```markdown
# Scenario: Token Expiry Enforced on Request

spec-ref: spec.md#token-lifecycle

## Context
A user has an active session token that has been inactive for 24 hours.

## Behavior
When the user makes a request with the expired token, the system rejects
it with a 401 and clears the token from the session store.

## Edge Cases
- Token that expires mid-request completes the current request but
  rejects the next
- Clock skew of up to 30 seconds is tolerated before expiry is enforced
```

No Given/When/Then syntax required. No separate methodology to learn. A scenario is
a spec at a lower level of abstraction — same format, same discipline, same source
of truth.

**What This Means for Bug Files**

If the spec absorbs the knowledge, what is a bug file actually for? In most cases,
nothing. The scenario captures the correct behavior. The git history on that file
records when it was added and why. A descriptive commit message covers the rest.

A bug file only earns its place when the root cause or reproduction context is complex
enough that losing it would be costly — a subtle race condition, a dependency on a
third-party service, a class of inputs that are genuinely hard to reason about. Even
then, the bar should be high. If the information belongs in the spec or scenario,
that is where it should live.

**Replacing Jira**

This is where the approach becomes genuinely disruptive. Jira exists because teams
need somewhere to put work that isn't code. But if every bug either corrects a spec
or adds a scenario, the spec repository becomes the complete record of system behavior
and the problems that refined it. Nothing lives in a separate tool. Nothing gets
buried in a closed ticket.

The one concession for brownfield projects is a temporary `triage.md` at the root of
the spec tree — a flat inbox for known issues that haven't been assigned to a feature
spec yet. As specs get written for each area of the system, items migrate from triage
into their proper home. The goal is for that file to eventually disappear.

**The Discipline That Makes It Work**

None of this functions without one rule: no work begins without a spec or scenario to
satisfy. Not a bug fix, not a refactor, not a hotfix under pressure. The spec always
comes first.

This is the discipline that separates SDD from documentation theater. When the spec
is genuinely the source of truth — when the implementation is derived from it, and
bugs flow back into it — the system gets more precise over time rather than more
fragile. Every bug that surfaces makes the spec stronger. Every scenario added narrows
the space where the next bug can hide.

The industry is converging on spec-driven development as the workflow for the
AI-assisted era. But the conversation has been almost entirely about greenfield
features. The teams that figure out how to handle bugs with the same rigor — without
the overhead of a separate tool — will end up with something much more powerful: a
living specification that is also a complete history of everything the system learned.
