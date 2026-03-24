# Question

Add an open question to the targeted artifact (spec or scenario).

## Purpose

Captures questions that arise at any point in the pipeline — during review, planning, implementation, or just thinking about the feature. Questions are refined collaboratively to be precise and self-contained, then appended to the targeted artifact's Open Questions section for resolution during `/gov:clarify`.

## Context

Use the session target from `.claude/gov-session.json`. If `$ARGUMENTS` is provided, use it as the initial question text. If no session target is set and no arguments provided, stop and tell the user to run `/gov:target` first.

## Target File Detection

Read `.claude/gov-session.json`. If the session includes a `scenario` and `scenarioPath`, the target artifact is the scenario file. Otherwise, the target artifact is the feature's spec file — check for `spec.md` first, then `spec-and-plan.md`. Use whichever exists. If neither exists, stop and report: "Spec does not exist. Run `/gov:specify` first."

## Scope Boundaries

- This command only reads the target artifact and appends to its Open Questions section. Do NOT modify any other section. Do NOT read or write plan files, tasks, source code, or test files.
- Reference: §spec-requirements (constitution loaded by `/gov:target` — do not re-read).

## Instructions

### Confirm target

1. Read `.claude/gov-session.json` to get the active feature and optional scenario.
2. Read the target artifact (scenario file if targeted, otherwise spec file).
3. Display the feature name, scenario name (if targeted), status, and a brief summary of what the artifact covers.

### Gather the question

1. If `$ARGUMENTS` is provided, use it as the initial question. Otherwise, ask the user: "What question do you want to capture?"

### Refine the question

The goal is a question that is precise, actionable, and self-contained — someone reading it during `/gov:clarify` should understand exactly what needs to be decided without extra context.

1. **Understand intent** — read the target artifact to understand how the question relates to its behaviors, contracts, acceptance criteria, or open areas. If the question's connection to the artifact is unclear, ask the user to explain how it applies.
2. **Draft a refined version** — rewrite the question so it:
   - Is specific to the spec's domain and terminology
   - Identifies which behavior, contract, or criterion it affects
   - States what decision or information is needed
   - Includes enough context that it stands alone
3. **Present for approval** — show the refined question and ask the user: accept this form, or continue working on it?
   - If the user wants to continue, incorporate their feedback and present a new version. Repeat until accepted.
   - If the user accepts, proceed to record.

### Record the question

1. Append the accepted question to the `## Open Questions` section of the target artifact. If the section does not exist, create it in the appropriate location per the template.
2. Run `markdownlint-cli2` on the modified file.

### Status warning

If the spec status is `clarified` or later, warn the user: "This spec is already clarified. Adding an open question means it will need to go through `/gov:clarify` again before advancing." Do not change the status — this is informational only.

### Prompt for another

Ask: "Do you have another question to add?" If yes, loop back to **Gather the question**. If no, display the next step:

- If spec status is `draft`: "Run `/gov:clarify` when ready to resolve open questions."
- If spec status is `clarified` or later: "Run `/gov:clarify` to re-resolve open questions before proceeding."
