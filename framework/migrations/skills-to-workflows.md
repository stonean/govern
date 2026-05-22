# skills-to-workflows

**Introduced in:** gvrn 0.2.0
**Summary:** Remove legacy `{config_dir}/commands/{project}/skills/` directory left over from the `skills/` → `workflows/` rename.

## Procedure

The rename (introduced by spec 010, delivered alongside spec 005's reopen) moved every workflow file into a new `workflows/` directory. The old `skills/` tree is unreferenced and safe to remove on any adopter project that scaffolded prior to the rename.

1. **Idempotency check.** If `{config_dir}/commands/{project}/skills/` does not exist, exit silently.
2. **Pinned-files check.** If the directory exists and **is** listed in `.govern.toml` `pinned.files` (path comparison after placeholder resolution), leave it alone and report `pinned (kept): {config_dir}/commands/{project}/skills/` in the post-scaffolding summary. Exit without further action.
3. **Recursive delete.** Otherwise, recursively delete the directory.
4. **Summary line.** Report `removed (legacy skills/ directory): {config_dir}/commands/{project}/skills/` in the post-scaffolding summary.

The cleanup is unconditional once the directory is detected and unpinned — the new `workflows/` directory has already replaced it on every `/govern` run since the rename, so any remaining `skills/` tree is necessarily stale.
