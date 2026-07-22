---
description: Resolve open questions and advance a spec from draft to clarified.
argument-hint: "[feature]"
---

# Clarify

Resolve open questions and advance a spec from `draft` to `clarified`, or resolve open questions in a targeted scenario.

## Purpose

Pipeline gate: `draft` → `clarified`. A spec cannot be planned until all open questions are resolved, edge cases documented, and acceptance criteria verified. When a scenario is targeted, resolves scenario-level open questions instead.

This command is the resolver, not the back-edge entry point. The `clarified` / `planned` / `in-progress` → `draft` back-edge is owned by `/gov:amend` (see §spec-lifecycle in the constitution and spec 014). The hot path here walks open questions on a `draft` spec and advances to `clarified`. A recovery branch handles hand-edited specs that arrive at `/gov:clarify` with a non-`draft` status and unresolved questions in the body — a state that should not occur via normal usage but can arise from manual frontmatter edits or migrations from other tools.

## Context

Use the session target from `.govern/session.toml`. If `$ARGUMENTS` is provided, use it to override the session target. If no session target is set and no arguments provided, stop and tell the user to run `/gov:target` first.

## Target File Detection

Read `.govern/session.toml`. If the session includes a `scenario` and `scenario-path`, operate on the scenario file (the scenario-targeted branch of the Instructions below; detailed walk under **Scenario-targeted clarify** in the Markdown-only reference). Otherwise, operate on the feature spec.

## Gate

On a feature-targeted run, read the spec's frontmatter `status` field and count the `-` list-item entries in the `## Open Questions` section (a question is one `-` bullet — exactly the entries `read-spec`'s parser and `append-question`'s dedup count; treat the section as having zero entries when it is missing, empty, or contains only a placeholder line such as `*None — all resolved.*`). Branch on the pair `(status, open-question count)`:

| Status | Open questions? | Behavior |
| --- | --- | --- |
| `draft` | yes | Walk questions, then verify acceptance criteria, then advance to `clarified` (existing hot path) |
| `draft` | no | Verify acceptance criteria, then advance to `clarified` (existing hot path) |
| `clarified` / `planned` / `in-progress` | no | Stop with: "Spec is already `{status}`. Run `/gov:plan` to create the technical plan." for `clarified`, or "Run `/gov:implement` to continue implementation." for `planned` / `in-progress`. |
| `clarified` / `planned` / `in-progress` | yes | Run the **Recovery path** (see the Markdown-only reference below). |
| `done` | (any) | Stop with: "Spec is `done`. Run `/gov:amend` to capture this as a scenario instead." Exit without mutation. |

The "already `{status}`" branch and the `done` branch never modify any file.

## Scope Boundaries

Feature-targeted:

- Read only the target feature's spec file (frontmatter and body) and dependency spec frontmatter. For the Recovery path, also list (without reading) `plan.md`, `tasks.md`, `data-model.md`, and `specs/{feature}/scenarios/`. Do NOT read plan files, tasks, source code, test files, scenarios, or unrelated specs' bodies *speculatively* or to browse. **Grounding carve-out (§grounding):** when an open question is a factual question about existing reality — how current code behaves, what a schema or interface holds, what a dev database contains — you MAY read the specific source that settles it (and MUST cite it in the resolution), rather than resolve the question from conjecture. Read narrowly, only what answers the question.
- Scenario-level open questions are not surfaced — spec-level and scenario-level questions are independent concerns.
- Do NOT begin planning or implementation work. This command resolves questions and verifies acceptance criteria only.
- Reference: §grounding, §spec-requirements, §spec-lifecycle, §pipeline-boundaries, §text-first-artifacts (constitution loaded by `/gov:target` — do not re-read).

Scenario-targeted:

- Read the targeted scenario file (frontmatter and body). May read the parent spec's frontmatter `status` field to decide which next-step suggestion to display. Do NOT read the parent spec's open questions or body, plan files, tasks, source code, test files, or unrelated specs *speculatively* or to browse. The **Grounding carve-out (§grounding)** above applies here too: a scenario question about existing reality may be settled by reading the specific source that answers it, cited in the resolution.
- Do NOT begin planning or implementation work. This command resolves scenario-level questions only.
- Reference: §grounding, §scenarios, §text-first-artifacts (constitution loaded by `/gov:target` — do not re-read).

## Instructions

> **For agent runtimes**: the Invoke steps below call the MCP tools of the optional gvrn runtime; the host-integration contract — bare↔prefixed tool names, lazy ToolSearch schema fetch, the no-shell-utilities rule, and the two-paths guarantee — lives once in the constitution, §runtime-host-integration. With no gvrn MCP server registered, walk the same prose using the host file-reading tools (Read, Edit, Write) per the Markdown-only reference below.

Steps 1–11 are the feature-targeted walk; a scenario-targeted session runs steps 1, 6, and 12. The detailed walk — the question-resolution sub-procedure, the recovery prompt wording, and the scenario-targeted variant — lives under the Markdown-only reference below.

