---
status: in-progress
dependencies: [017-derive-dont-ask]
---

# 018 — Adopter-Owned Pre-Commit

Split the adopter pre-commit hook into two files so `/govern` can keep its generators in sync without ever overwriting code the adopter added to their own pre-commit hook.

## Problem

Spec 017 shipped `framework/bootstrap/hooks/pre-commit` to adopters at `.githooks/pre-commit` with `update` strategy and a `# managed-by: govern` sentinel on line 2. On every `/govern` run, govern overwrites that file in full whenever the sentinel is present (see [017-derive-dont-ask](../017-derive-dont-ask/spec.md) §Generators and Hooks; `framework/bootstrap/govern.md` §Hook Installation, item 6).

The hook is the natural place for adopters to wire in their own pre-commit logic — lint, type-check, test, format, license-header check. Today, any line an adopter adds to `.githooks/pre-commit` is silently destroyed by the next `/govern`. The framework can't tell adopter-added lines from its own and treats the whole file as govern-owned.

The fix: govern should not own `.githooks/pre-commit` at all. It owns a separate inner file the outer hook invokes. The outer hook is created on first run and never overwritten thereafter — adopters edit it freely.

## Design

### File layout

| Path | Owner | Strategy | Contents |
| --- | --- | --- | --- |
| `.githooks/pre-commit` | Adopter | `create` (first run only) | Stub script that invokes the inner hook; adopter adds their own lines around the invocation |
| `.githooks/govern-pre-commit` | govern | `update` | The generator orchestration that ships with the framework (currently `gen-spec-deps.sh` + staging) |

`/govern` only ever overwrites the inner file. The outer file is created on first run and skipped on every subsequent run, like every other `create`-strategy file in the manifest.

### Outer file (initial content)

```bash
#!/usr/bin/env bash
# Project pre-commit hook. Edit this file freely — /govern does not overwrite it
# after the initial install. Add your project's pre-commit checks (lint, tests,
# format, etc.) above or below the govern hook invocation below.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

# Keeps govern's generated artifacts (spec deps, etc.) in sync. /govern updates
# this script on every run; do not edit it.
./.githooks/govern-pre-commit
```

No `# managed-by: govern` sentinel — this file is not managed by govern after creation. The two comment blocks signal the ownership boundaries (this file is adopter-owned; the inner file is govern-owned) so an adopter opening it for the first time knows where edits will persist.

### Inner file (govern-owned)

The inner file's body is what `framework/bootstrap/hooks/pre-commit` ships today: run the adopter-relevant generators, then `git add` their outputs. The `# managed-by: govern` sentinel stays on line 2 of the inner file as the marker that govern owns it.

### Hook Installation logic

The detection ladder in `framework/bootstrap/govern.md` §Hook Installation simplifies. The new ladder, replacing the current items 1–7:

1. **`core.hooksPath` already points at `.githooks`** — already wired. Run the manifest passes for the inner file (`update`) and the outer file (`create`), and continue.
2. **`core.hooksPath` points at any other path** — custom hooks dir. Skip wiring; report a warning with the manual integration snippet (see §Manual integration below).
3. **`.husky/`, `.pre-commit-config.yaml`, `lefthook.yml`, or `lefthook-local.yml` exist** — third-party hook system. Skip wiring; report a warning with the manual integration snippet.
4. **No conflicts** — run `git config core.hooksPath .githooks` and let the manifest passes write the two files. Report `pre-commit hook installed`.

The previous "existing `.githooks/pre-commit` from a prior `/govern` run, detected by sentinel" branch goes away — the outer file is no longer detected by sentinel because govern doesn't own it. Migration of pre-existing govern-installed hooks is handled separately (see §Migration below).

### Manual integration snippet

When detection skips wiring, the message becomes:

> The `govern` pre-commit hook was not wired up because your project already uses an existing hook system. To get automatic spec-deps regeneration on every commit, add this line to your existing pre-commit chain:
>
> ```bash
> ./.githooks/govern-pre-commit
> ```
>
> The shipped hook script is idempotent and safe to call from another hook runner.

The path changes from `./.githooks/pre-commit` to `./.githooks/govern-pre-commit`.

### Pinning

