---
spec: 022-deterministic-runtime
reviewed-at: 2026-07-12T21:07:13Z
reviewed-against: c617ba86606b5d0df89beee88ea8017e8d946695
diff-base: 5f25ebe3fc8801199506705c6c32f13b57f6f41a
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 1
skipped-passes: []
---

# Review — 022-deterministic-runtime

## Summary

Review of 022-deterministic-runtime, refreshed after task 69 (SkipScanner inline-code exemption) and its perf follow-up. The scope spans the full feature; the analytical passes focused on the delta since the prior review (4031ab9) — the changes to primitives/mod.rs. Across security, reuse, quality, efficiency, and simplicity, no MUST or SHOULD violations were found. The new inline-code-span parsing (inline_code_spans / find_outside_code) matches on ASCII delimiters and slices only on char boundaries, so it cannot panic on multibyte content; it introduces no external-input, injection, or resource-exhaustion surface (inputs are repo markdown, bounded); the span scan is short-circuited when a line carries no delimiter; and the helpers are shared, not duplicated. Earlier delta (tasks 65–68, rmcp 2.x) remains clean. 0 MUST / 0 SHOULD / 0 low-confidence. blocking: no.

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

## Skipped passes

*None.*
