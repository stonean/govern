# 038 — Backend concurrency rules Plan

Implements [038 — Backend concurrency rules](spec.md).

## Overview

A rule-introducing, markdown-tier feature — the 034/037 path exactly. It adds one rule file, `framework/rules/concurrency-backend.md`, on the **existing** `BE` surface (so no `scripts/lint-rule-ids.sh` change and no `data-model.md`), and wires it into the `/govern` Shared Files manifest. The §Shared Files note is already count-free (036), so no count to update.

Four categories ship — `RACE`, `LOCK`, `TXN`, `COORD` — with eight rules, four of them MUST. Verification is **design-time commitment** framing enforced by `/gov:analyze` against feature artifacts, matching 034/037.

## Technical Decisions

### The rule file — `framework/rules/concurrency-backend.md`

Modeled on `performance-backend.md` / `observability-backend.md` (same surface, same analyze-time framing):

- **Header.** Title `# Concurrency Rules — Backend`; an intro scoping it to server-side concurrency correctness; the RFC 2119 note; the ID-format / category-declaration line stating IDs follow `BE-{CATEGORY}-{NNN}` with categories `RACE` (shared-state races), `LOCK` (locking/deadlock), `TXN` (transaction isolation), `COORD` (distributed coordination), and a pointer to `specs/008-security-rules/data-model.md` for the schema; a default-**SHOULD** paragraph (MUST reserved for scale-independent correctness/corruption hazards) noting these verify design-time commitments enforced by `/{project}:analyze`; and the standard backend pin/surface note.
- **Categories disjoint** from the other backend files (verified: `RACE`/`LOCK`/`TXN`/`COORD` appear in none).

### Rule set

Eight rules across four `## BE-{CATEGORY}` sections. Verification is phrased as a design-time commitment a spec/plan must make.

| ID | Sev | Statement gist |
| --- | --- | --- |
| `BE-RACE-001` | **MUST** | Shared mutable state reachable from more than one concurrent execution context is protected by a synchronization mechanism (lock / atomic / single-owner / serialized); unsynchronized concurrent read-write is a data race. |
| `BE-RACE-002` | SHOULD | Prefer eliminating shared mutable state (immutability, confinement, message-passing) over guarding it, to shrink the surface that can race. |
| `BE-LOCK-001` | SHOULD | The concurrency-control strategy for a contended resource is stated — optimistic (version/CAS) vs. pessimistic (lock) — with rationale. |
| `BE-LOCK-002` | SHOULD | When multiple locks are acquired, a consistent global acquisition order is defined and followed, and hold time is bounded (lock timeout), to avoid deadlock and unbounded waits. |
| `BE-TXN-001` | SHOULD | Each transaction states its isolation level explicitly rather than silently relying on the engine default. |
| `BE-TXN-002` | **MUST** | A concurrent read-modify-write of persisted state prevents lost updates (optimistic version check, `SELECT … FOR UPDATE`, or an atomic conditional write); a bare read-then-write is silent corruption. |
| `BE-COORD-001` | **MUST** | A distributed lock carries a fencing token that the protected resource validates — a lease alone is insufficient under process pauses or clock skew (split-brain double-write). |
| `BE-COORD-002` | **MUST** | An operation invoked under at-least-once delivery or automatic retry is idempotent (cite `api-backend.md` `BE-IDEMP`) or dedup-guarded, and the delivery semantics (at-least-once vs. exactly-once) are stated. |

Cross-references, not restatements: `BE-COORD-002` cites `api-backend.md` `BE-IDEMP` (retry safety); `BE-LOCK-002`/`BE-RACE-001` cite `performance-backend.md` `BE-POOL-*` where lock/pool interaction matters; tunables (lock timeout, retry count) cite `configuration-cross.md` `CFG-*`.

### Manifest wiring — `framework/bootstrap/govern.md`

Add a `concurrency-backend.md` → `specs/rules/concurrency-backend.md` row to the `### govern-owned shared files` table, slotted between `api-backend.md` and `configuration-cross.md` (alphabetical: `concurrency` sorts before `configuration` — `conc` < `conf`). The `-backend.md` suffix means 024's loader selects it under the `backend` surface and 033's filter includes it. The §Shared Files note is count-free, so no count edit.

### What this feature does NOT touch

- **`scripts/lint-rule-ids.sh`** — `BE` is already registered; `BE-RACE-001` etc. match the regex unchanged.
- **`data-model.md`** — no new surface and no new schema; categories are declared in the file header (016 policy), schema is 008's. No data-model artifact (matching 034/037).

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/rules/concurrency-backend.md` | Create | The concurrency rule set (`RACE`/`LOCK`/`TXN`/`COORD`, eight rules, four MUST) |
| `framework/bootstrap/govern.md` | Modify | Add the manifest row (between `configuration-cross` and `observability-backend`) |

## Trade-offs

- **Four categories, `COORD` kept.** Distributed coordination fits the design-time-commitment model (fencing tokens, delivery semantics are per-feature commitments), so it stays rather than splitting to a separate distributed-systems set (clarify resolution).
- **`TXN` here, not a future data-handling set.** Isolation levels manage concurrent-access anomalies — concurrency, not data-at-rest (clarify resolution).
- **Four MUSTs.** Concurrency is unusually corruption-dense: data races (`RACE-001`), lost updates (`TXN-002`), missing fencing (`COORD-001`), and non-idempotent retries (`COORD-002`) each silently corrupt state regardless of scale. The contextual choices — optimistic vs. pessimistic, isolation-level selection, race-surface reduction — stay SHOULD.
- **Idempotency not duplicated.** Retry-safety cites `BE-IDEMP` rather than introducing a colliding `IDEMP` category (the spec's own boundary).
- **Known limitation.** Analyze-time verification checks that a spec/plan *commits* to these guards; it cannot prove the guard is correctly implemented (a lock taken in the wrong order, a fencing token not actually validated) — that residual is for `/gov:review` and tests, the standard design-time-rule limitation.

## Cross-spec impact

The spec/plan reference 008, 016, 024, 033, 034 (deps) and cite `BE-IDEMP` (api-backend), `BE-POOL-*` (performance-backend), `CFG-*` (configuration-cross). None need an edit: all are cited, not changed; 024/033 already select `-backend.md` files; 034/037 are sibling precedents, unaffected. Informational; does not block.
