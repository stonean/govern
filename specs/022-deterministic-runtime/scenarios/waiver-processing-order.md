---
section: "Follow-on scenarios"
---

# Waiver-processing-order

## Context

`/gov:review`'s procedure invokes `process-waivers` at step 3 to classify waivers "against the currently-firing findings" — but the five `performReview` passes that produce findings are steps 4–8, so at step 3 no findings exist. The primitive's `fired` argument defaults to empty, and its classification rule (`apply` requires the file to exist AND the rule to still fire) then classifies every waiver as `expire`; step 9's `write-review` "prunes expired waivers." On the exec path this mass-expires every valid waiver on every run; the parity test pins the same broken sequence. The primitive itself is correct — the procedure ordering and the exec-path binding of `fired` are the defects. Surfaced in the 2026-07-11 runtime review.

## Behavior

`process-waivers` runs after the review passes and before `write-review`, with the accumulated pass findings supplied as its `fired` input. On the exec surface the walker binds the accumulated findings to the primitive's `fired` argument; on the MCP/markdown-only paths the rewritten step order makes the LLM invoke it with the findings in hand. `write-review` receives waiver classifications computed against real findings, so a waiver expires only when its file is gone or its rule genuinely no longer fires. The markdown-only reference and the review sequence test reflect the corrected order.

## Edge Cases

- When `compute-review-scope` reports an empty scope and the procedure jumps straight to `write-review`, waivers are left untouched — no passes ran, so no expiry judgment is possible.
- A dimension-restricted run (`--security` etc.) classifies waivers against only the findings of the passes that ran; waivers anchored to skipped dimensions apply unchanged rather than expiring.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
