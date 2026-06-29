---
status: draft
dependencies: [008-security-rules, 016-cross-cutting-rules, 024-rule-loader, 033-rule-surface-setting, 034-performance-backend-rules]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 038 — Backend concurrency rules

Introduce `framework/rules/concurrency-backend.md`, an enforceable rule set covering server-side concurrency correctness concerns. The file follows the canonical rule schema and is installed and enforced under the **backend** surface. This is a rule-introducing feature spec — the same path [008-security-rules](../008-security-rules/spec.md) used — and one of the three Task-9 backend sets that follow [034-performance-backend-rules](../034-performance-backend-rules/spec.md).

## Motivation

The constitution names **concurrency** a governance-recognized cross-cutting category (§rules) — and names it twice. The backend surface has **zero** coverage of it today. Concurrency defects — shared-state races, lost updates under the wrong isolation level, deadlocks from inconsistent lock ordering, double-applied non-idempotent retries, distributed locks without fencing tokens — are among the costliest to diagnose because they are non-deterministic and often invisible until production load. There is nowhere to promote these recurring hazards to the rules tier, so each is re-litigated per feature or caught only after a corruption incident.

This spec closes that gap: a backend concurrency rule set citable by ID (`BE-{CATEGORY}-{NNN}`) that `/gov:analyze` checks against specs and plans (design-time concurrency commitments).

## Rule set scope

`concurrency-backend.md` uses the **backend** surface (`-backend.md` suffix, `BE-` ID prefix), with NEW categories disjoint from the existing `BE-` namespaces in `security-backend.md` (`AUTHN`/`AUTHZ`/`INPUT`/`DATA`/`API`/`LOG`/`DEPS`/`ERR`), `api-backend.md` (`SCHEMA`/`APIVER`/`ERRENV`/`STATUS`/`PAGE`/`IDEMP`/`COMPAT`), and `performance-backend.md` (`QUERY`/`CACHE`/`POOL`/`PAYLOAD`/`ASYNC`). Candidate category set (to be resolved at clarify):

| Category | Abbrev | Concern |
| --- | --- | --- |
| Shared-state races | `RACE` | guarded shared mutable state; no check-then-act without atomicity |
| Locking | `LOCK` | optimistic vs. pessimistic choice stated; consistent lock ordering to avoid deadlock; bounded hold time |
| Transaction isolation | `TXN` | explicit isolation level per transaction; awareness of anomalies (lost update, write skew) |
| Distributed coordination | `COORD` | distributed locks carry fencing tokens; exactly-once vs. at-least-once stated |

Idempotency of retried operations is **not** a new category here — it is already `api-backend.md` `BE-IDEMP`, cross-referenced in the Boundaries below rather than redefined (a duplicate `IDEMP` category would collide with the existing `BE-IDEMP` namespace).

### Severity posture

Concurrency rules default to **SHOULD** where the right approach is contextual. A rule is **MUST** only when its absence is a correctness/corruption hazard *regardless of scale* — unguarded shared mutable state on a concurrent path, a distributed lock without a fencing token, a non-idempotent operation placed behind an automatic retry.

### Boundaries (cross-reference, do not duplicate)

- **Idempotency of API operations** is already `api-backend.md` `BE-IDEMP` — this set cites it for the retry-safety case rather than restating it.
- **Connection pooling and acquisition bounds** are `performance-backend.md` `BE-POOL-*` — concurrency cites them where lock/pool interaction matters; it does not re-derive pool sizing.
- **Operator-tunable values** (lock timeouts, retry counts) are `configuration-cross.md` `CFG-*`.

## Acceptance Criteria

- [ ] `framework/rules/concurrency-backend.md` exists, ends in the `-backend.md` suffix, and follows the canonical rule schema (`### {ID}` headings; Statement / Rationale / Verification; RFC 2119 language) per [008-security-rules](../008-security-rules/spec.md)'s data-model.
- [ ] Every rule ID uses the `BE-{CATEGORY}-{NNN}` format with a concurrency category disjoint from the `security-backend.md`, `api-backend.md`, and `performance-backend.md` category sets; `scripts/lint-rule-ids.sh` passes.
- [ ] The file header declares the concurrency category abbreviations per the per-file category-declaration policy ([016-cross-cutting-rules](../016-cross-cutting-rules/spec.md)).
- [ ] The rule set covers, at minimum, shared-state races, locking/deadlock avoidance, and transaction isolation — each with a Verification clause expressed as a **design-time commitment** the spec/plan must make (not a code-pattern grep), consistent with how `/gov:analyze` audits artifacts.
- [ ] Each MUST rule is one whose absence is a correctness/corruption hazard regardless of scale; contextual trade-offs are SHOULD. The split is evident from the Statements.
- [ ] Rules whose surface overlaps an existing rule cite it rather than restating it (`BE-IDEMP` for retry safety, `BE-POOL-*` for pool interaction, `CFG-*` for tunable config).
- [ ] The file is added to the `/govern` **Shared Files** manifest in `framework/bootstrap/govern.md` and is selected under the `backend` surface by `/gov:review`, composing with [033-rule-surface-setting](../033-rule-surface-setting/spec.md) and [024-rule-loader](../024-rule-loader/spec.md).

## Open Questions

- **Exact category set and abbreviations.** Four candidates above (`RACE`/`LOCK`/`TXN`/`COORD`) — confirm or trim.
- **Distributed coordination (`COORD`) in scope or deferred?** Single-process concurrency (races/locks/isolation) vs. distributed coordination (distributed locks, fencing, delivery semantics) — keep together, or split the distributed half to its own set?
- **Boundary with transaction isolation vs. a future data-handling set.** Confirm `TXN` lives here rather than in a data-handling rule set.
- **MUST vs. SHOULD default.** Confirm default SHOULD with MUST reserved for scale-independent correctness hazards.
