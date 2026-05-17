---
title: "013-text-first-artifacts — plan"
---

# 013 — Text-First Artifacts Plan

## Overview

Land the principle in the constitution, update the templates and the slash command sources to read/write YAML frontmatter, and make migration the responsibility of the existing `/govern` bootstrap. Migration logic lives in `framework/bootstrap/govern.md` (no new command, no new download surface, no new failure mode). Governance dogfoods by self-migrating its own specs as a manual implementation task — there is no `/govern` to run on this repo.

The work is mostly textual: editing markdown source files. There is no library to add, no parser to write, no migration script in code form. The slash commands are markdown executed by an agent, so "update the parser" means rewriting the prose instructions in each command file from "read the `**Status:**` line" to "read `status` from the YAML frontmatter."

## Technical Decisions

### Frontmatter format

Standard YAML frontmatter — `---` delimiter line, YAML key-value block, `---` delimiter line, blank line, then the markdown body. No alternative format (TOML, JSON, custom) considered: YAML is the universal PKM convention (Quartz, Obsidian, Logseq, MkDocs, Hugo, Jekyll, Eleventy all expect it), and keeping with the convention is the entire point of opting into ecosystem compatibility.

Example (specs):

```yaml
---
status: clarified
dependencies: [000-slash-commands, 007-govern-workflow]
tags: [format, migration]
---
```

Example (scenarios):

```yaml
---
spec-ref: 000-slash-commands — Command Set / implement
tags: [format]
---
```

### No parser library

Slash commands are markdown the agent reads and follows. The agent parses YAML directly when instructed to. No npm/pip/go dependency is added. This preserves the "adopting governance requires no bootstrap tooling beyond the AI agent itself" property the principle is declaring.

### Constitution section placement

New `§text-first-artifacts` section placed immediately after `§scenarios` and before `§pipeline-boundaries`. Reasoning: scenarios introduce the last artifact kind covered by the schema, so the principle naturally follows; pipeline-boundaries is a higher-level governance rule that benefits from artifact rules being established first. The HTML comment anchor `<!-- §text-first-artifacts -->` follows existing convention.

### Schema declared as a markdown table

Per resolved Q1. The constitution table has columns `Field | Required | Type | Allowed values | Description`. Below the table:

- An explicit list of artifact kinds the schema applies to (specs, scenarios) and a note that other artifacts MAY include frontmatter when a consumer benefits.
- The open-schema rule (additional fields permitted, ignored by uninterested consumers).
- A starter tag vocabulary as guidance, not enforcement.

Schema details captured in `data-model.md` for cross-reference.

### Migration logic in `framework/bootstrap/govern.md`

Embedded as a new step in the existing govern.md flow, between the agent-selection phase and the file-manifest fetch phase. Migration step does:

1. Run `git status --porcelain -- specs/` (project-relative; the project has no `framework/templates/spec/` of its own — that was governance's path, not the adopter's). If output is non-empty, refuse with a clear message and exit.
2. Walk `specs/**/spec.md`, `specs/**/spec-and-plan.md`, and `specs/**/scenarios/*.md`.
3. For each file, check whether the first non-blank line is `---`. If yes, skip (already migrated). If no and bold-prefix lines (`**Status:**`, `**Dependencies:**`, `**spec-ref:**`) are present, convert: insert a frontmatter block at the top, remove the now-redundant bold-prefix lines from the body.
4. Print a per-file summary (`migrated`, `skipped (already frontmatter)`, `skipped (pinned)`, `skipped (no metadata to migrate)`).
5. Exit; the user reviews via `git diff` and commits or restores.

Idempotency falls out of the `---`-check on re-run. The clean-tree precheck makes the migration diff atomic and reviewable.

### Self-migration of governance's own specs

Governance has no `/govern`. The work is a manual implementation task: convert each `specs/NNN-*/spec.md` and each scenario file under `specs/*/scenarios/` to frontmatter format. The conversion is mechanical, the agent does it once per file, lint passes confirm correctness.

### Tag prompt UX in `/gov:specify`

