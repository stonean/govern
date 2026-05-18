---
spec: 026-framework-self-audit
reviewed-at: 2026-05-18T01:30:00Z
reviewed-against: b160cfb
diff-base: e6ba1af55f022750bdafce220cef1e395b08e416
must-violations: 0
should-violations: 1
low-confidence: 1
skipped-passes: []
---

# Review — 026-framework-self-audit

## Summary

Reviewed `/audit` and its nine family check scripts (Phase B/C/D + the Family 9 pulled in mid-implement). Stack: text-first markdown + bash with one Rust touch to `gen-claude-commands.sh` (a new `--check` flag added during Phase A to close check-zero's gap). Loaded rule files: `configuration-cross.md` only — none of its CFG-* triggers fire against the diff (no env-var lookups, operator-tunable constants, or shared cross-module values introduced). All five passes ran. 0 MUST, 1 SHOULD (acknowledged boilerplate across the nine family scripts), 1 low-confidence (bash function variable scoping in primitive-promotion-candidates). `blocking: no`.

**Scope.** `framework/commands/audit.md` + generated mirror; `scripts/audit/` (10 scripts — check-zero, run-all, 8 family scripts + Family 9 — plus README); `scripts/gen-claude-commands.sh` (--check mode added); `.github/workflows/{markdown-only-pipeline,runtime-release}.yml` (v1 soft-launch advisory steps); spec + tasks edits.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

### SHOULD: REUSE-001 — boilerplate duplication across family scripts

- **File**: `scripts/audit/*.sh` (cross-doc-consistency, manifest-parity, registry-equivalence, placeholder-roundtrip, template-alignment, ssot-invariants, sibling-coupling, introducing-drift, primitive-promotion-candidates)
- **Rule**: AGENTS.md §design-principles ("Never design framework features that depend on human diligence") — by extension, "Don't Repeat Yourself" applied to the framework's own auditor.
- **Finding**: Each family script repeats the same boilerplate: `set -uo pipefail`, `ROOT=...`, `cd "$ROOT"`, `drift=0`, an `emit()` function with the same pipe-separated output shape. ~10 lines × 9 scripts = ~90 lines of structurally-identical code. A shared `scripts/audit/lib.sh` would let each family script `source` the boilerplate.
- **Auto-fixable**: yes (mechanical extraction)
- **Suggested fix**: Extract the shared boilerplate to `scripts/audit/lib.sh` exposing `audit_emit FAMILY LOCATION MESSAGE FIX` and the `ROOT/cd` setup; each family script `source`s the lib and uses `audit_emit` instead of the local `emit()`. Per-family contract (stdout findings, exit 0/1, read-only) stays unchanged.
- **Trade-off accepted for v1**: shared lib couples the nine scripts and reduces their stand-alone-invocation ergonomics (`bash scripts/audit/X.sh` would require lib.sh on relative path). Per-script standalone invocation is documented in `scripts/audit/README.md` as the per-family contract — useful for triaging a single check failure in CI. Extracting the lib is an option, not a blocker.

## Low-confidence findings

### LOW-CONFIDENCE: QUALITY-001 — bash function scope reliance in primitive-promotion-candidates

- **File**: `scripts/audit/primitive-promotion-candidates.sh:46-69` (the `flush_step` function)
- **Rule**: implicit best practice — functions should not silently rely on caller-scope mutable state.
- **Finding**: `flush_step()` reads and mutates seven caller-scope variables (`step_start_line`, `step_buffer`, `step_has_primitive`, `step_has_llm_marker`, `step_has_ignore`, plus emit's `drift`). Bash semantics make this work (functions inherit caller scope by default), but the pattern is fragile to refactor — adding a `local` keyword anywhere in the function would silently break it. **Confidence: 70%** (works as tested but brittle).
- **Auto-fixable**: no — refactoring to pass state explicitly is non-mechanical
- **Suggested fix**: Convert the per-file loop into a function that returns the step list, then iterate the returned list to apply flush logic. Or document the scope contract explicitly in the function's leading comment so future maintainers don't introduce `local` keywords. v2 can return a structured array via a temp file.

## Waived findings

_None._

## Skipped passes

_None._

## Pass notes

### Security

No security rules apply at the framework level for the diff in scope. The bash scripts are read-only file comparisons; no HTTP, authentication, persistence, or shell-out beyond running other framework scripts with `--dry-run`. The new `--check` mode added to `gen-claude-commands.sh` creates a tempfile via `mktemp` with the `trap 'rm -rf "$tmpdir"' EXIT` cleanup pattern — correct.

### Reuse

One SHOULD finding (REUSE-001 above) on boilerplate duplication across the nine family scripts. The repeated pattern is intentional per the per-family standalone-invocation contract; extracting a shared lib is a v2 option, not a v1 blocker. Other extractor helpers (`extract_claude_mcp`, `extract_auggie_mcp`, `extract_paths`, etc.) appear similar at a glance but have meaningful per-family differences in regex shape and field semantics — not deduplicable without an awk-level abstraction that would obscure the per-family intent.

### Quality

One low-confidence finding (QUALITY-001 above) on bash function variable scoping in `primitive-promotion-candidates.sh`'s `flush_step`. The pattern works as tested but is fragile to refactor. Bash 3.2 compatibility was the major concern across all scripts (macOS default shell) — verified by smoke-testing each script locally; no associative arrays, no `${var,,}` lowercasing, parallel scalars used instead.

`gen-claude-commands.sh`'s `--check` mode: walked the diff line-by-line. Tempfile handling is correct (trap cleanup); the orphan-detection loop catches files in DEST that no longer have a source. Unit-test equivalent (a manual drift-injection smoke test in Phase A task 2) confirmed correct fail/exit semantics.

### Efficiency

N/A. Each script iterates `framework/commands/*.md` × primitives or specs × files — both small bounded sets (15 command files × ~25 primitive names; ~27 specs). Runtime is sub-second per family.

### Simplicity

Family 6 (`ssot-invariants.sh`) is a stub that exits 0 always. Strict reading is overengineering ("a check that does nothing"); the counterpoint accepted at design-time is that the script _is_ the planning artifact — header documents the curated list and the promotion path. Per the spec body Family 6 description, real pattern-based detection requires concrete duplicate cases to write the patterns against, which v1 doesn't yet have. Not a finding under the v1 design intent.

The mid-implement pull of Family 9 added an AC and a check family without going through clarify (the standard back-edge for adding scope to an in-progress spec). User-directed expansion is captured in the Family 9 commit message; not a quality issue, but worth noting as a pattern: future scope expansions on in-progress specs should ideally go through clarify or be captured as scenarios. For v1 of `/audit`, the bundling-candidate check (Family 7) would have flagged exactly this kind of mid-implement mutation if the new AC were in a _different_ in-progress spec.
