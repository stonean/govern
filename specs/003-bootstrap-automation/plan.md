# 003 — Bootstrap Automation Plan

## Overview

Create eleven slash commands in `.claude/commands/gov/`: ten standard pipeline commands copied from `commands/` templates with `{project}` replaced by `gov`, plus one governance-specific `init.md` that scaffolds new projects. The standard commands give governance the same pipeline enforcement as adopting projects. The init command automates the manual bootstrap process from the README.

## Technical Decisions

### Standard commands are literal copies with placeholder replacement

Each of the ten command templates in `commands/` is copied to `.claude/commands/gov/` with every occurrence of `{project}` replaced by `gov`. No other modifications. This ensures governance dogfoods the exact same commands adopting projects use. If a command template is updated later, the governance copy should be re-derived from the template.

### Init command is governance-specific

The init command does not exist in `commands/` — it is unique to the governance repo. It lives alongside the standard commands at `.claude/commands/gov/init.md` and is invoked as `/gov:init`. It orchestrates file copying, placeholder replacement, and gitignore fetching as a single slash command prompt.

### Placeholder replacement in init uses find-and-replace

The init command instructs the agent to replace `{project}` with the user-provided project name in all copied files. This is the same approach adopting projects already use — literal string replacement, not a templating engine.

### Gitignore language patterns fetched at runtime

The init command fetches `.gitignore` patterns from `https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore` for each primary language. The fetched content is appended below the governance template's entries, separated by a comment header identifying the language. If a fetch fails, the command reports the failure and continues with the minimal template.

### Session file path

Standard commands reference `.claude/gov-session.json` for session state, consistent with the `{project}-session.json` pattern from spec 000.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `.claude/commands/gov/about.md` | Create | Pipeline overview (from template) |
| `.claude/commands/gov/target.md` | Create | Set session target (from template) |
| `.claude/commands/gov/status.md` | Create | Spec dashboard (from template) |
| `.claude/commands/gov/setup.md` | Create | Configure permissions (from template) |
| `.claude/commands/gov/specify.md` | Create | Create new spec (from template) |
| `.claude/commands/gov/clarify.md` | Create | Resolve questions (from template) |
| `.claude/commands/gov/plan.md` | Create | Create plan and tasks (from template) |
| `.claude/commands/gov/implement.md` | Create | Execute tasks (from template) |
| `.claude/commands/gov/validate.md` | Create | Audit artifacts (from template) |
| `.claude/commands/gov/next.md` | Create | Auto-advance phase (from template) |
| `.claude/commands/gov/init.md` | Create | Scaffold new projects (governance-specific) |

## Trade-offs

### Considered: generating standard commands dynamically from templates

Rejected. Copying with replacement is simple and explicit. The governance repo has a fixed project name (`gov`) that never changes. Dynamic generation adds complexity for no benefit.

### Considered: a single `/gov:work` command instead of ten standard commands

Rejected. Governance should use the same commands as adopting projects. A custom command would diverge from the dogfooding principle and miss bugs or friction in the templates.

### Considered: skipping setup command for governance

Rejected. Even though governance already has a settings file, the setup command is part of the standard set. Keeping it maintains parity with adopting projects.

## Open Questions Resolved

All open questions were resolved during clarification. See spec.md Resolved Questions section.
