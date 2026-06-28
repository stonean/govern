# Performance Rules — Backend

Enforceable performance rules for server-side request handling, data access, caching, and resource management. These rules apply to projects adopting `govern` whose surface includes a backend.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `BE-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `QUERY` (query efficiency), `CACHE` (caching), `POOL` (connection pooling), `PAYLOAD` (payload budgets), `ASYNC` (async offloading). See `specs/008-security-rules/data-model.md` for the full schema.

Performance rules default to **SHOULD** — thresholds are context-dependent, so most are advisory. A rule is **MUST** only when its absence is a denial-of-service or resource-exhaustion risk regardless of scale. These rules verify **design-time commitments** — what a spec or plan must state — rather than code patterns; `/{project}:analyze` enforces them against feature artifacts. Server-side deadlines, timeouts, retries, and circuit breakers are reliability concerns covered elsewhere, not here.

Projects without a backend can pin this file in `.govern.toml` or set `[rules] surfaces` to exclude the backend surface, and it will be skipped during `govern` updates and reviews.

## BE-QUERY — Query Efficiency

### BE-QUERY-001

> A spec or plan that fetches data related to each item of a collection MUST commit to a batched access pattern (eager-load/join, a single `IN`/`ANY` query, or a dataloader) rather than one query per item. Per-item queries inside a loop over a collection MUST NOT be used.

**Rationale:** The N+1 query pattern turns one logical read into one query per row; over a user-controlled or growing collection the query count is unbounded, exhausting the database connection pool and inflating latency linearly with result size. Batching makes the cost independent of collection length.

**Verification:** Any spec or plan that renders, serializes, or aggregates a collection together with related data (orders with their line items, users with their roles) MUST state how the related data is fetched in bulk. Validate flags collection-handling plans that describe a per-item lookup, a query inside a loop, or that omit the access-pattern commitment.

**Source:** "N+1 query problem" (ORM performance literature)

### BE-QUERY-002

> A spec or plan that introduces a query filtering, sorting, or joining on a column SHOULD name the index that supports it, or state that the column is the primary key.

**Rationale:** A filter or sort on an unindexed column forces a full scan whose cost grows with table size — fine in development, an outage in production once the table is large. Naming the index at design time makes the access path reviewable before the slow query ships.

**Verification:** Any spec or plan that introduces a non-trivial query (a `WHERE`, `ORDER BY`, or `JOIN` on a non-primary-key column) SHOULD name the supporting index. Validate emits a warning when a query design describes filtering or sorting on a column without naming an index or stating primary-key coverage.

**Source:** Use The Index, Luke; database indexing guides

### BE-QUERY-003

> A query that can return an unbounded number of rows MUST apply an explicit limit. Endpoints that return collections to clients MUST paginate per `BE-PAGE` (api-backend.md); internal queries MUST cap the result set with a named maximum.

**Rationale:** An unbounded read loads the entire matching set into memory — a single large tenant or an unfiltered query can exhaust application memory and saturate the database. An explicit limit bounds the worst case regardless of how much data exists.

**Verification:** Any spec or plan that reads a collection MUST commit to a row bound — pagination for client-facing endpoints (deferring to `BE-PAGE`), or a named maximum for internal reads. Validate flags collection reads with no limit and no pagination commitment.

**Source:** OWASP Denial of Service Cheat Sheet; `api-backend.md` §BE-PAGE

## BE-CACHE — Caching

### BE-CACHE-001

> Every cache entry MUST have an explicit time-to-live or a documented invalidation trigger, and the cache MUST be size-bounded (a maximum entry count or an eviction policy). Caches that grow without expiry, invalidation, or eviction MUST NOT be used.

**Rationale:** A cache with neither expiry nor bounded size grows until it exhausts memory, and serves stale data indefinitely once the source changes. A named TTL or invalidation rule plus a size bound caps both memory and staleness.

**Verification:** Any spec or plan that introduces a cache MUST name its TTL and/or the event that invalidates an entry, and MUST bound its size (max entries / eviction policy). Validate flags caching plans that omit the expiry/invalidation commitment or describe an unbounded in-memory map used as a cache.

**Source:** OWASP Denial of Service Cheat Sheet; caching-strategy literature

### BE-CACHE-002

> Cache keys SHOULD incorporate every input that affects the cached value, including the authorization scope (user, tenant, role) when the value is principal-specific.

**Rationale:** A key that omits an input that changes the value serves the wrong result — and when the omitted input is the principal, it serves one user's data to another. The authorization requirement itself is `BE-AUTHZ-002` / `BE-AUTHZ-005` in security-backend.md; this rule is the caching-design reminder that keys must be scope-complete.

**Verification:** Any spec or plan that caches principal-specific or input-dependent data SHOULD enumerate the inputs composing the cache key and confirm the authorization scope is among them. Validate emits a warning when a cache over per-user or per-tenant data describes a key that omits the scoping dimension.

**Source:** OWASP Authorization Cheat Sheet; cache-key design guidance

### BE-CACHE-003

> Caches fronting expensive computation SHOULD protect against stampede (thundering herd): concurrent misses for the same key SHOULD be coalesced via single-flight, a short lock, or probabilistic early expiration.

**Rationale:** When a hot key expires, every concurrent request misses at once and recomputes the same expensive value simultaneously, spiking load on the very backend the cache exists to protect. Coalescing collapses the herd to a single recomputation.

**Verification:** Any spec or plan that caches the result of an expensive or slow operation (an aggregate query, a third-party call) SHOULD describe its stampede protection. Validate emits a warning when a cache over expensive work omits any coalescing or early-expiration strategy.

**Source:** "Cache stampede" / thundering-herd mitigation literature

## BE-POOL — Connection Pooling

### BE-POOL-001

> Connections to databases, caches, and external services MUST be obtained from a pool, not opened per request. The pool's maximum size MUST be a named constant (per `CFG-CONST-003`), not a bare literal.

**Rationale:** Opening a connection per request adds handshake latency to every call and, under load, opens more connections than the backend can accept — exhausting database connection slots and collapsing the service. A bounded pool caps concurrent connections and amortizes setup cost.

**Verification:** Any spec or plan that connects to a database or external service MUST commit to pooled connections and name the pool-size constant. Validate flags data-access plans that describe per-request connection creation or an unnamed/bare-literal pool size.

**Source:** CIS Database Benchmarks; connection-pooling guides; `configuration-cross.md` §CFG-CONST

### BE-POOL-002

> Connection-pool acquisition MUST use a bounded wait timeout. A caller that cannot obtain a connection within the timeout MUST fail fast with an error rather than block indefinitely.

**Rationale:** An unbounded acquisition wait turns pool saturation into a pile-up: requests block holding threads, the thread pool exhausts, and the outage cascades. A bounded wait converts saturation into fast, sheddable errors that protect the rest of the system.

**Verification:** Any spec or plan that introduces a connection pool MUST name the acquisition-timeout value (a named constant per `CFG-CONST-003`) and the fail-fast behavior on timeout. Validate flags pool configurations that omit an acquisition timeout or describe blocking until a connection is free.

**Source:** "Release It!" (bulkhead and fail-fast patterns)

### BE-POOL-003

> Pooled connections MUST be returned to the pool on every code path, including error and early-return paths. Acquisition and release MUST be paired by a construct that runs on scope exit (context manager, `defer`, try-with-resources, RAII).

**Rationale:** A connection leaked on an error path is never reused; enough leaks drain the pool and the service stops serving while orphaned connections sit idle. Pairing release to scope exit via an unwinding construct makes the leak structurally impossible.

**Verification:** Any spec or plan that acquires a pooled resource MUST commit to a scope-bound release mechanism that fires on error paths. Validate flags resource-acquisition plans that describe manual release without an unwinding guarantee, or that release only on the success path.

**Source:** "Release It!"; resource-management idioms

## BE-PAYLOAD — Payload Budgets

### BE-PAYLOAD-001

> Response payloads MUST have a bounded maximum size. Endpoints that return collections MUST paginate per `BE-PAGE` (api-backend.md) rather than return an unbounded list in a single response.

**Rationale:** An unbounded response serializes the entire result set into memory on the server and forces the client to receive it all — a large tenant or an unfiltered list exhausts server memory, saturates bandwidth, and can OOM the client. A size bound (via pagination or an explicit cap) makes the worst-case response independent of how much data exists.

**Verification:** Any spec or plan that returns a collection or a variable-size body MUST commit to a maximum response size or to pagination. Validate flags response designs that return unbounded lists or omit a size/pagination bound. Pagination commitments defer to `BE-PAGE`.

**Source:** OWASP Denial of Service Cheat Sheet; `api-backend.md` §BE-PAGE

### BE-PAYLOAD-002

> Endpoints returning large or composite resources SHOULD support field selection (sparse fieldsets) so clients can request only the fields they need.

**Rationale:** Returning every field of a large resource on every request wastes serialization time, bandwidth, and client memory when the caller needs only a few. Field selection lets the cost track what the client actually uses.

**Verification:** Any spec or plan that returns a large or deeply-nested resource SHOULD describe a field-selection mechanism (a `fields=` parameter, a GraphQL selection set, a projection). Validate emits a warning when a large-resource endpoint offers no way to limit returned fields.

**Source:** Google Cloud API Design Guide (partial responses); JSON:API sparse fieldsets

### BE-PAYLOAD-003

> Large text or JSON responses SHOULD be compressed (gzip, brotli, or zstd) when the client advertises support via `Accept-Encoding`.

**Rationale:** Text and JSON compress several-fold; sending them uncompressed wastes bandwidth and adds latency for every client, especially on mobile networks. Negotiated compression is a near-free latency win.

**Verification:** Any spec or plan covering response handling or edge configuration that serves sizable text/JSON SHOULD commit to negotiated compression. Validate emits a warning when a text/JSON-serving surface omits any compression commitment.

**Source:** RFC 9110 §8.4 (Content-Encoding); web-performance guidance

## BE-ASYNC — Async Offloading

### BE-ASYNC-001

> Work that is slow, unbounded, or dependent on a third party MUST NOT block the synchronous request path. It MUST be offloaded to a background job or queue, with the client receiving an asynchronous acknowledgment (e.g., `202 Accepted` per `BE-STATUS-001`) and a way to observe completion.

**Rationale:** Holding a request thread while a slow or third-party operation runs ties up a worker and a connection for the whole duration; enough concurrent slow requests exhaust the worker pool and the service stops accepting new work. Offloading frees the request path to return immediately and lets the slow work proceed under its own concurrency limit.

**Verification:** Any spec or plan that introduces slow processing (report generation, media transcoding, bulk import, an outbound third-party call on the request path) MUST commit to offloading it to a background job and returning an async acknowledgment. Validate flags request handlers that perform unbounded or third-party-dependent work synchronously without an offload commitment.

**Source:** "Release It!" (async patterns); `api-backend.md` §BE-STATUS
