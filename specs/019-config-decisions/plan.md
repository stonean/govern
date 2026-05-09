# 019 — Config-Persisted Decisions Plan

Implements [019 — Config-Persisted Decisions](spec.md).

## Overview

`/govern` (sourced from `framework/bootstrap/govern.md`) is an LLM-executed runbook. There is no compiler, no parser, no test suite — adopters paste the runbook into their AI agent and the agent follows the steps. Implementation here is therefore a **prose change**: edit two passages in `framework/bootstrap/govern.md` (the **Project Configuration** schema and the **Workflow recommendation** flow), edit the README's `.govern.toml` section, and add a small `data-model.md` declaring the new schema for cross-spec reference.

The scope is narrow by design — Q4 of the clarify pass scoped first delivery to workflows-only, deferring `[agents]` and other domains to follow-on specs.

## Technical Decisions

### Persistence layer is the runbook prose, not code

The `/govern` runbook tells the agent how to read, parse, edit, and write `.govern.toml`. There is no `govern`-side parser to update, no helper library, no test harness. The change is to the instructions the agent follows.

**Implication for the plan:** every "behavior change" in the spec maps to a prose edit in `framework/bootstrap/govern.md`. The plan tasks are edit-shaped, not code-shaped. Verification is reading the prose against the acceptance criteria, plus a bench-test pass where one of us walks the runbook against a synthetic `.govern.toml` to confirm the steps are unambiguous.

### Edit site 1: Project Configuration section

Currently at `framework/bootstrap/govern.md` lines 172–186 (the `[pinned] files = [...]` documentation). This is where the spec's schema documentation lives. Edits:

1. Add the `[workflows]` section to the schema example, keeping `[pinned]` exactly as today.
2. Document the `declined_categories` key, the case-insensitive match against the registry-derived category list, and the soft-validation behavior on unrecognized values.
3. Reword the section header/intro to reflect that `.govern.toml` is now multi-purpose (configuration + persisted decisions) rather than pin-only.

The schema declaration here is read by adopters; it does not drive parsing. The actual parse-and-act steps live in the workflow recommendation flow.

### Edit site 2: Workflow recommendation flow

Currently at `framework/bootstrap/govern.md` lines 481–535 (steps 1–11). Edits target three specific points in the flow:

- **Before step 4 (registry matching)** — add a "load decline list" sub-step. After step 3 (read tech stack), read `.govern.toml` (if present), parse `[workflows] declined_categories` into a normalized lowercase set, and stash it for steps 4 and 8. Unrecognized entries (those that don't match any registry-derived category name when checked at step 8) are recorded for the post-scaffolding summary; they do not abort the run.

- **At step 8 (per-category accept/skip prompts)** — split the prompt logic into two paths. For each category in this run's match set:
  - If the category (lowercased) is in the decline set → suppress the prompt entirely; emit a `suppressed (workflow): {Category} (declined in .govern.toml)` line into the post-scaffolding summary; do not scaffold any workflows in this category.
  - Otherwise → present the prompt with **three options** instead of two: `Yes, scaffold all in this category` (today's accept), `Skip this run` (today's skip), `Skip and don't ask again` (new — declines and persists).

- **Add a new sub-step after step 8 / before step 9** — for every category whose answer was "Skip and don't ask again," append the category name to `[workflows] declined_categories` in `.govern.toml`. Behavior:
  - If `.govern.toml` does not exist, create it with just `[workflows] declined_categories = ["{Category}"]`. Emit `created .govern.toml to record decline` into the post-scaffolding summary.
  - If `.govern.toml` exists without `[workflows]`, add the section.
  - If `[workflows]` exists without `declined_categories`, add the key.
  - If the key exists, append the category. Deduplicate (case-insensitive) before writing.
  - Preserve any existing TOML the file already has (`[pinned]`, comments, ordering) — read, modify in place, write back.

- **At step 11 / post-scaffolding summary** — already covered by the per-step summary lines above. The unrecognized-value lines (`unrecognized workflow decline: "{value}" (in .govern.toml)`) are emitted once each from the load step at the top.

### Edit site 3: README.md

The "Pinning files with .govern.toml" section (currently at README.md lines 282–296) becomes "Configuring `.govern.toml`":

- Keep the existing `[pinned]` example and explanation.
- Add a `[workflows]` example showing `declined_categories = [...]`.
- Document how to undo a recorded decline (delete the entry from the array, or delete the section, or delete the file).
- Cross-link to spec 019 for the full schema rationale.

### Edit site 4: data-model.md (new)

Add a brief `data-model.md` declaring the `.govern.toml` schema additions. The file is small but worth having because:

- It's the canonical source for the schema (the README is for adopters; `data-model.md` is for the framework's internal cross-reference).
- Future specs that extend `.govern.toml` (e.g., the deferred `[agents]` domain) can reference this file to confirm shape conventions.
- `/gov:validate` will pick it up if rules are added later.

