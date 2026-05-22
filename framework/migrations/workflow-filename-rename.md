# workflow-filename-rename

**Introduced in:** _set during 027.3 back-fill_
**Summary:** Remove legacy `{category}-{language}-{tool}.md` workflow files left over after the post-005 filename simplification to `{tool}.md`.

## Procedure

Before reading the workflow registry, remove any workflow files left behind by `/govern` runs prior to the post-005 filename rename (which simplified `{category}-{language}-{tool}.md` to `{tool}.md`).

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
2. **Per-file pinned check.** For each existing match, skip it if listed in `.govern.toml` `pinned.files`. Adopters who customized a legacy file and want to keep it can pin its destination path.
3. **Delete.** Delete each remaining match.
4. **Summary line.** For each deletion, report `removed (legacy workflow): {filename}` in the post-scaffolding summary. Omit the line when nothing was deleted.

The check is by exact filename match against the set above; custom user files (e.g., `pytest-fast.md`) are never affected because they aren't in the set. The cleanup runs every `/govern` invocation; once the legacy files are gone, subsequent runs are silent no-ops for this step.
