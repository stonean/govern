---
status: done
dependencies: [008-security-rules, 016-cross-cutting-rules, 024-rule-loader, 033-rule-surface-setting, 034-performance-backend-rules]
review:
  last-run: 2026-06-29T02:16:18Z
  reviewed-against: e375c1cffc3e2127cd3520fb61fd108d211d1c24
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

`observability-backend.md` uses the **backend** surface (`-backend.md` suffix, `BE-` ID prefix), with NEW categories disjoint from the existing `BE-` namespaces in `security-backend.md` (`AUTHN`/`AUTHZ`/`INPUT`/`DATA`/`API`/`LOG`/`DEPS`/`ERR`), `api-backend.md` (`SCHEMA`/`APIVER`/`ERRENV`/`STATUS`/`PAGE`/`IDEMP`/`COMPAT`), and `performance-backend.md` (`QUERY`/`CACHE`/`POOL`/`PAYLOAD`/`ASYNC`). Category set (resolved at clarify — three categories ship; `SLO` and `ALERT` are deferred as operational policy, see Resolved Questions):

| Category | Abbrev | Concern |
| --- | --- | --- |
| Metrics | `METRIC` | RED (rate/errors/duration) for request handling; USE (utilization/saturation/errors) for resources; bounded-cardinality labels |
| Distributed tracing | `TRACE` | spans around significant work; trace-context propagation across service boundaries (extends `BE-LOG-006`) |
| Health endpoints | `HEALTH` | distinct liveness / readiness / startup probes; readiness reflects dependency reachability |

### Severity posture

Observability rules default to **SHOULD** (the right metrics, spans, and targets are context-dependent). A rule is **MUST** only when its absence blinds operators in a way that prevents detection or diagnosis of an outage *regardless of scale* — e.g. no readiness probe (silent bad deploys), broken trace-context propagation (undebuggable distributed failures).

### Boundaries (cross-reference, do not duplicate)

- **Logging and audit trail** are already `security-backend.md` §BE-LOG (`BE-LOG-005`/`BE-LOG-006`) — this set cites them and covers the rest of observability; `TRACE` extends `BE-LOG-006` rather than restating it.
- **Operator-tunable values** (scrape intervals, probe timeouts, alert thresholds) are `configuration-cross.md` `CFG-CONST-*` / `CFG-ENV-*` — these rules require the value to exist; the config rules govern how it is named and validated.

## Acceptance Criteria

- [x] `framework/rules/observability-backend.md` exists, ends in the `-backend.md` suffix, and follows the canonical rule schema (`### {ID}` headings; Statement / Rationale / Verification; RFC 2119 language) per [008-security-rules](../008-security-rules/spec.md)'s data-model.
- [x] Every rule ID uses the `BE-{CATEGORY}-{NNN}` format with an observability category disjoint from the `security-backend.md`, `api-backend.md`, and `performance-backend.md` category sets; `scripts/lint-rule-ids.sh` passes.
- [x] The file header declares the observability category abbreviations per the per-file category-declaration policy ([016-cross-cutting-rules](../016-cross-cutting-rules/spec.md)).
- [x] The rule set covers, at minimum, metrics, distributed tracing, and health endpoints — each with a Verification clause expressed as a **design-time commitment** the spec/plan must make (not a code-pattern grep), consistent with how `/gov:analyze` audits artifacts.
- [x] Each MUST rule is one whose absence prevents detection or diagnosis of an outage regardless of scale; contextual observability trade-offs are SHOULD. The split is evident from the Statements.
- [x] Rules whose surface overlaps an existing rule cite it rather than restating it (`BE-LOG-006` for tracing/correlation, `CFG-*` for tunable config).
- [x] The file is added to the `/govern` **Shared Files** manifest in `framework/bootstrap/govern.md` and is selected under the `backend` surface by `/gov:review`, composing with [033-rule-surface-setting](../033-rule-surface-setting/spec.md) and [024-rule-loader](../024-rule-loader/spec.md).

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Exact category set and abbreviations.** Resolved: **ship three categories — `METRIC`, `TRACE`, `HEALTH`** (the minimum AC #4 requires), each disjoint from the existing `BE-` category sets in `security-backend.md` / `api-backend.md` / `performance-backend.md`. `SLO` and `ALERT` are deferred (see next question). The `BE-{CATEGORY}-{NNN}` grammar leaves room to add them later.
- **Do SLOs and alerting belong in a rules file?** Resolved: **deferred — out of scope for this rule set.** SLOs and alerting are operational policy, not per-feature design commitments: `/gov:analyze` verifies a rule by checking whether a spec/plan makes a commitment, but a feature spec does not "define an SLO" or "author an alert" — those live at the service/ops layer. They do not fit the design-time-commitment verification model the other backend rules use. They can be promoted later if a concrete per-feature commitment shape emerges.
- **MUST vs. SHOULD default.** Resolved: **default SHOULD; MUST only when the absence prevents detection or diagnosis of an outage regardless of scale.** Mirrors 034's performance posture. The two MUSTs at launch: a readiness probe (its absence ships silent bad deploys) and trace-context propagation (its absence makes distributed failures undebuggable). Metric/span *coverage* choices stay SHOULD.
- **Overlap with `BE-LOG-006`.** Resolved: **`TRACE` extends and cites `BE-LOG-006`, never duplicates it.** `BE-LOG-006` owns correlation-ID / trace-context *in logs*; the new `TRACE` rules cover span creation around significant work and trace-context propagation across service boundaries, citing `BE-LOG-006` for the logging-correlation seam (and `CFG-*` for tunable config such as scrape intervals and probe timeouts).
