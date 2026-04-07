# Validate Fix Mode

**spec-ref:** 000-slash-commands — Command Set / validate

## Context

The `validate` command detects issues like unchecked checkboxes on completed items, but only reports them. The user must manually fix each one. For checkbox state mismatches — where the completion status is clear from context (e.g., spec is `done` but acceptance criteria are still `- [ ]`) — the fix is mechanical and safe to automate.

## Behavior

- The `validate` command accepts an optional `--fix` argument (or equivalent signal in the command args).
- In fix mode, after running all checks, validate automatically corrects fixable issues instead of just reporting them.
- Fixable issues:
  - Acceptance criteria checkboxes in specs with status `done` — update `- [ ]` to `- [x]`
  - Task checkboxes in `tasks.md` where all sub-items are marked `- [x]` — update the parent `- [ ]` to `- [x]`
  - Scenario-linked tasks where the spec status is `done` — update `- [ ]` to `- [x]`
- Non-fixable issues (report only, do not auto-correct):
  - Specs with status `in-progress` — cannot determine which criteria are truly met without verification
  - Missing artifacts (no plan, no tasks) — structural issues require human decisions
  - Lint failures — require manual correction
- Fix mode displays each correction it makes, showing the file, line, and change.
- Fix mode can target a single feature (using the session target) or scan all features when no target is set.
- After applying fixes, run `markdownlint-cli2` on modified files.

## Edge Cases

- If no fixable issues are found, report "No fixes needed" and exit cleanly.
- If validate is run without `--fix`, behavior is unchanged — report only.
