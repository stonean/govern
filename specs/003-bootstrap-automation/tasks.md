# 003 — Bootstrap Automation Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create standard pipeline commands

- [ ] Create `.claude/commands/gov/` directory
- [ ] Copy all ten command templates from `commands/` into `.claude/commands/gov/`
- [ ] Replace every `{project}` with `gov` in all ten files
- [ ] Verify commands reference `.claude/gov-session.json` for session state

Done when: all ten `/gov:*` commands exist, all `{project}` placeholders are replaced with `gov`, and no template placeholders remain.

## 2. Create /gov:init command

- [ ] Write `.claude/commands/gov/init.md` with instructions for: collecting inputs (project name, path, description, primary languages), pre-flight directory check, scaffolding steps 1–11 from the spec
- [ ] Include gitignore language fetch from `https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore`
- [ ] Include next-steps output directing user to new session, `/{project}:setup`, and AGENTS.md/system.md

Done when: `/gov:init` exists with all scaffolding steps, input collection, pre-flight check, and next-steps display.

## 3. Final review and lint

- [ ] Run `markdownlint-cli2` on all files in `.claude/commands/gov/`
- [ ] Verify no `{project}` placeholders remain in standard commands (should all be `gov`)
- [ ] Verify init command uses `{project}` only where it refers to the new project being scaffolded
- [ ] Spot-check a few commands against their `commands/` templates to confirm accurate derivation
- [ ] Update spec status to `planned`

Done when: all eleven commands pass lint, placeholders are correct, and spec status is `planned`.
