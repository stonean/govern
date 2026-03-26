# 011 — Brownfield Process

**Status:** in-progress
**Dependencies:** 006-bug-workflow, 007-govern-workflow

A formalized process for initializing and incrementally building out specs in brownfield projects. Unlike greenfield specs that aim for completeness upfront, brownfield specs start as skeletons — capturing what is known about an existing feature — and gain precision over time through real work: bug fixes, enhancements, and clarification.

## Problem

After a brownfield project adopts governance, the team faces existing features with no specs. The current framework assumes specs are written before code, but brownfield features already have code and no documentation. Reverse-engineering full acceptance criteria from existing code is impractical, and gathering accurate information from issue trackers and wikis is unreliable.

The current `/specify` command assumes a new feature is being defined. There is no path for initializing a spec that captures an existing feature's known behavior without pressure to be comprehensive.

## Capture Command

A new `/capture` command provides a lightweight entry point for brownfield spec initialization, separate from `/specify`. The user describes the feature in their own words — freeform, no guided checklist. The command drafts a skeleton spec from that input and presents it for review before writing.

`/capture` suggests starting broad. It is easier to decompose a broad feature into scenarios and eventually promote scenarios to their own specs than it is to combine over-partitioned specs back together.

### Behavior

- Accepts a freeform description from the user
- Drafts a skeleton spec using the standard `spec.md` template
- Populates with whatever behavior is known — sparse acceptance criteria are expected and valid
- Sets status to `draft`
- Sets the new feature as the session target
- Does not read existing code — the spec captures intended behavior as understood by the user, not implementation details from the codebase
- Does not create scenarios — the user runs `/scenario` separately to decompose
- Presents the draft for review before writing

### Post-capture

The command creates the spec and stops. It does not prescribe a next step. The post-capture message lists the user's options:

- Run `/scenario` to capture a bug or edge case
- Run `/clarify` to flesh out the spec
- Leave at `draft` and come back when real work arrives

The normal pipeline applies from that point. What happens next depends on why the user captured the feature.

### Edge cases

- Feature name conflicts with an existing spec directory — stop and report the conflict, suggest `/target` to work on the existing spec
- Description too vague to produce a meaningful skeleton — ask for more (at minimum: a name and one-sentence description)
- No acceptance criteria known at all — valid. The Acceptance Criteria section can be empty at `draft`. The first bug fix or enhancement adds the first criterion.

## Incremental Spec Growth

Every subsequent touch on the feature adds precision to the spec:

- **Bug fix** — the bug reveals missing behavior. The fix adds either:
  - An acceptance criterion to the spec (the behavior was never specified at a high level)
  - A scenario (the high-level behavior exists but a specific situation was not elaborated)
- **Enhancement** — new behavior is added to the spec before implementation, following the normal pipeline
- **Clarification** — an open question is resolved, narrowing ambiguity

Over time the spec converges on a complete description of the feature — not from a documentation effort, but as a side effect of doing work.

## Scenario Promotion

In brownfield projects, scenarios serve a dual purpose: they elaborate edge cases (as in greenfield) and they decompose broad features into distinct workflows.

When a scenario grows complex enough — multiple edge cases, its own open questions, distinct acceptance criteria that go beyond the parent spec's scope — it signals that the behavior warrants its own feature spec. The user runs `/specify` (for new behavior) or `/capture` (for another existing feature) to create the new spec. The original scenario is replaced with a dependency reference in the parent spec.

Indicators that a scenario should be promoted:

- The scenario has more than three edge cases
- The scenario's behavior section is longer than the parent spec's
- The scenario has open questions that are unrelated to the parent spec's domain
- Multiple scenarios in the same feature share overlapping concerns that would be better unified in their own spec

Promotion is a user decision, not automated. The framework provides the pattern; the user recognizes when decomposition is needed.

## Inbox

`specs/inbox.md` is the entry point for known issues in brownfield projects. When items are processed via `/inbox`, each item migrates to either:

- **Acceptance criteria** on an existing or new spec — when the item reveals a high-level behavior gap
- **A scenario** under an existing spec — when the item elaborates a specific situation within a known behavior

When an item does not map to any existing spec, `/inbox` tells the user to run `/capture` to initialize a spec first, then come back to process the item. The commands stay decoupled.

No inbox item remains as a standalone artifact. The spec or scenario is the permanent home. The goal is for `specs/inbox.md` to eventually be empty and deleted.

### Rename from triage

This spec renames `triage` to `inbox` throughout the framework:

- `specs/triage.md` → `specs/inbox.md`
- `templates/triage.md` → `templates/inbox.md`
- `/triage` command → `/inbox` command
- All references in constitution, sdd-context, README, and other commands

