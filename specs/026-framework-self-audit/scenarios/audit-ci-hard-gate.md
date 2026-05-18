---
section: "Follow-on scenarios"
---

# Audit-ci-hard-gate

## Context

[`/audit`](../../../framework/commands/audit.md) ships in v1 soft-launch mode: [`.github/workflows/markdown-only-pipeline.yml`](../../../.github/workflows/markdown-only-pipeline.yml) step (h) and [`.github/workflows/runtime-release.yml`](../../../.github/workflows/runtime-release.yml)'s `audit` job both run the audit with `continue-on-error: true` so v1 advisories from Families 4, 8, and 9 do not block PRs or releases while the framework drift is being resolved. Spec 026 §Q4 commits to flipping `continue-on-error: false` once the three families exit 0.

Origin: spec 026 Phase D CI integration, 2026-05-18. Captured via the inbox.

## Behavior

All three preconditions are already met as of commit `1aca768` ("fix(audit): resolve Family 4, 8, 9 findings — /audit now exits 0"):

1. **Family 4 clear.** `bash scripts/audit/placeholder-roundtrip.sh` exits 0. Resolution in 1aca768 included `/gov:` → `/{project}:` templating across `framework/commands/{review,analyze,implement}.md`, a `.claude/` → `{cli-config-dir}/` rewrite in `target.md`, and a file-scope `audit:ignore-placeholders:file` marker on the maintainer-only `framework/commands/audit.md`.
2. **Family 8 clear.** `bash scripts/audit/introducing-drift.sh` exits 0. Resolution in 1aca768 included mechanical substitutions in done specs 020/021/022, contextual rewrites in 011/014/017/026 (with historical signposts preserved), and a file-scope exemption on spec 023 (the introducing spec for the cataloged renames, where old names are first-class content).
3. **Family 9 clear.** `bash scripts/audit/primitive-promotion-candidates.sh` exits 0. Resolution in 1aca768 added 20 `audit:ignore-promotion` annotations across analyze, implement, plan, specify, status, and target for host-responsibility steps the runtime does not own.

Gate flip: remove `continue-on-error: true` from the audit step in [`.github/workflows/markdown-only-pipeline.yml`](../../../.github/workflows/markdown-only-pipeline.yml) and from the `audit` job in [`.github/workflows/runtime-release.yml`](../../../.github/workflows/runtime-release.yml). `/audit` becomes a hard PR gate and a hard release gate. The two CI files are the only edits; no script changes.

Verification: `bash scripts/audit/run-all.sh` exits 0 on `main` at HEAD (already confirmed); the next PR-check run passes with the audit step blocking-eligible; a deliberately-introduced finding (e.g., test commit adding a `/gov:` literal to a command source without the ignore marker) would be rejected.

## Edge Cases

- **A new advisory finding surfaces between flip and release** (regression). The hard gate catches it; the PR is blocked until the regression is fixed or the finding is annotated as a documented exception (the audit script must support a documented `audit:ignore-*` annotation form already used by Family 9's `audit:ignore-promotion` and Family 4's `audit:ignore-placeholders`).
- **A new family check ships** before flip. The new family inherits soft-launch (`continue-on-error: true`) by default; flip applies only to families whose v1 framework drift is resolved. Tracked per-family.
- **Flip happens out of order** (one of the three families regresses post-flip). The hard gate becomes a release blocker until repaired; this is intentional — the whole point of the flip is to make drift impossible to merge.

## Open Questions

- **Should the flip ship as one PR or three** (one per family clearance)? Default: one PR after all three clear, since the gate is binary. Resolve by inspection when the third family closes.

## Resolved Questions

*None.*
