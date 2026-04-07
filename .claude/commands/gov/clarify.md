# Clarify

Resolve open questions and advance a spec from `draft` to `clarified`, or resolve open questions in a targeted scenario.

## Purpose

Pipeline gate: `draft` → `clarified`. A spec cannot be planned until all open questions are resolved, edge cases documented, and acceptance criteria verified. When a scenario is targeted, resolves scenario-level open questions instead.

## Context

Use the session target from `.claude/gov-session.json`. If `$ARGUMENTS` is provided, use it to override the session target. If no session target is set and no arguments provided, stop and tell the user to run `/gov:target` first.

## Target File Detection

Read `.claude/gov-session.json`. If the session includes a `scenario` and `scenarioPath`, operate on the scenario file (see **Scenario-targeted clarify** below). Otherwise, operate on the feature spec (see **Feature-targeted clarify** below).

## Feature-targeted clarify

### Spec File Detection

Check for `spec.md` first, then `spec-and-plan.md`. Use whichever exists. If neither exists, stop and report: "Spec does not exist. Run `/gov:specify` first."

### Gate

Read the spec status. If the status is not `draft`, stop and report:

- `clarified` or later → "Already clarified. Run `/gov:plan` to create the technical plan."

### Scope Boundaries

- Read only the target feature's spec file and dependency spec statuses. Do NOT read plan files, tasks, source code, test files, scenarios, or unrelated specs.
- Scenario-level open questions are not surfaced — spec-level and scenario-level questions are independent concerns.
- Do NOT begin planning or implementation work. This command resolves questions and verifies acceptance criteria only.
- Reference: §spec-requirements, §spec-lifecycle, §pipeline-boundaries (constitution loaded by `/gov:target` — do not re-read).

### Instructions

Perform the clarify gate defined in `constitution.md` (§spec-requirements, §spec-lifecycle):

1. **Resolve open questions one at a time** — process each open question individually in sequence:
   1. Display the question with its full context.
   2. Propose an answer with rationale, or ask the user to decide.
   3. Wait for the user to review, discuss, refine, or approve the resolution.
   4. Only after the user confirms, move the question to Resolved Questions and proceed to the next one.
   5. If the user wants to skip a question, move to the next and revisit skipped questions at the end.
   6. If resolving one question invalidates or changes another, note the impact when presenting the affected question.
   - Do NOT present multiple questions at once. Do NOT batch resolutions.
2. **Enumerate edge cases** — for each behavior, identify what happens with empty inputs, missing data, duplicates, boundary values, and concurrent access.
3. **Confirm error scenarios** — verify every failure mode has a defined behavior (HTTP status, error code, message). Flag gaps.
4. **Verify acceptance criteria** — check each is concrete, testable, and unambiguous. Rewrite vague ones. Flag missing criteria.
5. **Check dependency readiness** — confirm dependent specs are at `clarified` or later. Flag blockers.

After the review:

- Update the spec with resolved questions and any new edge cases or acceptance criteria.
- If questions remain that need user input, list them and keep status as `draft`.
- If all open questions are resolved, run the validation gate before proposing the status transition:
  - All open questions are resolved (none remain in the Open Questions section)
  - Acceptance criteria are concrete and testable — no empty placeholders
  - Dependencies are at `clarified` or later
  - The modified spec file passes `markdownlint-cli2`
- If any validation check fails, report the specific failures and do not propose the transition. The user fixes the issues and re-runs the command.
- If all checks pass, present a summary of changes and ask the user to approve the transition to `clarified`. Do not update the status until the user confirms.
- Display the next step: "Run `/gov:plan` to create the technical plan."

## Scenario-targeted clarify

### Scope Boundaries

- Read only the targeted scenario file. Do NOT read the parent spec's open questions, plan files, tasks, source code, test files, or unrelated specs.
- Do NOT begin planning or implementation work. This command resolves scenario-level questions only.
- Reference: §scenarios (constitution loaded by `/gov:target` — do not re-read).

### Instructions

1. **Resolve open questions one at a time** — process each open question in the scenario's `## Open Questions` section individually in sequence:
   1. Display the question with its full context.
   2. Propose an answer with rationale, or ask the user to decide.
   3. Wait for the user to review, discuss, refine, or approve the resolution.
   4. Only after the user confirms, move the question to Resolved Questions and proceed to the next one.
   5. If the user wants to skip a question, move to the next and revisit skipped questions at the end.
   - Do NOT present multiple questions at once. Do NOT batch resolutions.
2. **Enumerate edge cases** — identify edge cases specific to the scenario's behavior (empty inputs, missing data, boundary values, concurrent access).
3. **Verify behavior section** — confirm the scenario's Behavior section is unambiguous and complete.

After the review:

- Move resolved questions from `## Open Questions` to `## Resolved Questions` with their answers.
- Add any new edge cases to the scenario's `## Edge Cases` section.
- If questions remain that need user input, list them.
- The scenario does not have its own status field — resolution is complete when all open questions are removed from the Open Questions section.
- Run `markdownlint-cli2` on the modified file.
- Display: "Scenario clarification complete." and suggest `/gov:implement` if the parent spec is `in-progress`.
