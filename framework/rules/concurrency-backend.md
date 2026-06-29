# Concurrency Rules — Backend

Enforceable concurrency-correctness rules for server-side shared state, locking, transactions, and distributed coordination. These rules apply to projects adopting `govern` whose surface includes a backend.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `BE-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `RACE` (shared-state races), `LOCK` (locking and deadlock avoidance), `TXN` (transaction isolation), `COORD` (distributed coordination). See `specs/008-security-rules/data-model.md` for the full schema.

Concurrency rules default to **SHOULD** where the right approach is contextual. A rule is **MUST** only when its absence is a correctness or corruption hazard regardless of scale — a defect two concurrent actors are enough to trigger. These rules verify **design-time commitments** — what a spec or plan must state — rather than code patterns; `/{project}:analyze` enforces them against feature artifacts.

Projects without a backend can pin this file in `.govern.toml` or set `[rules] surfaces` to exclude the backend surface, and it will be skipped during `govern` updates and reviews.

## BE-RACE — Shared-state races

### BE-RACE-001

> Shared mutable state reachable from more than one concurrent execution context MUST be protected by a synchronization mechanism — a lock, an atomic primitive, single-owner/actor confinement, or serialized access; unsynchronized concurrent read-write is a data race.

**Rationale:** A data race corrupts state non-deterministically and stays invisible until a production interleaving exposes it. It is a correctness hazard regardless of scale — two concurrent contexts are enough — which is why it is MUST.

**Verification:** Any spec or plan that introduces state shared across threads, goroutines, async tasks, or request handlers MUST name how that state is synchronized. Validate flags plans that introduce concurrent access to shared mutable state without a stated synchronization mechanism.

**Source:** Go / Java memory model (happens-before).

### BE-RACE-002

> Designs SHOULD prefer eliminating shared mutable state — immutability, confinement to a single owner, or message-passing — over guarding it, to shrink the surface that can race.

**Rationale:** Every guarded shared variable is a future race waiting for one missed lock. Removing the sharing removes the hazard class outright, rather than relying on perpetual discipline.

**Verification:** Any spec or plan that introduces shared mutable state SHOULD justify why it is shared rather than confined or immutable. Validate flags plans that add shared mutable state where confinement or immutability would serve.

## BE-LOCK — Locking and deadlock avoidance

### BE-LOCK-001

> A spec or plan that introduces a contended resource SHOULD state its concurrency-control strategy — optimistic (version / compare-and-set) or pessimistic (locking) — with the rationale for the choice.

**Rationale:** Optimistic and pessimistic control have opposite failure modes (retry storms under contention vs. lock waits and reduced throughput). Choosing implicitly hides the trade-off and the operational behavior under load.

**Verification:** Any spec or plan that introduces a contended resource SHOULD name and justify its optimistic-vs-pessimistic choice. Validate flags contention-bearing plans that commit to neither.

### BE-LOCK-002

> When more than one lock is held at once, locks SHOULD be acquired in a single consistent global order and released within a bounded hold time, to avoid deadlock and unbounded waits.

**Rationale:** Inconsistent acquisition order deadlocks reliably once two code paths take the same locks in opposite orders; unbounded hold time serializes throughput and stacks waiters behind a slow critical section. Lock timeouts and hold bounds are operator-tunable values governed by `configuration-cross.md` `CFG-*`.

**Verification:** Any spec or plan that acquires more than one lock SHOULD state the global acquisition order and the hold-time bound. Validate flags multi-lock plans with no stated ordering or timeout.

## BE-TXN — Transaction isolation

### BE-TXN-001

> Each transaction SHOULD state its isolation level explicitly rather than relying on the engine's default, which varies by database and is often weaker than assumed.

**Rationale:** The default isolation level (READ COMMITTED in PostgreSQL, REPEATABLE READ in MySQL/InnoDB) silently determines which anomalies are possible. An implicit default is an unstated correctness assumption that breaks when the database or its configuration changes.

**Verification:** Any spec or plan that introduces a multi-statement transaction SHOULD name the isolation level it requires and the anomalies that level prevents. Validate flags transactional plans that do not state an isolation level.

**Source:** ANSI SQL isolation levels.

### BE-TXN-002

> A concurrent read-modify-write of persisted state MUST prevent lost updates — via an optimistic version/ETag check, a `SELECT … FOR UPDATE` row lock, or an atomic conditional write; a bare read-then-write under concurrency is silent data corruption.

**Rationale:** Two transactions that read, then write, the same record without a guard silently discard one update — no error is raised, and the loss is scale-independent (two concurrent writers suffice). This is the canonical lost-update anomaly.

**Verification:** Any spec or plan that introduces a concurrent update to shared persisted state MUST name its lost-update guard (version check, row lock, or atomic update). Validate flags read-modify-write plans that commit to none.

**Source:** Berenson et al., "A Critique of ANSI SQL Isolation Levels" (SIGMOD 1995) — lost update (P4).

## BE-COORD — Distributed coordination

### BE-COORD-001

> A distributed lock MUST carry a fencing token that the protected resource validates, rejecting stale holders — a lease or lock alone is insufficient, because a paused or clock-skewed holder can act after its lease has expired.

**Rationale:** Without fencing, a GC pause, VM freeze, or network partition lets two nodes each believe they hold the lock and both write — split-brain corruption. A monotonic fencing token enforced at the resource makes the resource reject the stale writer, which is why it is MUST.

**Verification:** Any spec or plan that introduces a distributed lock or leader election MUST commit to a monotonic fencing token enforced at the protected resource. Validate flags distributed-lock plans that rely on lease expiry alone.

**Source:** Kleppmann, "How to do distributed locking" (fencing tokens).

### BE-COORD-002

> An operation invoked under at-least-once delivery or automatic retry MUST be idempotent (see `api-backend.md` `BE-IDEMP`) or deduplication-guarded, with its delivery semantics (at-least-once vs. exactly-once) stated.

**Rationale:** Retries and at-least-once brokers deliver duplicates by design; a non-idempotent handler double-applies them — a double charge, a double ship, a doubled balance — which is silent corruption regardless of scale. Stating the delivery semantics makes the duplicate-handling obligation explicit. Retry safety builds on `api-backend.md` `BE-IDEMP` rather than restating it; this rule owns the delivery-semantics and duplicate-handling obligation, while the retry *mechanics* — attempt bounds, backoff, and jitter — are `reliability-backend.md` `BE-RETRY-001`.

**Verification:** Any spec or plan that consumes from a broker, exposes a retried operation, or relies on at-least-once delivery MUST commit to idempotency or deduplication and state the delivery semantics. Validate flags retried or queue-consuming plans with no idempotency or dedup commitment.
