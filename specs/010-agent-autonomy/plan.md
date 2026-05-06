---
title: "010-agent-autonomy — plan"
---

# 010 — Agent Autonomy Plan

## Overview

Land six small, independent governance changes plus one cross-spec rename:

1. `[simple]` task tier marker (template + plan command)
2. Stuck detection in `/gov:implement` (no new artifact — reads `git log` and `tasks.md`)
3. `--auto` flag on `/gov:implement`
4. "Skills" index section in the `AGENTS.md` project template
5. Cost-conscious cross-reference paragraph in the constitution
6. Concurrent-features note (single-target sessions; point at `git worktree` and platform isolation)
7. Cross-spec rename: 005's "skills" → "workflows" (reopens 005 per §cross-spec-impact)

The work is almost entirely prompt and prose — no application code, no new schemas, no events, no error codes, no data model. The risk is in two places: the cross-spec rename (broad blast radius across paths and prose) and getting the command-file parity right so `framework/commands/*.md` and `.claude/commands/gov/*.md` stay in sync via `scripts/gen-claude-commands.sh`.

## Technical Decisions

### `[simple]` marker placement

The marker is an inline tag on a task header, e.g.:

```markdown
## 1. Update registry trigger field [simple]
```

Header placement (vs. checkbox-line placement) chosen because:

- `tasks.md` already uses `## N. Title` headers as the canonical task identifier — the marker rides on the durable identifier, not on a sub-checkbox.
- Easy to spot when scanning the file and grep-friendly (`rg '\[simple\]'`).
- Survives the existing template structure with no schema change.

Conventions:

