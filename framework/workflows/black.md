---
description: Format the Python codebase with Black
---

# Black

Format the Python codebase with Black.

## Instructions

1. Detect Black config — look for `[tool.black]` in `pyproject.toml`. If none is found, Black runs with defaults; mention this but continue.
2. If the project has a `format` task in `Makefile`, `justfile`, or similar, mention it but prefer the direct `black` invocation for predictability.
3. Run `black --check .` from the repository root by default. If the user explicitly asked to write changes, run `black .` instead. If `black` is not on PATH, try `uvx black --check .`, then `python -m black --check .`. Report and stop if all three fail.
4. Honor any `extend-exclude` patterns in `pyproject.toml`.
5. Display the results. For `--check`, list files that would be reformatted. For the write mode, summarize the count of files changed.
6. If `--check` finds files that need formatting, treat the run as failed.

## What this workflow does NOT do

- Reformat files without an explicit user request — default mode is `--check`
- Override `extend-exclude`
- Install Black

## Common follow-ups

- After review, re-run without `--check` to apply the formatting
- Use `--diff` to preview the changes for a single file
