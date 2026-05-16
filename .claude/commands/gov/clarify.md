---
description: Resolve open questions and advance a spec from draft to clarified.
argument-hint: "[feature]"
---

# Clarify

Resolve open questions and advance a spec from `draft` to `clarified`, or resolve open questions in a targeted scenario.

## Purpose

Pipeline gate: `draft` → `clarified`. A spec cannot be planned until all open questions are resolved, edge cases documented, and acceptance criteria verified. When a scenario is targeted, resolves scenario-level open questions instead.

This command is the resolver, not the back-edge entry point. The `clarified` / `planned` / `in-progress` → `draft` back-edge is owned by `/gov:ask` (see §spec-lifecycle in the constitution and spec 014). The hot path here walks open questions on a `draft` spec and advances to `clarified`. A recovery branch handles hand-edited specs that arrive at `/gov:clarify` with a non-`draft` status and unresolved questions in the body — a state that should not occur via normal usage but can arise from manual frontmatter edits or migrations from other tools.

## Context

Use the session target from `.claude/gov-session.json`. If `$ARGUMENTS` is provided, use it to override the session target. If no session target is set and no arguments provided, stop and tell the user to run `/gov:target` first.

## Target File Detection

Read `.claude/gov-session.json`. If the session includes a `scenario` and `scenarioPath`, operate on the scenario file (see **Scenario-targeted clarify** below). Otherwise, operate on the feature spec (see **Feature-targeted clarify** below).

## Feature-targeted clarify

### Spec File Detection

Read `spec.md`. If it does not exist, stop and report: "Spec does not exist. Run `/gov:specify` first."

### Gate

Read the spec's frontmatter `status` field and count entries in the `## Open Questions` section (entries are top-level list items or `**Bold-prefix**`-style headings; treat the section as having zero entries when it is missing, empty, or contains only a placeholder line such as `*None — all resolved.*`). Branch on the pair `(status, open-question count)`:

| Status | Open questions? | Behavior |
| --- | --- | --- |
| `draft` | yes | Walk questions, then verify acceptance criteria, then advance to `clarified` (existing hot path) |
| `draft` | no | Verify acceptance criteria, then advance to `clarified` (existing hot path) |
| `clarified` / `planned` / `in-progress` | no | Stop with: "Spec is already `{status}`. Run `/gov:plan` to create the technical plan." for `clarified`, or "Run `/gov:implement` to continue implementation." for `planned` / `in-progress`. |
| `clarified` / `planned` / `in-progress` | yes | Run the **Recovery path** below. |
| `done` | (any) | Stop with: "Spec is `done`. Run `/gov:elaborate` to capture this as a scenario instead." Exit without mutation. |

The "already `{status}`" branch and the `done` branch never modify any file.

### Scope Boundaries

- Read only the target feature's spec file (frontmatter and body) and dependency spec frontmatter. For the Recovery path, also list (without reading) `plan.md`, `tasks.md`, `data-model.md`, and `specs/{feature}/scenarios/`. Do NOT read plan files, tasks, source code, test files, scenarios, or unrelated specs' bodies.
- Scenario-level open questions are not surfaced — spec-level and scenario-level questions are independent concerns.
- Do NOT begin planning or implementation work. This command resolves questions and verifies acceptance criteria only.
- Reference: §spec-requirements, §spec-lifecycle, §pipeline-boundaries, §text-first-artifacts (constitution loaded by `/gov:target` — do not re-read).

### Hot path: `draft` spec

Perform the clarify gate defined in `constitution.md` (§spec-requirements, §spec-lifecycle):

0. **Recompute dependencies (safety net).** Run `scripts/gen-spec-deps.sh --dry-run` against the target spec. If it reports a diff, run it for real to sync `dependencies:` from body inline links before evaluating dependency readiness. The pre-commit hook normally keeps this in sync; this step catches uncommitted body edits made between commits.

1. **Resolve open questions one at a time** — process each open question individually in sequence:
   1. Display the question with its full context.
   2. Propose an answer with rationale, or ask the user to decide.
   3. Wait for the user to review, discuss, refine, or approve the resolution.
   4. Only after the user confirms, move the question from `## Open Questions` to `## Resolved Questions` and proceed to the next one.
   5. If the user wants to skip a question, move to the next and revisit skipped questions at the end.
   6. If resolving one question invalidates or changes another, note the impact when presenting the affected question.
   - Do NOT present multiple questions at once. Do NOT batch resolutions.
   - Process only items in `## Open Questions`. Items already in `## Resolved Questions` are never re-walked.
