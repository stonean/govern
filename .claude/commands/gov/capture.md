# Capture

Create a skeleton spec for an existing feature in a brownfield project.

## Purpose

Brownfield entry point. Creates a numbered feature directory with a skeleton spec from freeform user input. Unlike `/gov:specify`, this command does not ask qualifying questions or push for comprehensive criteria — it captures what is known and stops.

## Context

This command does not require a session target — it creates a new feature. If `.claude/gov-session.json` exists, the session target will be overwritten with the new feature.

If the constitution has not been loaded in this session (e.g., `/gov:target` has not been run), read `constitution.md` now to load governance rules. If the constitution was already loaded by `/gov:target`, do not re-read it.

## Scope Boundaries

- This command creates spec artifacts only. Do NOT read or write source code, test files, or implementation files.
- Do NOT read existing code to infer behavior — the spec captures intended behavior as understood by the user.
- Do NOT create scenarios — the user runs `/gov:scenario` separately to decompose.
- Read only what is needed: existing spec directory names (for numbering), the spec template, and `README.md` (for the feature table). Do NOT read other specs' contents unless checking for naming conflicts.
- Reference: §spec-phase, §spec-requirements, §brownfield-process, §numbering.

## Instructions

1. `$ARGUMENTS` is the feature description. If empty, ask the user to describe the existing feature they want to capture. Suggest starting broad — it is easier to decompose a broad feature into scenarios later than to combine over-partitioned specs.

2. At minimum, the user must provide a name and one-sentence description. If the input is too vague to produce a meaningful skeleton, ask for more.

3. Determine the next available feature number by checking existing directories under `specs/`.

4. **Check for naming conflicts.** If a spec directory already exists for this feature name, stop and report the conflict. Suggest `/gov:target` to work on the existing spec.

5. Create `specs/{NNN-feature-name}/`.

6. Copy `specs/templates/spec.md` into the directory as `spec.md`.

7. Fill in the spec from the user's description:
   - Set status to `draft`.
   - Populate sections with whatever behavior is known — sparse acceptance criteria are expected and valid.
   - If no acceptance criteria are known, leave the Acceptance Criteria section empty (with a comment noting criteria will emerge from real work).
   - List any open questions the user mentioned.
   - List dependencies on other specs if apparent.
   - Present the draft for user review before writing.

8. Add the new feature to the table in `README.md`.

9. Run `npx markdownlint-cli2` on the new spec file.

10. Write `.claude/gov-session.json` to set this feature as the session target.

11. Display the post-capture message:

    > Spec created at `specs/{NNN-feature-name}/spec.md` and set as session target.
    >
    > What to do next — depends on why you captured this:
    >
    > - `/gov:scenario` — capture a bug or edge case
    > - `/gov:clarify` — flesh out the spec
    > - Leave at `draft` and come back when real work arrives
