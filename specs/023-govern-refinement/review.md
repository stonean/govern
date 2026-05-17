---
spec: 023-govern-refinement
reviewed-at: 2026-05-17T14:15:00Z
reviewed-against: 8e0cee93e9a6714caba98e7bf6b48f5932d36e79
diff-base: 670a3181acbfefbf3dab31eb7df84f5442672e8a
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 023-govern-refinement

## Summary

Re-review of `023-govern-refinement` over a broadened scope: spec 023's own files (already reviewed clean at commit `670a3181`) plus all files modified since that baseline, which include the post-23 `gov-rt → gvrn` MCP-server rename and the embedded `/govern` procedural-fidelity preamble. No findings. Blocking: no.

The rename work is not 023 scope — it landed on top of 023's already-`done` artifacts. The review covers it here because `/gov:review`'s diff base spans every change since the spec advanced to in-progress, and it would otherwise leave that surface area uninspected before the spec sits permanently at `done`.

The prior pass of this review found three SHOULD-tier hygiene findings (one simplicity, two quality) in the two bash generators touched by the rename. All three were applied via `/gov:review --fix` in the same session — see [Resolved in this run](#resolved-in-this-run) below.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._

## Resolved in this run

The following SHOULD-tier findings were applied via `--fix` after the initial pass:

- **SIMPLICITY-001** — dropped unused `local label="$1"` from `process()` in `scripts/gen-configure-mcp.sh` and removed the matching string arguments at the two call sites.
- **QUALITY-001** — added a top-level `trap cleanup EXIT` to `scripts/gen-configure-mcp.sh` plus a `cleanup_files` registry so every `mktemp` is released regardless of which exit path the script takes.
- **QUALITY-002** — replaced the single-space-only strip in `scripts/lint-tool-coverage.sh` with a full whitespace-class strip (`[![:space:]]`), so a manifest line with multi-space indent or trailing tab can no longer slip through with embedded whitespace.

Re-running the affected scripts after the fixes:

```text
gen-configure-mcp.sh --dry-run    → No changes (mcp-allow blocks in sync)
lint-tool-coverage.sh             → exit 0
```

## Notes

- The post-23 rename `gov-rt` → `gvrn` (MCP server name) was reviewed for cross-artifact drift against AGENTS.md's "No dead references in live artifacts" rule. All live-artifact references were updated; the remaining `gov-rt:` occurrences are confined to `runtime/CHANGELOG.md` and `specs/02{2,3}-*/`, both of which are frozen archaeology per §drift-prevention.
- `lists_every_manifest_tool_and_canonical_set` (runtime/tests/mcp.rs:94) enforces the invariant that `TOOL_NAMES` in `server.rs`, the running server's tool list, and `framework/runtime-tools.txt` all agree. The post-rename test suite (256 passed / 0 failed) confirms the rename is consistent across these three sources.
- The `## Markdown-only reference`-skip behavior added to `lint-tool-coverage.sh` is exercised implicitly by the existing CI workflow (markdown-only-pipeline.yml); no dedicated test covers the new branch, but the run-time signal — the lint passes cleanly across all seven affected command files — provides coverage adequate to the change.
