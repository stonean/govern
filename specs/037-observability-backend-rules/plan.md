# 037 — Backend observability rules Plan

Implements [037 — Backend observability rules](spec.md).

## Overview

A rule-introducing, markdown-tier feature — the 034-performance-backend path exactly. It adds one rule file, `framework/rules/observability-backend.md`, on the **existing** `BE` surface (so no `scripts/lint-rule-ids.sh` change and no `data-model.md` — unlike 036, which introduced a new `QUAL` surface), and wires it into the `/govern` Shared Files manifest. The §Shared Files note is already count-free (036), so no count to update.

Three categories ship — `METRIC`, `TRACE`, `HEALTH` — with six rules, two of them MUST. Verification is **design-time commitment** framing (what a spec/plan must state), enforced by `/gov:analyze` against feature artifacts, matching how the 034 performance rules work.

## Technical Decisions

### The rule file — `framework/rules/observability-backend.md`

Modeled on `performance-backend.md` (same surface, same analyze-time design-time-commitment framing):

- **Header.** Title `# Observability Rules — Backend`; an intro scoping it to server-side observability beyond logging; the RFC 2119 note; the ID-format / category-declaration line stating IDs follow `BE-{CATEGORY}-{NNN}` with categories `METRIC` (metrics), `TRACE` (distributed tracing), `HEALTH` (health endpoints), and a pointer to `specs/008-security-rules/data-model.md` for the schema; a default-**SHOULD** paragraph (MUST reserved for detection/diagnosis-blocking absences) noting these verify design-time commitments enforced by `/{project}:analyze`; and the standard backend pin/surface note (pin in `.govern.toml`, or exclude the backend surface via `[rules] surfaces`).
- **Categories disjoint** from `security-backend.md`, `api-backend.md`, `performance-backend.md` (verified: `METRIC`/`TRACE`/`HEALTH` appear in none).

### Rule set

Six rules across three `## BE-{CATEGORY}` sections. Verification is phrased as a design-time commitment a spec/plan must make (not a code grep), per AC #4.

| ID | Sev | Statement gist |
| --- | --- | --- |
| `BE-METRIC-001` | SHOULD | Each request-handling path commits to RED metrics (request **r**ate, **e**rror rate, **d**uration). |
| `BE-METRIC-002` | SHOULD | Each managed resource pool / queue / worker set commits to USE metrics (utilization, saturation, errors). |
| `BE-METRIC-003` | SHOULD | Metric label sets are bounded-cardinality — no unbounded/user-controlled values as labels (cite `performance-backend.md` for the exhaustion angle). |
| `BE-TRACE-001` | **MUST** | Trace context is propagated across every service boundary the feature crosses (extract inbound, inject outbound); its absence makes distributed failures undebuggable. Extends/cites `security-backend.md` `BE-LOG-006` (correlation/trace-context in logs). |
| `BE-TRACE-002` | SHOULD | Significant work (external calls, DB queries, expensive computation) is wrapped in a named span with meaningful attributes. |
| `BE-HEALTH-001` | **MUST** | The service exposes a readiness signal distinct from liveness, and readiness reflects required-dependency reachability — so a not-ready instance is not routed traffic (its absence ships silent bad deploys). |
| `BE-HEALTH-002` | SHOULD | Liveness (and, for slow-start services, startup) probes are distinct from readiness — liveness means "process alive," not "dependencies healthy," to avoid restart loops on dependency blips. |

Operator-tunable values these rules imply (scrape interval, probe timeout) are governed by `configuration-cross.md` `CFG-*` — the observability rule requires the value to *exist*; `CFG-*` governs how it is named/validated. Cited, not restated.

### Manifest wiring — `framework/bootstrap/govern.md`

Add `| \`framework/rules/observability-backend.md\` | \`specs/rules/observability-backend.md\` |` to the `### govern-owned shared files` table, slotted between `configuration-cross.md` and `performance-backend.md` (alphabetical). The `-backend.md` suffix means 024's loader selects it under the `backend` surface and 033's surface filter includes it when `backend` is configured — no change needed in either. The §Shared Files note is count-free, so no count edit.

### What this feature does NOT touch

- **`scripts/lint-rule-ids.sh`** — `BE` is already a registered surface; the regex accepts `BE-METRIC-001` etc. unchanged.
- **`data-model.md`** — no new surface and no new schema; the file follows 008's rule schema and declares its categories in its own header (016 policy). No data-model artifact (matching 034).

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/rules/observability-backend.md` | Create | The observability rule set (`METRIC`/`TRACE`/`HEALTH`, six rules, two MUST) |
| `framework/bootstrap/govern.md` | Modify | Add the manifest row (between `configuration-cross` and `performance-backend`) |

## Trade-offs

- **Three categories vs. five.** Chose `METRIC`/`TRACE`/`HEALTH`; deferred `SLO`/`ALERT` (clarify resolution) — they are operational policy, not per-feature design commitments `/gov:analyze` can check against a spec/plan. The `BE-{CATEGORY}` grammar admits them later.
- **Analyze-time vs. review-time verification.** Chose analyze-time design-time commitments (like 034), not code-pattern review (like 036's `QUAL-STUB-001`). Observability is something a spec/plan *commits to* up front; the absence of a metric or probe is a planning gap, caught best against artifacts.
- **Two MUSTs only.** Readiness-distinct-from-liveness and trace-context propagation are the two absences that blind operators regardless of scale; everything else is contextual coverage (SHOULD). Keeping the MUST set tight avoids forcing instrumentation that may not fit every feature.
- **Known limitation.** `/gov:analyze` checks that a spec/plan *commits* to these; it cannot verify the commitment is actually implemented in code — that residual gap is what `/gov:review` and tests cover, the same limitation every design-time rule carries.

## Cross-spec impact

The spec/plan reference 008, 016, 024, 033, 034 (deps) and cite `BE-LOG-006` (security-backend) and `CFG-*` (configuration-cross). None need an edit: `BE-LOG-006` is cited/extended, not changed; `CFG-*` is cited; 024/033 already select `-backend.md` files by suffix/surface; 034 is the sibling precedent, unaffected. Informational; does not block.
