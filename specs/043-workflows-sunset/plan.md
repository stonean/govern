# 043 — Workflows sunset Plan

Implements [043 — Workflows sunset](spec.md).

## Overview

Delete the workflows feature in one coordinated pass: remove `framework/workflows/` and every procedural surface that drives it (the `/govern` Workflow recommendation flow, the Shared Files registry row, the `[workflows]` config schema, `init.md` §8), register a `workflows-sunset` migration that cleans adopter projects (subsuming the two dormant 005-era migrations, which are archived per 027's sunset flow), sweep the remaining references by meaning across live artifacts, annotate the affected done specs, and ship as gvrn 0.23.0 with the `gvrn-v0.23.0` tag as the completion gate.

## Technical Decisions

### Migration entry: one `workflows-sunset` entry, exact-set deletion

`framework/migrations.toml` gains one entry with `introduced_in = "0.23.0"` (next minor over the current `runtime/Cargo.toml:3` `version = "0.22.0"`) and `sunset_after = "0.25.0"` (the uniform +2-minors window, 027's back-fill convention). Its procedure file `framework/migrations/workflows-sunset.md` follows the shape of the subsumed procedures (idempotency check → per-file pinned check → delete → summary line, per `framework/migrations/workflow-filename-rename.md`):

1. Exact-set deletion in `{config_dir}/commands/{project}/workflows/` of the 22 known names — 13 current (`black.md`, `eslint.md`, `gofmt.md`, `golangci-lint.md`, `gotest.md`, `prettier.md`, `pytest.md`, `rails-migrate.md`, `rspec.md`, `rubocop.md`, `ruff.md`, `rufo.md`, `vitest.md`) + 9 legacy `{category}-{language}-{tool}.md` names (the set enumerated in `workflow-filename-rename`); remove the directory only when empty afterward.
2. Recursive deletion of a remaining `{config_dir}/commands/{project}/skills/` directory (subsumed from `skills-to-workflows`, including its pinned-directory check).
3. Deletion of the synced `workflows/registry.json` (pinned-checked); remove `workflows/` when empty.
4. Removal of the `[workflows]` section (header, keys, attached comment lines) from the **active config file** (write policy per spec 042), all other tables preserved byte-for-byte.

`target_paths` lists the adopter-relative paths (driving the bootstrap loop) plus `framework/workflows/` (driving `/audit` Family 10's no-stale-target-paths check). The explanatory comment about that dual role currently sits inside the `skills-to-workflows` entry (`framework/migrations.toml:44-45`) — it relocates to the `workflows-sunset` entry when the old entries leave.

### Archive per 027's sunset flow

`skills-to-workflows` and `workflow-filename-rename` leave `framework/migrations.toml`; their procedure-file text is appended to root `CHANGELOG.md` under `## Archived migrations` with headings naming id, `introduced_in`, and `sunset_after` (027 acceptance criterion, `specs/027-bootstrap-migration-registry/spec.md:99`), replacing the *"None yet"* placeholder; the two `framework/migrations/*.md` files are deleted. The registry's stale-reference behavior (`framework/bootstrap/govern.md` §Pre-run Migrations) already defines what happens when an adopter's `last_applied` names an archived id — no new mechanism needed.

### `framework/bootstrap/govern.md` surgery

Grounded inventory of every feature reference (line numbers pre-edit):

- `:798-879` — the entire **### Workflow recommendation** section (including the Tech Stack table parsing at `:806-830`, which has no consumer outside the trigger matching — clarify Q5 — and the Auggie discovery note at `:879`).
- `:647` — the `framework/workflows/registry.json` → `workflows/registry.json` Shared Files manifest row.
- `:442-475` — the `[workflows]` block in the config schema and the `workflows.declined_categories` documentation paragraph.
- `:24` — "per-category workflow prompts (§Workflow recommendation, step 8)" in the procedural-fidelity preamble.
- `:36` — `[workflows]` in the managed-block preserve list.
- `:40` — the enforce-manifest note's "legacy `skills/` directory, post-005 workflow filename rename" example (reword to a generic pointer at Pre-run Migrations).
- `:385` — `[workflows].declined_categories` in the write-policy enumeration.
- `:513` — "and the workflow-recommendation flow" in the manifest-entry scope sentence.
- `:733`, `:735` — the per-layout "skip **### Workflow recommendation**" instructions.

Generic uses of the word (`:30` "tar -xzf workflow", `:1072` "fits your workflow") stay.

### `init.md` is hand-maintained — §8 removed by hand

`.claude/commands/gov/init.md` is the one generator exception (AGENTS.md gotcha): its **### 8. Recommend and scaffold workflows** (`:144-168`) is removed by hand, later step numbers renumbered. The framework-implies-language inference at `:40` **stays** — only its justification changes: "since language-triggered workflows … match on it" becomes the table's remaining consumers (documentation value; `/gov:review`'s tech-stack alignment check reads the section as prose, `framework/commands/review.md:100`).

### Constitution edits

- Canonical-source map: drop the `Workflow registry | framework/workflows/registry.json` row (`framework/constitution.md:506`).
- §runtime-boundary eligibility criterion 2(b) (`framework/constitution.md:471`): "implemented as a bash script invoked by `govern` workflows" — the phrase reads as the feature post-removal; reword to "implemented as a bash script the framework invokes (pre-commit hooks, generators, CI)". Meaning unchanged; the criterion never depended on the workflows feature.

There is no root `constitution.md` to sync (verified absent; the AGENTS.md project-structure note claiming one is pre-existing drift, out of scope).

### Command-source and template rewording

- `framework/commands/groom.md` `:49`, `:82`: "the project's lint/format/test workflows cover the common cases" → "the project's lint/format/test tooling covers the common cases" (same meaning, no feature reference; placeholders untouched).
- `framework/commands/link.md` `:71`: drop `[workflows]` from the illustrative preserve list (the list is explicitly non-exhaustive, so removal is safe).
- `framework/templates/project/agents.md` `:74-79`: the Skills comment's disambiguation against "this repo's `framework/workflows/`" is dropped; the Skills description stands alone.
- Regenerate `.claude/commands/gov/*.md` via `scripts/gen-claude-commands.sh` (the pre-commit hook path) after source edits.

### Runtime: comment-only code changes, version bump, no golden re-bless for the version

- `runtime/src/schema/paths.rs:109`: drop `[workflows]` from the host-driven config-writes enumeration in the doc comment.
- `runtime/src/primitives/enforce_manifest.rs:8`: the module doc's "legacy workflow filenames" example becomes a generic "historical conventions" pointer. Test identifiers (`legacy-workflow.md` at `:273-296`, the `workflows` temp dir at `:394`) are arbitrary test data with no feature coupling — left as-is, as is the `govern-basic` fixture's `framework/skills/old-skill.md` (an adopter legacy-state fixture).
- `runtime/Cargo.toml` `0.22.0` → `0.23.0` + a `runtime/CHANGELOG.md` 0.23.0 section. Parity goldens store the version as `{{runtime-version}}` — no re-bless for the bump (AGENTS.md rule; refresh `runtime/target/release/gvrn` instead). If a parity golden embeds changed `groom.md` step text, that specific golden is re-blessed intentionally as a content change — never to absorb a version delta.

### Sibling done-spec annotations (mechanical-class, no back-edge)

Per clarify Q4: `specs/005-workflows/spec.md` gets a post-completion sunset blockquote (the idiom of its existing 019 note) pointing at 043; `004-tech-stack-selection`, `010-agent-autonomy`, and `019-config-decisions` get one-line notes marking their workflow-surface material historical. Passing historical mentions in other specs' plan/tasks bodies (design records) are left — they describe decision-time reality and assert no current behavior.

### Sweep verification

The acceptance sweep greps live artifacts (`framework/`, `scripts/`, `runtime/`, `.github/`, `README.md`, `AGENTS.md`, `.claude/commands/`, spec bodies) for feature tokens — `framework/workflows`, `workflows/registry.json`, `declined_categories`, `[workflows]`, `Workflow recommendation`, `workflow-filename-rename`, `skills-to-workflows`, `:workflows:` — and classifies each remaining hit as (a) archived-recipe text in `CHANGELOG.md`, (b) sunset annotation, (c) GitHub Actions path, (d) generic prose, or (e) 043's own artifacts; anything else fails the sweep. The prose-claim pass then greps by meaning ("scaffold", "recommend", "declined") for behavioral claims token greps miss.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/workflows/` (14 files) | Delete | The feature's templates + `registry.json` |
| `framework/bootstrap/govern.md` | Modify | Remove recommendation flow, manifest row, config schema, preamble/write-policy/layout mentions |
| `framework/migrations.toml` | Modify | Add `workflows-sunset`; remove two subsumed entries; relocate audit comment |
| `framework/migrations/workflows-sunset.md` | Create | Migration procedure |
| `framework/migrations/skills-to-workflows.md` | Delete | Archived to `CHANGELOG.md` |
| `framework/migrations/workflow-filename-rename.md` | Delete | Archived to `CHANGELOG.md` |
| `CHANGELOG.md` | Modify | § Archived migrations gains both recipes |
| `framework/constitution.md` | Modify | Drop map row; reword §runtime-boundary 2(b) |
| `framework/commands/groom.md` | Modify | Reword two "workflows" phrasings |
| `framework/commands/link.md` | Modify | Drop `[workflows]` from preserve list |
| `framework/templates/project/agents.md` | Modify | Drop workflows disambiguation from Skills comment |
| `README.md` | Modify | Drop `[workflows]` docs, example TOML section, repo-layout row; prose-claim pass |
| `AGENTS.md` | Modify | Drop project-structure bullet, gotcha, procedural-fidelity mention |
| `.claude/commands/gov/init.md` | Modify | Remove §8 by hand; reword `:40` justification |
| `.claude/commands/gov/groom.md`, `link.md` | Regenerate | Via `scripts/gen-claude-commands.sh` |
| `runtime/src/schema/paths.rs` | Modify | Doc-comment enumeration |
| `runtime/src/primitives/enforce_manifest.rs` | Modify | Module doc-comment example |
| `runtime/Cargo.toml`, `runtime/CHANGELOG.md` | Modify | 0.23.0 bump + entry |
| `specs/005-workflows/spec.md` | Modify | Sunset note (status stays `done`) |
| `specs/004-tech-stack-selection/spec.md` | Modify | Historical-surface note |
| `specs/010-agent-autonomy/spec.md` | Modify | Historical-surface note |
| `specs/019-config-decisions/spec.md` | Modify | Historical-surface note |

## Trade-offs

- **Deprecate-in-place (stop offering, keep files) — rejected.** Dead templates and an unreferenced registry are exactly the drift §drift-prevention exists to prevent; the manifest's `update` strategy would also keep rewriting an orphaned `workflows/registry.json` into adopter repos forever.
- **Relocate templates to an examples/ directory — rejected.** Still a maintenance surface making the same claim (that govern owns lint/test/format invocation) the removal rationale rejects; adopters wanting these recipes have their tools' own documentation.
- **Big-bang directory delete in the migration — rejected in clarify Q1.** Exact-set deletion costs a fixed filename list but preserves adopter-authored files; the subsumed `workflow-filename-rename` set this precedent.
- **Known limitation:** an adopter who pinned or customized scaffolded workflow files keeps orphaned slash commands under the `{project}:workflows:` namespace — their files, their choice; nothing references them after the sweep.
- **Known limitation:** after `sunset_after = "0.25.0"` passes, stragglers apply `workflows-sunset` manually from the `CHANGELOG.md` recipe — the accepted 027 contract for every migration.
