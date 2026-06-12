---
spec: 015-tarball-fetch
reviewed-at: 2026-06-12T01:08:23Z
reviewed-against: 7e19b6925f862d60bae30c1b19f05d79d4030419
diff-base: f4985f455f4f39ecacea2aeea1dc65c125b1b3fb
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 015-tarball-fetch

## Summary

Clean. This is a drift-sync reopen, not a behavior change: the archive fetch already moved to the direct `codeload.github.com` endpoint under 029's `archive-fetch-direct-codeload` scenario (reviewed there); this run only updates 015's `§Source` prose, which still described the superseded `github.com/.../archive` form and the 302 redirect. The implementation under review is the `framework/bootstrap/govern.md` §Archive fetch step, already at codeload, plus the one-paragraph spec body edit. Per AGENTS.md Tech Stack, govern is text-first; the code-security rule set (`*-backend.md`/`*-frontend.md`/`*-cross.md`) has no surface in a markdown procedure or spec body. The corrected URL was byte-verified earlier to return the same `govern-main/` tarball with no redirect (HTTP 200, zero redirects). `tech-stack-verified = true`, so the alignment precheck was skipped. **0 MUST, 0 SHOULD — not blocking.**

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no additions to `specs/inbox.md` in the review window._

## Skipped passes

_None — all five passes ran. (Tech-stack alignment precheck skipped per `[review] tech-stack-verified = true`.)_
