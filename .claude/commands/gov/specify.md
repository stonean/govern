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

First step in the pipeline. Creates a new numbered feature directory with a spec from template and sets it as the session target. Accepts both greenfield input (rich description with concrete acceptance criteria) and brownfield input (sparse description of an existing feature) — richness scales with the input. Sparse acceptance criteria are valid for brownfield use; the spec gains precision through subsequent bug fixes, scenarios, and clarifications.

## Context

This command does not require a session target — it creates a new feature. If `.govern.session.toml` exists, the session target will be overwritten with the new feature.

If the constitution has not been loaded in this session (e.g., `/gov:target` has not been run), read `constitution.md` now to load `govern` rules. If the constitution was already loaded by `/gov:target`, do not re-read it.

## Scope Boundaries

- This command creates spec artifacts only. Do NOT read or write source code, test files, or implementation files.
- Read only what is needed: existing spec directory names (for numbering) and the spec template. Do NOT read other specs' bodies unless checking for naming conflicts.
- Reference: §spec-phase, §spec-requirements, §numbering, §text-first-artifacts, §brownfield-process.

## Instructions

> **For agent runtimes**: the Invoke steps below call the MCP tools of the optional gvrn runtime; the host-integration contract — bare↔prefixed tool names, lazy ToolSearch schema fetch, the no-shell-utilities rule, and the two-paths guarantee — lives once in the constitution, §runtime-host-integration. With no gvrn MCP server registered, walk the same prose using the host file-reading tools (Read, Edit, Write).

1. Invoke `create-feature` with the feature description from `$ARGUMENTS` as the title (the description is required — if empty, ask the user what feature to specify). The primitive computes the next feature number from the existing NNN-prefixed directories under the configured specs root, derives the kebab-case slug, creates `specs/{NNN-slug}/`, and copies the spec template into it atomically (mode-preserving). An already-existing target directory is the `created: false` domain outcome — report the collision and stop rather than overwrite. Otherwise, fall back to the markdown-only path below.

2. <!-- llm:writeSpecBody --> Fill the new spec body following §spec-requirements: a Motivation section, Acceptance Criteria with concrete and testable checkboxes (sparse acceptance criteria are valid for brownfield use — leave the section with a comment noting criteria will emerge from real work), Open Questions, and any inline links to other specs that scripts/gen-spec-deps.sh will derive the frontmatter dependencies from. The host returns the markdown body for the new file; the walker forwards the response through the context.

3. Invoke `lint-markdown` against the new spec file to surface any markdown violations the LLM may have introduced. Otherwise, fall back to the markdown-only path.

4. Invoke `gate-confirm` with a prompt asking the user to approve creating the new feature and setting it as the session target before any session-file write. On confirmation, continue to the session write below; on denial, the walker exits cleanly without writing the session.

5. Invoke `write-session` with the new feature slug and its repo-relative spec directory — under the configured `[paths] specs-root` (default `specs`; spec 040) — as the feature and path arguments. This is a target write: the primitive stamps a fresh set-at while preserving any cli-config-dir already in the file (the per-contributor agent identity written by `/govern`), at `.govern.session.toml`, through tempfile + rename atomic-write semantics. On the markdown-only path, the host writes the file by hand per the markdown-only reference's Write the session target section — the cli-config-dir preservation rule there applies verbatim.

## Markdown-only reference

The full new-feature-creation procedure (directory creation, template copy, frontmatter conventions, session write, and next-step prompt) is documented below for the markdown-only path. The numbered steps above invoke the mechanical primitives plus the writeSpecBody extension that automate the deterministic phases.

> **Spec-root resolution.** Every `specs/…` path below is written under the configured `[paths] specs-root` (default `specs`; spec 040, constitution §spec-phase). When `.govern.toml` sets `[paths] specs-root`, substitute that name for the literal `specs/` throughout — the feature-number scan, the new feature directory, the `templates/spec.md` source, and the session `path`. The literal `specs/` is the documented default; the runtime primitives already resolve it, so only this markdown-only path performs the substitution by hand.

### Resolve feature number and slug

1. `$ARGUMENTS` is the feature description (e.g., "webhook delivery"). This is required — if empty, ask the user what feature to specify.
2. Determine the next available feature number by checking existing directories under `specs/` matching the NNN-feature pattern; the next number is the highest existing NNN plus one (zero-padded to three digits).
3. Generate the slug from the feature description: lowercase, hyphenated, no whitespace, no punctuation beyond hyphens.

### Create the feature directory

1. Create `specs/{NNN-feature-name}/`.
2. Copy `specs/templates/spec.md` into the directory as `spec.md`.

Both sections above are what the `create-feature` primitive automates on the runtime path (number scan, slug derivation, directory creation, atomic template copy); walk them by hand only when no runtime is available.

### Fill the spec body

Fill in the spec following `constitution.md` rules (§spec-requirements, §text-first-artifacts):

- Frontmatter `status` starts at `draft` (template default); `dependencies` starts at `[]` and is generator-managed (do not author by hand).
- Describe behavior and contracts, not implementation.
- No language-specific code, function signatures, or package paths.
- Acceptance criteria must be concrete and testable when present. For brownfield use, sparse acceptance criteria are expected and valid — leave the section with a placeholder comment if no criteria are known yet; criteria emerge as real work touches the feature (§brownfield-process).
- List all open questions in the spec body.
- When the spec depends on other specs, link them inline in the body (e.g., `[NNN-feature](../NNN-feature/spec.md)`) — `scripts/gen-spec-deps.sh` (run by the pre-commit hook) derives the `dependencies:` frontmatter from those links on every commit.

### Lint the new file

Run `npx markdownlint-cli2` on the new file (primitive: `lint-markdown`).

### Write the session target

Write `.govern.session.toml` to set this feature as the session target (primitive: `write-session`, gated by `gate-confirm` above). First read any existing `.govern.session.toml` to capture its cli-config-dir (the per-contributor agent identity written by /govern) and carry it forward, so creating a new feature never drops the agent identity. Use tempfile + rename atomic-write semantics analogous to the runtime's spec write primitives.

### Display the next step

Display: "Run `/gov:clarify` to resolve open questions and advance to clarified."
