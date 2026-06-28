# 034 — Backend performance rules Plan

Implements [034 — Backend performance rules](spec.md).

## Overview

Author `framework/rules/performance-backend.md` (13 rules across 5 categories) following the canonical rule schema, and register it in the `/govern` Shared Files manifest so it installs under the backend surface. Pure markdown-tier change — no runtime. Verification clauses are design-time commitments (what a spec/plan MUST state), consistent with `/gov:analyze` auditing artifacts. Default severity SHOULD; MUST only for the DoS/exhaustion-regardless-of-scale cases.

## Technical Decisions

### Categories and abbreviations

`BE-` prefix (backend surface), five categories declared in the file header, all disjoint from `security-backend.md` and `api-backend.md`:

`QUERY` (query efficiency), `CACHE` (caching), `POOL` (connection pooling), `PAYLOAD` (payload budgets), `ASYNC` (async offloading).

### Rule list (final)

| ID | Sev | Statement gist |
| --- | --- | --- |
| `BE-QUERY-001` | MUST | No N+1: a plan that fetches related data for a collection MUST batch (eager-load / join / `IN` / dataloader), not query per item |
| `BE-QUERY-002` | SHOULD | A plan that filters/sorts/joins on a column SHOULD name the supporting index (or state the column is the PK) |
| `BE-QUERY-003` | MUST | A query that can return an unbounded number of rows MUST apply an explicit limit (cite `BE-PAGE` for endpoint pagination) |
| `BE-CACHE-001` | MUST | Cache entries MUST have an explicit TTL or a documented invalidation trigger; never-expiring unbounded caches are forbidden |
| `BE-CACHE-002` | SHOULD | Cache keys SHOULD incorporate every input that affects the value, including auth scope/tenant (cite `BE-AUTHZ-002`/`BE-AUTHZ-005` for the authorization requirement) |
| `BE-CACHE-003` | SHOULD | Caches fronting expensive work SHOULD coalesce concurrent misses (single-flight / early expiration) to prevent stampede |
| `BE-POOL-001` | MUST | Connections to DBs/external services MUST be pooled, not opened per request; pool size MUST be a named constant (cite `CFG-CONST-003`) |
| `BE-POOL-002` | MUST | Pool acquisition MUST have a bounded timeout so a saturated pool fails fast rather than hanging requests |
| `BE-POOL-003` | MUST | Pooled connections MUST be released on every path, including error paths |
| `BE-PAYLOAD-001` | MUST | Response payloads MUST have a bounded maximum size; collection endpoints MUST paginate (cite `BE-PAGE`) rather than return unbounded lists |
| `BE-PAYLOAD-002` | SHOULD | Endpoints returning large resources SHOULD support field selection / sparse responses |
| `BE-PAYLOAD-003` | SHOULD | Large text/JSON responses SHOULD be compressed when the client advertises support |
| `BE-ASYNC-001` | MUST | Slow / unbounded / third-party work MUST NOT block the synchronous request path; offload to a background job with an async acknowledgment (cite `BE-STATUS-001` `202`) |

8 MUST (all DoS/exhaustion regardless of scale), 5 SHOULD (tunable efficiency / correctness-cross-referenced). `BE-ASYNC` carries one rule; background-job retry-safety/idempotency and request deadlines/timeouts are **reliability** concerns deferred to the future `reliability-backend.md` set (per the spec's Boundaries), so they are intentionally absent here.

### Verification style

Every Verification clause targets a **design-time commitment** in a spec or plan (e.g., "any plan that introduces a list-rendering endpoint MUST state its query strategy and the indexes it relies on; validate flags list-endpoint plans that omit the query/index commitment"), never a source-code grep — matching how `/gov:analyze` audits artifacts.

### Installation

Add one row to the `### govern-owned shared files` table in `framework/bootstrap/govern.md`: `framework/rules/performance-backend.md → specs/rules/performance-backend.md` (strategy: update). Spec 033's surface filter selects it under `backend` automatically via the `-backend.md` suffix — no 033 change needed. No `data-model.md`: the schema/format lives in 008; the category abbreviations live in the file header per the constitution's per-file policy.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/rules/performance-backend.md` | Create | The 13-rule performance rule set |
| `framework/bootstrap/govern.md` | Modify | Add the rule file to the Shared Files manifest |

## Trade-offs

- **Deadlines/timeouts/retries/circuit-breakers deferred** to a future `reliability-backend.md` — keeps this set purely throughput/latency (resolved at clarify).
- **Background-job idempotency** (retry-safety) deferred to reliability as well — `api-backend.md` `BE-IDEMP` already covers HTTP idempotency; the job-level version is reliability's domain.
- **Scope-complete cache keys is SHOULD, not MUST** — the unconditional authorization requirement (tenant isolation) is already `security-backend.md` `BE-AUTHZ-002`/`BE-AUTHZ-005` (MUST); the perf-cache rule cites it and stays SHOULD to keep every MUST in this file a DoS/exhaustion case.
- **No numeric thresholds** — rules require a bound to exist and be named (deferring the value to `CFG-CONST-003` constants), rather than hard-coding numbers that would be wrong for some projects.
