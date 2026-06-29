# 039 — Backend reliability rules Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Author `framework/rules/reliability-backend.md`

- [x] Write the file header: title (`# Reliability Rules — Backend`), an intro scoping it to server-side resilience under partial failure, the RFC 2119 note, the `BE-{CATEGORY}-{NNN}` ID-format / category-declaration line (categories `TIMEOUT`/`RETRY`/`BREAKER`/`DRAIN`/`BULK`, with the `See specs/008-security-rules/data-model.md` pointer), the default-SHOULD + design-time-commitment paragraph, and the backend pin/surface note.
- [x] Write the eight rules across the five `## BE-{CATEGORY}` sections per the plan's rule table: `BE-TIMEOUT-001` (MUST) / `BE-TIMEOUT-002` (SHOULD), `BE-RETRY-001` (MUST) / `BE-RETRY-002` (SHOULD), `BE-BREAKER-001` (SHOULD), `BE-DRAIN-001` (MUST) / `BE-DRAIN-002` (SHOULD), `BE-BULK-001` (SHOULD) — each with Statement (one RFC 2119 keyword), Rationale, and a design-time-commitment Verification clause, plus Source where apt.
- [x] Cross-reference rather than restate: `BE-RETRY-001` cites `api-backend.md` `BE-IDEMP`; `BE-DRAIN-002` cites `observability-backend.md` `BE-HEALTH-001`; `BE-BULK-001` cites `performance-backend.md` `BE-ASYNC`/`BE-POOL-*`; `BE-TIMEOUT`/`BE-BREAKER` cite `BE-POOL-002`; tunable values cite `configuration-cross.md` `CFG-*`.
- Done when: the file exists with eight well-formed rules (3 MUST / 5 SHOULD), categories disjoint from the other backend files, each MUST a scale-independent availability/cascading-failure risk, and the four 034-deferred concerns (deadlines, timeouts, retries, breakers) all present.

## 2. Register the file in the `/govern` manifest

- [x] Add `framework/rules/reliability-backend.md → specs/rules/reliability-backend.md` to the `### govern-owned shared files` table in `framework/bootstrap/govern.md`, slotted between `quality-cross.md` and `security-backend.md` (alphabetical; strategy: update).
- Done when: the manifest row is present; the `-backend.md` suffix makes 024's loader select it under the `backend` surface and 033's filter include it. (The §Shared Files note is count-free — no count edit.)

## 3. Validate

- [x] `scripts/lint-rule-ids.sh` passes (`BE-TIMEOUT`/`BE-RETRY`/`BE-BREAKER`/`BE-DRAIN`/`BE-BULK` IDs well-formed; categories disjoint from the other backend files; no duplicates).
- [x] `scripts/lint-rule-filenames.sh` passes (the `-backend.md` suffix).
- [x] `npx markdownlint-cli2`, the other `scripts/lint-*.sh`, and `scripts/audit/*` pass (frontmatter, tool-coverage, procedure-parseability, manifest-parity, ssot-invariants, cross-doc-consistency — all green).
- Done when: all lints/audits green.

## 4. Review and complete

- [x] Run `/gov:review` over the change set; resolve any MUST findings.
- Done when: `/gov:review` reports no blocking violations and the spec can advance to `done`.
