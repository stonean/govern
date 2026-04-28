# Specify

Create a new feature spec.

## Purpose

First step in the pipeline. Creates a new numbered feature directory with a spec from template. Automatically sets the new feature as the session target.

## Context

This command does not require a session target — it creates a new feature. If `{cli-config-dir}/{project}-session.json` exists, the session target will be overwritten with the new feature.

If the constitution has not been loaded in this session (e.g., `/{project}:target` has not been run), read `constitution.md` now to load governance rules. If the constitution was already loaded by `/{project}:target`, do not re-read it.

## Scope Boundaries

- This command creates spec artifacts only. Do NOT read or write source code, test files, or implementation files.
- Read only what is needed: existing spec directory names (for numbering), the spec template, and `README.md` (for the feature table). Do NOT read other specs' contents unless checking for naming conflicts.
- Reference: §spec-phase, §spec-requirements, §lightweight-track, §numbering.

## Instructions

1. `$ARGUMENTS` is the feature description (e.g., "webhook delivery"). This is required — if empty, ask the user what feature to specify.

2. Determine the next available feature number by checking existing directories under `specs/`.

3. **Lightweight track detection** — ask the user the following qualifying questions:

   - Does this touch more than one module or package?
   - Are there open questions or unknowns about the approach?
   - Does it involve data model changes beyond trivial?
   - Will it be more than ~50 lines of spec?

   If all answers indicate "small and clear," this is a lightweight track feature. Otherwise, use the standard track.

4. Create `specs/{NNN-feature-name}/`.

5. Copy the appropriate template:
   - **Standard track**: copy `specs/templates/spec.md` into the directory as `spec.md`
   - **Lightweight track**: copy `specs/templates/spec-and-plan.md` into the directory as `spec-and-plan.md`

6. Fill in the spec following `constitution.md` rules (§spec-requirements):
   - Describe behavior and contracts, not implementation.
   - No language-specific code, function signatures, or package paths.
   - Acceptance criteria must be concrete and testable.
   - List all open questions.
   - List dependencies on other specs.

7. Add the new feature to the table in `README.md`.

8. Run `npx markdownlint-cli2` on the new file.

9. Write `{cli-config-dir}/{project}-session.json` to set this feature as the session target.

10. Display the next step:
    - Standard track: "Run `/{project}:clarify` to resolve open questions and advance to `clarified`."
    - Lightweight track: "Run `/{project}:clarify` to review the combined spec-and-plan, then `/{project}:plan` to create tasks."
