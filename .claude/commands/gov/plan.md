---
description: Create a technical plan and task breakdown for a clarified spec.
argument-hint: "[feature]"
parity:
  strict-fields:
    - status-transition
  semantic-fields:
    - plan-body
---

# Plan

Create a technical plan and task breakdown for a clarified spec.

## Purpose

Pipeline gate: clarified → planned. A spec cannot be implemented until it has a plan with technical decisions, affected files, and an ordered task list. This command produces both `plan.md` and `tasks.md`.

## Context

Use the session target from `.govern.session.toml`. If `$ARGUMENTS` is provided, use it to override the session target. If no session target is set and no arguments provided, stop and tell the user to run `/gov:target` first.

## Spec File Detection

Read `spec.md`. If it does not exist, stop and report: "Spec does not exist. Run `/gov:specify` first."

## Gate

Read the spec's `status` field from the YAML frontmatter at the top of the file. If `status` is not `clarified`, stop and report:

- `draft` → "Spec has unresolved open questions. Run `/gov:clarify` first."
- `planned` / `in-progress` → "Spec is already planned. Run `/gov:implement` to begin implementation."
- `done` → "Spec is `done`. Run `/gov:amend` to capture new work as a scenario."

## Scope Boundaries

- Read only files needed for planning: the target spec, `specs/system.md`, and cross-spec files per the markdown-only reference below. Do NOT read source code, test files, or unrelated specs beyond what the checklist requires.
- Do NOT begin implementation. This command produces `plan.md` and `tasks.md` only.
- Reference: §plan-phase, §tasks-phase, §readiness-check, §text-first-artifacts (constitution loaded by `/gov:target` — do not re-read).

## Instructions

> **For agent runtimes**: the Invoke steps below call the MCP tools of the optional gvrn runtime; the host-integration contract — bare↔prefixed tool names, lazy ToolSearch schema fetch, the no-shell-utilities rule, and the two-paths guarantee — lives once in the constitution, §runtime-host-integration. With no gvrn MCP server registered, walk the same prose using the host file-reading tools (Read, Edit, Write).

**Exec-path scope** (`gvrn exec plan`): steps 4–6 cross the boundary at the `writeSpecBody` extension point, but the task breakdown (step 7) and the substantive readiness checks (the **Validation gate** reference below) are spec-wide semantic host work with no extension marker, so the subprocess walker no-ops them by design — the runtime owns no primitive for the task breakdown or the criteria/consistency judgments. A host driving `gvrn exec` (and the markdown-only path) performs them itself before accepting the step-8 gate. `markdownlint` (steps 2, 10) is advisory on every path — it never blocks the clarified → planned transition. This scope reduction mirrors clarify's and is not a silent gap.

1. Invoke `read-spec` against the targeted feature to load the spec's frontmatter, sections, acceptance criteria, and open-question count. The result drives downstream prompts; the procedure refuses to proceed when the spec's status is not clarified.

2. Invoke `lint-markdown` against the feature directory's markdown files. Pre-plan violations are surfaced as advisory findings; the procedure continues regardless.

3. Invoke `create-plan-artifacts` against the targeted feature to copy the plan and tasks templates into the feature directory — pass `include-data-model` when the feature introduces or modifies domain entities or data structures. Missing artifacts are created (atomic, mode-preserving); pre-existing ones are never touched and report back `kept`. When any artifact is `kept`, run the **Detect existing artifacts** prompt below: on **keep** (the default), proceed with the files as they stand; on **replace**, re-invoke with `overwrite: true` to copy fresh templates over them.

4. <!-- llm:writeSpecBody --> Fill the Technical Decisions section of the plan. The host returns the markdown body for the section; the walker forwards the response through the context.

5. <!-- llm:writeSpecBody --> Fill the Affected Files section of the plan. The host returns a table listing files this feature creates or modifies, alongside an action and purpose for each row. The runtime write boundary used by `/gov:implement` is derived from git history; this section is a planning aid, not authoritative.

6. <!-- llm:writeSpecBody --> Fill the Trade-offs section of the plan. The host enumerates the considered-and-rejected alternatives plus known limitations.

<!-- audit:ignore-promotion -->
7. **Author the task breakdown.** Break the plan into discrete, ordered work items in `tasks.md`, following the **Create the task breakdown** reference below. Step 3 copied the `tasks.md` template; this step fills it. This is spec-wide semantic host work with no extension marker (see the exec-path scope note above) — the runtime provides no primitive for the breakdown itself, so it is authored the same way on the MCP and markdown-only paths.

8. Invoke `gate-confirm` with a prompt that presents a summary of the plan body and the task breakdown and asks the user to approve the transition from clarified to planned. On confirmation, continue to step 9; on denial, the walker exits cleanly without modifying the spec.

9. Invoke `set-status` to flip the spec frontmatter's status from clarified to planned; the primitive guards against a stale "from" value so concurrent edits surface as an operational error rather than a silent overwrite.

10. Invoke `lint-markdown` a second time. Any violations surface as advisory findings the user resolves before running `/gov:implement` — markdownlint is advisory on both paths, never a transition blocker.

## Markdown-only reference

The full plan-creation procedure (existing-artifact protection, cross-spec context checklist, plan section contents, task breakdown rules, readiness gate, and cross-spec impact check) is documented below for the markdown-only path. The numbered steps above invoke the mechanical primitives that automate the deterministic phases; the host applies the same procedure against the markdown-only path when the runtime is unavailable.

