---
status: in-progress
dependencies: [004-tech-stack-selection, 005-workflows, 010-agent-autonomy, 018-adopter-owned-pre-commit, 019-config-decisions, 027-bootstrap-migration-registry]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 043 — Workflows sunset

Remove the workflows feature ([005-workflows](../005-workflows/spec.md)) from the framework: the `framework/workflows/` directory (tech-stack workflow templates plus `registry.json`), the `/govern` **Workflow recommendation** flow that scaffolds them as slash commands, and the `[workflows] declined_categories` config surface. A new entry in the [bootstrap migration registry](../027-bootstrap-migration-registry/spec.md) cleans up what earlier `/govern` runs scaffolded into adopter projects.

## Motivation

The workflows feature scaffolds tech-stack-specific lint/test/format/migrate slash commands (`/gov:workflows:eslint`, `/gov:workflows:pytest`, …) into adopter projects. In practice these are never run as slash commands: lint, format, and test invocation is the territory of pre-commit hooks, editor configs, and CI — automation surfaces the adopter already owns ([018-adopter-owned-pre-commit](../018-adopter-owned-pre-commit/spec.md) established that govern does not own the adopter's pre-commit rules; the same reasoning applies to the rest of this tooling class). Wrapping `eslint` or `pytest` in a slash command adds an LLM round-trip to commands that are cheaper, faster, and more reliable invoked directly by deterministic tooling.

Carrying the feature has real cost: a 13-template directory plus registry to maintain, an ~80-line recommendation flow in the bootstrap procedure with per-category prompts and decline persistence, a `[workflows]` config section with its own validation and write-policy rules, per-agent layout branches (the flow is already skipped entirely for `antigravity` and `opencode` layouts), and two historical migrations dedicated to renaming its files. Removing it narrows govern to its actual responsibility — the spec pipeline — and shortens every `/govern` run.

## Removal surface (framework side)

- **`framework/workflows/`** — the entire directory: `registry.json` and all workflow templates.
- **`framework/bootstrap/govern.md`** — the **Workflow recommendation** section; the `framework/workflows/registry.json` → `workflows/registry.json` Shared Files manifest row; the `[workflows]` section of the config schema and its `declined_categories` documentation; the workflow mentions in the procedural-fidelity preamble, the write-policy paragraph, the managed-block preserve list, the enforce-manifest step note, and the per-layout skip instructions. The **Tech Stack table parsing** spec text (canonical keys `backend_language` etc.) lives inside the recommendation flow and goes with it unless clarify surfaces a residual consumer.
- **`framework/constitution.md`** — the Workflow registry row in the canonical-source map.
- **Command sources** — the `[workflows]` mention in `link.md`'s illustrative preserve list; the "lint/format/test workflows" phrasing in `groom.md` (reword to name adopter-owned tooling rather than the removed feature).
- **Templates** — the `framework/workflows/` disambiguation note in `framework/templates/project/agents.md`.
- **Docs** — README (config-section docs, example TOML, repo-layout listing, any narrative claims that `/govern` offers workflows) and AGENTS.md (project-structure bullet, procedural-fidelity mirror, the ships-as-is gotcha).
- **Runtime** — comment-level references only (`[workflows]` writes in the config-path doc comment); no workflow-specific runtime behavior exists.
- **Generated copies** — `.claude/commands/gov/*.md` regenerate from source; the hand-maintained `.claude/commands/gov/init.md` is swept by hand.
- **Migration registry** — the `skills-to-workflows` and `workflow-filename-rename` entries and their `framework/migrations/*.md` procedure files leave the tree; their procedure text is archived under root `CHANGELOG.md` § Archived migrations (see Resolved Questions).
- **Sibling spec bodies** — done specs whose bodies document the removed surface as live behavior ([005-workflows](../005-workflows/spec.md) foremost; [004-tech-stack-selection](../004-tech-stack-selection/spec.md)'s trigger consumption, [019-config-decisions](../019-config-decisions/spec.md)'s `[workflows]` persistence, [010-agent-autonomy](../010-agent-autonomy/spec.md)'s rename delivery) receive the same post-completion sunset annotation treatment resolved for 005: a note marking the workflow-feature material as historical, applied as part of this spec's sweep (mechanical-class; no back-edge), bodies otherwise left as the record of what shipped.

Prose-claim sweep discipline applies (a behavior change, not just a rename): claims that `/govern` "recommends" or "scaffolds" workflows must be found by meaning, not token grep. Out of scope: `.github/workflows/` (GitHub Actions CI paths) and generic uses of the word "workflow" (e.g. the constitution's "principles, workflow, and quality gates") are unrelated and untouched.

## Adopter migration

A new registry entry (id `workflows-sunset`) drives cleanup on the next `/govern` run, per the [migration registry contract](../027-bootstrap-migration-registry/spec.md):

- **Exact-set file deletion** in `{config_dir}/commands/{project}/workflows/`: the 13 current template filenames plus the 9 legacy `{category}-{language}-{tool}.md` names (subsumed from `workflow-filename-rename`), each with a per-file `[pinned] files` check (`pinned (kept): …` summary line). Adopter-authored files never match the set and are never touched. The `workflows/` directory is removed only when empty afterward.
- **Legacy `skills/` removal**: a remaining `{config_dir}/commands/{project}/skills/` directory (subsumed from `skills-to-workflows`) is recursively deleted, pinned-checked, per that migration's own necessarily-stale reasoning.
- **Synced registry removal**: `workflows/registry.json` is deleted (same pinned check); the `workflows/` root directory goes with it when empty.
- **`[workflows]` config section removal**: header, keys, and attached comment lines leave the **active config file** (write policy: spec 042), every other table preserved byte-for-byte, reported in the post-scaffolding summary.
- Idempotent: a no-op when no target artifact is present (registry invariant). No inner prompt — the loop's outer "apply N pending migrations" prompt is the only gate; every removal reports a summary line.
- `introduced_in` matches the gvrn release that ships this spec; `sunset_after` follows the registry's per-entry window convention. `target_paths` also lists `framework/workflows/` so `/audit` Family 10's no-stale-target-paths check covers the framework-side removal. Per the AGENTS.md completion gate, the spec is not done until `gvrn-v{introduced_in}` is tagged.

### Edge cases

- **`last_applied` names an archived id** (`skills-to-workflows` or `workflow-filename-rename`): the registry's existing stale-reference behavior applies — treat as before-the-oldest-active-entry, run every active entry (all idempotent, so the re-run is safe), warn once pointing at `CHANGELOG.md`.
- **The subsumed entries are already dormant**: at 0.22.0 the `sunset_after` filter (bootstrap Pre-run Migrations step 3) has excluded both from the adopter loop since 0.10.0 — archiving them changes no runtime behavior, and `workflows-sunset` *restores* legacy-debris coverage that has been inactive since then.
- **Directory holds only custom files**: zero deletions, directory retained, no summary noise for untouched files.
- **Ordering with `govern-dir-consolidate`**: an adopter behind on both runs 0.22.0's consolidate first (SemVer order), so the `[workflows]` removal lands in `.govern/config.toml`; an unconsolidated adopter is handled by active-file resolution either way.
- **Layouts that never scaffolded** (`antigravity`, `opencode`): every target is absent — silent no-op.
- **`{config_dir}` resolution** follows the same placeholder convention the subsumed entries used; multi-agent behavior is the registry mechanism's, unchanged by this spec.

## Out of scope

- **[004-tech-stack-selection](../004-tech-stack-selection/spec.md) survives.** The AGENTS.md Tech Stack table keeps standalone documentation value and `/gov:review`'s `[review] tech-stack-verified` flow ([019-config-decisions](../019-config-decisions/spec.md)) still consumes it. Only the workflow-registry trigger consumption is removed.
- **GitHub Actions** — nothing under `.github/workflows/` or the CI template (`framework/templates/ci/adopter-generators.yml` references the Actions path, not the feature) changes beyond incidental prose.
- **Replacement tooling** — govern does not ship pre-commit rules or editor configs in workflows' place; that surface is adopter-owned by design.

## Acceptance Criteria

- [ ] `framework/workflows/` does not exist, and no live artifact (framework/, scripts/, runtime/, .github/, docs/, README.md, AGENTS.md, spec bodies) references it or the feature's surfaces (`workflows/registry.json`, `[workflows] declined_categories`, the Workflow recommendation flow), excluding GitHub Actions paths and generic uses of the word.
- [ ] The `/govern` procedure contains no workflow scaffolding: no recommendation flow, no registry manifest row, no `[workflows]` config schema, no workflow prompts in the procedural-fidelity preamble.
- [ ] `framework/migrations.toml` carries a `workflows-sunset` entry (id, `introduced_in`, `sunset_after`, summary, procedure file under `framework/migrations/`) whose target paths cover the scaffolded command directory's 22 known filenames (13 current + 9 legacy), the legacy `skills/` directory, the synced `workflows/registry.json`, and `framework/workflows/` (audit Family 10 coverage).
- [ ] `/govern` against an adopter with scaffolded workflow files removes every known-set file and the synced registry copy, removes the `[workflows]` section from the active config file while preserving all other tables byte-for-byte, reports each removal in the post-scaffolding summary, and advances `[migrations] last_applied`.
- [ ] A file listed in `[pinned] files` survives with a `pinned (kept):` summary line; an adopter-authored file not in the known set survives silently and keeps its directory alive.
- [ ] The migration is a no-op against an adopter with no workflow artifacts (fresh project, or `antigravity`/`opencode` layouts that never scaffolded).
- [ ] `skills-to-workflows` and `workflow-filename-rename` no longer appear in `framework/migrations.toml`; their procedure text appears under root `CHANGELOG.md` § Archived migrations and their `framework/migrations/*.md` files are gone.
- [ ] `specs/005-workflows/spec.md` carries a post-completion sunset note pointing at this spec, with `status: done` unchanged.
- [ ] The constitution's canonical-source map has no Workflow registry row.
- [ ] `scripts/audit/run-all.sh` passes clean after the removal.
- [ ] The `gvrn-v{introduced_in}` release tag is published (migration completion gate).

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Customized scaffolded workflow files: exact set + pinned.** The migration follows `workflow-filename-rename`'s shape, not `skills-to-workflows`' recursive delete: delete by exact filename match against the 13 registry template names in `{config_dir}/commands/{project}/workflows/`, honoring `.govern/config.toml` `[pinned] files` per file (`pinned (kept): …` summary line); remove the `workflows/` directory only when it is empty afterward, so adopter-authored files (e.g. `pytest-fast.md`) survive and keep the directory alive; remove the synced `workflows/registry.json` with the same pinned check. No inner prompt — the registry's outer "apply N pending migrations" prompt is the only gate (procedural-fidelity rule: pinning is the documented opt-out), and each removal reports a post-scaffolding summary line.
- **`[workflows]` config section: the migration deletes it.** The byte-for-byte preservation discipline governs writers touching a *sibling* section; a registered migration is the framework's designated mechanism for schema moves (042's migration relocated the whole config file). After this spec no shipped artifact documents `[workflows]`, and an undocumented section surviving in adopter configs is §drift-prevention's target case. The migration removes the section (header, keys, attached comment lines, normalizing the surrounding blank line), preserves every other table byte-for-byte, reports the removal in the post-scaffolding summary, and no-ops when the section is absent.
- **005-era migrations: subsumed and archived.** `workflows-sunset`'s deletion set covers the 13 current template filenames **and** the 9 legacy `{category}-{language}-{tool}.md` names, and it also removes a legacy `{config_dir}/commands/{project}/skills/` directory (recursive, pinned-checked — necessarily stale per `skills-to-workflows`' own reasoning), so an adopter stranded pre-rename is still fully cleaned. With their targets subsumed, `skills-to-workflows` and `workflow-filename-rename` (both past `sunset_after = "0.10.0"`) leave `framework/migrations.toml` and their procedure text is archived under root `CHANGELOG.md` § Archived migrations per [027](../027-bootstrap-migration-registry/spec.md)'s sunset flow — its first real exercise. The four other expired entries (`gitignore-marker-rename`, `governance-config-rename`, `spec-and-plan-sunset`, `rule-files-relocate`) are out of scope: unrelated maintenance, not bundled with this feature.
- **Spec 005 keeps `status: done` and gains a sunset note.** The lifecycle's five statuses gain no sixth value — `done` remains factually correct (the feature was delivered, criteria verified). A post-completion blockquote at the top of [005-workflows](../005-workflows/spec.md)'s body — the same idiom its existing 019 note uses — records that this spec removed the feature, that `framework/workflows/` and the `/govern` recommendation flow no longer exist, and that the body below stands as the historical record of the feature as shipped. The note lands as part of this spec's removal sweep (mechanical-class under §spec-lifecycle; no back-edge).
- **Tech Stack table parsing leaves `framework/bootstrap/govern.md` entirely.** Grounded by inspection: the canonical-key parsing (`Layer` → `backend_language` / `frontend_language` / …, inside the Workflow recommendation flow) feeds only the registry trigger matching, and `registry.json` is the only other file in the repo naming those keys. `/gov:review`'s tech-stack alignment check reads the AGENTS.md Tech Stack section as freeform prose under host judgment and never consumes the parsed keys. No residual consumer — the parsing text is removed with the flow; [004-tech-stack-selection](../004-tech-stack-selection/spec.md) survives with its documentation role and review-side consumer.
