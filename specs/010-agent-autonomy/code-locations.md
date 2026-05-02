# 010 — Agent Autonomy Code Locations

## AC: `tasks.md` template documents the optional `[simple]` inline marker convention (one tier; no marker = default)

- `framework/templates/spec/tasks.md`

## AC: `/gov:plan` command instructions include a step to propose `[simple]` markers on tasks the agent judges trivial

- `framework/commands/plan.md`

## AC: `/gov:implement` command instructions include a stuck-detection step that reads `git log` for affected paths and `tasks.md` checkbox state, surfaces cycles, and suggests decomposition

- `framework/commands/implement.md`

## AC: `/gov:implement` command accepts an `--auto` flag that skips per-task confirmations within a phase, with the documented gates (phase transitions, stuck detection, spec/plan edits, mid-implement discovery, risky actions) still firing

- `framework/commands/implement.md`

## AC: `AGENTS.md` project template gains an optional "Skills" index section listing available skill files and their activation conditions (empty by default)

- `framework/templates/project/agents.md`

## AC: Constitution `## Guiding Principles` → `Cost-conscious` (or a new dedicated subsection) gains a cross-reference paragraph naming governance's cost levers (lightweight track, `[simple]` marker, stuck detection, default-off autonomy) and pointing at platform tooling for runtime controls

- `framework/constitution.md`
