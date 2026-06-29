---
status: done
dependencies: [008-security-rules, 016-cross-cutting-rules, 024-rule-loader, 033-rule-surface-setting, 034-performance-backend-rules]
review:
  last-run: 2026-06-29T02:41:53Z
  reviewed-against: 38ef20a463fe2162ccf0888bcc56954a9b6e9c9d
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 039 — Backend reliability rules

Introduce `framework/rules/reliability-backend.md`, an enforceable rule set covering server-side reliability concerns — deadlines, timeouts, retries, circuit breakers, graceful shutdown, and backpressure. The file follows the canonical rule schema and is installed and enforced under the **backend** surface. This is a rule-introducing feature spec — the same path [008-security-rules](../008-security-rules/spec.md) used — and one of the three Task-9 backend sets that follow [034-performance-backend-rules](../034-performance-backend-rules/spec.md).

## Motivation

The constitution calls the system to be **Reliable** ("graceful degradation and automatic recovery when components fail"). [034-performance-backend-rules](../034-performance-backend-rules/spec.md) deliberately scoped *out* server-side deadlines, downstream-call timeouts, retries, and circuit breakers — "as much reliability as performance" — and forward-references a future `reliability-backend.md`. Landing this spec resolves that forward-reference and gives reliability concerns a home at the rules tier.

Reliability failures that recur across features — an unbounded downstream wait that exhausts threads, a retry storm that amplifies an outage, a deploy that drops in-flight requests because nothing drains on shutdown — have nowhere to be promoted today. Each is re-litigated per feature or surfaces only when a dependency degrades.

This spec closes that gap: a backend reliability rule set citable by ID (`BE-{CATEGORY}-{NNN}`) that `/gov:analyze` checks against specs and plans (design-time reliability commitments).

## Rule set scope

`reliability-backend.md` uses the **backend** surface (`-backend.md` suffix, `BE-` ID prefix), with NEW categories disjoint from the existing `BE-` namespaces in `security-backend.md` (`AUTHN`/`AUTHZ`/`INPUT`/`DATA`/`API`/`LOG`/`DEPS`/`ERR`), `api-backend.md` (`SCHEMA`/`APIVER`/`ERRENV`/`STATUS`/`PAGE`/`IDEMP`/`COMPAT`), and `performance-backend.md` (`QUERY`/`CACHE`/`POOL`/`PAYLOAD`/`ASYNC`). Category set (resolved at clarify — five categories ship; `DEADLINE` is folded into `TIMEOUT`; see Resolved Questions):

| Category | Abbrev | Concern |
| --- | --- | --- |
| Timeouts & deadlines | `TIMEOUT` | every downstream/IO call has a bounded timeout (no unbounded waits); an end-to-end request deadline is set and propagated to downstream calls |
| Retries | `RETRY` | bounded retries with backoff + jitter, idempotent ops only (cross-ref `BE-IDEMP`); no retry storms |
| Circuit breakers | `BREAKER` | breaker around failure-prone downstreams; fail fast when open |
| Graceful shutdown | `DRAIN` | drain in-flight work on shutdown; stop accepting new work; honor termination signals |
| Backpressure | `BULK` | bulkheads / bounded queues / load shedding to isolate failures and degrade gracefully |

### Severity posture

Reliability rules default to **SHOULD** where the approach is contextual. A rule is **MUST** only when its absence is an availability or cascading-failure risk *regardless of scale* — an unbounded downstream wait (no timeout), an unbounded/un-jittered retry (retry storm), no graceful drain (dropped in-flight requests on every deploy).

### Boundaries (cross-reference, do not duplicate)

- **Connection pooling** (including bounded acquisition timeout) is `performance-backend.md` `BE-POOL-002` — reliability cites it where pool exhaustion interacts with timeouts/breakers; it does not re-derive pooling.
- **Idempotency** is `api-backend.md` `BE-IDEMP` — the `RETRY` rule cites it (retries apply only to idempotent operations).
- **Async offloading** is `performance-backend.md` `BE-ASYNC` — reliability cites it where backpressure governs offloaded work.
- **Operator-tunable values** (timeout durations, retry counts, breaker thresholds) are `configuration-cross.md` `CFG-CONST-*` / `CFG-ENV-*`.

