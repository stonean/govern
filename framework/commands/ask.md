---
description: Append an open question to the targeted spec or scenario.
argument-hint: "[question]"
---

# Ask

Append an open question to the targeted artifact (spec or scenario). On a spec at `clarified` / `planned` / `in-progress`, also revert status to `draft` — the only state that tolerates open questions.

## Purpose

Captures questions that arise at any point in the pipeline — during review, planning, implementation, or just thinking about the feature. Questions are refined collaboratively to be precise and self-contained, then appended to the targeted artifact's Open Questions section for resolution during `/{project}:clarify`.

`/{project}:ask` owns the `clarified` / `planned` / `in-progress` → `draft` back-edge defined in the constitution's §spec-lifecycle. When recording an open question would leave a spec in an internally inconsistent state (status says "questions resolved" but the body has unresolved questions), the same write that records the question reverts status to `draft`. The user's acceptance of the refined question is the consent for that mutation; no separate confirmation prompt fires.

## Context

Use the session target from `{cli-config-dir}/{project}-session.json`. If `$ARGUMENTS` is provided, use it as the initial question text. If no session target is set and no arguments provided, stop and tell the user to run `/{project}:target` first.

## Target File Detection

Read `{cli-config-dir}/{project}-session.json`. If the session includes a `scenario` and `scenarioPath`, the target artifact is the scenario file. Otherwise, the target artifact is the feature's `spec.md`. If that file does not exist, stop and report: "Spec does not exist. Run `/{project}:specify` first."

## Scope Boundaries

- This command reads the target artifact, appends to its `## Open Questions` section, and — when the back-edge applies — updates the spec's frontmatter `status` field. No other artifact contents are modified. Plan files, tasks, source code, and test files are never read or written.
- Spec `status` is read from the YAML frontmatter at the top of the file. It is mutated by this command only on the `clarified+ → draft` back-edge described below.
- For the impact display, this command may read sibling specs' frontmatter (only) under `specs/` to detect dependents. It does not read sibling spec bodies.
- Reference: §spec-requirements, §spec-lifecycle, §text-first-artifacts (constitution loaded by `/{project}:target` — do not re-read).

## Instructions

### Confirm target

1. Read `{cli-config-dir}/{project}-session.json` to get the session target's feature and optional scenario.
2. Read the target artifact (scenario file if targeted, otherwise spec file).
3. **Recompute dependencies (safety net).** If the target is a spec, run `scripts/gen-spec-deps.sh --dry-run` against it. If it reports a diff, run it for real to sync `dependencies:` from body inline links. The pre-commit hook normally keeps this in sync; this step catches uncommitted body edits. (Skip on scenario targets — scenarios have no `dependencies` field.)
4. If the target is a spec, read its frontmatter `status` field now — the value is needed for the gate, the impact display, and the post-record mutation.
5. Display the feature name, scenario name (if targeted), status, and a brief summary of what the artifact covers.

### Gate: refuse on `done` spec

If the target is a spec (not a scenario) and `status` is `done`, stop without recording anything. Report:

> Spec is `done`. Run `/{project}:elaborate` to capture this as a scenario instead.

A question on a `done` spec means either the behavior needs lower-level elaboration (a scenario, owned by `/{project}:elaborate`) or the spec is wrong (manual revision). `/{project}:ask`'s back-edge does not cover either. No question is recorded; no status mutation occurs.

Scenario targets have no status field, so this gate does not apply to them — proceed to **Gather the question**.

### Gather the question

1. If `$ARGUMENTS` is provided, use it as the initial question. Otherwise, ask the user: "What question do you want to capture?"

### Refine the question

The goal is a question that is precise, actionable, and self-contained — someone reading it during `/{project}:clarify` should understand exactly what needs to be decided without extra context.

1. **Understand intent** — read the target artifact to understand how the question relates to its behaviors, contracts, acceptance criteria, or open areas. If the question's connection to the artifact is unclear, ask the user to explain how it applies.
2. **Draft a refined version** — rewrite the question so it:
   - Is specific to the spec's domain and terminology
   - Identifies which behavior, contract, or criterion it affects
   - States what decision or information is needed
   - Includes enough context that it stands alone
3. **Check for duplicates** — before presenting the refined version for approval, compare it against the entries already in the target artifact's `## Open Questions` section. Use a normalized-whitespace comparison (collapse runs of whitespace, trim, case-insensitive). If the refined form matches an existing entry, report:
   > An equivalent question is already recorded: "{existing entry}". Skip or refine further?
   - **Skip** — exit without recording. No question is appended; no status mutation occurs.
   - **Refine further** — incorporate the user's feedback and loop back to step 2.
4. **Present for approval** — show the refined question and ask the user: accept this form, or continue working on it?
   - If the user wants to continue, incorporate their feedback and present a new version. Repeat until accepted.
   - If the user accepts, proceed to the impact display (if applicable) and then record.

The user's acceptance of the refined question at this step is the consent for any status mutation that follows. Do not prompt again.

### Impact display (spec target only, status ∈ {clarified, planned, in-progress})

If the target is a spec and its `status` is `clarified`, `planned`, or `in-progress`, display the impact of recording the question before performing the write. The display surfaces what the user may need to re-review after the question is resolved; the artifacts themselves are not modified by this command.

Show:

- The spec's prior status (the value that will be reverted from).
- Existence and last-modified timestamp of `plan.md`, `tasks.md`, and `data-model.md` in the feature directory. Omit files that do not exist.
- The list of files in `specs/{feature}/scenarios/` if that directory exists.
- A one-line dependency note when this spec is named in any other spec's frontmatter `dependencies` field. Scan sibling specs' frontmatter only (no body reads). When matches exist, render: "Note: this spec is a dependency of {comma-separated dependent slugs}; their pipeline checks will block until this spec returns to `clarified`."

Do not prompt for confirmation here — the user's prior acceptance of the refined question is the consent. The display is informational only.

### Record the question

1. Append the accepted question to the `## Open Questions` section of the target artifact. If the section does not exist, create it in the appropriate location per the template.
2. If the target is a spec and its `status` is `clarified`, `planned`, or `in-progress`, update the frontmatter `status` field to `draft` in the same write. (For `draft` specs and scenario targets, no status mutation occurs.)
3. Run `npx markdownlint-cli2` on the modified file.

### Status mutation summary

| Target | Prior status | Behavior |
| --- | --- | --- |
| Spec | `draft` | Append question only. No status mutation. |
| Spec | `clarified` / `planned` / `in-progress` | Show impact display, append question, revert `status` to `draft` in the same write. |
| Spec | `done` | Refuse before recording (see **Gate: refuse on `done` spec**). |
| Scenario | (no status field) | Append question only. The parent spec's status is not read or mutated. |

### Prompt for another

Ask: "Do you have another question to add?" If yes, loop back to **Gather the question**. The mutation rules apply per question — once a spec has reverted to `draft`, subsequent questions in the same session simply append (no further status change).

When the user is done, display the next step:

- If a question was recorded (on a spec or a scenario), display: "Question recorded. Run `/{project}:clarify` to resolve it." On a spec, the status is now `draft` regardless of where it started; on a scenario, `/{project}:clarify` resolves the scenario's open questions per spec 009.
- If recording was refused on a `done` spec, the message in **Gate: refuse on `done` spec** has already been shown — do not display a separate next-step hint.
- If the user aborted the refinement loop without accepting any question, exit silently — no question was recorded and no status mutation occurred.
