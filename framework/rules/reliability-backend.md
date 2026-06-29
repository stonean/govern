# Reliability Rules — Backend

Enforceable reliability rules for server-side resilience under partial failure — timeouts and deadlines, retries, circuit breakers, graceful shutdown, and backpressure. These rules apply to projects adopting `govern` whose surface includes a backend. They land the deadlines, downstream-call timeouts, retries, and circuit breakers that `performance-backend.md` deliberately deferred.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `BE-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `TIMEOUT` (timeouts and deadlines), `RETRY` (retries), `BREAKER` (circuit breakers), `DRAIN` (graceful shutdown), `BULK` (backpressure and bulkheads). See `specs/008-security-rules/data-model.md` for the full schema.

Reliability rules default to **SHOULD** where the approach is contextual. A rule is **MUST** only when its absence is an availability or cascading-failure risk regardless of scale — an unbounded downstream wait, a retry storm, a deploy that drops in-flight work. These rules verify **design-time commitments** — what a spec or plan must state — rather than code patterns; `/{project}:analyze` enforces them against feature artifacts.

Projects without a backend can pin this file in `.govern.toml` or set `[rules] surfaces` to exclude the backend surface, and it will be skipped during `govern` updates and reviews.

## BE-TIMEOUT — Timeouts and deadlines

### BE-TIMEOUT-001

> Every outbound network call, downstream-service call, or blocking I/O operation MUST be made under a bounded timeout — no call waits unboundedly for a response.

**Rationale:** An unbounded wait on a slow or hung dependency holds the calling thread, connection, or goroutine indefinitely; under any load that exhausts the pool and the failure cascades back to every caller — independent of scale, since a single hung dependency suffices. Pool-acquisition timeouts are governed by `performance-backend.md` `BE-POOL-002`; this rule bounds the call itself.

**Verification:** Any spec or plan that introduces an outbound or blocking call MUST name the timeout that bounds it. Validate flags plans that add a downstream or I/O call with no stated timeout.

**Source:** Nygard, *Release It!* — Timeouts.

### BE-TIMEOUT-002

> A request that fans out to downstream calls SHOULD carry an end-to-end deadline that is propagated to those calls and shrinks as time is consumed, so total work respects the caller's budget.

**Rationale:** Independent per-call timeouts can sum to far more than the client will wait. A propagated deadline caps the whole request and lets downstreams abandon work the caller has already given up on, freeing resources sooner.

**Verification:** Any spec or plan that fans out to multiple downstream calls SHOULD name the request deadline and how it is propagated. Validate flags fan-out plans that bound each call but not the overall request.

## BE-RETRY — Retries

### BE-RETRY-001

> Automatic retries MUST be bounded by a maximum attempt count, spaced with exponential backoff plus jitter, and limited to idempotent operations (see `api-backend.md` `BE-IDEMP`).

**Rationale:** Unbounded or fixed-interval retries synchronize across callers and amplify a transient downstream blip into a self-sustaining retry storm that prevents recovery; backoff with jitter de-correlates and bounds the added load, and retrying a non-idempotent operation double-applies it. The storm is scale-independent — even a small fleet retrying in lockstep overwhelms a single recovering dependency — which is why it is MUST. This rule owns the retry mechanics (bound, backoff, jitter); the delivery-side duplicate-handling obligation for at-least-once consumers is `concurrency-backend.md` `BE-COORD-002`.

**Verification:** Any spec or plan that introduces automatic retries MUST state the attempt bound, the backoff-with-jitter policy, and the idempotency basis for the retried operation. Validate flags retry plans missing a bound, jitter, or an idempotency basis.

**Source:** AWS Architecture Blog — exponential backoff and jitter.

### BE-RETRY-002

> Retries SHOULD be governed by a retry budget (e.g. a token bucket capping retries as a fraction of requests) and coordinated with circuit breaking, so a degraded downstream is not hammered by aggregate retries.

**Rationale:** Per-call backoff bounds one caller's retries but not the system-wide aggregate; a budget caps total retry amplification and, paired with a breaker, stops retrying a dependency that is already known to be down.

**Verification:** Any spec or plan that introduces retries against a shared downstream SHOULD state its retry budget and how it composes with circuit breaking. Validate flags retry plans with no aggregate cap.

## BE-BREAKER — Circuit breakers

### BE-BREAKER-001

> Calls to a failure-prone downstream SHOULD be guarded by a circuit breaker that opens on a failure-rate threshold, fails fast (or falls back) while open, and half-opens to probe recovery.

**Rationale:** Without a breaker, every caller keeps trying a dead dependency — tying up timeouts, threads, and retries and slowing the whole service. Failing fast while open sheds that load and gives the dependency room to recover.

**Verification:** Any spec or plan that calls a downstream with a meaningful failure rate SHOULD state whether a circuit breaker guards it and the open/half-open policy. Validate flags integrations with failure-prone downstreams that commit to no breaker or fallback.

**Source:** Nygard, *Release It!* — Circuit Breaker.

## BE-DRAIN — Graceful shutdown

### BE-DRAIN-001

> On receiving a termination signal, a service MUST stop accepting new work, finish or safely hand off in-flight work within a bounded grace period, and only then exit.

**Rationale:** Exiting immediately on SIGTERM drops in-flight requests and abandons partial work on every deploy or scale-down — a recurring, scale-independent availability hit, since a single in-flight request is enough to be lost. A bounded grace period prevents both dropped work and unbounded shutdown hangs.

**Verification:** Any spec or plan for a deployable service MUST commit to a graceful-shutdown sequence — stop intake, drain in-flight work within a bounded grace period, then exit. Validate flags service plans that do not address shutdown draining.

**Source:** Kubernetes pod termination lifecycle (SIGTERM + grace period).

### BE-DRAIN-002

> At the start of shutdown, readiness SHOULD flip to not-ready before draining begins, so the orchestrator stops routing new traffic while in-flight work completes (see `observability-backend.md` `BE-HEALTH-001`).

**Rationale:** Draining without first failing readiness races the load balancer, which keeps routing new requests into a closing instance — re-introducing the dropped-request problem the drain was meant to prevent.

**Verification:** Any spec or plan that defines graceful shutdown SHOULD state that readiness fails first, before the drain. Validate flags shutdown sequences that drain without de-registering from routing.

## BE-BULK — Backpressure and bulkheads

### BE-BULK-001

> Failure-prone or resource-heavy work SHOULD be isolated behind a bulkhead — bounded concurrency or a bounded queue — that sheds load (rejects or degrades) when saturated rather than queueing unboundedly.

**Rationale:** A single slow dependency without isolation consumes all threads and memory and takes the whole service down (cascading failure); a bulkhead confines the blast radius to that dependency, and shedding load when saturated preserves partial availability instead of collapsing under an unbounded backlog. Bounded queues and offloaded work are `performance-backend.md` `BE-ASYNC` / `BE-POOL-*`; this rule adds the isolation-and-shedding commitment.

**Verification:** Any spec or plan that introduces a failure-prone or resource-heavy dependency SHOULD state its bulkhead (bounded concurrency or queue) and its load-shedding behavior at saturation. Validate flags plans that share one unbounded resource pool across independent workloads.

**Source:** Nygard, *Release It!* — Bulkheads.
