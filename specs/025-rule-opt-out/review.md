---
spec: 025-rule-opt-out
reviewed-at: 2026-05-17T20:35:00Z
reviewed-against: 95aacc64728bdefed5f658db5fa8814f6c039e11
diff-base: 95aacc64728bdefed5f658db5fa8814f6c039e11
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 025-rule-opt-out

## Summary

The implementation is framework-side, markdown-only — edits to `framework/commands/review.md` (§Inputs, §Behavior step 5, §Notes for adopters), `framework/commands/status.md` (a fourth below-the-table callout), `framework/bootstrap/govern.md` (a commented-out TOML example), `framework/constitution.md` (a 3-sentence pointer paragraph under §rules), plus the spec/plan/tasks for 025. No application code, no runtime change, no new security surface. The new `[[review.disabled-rule-files]]` key is read once per `/gov:review` run, parsed into a small operator-authored entry list, and dispatched through five mutually-exclusive branches (drop+notice, no-op notice, unknown warning, malformed warning, duplicate warning). Two files were dropped from the spec's original Affected list during planning (`framework/commands/analyze.md` does not read `.govern.toml` at all, structurally satisfying AC7; `scripts/lint-govern-toml.sh` does not exist and is out of scope per the Q3 resolution). Zero MUST, zero SHOULD, zero low-confidence findings. Blocking: no.

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
| Security | 0 | 0 | The mechanism could in principle let an adopter disable `security-backend.md` or `security-frontend.md`; AC8 documents this as uniform-by-design with the mandatory `reason` (≥ 16 codepoints, trimmed) as the audit trail. Carve-outs were rejected during clarify — they would require a hardcoded "real security" list that drifts. No security surface in the implementation itself: the new key is bounded (basename + reason), every parse path routes invalid input to a warn-and-skip branch, and warnings never taint the exit code (so a clean `blocking: false` result unambiguously means "no MUST rule fired" rather than "your `.govern.toml` is silently broken"). |
| Reuse | 0 | 0 | The malformed/duplicate warning branches in `framework/commands/review.md` §Behavior step 5 explicitly mirror the §Malformed and duplicate waivers pattern (lines 358–370) — same skip-with-warning posture, same first-applies-on-duplicate posture, same "operator-authored state, not framework-collected garbage" rationale. This is deliberate echoing across surrounding prose, not duplicated logic. The `/gov:status` callout shape (`disabled rule files: {N} (.govern.toml) — {basenames}`) follows the existing blocked-specs / recovery-state callout shape. No new abstraction is introduced; no shared utility is needed (markdown-only feature). |
| Quality | 0 | 0 | Absence handling is consistent with house convention — `framework/commands/status.md` explicitly enumerates the four "skip the line" cases (`.govern.toml` absent, no `[review]` section, no `disabled-rule-files` array, empty array); `framework/commands/review.md` step 5 inherits the same implicit-absence convention used by the surrounding step 4 `tech-stack-verified` read and the §waivers per-run processing (neither enumerates "what if the key is missing"). Branch ordering is unambiguous: the Malformed branch explicitly says "Skip the entry (no file is dropped)," so a malformed entry never reaches the file-resolution dispatch. Path-form values in `file` fall through cleanly to the Unknown warning branch per AC1 (no basename match in `framework/rules/`). The reason-length contract is codepoint-counted (AC1) and the notice collapses whitespace (AC2), so TOML multi-line strings render single-line in stdout. |
| Efficiency | 0 | 0 | One read of `.govern.toml` per `/gov:review` run; one linear pass over the disabled-files array (operator-authored, bounded by Q4's frequency analysis at 0–3 entries for most projects); one stdout line emitted per entry. No N+1, no unbounded iteration, no nested scans. |
| Simplicity | 0 | 0 | One new TOML key; one new branch in step 5 with five mutually-exclusive cases; one new callout in `/gov:status`; a 3-sentence pointer paragraph in the constitution; one commented-out example block in `bootstrap/govern.md`. The `--disable` CLI shortcut was explicitly rejected during clarify (Q4) and moved to Non-goals — no new flag surface. Two files were dropped from the spec's original Affected list after planning verified the work wasn't actually needed there. No operator-tunable values are introduced in code (the 16-codepoint threshold lives in markdown spec prose with a single source-of-truth in `framework/commands/review.md` §Inputs; CFG-CONST-003's "business logic" domain does not apply). |

## Acceptance criteria audit

All nine acceptance criteria are satisfied by the landed changes:

| # | Criterion | Status |
| --- | --- | --- |
| 1 | `[[review.disabled-rule-files]]` schema (array-of-tables; basename `file`; codepoint-counted reason) | ✓ — `framework/commands/review.md` §Inputs Config bullet; `framework/bootstrap/govern.md` example block; path-form values fall to unknown-warning per AC3 |
| 2 | `/gov:review` skips listed files + one-line notice; whitespace collapse; no-op notice variant | ✓ — `framework/commands/review.md` §Behavior step 5 Drop+notice and No-op notice branches |
| 3 | Unknown file → one-line warning, not fatal | ✓ — `framework/commands/review.md` §Behavior step 5 Unknown warning branch |
| 4 | Malformed → warn+skip; warnings do NOT taint the exit code | ✓ — `framework/commands/review.md` §Behavior step 5 Malformed warning branch + the post-list exit-code-invariance paragraph |
| 5 | Duplicate → warn + first applies | ✓ — `framework/commands/review.md` §Behavior step 5 Duplicate warning branch |
| 6 | `/gov:status` surfaces the disabled list | ✓ — `framework/commands/status.md` step 6 fourth callout |
| 7 | `/gov:analyze` does NOT error on the new key | ✓ — `framework/commands/analyze.md` does not read `.govern.toml` at all (grep verified during planning); structurally satisfied without an edit |
| 8 | Uniform across all rule files; no security carve-out | ✓ — `framework/commands/review.md` §Behavior step 5 has no conditional branches on rule filename |
| 9 | Documentation in this spec body + `framework/commands/review.md` (per spec 020 precedent; not retro-added to spec 019) | ✓ — spec body AC9 wording; `framework/commands/review.md` §Inputs/§Behavior/§Notes; `framework/constitution.md` §rules pointer paragraph; spec 019 untouched |

## Output

```text
/gov:review — 025-rule-opt-out

  security    ✓ 0 MUST   0 SHOULD
  reuse       ✓ 0 MUST   0 SHOULD
  quality     ✓ 0 MUST   0 SHOULD
  efficiency  ✓ 0 MUST   0 SHOULD
  simplicity  ✓ 0 MUST   0 SHOULD

  blocking: no
  report:   specs/025-rule-opt-out/review.md
```

Exit code: `0`.
