# CHANGELOG

Adopter-facing record of framework conventions that have been removed.
Each entry is a recipe for adopters who skipped past the active sunset
window and need to apply the removal manually. See spec
[027 — Bootstrap Migration Registry](specs/027-bootstrap-migration-registry/spec.md)
for the governing pipeline.

The runtime crate's own release notes live at [`runtime/CHANGELOG.md`](runtime/CHANGELOG.md).

## Archived migrations

Entries here left `framework/migrations.toml` after their sunset window closed. An adopter far enough behind to still need one applies the recipe by hand. Both entries below are additionally **subsumed** by the active `workflows-sunset` migration (spec 043): a current `/govern` run cleans their targets automatically, so the recipes matter only for manual application outside `/govern`.

### skills-to-workflows (introduced 0.2.0, sunset after 0.10.0)

**Summary:** Remove legacy `{config_dir}/commands/{project}/skills/` directory left over from the `skills/` → `workflows/` rename.

The rename (introduced by spec 010, delivered alongside spec 005's reopen) moved every workflow file into a new `workflows/` directory. The old `skills/` tree is unreferenced and safe to remove on any adopter project that scaffolded prior to the rename.

1. **Idempotency check.** If `{config_dir}/commands/{project}/skills/` does not exist, exit silently.
2. **Pinned-files check.** If the directory exists and **is** listed in the active config file's `pinned.files` (path comparison after placeholder resolution), leave it alone and report `pinned (kept): {config_dir}/commands/{project}/skills/` in the post-scaffolding summary. Exit without further action.
3. **Recursive delete.** Otherwise, recursively delete the directory.
4. **Summary line.** Report `removed (legacy skills/ directory): {config_dir}/commands/{project}/skills/` in the post-scaffolding summary.

The cleanup is unconditional once the directory is detected and unpinned — the `workflows/` directory replaced it on every `/govern` run after the rename, so any remaining `skills/` tree is necessarily stale.

### workflow-filename-rename (introduced 0.2.0, sunset after 0.10.0)

**Summary:** Remove legacy `{category}-{language}-{tool}.md` workflow files left over after the post-005 filename simplification to `{tool}.md`.

In `{config_dir}/commands/{project}/workflows/`, delete any file whose name appears in this exact set:

- `format-go-gofmt.md`
- `format-python-black.md`
- `format-typescript-prettier.md`
- `lint-go-golangci-lint.md`
- `lint-python-ruff.md`
- `lint-typescript-eslint.md`
- `test-go-gotest.md`
- `test-python-pytest.md`
- `test-typescript-vitest.md`

1. **Idempotency check.** For each filename in the set above, check whether the file exists in `{config_dir}/commands/{project}/workflows/`. If none of the files exist, exit silently.
2. **Per-file pinned check.** For each existing match, skip it if listed in the active config file's `pinned.files`. Adopters who customized a legacy file and want to keep it can pin its destination path.
3. **Delete.** Delete each remaining match.
4. **Summary line.** For each deletion, report `removed (legacy workflow): {filename}` in the post-scaffolding summary. Omit the line when nothing was deleted.

The check is by exact filename match against the set above; custom user files (e.g., `pytest-fast.md`) are never affected because they aren't in the set.
