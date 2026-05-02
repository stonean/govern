# Black

Format the Python codebase with Black.

## Instructions

1. Detect Black config — look for `[tool.black]` in `pyproject.toml`. If none is found, Black runs with defaults; mention this but continue.
2. Run `black --check .` from the repository root by default. If the user explicitly asked to write changes, run `black .` instead. If `black` is not on PATH, try `uvx black --check .`, then `python -m black --check .`. Report and stop if all three fail.
3. Honor any `extend-exclude` patterns in `pyproject.toml`.
4. Display the results. For `--check`, list files that would be reformatted. For the write mode, summarize the count of files changed.
5. If `--check` finds files that need formatting, treat the run as failed.

## What this workflow does NOT do

- Reformat files without an explicit user request — default mode is `--check`
- Override `extend-exclude`
- Install Black

## Common follow-ups

- After review, re-run without `--check` to apply the formatting
- Use `--diff` to preview the changes for a single file
