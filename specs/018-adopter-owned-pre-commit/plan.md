# 018 — Adopter-Owned Pre-Commit Plan

Implements [018 — Adopter-Owned Pre-Commit](spec.md).

## Overview

Three on-disk changes to `framework/bootstrap/hooks/`, one substantive rewrite of `framework/bootstrap/govern.md` §Hook Installation + §Shared Files, and a signpost block on spec 017. The work is plumbing — no new behavior the framework didn't already do, just a different ownership model for the hook file. Risk surface is the migration code path (one-time, recovery is documented in §Edge Cases). No data model, no event contracts, no error codes.

## Technical Decisions

### Rename, don't recreate, the inner file

The existing `framework/bootstrap/hooks/pre-commit` already contains exactly the body the new `govern-pre-commit` needs (sentinel on line 2, `gen-spec-deps.sh` invocation, `git add` staging). The implementation does `git mv framework/bootstrap/hooks/pre-commit framework/bootstrap/hooks/govern-pre-commit` rather than a copy + delete. This preserves git blame on the file and keeps the diff small (rename detection in `git log --follow`).

### Outer stub is a fresh file

The new `framework/bootstrap/hooks/pre-commit` is written from scratch as the adopter-owned outer stub. Content per spec §Design > Outer file. This file replaces the renamed one at the same source path; from git's perspective, it's a delete+add (because content differs significantly from the renamed file). Acceptable — the file is fundamentally a different artifact under the new model, and adopter projects don't have history on it anyway.

### govern.md §Hook Installation rewrite

Three substantive edits to the section, each motivated by the spec:

1. **Detection ladder collapses from 7 items to 4.** Items 2–5 (third-party hook systems) merge into a single "any third-party hook system detected" branch with the same skip-and-warn behavior. The old item 6 (sentinel-detected govern-installed file) is removed entirely — under the new model, `.githooks/pre-commit` is never govern-managed, so detecting it as such is meaningless. The old items 1 and 7 collapse to the new items 1 and 4 (already-wired vs. fresh-install).

2. **Migration subsection added.** New subsection between §Hook Installation's detection ladder and §Manual integration snippet. Specifies the line-2 sentinel check on `.githooks/pre-commit`, the conditional `git mv` (tracked vs. untracked file), the post-rename manifest behavior, and the post-scaffolding summary line. Edge cases from spec §Edge Cases (pre-existing inner, `git mv` failure) get explicit handling steps.

3. **Inline `core.hooksPath` + `chmod +x`.** The actions previously delegated to `framework/bootstrap/hooks/install.sh` move into the §Hook Installation section's fresh-install path: `git config core.hooksPath .githooks` and `chmod +x .githooks/pre-commit .githooks/govern-pre-commit`. Both are idempotent on re-runs.

### Manual integration snippet path change

The snippet (printed when the detection ladder hits a third-party hook system or non-`.githooks` `core.hooksPath`) changes from `./.githooks/pre-commit` to `./.githooks/govern-pre-commit`. Two callsites: the prose snippet in §Hook Installation > Manual integration snippet, and the post-scaffolding output's per-condition skip message. Both update.

### Spec 017 signpost

Inserted as a top-of-file block-quote, after the H1 and before the lead paragraph. Markdown form:

```markdown
> **Signpost (post-018):** The adopter pre-commit hook design described in
> this spec was superseded by [018-adopter-owned-pre-commit](../018-adopter-owned-pre-commit/spec.md).
> The shipped file at `framework/bootstrap/hooks/pre-commit` is now an
> adopter-owned outer stub written via `create` strategy, not a govern-owned
> hook updated via `update` strategy. The govern-owned generator orchestration
> moved to `framework/bootstrap/hooks/govern-pre-commit`. `install.sh` is
> deleted; install actions inline in `govern.md` §Hook Installation. AC21–AC23
> reflect the pre-018 design.
```

The 017 body, ACs, and resolved-question entries are not edited. The block-quote form makes the signpost visually distinct from 017's normal content.

### Verification approach

The end-to-end ACs (AC8, AC9, AC11) require a real `/govern` run against a sandbox project to verify migration and fresh-install paths. The verification is manual — run `/govern` against a temp directory in two configurations (existing-install with sentinel'd file; clean repo with no hooks) and inspect resulting file layouts. The govern repo's own CI does not exercise this code path (CI runs generators in dry-run; it does not boot a fresh adopter project). Manual sandbox verification is the discharge criterion for those ACs.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/bootstrap/hooks/pre-commit` | Rename → `govern-pre-commit`, then re-create at original path with new content | Old contents become the inner file (govern-owned); new contents at the same path become the outer stub (adopter-owned) |
| `framework/bootstrap/hooks/govern-pre-commit` | Create (via rename) | Govern-owned generator orchestration; sentinel on line 2 preserved |
| `framework/bootstrap/hooks/install.sh` | Delete | Replaced by inlined actions in `govern.md` §Hook Installation |
| `framework/bootstrap/govern.md` | Modify | §Shared Files manifest (split 1 row → 2); §Hook Installation (rewrite ladder, add Migration subsection, inline `core.hooksPath` + `chmod`); update Manual integration snippet path |
| `specs/017-derive-dont-ask/spec.md` | Modify (signpost only) | Insert block-quote signpost after the H1; do not touch body, ACs, or resolved questions |
| `specs/018-adopter-owned-pre-commit/plan.md` | Create | This file |
| `specs/018-adopter-owned-pre-commit/tasks.md` | Create | Task breakdown |

The CI workflow at `.github/workflows/` (if present) is not touched — generators run in dry-run mode and the rename is byte-identical for the inner file, so CI sees no diff.

## Trade-offs

- **Two files for adopters to inspect.** When debugging a failed pre-commit, the adopter has to know the wiring goes outer → inner. Mitigated by the comment blocks on each file naming the ownership boundary, and by the manual-integration snippet pointing at the inner file as the public entry point.
- **Migration is a one-time code path with edge-case branches.** §Edge Cases enumerates four recovery paths the migration code must handle. The complexity exists for one purpose (carrying spec-017 adopters forward); it'll be ambient drag in govern.md until it's deleted in some far-future cleanup. Acceptable price — the alternative is breaking 017's adopters or asking them to do the migration manually.
- **`install.sh` deletion drops a (theoretical) manual reinstall path.** Adopters who deleted their hooks and don't want to re-run `/govern` would have lost a single command (`bash framework/bootstrap/hooks/install.sh`). The replacement is two commands (`git config core.hooksPath .githooks; chmod +x .githooks/*`). This is fine — the path was never used in practice (verifiable by grep through any adopter project's history; nobody re-runs `install.sh` standalone).
- **The outer file's `set -euo pipefail` propagates the inner file's failures unchanged.** A failing generator in the inner file still aborts the commit, which is the desired behavior. The trade-off this avoids: trying to make the outer file "smart" about partial failures (e.g., letting the adopter's checks run even when govern's checks fail). Out of scope and would invert the ownership model.

## Cross-spec context

- **Dependency 017 is `done`.** No further reads needed beyond what was already loaded during clarify. The spec body links 017 inline; the plan does too. No other specs are referenced.
- **No event contracts.** The hook fires on `git commit`, but git's hook protocol is not an event in `events.md` terms.
- **No error codes.** Failures surface via shell exit codes; no application-level error codes are introduced.
- **No data model.** A file rename + a file-layout decision; no domain entities.
