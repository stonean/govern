# API Contract Rules — Backend

Enforceable rules for the shape, stability, and documentation of HTTP and RPC APIs. These rules apply to any project that exposes a programmatic interface (REST, JSON-RPC, gRPC, GraphQL) to clients outside its own process.

These rules cover **contract quality**, not security. For HTTP-surface security concerns (security headers, CORS, rate limits, method allowlisting, content-type validation, webhook signing, network exposure) see `security-backend.md` §BE-API.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `BE-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `SCHEMA` (schema publication and source-of-truth), `APIVER` (versioning and deprecation), `ERRENV` (error response envelope), `STATUS` (HTTP status code discipline), `PAGE` (pagination), `IDEMP` (idempotency), `COMPAT` (backwards compatibility). See `specs/008-security-rules/data-model.md` for the full schema.

Projects without a programmatic API can pin this file in `.govern.toml` to skip it during `govern` updates.

## BE-SCHEMA — Schema Publication

### BE-SCHEMA-001

> Every public HTTP or RPC API MUST publish a machine-readable schema describing every endpoint, request shape, response shape, and error: OpenAPI 3.x for REST, `.proto` files for gRPC, GraphQL SDL for GraphQL, JSON Schema for JSON-RPC. The schema MUST be served from a documented URL on the same origin as the API (e.g., `/openapi.json`, `/schema.graphql`) AND committed to the repository.

**Rationale:** A machine-readable schema is the single source of truth that client SDKs, mock servers, contract tests, and human documentation all derive from. Without it, every consumer reverse-engineers the API independently and integration breaks accumulate silently. Serving it from the API origin makes discovery automatic.

**Verification:** Any spec or plan that introduces a public HTTP/RPC endpoint MUST name the schema artifact path AND the URL it is served from. Validate flags endpoint specs that omit the schema commitment, and flags the absence of a schema file at the named path.

**Source:** OpenAPI Specification, gRPC IDL conventions, GraphQL Spec §3, JSON Schema

### BE-SCHEMA-002

> The published schema MUST be generated from the same code or annotations that implement the endpoints — not maintained as a parallel document. Acceptable generation sources include typed handler signatures and decorators (FastAPI, NestJS, ASP.NET, Spring), explicit `.proto`/SDL files that the server is built against, or build-time extraction from a typed routing layer. Hand-written schema files that the implementation does not derive from MUST NOT be used.

**Rationale:** Parallel schemas drift the moment a parameter is renamed, an optional field is made required, or a status code is added — and the drift is silent because the schema never executes. Code-derived schemas fail loudly at build time when implementation and contract diverge, and they make the schema as trustworthy as the code itself.

**Verification:** Any spec or plan that introduces or modifies an API endpoint MUST commit to a code-derived schema generation step in the build. Validate flags plans that propose hand-maintained schema files, and flags affected-files snippets where the schema and the endpoint annotations are edited independently of each other.

**Source:** OpenAPI Initiative best practices, "API-first" design literature

### BE-SCHEMA-003

> The build MUST fail when the regenerated schema differs from the committed schema. The check MUST run in CI on every pull request.

**Rationale:** Without an enforcing check, generated schemas drift the moment a developer skips the regeneration step. A CI gate makes the regeneration step impossible to skip and turns schema drift into a build failure instead of a production discovery.

**Verification:** Any spec or plan covering API endpoints OR CI/build configuration MUST commit to a `schema regenerate && diff --exit-code` step in the CI pipeline. Validate flags CI specs that omit the schema-diff check, and flags API specs that omit the CI-gate commitment.

**Source:** OpenAPI Initiative governance guides

## BE-APIVER — Versioning and Deprecation

### BE-APIVER-001

> Every public API MUST declare its versioning strategy explicitly: URI path versioning (`/v1/`, `/v2/`), media-type versioning (`Accept: application/vnd.app.v2+json`), or header versioning (`API-Version: 2`). The strategy MUST be documented in `specs/system.md` and applied consistently across all endpoints of the API.

**Rationale:** Without a declared strategy, breaking changes have no migration path — every consumer breaks simultaneously when an endpoint changes. A declared strategy lets old and new versions coexist while consumers migrate at their own pace.

**Verification:** Any spec or plan that introduces a public API MUST name the versioning strategy and reference its declaration in `specs/system.md`. Validate flags public-API specs that omit the strategy commitment, and flags inconsistencies where some endpoints version one way and others a different way.

**Source:** Microsoft REST API Guidelines, Google Cloud API Design Guide

### BE-APIVER-002

> Deprecated endpoints MUST emit the `Deprecation` and `Sunset` HTTP response headers (RFC 8594, RFC 9745). `Sunset` MUST carry a future date past which the endpoint will be removed; `Deprecation` MUST carry the date the deprecation was announced. The corresponding schema definition MUST mark the operation deprecated.

**Rationale:** Consumers cannot plan migrations they do not know about. Emitting the headers on every response from a deprecated endpoint guarantees visibility in client logs, monitoring dashboards, and integration tests. The schema annotation makes generated client SDKs surface the deprecation at compile time.

**Verification:** Any spec or plan that removes or deprecates an endpoint MUST commit to (a) `Deprecation` + `Sunset` headers with concrete dates, and (b) the schema-level deprecation annotation. Validate flags deprecation specs that omit either commitment.

**Source:** RFC 8594, RFC 9745, OpenAPI Specification

## BE-ERRENV — Error Response Envelope

### BE-ERRENV-001

> Error responses from HTTP APIs MUST use a single documented envelope shape across all endpoints. The default SHOULD be RFC 7807 `application/problem+json` (with at least `type`, `title`, `status`, `detail` fields). A project-specific envelope is acceptable when documented in `specs/system.md` and used uniformly.

**Rationale:** Without a uniform shape, every consumer writes per-endpoint error parsing — and gets it wrong for endpoints added later. A single shape lets clients write one error handler that works everywhere, and lets the API evolve error categories without breaking consumers.

**Verification:** Any spec or plan that introduces an HTTP API endpoint MUST commit to the project's error envelope shape and reference its definition in `specs/system.md`. Validate flags endpoint specs that describe ad-hoc error formats (per-endpoint shapes, mixed error types, naked strings) without naming the project envelope.

**Source:** RFC 7807 (Problem Details for HTTP APIs), Google Cloud API Design Guide §error-model

### BE-ERRENV-002

> The error envelope MUST include a machine-readable, stable error code distinct from the HTTP status code (e.g., `code: "USER_NOT_FOUND"`, `code: "PAYMENT_DECLINED_INSUFFICIENT_FUNDS"`). Error codes MUST be documented in the schema, MUST be stable across versions, and MUST NOT be derived from or parsed out of the human-readable message.

**Rationale:** HTTP status codes are coarse — `400` covers thousands of failure modes. Stable error codes let clients write conditional logic ("retry on `RATE_LIMITED`, surface to user on `VALIDATION_FAILED`") that survives message rewording and localization. Deriving codes from messages couples the two and breaks clients when copy is edited.

**Verification:** Any spec or plan that introduces an HTTP API endpoint MUST commit to documented error codes for each named failure mode, with the code values declared in the schema. Validate flags endpoint specs that describe errors only by HTTP status or only by message text without a stable code.

**Source:** Stripe API error documentation, Google Cloud API Design Guide

## BE-STATUS — HTTP Status Code Discipline

### BE-STATUS-001

> HTTP status codes MUST be used with their RFC 9110 semantics. Required mappings: `200` successful retrieval, `201` resource creation (with `Location` header naming the new resource), `202` accepted-but-not-yet-processed, `204` successful no-content, `400` malformed request, `401` missing/invalid authentication, `403` authenticated-but-not-authorized, `404` resource-not-found, `409` conflict, `410` permanently-removed, `422` well-formed-but-semantically-invalid input, `429` rate-limited, `500` unexpected server error, `503` unavailable-with-retry. Endpoints MUST NOT collapse multiple outcomes into `200` with an `error` field in the body.

**Rationale:** HTTP intermediaries — load balancers, caches, retry middleware, monitoring, on-call alerting — all act on status codes. An endpoint that returns `200` with an `error` field bypasses every layer of the HTTP-aware stack and forces every consumer (and operator) to parse the body to know what happened. Correct status codes let off-the-shelf infrastructure do its job.

**Verification:** Any spec or plan that introduces an endpoint MUST name the success status code AND each failure status code mapped to its failure mode. Validate flags endpoint specs that describe errors via `200 OK` with an `error` field, that conflate `401` and `403`, or that omit the status-code mapping.

**Source:** RFC 9110 §15, Microsoft REST API Guidelines

## BE-PAGE — Pagination

### BE-PAGE-001

> List endpoints that can return more than a small fixed maximum number of items MUST paginate. The pagination strategy MUST be documented per endpoint and SHOULD default to cursor-based pagination (opaque continuation tokens) for any collection that supports concurrent inserts or deletes. Offset-based pagination MAY be used only for static or append-only-at-tail collections.

**Rationale:** Unpaginated list endpoints are a DoS vector (large responses, slow queries) and an OOM-on-client risk. Offset pagination loses items when new rows arrive between page reads, and shows duplicates when rows are deleted; cursor pagination is stable under concurrent writes. Defaulting to cursors forces the safe choice without per-endpoint debate.

**Verification:** Any spec or plan that introduces a list endpoint MUST commit to a pagination strategy, a default page size, and a maximum page size — all as named constants per `CFG-CONST-003`. Validate flags list-endpoint specs that omit pagination or that propose offset pagination on collections that accept concurrent inserts.

**Source:** Stripe API pagination, GraphQL Cursor Connections Specification

### BE-PAGE-002

> Paginated responses MUST include an envelope distinguishing the page from the items, with an explicit continuation field: e.g., `{"data": [...], "next_cursor": "..." | null, "has_more": true | false}`. The presence of more pages MUST be expressed by a dedicated field — not inferred from a full-page item count.

**Rationale:** Inferring "more pages exist when `len(items) == page_size`" is wrong at the exact boundary where the total happens to equal the page size — the client requests an empty page on the next call. A dedicated `has_more` (or non-null `next_cursor`) makes the boundary unambiguous and saves one round trip.

**Verification:** Any spec or plan that introduces a paginated endpoint MUST name the response envelope including the page-continuation field. Validate flags paginated-endpoint specs that infer continuation from item count or that omit a continuation field.

**Source:** Stripe API pagination, GraphQL Cursor Connections Specification

## BE-IDEMP — Idempotency

### BE-IDEMP-001

> Endpoints that create resources or trigger side-effecting state changes via `POST` MUST accept an `Idempotency-Key` request header. The server MUST persist key→response mappings for at least 24 hours and return the cached response on retry with the same key (matching auth principal, matching request body hash). Retries that present the same key with a divergent body MUST return `409 Conflict`.

**Rationale:** Networks fail mid-request. Without idempotency, a client that did not receive a response cannot safely retry — every retry risks a duplicate side effect (double-charge, double-send, duplicate order). The Stripe-pioneered `Idempotency-Key` pattern makes safe retry possible and is now the industry-standard contract for side-effecting endpoints.

**Verification:** Any spec or plan that introduces a side-effecting `POST` endpoint MUST commit to `Idempotency-Key` support, the persistence window, and the divergent-body behavior. Validate flags side-effecting-POST specs that omit idempotency support, and flags persistence windows that are shorter than the documented client retry policy.

**Source:** Stripe API idempotency documentation, IETF draft-ietf-httpapi-idempotency-key-header

### BE-IDEMP-002

> `PUT` and `DELETE` operations MUST be idempotent in the HTTP sense: repeated identical requests MUST produce the same observable resource state and the same response (or a `404` on a `DELETE` retry once the resource is gone). Per-call side effects (counter increments, notification sends, audit-log entries) MUST NOT be emitted on `PUT`/`DELETE` retries.

**Rationale:** Idempotency on `PUT`/`DELETE` is a contract of the HTTP method per RFC 9110 §9.2.2. Intermediate proxies, retry middleware, and clients all assume it. Per-call side effects break the contract and produce confusing duplicate-notification incidents.

**Verification:** Any spec or plan that introduces a `PUT` or `DELETE` endpoint MUST commit to operation idempotency including side-effect deduplication for retries. Validate flags `PUT`/`DELETE` specs that describe per-call counter increments, notification sends, or audit-row inserts without dedup.

**Source:** RFC 9110 §9.2.2

## BE-COMPAT — Backwards Compatibility

### BE-COMPAT-001

> Within a major version, schema changes MUST be additive only. Removing a field, renaming a field, narrowing a type (`string` → `enum`), making an optional field required, or adding a required request field MUST NOT happen without a major version bump. Adding new optional request fields, new response fields, new enum values for response-only fields, and new endpoints is permitted.

**Rationale:** Every consumer of the API has code that depends on the current shape. Removing or renaming fields breaks clients silently — the client deserializes a response and gets `null` where it expected a value, or the server rejects a request that worked yesterday. Additive evolution within a major version is the only contract that lets clients upgrade lazily.

**Verification:** Any spec or plan that modifies an existing API endpoint MUST classify the change as additive (compatible) or breaking (requires major version). Validate runs schema-diff against the committed schema and flags subtractive or narrowing changes within a major version; flags plans that propose breaking changes without a major-version commitment.

**Source:** Semantic Versioning for APIs, Google Cloud API Design Guide §compatibility
