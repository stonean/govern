---
description: Format the Ruby codebase with rufo
---

# rufo

Format the Ruby codebase with rufo.

## Instructions

1. Detect rufo config — look for a `.rufo` file at the repository root. If none is found, rufo runs with its defaults; mention this but continue.
2. If the project has a `format` or `fmt` task in `Rakefile`, `Makefile`, or `justfile`, mention it but prefer the direct `rufo` invocation for predictability.
3. Run `rufo --check .` from the repository root by default. The `--check` flag lists files that differ from rufo's output without modifying them. If the user explicitly asked to write changes, run `rufo .` instead. Prefer `bundle exec rufo` when a `Gemfile` is present. If rufo is not on PATH, report `rufo is not installed` and stop.
4. Display the results. For check mode, list each file that needs formatting. For write mode, summarize the count of files changed.
5. If check mode lists any files, treat the run as failed.

## What this workflow does NOT do

- Reformat files without an explicit user request — default mode is `--check`
- Apply RuboCop style cops — rufo only normalizes whitespace and layout
- Install rufo

## Common follow-ups

- After review, re-run with `rufo .` to apply the formatting
- Run RuboCop's `Layout` cops alongside rufo if the project enforces stricter style
