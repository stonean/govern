# 024 — Stack-aware rule-file loader for `/gov:review` Plan

Implements [024 — Stack-aware rule-file loader for `/gov:review`](spec.md).

## Overview

`/gov:review` (and `/gov:analyze`) stop carrying a hardcoded list of rule filenames. They walk `framework/rules/*.md`, read each file's basename suffix, and load it on that signal alone:

- `*-backend.md` → backend surface
- `*-frontend.md` → frontend surface
- `*-cross.md` → cross-cutting (applies to all stacks)
- anything else → load for all stacks + emit a stdout warning

`/gov:review` filters the discovered set by the tech stack the existing step-4 alignment check already determines. `/gov:analyze` loads the full discovered set — citation verification spans surfaces. A CI lint (`scripts/lint-rule-filenames.sh`) keeps govern's own `framework/rules/` honest; adopter-local rule files are governed by the runtime over-apply-and-warn safety net, not by lint. `framework/rules/configuration.md` is renamed to `framework/rules/configuration-cross.md` so the closed-suffix policy needs no allowlist; rule IDs (`CFG-CONST-*`, `CFG-ENV-*`) are content-anchored and unchanged.

## Technical Decisions

### Discovery algorithm (shared between `/gov:review` and `/gov:analyze`)

Both commands describe the same prose procedure in their own §Behavior sections — no shared command file or library. Rationale: per §text-first-artifacts, commands are markdown procedures; prose duplication of a five-line algorithm is cheaper than introducing a third shared command file and changing the parser's understanding of command composition. The two commands are already independently parseable today; this preserves that.

The procedure:

1. List `framework/rules/*.md` (govern's own repo) or `specs/rules/*.md` (adopter projects).
2. For each file, take the basename and inspect its suffix.
3. Classify into one of `{backend, frontend, cross, unrecognized}`.
4. `/gov:review`: keep files whose surface matches the detected stack (the alignment-check output from step 4), keep all `cross`, keep all `unrecognized` (with the warning).
5. `/gov:analyze`: keep every file. Citation verification spans surfaces, so a backend-only project legitimately cites `FE-XSS-001` in a scenario that covers HTML output, and that citation needs to be verifiable.

### Stdout notices

`/gov:review` emits at most three kinds of stdout line during rule-file discovery:

1. **Per unrecognized-suffix file (zero or more):**

   ```text
   rule file <name> has unrecognized suffix — loading for all stacks; rename to -backend.md, -frontend.md, or -cross.md
   ```

2. **One summary line after discovery is complete:**

   ```text
   loading rule files: <comma-separated basenames>
   ```

3. The existing tech-stack alignment messages from step 4 are unchanged.

`/gov:analyze` emits the same `loading rule files:` notice; the unrecognized-suffix warning is identical (the discovery procedure is shared). Adopters reading either command's stdout can confirm which files were considered.

### `configuration.md` → `configuration-cross.md` rename

- `git mv framework/rules/configuration.md framework/rules/configuration-cross.md`. Rule IDs are content-anchored; only the path moves.
- Bootstrap map (`framework/bootstrap/govern.md`) updates both the source path (`framework/rules/configuration-cross.md`) and the destination path. Without renaming the destination, adopter-side `/gov:review` would emit the unrecognized-suffix warning against its own bundled file on every run. (The destination path was subsequently moved to `specs/rules/configuration-cross.md` by the rule-file relocation sweep; see the current bootstrap manifest.)
- A one-pass migration sits in `framework/bootstrap/govern.md` alongside the existing `spec-and-plan.md` cleanup (precedent: spec 023). On each `/govern` invocation: if `specs/configuration.md` exists in the adopting project, offer to rename it and emit a one-line notice. Adopters who skip running `/govern` after upgrade hit the runtime warning at `/gov:review` time until they rename manually — the warning is the discoverability surface for that case. (The migration has since been folded into the broader rule-file relocation migration that moves all closed-suffix rule files to `specs/rules/`.)
- Live references in `framework/` are swept (see Affected Files). Done-spec bodies under `specs/NNN-*/` are frozen archaeology per §drift-prevention and stay as written.
- Cross-reference recorded in `specs/README.md` §Past Renames so historical references in done-spec bodies remain discoverable without rewriting them.

### CI lint (`scripts/lint-rule-filenames.sh`)

Closed-suffix policy — no allowlist. A bash script that iterates `framework/rules/*.md` and exits non-zero if any basename does not end in `-backend.md`, `-frontend.md`, or `-cross.md`. Error message names the three valid suffixes. The lint runs in govern's repo only; adopter repos rely on the runtime over-apply-and-warn rule.

Wiring: a new step in `.github/workflows/markdown-only-pipeline.yml`, alongside the existing `lint-tool-coverage.sh`, `lint-frontmatter.sh`, `lint-procedure-parseability.sh` invocations. This workflow is the right home — rule-filename hygiene is a markdown-pipeline invariant, not a generator-output invariant.

### Constitution edit

`framework/constitution.md` §rules gains one new subsection — `#### Filename suffix` — before `#### Lifecycle`. Body: states the closed suffix set, names the surface each suffix selects, references `scripts/lint-rule-filenames.sh` as the govern-side enforcement and the runtime warning as the adopter-side safety net. Two existing inline references (`framework/rules/configuration.md` at lines 173 and 179) update to `framework/rules/configuration-cross.md`. No anchor changes, no cascading reference updates.

### `AGENTS.md` fallback narrowed (not removed)

§Notes for adopters in `framework/commands/review.md` rewrites:

- Files inside `framework/rules/` are auto-discovered by directory walk — no `AGENTS.md` reference required.
- The `AGENTS.md` fallback survives strictly for adopter-local rule files placed outside `framework/rules/` (e.g., `docs/rules/internal-api.md`). The framework cannot directory-walk arbitrary adopter paths; an explicit reference is the discovery signal.

This drops the "depend on author diligence" failure mode for files inside `framework/rules/` while preserving the legitimate use case for files outside it.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/rules/configuration.md` → `framework/rules/configuration-cross.md` | Rename | Conform to closed-suffix policy; rule IDs unchanged |
| `framework/constitution.md` | Edit | Add §rules `#### Filename suffix` subsection; update two `configuration.md` refs |
| `framework/commands/review.md` | Edit | Rewrite §Behavior step 5 + §Load rules for suffix discovery; add `loading rule files:` notice; rewrite §Notes for adopters |
| `framework/commands/analyze.md` | Edit | Apply shared suffix discovery; remove the closed list at lines 137–141; clarify "no stack filtering" |
| `framework/commands/implement.md` | Edit | Update §Scope Boundaries reference at line 49 |
| `framework/commands/groom.md` | Edit | Update `specs/configuration.md` reference at line 43 |
| `framework/bootstrap/govern.md` | Edit | Update bootstrap map source/destination; add one-pass migration for `specs/configuration.md` → `specs/rules/configuration-cross.md` |
| `specs/README.md` | Edit | Record rename under §Past Renames |
| `scripts/lint-rule-filenames.sh` | Create | Closed-suffix lint, exits non-zero on any `framework/rules/*.md` without a valid suffix |
| `.github/workflows/markdown-only-pipeline.yml` | Edit | Wire `scripts/lint-rule-filenames.sh` into the lint phase as a new step |
| `.claude/commands/gov/review.md`, `.claude/commands/gov/analyze.md`, `.claude/commands/gov/implement.md`, `.claude/commands/gov/groom.md` | Derived | Regenerated by `scripts/gen-claude-commands.sh` |

## Trade-offs

### Considered and rejected

- **`surface:` frontmatter field on every rule file.** Rejected — per Non-goals, filename suffix is sufficient and visible at directory-listing time; frontmatter duplicates the signal and creates a "what if they disagree?" problem.
- **Hardcoded allowlist for valid cross-cutting names.** Rejected — per Resolved Questions, the allowlist rots silently and reintroduces the author-discipline failure mode AGENTS.md forbids. Closed-suffix policy needs no allowlist.
- **`/gov:analyze` applies the same stack filter as `/gov:review`.** Rejected — per Resolved Questions, citation verification spans surfaces. A backend project legitimately cites `FE-XSS-001` in a scenario; that citation needs to be verifiable regardless of detected stack.
- **Leave the bootstrap destination at `specs/configuration.md`.** Rejected — the adopter-side discovery algorithm needs the same closed-suffix signal as govern's. Without renaming the destination, adopter-side `/gov:review` would emit the unrecognized-suffix warning against govern's own bundled file on every run.
- **Skip the one-pass adopter migration in `framework/bootstrap/govern.md`.** Rejected — without it, adopters who upgrade get both `specs/configuration.md` (now orphaned) and `specs/rules/configuration-cross.md` (newly bootstrapped) and the older file silently sticks around. The migration mirrors spec 023's `spec-and-plan.md` precedent.
- **A shared discovery library file referenced from both commands.** Rejected — adds a third file to the parser's understanding for a five-line algorithm. Prose duplication is cheaper and keeps each command independently parseable.

### Known limitations

- **Lint runs in govern's repo only.** Adopter-local rule files with non-closed suffixes load with a runtime warning rather than a CI failure. This is the intended safety net (Acceptance Criterion: load + warn, never silent skip) — adopters whose stack is genuinely outside backend/frontend have three legitimate paths documented in the spec's Resolved Questions (rename to `-cross.md`, keep the warning, or place the file outside `framework/rules/`).
- **Migration depends on `/govern` being re-run.** Adopters who upgrade `govern` without re-running `/govern` will hit the unrecognized-suffix warning at `/gov:review` time until they rename `specs/configuration.md` manually. The warning is the discoverability surface for that case; this matches the spec 023 precedent.
- **No mobile suffix yet.** Mobile-specific rules are deferred per Non-goals. The closed-suffix set extends with `-mobile.md` (plus a stack-detection update) in a future spec when mobile rule files are first proposed.