`.govern.toml` `pinned.files` continues to apply. Pinning the inner file (`.githooks/govern-pre-commit`) freezes it at the pinned version — `/govern` will not overwrite. Pinning the outer file is a no-op (it's already `create`-strategy and never overwritten after first run); listing it in `pinned.files` is harmless.

## Migration

Existing adopters who already ran `/govern` from spec 017 have a govern-owned `.githooks/pre-commit` with the `# managed-by: govern` sentinel on line 2. On the first `/govern` run after this lands:

1. Detect the sentinel on line 2 of `.githooks/pre-commit`.
2. Rename the file: `git mv .githooks/pre-commit .githooks/govern-pre-commit` if the file is tracked, otherwise plain `mv`.
3. Apply the `update` strategy to the renamed file with the new shipped contents (which are byte-identical to the pre-rename contents for adopters who never edited theirs — so the rename is the only on-disk change for the common case).
4. Apply the `create` strategy for the new outer `.githooks/pre-commit`, writing the stub above. Because the old file has been renamed, the destination no longer exists and `create` proceeds.
5. Report `migrated pre-commit hook: .githooks/pre-commit → .githooks/govern-pre-commit; created adopter-owned .githooks/pre-commit stub`.

If `.githooks/pre-commit` exists but does **not** carry the sentinel, leave it alone — it's an adopter file, follow the existing detection ladder (skip wiring, manual integration snippet). The new layout still installs `.githooks/govern-pre-commit` via the manifest in this case (it's the inner file, useful even when not wired).

Adopters who edited their govern-installed pre-commit despite the sentinel: their edits live in the renamed file (now `govern-pre-commit`). The next `/govern` will overwrite that file with the shipped version, dropping their edits. This is the same fate those edits had under the prior design — the migration does not make things worse, and the post-migration model gives adopters a safe place (the new outer file) to put edits going forward.

## Affected Surfaces

