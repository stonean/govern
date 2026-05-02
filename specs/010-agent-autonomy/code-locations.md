# 010 — Agent Autonomy Code Locations

## AC: `tasks.md` template documents the optional `[simple]` inline marker convention (one tier; no marker = default)

- `framework/templates/spec/tasks.md`

## AC: `/gov:plan` command instructions include a step to propose `[simple]` markers on tasks the agent judges trivial

- `framework/commands/plan.md`

## AC: `/gov:implement` command instructions include a stuck-detection step that reads `git log` for affected paths and `tasks.md` checkbox state, surfaces cycles, and suggests decomposition

- `framework/commands/implement.md`