`/gov:specify` reads `specs/*/spec.md` (and `spec-and-plan.md`) frontmatter to collect the union of existing `tags`. At creation time, prompt: *"Tags for this spec? Existing tags in this repo: \[cli, bootstrap, process, ...\]. Enter one or more (or skip)."* Author input either selects from suggestions, adds new tags, or skips (writes `tags: []`). This implements Q2's reinforcement model without enforcing non-empty tags.

### Missing-tags advisory in `/gov:clarify`

At the `draft → clarified` transition, after all open questions are resolved and before the validation gate, check whether `tags` is empty. If so, surface as one of the validation findings with severity advisory: *"Tags are empty. Adding tags helps cross-cutting graph views. Add some, or proceed without."* The advisory does not block the transition.

### Validate strict/advisory split

Per Q4. The validate command's instructions document the severity per check:

- Hard fail: missing/invalid `status`, missing/invalid `dependencies`, malformed YAML, no frontmatter block on a spec or scenario file.
- Advisory: empty `tags`, unknown fields (informational), checkbox mismatches (existing), cross-spec reference issues (existing).

### Order of operations

Strict ordering matters because later steps depend on earlier work being correct:

1. Constitution (declares the rule and schema)
2. Templates (newly created specs use the format from day one)
3. Slash command sources (read/write the format the templates produce)
4. Regenerate `.claude/commands/gov/*` from the updated framework sources
5. Govern.md migration logic (relies on schema being canonical in the constitution)
6. Self-migration of governance's existing specs (uses the new template format and the new commands as their canonical reference)
7. README "Viewing artifacts" section (documents the rendering convention adopted)
8. Code-location-index scenario note (refers to a constitution section that now exists)

