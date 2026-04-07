# 007 — Govern Command Plan

## Overview

Create one govern command file per supported CLI (`govern/govern.md` for Claude Code, `govern/govern-auggie.md` for Auggie) in the `govern/` directory. Each file is a self-contained prompt that instructs the AI agent to fetch templates from GitHub, scaffold governance files, resolve placeholders, and display next steps. The command templates in `commands/` gain a `{cli-config-dir}` placeholder so a single template set serves all CLIs.

## Technical Decisions

### One govern file per CLI, not a single file with CLI detection

Each CLI variant is a separate markdown file in the `govern/` directory. The CLI-specific values (config directory, session file path, setup behavior) are hardcoded in each variant. This avoids runtime CLI detection logic in a prompt — the user's choice of which file to curl determines the target.

Adding a new CLI means creating a new govern variant. The governance core (constitution, templates, command templates) does not change.

### Command templates use `{cli-config-dir}` placeholder

The existing command templates in `commands/` reference `.claude/` paths for session state and settings. To support multiple CLIs, these references change to `{cli-config-dir}` — a placeholder resolved during adoption alongside `{project}`.

For governance's own commands in `.claude/commands/gov/`, the placeholder is already resolved to `.claude`. Adopting projects get it resolved to whichever CLI they chose.

Affected references in command templates:

- `.claude/{project}-session.json` → `{cli-config-dir}/{project}-session.json`
- `.claude/settings.local.json` → `{cli-config-dir}/settings.local.json`
- `.claude/commands/{project}/` → `{cli-config-dir}/commands/{project}/`

### Setup command skipped for Auggie

The Auggie govern variant omits the `/{project}:setup` step from post-scaffolding output. Auggie uses global permissions (`~/.augment/settings.json`) rather than per-project settings. This is documented as a future consideration in `specs/spec.md`.

The setup command template itself is still copied — it just won't work correctly for Auggie until per-project permissions are supported. The govern variant's next-steps output skips mentioning it.

### Govern files live in `govern/`, not in templates/

The govern files are the distribution entry point — users curl them directly. Placing them in the `govern/` directory keeps them organized while maintaining a clean URL: `https://raw.githubusercontent.com/stonean/govern/main/govern/govern.md`. They are not templates to be copied into projects; they are one-time bootstrap commands. The directory also accommodates future CLI variants without cluttering the repo root.

### File manifest is a markdown table in the govern file

Each govern variant contains a hardcoded manifest table mapping source paths to destination paths with conflict strategies. This is readable, self-documenting, and easy to update. The agent reads the table and executes the fetches.

### Gitignore language patterns fetched from GitHub

Same approach as the init command: fetch from `https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore` and append below a comment header. Failures are reported but don't abort.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `govern/govern.md` | Create | Claude Code govern command |
| `govern/govern-auggie.md` | Create | Auggie govern command |
| `commands/about.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/specify.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/clarify.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/plan.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/implement.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/status.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/next.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/target.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/setup.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/validate.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/scenario.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/triage.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` |
| `commands/commit-push.md` | Modify | Replace `.claude/` with `{cli-config-dir}/` (if it references CLI paths) |
| `.claude/commands/gov/*.md` | Modify | Re-derive from updated templates with `{cli-config-dir}` → `.claude` |
| `.claude/commands/gov/init.md` | Modify | Update to resolve `{cli-config-dir}` during scaffolding |
| `specs/007-govern-workflow/spec.md` | Modify | Update status to `done` |

## Trade-offs

### Considered: single govern file with CLI prompt

A single `govern.md` that asks "Which CLI are you using?" at runtime. Rejected because the user already chose their CLI by installing the file into a specific directory. A prompt adds friction and the file can't know which directory it was placed in.

### Considered: keeping `.claude/` hardcoded in command templates, substituting only in govern

The govern command could do `.claude/` → `.augment/` replacement at copy time without changing the source templates. Rejected because it's fragile — any new `.claude/` reference in a template would silently break Auggie adoption. The `{cli-config-dir}` placeholder makes the abstraction explicit and grep-able.

### Considered: separate command template directories per CLI

Maintaining `commands/` for Claude Code and `commands-auggie/` for Auggie. Rejected — the commands are identical except for path prefixes. Duplicating them would create drift. The placeholder approach keeps one source of truth.

## Open Questions Resolved

All open questions were resolved during clarification. See spec.md Open Questions section.
