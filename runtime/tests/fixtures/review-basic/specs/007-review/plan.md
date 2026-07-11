# 007 — Review Fixture Plan

## Technical Decisions

A single-file scope keeps the golden deterministic: the plan's Affected Files
names one in-repo source file, so `compute-review-scope` resolves a stable,
non-empty scope regardless of the single-commit fixture's git history.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `src/reviewed.rs` | Edit | The single in-scope source file the passes review. |
