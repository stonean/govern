---
title: "003-bootstrap-automation — tasks"
---

# 003 — Bootstrap Automation Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create standard pipeline commands

- [x] Create `.claude/commands/gov/` directory
- [x] Copy all ten command templates from `commands/` into `.claude/commands/gov/`
- [x] Replace every `{project}` with `gov` in all ten files
- [x] Verify commands reference `.govern.session.toml` for session state (was `.claude/gov-session.json` pre-0.10.0)

Done when: all ten `/gov:*` commands exist, all `{project}` placeholders are replaced with `gov`, and no template placeholders remain.

## 2. Create /gov:init command

- [x] Write `.claude/commands/gov/init.md` with instructions for: collecting inputs (project name, path, description, primary languages), pre-flight directory check, scaffolding steps 1–11 from the spec
- [x] Include gitignore language fetch from `https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore`
- [x] Include next-steps output directing user to new session, `/{project}:setup`, and AGENTS.md/system.md

Done when: `/gov:init` exists with all scaffolding steps, input collection, pre-flight check, and next-steps display.

## 3. Final review and lint

- [x] Run `npx markdownlint-cli2` on all files in `.claude/commands/gov/`
- [x] Verify no `{project}` placeholders remain in standard commands (should all be `gov`)
- [x] Verify init command uses `{project}` only where it refers to the new project being scaffolded
- [x] Spot-check a few commands against their `commands/` templates to confirm accurate derivation
- [x] Update spec status to `planned`

Done when: all eleven commands pass lint, placeholders are correct, and spec status is `planned`.

## 4. Installer script (`install.sh`)

- [x] `install.sh` resolves the agent (positional arg, default `claude`) and places the bootstrap at the 012-registry path for claude, auggie, and antigravity
- [x] Antigravity placement wraps the body as a `name: govern` skill; the fetch is tempfile-guarded (`mktemp` + `EXIT` trap) and idempotent
- [x] README Quick start and per-agent installs reduced to a single `curl … | sh` line each
- [x] Verified end to end against the live `govern.md` for all three agents
- [x] Resolve the installer↔registry parity open question — enforced by `scripts/audit/installer-registry-parity.sh` (audit Family 14)

Done when: the one-line `curl … | sh` installer places the bootstrap for every supported agent per `scenarios/curl-sh-installer.md`, and the README installs are reduced to one line per agent.
