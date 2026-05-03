---
description: Format the TypeScript codebase with Prettier
---

# Prettier

Format the TypeScript codebase with Prettier.

## Instructions

1. Detect Prettier — look for a config file (`.prettierrc*`, `prettier.config.{js,ts,mjs,cjs}`) or a `prettier` key in `package.json`. If none is found, report `No Prettier config found` and stop.
2. Check `package.json` for a `format` script that wraps Prettier. If present, prefer running it via the project's package manager.
3. Otherwise, run `npx prettier --check .` from the repository root by default. If the user explicitly asked to write changes, run `npx prettier --write .` instead.
4. Honor `.prettierignore` — Prettier respects it automatically, no extra flag needed.
5. Display the results. For `--check`, list files that would be reformatted. For `--write`, summarize the count of files changed.
6. If `--check` finds files that need formatting, treat the run as failed.

## What this workflow does NOT do

- Reformat files without an explicit user request — default mode is `--check`
- Override `.prettierignore`
- Install Prettier or any plugins

## Common follow-ups

- After review, re-run with `--write` to apply the formatting
- Diff a single file: `npx prettier --check path/to/file.ts`