- `framework/bootstrap/hooks/pre-commit` — split into two files. The current contents become `framework/bootstrap/hooks/govern-pre-commit`. A new `framework/bootstrap/hooks/pre-commit-stub` (or similar) holds the adopter-owned outer file's initial content.
- `framework/bootstrap/hooks/install.sh` — deleted. The two install actions (`git config core.hooksPath .githooks` and `chmod +x` on both hook files) are inlined into `framework/bootstrap/govern.md` §Hook Installation. The conflict-detection logic the script duplicated already lives in `/govern`'s detection ladder, and a manual `bash install.sh` would not regenerate the outer file (only `/govern` writes it via the manifest), so the satellite script's only remaining role goes away.
- `framework/bootstrap/govern.md` §Hook Installation — rewrite the detection ladder per §Design above; update the manual integration snippet path; add the migration step.
- `framework/bootstrap/govern.md` §Shared Files — replace the single `framework/bootstrap/hooks/pre-commit` → `.githooks/pre-commit` (`update`) row with two rows: inner file (`update`) and outer file (`create`).
- govern's own `.githooks/pre-commit` — unaffected. This is the dogfood repo's hook; it lives under a different ownership model (govern's own repo, edited by maintainers).
- CI safety net (017 AC24) — unaffected. The dry-run check still exercises the same generators; the indirection through the new outer file does not change what runs.

## Edge Cases

- **Pre-existing `.githooks/govern-pre-commit` blocks the migration rename.** A partial prior migration or an adopter's hand-created file at the inner path can pre-occupy the destination of `git mv .githooks/pre-commit .githooks/govern-pre-commit`. The migration aborts the rename, reports `migration skipped: .githooks/govern-pre-commit already exists; resolve manually`, and proceeds with the manifest passes regardless. The `update`-strategy pass overwrites the pre-existing inner file with the shipped contents; the old `.githooks/pre-commit` (still carrying the sentinel) is left in place but the new §Hook Installation ladder no longer treats sentinels in the outer file as govern-managed, so it is treated as adopter-owned going forward. The adopter resolves the duplicate manually.

- **`core.hooksPath` is unset during migration.** Possible when an adopter manually edited git config or never ran `git config core.hooksPath .githooks` (e.g., 017's `install.sh` was skipped). The migration path runs `git config core.hooksPath .githooks` after the rename — idempotent if already pointing at `.githooks`, corrective if unset, and the existing detection ladder handles the conflict case (`core.hooksPath` pointing elsewhere) before the migration ever fires.

- **Executable bits on the new files.** `update` and `create` strategies write file content but do not chmod. After the manifest passes complete, §Hook Installation runs `chmod +x .githooks/pre-commit .githooks/govern-pre-commit` once. Subsequent `/govern` runs re-chmod (idempotent) — covers the case where an adopter's git config or worktree configuration dropped the executable bit.

- **Outer file deleted, inner file remains.** Adopter or tooling removed `.githooks/pre-commit` while leaving `.githooks/govern-pre-commit` in place. With `core.hooksPath` still set, git silently skips the missing pre-commit hook on every commit and generators stop running. The next `/govern` run's `create`-strategy pass detects the missing destination and rewrites the outer stub, restoring the wiring. No new code path needed; this falls out of `create` strategy's normal "skip if present, write if missing" semantics.

- **Migration rename fails partway through.** If `git mv` succeeds but the run aborts before the manifest passes (network failure during a later step, user `^C`, etc.), the legacy file is now at `.githooks/govern-pre-commit` and `.githooks/pre-commit` does not exist. The next `/govern` run's detection ladder item 1 fires (`core.hooksPath` already points at `.githooks`), the `update` pass is a no-op for the inner file (content matches), and the `create` pass writes the new outer stub — self-healing without operator intervention. If `git mv` itself fails (permissions, repo locked, file in use), the migration aborts the rename, reports `migration failed: could not rename .githooks/pre-commit; resolve manually` and continues with the manifest passes. The `update` strategy still installs `.githooks/govern-pre-commit` (writing it from scratch since the destination doesn't exist); the `create` strategy sees `.githooks/pre-commit` still in place and skips. Adopter ends up with both files: legacy sentinel'd outer (still functional) and new govern-owned inner (idle until the outer is updated to call it). Adopter completes the migration manually by editing the outer to call `./.githooks/govern-pre-commit`.

## Acceptance Criteria

- [ ] AC1: `framework/bootstrap/hooks/govern-pre-commit` ships with the framework, contains the `# managed-by: govern` sentinel on line 2, and runs the adopter-relevant generators (currently `scripts/gen-spec-deps.sh`) plus the existing `git add` staging
- [ ] AC2: A second shipped file (`framework/bootstrap/hooks/pre-commit`, replacing the file currently at that path) holds the initial content for the adopter-owned outer hook: invokes `./.githooks/govern-pre-commit` and contains no `# managed-by: govern` sentinel
- [ ] AC3: `framework/bootstrap/govern.md` §Shared Files manifest lists `framework/bootstrap/hooks/govern-pre-commit` → `.githooks/govern-pre-commit` with `update` strategy
- [ ] AC4: `framework/bootstrap/govern.md` §Shared Files manifest lists the new outer-hook source → `.githooks/pre-commit` with `create` strategy
- [ ] AC5: `framework/bootstrap/govern.md` §Hook Installation detection ladder is rewritten to the four-item form in §Design above; the "existing `.githooks/pre-commit` from a prior `/govern` run, detected by sentinel" branch is removed
- [ ] AC6: `framework/bootstrap/govern.md` §Hook Installation manual integration snippet references `./.githooks/govern-pre-commit`, not `./.githooks/pre-commit`
- [ ] AC7: `framework/bootstrap/govern.md` includes a migration subsection: when `.githooks/pre-commit` exists with the `# managed-by: govern` sentinel on line 2 and no `.githooks/govern-pre-commit` exists, rename the file and apply the manifest passes; report the migration in the post-scaffolding summary
- [ ] AC8: A `/govern` run on an adopter project that previously installed the spec-017 hook produces: a renamed inner file at `.githooks/govern-pre-commit` and a fresh outer `.githooks/pre-commit` stub; subsequent `/govern` runs leave the outer file untouched even if the adopter added lines to it
- [ ] AC9: A `/govern` run on a fresh project (no existing hook) produces both files with executable bits set, sets `core.hooksPath .githooks`, and leaves only the inner file (`.githooks/govern-pre-commit`) carrying the `# managed-by: govern` sentinel
- [ ] AC10: Spec 017 carries a signpost block immediately after its H1 (before the lead paragraph) pointing at 018, naming the superseded surfaces (the adopter pre-commit ownership model; `framework/bootstrap/hooks/pre-commit` strategy; the new `govern-pre-commit` inner file; the `install.sh` deletion; AC21–AC23). The 017 body and ACs are not edited beyond the inserted signpost block, per the constitution's frozen-archaeology rule
- [ ] AC11: `/govern` end-to-end run executed against a sandbox adopter directory (existing-install case and fresh-install case) produces the file layouts described in AC8 and AC9 with no manual intervention
- [ ] AC12: `framework/bootstrap/hooks/install.sh` is deleted; its install actions (`git config core.hooksPath .githooks` and `chmod +x` on the two hook files) are inlined into `framework/bootstrap/govern.md` §Hook Installation; no other artifact references the deleted file

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Q1 (inner file name):** `.githooks/govern-pre-commit`. The `govern-` prefix matches existing convention (`.govern.toml`, `# managed-by: govern`) and reads naturally as "the govern hook called by pre-commit." The alternative `.githooks/pre-commit-govern` would sort adjacent to `pre-commit` in `ls`, but the `.githooks/` directory rarely holds enough files for sort order to matter and naming for convention beats naming for sort.
- **Q2 (outer file initial content):** Keep the friendly version with two comment blocks. The hook fires for every contributor on every commit; anyone opening the file should be able to tell immediately that their edits persist and where to add steps. Since `create` strategy writes the file once, the "noise after customization" cost is bounded — adopters can delete the comments if they're noisy, or keep them since the boundaries they describe stay accurate. Concretely, the initial content has (1) a top comment explaining the file is adopter-owned and where to add checks, (2) the `./.githooks/govern-pre-commit` invocation prefixed by a comment explaining that file is govern-owned. No decorative comments beyond those two blocks.
- **Q3 (`install.sh` retention):** Delete `framework/bootstrap/hooks/install.sh`. The script had a useful semantic role under the spec-017 design (a single point that "installed" the hook by setting `core.hooksPath` and chmodding the file), but under the split-file design that role collapses: install is no longer separable from the manifest pass because the outer file is `create`-strategy and only `/govern` writes it via the manifest. A manual `bash install.sh` from a fresh clone could not regenerate the outer file. The two install actions (`git config core.hooksPath .githooks` and `chmod +x .githooks/pre-commit .githooks/govern-pre-commit`) are inlined into `framework/bootstrap/govern.md` §Hook Installation. The conflict-detection logic the script duplicated already lives in `/govern`'s detection ladder before invocation, so nothing is lost.
- **Q4 (migration detection scope):** Strict line-2 sentinel check. The shipped 017 hook always puts `# managed-by: govern` on line 2; an unmodified adopter file matches. Adopters who edited the file enough to push the sentinel off line 2 (added shebang flags, prepended custom comments, etc.) are treated as adopter-owned: the migration does not auto-rename, the manual integration warning fires, and the new `.githooks/govern-pre-commit` is still installed via the manifest. Bias is toward false negatives — they cost a one-time warning and manual step; false positives (auto-renaming a customized file) destroy work and are the exact case this spec exists to prevent.
- **Q5 (017 signpost):** A single signpost block at the top of `specs/017-derive-dont-ask/spec.md`, inserted immediately after the H1 and before the lead paragraph. The block links to 018, names the superseded surfaces (the adopter pre-commit ownership model; `framework/bootstrap/hooks/pre-commit` is now adopter-owned `create`-strategy, not govern-owned `update`-strategy; the new `framework/bootstrap/hooks/govern-pre-commit` holds the govern-owned generator orchestration; `install.sh` is deleted; 017's AC21–AC23 reflect the pre-018 design). The 017 body, ACs, and resolved-question entries are not edited beyond the inserted signpost block, per the constitution's frozen-archaeology rule.
- **Q6 (post-migration follow-on):** No CI grace pass needed. govern's own repo `.githooks/pre-commit` is the maintainer file at the govern repo root, untouched by this spec; CI generators don't span the hook layout. Adopter projects: the migration is a `/govern`-triggered commit event, not a CI event. Adopters who pull the new govern and run CI before re-running `/govern` keep building green against the old file (which still works). On the `/govern` re-run, `git mv` renames the inner file (byte-identical contents for unmodified adopters) and `create` writes the new outer stub — both are explicit changes, not silent CI failures. Generator dry-runs in CI only watch generator outputs, which don't differ across the rename.
