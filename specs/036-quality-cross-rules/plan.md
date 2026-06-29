# 036 — Cross-cutting code-quality rules Plan

Implements [036 — Cross-cutting code-quality rules](spec.md).

## Overview

A rule-introducing, markdown-tier feature — the same path 008 and 034 used. It adds one cross-cutting rule file, `framework/rules/quality-cross.md`, whose inaugural rule `QUAL-STUB-001` forbids silent stubs, and wires it into the `/govern` Shared Files manifest so adopters receive it on the next run.

The one structural difference from 034 (which reused the existing `BE` surface): `QUAL` is a **new rule-ID surface**. The runtime `check-rule-ids` harvester already accepts it (its grammar is the generic `[A-Z]{2,5}-[A-Z][A-Z0-9]+-\d{3,4}`), but `scripts/lint-rule-ids.sh` carries a **closed surface allowlist** (`^(BE|FE|CFG)-…`) that must be extended to `QUAL` for AC #3 to pass. Registering a new surface follows the 008 (`BE`/`FE`) and 017 (`CFG`) precedent: each documented its surface in a `data-model.md`, and `lint-rule-ids.sh` cites those data-models as its source of truth — so 036 adds `data-model.md` for the `QUAL` surface and extends that citation.

## Technical Decisions

### The rule file — `framework/rules/quality-cross.md`

Modeled on `configuration-cross.md` (the closest sibling: `-cross.md` suffix, domain-prefixed IDs) and `performance-backend.md`'s header style:

- **Header.** `# Code Quality Rules` + an intro paragraph stating the discipline is cross-cutting (applies to every stack); the RFC 2119 note; the ID-format / category declaration line (per the 016 per-file category-declaration policy) —
  `Rule IDs follow the format \`QUAL-{CATEGORY}-{NNN}\` and are permanent … Categories: \`STUB\` (silent stubs). See \`specs/036-quality-cross-rules/data-model.md\` for the \`QUAL\` surface and \`specs/008-security-rules/data-model.md\` for the full rule schema.`
- **Pin note** adapted for a cross file: cross-cutting files always apply (never surface-filtered), so the note is the standard "pin in `.govern.toml` `[pinned]` if you customize it," not a "projects without X can exclude it" note.
- **Category section** `## QUAL-STUB — Silent stubs` containing the one rule.
- **`### QUAL-STUB-001`** (MUST — per the clarify resolution):
  - **Statement:** *Partial or unimplemented code paths MUST fail loudly (panic / explicit error / failing test fixture) rather than silently pass through. Stubs that return zero values, no-op middleware that returns `next` unchanged, handlers that return early without an error, and methods that return `nil, nil` are forbidden when the surrounding contract implies the path performs work.*
  - **Rationale:** silent stubs ship indistinguishably from working code; the gap surfaces only under stress (the anvil rate-limiter passthrough that would have silently disabled rate limiting in production).
  - **Verification (review-time):** `/gov:review`'s quality pass flags a path only when **all three** hold — (1) reachable under the current spec, (2) the surrounding contract implies work, (3) it returns success/zero/pass-through with no loud signal. Exemptions (not flagged): an explicit incompleteness marker is compliant (`panic`/`todo!`/`unimplemented!`, a raised `NotImplementedError`, a failing/skipped test fixture); intentional pass-through middleware documented as deliberate; default/interface implementations meant to be empty; not-yet-reachable branches behind a flag or guard. Cite `api-backend.md` `BE-SCHEMA-002` for the build-time schema fail-loud case rather than restating it.

### New surface registration — `scripts/lint-rule-ids.sh`

Extend the closed allowlist regex from `^(BE|FE|CFG)-[A-Z][A-Z0-9]*-[0-9]{3,4}$` to include `QUAL`, update the matching error-message string (`{BE|FE|CFG|QUAL}`), and add `specs/036-quality-cross-rules/data-model.md` to the "Source of truth" comment block alongside the 008/017 citations. No runtime change — `check_rule_ids.rs`'s generic grammar already matches `QUAL`.

