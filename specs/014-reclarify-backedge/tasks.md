# 014 — Re-clarify Back-Edge Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Update `/ask` source for back-edge ownership

- [x] In `framework/commands/ask.md`, remove the "Status warning" subsection (informational-only)
- [x] Add a "Status mutation" subsection that branches on the spec's current status: `draft` → no mutation; `clarified` / `planned` / `in-progress` → revert frontmatter `status` to `draft` in the same write that appends the question; `done` → refuse the recording (no question added, no mutation) and redirect to `/{project}:elaborate` with the spec's exact message
- [x] Add an "Impact display" step that runs only on a non-`draft`, non-`done` mutation: prior status, plan-artifact list with last-modified timestamps, scenario-file list, and a one-line dependency note when this spec is named in any other spec's frontmatter `dependencies` field
- [x] Update the "Refine the question" step to detect when the refined form matches an existing `## Open Questions` entry (normalized whitespace) and prompt skip-or-refine; on skip, exit without recording or mutating
- [x] Update the next-step hint after a recorded question to "Question recorded. Run `/{project}:clarify` to resolve it." in every case where a question is recorded; on `done` the hint is the elaborate redirect message instead
- [x] Confirm the scenario-targeted branch is unchanged — scenarios have no status field, so the back-edge does not apply

Done when: the source file describes the new ownership rules end-to-end and explicitly covers all four target shapes (draft spec, clarified+/in-progress spec, done spec, scenario target). All AC bullets under "`/ask` back-edge" in the spec are satisfied by the source text.

## 2. Update `/clarify` source for data-driven gate and recovery path

- [x] In `framework/commands/clarify.md`, replace the Gate subsection with a branch on `(status, open-question count)` per the spec's table
- [x] Preserve the existing draft hot path verbatim (walk questions if any, verify acceptance criteria, advance to `clarified`)
- [x] Add a "Recovery path" subsection that triggers when status is `clarified` / `planned` / `in-progress` and `## Open Questions` has at least one entry: display status, open-question titles, plan-artifact list with timestamps, and scenario-file list, then prompt with the wording in the spec's recovery-path acceptance criterion; on confirm, revert frontmatter `status` to `draft` and run the standard walk; on decline, exit with no file modifications
- [x] Add the explicit `done` branch: stop with "Spec is `done`. Run `/{project}:elaborate` to capture this as a scenario instead." and exit without mutation
- [x] Lightly tighten the existing "already `{status}`" message to mention the next pipeline command per the spec's table
- [x] Confirm the scenario-targeted clarify section is unchanged, including that `## Resolved Questions` is never re-walked
- [x] Confirm the existing dependency-readiness check still runs on the post-revert walk

Done when: the source file enumerates all five status × open-questions branches and their behaviors; the recovery prompt wording matches the spec's acceptance criterion; the scenario-clarify section is byte-for-byte unchanged except where the gate refactor unavoidably touches it.

## 3. Update `/plan` source for overwrite-protection

- [x] In `framework/commands/plan.md`, add a "Detect existing artifacts" step after the Gate but before "Create the plan"; check the feature directory for `plan.md`, `tasks.md`, and `data-model.md`
- [x] If any of those files exists, list them with last-modified timestamps and prompt "Plan artifacts exist from a prior `/plan` run. Keep them and run the readiness check, or replace with fresh templates?" with keep as the default
- [x] On keep: skip the template copy entirely, run the existing readiness check on the kept files, advance status to `planned` only if all checks pass; on failure, report the specific failures and do not advance
- [x] On replace: copy fresh templates over the existing files, then proceed with the standard plan flow
- [x] Confirm the existing lightweight-track branch (`spec-and-plan.md` causes plan creation to be skipped) is unchanged
- [x] Confirm behavior on a feature directory with no existing artifacts is unchanged from today

Done when: the source file describes both keep and replace branches and runs the readiness check on the keep path; the prompt wording matches the spec; an empty feature directory still behaves as it does today.

## 4. Rewrite constitution §spec-lifecycle back-edge bullet

- [x] In `framework/constitution.md` §spec-lifecycle, rewrite the second back-edge bullet to read: "`clarified` / `planned` / `in-progress` → `draft` when `/ask` records a new open question; the next `/clarify` resolves the question and the spec advances forward again."
- [x] Leave the first back-edge bullet (`/elaborate` for `done → in-progress`) untouched
- [x] Confirm the surrounding lifecycle prose still reads coherently with the new wording

Done when: the §spec-lifecycle bullet matches the **Constitution Updates** section of the spec exactly.

## 5. Add 014 signpost to spec 000

- [x] In `specs/000-slash-commands/spec.md`, add a Note line in the same style as the existing "subsequent specs renamed" note that records: 014 made `/ask` the owner of the `clarified+ → draft` back-edge (status mutation on non-`draft` specs), added a recovery path to `/clarify` for hand-edited inconsistent state, and added overwrite-protection to `/plan` for existing artifacts
- [x] Place the new note adjacent to the existing rename note for discoverability

Done when: the note appears near the existing rename note and references spec 014 explicitly.

## 6. Regenerate `.claude/commands/gov/` mirrors

- [x] Run `./scripts/gen-claude-commands.sh`
- [x] Verify `.claude/commands/gov/ask.md`, `clarify.md`, and `plan.md` reflect the source edits with `{project}` and `{cli-config-dir}` substituted correctly
- [x] Confirm `git diff` shows changes only in those three generated files (plus the source edits and other expected files)

Done when: the generator runs to completion and the regenerated files contain the new behaviors.

## 7. Lint all modified markdown

- [x] Run `npx markdownlint-cli2` on every modified file: the three sources, the constitution, the 000 spec, the three regenerated mirrors, this plan, and this tasks file
- [x] Fix any reported issues at the source (not in generated files); if a fix is needed in a generated file, fix it in the source and rerun the generator

Done when: `npx markdownlint-cli2` exits 0 across all modified files.