### TOML editing without a parser

`/govern` is run by an LLM; in-place TOML editing is well within the agent's reach (read the file, locate the `[workflows]` section by line search, append to the array, write the file back). The runbook prose specifies the edit at the level of "find the `[workflows]` section, ensure `declined_categories` exists as a TOML array, append `{Category}` if not already present (case-insensitive), and write the file." No CLI dependency, no parser library — same posture as the existing `[pinned]` documentation.

**Edge case to spell out in prose:** if the existing TOML has comments inside `declined_categories` (e.g., a `# comment` inline), the agent should preserve them. The runbook will instruct: "preserve all existing content; only append a new entry to the array."

### Three-option prompt mechanics

Today the workflow prompt uses `AskUserQuestion` with two options. The change is to provide a third option. `AskUserQuestion`'s schema accepts an arbitrary list of options, so this is a schema-data change in the prompt invocation, not a different mechanism. The runbook prose spells out all three option labels exactly so adopters across agents (Claude Code, Auggie, future agents) get uniform behavior.

### What is intentionally NOT in scope

- **No `[agents]` section, `[cleanup]` section, or any non-workflow domain.** Deferred per Q4 to follow-on specs.
- **No commit hook or `/gov:validate` rule for `.govern.toml`.** The summary-line surface is the only enforcement layer, by Q9.
- **No CLI tooling to add/remove declines.** The third prompt option is the add-path; manual editing is the remove-path.
- **No telemetry, version-tagging, or expiration logic.** Permanent until manually edited, by Q3.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/bootstrap/govern.md` | Modify | Edit site 1 (Project Configuration schema, lines ~172–186); edit site 2 (Workflow recommendation flow, steps 4/8/9, lines ~481–535) |
| `README.md` | Modify | Rename and expand the "Pinning files with .govern.toml" section (lines ~282–296) |
| `specs/019-config-decisions/data-model.md` | Create | Canonical `.govern.toml` schema declaration |
| `specs/005-workflows/spec.md` | Modify | Add a top-of-spec signpost noting that the per-category prompt now has three options (006-spec is `done`, frozen archaeology — signpost only, not body rewrite); back-link to spec 019 |

This is a planning aid; `/gov:implement` derives the runtime write boundary from `git diff`. Implement-time additions (e.g., a small README cross-link adjustment) surface naturally.

## Trade-offs

- **No structured TOML validator.** We rely on the agent's literal adherence to runbook prose. If an adopter hand-edits `.govern.toml` into something the runbook's loose parsing can't handle (unusual whitespace, multi-line arrays with comments), the worst case is the load step fails to find the entries and `/govern` falls back to "no declines recorded" — the prompts re-fire. This is annoying but not destructive. The TOML parse-error abort path remains the hard fail mode.
- **Prose-driven edits are reviewable but not test-runnable.** A human (or a second agent) has to walk the runbook against a synthetic project to confirm the steps actually do what the spec promises. This is the same posture as every other change to `framework/bootstrap/govern.md`; it's the cost of the LLM-runbook architecture.
- **README and bootstrap drift risk.** Two places now describe `.govern.toml`. Keeping them in sync is on the author of any future schema change. Per the constitution's drift-prevention discipline, the bootstrap is the canonical source and the README links to spec 019 for the full schema; the README example stays minimal and references the bootstrap.
- **Spec 005 is `done`.** Adding the three-option prompt is a behavior change to a flow 005 defined. Per `done specs are frozen archaeology`, we add a top-of-spec signpost rather than rewriting the body. This keeps 005's history intact while pointing readers at 019 for the current behavior. The signpost is part of this spec's tasks.
- **No first-run safety net for malformed user TOML.** If an adopter has manually written `[workflows]` content that doesn't match the documented shape (e.g., used a different key name), the soft-validation summary line is the only signal. The runbook explicitly does not coerce or auto-correct user-written keys.
