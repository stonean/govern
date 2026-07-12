---
spec: 022-deterministic-runtime
reviewed-at: 2026-07-12T19:41:19Z
reviewed-against: 4031ab945d11f432e974eddf77a2eec2aeca621a
diff-base: 5f25ebe3fc8801199506705c6c32f13b57f6f41a
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 2
skipped-passes: []
---

# Review — 022-deterministic-runtime

## Summary

Review of 022-deterministic-runtime. The scope spans the full feature; the analytical passes focused on the delta since the prior clean review (dbf91df) — tasks 65–68 and the groom-added scenarios. Across security, reuse, quality, efficiency, and simplicity, no MUST or SHOULD violations were found. The changes are hardening/correctness: task 66 strengthens BE-INPUT-007 SSRF defense by pinning the connection to the validated address (closing the DNS-rebinding TOCTOU); task 68 hardens `review:` frontmatter against structure-injection/retyping by quoting known waiver fields; task 65 adds a schema-derived allowlist validation at the MCP boundary; task 67 aligns the inbox write path with its comment-aware read path. rmcp 2.x is source-compatible with certificate validation intact (rustls default). 0 MUST / 0 SHOULD / 0 low-confidence. blocking: no.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

*None.*

## Waived findings

*None.*

## Captured issues

- Architectural exploration: re-frame the runtime's LLM extension points (`writeCode`, `writeSpecBody`, `assessSpecQuality`, future multi-turn points) as named Anthropic-style Skills the host loads at the seam, rather than ad-hoc JSON envelopes. Potential benefits: structural cache anchoring (Skills are a natural cache boundary); third-party hosts integrate against an emerging Skills protocol instead of govern-specific JSON; `constitution-excerpts` becomes a bundled resource rather than an inline string array. Speculative — depends on Anthropic's Skills protocol stabilizing and is a larger redesign than 022's current scope. Revisit after the writeCode payload-bundling scenario on 022 ships and the cache-anchored shape proves out the pattern. Surfaced 2026-05-19 during runtime-improvement investigation. **On hold per user 2026-07-11.**
- [ ] Runtime `SkipScanner` (`runtime/src/primitives/mod.rs`) scans lines for `<!--`/`-->` and code-fence delimiters without exempting inline-code spans, so any tasks.md/spec/scenario prose mentioning a backticked `<!--` with no closing `-->` later on the same line opens a comment region for every comment-aware parser (`read-tasks`/`mark-task` task parsing, `dashboard` open-question counts, the section walkers), silently hiding all following structure. Hit concretely during 022 tasks 66-68: task 67's done-when embedded a backticked `<!--` and hid task 68 from `read-tasks`/`mark-task` until reworded (worked around in tasks.md and the append-inbox-comment-aware-write scenario, not fixed at the parser). Hardening: `SkipScanner.skip` should ignore comment/fence delimiters inside inline-code spans (skip backtick-delimited runs before matching). Surfaced 2026-07-12 during 022 tasks 66-68 implementation.

## Skipped passes

*None.*
