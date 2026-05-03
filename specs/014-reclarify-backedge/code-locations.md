# 014 — Re-clarify Back-Edge Code Locations

## AC: `framework/commands/ask.md` reverts spec status to `draft` after appending an open question to a spec at `clarified`, `planned`, or `in-progress`

- `framework/commands/ask.md`

## AC: On a `draft` spec, `/ask` records the question without status mutation (existing behavior preserved)

- `framework/commands/ask.md`

## AC: On a `done` spec, `/ask` refuses and reports: "Spec is `done`. Run `/{project}:elaborate` to capture this as a scenario instead." No question is recorded; no status mutation occurs

- `framework/commands/ask.md`

## AC: When `/ask` mutates status, it displays the prior status, plan artifacts that exist (with last-modified timestamps), and scenario files — so the user can see what may need re-review

- `framework/commands/ask.md`

## AC: `/ask` does not prompt for separate yes/no confirmation before mutating status — the user's acceptance of the refined question is the consent

- `framework/commands/ask.md`

## AC: When `/ask` targets a scenario (per spec 009), it appends to the scenario's `## Open Questions` and does not mutate any spec or scenario status (scenarios have no status field)

- `framework/commands/ask.md`

## AC: `/ask`'s post-question hint is "Question recorded. Run `/{project}:clarify` to resolve it." in every case where a question is recorded

- `framework/commands/ask.md`

## AC: `framework/commands/clarify.md` Gate branches on the spec's open-question count, not on a flag

- `framework/commands/clarify.md`

## AC: On a `draft` spec (with or without open questions), the existing behavior is preserved — walk questions if present, verify ACs, advance to `clarified`

- `framework/commands/clarify.md`

## AC: On a `clarified` / `planned` / `in-progress` spec with no open questions, the command stops with the existing "Spec is already `{status}`" message (lightly tightened to mention the next pipeline command)

- `framework/commands/clarify.md`

## AC: On a `done` spec (any open-question count), the command stops with "Spec is `done`. Run `/{project}:elaborate` to capture this as a scenario instead." and exits without mutation

- `framework/commands/clarify.md`

## AC: No downstream artifacts (`plan.md`, `tasks.md`, `data-model.md`, scenario files) are deleted or rewritten by `/clarify`

- `framework/commands/clarify.md`

## AC: On a `clarified` / `planned` / `in-progress` spec with one or more open questions (an inconsistent state usually arising from manual frontmatter edit), the command displays the current status, open-question titles, plan-artifact list with timestamps, and scenario-file list, then prompts: "Spec is `{status}` but has {N} unresolved open questions — revert status to `draft` and walk the questions?"

- `framework/commands/clarify.md`

## AC: On confirm, the spec's frontmatter `status` field is updated to `draft` before the standard clarify walk proceeds

- `framework/commands/clarify.md`

## AC: On decline, no files are modified — the spec retains its inconsistent state and open questions remain in `## Open Questions`

- `framework/commands/clarify.md`

## AC: Previously-resolved questions in `## Resolved Questions` are never re-walked — `/clarify` only processes items in `## Open Questions`

- `framework/commands/clarify.md`

## AC: `framework/commands/plan.md` detects whether `plan.md`, `tasks.md`, or `data-model.md` already exist in the feature directory before generating

- `framework/commands/plan.md`

## AC: If any plan artifact exists, `/plan` lists the existing files with their last-modified timestamps and prompts the user to keep or replace

- `framework/commands/plan.md`

## AC: On "keep" (default), `/plan` skips template copy, runs the existing readiness check on the kept artifacts, and advances status to `planned` only if all checks pass

- `framework/commands/plan.md`

## AC: On "replace", `/plan` copies fresh templates over the existing files, then proceeds with the standard plan flow

- `framework/commands/plan.md`

## AC: If no plan artifacts exist, `/plan` behavior is unchanged

- `framework/commands/plan.md`

## AC: The protection applies to every `/plan` run, not only those triggered after a back-edge cycle

- `framework/commands/plan.md`

## AC: `framework/constitution.md` §spec-lifecycle back-edge bullet for `/ask` is rewritten per the **Constitution Updates** section above (named `/ask` as the entry point, `draft` as the destination)

- `framework/constitution.md`

## AC: `specs/000-slash-commands/spec.md` gains a signpost noting that `/ask` becomes the back-edge owner in 014 (mutating status to `draft` on non-`draft` specs), `/clarify` gains the open-questions-on-non-`draft`-spec recovery path in 014, and `/plan` gains overwrite-protection on existing artifacts in 014

- `specs/000-slash-commands/spec.md`

## AC: `.claude/commands/gov/ask.md`, `.claude/commands/gov/clarify.md`, and `.claude/commands/gov/plan.md` are regenerated via `scripts/gen-claude-commands.sh`

- `.claude/commands/gov/ask.md`
- `.claude/commands/gov/clarify.md`
- `.claude/commands/gov/plan.md`
