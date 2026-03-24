# Scenario

Create a scenario file for the targeted feature.

## Purpose

Walks the bug decision tree and creates a scenario file under the active feature's `scenarios/` directory. Appends a linked task to the feature's `tasks.md`. This is the primary mechanism for capturing bugs, edge cases, and detailed behavior.

## Context

Use the session target from `.claude/gov-session.json`. If no session target is set, stop and tell the user to run `/gov:target` first.

## Scope Boundaries

- This command creates a scenario file and appends a task. Do NOT begin implementing the scenario. Do NOT read or modify source code or test files.
- Read only the target feature's spec file, existing scenarios (for duplicate check), and `tasks.md` (for appending). Do NOT read plans, other features' specs, or source code.
- Reference: §bug-handling, §scenarios (constitution loaded by `/gov:target` — do not re-read).

## Instructions

### Confirm target

1. Read `.claude/gov-session.json` to get the active feature.
2. Read the feature's spec file (`spec.md` or `spec-and-plan.md`).
3. Display the feature name and status, and ask the user to confirm this is the correct target.

### Walk the decision tree

Ask the user to describe the bug, edge case, or behavior they want to capture. Then walk the decision tree:

1. **Does a spec exist for this behavior?**
   - If no — stop. Tell the user to create the spec first via `/gov:specify`, then come back to create the scenario.
2. **Is the spec ambiguous or incomplete for this behavior?**
   - If yes — tell the user to fix the spec directly. A scenario is not needed for spec-level gaps. Offer to help update the spec.
3. **Is the spec clear but the behavior needs lower-level elaboration?**
   - Proceed to create the scenario.

### Check for duplicates

1. Check if `specs/{feature}/scenarios/` exists. If not, create it.
2. List existing scenario files in the directory.
3. Derive a slug from the user's description (lowercase, hyphenated).
4. If a file with that slug already exists, stop and report the conflict. Ask the user to choose a different name or update the existing scenario.

### Create the scenario file

1. Create `specs/{feature}/scenarios/{slug}.md` using the `templates/scenario.md` template.
2. Fill in the spec-ref with the feature name and the relevant section.
3. Fill in Context and Behavior based on the user's description.
4. Include Edge Cases if the user mentioned any; otherwise remove that section.

### Append task to tasks.md

1. If `specs/{feature}/tasks.md` does not exist, create it with a heading: `# {NNN} — {Feature Name} Tasks`.
2. Append a new task entry referencing the scenario:

   ```markdown
   ## {next-number}. Implement scenario: {slug}

   - [ ] Implement the behavior described in `scenarios/{slug}.md`

   Done when: the scenario's described behavior is correctly implemented and tested.
   ```

### Update spec status

If the spec status is `done`, update it to `in-progress` — a bug can surface after completion and the spec needs rework.

### Set scenario as session target

Update `.claude/gov-session.json` to include the new scenario as the active target. Do not prompt for confirmation — the user explicitly asked to create this scenario.

```json
{
  "feature": "{NNN-feature-name}",
  "path": "specs/{NNN-feature-name}",
  "scenario": "{slug}",
  "scenarioPath": "specs/{NNN-feature-name}/scenarios/{slug}.md",
  "setAt": "{ISO 8601 timestamp}"
}
```

### Report

Display:

- The scenario file path created
- The task appended to `tasks.md`
- Suggested next step: `/gov:implement` to work on the new task
- **Stop here.** Do not begin implementation. The user will run `/gov:implement` when ready.
