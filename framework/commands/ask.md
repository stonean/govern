---
description: Add a question or a scenario to the targeted spec (classifier-driven).
argument-hint: "[input text]"
---

# Ask

Add input to the targeted spec or scenario. `/ask` classifies the input as either a **question** (an unresolved decision recorded under `## Open Questions`) or a **scenario** (a concrete behavior captured under `scenarios/{slug}.md`), routes through the matching path, and on the spec target performs whichever back-edge keeps the lifecycle invariant.

## Purpose

Captures additions to a spec that arise at any point in the pipeline — during review, planning, implementation, or just thinking. `/ask` is the single verb for "I have a thing to add to this spec." The framework classifies the input and routes it; the user approves the classification (or flips it) at the same approval gate that already exists for the refined wording.

Two back-edges keep the spec lifecycle honest, both owned by `/ask`:

- **Question route — `clarified` / `planned` / `in-progress` → `draft`.** Recording a new open question on a non-`draft` spec leaves the spec in an internally inconsistent state ("status says questions resolved, body has unresolved questions"); the same write reverts status to `draft`. The user's acceptance of the refined question at the approval gate is the consent for the mutation; no separate prompt fires.
- **Scenario route — `done` → `in-progress`.** Recording a scenario on a `done` spec reopens it via the documented reopen cycle (§spec-lifecycle). The scenario's task is implemented, the spec returns to `done`.

## Context

Use the session target from `{cli-config-dir}/{project}-session.json`. If `$ARGUMENTS` is provided, use it as the initial input text. If no session target is set and no arguments provided, stop and tell the user to run `/{project}:target` first.

## Target File Detection

Read `{cli-config-dir}/{project}-session.json`. If the session includes a `scenario` and `scenarioPath`, the target artifact is the scenario file and the input is always treated as a question (scenarios do not nest under scenarios; the classifier is bypassed). Otherwise, the target artifact is the feature's `spec.md`. If that file does not exist, stop and report: "Spec does not exist. Run `/{project}:specify` first."

## Scope Boundaries

- This command reads the target artifact, appends to its `## Open Questions` section or writes a new `scenarios/{slug}.md` file and appends a linked task to `tasks.md`, and — when a back-edge applies — updates the spec's frontmatter `status` field. No other artifact contents are modified. Plan files and source code are never read or written.
- Spec `status` is read from the YAML frontmatter at the top of the file. It is mutated by this command only on a back-edge (clarified+ → draft or done → in-progress).
- For the impact display, this command may read sibling specs' frontmatter (only) under `specs/` to detect dependents. It does not read sibling spec bodies.
- Reference: §spec-requirements, §spec-lifecycle, §scenarios, §text-first-artifacts, §bug-handling (constitution loaded by `/{project}:target` — do not re-read).

## Instructions

