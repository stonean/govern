---
status: draft
dependencies: [008-security-rules, 016-cross-cutting-rules, 017-derive-dont-ask, 024-rule-loader, 033-rule-surface-setting]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 036 — Cross-cutting code-quality rules

Introduce `framework/rules/quality-cross.md`, a cross-cutting code-quality rule set whose inaugural rule `QUAL-STUB-001` forbids silent stubs: partial or unimplemented code paths that pass through without failing loudly. The file follows the canonical rule schema, uses the `-cross.md` suffix (loaded for every stack), and is wired into the `/govern` Shared Files manifest. This is a rule-introducing feature spec — the same path [008-security-rules](../008-security-rules/spec.md) used to introduce the first rule files.

## Motivation

Silent stubs ship indistinguishably from working implementations — a no-op rate-limiter, an always-allow permission check, a publisher that drops events on the floor — and the failure mode surfaces only when the missing behavior is needed, which is precisely when the system is under stress. A partial implementation that returns a zero value, a no-op middleware that returns `next` unchanged, a handler that returns early without an error, or a method that returns `nil, nil` is not visibly distinct from a correct one until production exercises the gap.

None of the existing rule files cover this discipline. `api-backend.md` `BE-SCHEMA-002` carries a "fail loudly at build time" rationale, but it is API-schema-specific; the stub-discipline is genuinely cross-cutting — it applies to background workers, domain methods, frontend stores, middleware, anything whose surrounding contract implies the path performs work. The constitution treats code-quality discipline as part of the governance-recognized cross-cutting class (§rules, the "etc." that sits alongside security, performance, concurrency, observability, accessibility, audit/compliance, data handling), so the concern belongs at the rules tier rather than re-litigated per feature.

**Motivating incident.** In the anvil adopter project, 008-rate-limiting Task 4 left a passthrough stub in `RateLimit` enabled-mode that would have silently disabled rate limiting in production had Task 5 been skipped. The team manually added a panic with a "not implemented" message until Task 5 landed — a discipline the framework should make automatic rather than depending on a contributor remembering to add the guard.

## Rule set scope

`quality-cross.md` uses the **cross** surface (`-cross.md` suffix → loaded for every stack via the rule-loader's suffix discovery, [024-rule-loader](../024-rule-loader/spec.md)). Its ID grammar is `QUAL-{CATEGORY}-{NNN}`, mirroring the domain-prefixed cross-file convention `configuration-cross.md` established (`CFG-{CONST|ENV}-{NNN}`, [017-derive-dont-ask](../017-derive-dont-ask/spec.md)). The inaugural category is `STUB`; the file header declares the `QUAL` category abbreviation(s) per the per-file category-declaration policy framed in [016-cross-cutting-rules](../016-cross-cutting-rules/spec.md).

Inaugural rule:

- **`QUAL-STUB-001`** — *partial or unimplemented code paths MUST fail loudly (panic / explicit error / failing test fixture) rather than silently pass through; stubs that return zero values, no-op middleware that returns `next` unchanged, handlers that return early without an error, and methods that return `nil, nil` are forbidden when the surrounding contract implies the path performs work.*

### Boundaries (cross-reference, do not duplicate)

- **Build-time schema fail-loud** is already `api-backend.md` `BE-SCHEMA-002` — `quality-cross.md` cites it for the build-time case rather than restating it; `QUAL-STUB-001` governs the broader runtime/contract case across all surfaces.

## Acceptance Criteria

- [ ] `framework/rules/quality-cross.md` exists, ends in the `-cross.md` suffix, and follows the canonical rule schema (`### {ID}` headings; Statement / Rationale / Verification; RFC 2119 language) per [008-security-rules](../008-security-rules/spec.md)'s data-model.
- [ ] `QUAL-STUB-001` is present with a Statement (RFC 2119), a Rationale, and a Verification clause.
- [ ] Every rule ID uses the `QUAL-{CATEGORY}-{NNN}` format with the `QUAL` prefix and category abbreviation disjoint from the existing `BE-`/`FE-`/`CFG-` namespaces; `scripts/lint-rule-ids.sh` passes.
- [ ] The file header declares the `QUAL` category abbreviation(s) per the per-file category-declaration policy ([016-cross-cutting-rules](../016-cross-cutting-rules/spec.md)).
- [ ] The Verification clause is expressed as a check `/gov:review` can apply to code (silent passthrough vs. loud failure), and is scoped so legitimately-empty implementations are not flagged (see Open Questions).
- [ ] Rules whose surface overlaps an existing rule cite it rather than restating it (`BE-SCHEMA-002` for the build-time fail-loud case).
- [ ] The file is added to the `/govern` **Shared Files** manifest in `framework/bootstrap/govern.md` (slotted between `performance-frontend.md` and `security-backend.md`) and is auto-selected for every stack via the `-cross.md` suffix ([024-rule-loader](../024-rule-loader/spec.md)), composing with [033-rule-surface-setting](../033-rule-surface-setting/spec.md).

## Open Questions

- **Single category at launch, or seed siblings now?** Ship `STUB` alone (the file grows as concerns promote, mirroring how the performance set started), or also seed adjacent code-quality categories (e.g. swallowed errors, dead code) in this introducing spec?
- **Verification mechanism.** `QUAL-STUB-001` is fundamentally a code-pattern concern, so its primary checker is `/gov:review` against source rather than `/gov:analyze` against artifacts (unlike the 034 performance rules, which were design-time commitments). Confirm the review-time framing, and decide how the Verification clause operationalizes "the surrounding contract implies the path performs work" so genuine no-ops (default interface implementations, intentional pass-through middleware, not-yet-reachable branches) are not false-flagged.
- **MUST vs. SHOULD.** The statement is drafted as MUST (a silent stub is a correctness/availability hazard, not a tunable trade-off). Confirm MUST is the right severity.
- **`-cross` vs. surface split.** Confirm the discipline belongs in a single `-cross.md` file rather than being split into `-backend.md`/`-frontend.md` variants.
