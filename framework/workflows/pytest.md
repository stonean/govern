---
description: Run the pytest suite for the Python codebase
---

# pytest

Run the pytest suite for the Python codebase.

## Instructions

1. Detect pytest — look for `[tool.pytest.ini_options]` in `pyproject.toml`, a `pytest.ini`, `tox.ini`, or `setup.cfg` with pytest config. If none is found, pytest still runs with defaults; warn the user but continue.
2. Check the project for a `test` task in `Makefile`, `justfile`, or similar. Mention it but prefer direct `pytest` invocation for predictability.
3. Run `pytest` from the repository root. If `pytest` is not on PATH, try `uvx pytest`, then `python -m pytest`. Report and stop if all three fail.
4. If the user specified a path, test name, or marker filter, append it (e.g., `pytest path/to/test_file.py::test_case` or `pytest -k "pattern"`).
5. Display the results. For each failure, show the test id, the assertion failure or traceback, and the captured output if relevant.
6. Summarize: passed, failed, skipped, errored counts and total duration.
7. If any tests failed or errored, treat the run as failed.

## What this workflow does NOT do

- Modify test files or fixtures
- Generate `.snapshot` or `pytest-snapshot` artifacts without confirmation
- Install pytest or plugins

## Common follow-ups

- Re-run with `-x` to stop at the first failure
- Re-run with `--lf` to repeat only the last-failed tests
- Add `--cov` if `pytest-cov` is configured