> **For agent runtimes**: backticked primitive names in this section map to MCP tools the optional [gvrn runtime](https://crates.io/crates/gvrn) exposes under bare `<primitive>` names (e.g., `set-status`). Hosts wrap them with a server-name prefix taken from `.mcp.json` (Claude: `mcp__gvrn__set-status`; Auggie: `mcp:gvrn:set-status`). When the server is registered for your session, **call the corresponding tool** for each primitive referenced below — that is the deterministic path. If your host loads MCP tool schemas lazily (e.g., Claude Code lists tool names in a deferred-tool system reminder before exposing their schemas), the runtime is still registered: fetch the schema via the host's mechanism (`ToolSearch` on Claude Code) and call the tool — do not bail to the markdown-only fallback. When no `gvrn` MCP server is configured, walk the prose using the host's file-reading tool (e.g., `Read`) to produce the same result; do **not** substitute shell utilities (`awk`, `sed`, `grep` pipelines, `for` loops over files) for the prescribed file reads. The two paths share a contract; neither one wraps the other.

### Confirm target

1. Read `{cli-config-dir}/{project}-session.json` to get the session target's feature and optional scenario.
2. Read the target artifact (scenario file if targeted, otherwise `spec.md`).
3. **Recompute dependencies (safety net).** If the target is a spec, run `scripts/gen-spec-deps.sh --dry-run` against it. If it reports a diff, run it for real to sync `dependencies:` from body inline links. The pre-commit hook normally keeps this in sync; this step catches uncommitted body edits. (Skip on scenario targets — scenarios have no `dependencies` field.)
4. If the target is a spec, read its frontmatter `status` field now — the value is needed for the gate, the impact display, the classifier's status tiebreaker, and the post-record mutation.
5. Display the feature name, scenario name (if targeted), status, and a brief summary of what the artifact covers.

### Gather the input

If `$ARGUMENTS` is provided, use it as the initial input. Otherwise, ask the user: "What do you want to add to this spec?"

When a scenario is the target artifact, skip the classifier (next section) — scenarios accept questions only, not nested scenarios. Continue directly to **Refine the input (question route)**.

### Classify the input

Apply the heuristic to route the input. The classification is provisional — the user can flip it at the approval gate.

**Question signals (route → question):**

- The input ends with `?`.
- The input starts with an interrogative: `how`, `what`, `when`, `should`, `could`, `would`, `is`, `are`, `do`, `does`, `can`, `which`, `why`, `who`.
- The input contains hedge words: `maybe`, `perhaps`, `not sure`, `unclear`, `unsure`.

**Scenario signals (route → scenario):**

- Declarative or imperative phrasing: `when X happens, Y`; `X must Y`; `X should do Y` (without `?`).
- Concrete event/state language: `on`, `when`, `if`, `after`, `during`, `before`.
- No terminal `?`; no interrogative starter.

**Status tiebreaker:** when signals are mixed or absent on a **`done` spec**, default to scenario (the back-edge from `done` is owned by the scenario path; the question path refuses on `done`). When signals are mixed on any other status, default to question.

### Refine the input (question route)

The goal is a question that is precise, actionable, and self-contained — someone reading it during `/{project}:clarify` should understand exactly what needs to be decided without extra context.

1. **Understand intent** — read the target artifact to understand how the question relates to its behaviors, contracts, acceptance criteria, or open areas. If the question's connection to the artifact is unclear, ask the user to explain how it applies.
2. **Draft a refined version** — rewrite the question so it is specific to the spec's domain and terminology, identifies which behavior or criterion it affects, states what decision or information is needed, and stands alone.
3. **Check for duplicates** — compare against entries already in the target artifact's `## Open Questions` section. Use a normalized-whitespace comparison (collapse runs of whitespace, trim, case-insensitive). If the refined form matches an existing entry, report: "An equivalent question is already recorded: '{existing entry}'. Skip or refine further?" On skip, exit without recording; on refine further, incorporate feedback and loop.

### Refine the input (scenario route)

The goal is a scenario that captures a specific situation and the concrete behavior it triggers. Scenarios live at a lower level of abstraction than the parent spec — narrower scope, plain language.

1. **Walk the bug decision tree** (§bug-handling):
   - **Does a spec exist for the behavior?** If no, stop. Tell the user to create the spec first via `/{project}:specify`, then come back. (`/ask` requires a session target with a real spec file.)
   - **Is the spec ambiguous or incomplete?** If yes — the right fix is to update the spec directly, not record a scenario. Offer to help edit the spec; exit without recording.
   - **Is the spec clear but the behavior needs lower-level elaboration?** Proceed to draft the scenario.
2. **Derive a slug** — lowercase, hyphenated, no whitespace, no punctuation beyond hyphens. Check `specs/{feature}/scenarios/` for slug conflicts; if a file with that slug exists, ask the user for a different name.
3. **Identify the parent-spec section** — the `section:` frontmatter value names the spec section the scenario elaborates. Read the spec's body to pick an appropriate section, or ask the user.
4. **Draft Context, Behavior, and (optional) Edge Cases** for the scenario — plain language; Given/When/Then syntax is not required.

### Approval gate (both routes)

Show the user:

```text
Recording as [question|scenario] — preview drafted at [`## Open Questions` entry | `scenarios/{slug}.md`].

{preview of the refined content}

Accept this form, refine further, or `flip` to switch route?
```

- **Accept** → proceed to **Record the input**.
- **Refine further** → incorporate feedback, redraft, re-present.
- **`flip`** → switch the classification to the other route. Discard the current refined draft. Re-enter the appropriate **Refine the input** section under the new route. The flip keyword is recognized only as a standalone command at this prompt — text that includes "flip" mid-sentence as part of a refined question or scenario is recognized as user-provided content via the existing approve/refine selector, not as the override keyword.

The user's acceptance at this gate is the consent for any status mutation that follows. Do not prompt again for the back-edge.

### Impact display (spec target, question route, status ∈ {clarified, planned, in-progress})

When the question route is recording on a non-`draft` spec, display the impact before performing the write:

- The spec's prior status (the value that will be reverted from).
- Existence and last-modified timestamp of `plan.md`, `tasks.md`, and `data-model.md` in the feature directory. Omit files that do not exist.
- The list of files in `specs/{feature}/scenarios/` if that directory exists.
- A one-line dependency note when this spec is named in any other spec's frontmatter `dependencies` field. Scan sibling specs' frontmatter only (no body reads). When matches exist, render: "Note: this spec is a dependency of {comma-separated dependent slugs}; their pipeline checks will block until this spec returns to `clarified`."

This display is informational only — the user's prior acceptance is the consent.

### Impact display (spec target, scenario route, status = done)

When the scenario route is recording on a `done` spec, display the reopen impact:

- The spec's prior status (`done`, which will revert to `in-progress`).
- The new scenario's path: `scenarios/{slug}.md`.
- A note that the scenario adds a task to `tasks.md` and must be implemented before the spec returns to `done`.

Informational; no separate confirmation prompt.

### Record the input

**Question route:**

1. Append the accepted question to the `## Open Questions` section of the target artifact. If the section does not exist, create it in the appropriate location per the template.
2. If the target is a spec and its `status` is `clarified`, `planned`, or `in-progress`, update the frontmatter `status` field to `draft` in the same write (the back-edge). (For `draft` specs and scenario targets, no status mutation occurs.) Use the `set-status` primitive (MCP: `set-status`) when the runtime is registered; otherwise edit the frontmatter directly.
3. Run `npx markdownlint-cli2` on the modified file (primitive: `lint-markdown`, MCP: `lint-markdown`).

**Scenario route:**

1. Invoke `create-scenario` (MCP: `create-scenario`) to write `specs/{feature}/scenarios/{slug}.md` from the scenario template with the accepted `section`, Context, Behavior, and (optional) Edge Cases. The primitive creates the `scenarios/` subdirectory if absent and refuses on slug conflict. Otherwise, follow the markdown-only path: copy `specs/templates/spec/scenario.md` and substitute the fields by hand.
2. Invoke `append-task` (MCP: `append-task`) to append a numbered task block to `specs/{feature}/tasks.md` referencing the new scenario. The default body is a single checkbox `- [ ] Implement the behavior described in scenarios/{slug}.md`; the done-when condition is "the scenario's described behavior is correctly implemented and tested." Otherwise, follow the markdown-only path: append the task block by hand, computing the next task number as `max(existing) + 1`.
3. If the spec's `status` is `done`, invoke `set-status` (MCP: `set-status`) to flip `done → in-progress`. (For other spec statuses, no status mutation occurs.) Otherwise, edit the frontmatter directly.
4. Invoke `write-session` (MCP: `write-session`) to set the new scenario as the session target: pass the feature slug as the feature argument, the repo-relative spec directory as the path argument, the new scenario slug as the scenario argument, and `specs/{feature}/scenarios/{slug}.md` as the scenario-path argument. The primitive rewrites `{cli-config-dir}/{project}-session.json` atomically (tempfile + rename). On the markdown-only path, rewrite the JSON directly with top-level fields feature, path, scenario, scenarioPath, setAt (ISO 8601 UTC) in that order through the same tempfile + rename pattern.
5. Invoke `lint-markdown` (MCP: `lint-markdown`) on every modified file. Otherwise, follow the markdown-only path: run `npx markdownlint-cli2` directly.

### Status mutation summary

| Target | Prior status | Route | Behavior |
| --- | --- | --- | --- |
| Spec | `draft` | question | Append question only. No status mutation. |
| Spec | `clarified` / `planned` / `in-progress` | question | Show impact display, append question, revert `status` to `draft` in the same write. |
| Spec | `done` | question | Status tiebreaker auto-routes to scenario instead. The classifier never selects "question" on a `done` spec. |
| Spec | `draft` / `clarified` / `planned` / `in-progress` | scenario | Show reopen-not-needed impact (the spec is already accepting work), create scenario, append task, update session target. No status mutation. |
| Spec | `done` | scenario | Show reopen impact, create scenario, append task, revert `status` to `in-progress` in the same write, update session target. |
| Scenario | (no status field) | (forced question) | Append question to the scenario's Open Questions section. The parent spec's status is not read or mutated. |

### Prompt for another

Ask: "Do you have another input to add?" If yes, loop back to **Gather the input**. The mutation rules apply per input — once a spec has reverted to `draft` or reopened to `in-progress`, subsequent inputs in the same session just append.

When the user is done, display the next step:

- If a question was recorded on a spec: "Question recorded. Run `/{project}:clarify` to resolve it." On a spec, the status is now `draft` regardless of where it started.
- If a question was recorded on a scenario: "Question recorded. Run `/{project}:clarify` to resolve it." The parent spec's status is unchanged.
- If a scenario was recorded: "Scenario recorded at `specs/{feature}/scenarios/{slug}.md` and set as the session target. Run `/{project}:implement` to work on the new task."
- If the user aborted before accepting any input, exit silently — no input was recorded and no status mutation occurred.