### Recompute dependencies (safety net)

Run `scripts/gen-spec-deps.sh --dry-run` (via the `run-generator` primitive; the generator walks every spec — there is no per-spec mode). If it reports a diff, the `dependencies:` frontmatter is stale from uncommitted body edits; surface that and recommend committing (the pre-commit hook syncs it) or running the generator manually, then evaluate cross-spec context against the current frontmatter. Do not run the generator for real from this command.

### Detect existing artifacts

Before generating any artifacts, check the feature directory for existing plan files. This protects work the user may have already invested — including plans that survived a `/gov:amend` back-edge cycle.

1. Check the feature directory for `plan.md`, `tasks.md`, and `data-model.md`.
2. If none of those files exist, skip this section and proceed to the cross-spec context checklist with the standard template-copy flow unchanged.
3. If any of those files exists, list each one that exists with its last-modified timestamp (stat the file for the mtime — `create-plan-artifacts` reports each pre-existing artifact as `kept` but carries no wall-clock data), then prompt: "Plan artifacts exist from a prior `/gov:plan` run. Keep them and run the readiness check, or replace with fresh templates?" The default is **keep**.
4. **Keep** — skip the template copy entirely. Do not overwrite or modify the existing artifacts during this step. Proceed to the cross-spec context checklist; in **Create the plan** and **Create the task breakdown**, skip the "copy template" steps and treat the existing files as the working artifacts. Then run the validation gate. Advance status to planned only if all readiness checks pass; on failure, report the specific failures and exit without advancing.
5. **Replace** — copy fresh templates over the existing files. The user is responsible for re-applying any kept content.

### Cross-spec context checklist

Before creating the plan, load only the cross-spec context this feature actually needs:

- **Always read:** `specs/system.md` — architecture patterns and shared conventions.
- **Read if the feature emits or consumes events:** `specs/events.md` — check for naming conflicts and reuse opportunities.
- **Read if the feature introduces error codes:** `specs/errors.md` — check code ranges and format conventions.
- **Read if the feature has dependencies:** the spec file (not plan or tasks) of each dependency listed in this spec's frontmatter `dependencies` field — confirm `status` and understand the contracts this feature builds on.
- **Read if the feature introduces or modifies domain entities or data structures:** `data-model.md` files from related specs — check for structural conflicts.
- **Do NOT read** plans, tasks, scenarios, or source code from other features.

### Create the plan

1. **If the user picked "keep" in the existing-artifact prompt above**, skip the template copy — `plan.md` is already on disk and is the working artifact. Otherwise (no prior artifacts, or "replace"), copy `specs/templates/plan.md` into the feature directory as `plan.md`.
2. Fill in (or, on the keep path, edit/extend the existing content):
   - **Technical Decisions**: each decision with rationale. Code snippets, function signatures, and package paths belong here.
   - **Affected Files**: a *planning aid* — list the files you expect to create or modify so reviewers can sanity-check scope.
   - **Data Model**: data structure definitions. Create `data-model.md` if the feature introduces or modifies domain entities or data structures.
   - **Trade-offs**: what was considered and rejected, known limitations.
3. Cross-validate against the files loaded in the checklist above:
   - Plan must not conflict with `specs/system.md`.
   - Data model must be consistent with related specs.
   - Event types must align with `specs/events.md`.

### Create the task breakdown

1. **If the user picked "keep" in the existing-artifact prompt above**, skip the template copy — `tasks.md` is already on disk and is the working artifact. Otherwise (no prior artifacts, or "replace"), copy `specs/templates/tasks.md` into the feature directory as `tasks.md`.
2. Break the plan into discrete, ordered work items:
   - Each task is small enough to complete and verify in a single session.
   - Each task has a clear "done when" condition.
   - Tasks respect dependency order.
   - Tasks are derived from the plan, not invented independently.

### Validation gate

Before proposing the status transition, run the readiness check. The substantive checks must pass — failures block the transition:

- Acceptance criteria are concrete and testable
- All open questions are resolved
- Data model exists if the feature introduces or modifies domain entities or data structures
- Plan does not conflict with `system.md` or other feature specs
- Data model is consistent with related specs
- Event types align with `events.md`
- Tasks are ordered and each has a clear definition of done

Markdownlint (`npx markdownlint-cli2` over the feature directory's `.md` files) runs as an **advisory** check on both paths — surface any violations for the user to resolve before `/gov:implement`, but do not block the transition on them (this matches runtime step 10).

If any substantive check fails, report the specific failures and do not propose the transition. The user fixes the issues and re-runs the command.

### Cross-spec impact check

After the plan is written and before finalizing, list every sibling spec referenced by inline markdown link in the spec or plan body. Ask: "Do any of these referenced specs need an update because of decisions made here?" If yes, the §cross-spec-impact rule applies — record the change in the affected spec as a new acceptance criterion or scenario, with a back-link to this spec. Informational; does not block.

### Finalize

1. Present a summary of the plan, task breakdown, and validation gate results. Ask the user to approve the transition to planned. Do not update the status until the user confirms.
2. On confirmation, update the spec's frontmatter `status` field from clarified to planned.
3. Display the next step: "Run `/gov:implement` to begin implementation."