## Acceptance Criteria

- [x] `framework/rules/reliability-backend.md` exists, ends in the `-backend.md` suffix, and follows the canonical rule schema (`### {ID}` headings; Statement / Rationale / Verification; RFC 2119 language) per [008-security-rules](../008-security-rules/spec.md)'s data-model.
- [x] Every rule ID uses the `BE-{CATEGORY}-{NNN}` format with a reliability category disjoint from the `security-backend.md`, `api-backend.md`, and `performance-backend.md` category sets; `scripts/lint-rule-ids.sh` passes.
- [x] The file header declares the reliability category abbreviations per the per-file category-declaration policy ([016-cross-cutting-rules](../016-cross-cutting-rules/spec.md)).
- [x] The rule set covers, at minimum, deadlines/timeouts, bounded retries (backoff + jitter, idempotent only), circuit breakers, and graceful shutdown — each with a Verification clause expressed as a **design-time commitment** the spec/plan must make (not a code-pattern grep), consistent with how `/gov:analyze` audits artifacts.
- [x] Each MUST rule is one whose absence is an availability/cascading-failure risk regardless of scale; contextual trade-offs are SHOULD. The split is evident from the Statements.
- [x] Rules whose surface overlaps an existing rule cite it rather than restating it (`BE-POOL-002` for pooling, `BE-IDEMP` for retry safety, `BE-ASYNC` for offloaded work, `CFG-*` for tunable config).
- [x] The file is added to the `/govern` **Shared Files** manifest in `framework/bootstrap/govern.md` and is selected under the `backend` surface by `/gov:review`, composing with [033-rule-surface-setting](../033-rule-surface-setting/spec.md) and [024-rule-loader](../024-rule-loader/spec.md).
- [x] 034's forward-reference to a future `reliability-backend.md` resolves to this rule set (the deferred deadlines/timeouts/retries/circuit-breakers land here).

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Exact category set and abbreviations.** Resolved: **five categories — `TIMEOUT`, `RETRY`, `BREAKER`, `DRAIN`, `BULK`**, each disjoint from the existing `BE-` category sets. `DEADLINE` is folded into `TIMEOUT` (per-call timeout and end-to-end deadline propagation are the same concern — bounding the wait — at two scopes, carried as two rules in one category) rather than a separate category.
- **Backpressure/bulkheads (`BULK`) in scope or deferred?** Resolved: **kept in scope.** Bulkheads, bounded queues, and load shedding are the canonical graceful-degradation pattern (the constitution's reliability principle) and prevent one slow dependency from consuming all resources (cascading failure). It fits the design-time-commitment model and cites `performance-backend.md` `BE-ASYNC`/`BE-POOL-*` where it touches offloaded-work and pool concerns rather than re-deriving them.
- **MUST vs. SHOULD default.** Resolved: **default SHOULD; MUST only when the absence is an availability or cascading-failure risk regardless of scale.** Mirrors 034/037/038. The three MUSTs at launch: an unbounded downstream wait (no timeout), an unbounded or un-jittered retry (retry storm), and no graceful drain on shutdown (dropped in-flight requests every deploy). Contextual choices (timeout durations, breaker thresholds, retry budgets, which downstreams warrant a breaker) stay SHOULD.
- **Confirm the perf/reliability boundary.** Resolved: **confirmed.** All four concerns 034 deferred — request deadlines, downstream-call timeouts, retries, and circuit breakers — land here (`TIMEOUT`/`RETRY`/`BREAKER`), resolving 034's forward-reference. `performance-backend.md` `BE-POOL-002` (bounded pool-acquisition timeout) remains in the performance set and is cited here where pool exhaustion interacts with timeouts/breakers; it is not moved.
