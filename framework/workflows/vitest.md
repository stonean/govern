---
description: Run the Vitest suite for the TypeScript codebase
---

# Vitest

Run the Vitest suite for the TypeScript codebase.

## Instructions

1. Detect Vitest — look for `vitest.config.{js,ts,mjs}` or a `vitest` entry in `package.json` (`devDependencies` or scripts). If neither is found, report `Vitest is not configured for this project` and stop.
2. Check `package.json` for a `test` script that wraps Vitest. If present, prefer running it via the project's package manager (`npm test`, `pnpm test`, `yarn test`, `bun test`).
3. Otherwise, run `npx vitest run` (one-shot mode, not watch) from the repository root.
4. If the user specified a path or test name filter, append it to the command (e.g., `npx vitest run path/to/file.test.ts -t "case name"`).
5. Display the results. For each failing test, show the file path, test name, and the assertion failure or error.
6. Summarize: passed, failed, skipped counts and total duration.
7. If any tests failed, treat the run as failed.

## What this workflow does NOT do

- Run Vitest in watch mode — use `npx vitest` directly if you want watch
- Modify test files or fixtures
- Generate snapshots automatically without confirmation

## Common follow-ups

- Re-run with `--coverage` for coverage output
- Filter by test name with `-t "pattern"` to iterate on a single failure
