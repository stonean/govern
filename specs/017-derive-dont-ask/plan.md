# 017 — Derive, Don't Ask Plan

Implements [017 — Derive, Don't Ask](spec.md).

## Overview

Discipline-cleanup pass across templates, commands, constitution, validate, and the bootstrap installer. Adds three new generators (`gen-readme-table.sh`, `gen-help-tables.sh`, `gen-spec-deps.sh`), one new pre-commit hook (`.githooks/pre-commit`), one new install script (`scripts/install-hooks.sh`), one new rule file (`framework/rules/configuration.md`), and one new adopter hook surface under `framework/bootstrap/hooks/`. Migrates existing dogfood specs only insofar as adding inline links to spec bodies for any frontmatter dependency not already linked (so the new `gen-spec-deps.sh` generator does not strip them on first run). Frontmatter field deletions are not retroactively migrated — done specs are frozen archaeology per the constitution. Collapses the twin constitutions to a single canonical file at `framework/constitution.md` and deletes root `constitution.md`.

The work is intentionally additive in mechanism (add generators and hooks) and subtractive in surface (remove fields, sections, and `--fix` mode). No new agent-facing concepts are introduced.

## Technical Decisions

### Single canonical constitution at `framework/constitution.md`

Per spec Q2: collapse the twin constitutions. Root `constitution.md` is deleted. Root `CLAUDE.md` updates `@import constitution.md` → `@import framework/constitution.md`. Root `README.md` link to `constitution.md` updates likewise. `/govern`'s install-time remap (`framework/constitution.md` → `constitution.md` at the adopter root) is unchanged — adopter projects continue to receive `constitution.md` at their root. The AGENTS.md "mirror constitutions" instruction is removed.

Alternative considered: generator with section markers — rejected because the historical divergence between the two files was zero at decision time, so the discipline trap was guarding nothing. Pre-emptive scaffolding for hypothetical divergence violates the "don't design for hypothetical future requirements" guidance in the system prompt.

### `--fix` mode removed entirely from `/validate`

Per spec Q1: checkboxes are flipped only by `/implement` (which marks parents and sub-items together at verification, `framework/commands/implement.md:91, 99`) and by `/clarify` (which moves resolved questions section-to-section). Hand-implementation that bypasses `/implement` is treated as out-of-framework — no fix-up tool ships for it. Combined with the title-field removal, `--fix` mode has nothing to do, so the entire flag is deleted from `/validate`.

### Body inline links are authoritative for `dependencies`

Per spec Q7: frontmatter `dependencies` is fully derived from inline markdown links to sibling specs in the spec body, excluding code fences. The new generator `scripts/gen-spec-deps.sh` walks every `specs/*/spec*.md`, computes the union, and rewrites the frontmatter list. The author maintains links in the body — no place to author the frontmatter list directly.

Sync runs in two places:

1. **Pre-commit hook** — primary sync. Runs `gen-spec-deps.sh` on every commit and stages the result.
2. **Command-entry recompute** — `/clarify`, `/plan`, `/implement`, `/elaborate`, `/ask`, and `/target` recompute on entry as an idempotent safety net for uncommitted body edits.

Removal mechanism: remove the inline link, or move it inside a code fence (the scanner ignores fenced-code links). To mention a spec without depending on it, use a bare slug per Q4.

### Pre-commit hook architecture

govern repo's `.githooks/pre-commit` orchestrates all four generators:

1. `scripts/gen-claude-commands.sh` (existing)
2. `scripts/gen-readme-table.sh` (new, this spec)
3. `scripts/gen-help-tables.sh` (new, this spec)
4. `scripts/gen-spec-deps.sh` (new, this spec)

Hook installation via `scripts/install-hooks.sh` (sets `git config core.hooksPath .githooks`). Idempotent — safe to run repeatedly.

Generators run unconditionally on every commit (no file-change gating). Trades a fraction of a second per commit for a one-line implementation that can't get the gate logic wrong.

### Adopter hook surface and `/govern` integration

Per spec Q7 expansion: `framework/bootstrap/hooks/pre-commit` and `framework/bootstrap/hooks/install.sh` ship with the framework. The shipped hook calls only adopter-relevant generators — initially just `gen-spec-deps.sh`. Slot is extensible.

`/govern` adds a new "Hook installation" section between **Per-Agent Scaffolding** and **Post-Scaffolding Output**. On every run it detects state and acts:

| Detected state | Action |
| --- | --- |
| `core.hooksPath` unset and `.githooks/pre-commit` absent | Install both, set `core.hooksPath .githooks`, report installed |
| `.githooks/pre-commit` exists from a prior `/govern` run (detected by a sentinel comment in the file) | Overwrite (`update` strategy, pinnable via `.govern.toml`) |
| `core.hooksPath` points elsewhere, OR `.githooks/pre-commit` exists without the sentinel comment, OR husky/lefthook/`.pre-commit-config.yaml` detected | Skip install; report a warning with a manual integration snippet; continue |

The sentinel comment is a single line near the top of the shipped hook (e.g., `# managed-by: govern`) that the detection logic looks for to distinguish a govern-installed hook from a hand-rolled one. The same sentinel survives `/govern` updates because it's part of the shipped file.

`scripts/gen-spec-deps.sh` ships to adopters with `create` strategy — first `/govern` run installs it, subsequent runs leave it alone. Adopters can edit the generator without `/govern` clobbering. The shipped pre-commit hook references it via the project-relative path (`scripts/gen-spec-deps.sh`).

### Generated artifacts use marker comments

The two generated content blocks (`README.md` Feature Specs table, `framework/commands/help.md` command tables) are bounded by HTML marker comments:

```markdown
<!-- generated:feature-specs:start -->
| Spec | Status | Dependencies | Description |
| --- | --- | --- | --- |
| ... |
<!-- generated:feature-specs:end -->
```

The generator scripts find the markers, splice the regenerated content between them, and leave everything outside the markers untouched. This lets `README.md` retain hand-authored prose around the generated table.

For `framework/commands/help.md`, each of the five command tables (Pipeline, Elaborate, Brownfield, Orient, Bootstrap) gets its own marker pair. The generator emits one block per table from the appropriate command-source frontmatter.

### `/validate` schema and check changes

The frontmatter schema in `framework/constitution.md` §text-first-artifacts is updated to:

- **Spec files** (`spec.md`, `spec-and-plan.md`): required = `status`, `dependencies`. Removed = `tags` (deleted), `track` (deleted from `spec-and-plan.md`).
- **Scenario files** (`scenarios/{slug}.md`): required = `section`. Replaced from `spec-ref`. Removed = `tags`.
- **Other artifacts** (`plan.md`, `tasks.md`, `data-model.md`, `research.md`): no required frontmatter. The `title` field is deleted from all templates.

Validate check changes:

- PKM `title` advisory check: **removed**.
- `tags` advisory check: **removed**.
- `spec-ref` hard fail check: **renamed** to `section`.
- `--fix` mode: **removed entirely** (Fix Mode section deleted).
- Help-equivalence check: **changed from per-row description match to dry-run-of-generator check** — `/validate` runs `gen-help-tables.sh --dry-run` and reports if it would produce a diff.
- New advisory: when a body inline link to a sibling spec is found that is not yet in the generator-managed `dependencies` list, surface as `Body link to {slug} not in dependencies; the next commit will add it.` (Generator runs at commit, so this is informational — a hint that the frontmatter is currently stale.)

### Configuration rule file

Per spec Q5/Q6: a single file at `framework/rules/configuration.md` with `CFG-` prefix and `CONST`/`ENV` categories. Format `CFG-{CONST|ENV}-{NNN}`. Initial rule set:

- `CFG-CONST-001` — Shared constants live in a centralized location
- `CFG-CONST-002` — Module-local constants live in the module's own constants file
- `CFG-CONST-003` — Configurable values are not bare literals
- `CFG-ENV-001` — Every env var has a default constant
- `CFG-ENV-002` — `.env.example` contains every introduced var with a descriptive comment and safe placeholder
- `CFG-ENV-003` — Required env vars are validated at startup (fail fast with naming)
- `CFG-ENV-004` — Time-value env vars include unit suffix in name (`_MS`, `_SECONDS`, `_MINUTES`); the corresponding constant makes the unit explicit

Rule format details (Statement, Rationale, Verification, Source, ID stability) declared in `data-model.md` alongside this plan, mirroring the precedent set by `specs/008-security-rules/data-model.md`.

Adopter shipping: added to `framework/bootstrap/govern.md`'s Shared Files manifest under "govern-owned shared files (strategy: update)" — alongside the security rule files.

### Migration of existing dogfood specs

Done specs are frozen archaeology — stale `title:`, `tags:`, `spec-ref:`, and `track:` fields remain. `/validate` stops checking them, so they cause no findings.

The one exception that needs active migration: **inline links for dependencies**. Each existing spec's frontmatter `dependencies` list must be reflected in the spec body via inline markdown links — otherwise the first run of `gen-spec-deps.sh` strips them. Migration task: for each existing spec, scan the body for inline links to declared deps; add a "References" list at the bottom of the spec body for any declared dep not already inline-linked.

### CI safety net

A GitHub Actions workflow (`.github/workflows/generators.yml`) runs all four generators in dry-run mode on PR. Non-empty diff fails the build. Catches contributors who never ran `scripts/install-hooks.sh`.

A second workflow file (`.github/workflows/adopter-generators.yml`) ships as a template referenced from the adopter README — not auto-installed. Adopters who want CI enforcement copy it into their own `.github/workflows/`. Auto-shipping CI files via `/govern` would require platform detection (GitHub Actions vs. GitLab CI vs. Buildkite) that's out of scope for this spec.

## Affected Files

### Templates

| File | Action | Purpose |
| --- | --- | --- |
| `framework/templates/spec/spec.md` | Modify | Remove `title:` from frontmatter |
| `framework/templates/spec/spec-and-plan.md` | Modify | Remove `title:` and `track:` from frontmatter; remove track-related comment |
| `framework/templates/spec/plan.md` | Modify | Remove `title:` from frontmatter; remove "Open Questions Resolved" section |
| `framework/templates/spec/tasks.md` | Modify | Remove `title:` from frontmatter; remove `[simple]` marker documentation |
| `framework/templates/spec/data-model.md` | Modify | Remove `title:` from frontmatter |
| `framework/templates/spec/research.md` | Modify | Remove `title:` from frontmatter |
| `framework/templates/spec/scenario.md` | Modify | Replace `spec-ref:` with `section:`; remove `title:` from frontmatter |

### Commands

| File | Action | Purpose |
| --- | --- | --- |
| `framework/commands/specify.md` | Modify | Remove tag prompt step; remove title fill-in; remove tags-related instructions |
| `framework/commands/capture.md` | Modify | Remove title placeholder; remove tags reference |
| `framework/commands/clarify.md` | Modify | Remove title check; remove tags advisory; add cross-spec scan step; add deps recompute on entry |
| `framework/commands/plan.md` | Modify | Remove title fill-in; remove "Open Questions Resolved" reference; remove `[simple]` marker proposal step; reframe Affected Files as planning aid; add cross-spec scan; add deps recompute on entry |
| `framework/commands/implement.md` | Modify | Remove `[simple]` marker reading; replace Affected Files boundary with `git diff` derivation; add cross-spec scan; add deps recompute on entry |
| `framework/commands/elaborate.md` | Modify | Remove title placeholder; change `spec-ref` to `section`; add deps recompute on entry |
| `framework/commands/groom.md` | Modify | Remove `[promote-to-rule]` prefix instruction; specify "always re-walk every item not migrated on next pass" |
| `framework/commands/validate.md` | Modify | Remove Fix Mode section; remove PKM title check; remove tags advisory; rename `spec-ref` check to `section`; change help-equivalence to dry-run-of-generator check; add body-link-vs-deps advisory; add configuration rule file to rule-file list |
| `framework/commands/ask.md` | Modify | Add deps recompute on entry |
| `framework/commands/target.md` | Modify | Remove `tags` from frontmatter parse; add deps recompute on entry |
| `framework/commands/help.md` | Modify | Add HTML marker comments around the five command tables; tables become generator output |
| `framework/commands/log.md` | No change | — |
| `framework/commands/spawn.md` | No change | — |
| `framework/commands/status.md` | No change | — |

### Constitution

| File | Action | Purpose |
| --- | --- | --- |
| `framework/constitution.md` | Modify | Remove `tags` from frontmatter schema; remove "Starter Tag Vocabulary" table; remove `[simple]` marker bullet from §cost-levers; replace §constants and §env-vars with one-line pointer to `framework/rules/configuration.md` |
| `constitution.md` (root) | Delete | Collapse to single canonical at `framework/constitution.md` |

### Project root files

| File | Action | Purpose |
| --- | --- | --- |
| `CLAUDE.md` (root) | Modify | Update `@import constitution.md` → `@import framework/constitution.md` |
| `README.md` (root) | Modify | Update `constitution.md` link to `framework/constitution.md`; add HTML marker comments around Feature Specs table |
| `AGENTS.md` (root) | Modify | Remove "mirror constitutions" instruction; remove "run gen-claude-commands.sh" instruction |

### Generators (new)

| File | Action | Purpose |
| --- | --- | --- |
| `scripts/gen-readme-table.sh` | Create | Rebuild the README Feature Specs table from spec frontmatter |
| `scripts/gen-help-tables.sh` | Create | Rebuild the help.md command tables from each command's frontmatter |
| `scripts/gen-spec-deps.sh` | Create | Scan spec bodies for sibling-spec inline links; rewrite frontmatter `dependencies` |
| `scripts/install-hooks.sh` | Create | One-time `git config core.hooksPath .githooks`; idempotent |

### Hooks (new)

| File | Action | Purpose |
| --- | --- | --- |
| `.githooks/pre-commit` | Create | govern repo's hook — orchestrates all four generators, stages outputs |
| `framework/bootstrap/hooks/pre-commit` | Create | Adopter-shipped hook — calls only adopter-relevant generators (initially `gen-spec-deps.sh`); contains `# managed-by: govern` sentinel |
| `framework/bootstrap/hooks/install.sh` | Create | Adopter-side install script invoked by `/govern` |

### New rule file

| File | Action | Purpose |
| --- | --- | --- |
| `framework/rules/configuration.md` | Create | `CFG-CONST-NNN` and `CFG-ENV-NNN` rules with Statement / Rationale / Verification / Source per data-model |

### Bootstrap installer

| File | Action | Purpose |
| --- | --- | --- |
| `framework/bootstrap/govern.md` | Modify | Add Hook Installation section; add `framework/rules/configuration.md` to Shared Files (update strategy); add `framework/bootstrap/hooks/pre-commit` and `scripts/gen-spec-deps.sh` to Shared Files (create strategy for the script, update for the hook); add `framework/bootstrap/hooks/install.sh` to per-agent scaffolding logic |
| `framework/bootstrap/configure/claude.md` | Modify | Add Bash permissions for hook install/run paths (`Bash(git config *)`, `Bash(.githooks/*)`, `Bash(scripts/gen-*)`) |
| `framework/bootstrap/configure/auggie.md` | Modify | Same permission additions in Auggie's format |

### CI

| File | Action | Purpose |
| --- | --- | --- |
| `.github/workflows/generators.yml` | Create | govern repo CI: dry-run all four generators; fail on diff |
| `framework/templates/ci/adopter-generators.yml` | Create | Shipped template adopters can copy into their own `.github/workflows/` |

### Existing dogfood spec migration

| File | Action | Purpose |
| --- | --- | --- |
| `specs/000-016/spec.md` (and one `spec-and-plan.md` if any) | Modify | Add inline links in body for any declared frontmatter dependency not already linked (so first `gen-spec-deps.sh` run does not strip them) |

(No frontmatter migration on done specs — frozen archaeology.)

### This spec's own artifacts

| File | Action | Purpose |
| --- | --- | --- |
| `specs/017-derive-dont-ask/spec.md` | Modify (final task) | Strip `title:` and `tags:` from frontmatter |
| `specs/017-derive-dont-ask/plan.md` | Modify (final task) | Strip `title:` from frontmatter |
| `specs/017-derive-dont-ask/tasks.md` | Modify (final task) | Strip `title:` from frontmatter |
| `specs/017-derive-dont-ask/data-model.md` | Modify (final task) | Strip `title:` from frontmatter |

## Data Model

See `data-model.md` for the configuration rule file structure, ID format, category abbreviations, and the new spec/scenario frontmatter schema after deletions.

## Trade-offs

- **Symmetric hooks across govern and adopters.** Considered: command-entry recompute only, no adopter hook. Rejected because drift between commits is a real failure mode (a teammate pulls an out-of-date branch and reads stale deps without running a govern command). The user explicitly pushed for symmetry — if hooks are right for govern, they're right for adopters.
- **Adopter hook installs only when no existing hook system is detected.** Considered: always install (clobbering existing hooks). Rejected because adopter projects already use husky/lefthook/pre-commit-py and clobbering their setup is a pipeline-violating action. Skip-and-warn-with-snippet keeps the adopter in control.
- **No `gen-root-constitution.sh` generator.** Q2 collapsed the twin constitutions; the divergence the generator would have managed is gone.
- **Existing dogfood specs not migrated for deleted fields.** Done specs are frozen archaeology per `framework/constitution.md` §done-specs-are-frozen-archaeology. Stale `title:`, `tags:`, `spec-ref:`, `track:` fields remain; the open-schema rule ignores them. The cost is one-time visual noise in old specs; the benefit is no rewrite of merged history.
- **CI templates ship but aren't auto-installed for adopters.** Auto-installing requires CI-platform detection (GHA vs. GitLab vs. Buildkite) that's out of scope. Adopters with CI enforcement opt in by copying the template.
- **The pre-commit hook runs all generators unconditionally** rather than gating on which files changed. Trades sub-second hook execution time for a hook script that can't have wrong gate logic.
- **`--fix` mode deletion is irreversible.** A user who relied on `/validate --fix` for hand-implementation flows loses that workflow. The framework's position becomes "use `/implement`," and the trade-off is documented in the spec's Q1 resolution. Rolling back requires another spec.
