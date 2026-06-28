# 034 — Backend performance rules Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Author `framework/rules/performance-backend.md`

- [x] Write the file header: title, RFC 2119 note, `BE-{CATEGORY}-{NNN}` ID format, the five category abbreviations (`QUERY`/`CACHE`/`POOL`/`PAYLOAD`/`ASYNC`), the `See specs/008-security-rules/data-model.md for the full schema` pointer, and the pin note.
- [x] Write the 13 rules across the five `## BE-{CATEGORY}` sections per the plan's rule list, each with Statement (RFC 2119), Rationale, Verification (design-time commitment), and Source.
- [x] Cross-reference rather than restate: `BE-PAGE` (pagination), `BE-AUTHZ-002`/`BE-AUTHZ-005` (cache-key tenant isolation), `BE-STATUS-001` (202 for async), `CFG-CONST-003` (named tunable constants).
- Done when: the file exists with 13 well-formed rules, 8 MUST / 5 SHOULD, each MUST a DoS/exhaustion case.

## 2. Register the file in the `/govern` manifest

- [x] Add `framework/rules/performance-backend.md → specs/rules/performance-backend.md` to the `### govern-owned shared files` table in `framework/bootstrap/govern.md` (alphabetical position, strategy: update).
- Done when: the manifest row is present; `/govern` would install the file, and 033's `-backend` surface filter selects it.

## 3. Validate

- [x] `scripts/lint-rule-ids.sh` passes (all IDs well-formed, categories disjoint, no duplicates).
- [x] `scripts/lint-rule-filenames.sh` passes (the `-backend.md` suffix).
- [x] `npx markdownlint-cli2` and `scripts/audit/*` pass (also lint-frontmatter, lint-tool-coverage, lint-procedure-parseability — all green).
- [x] Category abbreviations `QUERY`/`CACHE`/`POOL`/`PAYLOAD`/`ASYNC` confirmed disjoint from `security-backend.md` and `api-backend.md`.
- Done when: all lints/audits green.

## 4. Review and complete

- [x] Run `/gov:review` over the change set; resolve any MUST findings. (0 MUST / 0 SHOULD / 0 low-confidence — clean; see `review.md`.)
- Done when: `/gov:review` reports no blocking violations and the spec can advance to `done`.
