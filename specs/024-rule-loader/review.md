---
spec: 024-rule-loader
reviewed-at: 2026-05-17T20:00:00Z
reviewed-against: 041b8ccc1fa655b76608fd7c65ec5781c28eeda3
diff-base: 041b8ccc1fa655b76608fd7c65ec5781c28eeda3
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 024-rule-loader

## Summary

The implementation is framework-side only — one new bash lint (`scripts/lint-rule-filenames.sh`), one CI workflow step, a rule file rename (`configuration.md` → `configuration-cross.md`), and edits to the constitution, four command files, the bootstrap doc, and `specs/README.md`. No application code, no runtime change, no security surface. The shipped rule set (security/api/accessibility/performance) targets `src/` patterns that do not exist in this change. Zero MUST, zero SHOULD, zero low-confidence findings. Blocking: no.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

*None.*

## Waived findings

*None.*

## Skipped passes

*None — all five passes ran.*

## Pass summary

| Pass | MUST | SHOULD | Notes |
| --- | --- | --- | --- |
| Security | 0 | 0 | New code (`scripts/lint-rule-filenames.sh`) reads filenames from the repo; no auth, no secrets, no user input, no DB. Workflow YAML invokes the lint with no parameters. No surface for the loaded backend/frontend security rules to fire against. |
| Reuse | 0 | 0 | `scripts/lint-rule-filenames.sh` follows the established shape of `scripts/lint-frontmatter.sh` (header preamble, `set -euo pipefail`, `--help` block, `$ROOT` derivation, `shopt -s nullglob`, error-count + exit) — house style, not duplicated logic. The plan explicitly chose to duplicate the suffix-discovery prose in `framework/commands/review.md` and `framework/commands/analyze.md` rather than introduce a shared file; this is a documented trade-off honoring §text-first-artifacts. |
| Quality | 0 | 0 | Lint script: closed-suffix `case` statement covers all four classes (3 valid + unrecognized); error message names every valid suffix; exit codes match house convention (0 clean / 1 violations / 2 usage). The CI step is correctly placed in the lint phase (no rust toolchain required). The `configuration.md` → `configuration-cross.md` rename was performed via `git mv` (rename detection preserved); rule IDs (`CFG-CONST-*`, `CFG-ENV-*`) are content-anchored and verified unchanged. The bootstrap migration mirrors the spec 023 `spec-and-plan.md` precedent and uses the same prompt/abort/notice shape, preserving idempotency. |
| Efficiency | 0 | 0 | Lint script iterates the rule files directory once (handful of entries); single `basename` + `case` per file — O(n) on rule-file count, runs in milliseconds. No N+1 or unbounded loops. |
| Simplicity | 0 | 0 | The change adds no new abstraction, no flags, no config keys. The lint script is ~50 lines of straightforward bash; the workflow edit is a single step; the prose edits replace one hardcoded list with one suffix derivation. No premature abstraction; no dead branches; no operator-tunable values introduced. |

## Acceptance criteria audit

All nine acceptance criteria are satisfied by the landed changes:

| # | Criterion | Status |
| --- | --- | --- |
| 1 | Closed suffix policy documented in `constitution.md` §rules and enforced by `scripts/lint-rule-filenames.sh` in CI | ✓ — new `#### Filename suffix` subsection; lint passes |
| 2 | `/gov:review` selection rewritten to suffix-based discovery; hardcoded names no longer drive selection | ✓ — §Behavior step 5 and §Load rules rewritten |
| 3 | Three new files (`api-backend.md`, `accessibility-frontend.md`, `performance-frontend.md`) load automatically | ✓ — all have closed suffixes; discovery picks them up |
| 4 | `configuration.md` → `configuration-cross.md` rename; rule IDs unchanged; §Past Renames updated | ✓ — `git mv` preserved rename; 11 rule IDs intact; entry added |
| 5 | `AGENTS.md` fallback narrowed to adopter-local rule files outside `framework/rules/` | ✓ — §Notes for adopters rewritten |
| 6 | Unrecognized-suffix rule files load + emit one-line stdout warning | ✓ — described in §Behavior step 5 of `review.md` and §Rules of `analyze.md` |
| 7 | `/gov:review` emits `loading rule files: <list>` notice | ✓ — described in §Behavior step 5 |
| 8 | §Notes for adopters rewritten to describe the new contract | ✓ |
| 9 | `/gov:analyze` uses the shared discovery; no stack filtering | ✓ — §Rules rewritten to load every discovered file unconditionally |

## Output

```text
/gov:review — 024-rule-loader

  security    ✓ 0 MUST   0 SHOULD
  reuse       ✓ 0 MUST   0 SHOULD
  quality     ✓ 0 MUST   0 SHOULD
  efficiency  ✓ 0 MUST   0 SHOULD
  simplicity  ✓ 0 MUST   0 SHOULD

  blocking: no
  report:   specs/024-rule-loader/review.md
```

Exit code: `0`.
