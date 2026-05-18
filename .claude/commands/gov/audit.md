---
description: Audit framework artifacts for cross-doc, cross-manifest, cross-registry drift. Maintainer-only.
---

# Audit

Audit `govern`'s own framework artifacts for the kinds of drift `/gov:analyze` is not scoped to catch. Maintainer-only — adopters never invoke this command. Runs without a session target.

## Purpose

`/gov:analyze` audits a single feature spec's artifacts against each other (frontmatter, plan, tasks, data-model, dependencies, rule citations). Its contract is bounded to one feature directory plus declared dependencies, so it cannot see drift across the framework: README spec-status table vs. per-spec frontmatter, `configure/claude.md` vs. `configure/auggie.md` canonical permission set, workflow registry vs. workflow files, etc.

`/audit` fills that gap. It loads no rule files — its checks are about *framework consistency*, not spec quality. Each check family produces structured findings on stdout. Exit code is binary: `0` when no findings, `1` when any finding is present. CI uses the exit code as a release gate.

See [spec 026](../../specs/026-framework-self-audit/spec.md) for the design and the [026 plan](../../specs/026-framework-self-audit/plan.md) for the eight family check families and the check-zero precondition pass.

## Scope Boundaries

- Read-only against the framework's cross-cutting artifacts. Do NOT modify any file.
- No session target required; the command operates on the framework as a whole.
- Reference: §drift-prevention, §design-principles. The constitution is loaded by other pipeline commands; `/audit` re-reads it independently because it runs without `/gov:target`.

## Instructions

> **For agent runtimes**: backticked primitive names in this section map to MCP tools the optional [gvrn runtime](https://crates.io/crates/gvrn) exposes under bare `<primitive>` names (e.g., `run-generator`). Hosts wrap them with a server-name prefix taken from `.mcp.json` (Claude: `mcp__gvrn__run-generator`; Auggie: `mcp:gvrn:run-generator`). When the server is registered for your session, **call the corresponding tool** for each step listed below — that is the deterministic path. Otherwise, walk the markdown-only path below to produce the same result.

1. Invoke `run-generator` against `scripts/audit/run-all.sh` — the orchestrator that runs the check-zero precondition pass followed by the eight family check scripts. The script emits findings to stdout under per-family headers and exits 0 (no findings) or 1 (any family produced findings).

Otherwise, walk the markdown-only path below.

## Markdown-only reference

When the runtime is not on `PATH`, walk the same nine scripts directly. Each prints findings to stdout and exits `0` (no findings) or `1` (findings present). Aggregate across all families; `/audit`'s exit code is the logical OR.

1. Run `scripts/audit/check-zero.sh` — generator/lint precondition. Halt on findings; do not run family checks against known-stale generator output.
2. Run `scripts/audit/cross-doc-consistency.sh` (Family 1).
3. Run `scripts/audit/manifest-parity.sh` (Family 2).
4. Run `scripts/audit/registry-equivalence.sh` (Family 3).
5. Run `scripts/audit/placeholder-roundtrip.sh` (Family 4).
6. Run `scripts/audit/template-alignment.sh` (Family 5).
7. Run `scripts/audit/ssot-invariants.sh` (Family 6).
8. Run `scripts/audit/sibling-coupling.sh` (Family 7).
9. Run `scripts/audit/introducing-drift.sh` (Family 8).
10. Run `scripts/audit/primitive-promotion-candidates.sh` (Family 9).

## Boundary with `/gov:analyze`

| Concern | Owner |
| --- | --- |
| Spec's frontmatter parses; required fields present | `/gov:analyze` |
| Dependency graph well-formed for one feature | `/gov:analyze` |
| Rule IDs cited in spec exist in loaded rule files | `/gov:analyze` |
| Plan / tasks / data-model present per status tier | `/gov:analyze` |
| Cross-doc claim consistency (README vs frontmatter, etc.) | `/audit` |
| Manifest / permission / registry parity | `/audit` |
| Sibling-spec coupling (bundling candidates) | `/audit` |
| Introducing-spec body drift (current-tense prose around renamed names) | `/audit` |

Rule of thumb: `/gov:analyze` reads within one spec's directory plus its declared dependencies; `/audit` reads across the framework's cross-cutting artifacts. The two never duplicate a check.

## Output

`/audit` writes findings to stdout in a maintainer-friendly format: family header, then one finding per row with location / message / suggested-fix columns. Exit code `0` when no findings; `1` when any finding is present. No `audit.md` artifact is produced — the audit runs interactively, not stored as a per-run report.
