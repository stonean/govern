---
section: "Follow-on scenarios"
---

# Extension-request-hygiene

## Context

From the 2026-07-11 review: the extension-request builders append the entire walker context after each typed prefix, so prior `llm:*` responses and the accumulated `findings` array ride along in every request — `performReview` pass N sees passes 1..N-1's full findings and raw responses (payload bloat, prompt-bias risk, cache-anchor erosion as the suffix grows per pass), and `writeCode` requests carry the whole session dump. Separately, `writeSpecBody` never populates its documented `template-path`/`template-content`/`feature-description` fields, and its `read_existing_section` helper prefers `plan.md` over `spec.md` regardless of which command is running — a `/gov:specify` re-run on a feature that has since gained a plan reads the wrong file's section. Finally, the two deferred extension points (`askClarifyQuestion`, `routeInboxItem`) have no typed request builders at all — any future procedure hitting them would get the raw context dump, the exact defect the `host-protocol-conformance` scenario just fixed for `assessSpecQuality`.

## Behavior

- Walker-internal keys (`llm:*` responses, `findings`, and other accumulator state) are filtered from the legacy-compat context merge in every extension request; the typed prefix fields lead and the per-request variable suffix stays minimal, preserving the writeCode cache-anchor contract.
- `writeSpecBody` requests populate `template-path`, `template-content`, and `feature-description` per the data model (or the data model is amended to drop fields that are deliberately unused — one of the two, no silent drift).
- `read_existing_section` selects the file by the running command (`/gov:plan` reads plan.md, `/gov:specify` reads spec.md), threaded explicitly rather than by fallback order.
- `askClarifyQuestion` and `routeInboxItem` each have a typed request builder and schema types, ready for the clarify/groom acceleration scenarios; unknown extension points remain an error rather than a dump.

## Edge Cases

- `performReview` requests keep the scope/rules fields threaded by task 46's result-threading — filtering removes accumulator keys, not primitive results the pass legitimately needs.
- The exec walker's internal `fired` binding for `process-waivers` (waiver-processing-order scenario) reads from walker context, not from request payloads, and is unaffected by request filtering.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
