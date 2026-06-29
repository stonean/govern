# 039 — Backend reliability rules Plan

Implements [039 — Backend reliability rules](spec.md).

## Overview

A rule-introducing, markdown-tier feature — the 034/037/038 path exactly. It adds one rule file, `framework/rules/reliability-backend.md`, on the **existing** `BE` surface (so no `scripts/lint-rule-ids.sh` change and no `data-model.md`), and wires it into the `/govern` Shared Files manifest. The §Shared Files note is already count-free (036), so no count to update. Landing this set resolves 034's forward-reference to a future `reliability-backend.md` (AC #8).

Five categories ship — `TIMEOUT`, `RETRY`, `BREAKER`, `DRAIN`, `BULK` — with eight rules, three of them MUST. Verification is **design-time commitment** framing enforced by `/gov:analyze` against feature artifacts, matching 034/037/038.

## Technical Decisions

### The rule file — `framework/rules/reliability-backend.md`

Modeled on `performance-backend.md` / `observability-backend.md` / `concurrency-backend.md`:

- **Header.** Title `# Reliability Rules — Backend`; an intro scoping it to server-side resilience under partial failure; the RFC 2119 note; the ID-format / category-declaration line stating IDs follow `BE-{CATEGORY}-{NNN}` with categories `TIMEOUT` (timeouts & deadlines), `RETRY` (retries), `BREAKER` (circuit breakers), `DRAIN` (graceful shutdown), `BULK` (backpressure), and a pointer to `specs/008-security-rules/data-model.md` for the schema; a default-**SHOULD** paragraph (MUST reserved for scale-independent availability / cascading-failure risks) noting these verify design-time commitments enforced by `/{project}:analyze`; and the standard backend pin/surface note.
- **Categories disjoint** from the other backend files (verified: `TIMEOUT`/`RETRY`/`BREAKER`/`DRAIN`/`BULK` appear in none).

### Rule set

Eight rules across five `## BE-{CATEGORY}` sections. Verification is phrased as a design-time commitment a spec/plan must make.

| ID | Sev | Statement gist |
| --- | --- | --- |
| `BE-TIMEOUT-001` | **MUST** | Every outbound/downstream/IO call has a bounded timeout — no unbounded waits (an unbounded wait exhausts threads/connections and cascades). |
| `BE-TIMEOUT-002` | SHOULD | An end-to-end request deadline is established and propagated to downstream calls, so per-call timeouts respect the caller's remaining budget. |
| `BE-RETRY-001` | **MUST** | Automatic retries are bounded (max attempts) with exponential backoff + jitter, and apply only to idempotent operations (cite `api-backend.md` `BE-IDEMP`) — unbounded/un-jittered retries are a retry storm that amplifies an outage. |
| `BE-RETRY-002` | SHOULD | Retries are budgeted/coordinated with circuit breaking (retry budget or token bucket) so a degraded downstream is not hammered. |
| `BE-BREAKER-001` | SHOULD | Calls to a failure-prone downstream are guarded by a circuit breaker that opens on a failure threshold, fails fast while open, and half-opens to probe recovery. |
| `BE-DRAIN-001` | **MUST** | On a termination signal the service stops accepting new work and drains in-flight work within a bounded grace period before exiting — no dropped in-flight requests on deploy. |
| `BE-DRAIN-002` | SHOULD | Readiness flips to not-ready at the start of shutdown so the orchestrator stops routing before the drain begins (cite `observability-backend.md` `BE-HEALTH-001`). |
| `BE-BULK-001` | SHOULD | Failure-prone or resource-heavy work is isolated behind a bulkhead (bounded concurrency / bounded queue) and sheds load when saturated rather than queueing unboundedly, to degrade gracefully. |

Cross-references, not restatements: `BE-RETRY-001` cites `api-backend.md` `BE-IDEMP`; `BE-DRAIN-002` cites `observability-backend.md` `BE-HEALTH-001`; `BE-BULK-001` cites `performance-backend.md` `BE-ASYNC`/`BE-POOL-*`; `BE-TIMEOUT`/`BE-BREAKER` cite `performance-backend.md` `BE-POOL-002` for the pool-acquisition-timeout interaction; tunables (timeout durations, retry counts, breaker thresholds) cite `configuration-cross.md` `CFG-*`.

### Manifest wiring — `framework/bootstrap/govern.md`

Add a `reliability-backend.md` → `specs/rules/reliability-backend.md` row to the `### govern-owned shared files` table, slotted between `quality-cross.md` and `security-backend.md` (alphabetical). The `-backend.md` suffix means 024's loader selects it under the `backend` surface and 033's filter includes it. The §Shared Files note is count-free, so no count edit.

### What this feature does NOT touch

- **`scripts/lint-rule-ids.sh`** — `BE` is already registered; `BE-TIMEOUT-001` etc. match the regex unchanged.
- **`data-model.md`** — no new surface and no new schema; categories are declared in the file header (016 policy), schema is 008's. No data-model artifact (matching 034/037/038).
- **`performance-backend.md`** — `BE-POOL-002` is cited, not moved; 034's body already forward-references this set, so no edit to 034 is required (the forward-reference resolves by this file existing).

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/rules/reliability-backend.md` | Create | The reliability rule set (`TIMEOUT`/`RETRY`/`BREAKER`/`DRAIN`/`BULK`, eight rules, three MUST) |
| `framework/bootstrap/govern.md` | Modify | Add the manifest row (between `quality-cross` and `security-backend`) |

## Trade-offs

- **Five categories; `DEADLINE` folded into `TIMEOUT`.** Per-call timeout and end-to-end deadline propagation are the same concern (bounding the wait) at two scopes — one category, two rules — rather than a sixth category (clarify resolution).
- **`BULK` kept.** Backpressure/load-shedding is the graceful-degradation primitive named by the constitution's reliability principle and fits the design-time-commitment model; it cites the performance set where it overlaps rather than re-deriving (clarify resolution).
- **Three MUSTs.** Unbounded waits (`TIMEOUT-001`), retry storms (`RETRY-001`), and no-drain deploys (`DRAIN-001`) are the absences that cause availability loss or cascading failure regardless of scale; breaker adoption, deadline propagation, retry budgeting, and bulkheading are contextual (SHOULD).
- **Known limitation.** Analyze-time verification checks that a spec/plan *commits* to these resilience measures; it cannot prove the timeout is actually wired or the drain actually waits — that residual is for `/gov:review` and tests, the standard design-time-rule limitation.

## Cross-spec impact

The spec/plan reference 008, 016, 024, 033, 034 (deps) and cite `BE-IDEMP` (api-backend), `BE-POOL-002`/`BE-ASYNC` (performance-backend), `BE-HEALTH-001` (observability-backend, 037), `CFG-*` (configuration-cross). None need an edit: all are cited, not changed. 034 forward-references this set in prose; that reference resolves by this file existing — no 034 edit needed (and editing a `done` 034 body would trip its own back-edge). 024/033 already select `-backend.md` files. Informational; does not block.
