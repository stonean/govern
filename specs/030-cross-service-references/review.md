---
spec: 030-cross-service-references
scenario: referenced-service-spec-root
reviewed-at: 2026-06-30T15:51:10Z
reviewed-against: dae9e8214a597ad69599fd583f492620ab59647d
diff-base: 40c431759fb7306f34bbd9972e77fd1247be3bbd
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 030-cross-service-references (scenario: referenced-service-spec-root)

## Summary

Scoped to task 13 — the root-aware cross-service harvest (`scripts/gen-cross-service-refs.sh`, `scripts/lib/specs-root.sh`, and the shell test harness), the diff `40c4317..HEAD`. **0 MUST, 0 SHOULD — not blocking.** One low-confidence observation (a `svc_raw` tempfile leak on a `set -e` error path) surfaced and was **fixed inline** during the review (the `EXIT` trap now also removes `svc_raw`). The five passes ran against the backend + cross rule files (the detected stack; `[rules] surfaces` unset). Most backend rule files (`api`, `concurrency`, `observability`, `reliability`, `performance`) are genuinely N/A: the change is build-time bash/awk tooling run in pre-commit and CI, not a deployable service with HTTP, DB, async, or concurrency surfaces. The security input-handling family (`BE-INPUT-003/004`) was checked and **not** violated — every operator-controlled expansion is double-quoted, there is no `eval` or string-built shell command, and the referenced spec-root name is only string-compared in awk (never interpolated into a dynamic regex), on top of the `[A-Za-z0-9_-]` charset clamp in `specs_root_of`. The refactor into the shared `specs_root_of` helper removed duplication, and the two-tier matcher is faithful to the `referenced-service-spec-root` scenario's Resolved Q1 (verified: tests M/N/O plus independent fixtures for reachable-renamed, reachable-default, reachable-no-toml, not-checked-out, no-path, and absolute-path all behave as documented).

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None remaining._ The quality pass surfaced one low-confidence observation (confidence 70, below the blocking threshold, and covered by no loaded rule): `svc_raw` was created with `mktemp` and removed explicitly at the end of the registry-build block, but only `reg_file` was registered in `trap '…' EXIT`, so a failure under `set -euo pipefail` would leak one tempfile in `$TMPDIR`. It was **fixed inline** in this change — `svc_raw` is now initialised to `""` before the trap and the trap removes both tempfiles (`trap 'rm -f "$reg_file" "$svc_raw"' EXIT`, safe under `set -u`). Tests, shellcheck, and `--dry-run` re-verified green after the fix.

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no lines were added to `specs/inbox.md` in the review window._

## Skipped passes

_None — all five passes ran._

## Notes

- Rule selection: `[rules] surfaces` unset → detected stack (backend). Loaded `security-backend`, `configuration-cross`, `quality-cross`, `performance-backend`, `reliability-backend`, `api-backend`, `concurrency-backend`, `observability-backend`. Frontend files excluded by stack. No `[[review.disabled-rule-files]]`.
- `tech-stack-verified = true` in `.govern.toml` — the tech-stack alignment check was skipped per config.