Phases 2 and 3 can run in parallel within the same session; everything else is sequential.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/constitution.md` | Modify | Add `§text-first-artifacts` section: principle, schema table, starter tag vocabulary, scope note |
| `framework/templates/spec/spec.md` | Modify | Replace `**Status:**` / `**Dependencies:**` with YAML frontmatter; include `tags: []` |
| `framework/templates/spec/spec-and-plan.md` | Modify | Same as `spec.md` |
| `framework/templates/spec/scenario.md` | Modify | Replace bold-prefix `**spec-ref:**` with YAML frontmatter `spec-ref` |
| `framework/commands/specify.md` | Modify | Write frontmatter on creation; add tag prompt with sibling-spec autocomplete |
| `framework/commands/clarify.md` | Modify | Read frontmatter status; write status on advance; advisory for empty `tags` at draft→clarified |
| `framework/commands/plan.md` | Modify | Read frontmatter status; write status on advance |
| `framework/commands/implement.md` | Modify | Read frontmatter status; write status on advance |
| `framework/commands/status.md` | Modify | Parse frontmatter for status, dependencies, tags in dashboard |
| `framework/commands/target.md` | Modify | Read frontmatter status when displaying target detail |
| `framework/commands/analyze.md` | Modify | Implement strict/advisory split per data-model schema |
| `framework/commands/capture.md` | Modify | Write frontmatter for new sketch specs |
| `framework/commands/groom.md` | Audit/Modify | Update only if it reads spec metadata for routing |
| `framework/commands/elaborate.md` | Audit/Modify | Update only if it touches scenario metadata (will: scenarios now use frontmatter `spec-ref`) |
| `framework/commands/ask.md` | Audit/Modify | Update only if it reads spec/scenario metadata to identify the target |
| `framework/commands/log.md` | Audit | Likely no change (inbox.md is out of frontmatter scope) |
| `framework/commands/help.md` | No change | Fixed text, no metadata parsing |
| `framework/bootstrap/govern.md` | Modify | Add migration step (git precheck, walk, convert); add `npx quartz specs/` to post-run tip output |
| `.claude/commands/gov/*.md` | Regenerate | Run `scripts/gen-claude-commands.sh` after framework command sources are updated |
| `specs/000-slash-commands/spec.md` | Migrate | Self-migration to frontmatter |
| `specs/001-system-spec-templates/spec.md` | Migrate | Self-migration |
| `specs/002-project-scaffolding/spec.md` | Migrate | Self-migration |
| `specs/003-bootstrap-automation/spec.md` | Migrate | Self-migration |
| `specs/004-tech-stack-selection/spec.md` | Migrate | Self-migration |
| `specs/005-workflows/spec.md` | Migrate | Self-migration |
| `specs/006-bug-workflow/spec.md` | Migrate | Self-migration |
| `specs/007-govern-workflow/spec.md` | Migrate | Self-migration |
| `specs/008-security-rules/spec.md` | Migrate | Self-migration |
| `specs/009-scenario-targeting/spec.md` | Migrate | Self-migration |
| `specs/010-agent-autonomy/spec.md` | Migrate | Self-migration |
| `specs/011-brownfield-process/spec.md` | Migrate | Self-migration |
| `specs/012-multi-agent-govern/spec.md` | Migrate | Self-migration |
| `specs/013-text-first-artifacts/spec.md` | Migrate | Self-migration (this spec itself, last so the migration process can be exercised on a known-good source) |
| `README.md` | Modify | Add "Viewing artifacts" section documenting `npx quartz` as recommended viewer |
| `specs/README.md` | No change | Cross-cutting decisions doc; no spec metadata |

## Trade-offs

### Considered and rejected

- **Add a JSON Schema file alongside the constitution table.** Rejected per Q1 — single source of truth, no drift, and no current tooling consumer needs it. Reversible if a consumer emerges.
- **Tags required (non-empty).** Rejected per Q2 — performative placeholders pollute graph views, migration friction outweighs the marginal consistency gain over strong-default-optional.
- **Object form for `dependencies`.** Rejected per Q3 — preemptive structuring with no consumer that distinguishes hard from soft. YAML's mixed-mode lists keep the forward path open.
- **Pure-advisory validate.** Rejected per Q4 — required-field violations make the validator's report less reliable than runtime; hard fail on parsing failures keeps the contract tight.
- **Frontmatter on all artifacts (system.md, errors.md, events.md, inbox.md, plan/tasks files).** Rejected per Q5 — none have lifecycle or pipeline-readable metadata; stubs would be ceremony for nothing.
- **Quartz recommendation in project-readme template.** Rejected per Q6 — adopters' READMEs serve their product's users, not their governance maintainers; templates calcify, central recommendation does not.
- **`.governance-migration-backup/` directory.** Rejected per Q8 — parallel undo system that disagrees with git, rots after migration, contradicts existing `update`/`create`/`skip` strategy.

### Known limitations

- **Tag fragmentation risk.** Free-form tags can drift (`cli` vs. `commands` vs. `slash-commands`). The starter vocabulary in the constitution and the sibling-spec autocomplete in `/gov:specify` mitigate but don't enforce. Accepted because rigid taxonomy adds ceremony without proportional value.
- **Single-commit migration diff.** Adopters with many specs see one large diff when `/govern` runs the migration. Accepted because git review tools handle this fine, atomicity is a feature, and the alternative (multi-commit migration) requires the bootstrap to make multiple commits in the user's repo, which is out of scope for governance.
- **Scenario non-spec-ref metadata is preserved as-is.** Migration converts `**spec-ref:**` to frontmatter but leaves any other bold-prefix patterns in scenario bodies untouched (there are no other recognized scenario fields today). If projects have invented scenario metadata conventions, those stay in body prose unchanged.
- **No automated test for govern.md migration.** The migration step is markdown instructions executed by the agent, not code. Verification is manual: run on a test fixture (a copy of a brownfield project's `specs/`) and confirm the diff. Tracked as a task.

## Open Questions Resolved

All eight resolved during `/gov:clarify`. Resolutions of record live in `spec.md`'s **Resolved Questions** section. Summary:

- **Schema location and format** — markdown table in constitution; defer JSON Schema.
- **Required vs. optional fields** — required: `status`, `dependencies`. Tags optional with three reinforcement points.
- **Field naming for dependencies** — flat list of slugs; defer object form.
- **Validation strictness** — hard fail on required-field violations; advisory on rest.
- **Migration scope for non-spec artifacts** — schema covers specs and scenarios only.
- **Quartz recommendation scope** — this repo's README only; one-line tip in govern.md post-run output.
- **Code-location-index scenario interaction** — scenario stays parked under 000; resolutions derive from 013.
- **Migration safety net** — rely on git; clean-tree precheck scoped to `specs/`.
