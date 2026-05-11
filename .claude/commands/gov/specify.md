---
description: Create a new feature spec.
argument-hint: "[feature description]"
parity:
  strict-fields:
    - frontmatter
  strict-files:
    - "specs/{NNN-feature}/spec.md"
  semantic-fields:
    - spec-body
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
- Read only what is needed: existing spec directory names (for numbering) and the spec template. Do NOT read other specs' bodies unless checking for naming conflicts.
- Reference: §spec-phase, §spec-requirements, §lightweight-track, §numbering, §text-first-artifacts.

## Instructions

1. The walker context carries the feature description, the resolved NNN-slug, and the chosen track (standard or lightweight). The host pre-computes these from `$ARGUMENTS` and the lightweight-track detection prompts before invoking the runtime; the runtime steps below assume the new feature's directory already exists with an empty spec file copied from the template.

2. <!-- llm:writeSpecBody --> Fill the new spec body following §spec-requirements: a Motivation section, Acceptance Criteria with concrete and testable checkboxes, Open Questions, and any inline links to other specs that scripts/gen-spec-deps.sh will derive the frontmatter dependencies from. The host returns the markdown body for the new file; the walker forwards the response through the context. Otherwise, follow the markdown-only path: hand-write the spec body directly.

3. Invoke `lint-markdown` against the new spec file to surface any markdown violations the LLM may have introduced. Otherwise, fall back to the markdown-only path.

4. Ask the user to approve creating the new feature and setting it as the session target before any session-file write. On confirmation, the host writes the session JSON to point at the new feature; on denial, the walker exits cleanly without writing the session.

## Markdown-only reference

The full new-feature-creation procedure (track detection, directory creation, template copy, frontmatter conventions, session write, and next-step prompt) is documented below for the markdown-only path. The numbered steps above invoke the mechanical primitives plus the writeSpecBody extension that automate the deterministic phases.

### Resolve feature number and slug

1. `$ARGUMENTS` is the feature description (e.g., "webhook delivery"). This is required — if empty, ask the user what feature to specify.
2. Determine the next available feature number by checking existing directories under `specs/` matching the NNN-feature pattern; the next number is the highest existing NNN plus one (zero-padded to three digits).
3. Generate the slug from the feature description: lowercase, hyphenated, no whitespace, no punctuation beyond hyphens.

### Lightweight track detection

Ask the user the following qualifying questions:

- Does this touch more than one module or package?
- Are there open questions or unknowns about the approach?
- Does it involve data model changes beyond trivial?
- Will it be more than ~50 lines of spec?

If all answers indicate "small and clear," this is a lightweight track feature. Otherwise, use the standard track. The track is encoded by filename (`spec.md` vs `spec-and-plan.md`) — there is no track frontmatter field.

### Create the feature directory

1. Create `specs/{NNN-feature-name}/`.
2. Copy the appropriate template:
   - **Standard track**: copy `specs/templates/spec.md` into the directory as `spec.md`.
   - **Lightweight track**: copy `specs/templates/spec-and-plan.md` into the directory as `spec-and-plan.md`.

### Fill the spec body

Fill in the spec following `constitution.md` rules (§spec-requirements, §text-first-artifacts):

- Frontmatter `status` starts at `draft` (template default); `dependencies` starts at `[]` and is generator-managed (do not author by hand).
- Describe behavior and contracts, not implementation.
- No language-specific code, function signatures, or package paths.
- Acceptance criteria must be concrete and testable.
- List all open questions in the spec body.
- When the spec depends on other specs, link them inline in the body (e.g., `[NNN-feature](../NNN-feature/spec.md)`) — `scripts/gen-spec-deps.sh` (run by the pre-commit hook) derives the `dependencies:` frontmatter from those links on every commit.

### Lint the new file

Run `npx markdownlint-cli2` on the new file (primitive: `lint-markdown`).

### Write the session target

Write `.claude/gov-session.json` to set this feature as the session target (host responsibility; the runtime exposes no session-shaped primitive). Use tempfile + rename atomic-write semantics analogous to the runtime's spec write primitives.

### Display the next step

- Standard track: "Run `/gov:clarify` to resolve open questions and advance to clarified."
- Lightweight track: "Run `/gov:clarify` to review the combined spec-and-plan, then `/gov:plan` to create tasks."

The `README.md` Feature Specs table is regenerated by `scripts/gen-readme-table.sh` (run by the pre-commit hook); this command does not edit it.
