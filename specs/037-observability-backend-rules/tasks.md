# 037 — Backend observability rules Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Author `framework/rules/observability-backend.md`

- [x] Write the file header: title (`# Observability Rules — Backend`), an intro scoping it to server-side observability beyond logging, the RFC 2119 note, the `BE-{CATEGORY}-{NNN}` ID-format / category-declaration line (categories `METRIC`/`TRACE`/`HEALTH`, with the `See specs/008-security-rules/data-model.md` pointer), the default-SHOULD + design-time-commitment paragraph, and the backend pin/surface note.
- [x] Write the six rules across the three `## BE-{CATEGORY}` sections per the plan's rule table: `BE-METRIC-001/002/003` (SHOULD), `BE-TRACE-001` (MUST) / `BE-TRACE-002` (SHOULD), `BE-HEALTH-001` (MUST) / `BE-HEALTH-002` (SHOULD) — each with Statement (one RFC 2119 keyword), Rationale, and a design-time-commitment Verification clause, plus Source where apt.
- [x] Cross-reference rather than restate: `BE-TRACE-001` extends/cites `security-backend.md` `BE-LOG-006`; `BE-METRIC-003` cites `performance-backend.md` for the cardinality-exhaustion angle; tunable values cite `configuration-cross.md` `CFG-*`.
- Done when: the file exists with six well-formed rules (2 MUST / 4 SHOULD), categories disjoint from the other backend files, each MUST a detection/diagnosis-blocking absence.

## 2. Register the file in the `/govern` manifest

- [x] Add `framework/rules/observability-backend.md → specs/rules/observability-backend.md` to the `### govern-owned shared files` table in `framework/bootstrap/govern.md`, slotted between `configuration-cross.md` and `performance-backend.md` (strategy: update).
- Done when: the manifest row is present; the `-backend.md` suffix makes 024's loader select it under the `backend` surface and 033's filter include it. (The §Shared Files note is already count-free — no count edit.)

## 3. Validate

- [x] `scripts/lint-rule-ids.sh` passes (`BE-METRIC`/`BE-TRACE`/`BE-HEALTH` IDs well-formed; categories disjoint from `security-backend.md` / `api-backend.md` / `performance-backend.md`; no duplicates).
- [x] `scripts/lint-rule-filenames.sh` passes (the `-backend.md` suffix).
- [x] `npx markdownlint-cli2`, the other `scripts/lint-*.sh`, and `scripts/audit/*` pass (frontmatter, tool-coverage, procedure-parseability, manifest-parity, ssot-invariants, cross-doc-consistency — all green).
- Done when: all lints/audits green.

## 4. Review and complete

- [x] Run `/gov:review` over the change set; resolve any MUST findings.
- Done when: `/gov:review` reports no blocking violations and the spec can advance to `done`.