### Surface registry — `specs/036-quality-cross-rules/data-model.md`

A small data-model documenting the `QUAL` surface (rule-ID prefix; cross-cutting; lives in `*-cross.md`), the inaugural `STUB` category, and a pointer to `008-security-rules/data-model.md` for the canonical rule schema (Statement / Rationale / Verification; `### {ID}` headings; permanent IDs). It does **not** duplicate the schema — it registers the new surface, mirroring how 017 registered `CFG`.

### Manifest wiring — `framework/bootstrap/govern.md`

Add `| \`framework/rules/quality-cross.md\` | \`specs/rules/quality-cross.md\` |` to the `### govern-owned shared files` table, slotted between `performance-frontend.md` and `security-backend.md` (AC #7). Update the §Shared Files "Rule-file surface filter" note count from **six** to **seven** entries. The `-cross.md` suffix means 024's loader auto-selects it for every stack and 033's surface filter keeps it unconditionally — no change needed in either; the wiring is purely the manifest row + count.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/rules/quality-cross.md` | Create | The `quality-cross` rule set with `QUAL-STUB-001` |
| `specs/036-quality-cross-rules/data-model.md` | Create | Register the `QUAL` surface + `STUB` category; reference 008's schema |
| `scripts/lint-rule-ids.sh` | Modify | Add `QUAL` to the surface allowlist regex + error message + source-of-truth comment |
| `framework/bootstrap/govern.md` | Modify | Add the manifest row (between `performance-frontend` and `security-backend`); bump the rule-file count six → seven |

`.claude/commands/*` are unaffected — no command source changes (the bootstrap `govern.md` is not generated into `.claude/`).

## Trade-offs

- **New `QUAL` surface vs. reusing an existing prefix.** Chose a new surface. The discipline is genuinely cross-cutting; forcing it under `BE`/`FE` would misclassify it, and `CFG` is configuration-specific. AC #3 mandates a disjoint namespace. The cost is one allowlist edit in `lint-rule-ids.sh` + a `data-model.md`, both one-time per surface.
- **`data-model.md` vs. documenting the surface inline.** Chose a `data-model.md`, matching 008/017 (every prior new-surface spec has one) and keeping `lint-rule-ids.sh`'s source-of-truth comment honest. A leaner alternative (spec-body-only registration) would diverge from precedent and leave the lint comment citing only 008/017 for a surface 036 owns.
- **Ship `STUB` alone vs. seed sibling categories.** Chose `STUB` alone (clarify resolution) — only `STUB` has a motivating incident; speculative categories would lack a verifiable Verification clause. The `QUAL-{CATEGORY}-{NNN}` grammar leaves room to promote siblings later.
- **Review-time vs. analyze-time verification.** Chose review-time (clarify resolution) — `QUAL-STUB-001` is a source-code pattern, not a design-time artifact commitment like the 034 performance rules, so `/gov:review`'s quality pass is the right checker.
- **Known limitation.** The Verification check is a heuristic the reviewer applies; the three-part discriminator plus exemption list bound the false-positive surface, but a sufficiently obfuscated stub (e.g., one that computes and discards a value) can still evade it. Waivers and `[[review.disabled-rule-files]]` remain the escape hatches for the rare false positive.

## Cross-spec impact

The spec/plan reference 008, 016, 017, 024, 033 (deps) and `api-backend.md` `BE-SCHEMA-002` (cited, not changed). None need an edit:

- **024 (rule-loader)** auto-selects `-cross.md` by suffix — adding a cross file is exactly what it supports; no contract change.
- **033 (rule-surface-setting)** treats cross files as unconditional — a new cross file composes without falsifying any 033 claim.
- **008 / 017** define the rule schema and the `BE`/`FE` / `CFG` surfaces; 036 adds the `QUAL` surface in its own `data-model.md` and cites theirs. The constitution §rules governs the closed **filename-suffix** convention (unchanged) but does not enumerate rule-ID surfaces as a closed list, so no constitution edit. Informational; does not block.
