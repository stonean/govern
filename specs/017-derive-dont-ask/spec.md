---
status: done
dependencies: []
---

# 017 — Derive, Don't Ask

Apply the **Design Principles** rule added to `AGENTS.md` on 2026-05-06 ("Never design framework features that depend on human diligence or discipline") to every existing framework input that violates it. For each violation, land on a derivable design or an explicit deletion. No item is deferred to inbox — the principle's own escape hatch is exercised in this spec.

The principle, restated: any framework input that requires an author to *remember* to fill it in, set a flag, update a doc alongside code, or otherwise be careful will fail in practice — silently, and exactly in the cases where it mattered most. The remedy is to derive the input from existing artifacts, frontmatter, git history, or code analysis; or, if no derivation is viable, to delete the input.

This spec is a discipline-cleanup pass. It does not introduce new framework capabilities — every change either removes an input or replaces a manual input with a generator/hook/derivation.

## Violation Inventory

The 17 violations identified during the audit, classified by disposition. Acceptance criteria below trace back to this inventory by item number.

### Delete entirely (no viable derivation, not load-bearing)

| # | Violation | Where it lives |
| --- | --- | --- |
| 1 | `title:` PKM frontmatter | `framework/templates/spec/{spec,plan,tasks,data-model,research,scenario,spec-and-plan}.md`; written by `/specify`, `/capture`, `/plan`, `/elaborate`; checked + auto-fixed by `/validate` |
| 2 | `track: lightweight` field | `framework/templates/spec/spec-and-plan.md` only |
| 3 | `tags` frontmatter | Constitution §text-first-artifacts schema; prompted by `/specify`; checked advisory by `/clarify` and `/validate`; "Starter Tag Vocabulary" table in constitution |
| 4 | `[simple]` task tier marker | Constitution §cost-levers; `framework/templates/spec/tasks.md` template; proposed by `/plan`; surfaced by `/implement` |
| 5 | `[promote-to-rule]` inbox prefix | `framework/commands/groom.md` |
| 6 | Plan template "Open Questions Resolved" section | `framework/templates/spec/plan.md` |
| 7 | AGENTS.md "remember to mirror" instruction (constitutions) | `AGENTS.md` Workflow section |
| 8 | AGENTS.md "remember to run gen-claude-commands.sh" instruction | `AGENTS.md` Workflow section |

### Derive (replace the input with a scaffold/generator/hook)