**Exec-path scope** (`gvrn exec clarify`): steps 7–8 are semantic host work with no extension marker, so the subprocess walker no-ops them by design — they cannot fold into the `askClarifyQuestion` round trip, which is one question per trip, because they are spec-wide passes that must run even when the question loop has nothing to walk (the zero-questions short-circuit in step 2). A host walking this command file directly (the MCP path) and the markdown-only path both perform steps 7–8 in full; a host driving exec performs them itself before accepting the step-10 gate. This scope reduction is deliberate and recorded in the spec 022 data-model's exec-path note — not a silent gap.

<!-- audit:ignore-promotion -->
1. Resolve the target from `.govern/session.toml`; `$ARGUMENTS` overrides the session target. If no session target is set and no arguments are provided, stop and tell the user to run `/gov:target` first. When the session includes a `scenario` and `scenario-path`, this is a **scenario-targeted** run: read the scenario file, run the question loop (step 6) against it, then wrap up at step 12 — steps 2–5 and 7–11 are feature-spec work and do not apply.

2. Invoke `read-spec` against the target feature (with `include-body`) and branch on the pair `(status, open-question count)` per the Gate table above — the result's frontmatter carries the status and its open-questions list carries the count (the Gate's entry-counting rule; placeholder lines are not entries):
   - Missing feature or `spec.md`: stop and report: "Spec does not exist. Run `/gov:specify` first."
   - `draft` with open questions: continue the full walk (steps 4–11).
   - `draft` with zero open questions: short-circuit — skip the question loop (step 6 runs no extension round trip) and continue at step 7 toward the status-advance gate.
   - `clarified` / `planned` / `in-progress` with zero open questions: stop with the "already `{status}`" message from the Gate table. No file is modified.
   - `clarified` / `planned` / `in-progress` with one or more open questions: take the **recovery branch** — display the inconsistency and prompt the user per the Recovery path reference below, then hand off to step 3 for the guarded revert.
   - `done` (any question count): stop with the `done` message from the Gate table. Exit without mutation.

3. **Recovery-branch revert** (only when step 2 took the recovery branch): on the user's confirmation, invoke `set-status` (from the current status, to `draft`) and continue the full walk (steps 4–11); on decline, exit without modifying any file.

4. **Recompute dependencies (safety net).** Invoke `run-generator` against the spec-dependency generator script (the Markdown-only reference names it) for the dry-run check. When it reports drift, the `dependencies:` frontmatter is stale relative to the body's inline links (uncommitted edits the pre-commit hook has not yet synced) — surface that and recommend committing (which runs the hook) or running the generator manually. Do **not** run the generator for real here: this command writes only the spec's questions/status and the session file, while the generator rewrites `dependencies:` across every spec. Evaluate dependency readiness against the current frontmatter, noting the caveat when drift was reported.

5. Invoke `traverse-deps` against the feature to check dependency readiness. Read each returned edge's `status` and require it to exist and be `clarified` or later — the clarify-time readiness rule (the same clarified-or-later threshold §blocked-by and the dashboard's blocked-by computation use). Do **not** gate on the result's top-level `compatible` flag: that flag encodes the stricter *planned*-or-later rule used at plan/implement time, so a dependency sitting at `clarified` reports `compatible: false` while still satisfying clarify's gate. Flag blockers — the validation gate (step 9) does not pass while a dependency is below `clarified`.

6. <!-- llm:askClarifyQuestion --> Resolve open questions **one at a time** — one extension round trip per open question, in sequence — following the question-resolution sub-procedure in the Markdown-only reference below (the per-question round trip, the no-batching rule, skip-and-revisit handling, and the `## Open Questions` → `## Resolved Questions` movement; items already in `## Resolved Questions` are never re-walked). Spec-body edits applying each answer remain LLM work on both paths — no primitive writes prose.

<!-- audit:ignore-promotion -->
7. **Enumerate edge cases and confirm error scenarios** — for each behavior, identify what happens with empty inputs, missing data, duplicates, boundary values, and concurrent access; verify every failure mode has a defined behavior (HTTP status, error code, message) and flag gaps. Update the spec body with the resolved questions and any new edge cases or acceptance criteria.

<!-- audit:ignore-promotion -->
8. **Verify acceptance criteria and cross-spec impact** — check each criterion is concrete, testable, and unambiguous; rewrite vague ones; flag missing criteria. Then list every sibling spec referenced by inline markdown link in the body (the union the dependency scan already computed) and ask: "Do any of these referenced specs need an update because of decisions made here?" If yes, the §cross-spec-impact rule applies — the change goes in the affected spec as a new acceptance criterion or scenario, with a back-link to this spec. This check is informational; it does not block the transition.

9. Run the **validation gate** before proposing the status transition — every check must pass: all open questions are resolved (none remain in the Open Questions section — if questions remain that need user input, list them and keep `status` at `draft`); acceptance criteria are concrete and testable with no empty placeholders; dependencies are at `clarified` or later (step 5); and invoke `lint-markdown` against the modified spec file, requiring a clean result. If any check fails, report the specific failures and do not propose the transition — the user fixes the issues and re-runs the command.