- A task header MAY end with `[simple]`. No marker = default tier (whatever the adopter's platform config maps to "standard").
- Only one tier defined: `[simple]`. `[complex]` was declined in the spec.
- Markers live on individual task headers, not on the file as a whole — different tasks in the same feature can have different tiers.

### `/gov:plan` proposes markers; user has final say

`/gov:plan` already produces `tasks.md` from the plan. New step: after generating the task list, scan each task and append `[simple]` to the header if it judges the task trivial (single small file, no logic, no schema change, no new behavior). The summary surfaced to the user before status transition lists which tasks were marked. The user adds, removes, or accepts markers before approving the transition.

This mirrors how the lightweight-track decision works: the agent proposes from heuristics, the user confirms.

### Stuck-detection algorithm

`/gov:implement` gains a setup-time check that runs before walking tasks:

1. Read `tasks.md` and identify the next incomplete task (first `- [ ]` checkbox group).
2. Run `git log --oneline -- specs/{feature}/tasks.md` to count commits that touched `tasks.md` since the spec entered `in-progress`.
3. If `git log` shows ≥ 3 commits on `tasks.md` since `in-progress` AND the same task is still the next incomplete one (i.e., no checkboxes have flipped to `[x]` between those commits for that task), surface the cycle to the user with the message: "Task {N} ({title}) has been touched in {count} prior implement runs without completing. Consider decomposing it into smaller subtasks before continuing." Pause and wait for user direction.
4. If `--auto` is set, the stuck-detection event still pauses (per spec: "auto mode does not power through cycles").

Threshold of 3 chosen as the smallest count that distinguishes routine multi-session work (1–2 invocations) from a cycle. The threshold is not configurable in v1 — keep the rule simple; if it proves wrong in practice, revisit.

The detection uses commit count on `tasks.md`, not on the full affected-files list, because:

- `tasks.md` is updated each time a task is touched (checkbox flips, retries, scope changes), so its commit history is the most reliable signal of "this task was the focus."
- Affected-file commit history is noisy (one commit can touch many files for unrelated reasons).
- It avoids the implementation having to parse the plan's affected-files table.

### `--auto` flag

`/gov:implement --auto` is a per-invocation flag. Argument parsing in the command instructions:

- The `Context` section already supports `$ARGUMENTS` as a feature override. Extend to recognize `--auto` as a known flag, stripped from the value before treating remaining text as the feature override.
- Default is unset (default off).

Behavior with `--auto`:

- Skip the per-task "prompt the user to commit and push changes" confirmation (current step 8 of "Walk through tasks"). The agent commits and pushes (or just commits — see below) on its own and proceeds.
- All other gates still fire and pause:
  - Phase transitions (`planned`→`in-progress`, `in-progress`→`done`)
  - Stuck-detection events
  - Discovering an out-of-bounds file write (current "If you need to modify files outside the plan's affected files list, notify the user…")
  - Spec edits, plan edits, or new tasks discovered mid-implement
  - Risky actions (destructive ops, secrets, force pushes — covered by the agent's safety rules)

Auto-mode commit policy: commit, do not push. Push is a hard-to-reverse, externally visible action; it stays gated even with `--auto`. The flag's job is to remove per-task confirmation friction, not to publish changes silently. The instructions document this explicitly.

`gov-session.json` is unchanged — autonomy is not session state.

### Cost-conscious cross-reference: location

Add a new short subsection `### Cost levers` immediately following the `### Business` principles list in `framework/constitution.md`. A subsection is preferred over inlining a paragraph under the bullet because:

- The `Cost-conscious` line is a one-line bullet in a structured list; a multi-sentence paragraph would break the list's visual rhythm.
- A named subsection (`### Cost levers`) is greppable and gets its own anchor for cross-references.

Content: one paragraph naming governance's existing levers (lightweight track, `[simple]` marker, stuck detection, default-off autonomy) and pointing at the adopter's platform tooling for runtime cost controls (Claude Code's `/cost`, Anthropic usage dashboard, Cursor's request limits). No commitment to a specific platform — just examples. Paragraph stays short (4–6 sentences) so it reads as guidance, not a manual.

### Concurrent-features note: location

Add a new short subsection `### Concurrent Features` under `## Pipeline Boundaries` in the constitution. A subsection there is preferred over a note in the AGENTS.md template because:

- "Single-target sessions" is a constitutional property of the pipeline, not an adopter-customizable choice — it belongs with the other pipeline rules.
- Pipeline Boundaries is already where serial-by-design rules live (the bullet list).
- AGENTS.md is the adopter's project-level operating doc; constitutional invariants are out of scope for it.

Content: one paragraph stating that `gov-session.json` holds a single target by design, and that concurrent work on independent features uses two independent sessions in two terminals, with isolation provided by `git worktree` and platform features (Claude Code's `isolation: "worktree"` agent parameter, Cursor's worktree integration, etc.). 3–5 sentences.

### Skills index in `AGENTS.md` template

Insert a new optional `## Skills` section after `## Project Structure` and before `## Code Style` in `framework/templates/project/agents.md`. The section is empty by default with an HTML-comment guide explaining when and how to populate it:

```markdown
## Skills

<!-- Optional. List skill files (Anthropic/Claude Code "skills" — context-loaded
     instruction packs) that augment AGENTS.md for specific task types. Leave
     empty if you don't decompose into skills.

     Example:
     | Skill | Activates on |
     | --- | --- |
     | `skills/security-review.md` | Code review on auth or session paths |
     | `skills/db-migration.md` | Editing migration files |

     Per-platform mapping (Claude Code skills, Cursor rules, etc.) is the
     adopter's call — governance defines the index pattern, not the location.
-->
```

Empty-by-default ensures backwards-compatibility for projects that don't decompose. The HTML comment teaches the pattern in place without bloating the rendered template.

### Cross-spec rename of 005's "skills" → "workflows"

Per §cross-spec-impact, 005 is reopened from `done` to `in-progress` because 010's adoption of "skills" terminology conflicts with 005's existing use. 010 owns the implementation; 005 records the new acceptance criterion as a signpost.

Term chosen: **workflows**. The .md files literally are workflow definitions (lint, test, format, migrate) and 005's existing template-naming convention is `{workflow}-{language}-{tool}.md` — so "workflow" is the unit the artifacts already describe themselves with. "Workflows" reads cleanly alongside the other one-word framework directories (`commands/`, `templates/`, `rules/`, `bootstrap/`) and avoids the redundant `templates/templates/` nesting that "command templates" would have produced. Initial plan-time term was "command templates" (per 010's spec); the term was revisited mid-implement and the spec updated to "workflows" once the redundant-nesting drawback became apparent.

Scope of the rename:

- **Directory rename and flatten:** `framework/skills/` → `framework/workflows/` (directory move) **and flattened** — `registry.json` and the nine workflow `.md` files now sit at the same level under `framework/workflows/`, no inner `templates/` directory. Flattening is included in this pass because the new top-level name "workflows" makes the inner `templates/` redundantly named ("workflows/templates" reads worse than "command-templates/templates").
- **Project-side path rename:** in `framework/bootstrap/govern.md` manifest and recommendation step, `skills/registry.json` (project-side) → `workflows/registry.json`. Adopters who already ran `/gov:govern` will have a `skills/` directory in their project — govern's update strategy will replace it on the next run; we do not write a migration tool. A one-line note is added to the rename task documenting that adopted projects should manually delete the old `skills/` directory after re-running `/gov:govern` (low cost — adopters with active workflow files will see them re-created under `workflows/`).
- **Scaffold destination rename:** `{config_dir}/commands/{project}/skills/` → `{config_dir}/commands/{project}/workflows/`. Affects `init.md` and `govern.md` instructions and the slash-command cleanup walk.
- **Prose:** update "skill" / "skills" terminology to "workflow" / "workflows" wherever it refers to 005's concept (NOT where it refers to 010's new concept of context-loaded instruction packs, e.g., in the new `AGENTS.md` Skills index section, the constitution Cost levers paragraph, or anywhere we describe Anthropic/Claude Code's skills feature). Files with prose to update:
  - `specs/005-workflows/spec.md` (title, body, acceptance criteria, resolved questions)
  - `specs/005-workflows/plan.md` (title, body, affected files table, trade-offs)
  - `specs/005-workflows/tasks.md` (title, body)
  - `specs/005-workflows/data-model.md` (terminology in schema description)
  - `framework/bootstrap/govern.md` (manifest row, recommendation step, all prose)
  - `.claude/commands/gov/init.md` (recommendation step, all prose — hand-maintained, generator skips)
  - `framework/bootstrap/configure/claude.md` ("Bash commands used by skills" comment label)
  - `specs/013-text-first-artifacts/plan.md` (one-row migration entry references the old spec dir)
  - `README.md` (any references)
- **Spec directory renamed.** `specs/005-skills-and-plugins/` → `specs/005-workflows/`. Initial plan-time decision was to leave the slug as a historical artifact, but a quick blast-radius check during implementation found only seven files reference the old slug, six of which are already on the touch list for prose updates — the seventh is a single-row mention in `specs/013-text-first-artifacts/plan.md`. Adopter projects do not reference 005's spec directory (they only consume the template files via govern), so the rename is internal-only. Number `005` stays; only the slug changes. Git detects the rename automatically when contents are unchanged on the move.
- **005's reopen path:**
  1. Add a new acceptance criterion to 005's spec: "Rename internal terminology from 'skills' to 'workflows' to free the term 'skills' for Anthropic-style context-loaded instruction packs (signpost: driven by 010-agent-autonomy)."
  2. Set 005's frontmatter `status` from `done` to `in-progress`.
  3. Add a task to `specs/005-workflows/tasks.md` for the rename, marked as carried out by 010's implementation.
  4. After 010's implementation completes, the user runs `/gov:implement` against 005 separately to verify the new AC and advance 005 back to `done`. 010 does not auto-advance 005's status — that follows the normal pipeline.

### Command file parity

Two of 010's deliverables touch `framework/commands/*.md`: `plan.md` (proposes `[simple]` markers) and `implement.md` (stuck detection + `--auto`). Per `CLAUDE.md`, never edit `.claude/commands/gov/*.md` directly — always edit the source under `framework/commands/` and run `./scripts/gen-claude-commands.sh`. The implementation runs the generator after editing the sources; tasks.md includes the generator step explicitly.

`init.md` is hand-maintained per `CLAUDE.md`; it is edited directly for the rename. The generator skips it.

`framework/bootstrap/configure/claude.md` is the source for `.claude/commands/gov/configure.md`; the generator writes the configure file from the bootstrap source. It is edited at the source location and the generator picks it up.

### What this plan does NOT do

Explicit non-goals to keep the scope tight:

- No new file format, no new artifact type, no schema change. Everything rides on existing markdown conventions.
- No platform-specific shipping (no Claude Code skills directory, no Cursor rules directory). Governance documents the pattern only.
- No execution log, no per-task token tracking, no budget files, no `[complex]` tier — all explicitly declined in the spec.
- No multi-target session, no `--feature` flag on commands, no worktree management — declined in the spec.
- No platform-specific install of the new workflow files (governance ships the registry + workflow definitions; init/govern scaffold them per agent).

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/templates/spec/tasks.md` | Modify | Document optional `[simple]` task header marker |
| `framework/commands/plan.md` | Modify | Add step proposing `[simple]` markers on trivial tasks |
| `framework/commands/implement.md` | Modify | Add stuck-detection step; accept `--auto` flag with documented gates |
| `framework/templates/project/agents.md` | Modify | Insert optional `## Skills` index section (empty by default) |
| `framework/constitution.md` | Modify | Add `### Cost levers` subsection and `### Concurrent Features` subsection |
| `framework/skills/` → `framework/workflows/` | Rename + flatten | Directory move and flatten — registry and nine workflow files sit at the same level (no inner `templates/`) |
| `framework/skills/registry.json` → `framework/workflows/registry.json` | Rename | (carried by directory rename) |
| `framework/skills/templates/*.md` → `framework/workflows/*.md` | Rename + move-up | Nine workflow files moved out of inner `templates/` directory |
| `framework/bootstrap/govern.md` | Modify | Update manifest path, recommendation-step paths, and all prose to "workflows" |
| `framework/bootstrap/configure/claude.md` | Modify | Replace "Bash commands used by skills" comment label with "workflows" |
| `.claude/commands/gov/init.md` | Modify | Hand-edit (generator skips): update recommendation step paths and prose |
| `specs/005-skills-and-plugins/` → `specs/005-workflows/` | Rename | Spec directory rename |
| `specs/005-workflows/spec.md` | Modify | Reopen to `in-progress`; add new AC + signpost; rename prose; update title |
| `specs/005-workflows/plan.md` | Modify | Update title, prose, affected-files table, trade-offs |
| `specs/005-workflows/tasks.md` | Modify | Update title, prose; add new task for the cross-spec rename |
| `specs/005-workflows/data-model.md` | Modify | Update terminology in schema description |
| `specs/013-text-first-artifacts/plan.md` | Modify | Update one-row migration entry that references the old spec directory |
| `README.md` | Modify | Update references to "skills" feature where they refer to 005's concept |
| `.claude/commands/gov/plan.md` | Regenerate | Output of `gen-claude-commands.sh` after editing source |
| `.claude/commands/gov/implement.md` | Regenerate | Output of `gen-claude-commands.sh` after editing source |
| `.claude/commands/gov/configure.md` | Regenerate | Output of `gen-claude-commands.sh` after editing source |

The directory rename is recorded as a single conceptual change but executes as `git mv` for the directory plus per-file follow-ups for any path strings inside the templates that reference the old path (none expected — templates use `{project}` and `{cli-config-dir}` placeholders, not absolute framework paths).

## Trade-offs

### Stuck-detection threshold of 3, not configurable

A fixed threshold keeps the rule simple and predictable. A configurable threshold would require a place to store the value (somewhere in `gov-session.json`? `AGENTS.md`?) and a way to surface it in command output. None of those carry their weight for v1. If the threshold proves wrong in practice, change the constant and reissue.

### Stuck detection reads `tasks.md` commits, not affected-files commits

Affected-files commit history is noisier and forces the implementation to parse the plan's affected-files table. `tasks.md` commit history is a clean signal because `tasks.md` is touched on every implement-pass that does work in the feature. The downside: a stuck task that doesn't trigger any `tasks.md` commits (e.g., the agent keeps trying without flipping checkboxes or even noting attempts) goes undetected. Acceptable — every realistic implement loop already touches `tasks.md` for the checkbox transitions.

### Auto-mode commits but does not push

Push is hard-to-reverse and externally visible. Keeping it gated preserves the spirit of `--auto` (skip per-task confirmation) without granting the agent silent publishing rights. Adopters who want full auto-publish can wrap `/gov:implement --auto` in a script that pushes after each session — they're opting into more risk explicitly.

### Cross-spec rename has broad blast radius

The 005 rename touches both 005's spec directory and three governance-owned files (`govern.md`, `init.md`, `configure.md`) plus templates. We accept the churn because:

- Leaving 005's "skills" term in place would create permanent terminology ambiguity ("skills" meaning two different things depending on which spec you're reading).
- The spec directory name stays, so cross-references from other specs and external docs continue to resolve.
- The rename happens once and is one PR.

### Skills index empty by default

An empty section with an HTML-comment guide adds slight visual weight to AGENTS.md compared to omitting the section entirely. Acceptable because the section teaches the pattern in place — adopters who don't decompose see exactly what skills would look like and can add them when ready, without re-reading governance docs.

### Concurrent-features note in constitution, not AGENTS.md template

Pipe-and-pull on this one: AGENTS.md is the adopter's customizable doc, but single-target sessions are a constitutional invariant of governance. Putting the note in the constitution communicates the invariant correctly; putting it in AGENTS.md would imply it's an adopter choice (it isn't). Adopters can still reference the constitutional section from their own docs.

## Open Questions Resolved

All open questions were resolved during clarification. See `spec.md` "Resolved Questions" section.

The plan-time decisions made above (locations for the cost-conscious paragraph and concurrent-features note; stuck-detection threshold of 3; auto-mode commit-but-don't-push; `[simple]` marker on task headers; spec directory name unchanged) are not new open questions — they are implementation-detail choices that the spec deliberately left to planning.