| # | Violation | Derivation path |
| --- | --- | --- |
| 9 | `spec-ref` parent half in scenario frontmatter | Parent feature is the directory the scenario lives in; scenario frontmatter shrinks to the section name only (`section:` field), and `/elaborate` writes it. |
| 10 | README feature table | Generator script reads `specs/*/spec*.md` frontmatter (`status`, `dependencies`, body's first paragraph) and rewrites the table between marker comments. Wired into a pre-commit hook. |
| 11 | `.claude/commands/gov/*.md` generator drift | Pre-commit hook auto-runs `./scripts/gen-claude-commands.sh` whenever `framework/commands/**` or `framework/bootstrap/configure/claude.md` changes, and stages the result. |
| 12 | `help.md` ↔ command `description:` equivalence | `framework/commands/help.md` "Pipeline / Elaborate / Brownfield / Orient / Bootstrap" command tables generated from each command's frontmatter `description:` between marker comments. Same pre-commit hook. |
| 13 | `dependencies` frontmatter | `/clarify` and `/plan` scan the spec body for relative markdown links to sibling spec files (`../NNN-feature/...` or `[NNN-feature](...)`) and propose the union as `dependencies`. The author confirms or removes; the field is no longer hand-authored from scratch. |

### Replace with structural enforcement (Bucket 3 — real designs)

| # | Violation | Design |
| --- | --- | --- |
| 14 | Plan "Affected Files" used as the implement write boundary | `/implement` derives the boundary at runtime from `git diff --name-only` between the spec dir's first commit and `HEAD`, plus uncommitted changes. The plan's Affected Files section becomes a *planning aid* (proposal during `/plan`), not the authoritative boundary. The implement command stops asking the author to backfill files into the section after-the-fact. |
| 15 | Acceptance-criteria + task checkbox flipping | `/validate --fix` for checkboxes is removed. The framework's position becomes: checkboxes are flipped only by `/implement` (per its existing per-criterion verification step) and `/clarify` (for resolved questions). Hand-implementation flows that bypass `/implement` are out-of-framework — the discipline patch is not shipped. |
| 16 | Twin constitutions (root ↔ framework) mirror | Per Q2: collapse to a single canonical file at `framework/constitution.md`. Delete root `constitution.md` entirely. Update root `CLAUDE.md` to `@import framework/constitution.md` and any root README link to point at `framework/constitution.md`. No generator, no section markers, no install-time stripper. The AGENTS.md mirror instruction is deleted. |
| 17 | §cross-spec-impact rule | `/clarify`, `/plan`, and `/implement` each gain a "cross-spec scan" step: enumerate sibling spec slugs cited (by inline link or bare slug) in the target's body that are not in `dependencies`, and surface them. At `/implement` completion, additionally diff `git diff --stat specs/` over the implementation window and prompt the user when files outside the target spec dir were modified, asking whether those changes need to be recorded as new acceptance criteria/scenarios in the affected spec. The constitution rule stays as the human-readable principle; enforcement moves into the commands. |

### Promote to rule file (the framework's own enforceable-rule mechanism)

| # | Violation | Design |
| --- | --- | --- |
| 18 | §constants + §env-vars rules | Move both constitution sections into a new rule file at `framework/rules/configuration.md` with rules carrying RFC 2119 statements + Verification steps that `/validate` runs against feature artifacts. The constitution sections shrink to a one-line pointer ("See `framework/rules/configuration.md`"). Adoption: shipped to adopters via `/govern` like other rule files. |

(Original audit numbered violations 1–17 across templates, commands, constitution, and AGENTS.md. The inventory above splits violation #18 — promoting constants + env-vars to a rule file — out as its own item because it requires creating a new rule file rather than editing existing artifacts.)

## Schema Changes

Frontmatter schema (constitution §text-first-artifacts) after this spec:

| File | Required | Removed |
| --- | --- | --- |
| `spec.md` / `spec-and-plan.md` | `status`, `dependencies` | `tags` (deleted), `title` (deleted), `track` (deleted from `spec-and-plan.md`) |
| `scenarios/{slug}.md` | `section` | `spec-ref` (replaced by `section`), `tags` (deleted), `title` (deleted) |
| `plan.md`, `tasks.md`, `data-model.md`, `research.md` | (none — frontmatter is optional) | `title` (deleted) |

Validate severity changes:

- Hard fail: unchanged for `status`, `dependencies`, frontmatter parse.
- Hard fail (new): `section` on scenario files (replaces `spec-ref`).
- Advisory: PKM `title:` check **removed**; `tags` advisory **removed**.
- Fixable (`--fix`): PKM title fix **removed**; checkbox fixes **removed** (per item #15).
- New blocking checks: project-level `help.md` equivalence becomes structural (the table is generated, so the check verifies the file is up-to-date with the generator output, not that descriptions match by hand).

## Migration

Existing `govern` repo dogfood specs (000–016) and any adopter projects already on the current schema will have stale `title:`, `tags:`, `spec-ref:`, `track:` fields and `[simple]` task markers. Per constitution §done-specs-are-frozen-archaeology, done specs are not rewritten retroactively.

- Existing `done` specs: no migration. Stale fields remain; the open-schema rule (constitution §text-first-artifacts) ignores unknown fields. Validate stops checking them, so they cause no findings.
- New specs created after this lands: no longer have the deleted fields.
- Adopter projects: receive the new templates on next `/govern`. Existing specs in adopter projects are also frozen archaeology.
- Active in-flight specs in this repo (none at the moment beyond this spec): the author can strip the deleted fields opportunistically; it is not enforced.

This spec's own frontmatter contains `title:` and `tags:` because they are still required by the current templates at spec-creation time. They will be stripped from this spec's frontmatter as part of the implementation tasks (the spec edits its own frontmatter as a final task).

## Generators and Hooks

Per Q2 (twin constitutions collapsed) and Q7 (spec-deps derivation), three new generators land alongside the existing `gen-claude-commands.sh`:

1. `scripts/gen-readme-table.sh` — rebuilds the Feature Specs table in the root `README.md` between marker comments from `specs/*/spec*.md` frontmatter.
2. `scripts/gen-help-tables.sh` — rebuilds the command tables in `framework/commands/help.md` from each command file's frontmatter `description:`.
3. `scripts/gen-spec-deps.sh` — scans every `specs/*/spec*.md` body for inline markdown links to sibling specs (excluding code fences) and rewrites the frontmatter `dependencies` list to match.

(The fourth proposed generator, `gen-root-constitution.sh`, is *not* needed — Q2 collapsed the twin constitutions to a single canonical file at `framework/constitution.md`, removing the divergence the generator would have managed.)

### govern repo's pre-commit hook

`.githooks/pre-commit` orchestrates all four generators (`gen-claude-commands`, `gen-readme-table`, `gen-help-tables`, `gen-spec-deps`). Installed via `git config core.hooksPath .githooks`, run by `scripts/install-hooks.sh`. Runs all generators unconditionally on every commit and stages any changes — trades a fraction of a second per commit for a one-line implementation that can't get the gate logic wrong.

### Adopter projects' pre-commit hook

`framework/bootstrap/hooks/pre-commit` ships with the framework. Runs only adopter-relevant generators — initially `gen-spec-deps.sh`. The slot is extensible for future adopter-relevant generators.

`/govern` manages the adopter hook (see Q7 resolution for the install/update/skip behavior). `scripts/gen-spec-deps.sh` ships to adopters with `create` strategy on first `/govern` run; adopters can edit it without `/govern` clobbering.

### CI safety net

Both repos run all generators in dry-run mode in CI; non-empty diff fails the build. Catches contributors who never installed the hook locally and adopters whose hook was skipped due to existing-hook detection.

## Acceptance Criteria

- [x] AC1: All six template files (spec, spec-and-plan, plan, tasks, data-model, research, scenario) have no `title:` field in frontmatter
- [x] AC2: `framework/templates/spec/spec-and-plan.md` has no `track:` field and no comment about it
- [x] AC3: Scenario template uses `section:` (not `spec-ref:`) and parent feature is no longer encoded in the field
- [x] AC4: No template, command, or constitution section references `tags` as a frontmatter field; the "Starter Tag Vocabulary" table is removed from the constitution
- [x] AC5: No template, command, or constitution section references the `[simple]` task marker; the §cost-levers reference is updated to remove the bullet
- [x] AC6: `framework/commands/groom.md` no longer instructs the agent to prefix items with `[promote-to-rule]`; groom always re-walks every item it didn't migrate on the next pass
- [x] AC7: `framework/templates/spec/plan.md` does not contain an "Open Questions Resolved" section
- [x] AC8: `AGENTS.md` does not contain the "mirror constitutions" or "run gen-claude-commands.sh" instructions
- [x] AC9: `/specify`, `/capture`, `/plan`, and `/elaborate` each write the canonical filename-derived metadata (formerly `title:`) at scaffold time without prompting the author
- [x] AC10: The README's "Feature Specs" table is bounded by marker comments and produced by `scripts/gen-readme-table.sh`; running the script on a fresh checkout produces no diff
- [x] AC11: `framework/commands/help.md` command tables are bounded by marker comments and produced by `scripts/gen-help-tables.sh`; running the script on a fresh checkout produces no diff
- [x] AC12: A pre-commit hook at `.githooks/pre-commit` runs all four generators (claude-commands, readme-table, help-tables, spec-deps) and stages outputs; `scripts/install-hooks.sh` configures `core.hooksPath`. (No `root-constitution` generator — Q2 collapsed the twin constitutions to a single canonical file, removing the need.)
- [x] AC13: `/clarify` and `/plan` scan the target spec body for sibling-spec links and propose missing `dependencies` entries; the author confirms or removes
- [x] AC14: `/implement` derives the write-boundary check from `git diff` against the spec dir's first commit, not from a manually maintained Affected Files list; the plan's Affected Files section is documented as a planning aid only
- [x] AC15: `/validate` no longer offers `--fix` for checkbox state; the Fix Mode section is removed and the checkbox-related entries in the Fixable list are gone
- [x] AC16: Root `constitution.md` is deleted; `framework/constitution.md` is the sole canonical file; root `CLAUDE.md` imports it via `@import framework/constitution.md`; no root-constitution generator exists
- [x] AC17: `/clarify`, `/plan`, and `/implement` each include a cross-spec scan step that surfaces sibling slugs cited in the body but not in `dependencies`; `/implement` completion additionally surfaces files outside the target spec dir touched during the implementation window
- [x] AC18: A new rule file `framework/rules/configuration.md` exists with rules covering shared constants, module-local constants, configurable values backed by env vars, env-var defaults, env-var validation, and unit-suffixed time variables; each rule has a Verification step `/validate` runs; constitution §constants and §env-vars shrink to one-line pointers
- [x] AC19: `/validate` runs cleanly on every existing spec (000–016) plus this spec after migration — no new findings introduced by schema changes; stale fields in done specs are silently ignored per the open-schema rule
- [x] AC20: This spec's own frontmatter has `title:` and `tags:` removed by the final task
- [x] AC21: `framework/bootstrap/hooks/pre-commit` and `framework/bootstrap/hooks/install.sh` ship with the framework; the shipped hook calls only adopter-relevant generators (initially `gen-spec-deps.sh`)
- [x] AC22: `/govern` installs the adopter hook on first run when no existing hook system is detected; updates on subsequent runs; warns and skips with a manual integration snippet when an existing hook system is detected (`.githooks/pre-commit` not from `/govern`, husky, lefthook, pre-commit-py, or `core.hooksPath` pointing elsewhere); respects `.govern.toml` pinning
- [x] AC23: `scripts/gen-spec-deps.sh` ships to adopter projects with `create` strategy on first `/govern` run; the shipped pre-commit hook references it via the project-relative path
- [x] AC24: A CI workflow runs all generators in dry-run mode and fails the build on non-empty diff, in both this repo and (as a shipped example) adopter projects; protects against contributors or adopters whose hook was skipped or never installed

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Q1 (item #15, checkbox flipping):** Remove `--fix` entirely. Checkboxes are flipped only by `/implement` (per its existing per-criterion verification step at `framework/commands/implement.md:99`) and by `/clarify` (which moves resolved questions section-to-section). Hand-implementation that bypasses `/implement` is treated as out-of-framework — the framework does not ship a fix-up tool for it. Combined with the title-field removal in AC1, this leaves `--fix` mode with nothing to do, so the entire flag goes away from `/validate`. Friction with parent-checkbox-not-flipped surfaces only outside the framework path; `/implement` already marks parents and sub-items together (`implement.md:91`).
- **Q2 (item #16, twin constitutions):** Collapse to a single canonical file at `framework/constitution.md`. Delete root `constitution.md`. Update root `CLAUDE.md` to `@import framework/constitution.md` and any root README link to point at `framework/constitution.md`. No generator, no section markers, no install-time stripper. Verified the two files were byte-identical at decision time — the documented "may diverge" pattern was theoretical and the discipline trap was guarding nothing. AGENTS.md already exists as the govern-internal home for project-specific rules; anything that ever needs to be govern-only belongs there, not in the constitution. If govern-internal content ever needs to live in the constitution, a section-marker mechanism is introduced then, not preemptively. Adopter installs are unchanged: `/govern` still remaps `framework/constitution.md` → `constitution.md` at the adopter root. The AGENTS.md "mirror constitutions" instruction is removed by AC8.
- **Q3 (item #14, Affected Files diff window):** Diff against the spec dir's first commit (`git log specs/{feature}/ | tail -1`), filtered to files outside `specs/{feature}/`. Cumulative across multi-session implements — each new out-of-plan file surfaces once on first appearance, then is added to the plan and not re-surfaced. Equivalent to "since planned→in-progress transition" for clean pipelines (pre-implement work touches only the spec dir, which the filter excludes), but simpler to compute. No new state in `session.json`.
- **Q4 (item #17, cross-spec scan precision):** Inline markdown links only — match `[label](../NNN-feature/...)` or paths to `specs/NNN-feature/`. Bare slug mentions in prose are treated as commentary, not cross-references, on two grounds: (1) prose references to other specs are common and would flood the proposal with false positives, and (2) "always write proper links" is a markdown convention authors already follow, not a govern-specific discipline. Rule-ID citations (e.g., `BE-AUTHN-001`) are out of scope here — those go through the existing rule-reference scan in `/validate` (`validate.md:163-166`).
- **Q5 (item #18, configuration rule file naming):** Single file at `framework/rules/configuration.md`. Both rule sets cover the same domain (operator-tunable values) and apply equally to backend and frontend code, so the security-files split-by-surface precedent does not apply. Splitting by topic would create artificial boundaries — env-var defaults *are* constants and `.env.example` is the manifest of env vars; they belong together.
- **Q6 (item #18, configuration rule ID prefix):** `CFG-` with two categories: `CONST` (shared / module-local constants) and `ENV` (env-var rules). Format: `CFG-{CONST|ENV}-{NNN}`. Examples: `CFG-CONST-001` (shared constants in centralized location), `CFG-ENV-001` (every env var has a default constant), `CFG-ENV-002` (`.env.example` contains every introduced var), `CFG-ENV-003` (time variables include unit suffix in name and constant). Rule files declare their own ID format per `validate.md:145`; this spec sets the configuration file's pattern.
- **Q7 (item #13, dependencies derivation + sync mechanism):** Body is authoritative; frontmatter `dependencies` is fully derived. The derivation runs in two places: (a) a new generator `scripts/gen-spec-deps.sh` invoked by a pre-commit hook, which scans every `specs/*/spec*.md` body for inline markdown links to sibling specs (excluding code fences), recomputes the union, and rewrites the frontmatter list; (b) `/clarify`, `/plan`, `/implement`, `/elaborate`, `/ask`, and `/target` recompute on entry as an idempotent safety net for uncommitted body edits. The author maintains links in the body — there is no place to author the frontmatter list. To remove a dep, remove the inline link (or move it inside a code fence). To mention a spec without depending on it, use a bare slug per Q4.

  **Hooks are symmetric across govern and adopters; content is asymmetric.** govern's hook runs all four generators (`gen-claude-commands`, `gen-readme-table`, `gen-help-tables`, `gen-spec-deps`). Adopter projects' hook runs only adopter-relevant generators (initially just `gen-spec-deps`; the slot is extensible).

  **`/govern` manages the adopter hook on every install/update run.** Ships `framework/bootstrap/hooks/pre-commit` and `framework/bootstrap/hooks/install.sh`. On run, detects state and acts:

  - No `core.hooksPath` set, no `.githooks/pre-commit` → install both, set `core.hooksPath .githooks`, report installed.
  - `.githooks/pre-commit` exists from a prior `/govern` run → overwrite (`update` strategy, pinnable via `.govern.toml`).
  - Existing hook system detected (`.githooks/pre-commit` not from `/govern`, husky, lefthook, pre-commit-py, or `core.hooksPath` pointing elsewhere) → do not install; report a warning with a manual integration snippet; continue.

  `scripts/gen-spec-deps.sh` ships to adopters with `create` strategy on first `/govern` run so adopters can edit it without `/govern` clobbering. The shipped pre-commit hook calls it via the project-relative path.

  **CI safety net for both surfaces.** The same generators run in dry-run mode in CI; non-empty diff fails the build. Catches contributors who never installed the hook locally and adopters whose hook was skipped due to existing-hook detection.

  **Migration of existing specs.** The 16 existing specs have hand-authored frontmatter deps that may not match their body inline links. On first run after this lands, `gen-spec-deps.sh` may *remove* deps that aren't body-linked. Mitigation: a one-time migration step adds inline links to spec bodies for any frontmatter dep not already linked. This is an implementation task, not a runtime surprise.

  **Recategorization.** Item #13 moves from "Derive (proposal + author confirms)" to the same conceptual bucket as items #10–#12 — generated artifacts maintained by the pre-commit hook.
- **Q8 (constitutional promotion of the discipline principle):** Leave the principle in `AGENTS.md` for now. It is one day old (added 2026-05-06) and has been applied to two decisions (the upgrade-impact deferral and this spec). The framework's maturation path is: principles prove themselves in `AGENTS.md`, then promote to the constitution if they generalize — the §rules tier in spec 016 followed exactly this path after specs 008 and 015 established the pattern. Constitutional changes ship via `/govern` to every adopter and are stickier to roll back; preemptive promotion skips the proving step. Re-evaluate in a follow-up spec once a third or fourth application accumulates.
