# 009 — Scenario Targeting Plan

## Overview

Extend the governance pipeline so individual scenarios can be targeted alongside features. This involves modifying the session file format, updating the target command to support scenario syntax and no-argument display, adding an Open Questions section to the scenario template, and updating four scenario-aware commands (question, clarify, status, implement) plus the scenario-creating command. All artifacts are markdown command files and a template — no application code, no persistence.

## Technical Decisions

### Session file gains optional `scenario` and `scenarioPath` fields

The existing session JSON structure is extended with two optional fields. When present, scenario-aware commands use `scenarioPath` to locate the primary artifact. When absent, behavior is unchanged. The `scenario` field holds the slug; `scenarioPath` holds the full relative path for direct file access without path construction.

### Target command supports three invocation modes

- No arguments: display current target (feature + scenario if set) and inform user how to change focus
- Feature only: existing behavior, clears any scenario field
- Feature/scenario-slug: sets both feature and scenario, validates the scenario file exists

Validation order for `{feature}/{scenario-slug}`: check feature exists → check `scenarios/` directory exists → check slug matches a file. Each failure has a distinct error message per the spec.

### Scenario template extended with Open Questions section

The `templates/scenario.md` file gets `## Open Questions` and `## Resolved Questions` sections appended — the same pattern specs use. Existing scenario files are not retroactively modified — they gain the sections when questions are added via the question command.

### Scenario-aware commands check session for scenario field

Each scenario-aware command (question, clarify, status, implement) reads the session file and checks for the `scenario`/`scenarioPath` fields. If present, the command operates on the scenario file instead of the spec. The branching is at the "target file detection" step — the rest of each command's logic operates on whichever file was selected.

### Clarify operates on scenario Open Questions when scenario-targeted

When a scenario is targeted, clarify resolves questions in the scenario file and enumerates scenario-specific edge cases. It does not touch the spec. When no scenario is targeted, existing behavior is unchanged and scenario-level questions are not surfaced.

### Scenario command sets target without confirmation

After creating a scenario file, the scenario command writes the session file with the new scenario as the target. No confirmation prompt — the user explicitly asked to create it.

### Feature-only commands ignore the scenario field

Specify, plan, and validate read the session file for the feature but disregard the scenario field entirely. No changes needed to these commands beyond documenting the behavior (which the spec already covers).

### Command file parity maintained via paired edits

Every command change is applied to both `commands/` (templates with `{project}` and `{cli-config-dir}` placeholders) and `.claude/commands/gov/` (governance-specific copies with `gov` and `.claude`). The govern file manifest already includes these command files, so adopting projects get the updates on next `/govern` run.

### Govern file parity maintained across variants

The scenario template's Open Questions section is already in the govern file manifest (`templates/scenario.md` → `specs/templates/scenario.md`). No new files need to be added to the manifest. The govern files themselves need no structural changes — only the command files they reference are updated.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `templates/scenario.md` | Modify | Add `## Open Questions` section |
| `commands/target.md` | Modify | Add no-argument display, scenario targeting syntax, validation, error messages |
| `commands/scenario.md` | Modify | Set session target after scenario creation (no confirmation) |
| `commands/question.md` | Verify | Already scenario-aware — verify target file detection covers scenario path |
| `commands/clarify.md` | Modify | Add scenario-targeted behavior (resolve scenario open questions, skip spec questions) |
| `commands/status.md` | Modify | Add scenario-level detail display when scenario is targeted |
| `commands/implement.md` | Modify | Add scenario context loading when scenario is targeted |
| `.claude/commands/gov/target.md` | Modify | Re-derive from updated `commands/target.md` |
| `.claude/commands/gov/scenario.md` | Modify | Re-derive from updated `commands/scenario.md` |
| `.claude/commands/gov/question.md` | Verify | Re-derive from updated `commands/question.md` if changed |
| `.claude/commands/gov/clarify.md` | Modify | Re-derive from updated `commands/clarify.md` |
| `.claude/commands/gov/status.md` | Modify | Re-derive from updated `commands/status.md` |
| `.claude/commands/gov/implement.md` | Modify | Re-derive from updated `commands/implement.md` |

## Trade-offs

### Considered: storing scenario status in the session file

Rejected. Scenarios do not have their own status field (per 006-bug-workflow). Resolution is determined by whether open questions remain. Adding status to the session would create a parallel tracking mechanism that conflicts with the established convention.

### Considered: making all commands scenario-aware

Rejected. Specify, plan, and validate are inherently feature-level operations. Making them scenario-aware would add complexity without benefit — there is nothing to specify, plan, or validate at the scenario level that isn't already covered by the feature-level operation.

### Considered: requiring target confirmation when scenario is set

Rejected per user decision during clarification. Over-prompting for confirmations is annoying. The scenario command creates what the user asked for and targets it — the next step is obvious.

### Considered: surfacing scenario questions during feature-level clarify

Rejected per user decision during clarification. Spec-level and scenario-level questions are independent concerns. Mixing them would blur scope boundaries and add noise.

## Open Questions Resolved

All open questions were resolved during clarification. See spec.md — Resolved Questions section.
