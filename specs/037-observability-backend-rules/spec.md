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

# 037 — Backend observability rules

Introduce `framework/rules/observability-backend.md`, an enforceable rule set covering server-side observability concerns beyond logging and audit. The file follows the canonical rule schema and is installed and enforced under the **backend** surface. This is a rule-introducing feature spec — the same path [008-security-rules](../008-security-rules/spec.md) used — and one of the three Task-9 backend sets that follow [034-performance-backend-rules](../034-performance-backend-rules/spec.md).

## Motivation

The constitution names **observability** a governance-recognized cross-cutting category (§rules), and the principles list calls the system to be **Supportable** and **Observable**. The backend surface today covers only the logging/audit slice via `security-backend.md` §BE-LOG; the rest of observability — metrics, distributed tracing, health/readiness, SLOs, alerting — has nowhere to be promoted to the rules tier. Each is re-litigated per feature or discovered missing only during an incident, when the absence of a metric, span, or readiness probe is what turns a degradation into an outage.

This spec closes that gap: a backend observability rule set citable by ID (`BE-{CATEGORY}-{NNN}`) that `/gov:analyze` checks against specs and plans (design-time observability commitments) and `/gov:review` references where applicable.

## Rule set scope

`observability-backend.md` uses the **backend** surface (`-backend.md` suffix, `BE-` ID prefix), with NEW categories disjoint from the existing `BE-` namespaces in `security-backend.md` (`AUTHN`/`AUTHZ`/`INPUT`/`DATA`/`API`/`LOG`/`DEPS`/`ERR`), `api-backend.md` (`SCHEMA`/`APIVER`/`ERRENV`/`STATUS`/`PAGE`/`IDEMP`/`COMPAT`), and `performance-backend.md` (`QUERY`/`CACHE`/`POOL`/`PAYLOAD`/`ASYNC`). Candidate category set (to be resolved at clarify):

| Category | Abbrev | Concern |
| --- | --- | --- |
| Metrics | `METRIC` | RED (rate/errors/duration) for request handling; USE (utilization/saturation/errors) for resources; bounded-cardinality labels |
| Distributed tracing | `TRACE` | spans around significant work; trace-context propagation across service boundaries (extends `BE-LOG-006`) |
| Health endpoints | `HEALTH` | distinct liveness / readiness / startup probes; readiness reflects dependency reachability |
| SLOs / error budgets | `SLO` | defined SLIs and SLO targets; an error-budget policy |
| Alerting | `ALERT` | alert on symptoms (SLO burn), not causes; every alert actionable and linked to a runbook |

### Severity posture

Observability rules default to **SHOULD** (the right metrics, spans, and targets are context-dependent). A rule is **MUST** only when its absence blinds operators in a way that prevents detection or diagnosis of an outage *regardless of scale* — e.g. no readiness probe (silent bad deploys), broken trace-context propagation (undebuggable distributed failures).

### Boundaries (cross-reference, do not duplicate)

- **Logging and audit trail** are already `security-backend.md` §BE-LOG (`BE-LOG-005`/`BE-LOG-006`) — this set cites them and covers the rest of observability; `TRACE` extends `BE-LOG-006` rather than restating it.
- **Operator-tunable values** (scrape intervals, probe timeouts, alert thresholds) are `configuration-cross.md` `CFG-CONST-*` / `CFG-ENV-*` — these rules require the value to exist; the config rules govern how it is named and validated.

## Acceptance Criteria

- [ ] `framework/rules/observability-backend.md` exists, ends in the `-backend.md` suffix, and follows the canonical rule schema (`### {ID}` headings; Statement / Rationale / Verification; RFC 2119 language) per [008-security-rules](../008-security-rules/spec.md)'s data-model.
- [ ] Every rule ID uses the `BE-{CATEGORY}-{NNN}` format with an observability category disjoint from the `security-backend.md`, `api-backend.md`, and `performance-backend.md` category sets; `scripts/lint-rule-ids.sh` passes.
- [ ] The file header declares the observability category abbreviations per the per-file category-declaration policy ([016-cross-cutting-rules](../016-cross-cutting-rules/spec.md)).
- [ ] The rule set covers, at minimum, metrics, distributed tracing, and health endpoints — each with a Verification clause expressed as a **design-time commitment** the spec/plan must make (not a code-pattern grep), consistent with how `/gov:analyze` audits artifacts.
- [ ] Each MUST rule is one whose absence prevents detection or diagnosis of an outage regardless of scale; contextual observability trade-offs are SHOULD. The split is evident from the Statements.
- [ ] Rules whose surface overlaps an existing rule cite it rather than restating it (`BE-LOG-006` for tracing/correlation, `CFG-*` for tunable config).
- [ ] The file is added to the `/govern` **Shared Files** manifest in `framework/bootstrap/govern.md` and is selected under the `backend` surface by `/gov:review`, composing with [033-rule-surface-setting](../033-rule-surface-setting/spec.md) and [024-rule-loader](../024-rule-loader/spec.md).

## Open Questions

- **Exact category set and abbreviations.** Five candidates above — confirm the set, or trim (e.g. defer `SLO`/`ALERT` if they read as operational policy rather than per-feature design commitments).
- **Do SLOs and alerting belong in a rules file?** They are more operational than code-shaped; decide whether they are in scope here or out of scope for the rules tier.
- **MUST vs. SHOULD default.** Confirm default SHOULD with MUST reserved for detection/diagnosis-blocking absences.
- **Overlap with `BE-LOG-006`.** Confirm `TRACE` extends and cites the existing logging/tracing rule rather than duplicating it.
