<!--
  The Affected Files section below is a BULLET LIST, not a table: that is
  the shape `compute-review-scope`'s read_plan_affected currently parses
  (see specs/inbox.md — the table/list parser mismatch). Rendering it as a
  table would make plan-affected empty and collapse the review scope, so
  keep it a list until that inbox item is resolved.
-->
# 007 — Review Fixture Plan

## Technical Decisions

A single-file scope keeps the golden deterministic: the plan's Affected Files
names one in-repo source file, so `compute-review-scope` resolves a stable,
non-empty scope regardless of the single-commit fixture's git history.

## Affected Files

- `src/reviewed.rs` — the single in-scope source file the passes review.
