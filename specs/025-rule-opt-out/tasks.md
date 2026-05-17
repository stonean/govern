# 025 — Rule-file opt-out via `.govern.toml` Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Add `[[review.disabled-rule-files]]` to `framework/commands/review.md` §Inputs

- [x] Add a new bullet to the **Config** section of §Inputs describing `[[review.disabled-rule-files]]` as an array-of-tables with required `file` (basename) and `reason` (non-empty; trimmed length ≥ 16 Unicode codepoints) fields.
- [x] Reference §Behavior step 5 for where the key is consulted.
- [x] Done when: the §Inputs block lists both `[review] tech-stack-verified` and `[[review.disabled-rule-files]]`, and reading either pre-existing Config bullet still parses cleanly.

## 2. Extend `framework/commands/review.md` §Behavior step 5 with the disabled-files filter

- [x] Insert a new sub-step between the existing stack-filtering and the `loading rule files: <list>` notice that:
  - Reads `.govern.toml` `[[review.disabled-rule-files]]`.
  - For each entry, applies one of: drop + notice (stack-selected match), no-op notice (non-stack-selected match), unknown warning (basename does not exist), malformed warning (missing field or reason < 16 codepoints), duplicate warning (same `file` listed twice).
  - Collapses internal whitespace in `reason` (including newlines from TOML multi-line strings) to single spaces before emitting the notice.
- [x] Ensure the `loading rule files: <list>` notice fires AFTER the disabled-file notices and excludes any dropped file from its list.
- [x] State explicitly that warnings do NOT taint the exit code.
- [x] Done when: step 5 reads top-down as discovery → suffix classify → stack filter → disabled filter → load notice; the order of stdout lines in a normal run is `disabled-rule-file: …` lines first, then `loading rule files: …` last.

## 3. Update `framework/commands/review.md` §Notes for adopters

- [x] Add one bullet noting the `[[review.disabled-rule-files]]` override exists, with a reason field for the audit trail. Cross-link to §Inputs (or §Behavior step 5) for the schema.
- [x] Done when: the bullet list under §Notes for adopters covers the opt-out alongside the existing pinning / auto-discovery / unrecognized-suffix notes.

## 4. Update `framework/bootstrap/govern.md` example TOML block

- [x] In the example TOML block (currently lines 246–262, showing `[pinned]` and `[workflows]`), add a commented-out `[[review.disabled-rule-files]]` example block. Show the `file` and `reason` fields with realistic placeholder content.
- [x] Done when: an adopter running through bootstrap sees the three TOML sections side-by-side and the new block is unambiguously commented out (will not actually disable anything if uncommented without editing).

## 5. Update `framework/constitution.md` §rules

- [x] After the filename-suffix subsection (currently lines 285–295), append a brief paragraph naming the file-level opt-out, summarizing that adopters can list a rule file in `.govern.toml` `[[review.disabled-rule-files]]` with a mandatory reason, and pointing at `framework/commands/review.md` for the schema and behavior.
- [x] Keep the addition to ≤ 3 sentences — the constitution describes contracts, not implementation.
- [x] Done when: the §rules anchor reads naturally with the suffix rule, the opt-out paragraph, and the existing lifecycle subsection in sequence.

## 6. Update `framework/commands/status.md`

- [x] In step 6 (below-the-table callouts), add a fourth conditional callout: when `.govern.toml` `[[review.disabled-rule-files]]` is non-empty, emit a single line of the form `disabled rule files: <N> (.govern.toml) — <comma-separated basenames>`.
- [x] Done when: status's instructions enumerate four callouts (blocked specs, recovery-state specs, tags-in-use, disabled rule files); no verbose listing of reasons is added.

## 7. Verify all 9 acceptance criteria

- [x] Walk each AC in `spec.md` against the changes from tasks 1–6. Mark each `[ ]` as `[x]` only after the corresponding behavior is documented in the edited file(s).
- [x] Specifically verify AC7 (analyze does NOT error on the new key) and AC9 (documentation lives in this spec + `framework/commands/review.md`) are satisfied by the planning decisions (analyze.md not edited; canonical docs in review.md).
- [x] Done when: all 9 ACs are checked off with a clear pointer (file + section) for each.

## 8. Lint

- [x] Run `npx markdownlint-cli2` against every file modified in tasks 1–6 plus `specs/025-rule-opt-out/spec.md` and this `tasks.md`.
- [x] Done when: zero violations.

## 9. Run `/gov:review`

- [ ] After the spec advances to `in-progress` and tasks 1–7 are complete, run `/gov:review` to populate `spec.md`'s `review` frontmatter block and produce `review.md`.
- [ ] Done when: `review.md` exists, `must-violations: 0` (or all surfaced findings are resolved/waived), and `review.blocking: false`.
