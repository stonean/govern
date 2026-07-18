---
status: in-progress
dependencies: [008-security-rules, 016-cross-cutting-rules, 017-derive-dont-ask, 024-rule-loader, 033-rule-surface-setting]
review:
  last-run: 2026-06-29T02:01:56Z
  reviewed-against: 7615f7fe656b26db674656e8c54068e6d892ae7c
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

### Added categories

- **`GROUND`** (`QUAL-GROUND-001`, SHOULD) — code whose correctness depends on an external contract it does not own (database schema, external API shape, config key, file/wire format) should bind to it so a wrong assumption fails loudly, rather than silently encoding a guess. The code-side counterpart to `/gov:analyze`'s artifact-grounding check; both enforce constitution §grounding. Added after the inaugural delivery per the category-growth policy (`data-model.md` §Category abbreviations), consolidating the code-side grounding enforcement into this existing `QUAL`-surface home rather than a new spec.

## Acceptance Criteria

- [x] `framework/rules/quality-cross.md` exists, ends in the `-cross.md` suffix, and follows the canonical rule schema (`### {ID}` headings; Statement / Rationale / Verification; RFC 2119 language) per [008-security-rules](../008-security-rules/spec.md)'s data-model.
- [x] `QUAL-STUB-001` is present with a Statement (RFC 2119), a Rationale, and a Verification clause.
- [x] Every rule ID uses the `QUAL-{CATEGORY}-{NNN}` format with the `QUAL` prefix and category abbreviation disjoint from the existing `BE-`/`FE-`/`CFG-` namespaces; `scripts/lint-rule-ids.sh` passes.
- [x] The file header declares the `QUAL` category abbreviation(s) per the per-file category-declaration policy ([016-cross-cutting-rules](../016-cross-cutting-rules/spec.md)).
- [x] The Verification clause is expressed as a check `/gov:review` can apply to code (silent passthrough vs. loud failure), and is scoped so legitimately-empty implementations are not flagged (the three-part discriminator and exemption list in Resolved Questions — Verification mechanism).
- [x] Rules whose surface overlaps an existing rule cite it rather than restating it (`BE-SCHEMA-002` for the build-time fail-loud case).
- [x] The file is added to the `/govern` **Shared Files** manifest in `framework/bootstrap/govern.md` (slotted between `performance-frontend.md` and `security-backend.md`) and is auto-selected for every stack via the `-cross.md` suffix ([024-rule-loader](../024-rule-loader/spec.md)), composing with [033-rule-surface-setting](../033-rule-surface-setting/spec.md).
- [x] `QUAL-GROUND-001` (SHOULD) is present with Statement / Rationale / Verification, the `GROUND` category is declared in the file header and registered in the data-model, and the rule is enforced by `/gov:review`'s quality pass as the code-side counterpart to `/gov:analyze`'s grounding check (constitution §grounding).

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Single category at launch, or seed siblings now?** Resolved: **ship `STUB` alone.** Mirrors how every prior rule set was introduced (008-security, 034-performance each started from a motivated nucleus and grew as concerns promoted) — a single-rule file is valid. `QUAL-STUB-001` has a concrete motivating incident (the anvil rate-limiter passthrough); "swallowed errors" and "dead code" have no grounding incident yet, so seeding them now would be speculative governance with no verifiable Statement/Rationale. The `QUAL-{CATEGORY}-{NNN}` grammar leaves room to promote adjacent categories later via `/gov:amend` or a follow-on spec when a concrete need appears.
- **Verification mechanism.** Resolved: **review-time framing confirmed, with a three-part discriminator and an explicit exemption list.** `QUAL-STUB-001` is a source-code pattern, so its checker is `/gov:review`'s quality pass against code in scope — not `/gov:analyze` (which audits artifacts). The Verification clause flags a path only when **all three** hold: (1) it is **reachable** under the current spec, (2) its **surrounding contract implies work** (named for a behavior, documented to do something, or called by code that depends on its effect), and (3) it returns a success/zero/pass-through value with **no loud signal**. Exemptions named in the clause so genuine no-ops are not flagged: an explicit incompleteness marker *is* compliance (`panic`/`todo!`/`unimplemented!`, a raised `NotImplementedError`, or a failing/skipped test fixture — that *is* failing loudly); intentional pass-through middleware documented as deliberate; default/interface implementations meant to be empty; and not-yet-reachable branches behind a feature flag or guard. One-line discriminator: a stub is forbidden only when the contract implies work **and** there is no loud signal marking it incomplete.
- **MUST vs. SHOULD.** Resolved: **MUST.** A silent stub is a correctness/availability hazard, not a tunable trade-off — the motivating incident (a rate-limiter that would have silently disabled itself in production) is exactly what RFC 2119 MUST exists for; SHOULD would let it pass `/gov:review` without blocking. The compliance cost is trivial and always available (one `panic`/`todo!`/error line to fail loudly), while the violation cost is a production outage, so it belongs at MUST — making it a blocking violation that gates `done`. MUST does not trap legitimate cases: the Verification exemptions exclude genuine no-ops, and per-finding waivers (`--waive` with a recorded reason) plus whole-file `[[review.disabled-rule-files]]` remain available for any case the rule should not apply to.
- **`-cross` vs. surface split.** Resolved: **single `-cross.md` file.** The discipline is surface-independent — the failure mode (a contract-implying path that silently passes through) and the remedy (fail loudly) are identical for backend workers/domain methods/middleware and frontend stores/handlers/reducers. Splitting would duplicate one rule across two files (a `QUAL-STUB-001` wearing two IDs), which the framework avoids — overlapping rules cite, not restate. `-cross.md` is exactly the "applies to every stack" suffix: the rule-loader (024) auto-selects it regardless of detected stack and it composes with 033 (cross files are unconditional, never surface-filtered), so backend-only and frontend-only projects both get stub discipline. A genuinely surface-specific stub concern, if one ever emerges, can live in a `-backend.md`/`-frontend.md` file later; the general discipline stays cross.
