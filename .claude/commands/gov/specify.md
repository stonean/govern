---
description: Create a new feature spec.
argument-hint: "[feature description]"
---

# Specify

Create a new feature spec.

## Purpose

First step in the pipeline. Creates a new numbered feature directory with a spec from template. Automatically sets the new feature as the session target.

## Context

This command does not require a session target — it creates a new feature. If `.claude/gov-session.json` exists, the session target will be overwritten with the new feature.

If the constitution has not been loaded in this session (e.g., `/gov:target` has not been run), read `constitution.md` now to load `govern` rules. If the constitution was already loaded by `/gov:target`, do not re-read it.

## Scope Boundaries

- This command creates spec artifacts only. Do NOT read or write source code, test files, or implementation files.
- Read only what is needed: existing spec directory names (for numbering), existing specs' frontmatter (for tag suggestions), the spec template, and `README.md` (for the feature table). Do NOT read other specs' bodies unless checking for naming conflicts.
- Reference: §spec-phase, §spec-requirements, §lightweight-track, §numbering, §text-first-artifacts.

## Instructions

1. `$ARGUMENTS` is the feature description (e.g., "webhook delivery"). This is required — if empty, ask the user what feature to specify.

2. Determine the next available feature number by checking existing directories under `specs/`.

3. **Lightweight track detection** — ask the user the following qualifying questions:

   - Does this touch more than one module or package?
   - Are there open questions or unknowns about the approach?
   - Does it involve data model changes beyond trivial?
   - Will it be more than ~50 lines of spec?

   If all answers indicate "small and clear," this is a lightweight track feature. Otherwise, use the standard track.

4. **Tag prompt** — read the YAML frontmatter `tags` field from each sibling spec in `specs/*/spec.md` and `specs/*/spec-and-plan.md`. Display the union of those tag values plus the constitution's starter vocabulary as suggestions. Prompt the author: *"Tags for this spec? (Pick from existing or enter new — comma-separated. Or skip.)"* Record the response. If the author skips, leave the new spec's `tags` field as `[]`. Tags help cross-cutting graph-view consumers (Quartz, Obsidian) but are not required.

5. Create `specs/{NNN-feature-name}/`.

6. Copy the appropriate template:
   - **Standard track**: copy `specs/templates/spec.md` into the directory as `spec.md`
   - **Lightweight track**: copy `specs/templates/spec-and-plan.md` into the directory as `spec-and-plan.md`

7. Fill in the spec following `constitution.md` rules (§spec-requirements, §text-first-artifacts):
   - Replace the frontmatter `title` placeholder with the resolved value: `"{NNN-feature-name} — spec"` for the standard track or `"{NNN-feature-name} — spec+plan"` for the lightweight track (e.g., `"005-authentication — spec"`). The title gives PKM tools (Obsidian graph, Quartz) a unique node label per artifact, since every feature directory contains a `spec.md`/`plan.md`/`tasks.md`.
   - Update the frontmatter `tags` field with the values collected in step 4 (or leave as `[]` if skipped).
   - Leave frontmatter `dependencies` as `[]`; add entries as the author identifies them or as `/clarify` resolves them.
   - Describe behavior and contracts, not implementation.
   - No language-specific code, function signatures, or package paths.
   - Acceptance criteria must be concrete and testable.
   - List all open questions in the spec body.
   - Add any identified dependencies to the frontmatter `dependencies` list.

8. Add the new feature to the table in `README.md`.

9. Run `npx markdownlint-cli2` on the new file.

10. Write `.claude/gov-session.json` to set this feature as the session target.

11. Display the next step:
    - Standard track: "Run `/gov:clarify` to resolve open questions and advance to `clarified`."
    - Lightweight track: "Run `/gov:clarify` to review the combined spec-and-plan, then `/gov:plan` to create tasks."
