---
spec: 029-bootstrap-runtime-autowire
reviewed-at: 2026-06-12T00:56:52Z
reviewed-against: 6f7504192c2dfa8c136fa4d7b9d3d9045a1963d6
diff-base: 0741350562a49cee39c8eb12d403fa275ae758c4
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 029-bootstrap-runtime-autowire

## Summary

Clean. This run reviews the two follow-on scenarios added on the reopened spec — `project-inputs-asked-once` (persist project inputs in `.govern.toml`'s `[project]` table; collect after the Pre-flight Phase, reading existing values back and prompting only for what is missing) and `archive-fetch-direct-codeload` (fetch the archive from the direct `codeload.github.com` endpoint to avoid the 302 redirect). The implementation is entirely in the `framework/bootstrap/govern.md` markdown procedure. Per AGENTS.md Tech Stack, govern is a text-first framework; the rule files (`*-backend.md`, `*-frontend.md`, `*-cross.md`) target application code — SQL injection, XSS, auth, N+1 queries — which has no surface in a bootstrap procedure document, so the security and efficiency passes have nothing to flag. The quality and simplicity passes assessed the procedure logic directly: the read→resolve→prompt→persist flow is internally consistent, the persistence preserves every other `.govern.toml` section (matching the existing `[host]`/`[migrations]`/`[workflows]` write convention), the `[project].name` ↔ `host.project` relationship is well-defined (single source of truth plus a derived runtime view, kept in sync by `/govern`), and the codeload URL was byte-verified to yield the same `govern-main/` tarball. `tech-stack-verified = true` in `.govern.toml`, so the alignment check was skipped. **0 MUST, 0 SHOULD — not blocking.**

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

_None — all five passes ran. (The tech-stack alignment precheck was skipped per `[review] tech-stack-verified = true`; this is a precheck, not one of the five review passes.)_