2. **Enumerate edge cases** — for each behavior, identify what happens with empty inputs, missing data, duplicates, boundary values, and concurrent access.
3. **Confirm error scenarios** — verify every failure mode has a defined behavior (HTTP status, error code, message). Flag gaps.
4. **Verify acceptance criteria** — check each is concrete, testable, and unambiguous. Rewrite vague ones. Flag missing criteria.
5. **Check dependency readiness** — for each entry in this spec's frontmatter `dependencies` list, read that spec's frontmatter `status` field. Confirm each dependency is at `clarified` or later. Flag blockers.

6. **Cross-spec impact check** — list every sibling spec referenced by inline markdown link in the body (the union the dependency scan already computed). Ask: "Do any of these referenced specs need an update because of decisions made here?" If yes, the §cross-spec-impact rule applies — the change goes in the affected spec as a new acceptance criterion or scenario, with a back-link to this spec. This step is informational; it does not block the transition.

After the review:

- Update the spec body with resolved questions and any new edge cases or acceptance criteria.
- If questions remain that need user input, list them and keep `status` at `draft`.
- If all open questions are resolved, run the validation gate before proposing the status transition:
  - All open questions are resolved (none remain in the Open Questions section)
  - Acceptance criteria are concrete and testable — no empty placeholders
  - Dependencies are at `clarified` or later
  - The modified spec file passes `npx markdownlint-cli2`
- If any check fails, report the specific failures and do not propose the transition. The user fixes the issues and re-runs the command.
- If all checks pass, present a summary of changes and ask the user to approve the transition to `clarified`. Do not update the status until the user confirms.
- On confirmation, update the frontmatter `status` field from `draft` to `clarified`.
- Display the next step: "Run `/gov:plan` to create the technical plan."

### Recovery path: non-`draft` spec with open questions

Triggered only when the gate sees `(status ∈ {clarified, planned, in-progress}) && open-question count ≥ 1`. This state should not occur via normal usage — `/gov:ask` reverts a spec to `draft` whenever it records a new open question on a non-`draft` spec — but it can arise from a manual frontmatter edit or a spec migrated from another tool.

Before mutating anything, surface the inconsistency to the user:

1. **Display the inconsistency:**
   - Current `status` value.
   - Count and titles of entries in `## Open Questions`.
   - Existence and last-modified timestamp of `plan.md`, `tasks.md`, and `data-model.md` in the feature directory. Omit files that do not exist.
   - The list of files in `specs/{feature}/scenarios/` if that directory exists.
2. **Prompt the user:**
   > Spec is `{status}` but has {N} unresolved open questions in the body — this state usually arises from a manual frontmatter edit. Revert status to `draft` and walk the questions?
3. **Confirm** — update the frontmatter `status` field to `draft`, then run the **Hot path: `draft` spec** procedure above (including the dependency-readiness check; the post-revert walk runs the same checks as a normal `draft` clarify). On successful resolution, the spec advances back to `clarified`. Downstream artifacts (`plan.md`, `tasks.md`, `data-model.md`, scenario files) are not deleted or rewritten by this command.
4. **Decline** — exit without modifying any file. The spec retains its inconsistent state and open questions remain in `## Open Questions`. The next `/gov:clarify` invocation offers the same prompt — the system surfaces the inconsistency on every clarify attempt rather than silently advancing.

`## Resolved Questions` is never re-walked even on the recovery path; only items in `## Open Questions` are processed.

## Scenario-targeted clarify

### Scope Boundaries

- Read the targeted scenario file (frontmatter and body). May read the parent spec's frontmatter `status` field to decide which next-step suggestion to display. Do NOT read the parent spec's open questions or body, plan files, tasks, source code, test files, or unrelated specs.
- Do NOT begin planning or implementation work. This command resolves scenario-level questions only.
- Reference: §scenarios, §text-first-artifacts (constitution loaded by `/gov:target` — do not re-read).

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
- Run `npx markdownlint-cli2` on the modified file.
- Read the parent spec's frontmatter `status` field. Display: "Scenario clarification complete." and suggest `/gov:implement` if the parent spec is `planned` or `in-progress` (both states are accepted by `/gov:implement`'s gate). For other parent-spec states (`draft`, `clarified`, `done`), display the completion message without a next-step suggestion — the parent spec's own pipeline state determines what comes next.
