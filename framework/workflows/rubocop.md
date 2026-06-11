---
description: Run RuboCop against the Ruby codebase and surface findings
---

# RuboCop

Run RuboCop against the Ruby codebase and surface findings.

## Instructions

1. Detect RuboCop config — look for `.rubocop.yml` or `.rubocop.toml` at the repository root. If none is found, RuboCop still runs with its default cops; warn the user but continue.
2. If the project has a `lint` task in `Rakefile`, `Makefile`, or `justfile`, mention it but prefer the direct `rubocop` invocation for predictability.
3. Run `bundle exec rubocop` from the repository root when a `Gemfile` is present, otherwise `rubocop`. If neither is on PATH, report `RuboCop is not installed` and stop — do not silently fall back to `ruby -c`.
4. Display the results. For each finding, show `file:line:col`, the cop that fired (e.g., `Style/StringLiterals`, `Lint/UselessAssignment`), and the message.
5. Summarize: count of offenses per cop department.
6. If offenses are present, treat the run as failed.

## What this workflow does NOT do

- Install RuboCop
- Auto-correct findings — the user runs `rubocop -a` (safe) or `rubocop -A` (unsafe) explicitly
- Modify the RuboCop config or regenerate `.rubocop_todo.yml`

## Common follow-ups

- Re-run with `-a` for safe auto-corrections after review
- Look up a cop at `https://docs.rubocop.org/rubocop/cops.html`
