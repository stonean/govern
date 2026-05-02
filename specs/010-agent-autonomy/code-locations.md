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

## AC: Documentation note added (constitution or `AGENTS.md` template) directing users to `git worktree` and platform isolation for concurrent feature work

- `framework/constitution.md`

## AC: Changes respect command file parity (commands/ and .claude/commands/gov/)

- `.claude/commands/gov/configure.md`
- `.claude/commands/gov/implement.md`
- `.claude/commands/gov/plan.md`
- `framework/commands/implement.md`
- `framework/commands/plan.md`
- `scripts/gen-claude-commands.sh`

## AC: Changes respect govern file parity (govern/ variants stay in sync)

- `.claude/commands/gov/init.md`
- `framework/bootstrap/configure/claude.md`
- `framework/bootstrap/govern.md`

## AC: If the skills capability is delivered, 005's concept is renamed from "skills" to "workflows" (cross-spec impact: reopens 005 to `in-progress` per §cross-spec-impact)

- `.claude/commands/gov/init.md`
- `README.md`
- `framework/bootstrap/configure/claude.md`
- `framework/bootstrap/govern.md`
- `framework/workflows/format-go-gofmt.md`
- `framework/workflows/format-python-black.md`
- `framework/workflows/format-typescript-prettier.md`
- `framework/workflows/lint-go-golangci-lint.md`
- `framework/workflows/lint-python-ruff.md`
- `framework/workflows/lint-typescript-eslint.md`
- `framework/workflows/registry.json`
- `framework/workflows/test-go-gotest.md`
- `framework/workflows/test-python-pytest.md`
- `framework/workflows/test-typescript-vitest.md`
- `specs/005-workflows/code-locations.md`
- `specs/005-workflows/data-model.md`
- `specs/005-workflows/plan.md`
- `specs/005-workflows/spec.md`
- `specs/005-workflows/tasks.md`
- `specs/013-text-first-artifacts/plan.md`
