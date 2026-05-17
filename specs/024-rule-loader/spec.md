---
status: in-progress
dependencies: [020-code-review, 023-govern-refinement]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 024 — Stack-aware rule-file loader for `/gov:review`

Generalize `/gov:review`'s rule-file selection so the set of `framework/rules/*.md` files loaded for any given run is derived from each file's declared surface and the project's detected tech stack — not from a hardcoded list of filenames in [`framework/commands/review.md`](../../framework/commands/review.md). Removes the per-rule-file maintenance bottleneck and removes the author-discipline failure mode created by the current "reference it from AGENTS.md to opt in" fallback.

## Motivation

Today, [`/gov:review`](../../framework/commands/review.md) selects rule files in two ways:

1. **Hardcoded** — `security-backend.md` is loaded for backend stacks; `security-frontend.md` is loaded for frontend stacks; full-stack loads both. The filenames are baked into `framework/commands/review.md` step 5 of §Behavior.
2. **Opt-in** — any other `framework/rules/*.md` file is loaded only when referenced from `AGENTS.md` (`framework/commands/review.md` §Notes for adopters).

Three new rule files have just landed — [`api-backend.md`](../../framework/rules/api-backend.md), [`accessibility-frontend.md`](../../framework/rules/accessibility-frontend.md), [`performance-frontend.md`](../../framework/rules/performance-frontend.md) — and neither path is right for them. Hardcoding three more filenames invites the same problem the fourth, fifth, and sixth rule file will reintroduce. The "reference it from `AGENTS.md`" path means an adopter ships an HTTP API that exposes no OpenAPI schema because the author didn't think to add `api-backend.md` to `AGENTS.md` — exactly the silent author-discipline failure mode AGENTS.md:58 ("Never design framework features that depend on human diligence") exists to prevent.

The fix: derive rule-file selection from observable signals. Each rule file declares which surface it applies to via its filename suffix (the convention `security-backend.md`, `security-frontend.md` already established). `/gov:review` matches the file's surface against the stack the tech-stack-alignment check already produces (see [`framework/commands/review.md`](../../framework/commands/review.md) §Behavior step 4). No author discipline; no command-source edits when a new shipped rule file is added; the `AGENTS.md` fallback survives only for adopter-local rule files that live outside `framework/rules/`.

## Acceptance Criteria

- [ ] Filename suffix is the surface signal: every `framework/rules/*.md` file MUST end in `-backend.md` (backend stacks), `-frontend.md` (frontend stacks), or `-cross.md` (all stacks; cross-cutting). The convention is documented in `framework/constitution.md` alongside the §rules anchor. `scripts/lint-rule-filenames.sh` enforces it in govern's CI.
- [ ] `/gov:review` rule-file selection is rewritten to iterate `framework/rules/*.md`, read each file's suffix, and load it when the suffix matches the project's detected stack (or the file is cross-cutting). The hardcoded names `security-backend.md` and `security-frontend.md` no longer appear in `framework/commands/review.md` as selection criteria — they are loaded by the same derivation as every other file.
- [ ] The three new rule files (`api-backend.md`, `accessibility-frontend.md`, `performance-frontend.md`) load automatically under the new derivation when their respective stacks are present — no `AGENTS.md` edit required.
- [ ] `framework/rules/configuration.md` is renamed to `framework/rules/configuration-cross.md` to match the closed-suffix policy. Rule IDs (`CFG-CONST-*`, `CFG-ENV-*`) are content-anchored and do not change; only the file path moves. Live references swept per AGENTS.md ("no dead references in live artifacts"); done-spec bodies stay as written and the rename is recorded in `specs/README.md` §Past Renames.
- [ ] The "load anything in `framework/rules/` referenced from `AGENTS.md`" fallback survives but narrows to its real purpose: project-local rule files placed outside `framework/rules/` (e.g., `docs/rules/internal-api.md`) that the framework cannot discover by directory walk. Files inside `framework/rules/` no longer need an `AGENTS.md` reference to be loaded.
- [ ] At runtime, a rule file whose name does not match the closed suffix set loads for every stack and emits a one-line stdout warning (`rule file <name> has unrecognized suffix — loading for all stacks; rename to -backend.md, -frontend.md, or -cross.md`). The over-apply-and-warn behavior is the safety net for adopter-local rule files outside govern's CI — the lint runs only in govern's repository. The default is "load + warn," never "silent skip."
- [ ] `/gov:review` emits a one-line `loading rule files: <list>` notice on stdout at the start of each run so adopters can see what was selected. The notice is the discoverability surface.
- [ ] `framework/commands/review.md` §Notes for adopters is updated to reflect the new selection algorithm; specifically, the line "`/gov:review` automatically loads anything in `framework/rules/` that's referenced from `AGENTS.md`" is rewritten to describe the new derivation and clarify the residual role of the `AGENTS.md` fallback.
- [ ] `framework/commands/analyze.md` uses the same suffix-based rule-file discovery as `/gov:review`. `/gov:analyze` consumes the full discovered set (no stack filtering) because rule-ID citation verification spans surfaces; `/gov:review` filters by detected stack on top of the shared discovery.

## Non-goals

