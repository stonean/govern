# 006 — Bug Workflow

**Status:** done
**Dependencies:** none

Bugs are unwritten scenarios. Rather than tracking defects in a separate system, the governance framework treats every bug as evidence that a spec is missing, ambiguous, or violated. This feature adds scenario support, a bug decision tree, and brownfield triage to the governance pipeline.

Most projects adopting governance are not greenfield — they have existing code, existing bugs, and incomplete specifications. Scenarios are the primary mechanism for incrementally bringing brownfield projects under governance. Every bug fix, edge case discovery, or behavior clarification produces a scenario that makes the specs more precise over time.

## Bug Decision Tree

When a bug is reported, the following decision tree determines the response — in order:

1. **No spec exists for the behavior** — the bug is a gap. Write the spec first, then fix the code.
2. **Spec exists but is ambiguous or incomplete** — the bug is a spec deficiency. Correct or enhance the spec, then fix the implementation.
3. **Spec is clear but implementation is wrong** — add a scenario capturing the correct behavior, then fix the code.

In all three cases, the spec becomes more precise. The scenario or spec update is the primary artifact, not a bug report.

## Scenarios as First-Class Artifacts

A scenario is a spec at a lower level of abstraction — same format, same discipline, narrower scope. Scenarios live in a `scenarios/` subdirectory alongside the spec they elaborate.

### Repository structure

```text
specs/
  {NNN-feature}/
    spec.md
    scenarios/
      token-expiry.md
      rounding-error.md
```

### Scenario format

Each scenario file follows a consistent structure:

- **spec-ref** — a reference to the parent spec and section the scenario elaborates
- **Context** — the specific situation or precondition
- **Behavior** — what the system does in that situation
- **Edge Cases** — boundary conditions and exceptions (optional)

Scenarios use plain language. Given/When/Then syntax is not required.

### Scenario lifecycle

Scenarios do not have their own status field. A scenario is either written (merged) or not. When `/gov:scenario` creates a scenario file, it also appends a task to the parent spec's `tasks.md` referencing the scenario. The task carries the completion status — the scenario itself is a permanent requirement document.

- The parent spec's status remains `in-progress` while tasks are being worked
- The task in `tasks.md` shows what is being worked on and links to the scenario
- When the task is complete, the scenario stays as documentation of the expected behavior
- If a scenario becomes obsolete, it is deleted — not marked with a status

### When to create a scenario

- A bug surfaces that the spec covers at a high level but does not describe in sufficient detail
- An edge case is discovered during implementation or review
- A spec section is growing too large and needs to be decomposed

### When a scenario is not needed

- The spec itself was missing or ambiguous — fix the spec directly
- The behavior is already captured by an existing scenario — update the existing file

## Bug Files

A dedicated bug file is rarely needed. The scenario captures the correct behavior, and git history records when and why it was added.

A bug file is only justified when:

- The root cause is complex enough that losing it would be costly
- Reproduction requires context that does not belong in the spec or scenario
- A workaround must be documented while a fix is deferred

The rule: a bug file should never be the first artifact created. The spec or scenario always comes first.

## Brownfield Triage

For projects adopting governance incrementally, a `specs/triage.md` file serves as a temporary inbox for known issues not yet assigned to a feature spec.

### Triage rules

- Do not frontfill bugs that are not being actively worked on
- Write specs for areas being actively touched — let adoption spread naturally
- As specs are written for each feature area, items migrate from triage into their proper home (spec updates or new scenarios)
- The goal is for `triage.md` to eventually be empty and deleted

## Governance Artifacts

This feature produces the following changes to the governance framework:

- **New template:** `templates/scenario.md` — starter file for scenario documents
- **New template:** `templates/triage.md` — temporary inbox format for brownfield adoption
- **Updated template:** `templates/spec.md` — reference to scenarios directory convention
- **Updated document:** `constitution.md` — bug handling section with decision tree and scenario lifecycle
- **New command:** `/gov:scenario` — standalone command that requires an active session target (set via `/gov:target`), confirms the target is correct, walks the decision tree, creates scenario files in the correct feature's `scenarios/` directory, and appends a linked task to the parent spec's `tasks.md`
- **New command:** `/gov:triage` — reviews `specs/triage.md`, walks each item through the decision tree, migrates items to the appropriate spec or scenario, and removes resolved items from triage
- **Updated command:** `/gov:about` — documents `/gov:scenario`, `/gov:triage`, scenario conventions, and bug workflow
- **Updated command:** `/gov:status` — displays scenario counts per spec in the pipeline dashboard
- **Updated command:** `/gov:next` — suggests `/gov:scenario` as a next action when appropriate (e.g., bug reported, spec is `in-progress`)
- **Updated command:** `/gov:validate` — checks that scenario-linked tasks are complete during validation
- **Updated document:** `README.md` — documents bug workflow and scenario convention

## Acceptance Criteria

- [ ] `templates/scenario.md` exists with spec-ref, Context, Behavior, and Edge Cases sections
- [ ] `templates/triage.md` exists with a flat inbox format and migration rules
- [ ] `templates/spec.md` references the scenarios directory convention
- [ ] `constitution.md` includes a bug handling section with the decision tree
- [ ] `constitution.md` defines scenarios as part of the spec lifecycle
- [ ] `constitution.md` documents the scenario directory convention in the spec phase file structure
- [ ] `/gov:triage` command exists and walks each triage item through the decision tree
- [ ] `/gov:triage` migrates resolved items from `specs/triage.md` to the appropriate spec or scenario
- [ ] `/gov:triage` removes migrated items from `specs/triage.md`
- [ ] `/gov:about` documents `/gov:scenario`, `/gov:triage`, scenario conventions, and the bug workflow
- [ ] `/gov:scenario` command exists and creates scenario files under the correct feature's `scenarios/` directory
- [ ] `/gov:scenario` requires an active session target and confirms the target before proceeding
- [ ] `/gov:scenario` follows the decision tree — checks for existing spec before creating a scenario
- [ ] `/gov:scenario` appends a task to the parent spec's `tasks.md` referencing the new scenario
- [ ] `/gov:status` displays scenario counts per spec in the pipeline dashboard
- [ ] `/gov:next` suggests `/gov:scenario` as a next action when context warrants it
- [ ] `/gov:validate` checks that scenario-linked tasks in `tasks.md` are complete
- [ ] `README.md` documents the bug workflow and scenario conventions
- [ ] All new and modified markdown files pass `markdownlint-cli2`

## Edge Cases

- **No session target set** — `/gov:scenario` stops and tells the user to run `/gov:target` first
- **Session target points to a spec that has no `tasks.md`** — `/gov:scenario` creates `tasks.md` before appending the task
- **Scenario file already exists with the same name** — `/gov:scenario` stops and reports the conflict; user must choose a different name or update the existing scenario
- **Parent spec is `done`** — `/gov:scenario` still allows creating a scenario (a bug can surface after completion); the spec status reverts to `in-progress`
- **Triage item matches an existing spec** — migration path: move the item into a scenario under the matching spec and remove it from `triage.md`
- **Bug spans multiple specs** — create a scenario under the most relevant spec; reference the other spec(s) in the scenario's spec-ref field
- **No spec exists for the bug** — decision tree step 1: create the spec first via `/gov:specify`, then create the scenario
- **`specs/triage.md` does not exist** — `/gov:triage` stops and reports nothing to triage
- **`specs/triage.md` is empty** — `/gov:triage` reports triage is clean; the file is kept to preserve git history

## Open Questions

<!-- All resolved. -->
