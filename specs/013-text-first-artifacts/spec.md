---
title: "013-text-first-artifacts — spec"
status: done
dependencies: [000-slash-commands, 007-govern-workflow, 012-multi-agent-govern]
tags: [format, migration, pipeline]
---

# 013 — Text-First Artifacts

Declare governance's implicit "all artifacts are markdown" principle in the constitution, formalize spec metadata as YAML frontmatter, and migrate adopted projects to the new format on the next `/govern` run.

## Problem

Governance has always treated every artifact — constitution, specs, plans, tasks, scenarios, rules — as plain markdown the agent edits with `Edit`. That stance is load-bearing in non-obvious ways: the agent's write path stays simple, PRs review glanceably, merge conflicts stay rare and human-resolvable, and adopting governance requires no bootstrap tooling beyond the AI agent itself. But the principle is implicit. As candidates for new artifacts surface (code-location indexes, dependency graphs, audit trails), there is no declared rule for when structured storage is permitted and what constraints it must meet, so each one risks relitigating the question from scratch.

Spec metadata today (`**Status:** in-progress`, `**Dependencies:** 002, 005`) is bold-prefix text parsed by every consuming slash command via custom regex. The cost compounds: adding a metadata field means touching every parser; type semantics (lists, enums, dates) must be string-decoded ad hoc; cross-spec schema validation is impossible without per-command logic; and any external tool that wants to read governance artifacts has to reimplement the parser. A move to YAML frontmatter — the de facto standard for markdown-with-metadata — preserves the text-first stance while unlocking structured types, schema-driven validation, and ecosystem compatibility (Quartz, Obsidian, Logseq, MkDocs, GitHub Actions, `yq`).

## Behavior

### Principle Declaration

The constitution gains a new section declaring text-first artifacts as a guiding rule:

- All governance artifacts are markdown by default. The agent reads and writes them with the same `Edit` flow used for code.
- Structured metadata lives in YAML frontmatter at the top of each markdown file; the document body remains markdown prose.
- Cross-artifact references use standard relative markdown links (`[label](../path.md)`), not wiki-links — this keeps PRs reviewable on GitHub and viewers like Quartz/Obsidian still resolve them.
- Source-of-truth artifacts are markdown. Structured derived views (SQLite caches, generated graph data, JSON indexes) are permitted only as gitignored build artifacts that consumers regenerate on demand. They never become the canonical record.
- Exceptions to text-first source-of-truth require an explicit constitutional amendment with stated rationale.

### Frontmatter Schema

The schema applies to **spec files** (`spec.md`, `spec-and-plan.md`) and **scenario files** (`scenarios/{slug}.md`). Other governance artifacts (`system.md`, `errors.md`, `events.md`, `inbox.md`, plan files, tasks files, rule files, README files) MAY include frontmatter when a specific consumer benefits, but are not required to. The schema is declared as a markdown table in `framework/constitution.md`, next to the text-first principle, and is the authoritative source for `/gov:validate` and any tooling.

**Required fields for spec files:**

- `status` — one of `draft`, `clarified`, `planned`, `in-progress`, `done`
- `dependencies` — list of spec slugs this feature depends on; empty list permitted

**Required fields for scenario files:**

- `spec-ref` — string identifying the parent spec and section the scenario elaborates (replaces the bold-prefix `**spec-ref:**` line)

**Standard optional fields (specs and scenarios):**

- `tags` — list of free-form strings used by graph-view consumers (Quartz, Obsidian, etc.) for cross-cutting groupings. Treated as first-class for spec files through three reinforcement points: (1) bundled spec templates include `tags: []` so authors see the field at every new spec, (2) `/gov:specify` prompts for at least one tag at creation and surfaces existing sibling specs' tags as suggestions, (3) `/gov:clarify` flags missing or empty tags as an advisory finding at the `draft → clarified` transition (not a hard gate). The constitution publishes a starter vocabulary as guidance, not enforcement; new tags can be introduced as needed. Scenarios may also carry tags for graph-view consistency.

The schema is open: additional fields beyond those listed are permitted and ignored by uninterested consumers, leaving room for future metadata (`owner`, `target_release`, etc.) without coordinated parser changes.

### Slash Command Updates

Every slash command source in `framework/commands/` (and the regenerated `.claude/commands/gov/` instances) that reads or writes spec metadata is updated to use frontmatter parsing instead of bold-prefix regex. At minimum this covers: `/gov:status`, `/gov:target`, `/gov:clarify`, `/gov:plan`, `/gov:implement`, `/gov:validate`, `/gov:groom`, `/gov:capture`. Commands that don't read metadata are unaffected.