- **A `surface:` frontmatter field on rule files.** Filename suffix is sufficient and visible at directory-listing time. Adding frontmatter would duplicate the signal and create a "what if they disagree?" problem.
- **A new surface taxonomy beyond backend / frontend / cross-cutting.** Mobile-specific rules are not in scope; if mobile rules are added later, this spec's pattern extends with a `-mobile.md` suffix and a corresponding stack-detection update (a separate spec).
- **The opt-out mechanism for excluding a specific rule file.** That is **spec 025 — rule-file opt-out** (sibling spec, listed as a forward reference, not a dependency — 025 depends on this spec, not the other way around) — 024 ships the auto-load; 025 ships the override.
- **Editing done specs that reference the old hardcoded behavior.** Per [§drift-prevention](../../framework/constitution.md#drift-prevention), [spec 020](../020-code-review/spec.md) (the spec that introduced `/gov:review`) is frozen archaeology. The current behavior of `/gov:review` is what `framework/commands/review.md` says today; that file is the live artifact this spec edits.

## Affected files

| File | Change | Strategy |
| --- | --- | --- |
| `framework/commands/review.md` | edit — §Behavior step 5 and §Load rules rewritten to derive selection from filename suffix; §Notes for adopters rewritten to describe the new contract | update |
| `framework/commands/analyze.md` | edit — apply the shared suffix-based discovery; no stack filtering (citation verification spans surfaces) | update |
| `framework/constitution.md` | edit — under §rules, document the filename-suffix convention as the framework-level naming rule | update |
| `framework/rules/security-backend.md` | none — file is already correctly suffixed | n/a |
| `framework/rules/security-frontend.md` | none — file is already correctly suffixed | n/a |
| `framework/rules/configuration.md` | rename to `framework/rules/configuration-cross.md` to match closed-suffix policy; rule IDs unchanged | update |
| `framework/rules/api-backend.md` | none — file is correctly suffixed | n/a |
| `framework/rules/accessibility-frontend.md` | none — file is correctly suffixed | n/a |
| `framework/rules/performance-frontend.md` | none — file is correctly suffixed | n/a |
| `scripts/lint-rule-filenames.sh` | required — fails CI if any `framework/rules/*.md` file does not end in `-backend.md`, `-frontend.md`, or `-cross.md` | new |
| `specs/README.md` | edit — record the `configuration.md` → `configuration-cross.md` rename under §Past Renames | update |
| `.github/workflows/*.yml` | edit — wire `scripts/lint-rule-filenames.sh` into the existing lint job | update |

## Open Questions

*None — all resolved.*

## Resolved Questions

- **CI lint for filename convention.** Add the lint, with a closed-suffix policy that needs no allowlist. Every `framework/rules/*.md` file MUST end in `-backend.md`, `-frontend.md`, or `-cross.md`; `scripts/lint-rule-filenames.sh` enforces it in govern's CI. The alternatives (hardcoded allowlist of "valid cross-cutting" names; a `surface:` frontmatter field on every rule file) both reintroduce the author-discipline failure mode AGENTS.md ("Never design framework features that depend on human diligence") forbids — the allowlist would silently rot, and the frontmatter field is the Non-goals explicitly reject. The "wait for evidence" path the original lean proposed was wrong for an open-source project: the maintainer has no reliable feedback loop to learn that adopters are misnaming files. Closed-suffix pays a one-time cost (rename `configuration.md` → `configuration-cross.md`; rule IDs are content-anchored and don't change) for a discipline-free pattern. Runtime over-applies + warns on adopter-local files with unrecognized suffix; the lint only runs in govern's repository.
- **`/gov:analyze` loader generalization.** In scope. `/gov:analyze` (renamed from `/gov:validate` in [spec 023](../023-govern-refinement/spec.md) §4) consumes the same suffix-based rule-file discovery as `/gov:review`, but does not apply stack filtering — rule-ID citation verification spans surfaces (a backend project may cite `FE-XSS-001` in a scenario covering HTML output, and that citation needs to be verifiable). The shared discovery logic is "iterate `framework/rules/*.md`, accept the three closed suffixes." `/gov:review` filters by detected stack on top of that; `/gov:analyze` does not. The "verify during plan" hedge that was originally on this acceptance criterion is removed — the divergence in filtering is decided here; only the implementation form is plan-phase.
- **Adopter-local rule files with non-closed suffixes.** Dissolved by the closed-suffix resolution. The original concern (an adopter adding `framework/rules/security-mobile.md` and expecting "mobile stack" detection) is no longer a separate behavior: the runtime over-apply + warn rule (Acceptance Criterion: unrecognized suffix loads for all stacks + emits stdout warning) handles it. Adopters with surface-specific rules outside backend/frontend have three legitimate paths: (a) name the file `*-cross.md` and accept universal application; (b) keep the non-closed suffix and live with the warning until a future spec extends the surface taxonomy; (c) place the file outside `framework/rules/` (e.g., `docs/rules/`) and reference it from `AGENTS.md` — the project-local fallback the spec preserves. The existing Non-goal "A new surface taxonomy beyond backend / frontend / cross-cutting" already captures the deferral; no new acceptance criterion is needed.