10. Invoke `gate-confirm` with a prompt that presents a summary of the changes and the resolved questions and asks the user to approve the transition from `draft` to `clarified`. On confirmation, continue to step 11; on denial, the walker exits cleanly without modifying the spec.

11. Invoke `set-status` to flip the spec frontmatter's status from `draft` to `clarified`; the primitive guards against a stale "from" value so concurrent edits surface as an operational error rather than a silent overwrite. Then display the next step: "Run `/gov:plan` to create the technical plan."

12. **Scenario-targeted wrap-up** (scenario-targeted runs only): after the question loop, enumerate edge cases specific to the scenario's behavior (empty inputs, missing data, boundary values, concurrent access) and add them to the scenario's `## Edge Cases` section; confirm the scenario's Behavior section is unambiguous and complete; if questions remain that need user input, list them. The scenario has no status field — resolution is complete when all open questions are removed from the Open Questions section. Invoke `lint-markdown` against the modified scenario file. Read the parent spec's frontmatter `status` field (a host read — this step already dispatches `lint-markdown`, so it does not also invoke read-spec), display "Scenario clarification complete.", and suggest `/gov:implement` if the parent spec is `planned` or `in-progress` (both states are accepted by `/gov:implement`'s gate); for other parent-spec states (`draft`, `clarified`, `done`), display the completion message without a next-step suggestion — the parent spec's own pipeline state determines what comes next.

## Markdown-only reference

With no gvrn runtime registered, the host walks the same contract with its own file tools (Read, Edit, Write) — no shell-pipeline substitution (§runtime-host-integration). The Gate table above governs both paths.

### Feature-targeted clarify (hot path: `draft` spec)

Read `spec.md`. If it does not exist, stop and report: "Spec does not exist. Run `/gov:specify` first." Then perform the clarify gate defined in `constitution.md` (§spec-requirements, §spec-lifecycle):

0. **Recompute dependencies (safety net).** Run `.govern/scripts/gen-spec-deps.sh --dry-run` (it walks every spec — there is no per-spec mode). If it reports a diff, the `dependencies:` frontmatter is stale from uncommitted body edits; surface that and recommend committing (the pre-commit hook syncs it) or running the generator manually. Do not run it for real from this command — evaluate dependency readiness against the current frontmatter and note the caveat.

1. **Resolve open questions one at a time** — process each open question individually in sequence:
   1. Display the question with its full context.
   2. Propose an answer with rationale, or ask the user to decide. **When the question turns on existing reality — how current code behaves, what a schema, interface, or dev database holds — consult the specific source that settles it and ground the proposed answer in what you found, citing it (`path:line`, the query, or the command). Do not resolve a factual question about existing code by conjecture; when no reachable source can settle it, say so and resolve it as an explicit assumption or leave it open (§grounding).**
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

Triggered only when the gate sees `(status ∈ {clarified, planned, in-progress}) && open-question count ≥ 1`. This state should not occur via normal usage — `/gov:amend` reverts a spec to `draft` whenever it records a new open question on a non-`draft` spec — but it can arise from a manual frontmatter edit or a spec migrated from another tool.

Before mutating anything, surface the inconsistency to the user:

1. **Display the inconsistency:**
   - Current `status` value.
   - Count and titles of entries in `## Open Questions`.
   - Existence and last-modified timestamp of `plan.md`, `tasks.md`, and `data-model.md` in the feature directory. Omit files that do not exist.
   - The list of files in `specs/{feature}/scenarios/` if that directory exists.
2. **Prompt the user:**
   > Spec is `{status}` but has {N} unresolved open questions in the body — this state usually arises from a manual frontmatter edit. Revert status to `draft` and walk the questions?
3. **Confirm** — update the frontmatter `status` field to `draft` (the `set-status` primitive on the runtime path; a direct frontmatter edit otherwise), then run the **Hot path: `draft` spec** procedure above (including the dependency-readiness check; the post-revert walk runs the same checks as a normal `draft` clarify). On successful resolution, the spec advances back to `clarified`. Downstream artifacts (`plan.md`, `tasks.md`, `data-model.md`, scenario files) are not deleted or rewritten by this command.
4. **Decline** — exit without modifying any file. The spec retains its inconsistent state and open questions remain in `## Open Questions`. The next `/gov:clarify` invocation offers the same prompt — the system surfaces the inconsistency on every clarify attempt rather than silently advancing.

`## Resolved Questions` is never re-walked even on the recovery path; only items in `## Open Questions` are processed.

### Scenario-targeted clarify

1. **Resolve open questions one at a time** — process each open question in the scenario's `## Open Questions` section individually in sequence:
   1. Display the question with its full context.
   2. Propose an answer with rationale, or ask the user to decide. **When the question turns on existing reality — how current code behaves, what a schema, interface, or dev database holds — consult the specific source that settles it and ground the proposed answer in what you found, citing it (`path:line`, the query, or the command). Do not resolve a factual question about existing code by conjecture; when no reachable source can settle it, say so and resolve it as an explicit assumption or leave it open (§grounding).**
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
