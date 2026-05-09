# 019 — Config-Persisted Decisions Tasks

Tasks derived from the [plan](plan.md). Complete in order. Each task is a documentation/runbook edit, not code; verification is reading the prose against the acceptance criteria, plus a synthetic walk-through where needed.

## 1. Document the extended `.govern.toml` schema in the bootstrap runbook

- [x] Edit `framework/bootstrap/govern.md` **Project Configuration** section (currently lines ~172–186):
  - Reword the section intro so `.govern.toml` is described as multi-purpose (configuration + persisted decisions), not pin-only.
  - Keep the existing `[pinned]` example and behavior text exactly as today.
  - Add a sibling `[workflows]` example showing `declined_categories = ["Linting", "Formatting"]`.
  - Add a brief explanation of `declined_categories`: case-insensitive match against the registry-derived category list, drives the suppression step in the workflow recommendation flow.
  - Add a back-link to `specs/019-config-decisions/data-model.md` as the canonical schema reference.
- [x] **Done when**: the section documents both `[pinned]` and `[workflows]` schemas with examples; the prose explicitly states the case-insensitive match rule; lints pass.

## 2. Add the decline-load step to the workflow recommendation flow

- [ ] Edit `framework/bootstrap/govern.md` **Workflow recommendation** flow (currently lines ~481–535).
- [ ] Add a new sub-step **between step 3 (read tech stack) and step 4 (match registry entries)** titled "Load recorded declines" that instructs the agent to:
  - Read `.govern.toml` if present.
  - Parse `[workflows] declined_categories` into a normalized lowercase set.
  - Stash the set for use in the prompt step.
  - Skip silently if `.govern.toml` is absent, has no `[workflows]` section, or has an empty `declined_categories` array.
- [ ] **Done when**: the new sub-step exists, is unambiguous about how to handle each "missing" case, and explicitly says "no abort on missing/empty."

## 3. Replace the per-category prompt step with the three-option flow

- [ ] Edit step 8 of the workflow recommendation flow ("Present per-category accept/skip prompts").
- [ ] Restate the step as a two-branch loop over candidate categories:
  - **Suppression branch**: if the lowercased category is in the decline set, do not invoke `AskUserQuestion`; emit `suppressed (workflow): {Category} (declined in .govern.toml)` into the summary; skip scaffolding for the category's workflows.
  - **Prompt branch**: invoke `AskUserQuestion` with three options exactly: `Yes, scaffold all in this category`, `Skip this run`, `Skip and don't ask again`.
- [ ] Define the answer-routing for the three options:
  - `Yes` → unchanged from today's accept path.
  - `Skip this run` → unchanged from today's skip path; nothing written to `.govern.toml`.
  - `Skip and don't ask again` → skip the category this run and add it to the persistence-write list (consumed in task 4).
- [ ] **Done when**: step 8 names all three options verbatim; the suppression branch is described with the summary-line text spelled out; the routing for each option is explicit.

## 4. Add the persistence-write step

- [ ] Add a new sub-step **between step 8 (prompts) and step 9 (fetch and write accepted workflows)** titled "Record persisted declines" that instructs the agent to:
  - For each category whose answer was `Skip and don't ask again`, append it to `[workflows] declined_categories` in `.govern.toml`.
  - If `.govern.toml` does not exist, create it with `[workflows] declined_categories = ["{Category}"]` and emit `created .govern.toml to record decline` into the summary.
  - If `.govern.toml` exists without `[workflows]`, add the section.
  - If `[workflows]` exists without `declined_categories`, add the key.
  - If the key exists, append the category, deduplicating case-insensitively.
  - Preserve all existing TOML content (other sections, comments, ordering).
- [ ] **Done when**: the sub-step covers every "shape of existing file" case (missing, present-without-section, present-without-key, present-with-key); the deduplication rule is explicit; the create-message text is spelled out.

## 5. Add the unrecognized-entry summary line to the load step

