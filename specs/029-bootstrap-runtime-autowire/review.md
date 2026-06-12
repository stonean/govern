---
spec: 029-bootstrap-runtime-autowire
reviewed-at: 2026-06-11T22:15:00Z
reviewed-against: f85565f0fa2709d15df059208cb4c2b0ca1c07c7
diff-base: f85565f0fa2709d15df059208cb4c2b0ca1c07c7
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 029-bootstrap-runtime-autowire

## Summary

Scoped to the Task 11 (`runtime-probe-parity-audit` scenario) code: the new
`scripts/audit/runtime-probe-parity.sh` (Family 15) and the `scripts/audit/run-all.sh`
wiring edit. Stack is deterministic shell tooling per AGENTS.md §Tech Stack, so
the only applicable rule file is the cross-cutting `configuration-cross.md`; the
`-backend`/`-frontend` rule files govern web surfaces not present in this scope.

Posture: **clean, non-blocking.** Zero MUST and zero SHOULD violations across all
five passes. An initial reuse SHOULD (the §Agent Registry section-locator idiom
overlapped the sibling `installer-registry-parity.sh`) was resolved in this pass
by eliminating the awk section-extraction altogether: each probe literal occurs
only once in `govern.md`, so seed presence is now a whole-file fixed-string match
that mirrors the configure-side grep — simpler, with no shared sourced library
(which the self-contained audit suite avoids). The script's correctness was
re-confirmed both ways after the change — parity holds today (exit 0, no findings)
and an injected asymmetric entry produces a finding and exit 1.
`tech-stack-verified = true`, so the alignment precheck was skipped.

- security ✓ 0 MUST 0 SHOULD
- reuse ✓ 0 MUST 0 SHOULD
- quality ✓ 0 MUST 0 SHOULD (0 low-confidence)
- efficiency ✓ 0 MUST 0 SHOULD
- simplicity ✓ 0 MUST 0 SHOULD

blocking: **no**

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._ An initial reuse finding — the §Agent Registry section-locator awk idiom
overlapping `installer-registry-parity.sh` — was fixed in this pass by removing
the section-extraction entirely: seed presence now matches the probe literal
against the whole `govern.md`, since each literal is unique to its agent's
`settings_template` cell. The fix removed the duplication without introducing a
shared sourced library (which the self-contained audit suite avoids).

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no lines were added to `specs/inbox.md` in the review window (the groom
pass earlier removed an item; it added none)._

## Skipped passes

_None — all five passes ran. (Tech-stack alignment precheck skipped per
`[review] tech-stack-verified = true`.)_

## Pass notes

- **Security.** `runtime-probe-parity.sh` takes no untrusted input — every path
  (`framework/bootstrap/govern.md`, the three `configure/{key}.md`) and every
  probe literal is hardcoded. No network, no secrets, no `eval`, no unquoted
  expansion into a command; the probe is passed quoted to every
  `grep -qF -- "$probe"` call. No security-rule surface is present in scope.
- **Quality.** Logic verified correct: `set -uo pipefail` with every variable
  assigned before use; a `grep -q` non-match (exit 1) is handled by the `if`
  without `-e` aborting; the bidirectional both-or-neither parity check matches
  the scenario contract; the missing-source and missing-configure paths emit
  findings and exit non-zero. Seed and configure presence are both whole-file
  fixed-string `grep -qF` matches; this relies on each probe literal being unique
  within `govern.md` (one occurrence each, in the agent's `settings_template`
  cell) — true for the three current agents and documented in the script header
  and the scenario's "New agent rows" edge case.
- **Efficiency.** Each agent runs two small `grep` passes (govern.md + the
  configure file); no repeated extraction, no unbounded loops.
- **Simplicity.** Hardcoding three agents with explicit probe literals matches
  the sibling `installer-registry-parity.sh` and is simpler than iterating
  registry rows with per-agent grammar mapping. No premature abstraction.
- **Configuration (`configuration-cross.md`).** The hardcoded framework paths and
  probe literals are fixed references to specific framework artifacts, not
  operator-tunable values (CFG-CONST-003 out of scope); no environment variables
  are introduced (CFG-ENV-* out of scope).
