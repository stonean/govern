---
spec-ref: "000-slash-commands — Command Set"
tags: [commands, ux]
---

# Command Autocomplete Summary

## Context

Claude Code's built-in slash commands (e.g., `/help`, `/clear`, `/model`) show a short summary next to each name in the autocomplete window, so the user can scan options without selecting one to see what it does. Governance's commands currently render with name only — the autocomplete shows `/gov:specify`, `/gov:plan`, etc., but not what each one does. Users have to remember the command set or run `/gov:about` to learn it.

The framework command sources under `framework/commands/*.md` start with a top-level `# Name` heading and a one-line description on the next line. The generated `.claude/commands/gov/*.md` files inherit that shape. Neither uses YAML frontmatter.

## Behavior

Each governance command surfaces a one-line summary in the Claude Code autocomplete window, matching the visual treatment of built-in commands.

The investigation has two parts:

1. **Confirm the platform mechanism.** Verify that Claude Code reads a `description:` field from YAML frontmatter at the top of a slash command markdown file and renders it in autocomplete. (This is the de facto convention for custom commands; confirm against current Claude Code behavior before committing to it.) If `description:` is not the field, identify the correct one.

2. **Apply it framework-wide.** Add a `description:` field to every command source under `framework/commands/*.md` (and `framework/bootstrap/configure/claude.md`). The description should be a single short sentence — concrete enough to disambiguate from sibling commands, short enough to fit alongside the command name. Re-run `scripts/gen-claude-commands.sh` so the generated `.claude/commands/gov/*.md` files inherit the field. Manually update the hand-maintained `.claude/commands/gov/init.md` to match.

The autocomplete summaries should match the existing one-line descriptions already present below each command's `# Name` heading (e.g., `Specify` → "Create a new feature spec."), so there is one source of truth per command.

## Edge Cases

- If Claude Code surfaces only the first N characters of `description:` in autocomplete, keep each summary under that limit. Verify by inspection in the Claude Code UI.
- If the platform later changes the field name or rendering rules, the framework's command-source layout — one file per command with frontmatter on top — keeps the migration mechanical (edit the field name across files; re-run the generator).
- If a description duplicates the `# Name` heading verbatim, it adds no information. Each command's `description:` should be substantively different from its bare name.

## Open Questions

_All open questions resolved. See Resolved Questions below._

## Resolved Questions

- **Frontmatter key for Claude Code autocomplete** — `description:`. Confirmed by adding `description:` frontmatter to a single command (`init.md`) and observing it surface in the Claude Code session-bound skills list ("gov:init: Scaffold a new project with governance files, templates, and commands.") while sibling commands without the field still rendered as bare names ("gov:specify: Specify"). After regenerating the rest, every gov command's description appeared in the same list. The same field name is reflected throughout Claude Code documentation as the canonical key for custom slash command summaries.
- **Frontmatter-only vs. body fallback** — frontmatter only, in practice. Commands without `description:` rendered using the `# Title` heading as a fallback label, not the first body line. So the explicit `description:` field is required to get a meaningful summary; relying on the body produces only the bare command name.
