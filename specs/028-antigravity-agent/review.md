---
spec: 028-antigravity-agent
reviewed-at: 2026-06-10T02:46:51Z
reviewed-against: 072593cdf6334cb5ff8554f9c0db07fb38c27c79
diff-base: 7142035b0d2063ae525662f8c7822145d98028a3
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 028-antigravity-agent

## Summary

028 implements Antigravity support entirely in framework **markdown**
(`govern.md`, `configure/antigravity.md`, README, the 012 signpost) plus one
**Bash** generator change (`gen-configure-mcp.sh`) and a focused test — no
application code. Per `AGENTS.md`'s Tech Stack (markdown + bash, no application
runtime, no backend/frontend surface), the suffix-based rule selection keeps
only the cross-cutting rule file; the backend/frontend security, API,
accessibility, and performance rule files do not match the stack and are not
loaded. All five passes ran against the diff (`7142035..HEAD`): **no MUST or
SHOULD violations**. The new generator block reuses the existing
`splice`/`process` infrastructure rather than duplicating it; the layout-profile
generalization adds no dead branches or premature abstraction; the new Bash is
free of `eval` / `curl | sh` / unsafe redirects. **Not blocking** — the spec may
advance to `done`.

loading rule files: configuration-cross

## MUST violations (blocking)

None.

## SHOULD violations (advisory)

None.

## Low-confidence findings

None.

## Waived findings

None.

## Captured issues (pending /gov:groom)

None — `specs/inbox.md` is unchanged since `diff-base`.

## Skipped passes

None — all five passes (security, reuse, quality, efficiency, simplicity) ran.

## Notes (non-blocking, informational)

- **Rule applicability.** The govern rule files target adopter *application*
  code (auth, XSS, SQL, API contracts, a11y, perf). 028 changes framework
  markdown and a Bash generator, so only `configuration-cross.md` is
  stack-eligible, and its constants/env-var rules do not fire on a generator's
  script-local path variables or on documentation. This is the expected posture
  for a framework-docs feature, not a coverage gap.
- **Deliberate duplication (recorded, not a finding).** Antigravity mirrors
  `specs/rules/*.md` into `.agents/rules/` for native loading; both copies
  regenerate from `framework/rules/` on every `/govern` run. This is a clarified
  design decision (spec Resolved Questions + plan §Technical Decision), generator
  -maintained rather than hand-synced, so it is logged here for transparency
  rather than flagged by the reuse/simplicity passes.
- **Quality spot-checks (passed).** The README bootstrap `awk` correctly strips
  govern.md's single frontmatter block (prints only after the 2nd `---`); the
  Self-Update and Post-Write Integrity branches keep the Antigravity `SKILL.md`
  install consistent (no infinite "stale" loop); the 012 signpost link sits in a
  blockquote so `gen-spec-deps` skips it (no `012↔028` cycle).