The term "inbox" describes the artifact's purpose (a temporary holding area) without implying a process methodology. Items should not stay there — the name communicates that naturally.

## Cross-Spec Impact

Specs are self-contained. When work on one spec identifies changes that affect another spec, those changes are recorded in the affected spec — not left as a note in the originating spec. The affected spec is the source of truth for its own behavior.

This applies when:

- A feature renames or supersedes an artifact from a prior spec (e.g., this spec renames `triage` to `inbox`, which was introduced by 006)
- Work on spec A reveals that spec B needs a new acceptance criterion or scenario
- A scenario in spec A exposes an edge case that belongs to spec B
- An implementation decision in spec A's plan creates a constraint for spec B

In each case:

- The change is recorded in the affected spec as a new acceptance criterion, scenario, or signpost note
- The signpost references the originating spec so the reader understands why the change was made
- If the affected spec is `done`, adding the change reopens it to `in-progress` per the normal lifecycle

The originating spec's acceptance criteria include delivering the cross-spec update. This ensures the change is tracked as part of the work that discovered it.

For this spec specifically: 006-bug-workflow gets a signpost noting that `triage` was renamed to `inbox` by 011-brownfield-process.

## Acceptance Criteria

- [ ] `/capture` command exists and creates a skeleton spec from freeform user input
- [ ] `/capture` uses the standard `spec.md` template
- [ ] `/capture` does not read existing code
- [ ] `/capture` does not create scenarios
- [ ] `/capture` sets the session target to the new feature
- [ ] `/capture` detects naming conflicts with existing spec directories
- [ ] Brownfield skeleton specs pass validation at `draft` status without requiring comprehensive acceptance criteria
- [ ] Bug fixes on a brownfield spec add either an acceptance criterion or a scenario
- [ ] Enhancements to a brownfield spec follow the normal pipeline (spec change before implementation)
- [ ] Inbox items migrate to acceptance criteria or scenarios — never remain standalone
- [ ] `/inbox` directs user to `/capture` when an item has no matching spec
- [ ] Scenario promotion pattern is documented in `constitution.md`
- [ ] `triage` is renamed to `inbox` across all governance artifacts (templates, commands, constitution, sdd-context, README)
- [ ] 006-bug-workflow spec includes a signpost noting the `triage` → `inbox` rename by this spec
- [ ] 007-govern-workflow spec includes a signpost noting the govern command gains a triage → inbox migration and `/capture` in the manifest by this spec
- [ ] Cross-spec impact pattern is documented in `constitution.md`
- [ ] The brownfield process is documented in `constitution.md` under brownfield adoption
- [ ] `sdd-context.md` is updated to reflect the brownfield process
- [ ] `README.md` brownfield section references the process

## Open Questions

*None — all resolved during clarification.*

## Resolved Questions

- **Visual indicator for brownfield specs:** No. A brownfield spec and a greenfield spec are the same artifact. The `draft` status already communicates incompleteness. The process converges to the same outcome regardless of origin.
- **Validate relaxation for brownfield:** No special treatment. Validate already scales checks to the spec's current status. A `draft` spec is not expected to have comprehensive criteria. Validate applies uniformly.
- **Automatic advancement to clarified:** No. Explicit user action, always. This is an existing pipeline boundary. The user runs `/clarify` when ready.
- **Automatic brownfield detection in `/specify`:** Superseded by the `/capture` decision. The user's choice of command is the signal — `/specify` for new features, `/capture` for existing ones.
- **Dedicated `/capture` command vs. modal `/specify`:** Yes, dedicated command. The workflows are different enough to justify separation. `/specify` asks qualifying questions and scaffolds comprehensively. `/capture` takes freeform input and creates a skeleton. Keeps each command's intent clear.
- **What `/capture` asks for:** Freeform. The user describes the feature in their own words. No guided checklist. The command suggests starting broad — decomposition happens through scenarios over time.
- **Code reading during capture:** No. The spec captures intended behavior as understood by the user. Existing code is referenced during `/implement` for task context, not during spec creation.
- **Template:** Standard `spec.md`. The output is indistinguishable from any other spec. The command provides brownfield framing; the artifact is the same.
- **Pipeline fit:** `/capture` creates the spec and stops. The normal pipeline applies from that point. No prescribed next step — depends on why the user captured it.
- **Interaction with inbox:** `/inbox` tells the user to run `/capture` when an item has no matching spec. Commands stay decoupled.
- **Scenario creation during capture:** No. `/capture` creates the spec only. The user runs `/scenario` separately.
