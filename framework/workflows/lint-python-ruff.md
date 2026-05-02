# Ruff

Run Ruff against the Python codebase and surface findings.

## Instructions

1. Detect Ruff config — look for `[tool.ruff]` in `pyproject.toml`, `ruff.toml`, or `.ruff.toml`. If none is found, Ruff still runs with defaults; warn the user but continue.
2. If the project has a `lint` or `check` task in `Makefile`, `justfile`, `tox.ini`, or similar, mention it but prefer the direct `ruff check` invocation for predictability.
3. Run `ruff check .` from the repository root. If `ruff` is not on PATH, try `uvx ruff check .`, then `python -m ruff check .`. Report and stop if all three fail.
4. Display the results. Ruff already groups findings clearly — show the output as-is. For each finding, the user should see `file:line:col`, the rule code (e.g., `F401`), and the message.
5. Summarize: count of errors per rule.
6. If findings are present, treat the run as failed.

## What this workflow does NOT do

- Auto-fix findings — the user runs `ruff check --fix` explicitly if desired
- Modify the Ruff config
- Run Ruff format (that is the formatter workflow)

## Common follow-ups

- Re-run with `--fix` after review
- Look up an unfamiliar rule code at `https://docs.astral.sh/ruff/rules/`
