# Observability Rules — Backend

Enforceable observability rules for server-side metrics, distributed tracing, and health signaling. These rules apply to projects adopting `govern` whose surface includes a backend.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `BE-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `METRIC` (metrics), `TRACE` (distributed tracing), `HEALTH` (health endpoints). See `specs/008-security-rules/data-model.md` for the full schema.

Observability rules default to **SHOULD** — the right metrics, spans, and probes are context-dependent. A rule is **MUST** only when its absence blinds operators in a way that prevents detection or diagnosis of an outage regardless of scale. These rules verify **design-time commitments** — what a spec or plan must state — rather than code patterns; `/{project}:analyze` enforces them against feature artifacts. Logging and audit-trail rules live in `security-backend.md` §BE-LOG, not here.

Projects without a backend can pin this file in `.govern.toml` or set `[rules] surfaces` to exclude the backend surface, and it will be skipped during `govern` updates and reviews.

## BE-METRIC — Metrics

### BE-METRIC-001

> Each request-handling path SHOULD commit to RED metrics — request **r**ate, **e**rror rate, and **d**uration (latency distribution) — so a degradation is detectable from request-side signals.

**Rationale:** Rate, errors, and duration are the minimal signal set that distinguishes "healthy," "slow," and "failing" for any request handler. A path with no RED metrics is invisible on a dashboard: an error spike or latency regression is noticed only when a user complains.

**Verification:** Any spec or plan that introduces a request-handling path (HTTP route, RPC handler, message consumer) SHOULD name the RED metrics it emits. Validate flags request-handling features whose plan commits to no rate/error/duration instrumentation.

**Source:** The RED method (Tom Wilkie, Weaveworks).

### BE-METRIC-002

> Each managed resource — connection pool, work queue, worker pool, cache — SHOULD commit to USE metrics: **u**tilization, **s**aturation, and **e**rror count.

**Rationale:** Resource exhaustion (a saturated pool, an unbounded queue backlog) is a leading indicator of outages that RED request metrics show only after latency has already degraded. USE metrics surface the cause before the symptom.

**Verification:** Any spec or plan that introduces a bounded resource pool, queue, or worker set SHOULD name its USE metrics. Validate flags resource-introducing plans with no saturation or utilization signal.

**Source:** The USE method (Brendan Gregg).

### BE-METRIC-003

> Metric label/tag sets SHOULD be bounded-cardinality — no unbounded or caller-controlled value (user ID, request ID, raw URL path, email) used as a label.

**Rationale:** Each distinct label combination is a separate time series; an unbounded label explodes storage and query cost and can exhaust the metrics backend — a resource-exhaustion vector adjacent to the budgets in `performance-backend.md`.

**Verification:** Any spec or plan that introduces metric labels SHOULD confirm each label is drawn from a bounded set. Validate flags plans that propose high-cardinality identifiers (user/request IDs, raw paths) as label values.

## BE-TRACE — Distributed tracing

### BE-TRACE-001

> Trace context MUST be propagated across every service boundary the feature crosses — inbound context extracted and continued, outbound context injected on every downstream call (e.g. W3C `traceparent`) — so a single request stays correlatable end-to-end.

**Rationale:** Without propagation a distributed failure fragments into disconnected per-service logs and spans, and the broken hop is invisible until an incident forces a manual reconstruction. This extends `security-backend.md` `BE-LOG-006` (correlation / trace IDs in log lines) from logs to the call graph; the absence makes cross-service failures undebuggable regardless of scale, which is why it is MUST.

**Verification:** Any spec or plan that adds or crosses a service boundary (calls another service, enqueues to or consumes from a broker) MUST commit to extracting inbound and injecting outbound trace context. Validate flags cross-service plans that do not state trace-context propagation.

**Source:** W3C Trace Context.

### BE-TRACE-002

> Significant units of work — outbound calls, database queries, expensive computation — SHOULD be wrapped in a named span carrying meaningful attributes (target, operation, result).

**Rationale:** Spans around significant work turn a trace into a latency breakdown, making the slow hop in a request obvious. Without them a trace shows that a request was slow but not where.

**Verification:** Any spec or plan that introduces significant units of work SHOULD name the spans it creates and their key attributes. Validate flags plans for externally-dependent or compute-heavy work that commit to no spans.

## BE-HEALTH — Health endpoints

### BE-HEALTH-001

> A service MUST expose a readiness signal distinct from its liveness signal, and that reflects the reachability of the dependencies it needs to serve, so an instance that cannot serve is not sent traffic.

**Rationale:** A missing readiness signal — or one that returns healthy unconditionally — lets the orchestrator route traffic to instances that cannot reach their database, cache, or upstreams, turning a bad deploy or a dependency blip into served errors with no signal. Distinguishing readiness from liveness is what makes a rollout safe; its absence ships silent bad deploys regardless of scale, which is why it is MUST.

**Verification:** Any spec or plan for a deployable service MUST commit to a readiness endpoint or signal distinct from liveness, and state which dependencies gate readiness. Validate flags service plans with no readiness signal or with readiness conflated with liveness.

**Source:** Kubernetes liveness/readiness/startup probes.

### BE-HEALTH-002

> Liveness — and, for slow-starting services, startup — probes SHOULD be distinct from readiness: liveness reflects "the process is alive and not deadlocked," not dependency health.

**Rationale:** Conflating liveness with dependency reachability causes restart storms during a downstream outage — restarting a pod cannot fix an unreachable database, so the restarts amplify the incident. A startup probe covers slow initialization without weakening the liveness check.

**Verification:** Any spec or plan that defines health probes SHOULD keep liveness independent of dependency reachability and use a startup probe for slow initialization. Validate flags plans where liveness fails on dependency unavailability. Probe intervals and timeouts are operator-tunable values governed by `configuration-cross.md` `CFG-CONST-*` / `CFG-ENV-*`.

**Source:** Kubernetes liveness/readiness/startup probes.
