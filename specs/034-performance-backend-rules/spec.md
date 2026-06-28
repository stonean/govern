---
status: planned
dependencies: [008-security-rules, 016-cross-cutting-rules, 033-rule-surface-setting]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 034 — Backend performance rules

Introduce `framework/rules/performance-backend.md`, the backend counterpart to `performance-frontend.md`: an enforceable rule set covering server-side performance concerns the framework should govern across features. The file follows the canonical rule schema and is installed and enforced under the **backend** surface.

## Motivation

The constitution names **performance** a governance-recognized cross-cutting category (§rules), and the frontend surface already ships `performance-frontend.md` (Core Web Vitals, bundle size, image/font discipline). The backend surface has no performance rule set at all — only `security-backend.md` and `api-backend.md`. Backend performance failures that recur across features — N+1 queries, missing indexes, unbounded caches, exhausted connection pools, oversized responses, slow work on the request path — have nowhere to be promoted to the rules tier; each is re-litigated per feature or caught only in production.

This spec closes the asymmetry: a backend performance rule set citable by ID (`BE-{CATEGORY}-{NNN}`) that `/gov:analyze` checks against specs and plans and `/gov:review` checks against code.

## Rule set scope

`performance-backend.md` uses the **backend** surface (`-backend.md` suffix, `BE-` ID prefix), with NEW performance categories that stay disjoint from the existing `BE-` namespaces in `security-backend.md` (`AUTHN`/`AUTHZ`/`INPUT`/`DATA`/`API`/`LOG`/`DEPS`/`ERR`) and `api-backend.md` (`SCHEMA`/`APIVER`/`ERRENV`/`STATUS`/`PAGE`/`IDEMP`/`COMPAT`). The category set (resolved at clarify):

| Category | Abbrev | Concern |
| --- | --- | --- |
| Query efficiency | `QUERY` | N+1 avoidance; indexes for filtered/sorted/joined columns; bounded result sets |
| Caching | `CACHE` | cache keys (scope-complete), TTLs / invalidation, stampede protection |
| Connection pooling | `POOL` | pooled connections, sized by named constant, bounded acquisition timeout, release on all paths |
| Payload budgets | `PAYLOAD` | response size caps, field selection, compression |
| Async offloading | `ASYNC` | move slow / unbounded / third-party work off the synchronous request path |

### Severity posture

Performance rules default to **SHOULD** (advisory; thresholds are context-dependent). A rule is **MUST** only when its absence is a denial-of-service or resource-exhaustion risk *regardless of scale* — unbounded queries, never-expiring caches, per-request connections, unbounded pool waits, unbounded response sizes, request-blocking slow work. Tunable efficiency trade-offs (index choice, field selection, compression) stay SHOULD.

### Boundaries (cross-reference, do not duplicate)

- **Pagination** is already `api-backend.md` §BE-PAGE — `performance-backend.md` cites it, never restates it.
- **Unbounded-input DoS bounds** are `security-backend.md` `BE-INPUT-006` — the perf rules cite it for request-size limits rather than re-deriving them.
- **Operator-tunable values** (the *configuration* of timeouts, pool sizes, batch sizes) are `configuration-cross.md` `CFG-CONST-*` / `CFG-ENV-*` — perf rules require the value to exist and be bounded; the config rules govern how it is named and validated.
- **Server-side deadlines, downstream-call timeouts, retries, and circuit breakers** are **out of scope** here: they are as much reliability as performance and are deferred to a future `reliability-backend.md` rule set (a separate Task-9 set). This keeps `performance-backend.md` focused on throughput and latency efficiency.

## Acceptance Criteria

- [ ] `framework/rules/performance-backend.md` exists, ends in the `-backend.md` suffix, and follows the canonical rule schema (`### {ID}` headings; Statement / Rationale / Verification; RFC 2119 language) per [008-security-rules](../008-security-rules/spec.md)'s data-model.
- [ ] Every rule ID uses the `BE-{CATEGORY}-{NNN}` format with a performance category (`QUERY`/`CACHE`/`POOL`/`PAYLOAD`/`ASYNC`) disjoint from the `security-backend.md` and `api-backend.md` category sets; `scripts/lint-rule-ids.sh` passes.
- [ ] The file header declares the five performance category abbreviations (per the constitution's per-file category-declaration policy, as framed in [016-cross-cutting-rules](../016-cross-cutting-rules/spec.md)).
- [ ] The rule set covers, at minimum, query efficiency (N+1 + indexes + bounded results), caching (TTL/invalidation, scope-complete keys, stampede), connection pooling (pooled + sized + bounded acquisition + release), and payload budgets (size cap + field selection + compression) — each with a Verification clause expressed as a **design-time commitment** the spec/plan must make (not a code-pattern grep), consistent with how `/gov:analyze` audits artifacts.
- [ ] Each MUST rule is one whose absence is a DoS/exhaustion risk regardless of scale; tunable efficiency trade-offs are SHOULD. The split is evident from the Statements.
- [ ] Rules whose surface overlaps an existing rule cite it rather than restating it (BE-PAGE for pagination, BE-INPUT-006 for input bounds, BE-IDEMP for retry-safe async, CFG-* for tunable-value config).
- [ ] The file is added to the `/govern` **Shared Files** manifest in `framework/bootstrap/govern.md` and is selected under the `backend` surface by `/gov:review`, composing with [033-rule-surface-setting](../033-rule-surface-setting/spec.md).

## Resolved Questions

- **Exact category set and abbreviations.** Resolved: five categories — `QUERY`, `CACHE`, `POOL`, `PAYLOAD`, `ASYNC`. `DEADLINE` (the originally-proposed sixth) is dropped from this set (see next question).
- **Deadlines/timeouts here vs. a future reliability set.** Resolved: **moved out.** Server-side deadlines, downstream-call timeouts, retries, and circuit breakers go in a future `reliability-backend.md` (a separate Task-9 rule set), not here. `performance-backend.md` stays purely about throughput/latency efficiency; this avoids a perf/reliability scope blur and keeps each set coherent.
- **MUST vs SHOULD default.** Resolved: default **SHOULD**; **MUST** only when absence is a DoS/exhaustion risk regardless of scale (unbounded queries/caches/payloads, per-request connections, unbounded pool waits, request-blocking slow work). Tunable efficiency trade-offs stay SHOULD. Recorded as the **Severity posture** above.
- **Verification style for N+1 / index rules.** Resolved: **design-time commitments** targeting specs/plans (e.g., "any plan that introduces a list-rendering endpoint MUST state its query strategy and the indexes it relies on"), not code-pattern detection — consistent with `/gov:analyze` auditing artifacts rather than source.
- **Threshold values.** Resolved: **qualitative.** Rules require a budget to *exist and be bounded*; the concrete number is deferred to each project's named constants per `CFG-CONST-003`. No rule hard-codes a numeric threshold.
