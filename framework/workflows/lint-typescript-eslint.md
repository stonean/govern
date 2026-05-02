# ESLint

Run ESLint against the TypeScript codebase and surface findings.

## Instructions

1. Detect the project's ESLint config — look for `eslint.config.{js,ts,mjs,cjs}` (flat config) or `.eslintrc*` (legacy). If none is found, report `No ESLint config found` and stop.
2. Check `package.json` for an `eslint` or `lint` script. If present, prefer running it via `npm run {script}` (or the project's package manager — `pnpm run`, `yarn`, `bun run`).
3. Otherwise, run `npx eslint .` from the repository root.
4. Display the results. For each finding, show `file:line:col`, the rule id, and the message. Group by file.
5. Summarize: count of errors, warnings, files with findings.
6. If errors are present, treat the run as failed.

## What this workflow does NOT do

- Auto-fix findings — the user runs `--fix` explicitly if desired
- Modify the ESLint config
- Install ESLint or its plugins

## Common follow-ups

- Re-run with `--fix` after the user reviews the findings
- Open the rule documentation for an unfamiliar finding
