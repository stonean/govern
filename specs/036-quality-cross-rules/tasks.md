# 036 — Cross-cutting code-quality rules Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Author `framework/rules/quality-cross.md`

- [x] Write the file header: title (`# Code Quality Rules`), an intro stating the discipline is cross-cutting (applies to every stack), the RFC 2119 note, the `QUAL-{CATEGORY}-{NNN}` ID-format / category-declaration line (category `STUB`, with the `See specs/036-quality-cross-rules/data-model.md` + `specs/008-security-rules/data-model.md` pointers), and the pin note adapted for a cross file (always applies; pin in `.govern.toml` `[pinned]` if customized).
- [x] Write the `## QUAL-STUB — Silent stubs` section with `### QUAL-STUB-001`: Statement (MUST), Rationale (silent-stub hazard + the anvil rate-limiter incident), and Verification (review-time three-part discriminator — reachable + contract-implies-work + no-loud-signal — plus the exemption list).
- [x] Cross-reference rather than restate: cite `api-backend.md` `BE-SCHEMA-002` for the build-time schema fail-loud case.
- Done when: the file exists with one well-formed `QUAL-STUB-001` rule (MUST), the `-cross.md` schema, and the `BE-SCHEMA-002` citation.

## 2. Register the `QUAL` surface

- [x] Write `specs/036-quality-cross-rules/data-model.md` registering the `QUAL` surface and `STUB` category, referencing 008's schema (already drafted at plan time — confirm it matches the shipped rule file).
- [x] In `scripts/lint-rule-ids.sh`, extend the allowlist regex `^(BE|FE|CFG)-…` to include `QUAL`, update the error-message string to `{BE|FE|CFG|QUAL}`, and add `specs/036-quality-cross-rules/data-model.md` to the "Source of truth" comment block.
- Done when: `scripts/lint-rule-ids.sh` accepts `QUAL-STUB-001` and still rejects malformed IDs; the data-model and the rule file agree.

## 3. Register the file in the `/govern` manifest

- [x] Add `framework/rules/quality-cross.md → specs/rules/quality-cross.md` to the `### govern-owned shared files` table in `framework/bootstrap/govern.md`, slotted between `performance-frontend.md` and `security-backend.md` (strategy: update).
- [x] Make the §Shared Files "Rule-file surface filter" note count-free (dropped the hard-coded number, which had silently drifted — it read "six" while seven rule files were already listed — and which nothing machine-checks; removing it eliminates the drift class rather than just correcting the value).
- Done when: the manifest row is present, the note no longer hard-codes a rule-file count, and the `-cross.md` suffix makes 024's loader select it for every stack and 033's filter keep it unconditionally.

## 4. Validate

- [x] `scripts/lint-rule-ids.sh` passes (`QUAL-STUB-001` well-formed and accepted; categories disjoint from `BE`/`FE`/`CFG`; no duplicates).
- [x] `scripts/lint-rule-filenames.sh` passes (the `-cross.md` suffix).
- [x] `npx markdownlint-cli2`, the other `scripts/lint-*.sh`, and `scripts/audit/*` pass (frontmatter, tool-coverage, procedure-parseability, manifest-parity, ssot-invariants, cross-doc-consistency — all green).
- [x] `STUB` confirmed disjoint from existing categories; `QUAL` confirmed disjoint from `BE`/`FE`/`CFG` surfaces.
- Done when: all lints/audits green.

## 5. Review and complete

- [x] Run `/gov:review` over the change set; resolve any MUST findings.
- Done when: `/gov:review` reports no blocking violations and the spec can advance to `done`.