Templates (`framework/templates/spec/spec.md`, `spec-and-plan.md`) ship with the new frontmatter format so newly created specs use it from day one.

### Migration via `/govern`

The next `/govern` run in any adopted project performs the migration:

1. Precheck `git status --porcelain` scoped to `specs/` and `framework/templates/spec/` (or the project's equivalent). If dirty, refuse with a clear message instructing the user to commit or stash their in-flight changes, then exit. Unrelated in-flight work elsewhere in the tree does not block migration.
2. Walk the project's `specs/` directory and detect spec files (`spec.md`, `spec-and-plan.md`) and scenario files (`scenarios/{slug}.md`) using bold-prefix metadata (no frontmatter block present at the top of the file).
3. Convert bold-prefix lines (`**Status:**`, `**Dependencies:**` for specs; `**spec-ref:**` for scenarios; any other recognized fields) to YAML frontmatter at the top of the file.
4. Strip the now-redundant bold-prefix lines from the document body.
5. Leave non-spec artifacts (`system.md`, `errors.md`, `events.md`, `inbox.md`, plan files, tasks files, rule files) untouched — frontmatter is not required for these.
6. Apply governance's standard `update`/`create`/`skip` strategy to the project's bundled spec and scenario templates and slash commands so they pick up the new format.
7. Pinned files (via `.governance.toml`) are skipped — the adopter is responsible for their own migration of pinned files.
8. Print a summary of converted files. The user reviews the result via `git diff`, commits, or aborts via `git restore`. No backup directory is created — git is the recovery mechanism.

Migration is idempotent: re-running `/govern` on an already-migrated project produces no further metadata changes.

### Rendering Convention

This repo's `README.md` documents `npx quartz` as the recommended viewer for browsing governance artifacts as a graph. Quartz is recommended, not enforced; the artifacts work unchanged in Obsidian, Logseq, Foam, MkDocs, or no viewer at all. The point of the recommendation is to give adopters a default answer to "how do I see this as a graph?" without prescribing tooling.

### Governance Self-Migration

This repo dogfoods the principle: every existing spec under `specs/` in this repo is migrated to frontmatter as part of implementation. Governance has no `/govern` to run on itself — the migration is manual work captured in `tasks.md`.

## Edge Cases

- **Spec with malformed bold-prefix metadata** (missing `**Status:**` line, typo in field name): migration logs a warning and skips the file; the user repairs manually before re-running.
- **Spec already partially migrated** (frontmatter present but body still has bold-prefix lines): migration completes the conversion idempotently — frontmatter wins, redundant body lines are removed.
- **Pinned spec files via `.governance.toml`**: skipped during migration. The adopter receives a summary listing pinned files so they know which need manual conversion.
- **Project on an older governance version**: `/govern` always migrates to the current schema. There is no version negotiation; older projects pull current.
- **Spec with custom non-schema fields in bold-prefix form** (e.g., a project added their own `**Owner:**` line): migration preserves these as additional frontmatter fields. The schema permits unknown fields by design.
- **Spec created manually outside `/gov:specify`** (e.g., direct file creation): the bundled template's `tags: []` placeholder is visible as a reminder, but no creation-time prompt fires. The advisory finding at `/gov:clarify` catches missing tags before the spec advances to `clarified`.
- **Migrated specs with no tag signal**: migration adds frontmatter without populating `tags`. Backfill is organic — every subsequent `/gov:clarify` pass on the spec is a chance to add tags.
- **Non-spec artifacts** (`system.md`, `errors.md`, `events.md`, `inbox.md`, plan files, tasks files, rule files): migration leaves them untouched. They have no required schema and are not part of the lifecycle the schema models.
- **Scenario files using bold-prefix `spec-ref`**: migration converts the bold-prefix line to a `spec-ref` frontmatter field and removes the redundant body line. Scenarios remain status-less per the constitution; only `spec-ref` is required.

## Acceptance Criteria

- [x] `framework/constitution.md` declares the text-first artifacts principle in a new section, including the frontmatter requirement, relative-link rule, and structured-derived-view caveat.
- [x] The frontmatter schema is declared as a markdown table in `framework/constitution.md`, listing required fields per artifact kind (specs: `status`, `dependencies`; scenarios: `spec-ref`), standard optional fields (`tags`), and the open-schema rule for additional fields.
- [x] The constitution declares the schema applies to spec files and scenario files only; other artifacts (`system.md`, `errors.md`, `events.md`, `inbox.md`, plan files, tasks files, rule files) MAY include frontmatter when a consumer benefits but are not required to.
- [x] The constitution publishes a starter `tags` vocabulary as guidance (not enforcement).
- [x] `framework/templates/spec/spec.md` and `framework/templates/spec/spec-and-plan.md` use YAML frontmatter and include `tags: []` as a visible placeholder so authors see the field at every new spec.
- [x] `framework/templates/spec/scenario.md` uses YAML frontmatter for `spec-ref` instead of bold-prefix.
- [x] `/gov:specify` prompts for at least one tag at spec creation time, surfacing existing sibling specs' tags as suggestions; the author can decline (leaving the list empty) without blocking creation.
- [x] `/gov:clarify` flags missing or empty `tags` as an advisory finding at the `draft → clarified` transition; the finding does not block the transition.
- [x] Every existing spec in this repo's `specs/` directory uses frontmatter; no spec file contains both formats.
- [x] Every slash command source under `framework/commands/` that reads or writes spec metadata parses frontmatter, not bold-prefix lines.
- [x] `/gov:validate` hard-fails on required-field violations (missing or invalid `status`, missing or invalid `dependencies`, malformed YAML, or no frontmatter block) and reports missing `tags`, unknown fields, and other discrepancies as advisory findings.
- [x] `/govern` (the unified bootstrap from 012) detects pre-frontmatter spec files in adopted projects and migrates them on its next run.
- [x] `/govern` migration is idempotent — running it twice on the same project produces no second-run changes.
- [x] `/govern` migration prechecks `git status --porcelain` scoped to `specs/` and refuses to run on a dirty tree, instructing the user to commit or stash. No automatic backup directory is created; git is the recovery mechanism.
- [x] `/govern` migration respects `.governance.toml` pinning — pinned files are skipped and surfaced in the post-run summary.
- [x] The root `README.md` includes a "Viewing artifacts" section that documents `npx quartz` as the recommended viewer and notes that other PKM tools work unchanged. The recommendation lives in this repo only — the project-readme template is unchanged.
- [x] `framework/bootstrap/govern.md` post-run output mentions `npx quartz specs/` as a one-line tip so adopters discover the viewer at bootstrap time without the recommendation being baked into their own README.
- [x] During 013's implementation, a note is added to `specs/000-slash-commands/scenarios/code-location-index.md` pointing at 013 as the resolving framework, so 000's next `/gov:clarify` pass on the scenario can resolve its open questions through the lens of the text-first principle (location and maintenance auto-resolved by "structured derived view"; consumer question becomes a gate, not a survey).
- [x] All updated and migrated `.md` files pass `npx markdownlint-cli2`.

## Open Questions

_All open questions resolved. See Resolved Questions below._

## Resolved Questions

- **Schema location and format** — markdown table in the constitution, next to the text-first principle declaration. Defer JSON Schema until a concrete tooling consumer asks for it. This aligns with the principle the spec is declaring (markdown canonical, structured forms are derived views regenerated on demand), keeps a single source of truth that cannot drift, and is sufficient because the agent is the primary validator and reads the table natively. The table has columns `Field | Required | Type | Allowed values | Description` and is open — additional fields beyond the required set are permitted and ignored by uninterested consumers, preserving the cheap-to-extend property. JSON Schema can be added later as a derived artifact if a tool that needs it (e.g., editor autocomplete, CI hook) emerges.
- **Required vs. optional fields at launch** — required fields are `status` and `dependencies` only. `tags` is a standard optional field treated as first-class through three reinforcement points: bundled spec templates include `tags: []` as a visible placeholder, `/gov:specify` prompts for at least one tag at creation (with autocomplete from existing sibling specs), and `/gov:clarify` flags missing tags as an advisory finding at the `draft → clarified` transition (not a hard gate). Tag values are free-form strings; the constitution publishes a starter vocabulary as guidance, not enforcement, so adopters converge on a small consistent set without rigid ceremony. This avoids forced placeholders that pollute graph views and keeps migration friction at zero — graph-view value accrues organically as specs are touched. Other speculative fields (`description`, `created_at`) remain optional or omitted: `description` duplicates the prose body, and `created_at` is authoritative in git. The schema is open — additional fields beyond required and standard-optional are permitted and ignored by uninterested consumers.
- **Field naming for dependencies** — flat list of slugs (`dependencies: [002-events, 005-auth]`). No current consumer distinguishes hard from soft dependencies — `/gov:status` and `/gov:clarify`'s gate check both treat the relationship as binary. Object form (`[{slug: 002-events, kind: hard}]`) preemptively encodes a distinction nothing reads, and migration from bold-prefix to flat list is mechanical. The forward path is open: YAML accepts mixed flat-and-object lists in the same document, so a future per-dep metadata need (kind, via, since) can be introduced without re-flattening — and starting simple carries lower regret risk than starting structured. Empty list permitted (replaces the bold-prefix `none` convention).
- **Validation strictness** — split by field criticality. `/gov:validate` hard-fails on required-field violations: missing or invalid `status`, missing or invalid `dependencies`, malformed YAML in the frontmatter block, or no frontmatter block at all. Everything else stays advisory: missing or empty `tags`, unknown fields (legal under the open-schema rule, surfaced as informational), and existing advisory checks (checkbox mismatches, cross-spec reference issues). Required fields are load-bearing — `/gov:status` and the pipeline gates cannot function if `status` is unparseable, so advisory treatment would make the validator's report less reliable than runtime behavior. Pipeline gates already do runtime enforcement on `status`; validate's job is batch surfacing for the rest, not duplicate gating. The hard-fail surface is small enough that adopters running `/govern` to migrate will not see a spec graveyard.
- **Migration scope for non-spec artifacts** — frontmatter-free for now. The schema applies to spec files (`spec.md`, `spec-and-plan.md`) and scenario files (`scenarios/{slug}.md`); other markdown artifacts (`system.md`, `errors.md`, `events.md`, `inbox.md`, plan files, tasks files, rule files, README files) MAY include frontmatter when a specific consumer benefits but are not required to. None of the non-spec artifacts have a status lifecycle, none have dependencies the pipeline reads, and no current consumer needs metadata on them — so a stub would be performative and migration ceremony for nothing. Scenarios are pulled in scope because they have a meaningful required field (`spec-ref`) currently in bold-prefix form, and uniform parsing across pipeline-relevant artifacts keeps the slash-command parser story simple. Forward path is open: any artifact kind can declare its own schema later if a need emerges. Concrete deferred candidate: optional frontmatter on `tasks.md` (and possibly `plan.md`), surfaced during 010 clarify, with potential consumers being a `/gov:validate` rule, a graph view, or cross-cutting task queries; deferred until one of those consumers actually drives the requirement.
- **Quartz recommendation scope** — this repo's README only. The project-readme template is not modified. Adopters' READMEs serve their _product's_ users, not their governance maintainers — telling product users to browse specs with `npx quartz` assumes specs are a primary artifact for that audience, which usually they aren't. Quartz is a maintainer concern, and the maintainer is the same person who runs `/govern` and reads governance's README, so they'll find the recommendation where it lives. Templates calcify — embedding the recommendation in the project-readme template means every adopter ships a README mentioning Quartz, and if Quartz is later deprecated, every adopter's README is wrong. The principle being declared (text-first, portable artifacts) means any adopter who wants Quartz adopts it in three lines without governance prescribing it. Discoverability is preserved by adding a one-line tip to `framework/bootstrap/govern.md`'s post-run output ("`npx quartz specs/` for a graph view"), so adopters encounter the viewer at bootstrap time without committing the recommendation to their own repo permanently.
- **Interaction with the `code-location-index` scenario under 000** — the scenario stays parked under 000 until 013 ships. Once the text-first principle lands in the constitution, most of the scenario's open questions resolve automatically through the "structured derived view" framing: location and maintenance collapse to "derived view, regenerated on demand" (whether top-level or per-spec is still a small design choice for whoever builds the consumer); the consumer question becomes a gate on building anything, not a survey of speculative readers. Granularity remains the one open question that 013 doesn't auto-resolve. During 013's implementation, a note is added to the scenario file pointing at 013 as the resolving framework, so 000's next `/gov:clarify` pass on the scenario picks up where this one leaves off.
- **Migration safety net** — rely on git, no backup directory. `/govern`'s migration step prechecks `git status --porcelain` scoped to `specs/` and `framework/templates/spec/`; if dirty, refuse with a message instructing the user to commit or stash. If clean, migrate in place and print a summary; the user reviews via `git diff`, commits, or aborts via `git restore`. A `.governance-migration-backup/` directory would be a parallel undo system that disagrees with git the moment anyone runs `git restore`, would rot once the migration is complete (orphaned files no one trusts), and would contradict `/govern`'s existing `update`/`create`/`skip` strategy that overwrites files in place. The clean-tree precheck is the actual safety net — it ensures the migration diff is reviewable, atomic, and revertable. The check is scoped (not whole-repo) so unrelated in-flight work elsewhere does not block migration.