- [ ] In task 2's "Load recorded declines" sub-step, add a clause that records any `declined_categories` entry that doesn't match a canonical category name (case-insensitive against the registry-derived list).
- [ ] In step 11 (post-scaffolding summary) or wherever the summary is assembled, instruct the agent to emit one `unrecognized workflow decline: "{value}" (in .govern.toml)` line per recorded unrecognized entry.
- [ ] **Done when**: an unrecognized entry produces exactly one summary line, the run continues normally, and the prompts for valid categories are unaffected.

## 6. Create `data-model.md`

- [ ] Already drafted as part of `/gov:plan`; verify it lints, references 005, and covers `[pinned]` (existing), `[workflows]` (new), category list, case-insensitive matching, unrecognized entries, empty cases, future-section guidance, and backwards compatibility.
- [ ] **Done when**: `npx markdownlint-cli2 specs/019-config-decisions/data-model.md` passes; the schema declaration matches the runbook prose word-for-word on category names and key names.

## 7. Update README's `.govern.toml` section

- [ ] Edit `README.md` lines ~282–296. Rename **"Pinning files with .govern.toml"** to **"Configuring `.govern.toml`"**.
- [ ] Keep the existing `[pinned]` example.
- [ ] Add a `[workflows]` example showing `declined_categories = ["Linting"]` with one or two sentences explaining the prompt origin and how to undo (delete the entry, or the section, or the file).
- [ ] Cross-link to `specs/019-config-decisions/spec.md` and `specs/019-config-decisions/data-model.md` for full schema rationale.
- [ ] **Done when**: the renamed section documents both sections with minimal examples and links to the canonical sources; lints pass; section anchors that may be referenced elsewhere in the README remain resolvable (or are updated).

## 8. Add a signpost to spec 005

- [ ] Spec `005-workflows` is `done`. Per `done specs are frozen archaeology`, do not edit the body. Instead, add a top-of-spec signpost (between the frontmatter and the `# 005 — Workflows` heading area, sitting alongside the existing post-completion Note about the filename rename) that:
  - States: the per-category prompt now has three options instead of two, with a third `Skip and don't ask again` option that records the decline in `.govern.toml`.
  - Back-links to `specs/019-config-decisions/spec.md` for the current behavior.
  - Preserves the existing post-completion Note about the `{tool}.md` filename rename.
- [ ] **Done when**: 005's `spec.md` body is otherwise untouched; the signpost is at the top of the body and back-links to 019; lints pass.

## 9. Walk-through verification

- [ ] Bench-test the runbook against three synthetic `.govern.toml` shapes by reading the prose end-to-end:
  - **Shape A**: file does not exist; user picks `Skip and don't ask again` for `Linting`. Verify the runbook prose unambiguously walks the agent to: prompt with three options, write a new `.govern.toml` with `[workflows] declined_categories = ["Linting"]`, emit both summary lines (`created .govern.toml...` and the suppressed line on the *next* hypothetical run).
  - **Shape B**: file has `[pinned] files = [...]` and `[workflows] declined_categories = ["Formatting"]`. Verify the agent suppresses the `Formatting` prompt (with summary line), prompts for the rest with three options, and preserves `[pinned]` if the user adds a new decline.
  - **Shape C**: file has `[workflows] declined_categories = ["Linitng"]` (typo). Verify the agent prompts for `Linting` normally and emits exactly one `unrecognized workflow decline: "Linitng"` summary line.
- [ ] **Done when**: all three walks confirm the runbook prose is unambiguous. If any shape exposes ambiguity, refine the prose in tasks 1–5 and re-walk.

## 10. Lint and finalize

- [ ] Run `npx markdownlint-cli2` against the entire feature directory and any modified files (`framework/bootstrap/govern.md`, `README.md`, `specs/005-workflows/spec.md`).
- [ ] Verify `scripts/gen-spec-deps.sh --dry-run` reports no changes (the dependency on 005 was already added during clarify).
- [ ] **Done when**: all lints pass; the dependencies frontmatter is in sync; spec status is ready to advance to `done`.
