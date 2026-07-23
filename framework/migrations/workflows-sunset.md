# workflows-sunset

**Introduced in:** gvrn 0.23.0
**Summary:** Remove the scaffolded workflow slash commands, the synced `workflows/registry.json`, and the `[workflows]` config section — the workflows feature was removed from the framework (spec 043).

## Procedure

Spec 043 removed the workflows feature: `/govern` no longer offers or scaffolds tech-stack workflow commands, and nothing reads the `[workflows]` config section. This migration cleans up what earlier `/govern` runs scaffolded. It subsumes the retired `skills-to-workflows` and `workflow-filename-rename` migrations (see `CHANGELOG.md` § Archived migrations), so it also covers adopters carrying pre-rename debris.

Every deletion below honors `.govern/config.toml` `[pinned] files` (path comparison after placeholder resolution): a pinned path is left alone and reported as `pinned (kept): {path}` in the post-scaffolding summary. Adopters who customized a scaffolded file and want to keep it can pin its destination path.

1. **Idempotency check.** If none of the targets below exist — no `{config_dir}/commands/{project}/workflows/` directory, no `{config_dir}/commands/{project}/skills/` directory, no `workflows/registry.json` at the project root, and no `[workflows]` section in the active config file — exit silently.

2. **Exact-set workflow-file deletion.** In `{config_dir}/commands/{project}/workflows/`, delete each unpinned file whose name appears in this exact set (13 current template names plus the 9 legacy `{category}-{language}-{tool}.md` names):

   - `black.md`, `eslint.md`, `gofmt.md`, `golangci-lint.md`, `gotest.md`, `prettier.md`, `pytest.md`, `rails-migrate.md`, `rspec.md`, `rubocop.md`, `ruff.md`, `rufo.md`, `vitest.md`
   - `format-go-gofmt.md`, `format-python-black.md`, `format-typescript-prettier.md`, `lint-go-golangci-lint.md`, `lint-python-ruff.md`, `lint-typescript-eslint.md`, `test-go-gotest.md`, `test-python-pytest.md`, `test-typescript-vitest.md`

   The check is by exact filename match; adopter-authored files (e.g., `pytest-fast.md`) are never affected because they aren't in the set. Report each deletion as `removed (workflow): {filename}`. After the pass, remove the `workflows/` directory itself only if it is empty — a directory kept alive by custom files is left in place.

3. **Legacy `skills/` removal.** If `{config_dir}/commands/{project}/skills/` exists and is not pinned, recursively delete it and report `removed (legacy skills/ directory): {config_dir}/commands/{project}/skills/`. Any remaining `skills/` tree predates the 0.2.0 rename and is necessarily stale.

4. **Synced registry removal.** If `workflows/registry.json` exists at the project root and is not pinned, delete it and report `removed (workflow registry): workflows/registry.json`. Remove the root `workflows/` directory if that leaves it empty.

5. **`[workflows]` config-section removal.** In the **active config file** (`.govern/config.toml` when it exists, else the legacy root `.govern.toml` — the standard write policy), if a `[workflows]` section exists, remove it: the `[workflows]` header line, every key under it (`declined_categories`), and the comment lines attached directly above the header or its keys, normalizing to a single blank line between the surrounding sections. Preserve every other table, comment, and formatting byte-for-byte. Report `removed (config): [workflows] section`. If no `[workflows]` section exists, skip silently.

6. **Summary.** The lines reported above appear in the post-scaffolding summary. The migration runs once per adopter (tracked via `[migrations].last_applied`); every step is a no-op when its target is absent, so a re-run is silent.
