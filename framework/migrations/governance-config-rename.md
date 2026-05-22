# governance-config-rename

**Introduced in:** gvrn 0.2.0
**Summary:** Rename `.governance.toml` → `.govern.toml` in the project root.

## Procedure

1. **Idempotency check.** If `.governance.toml` does not exist in the project root, exit silently. If both `.governance.toml` and `.govern.toml` exist, leave them alone and warn `Both .governance.toml and .govern.toml exist; remove the legacy file to silence this warning.` Then exit without modifying either file.
2. **Rename.** Rename `.governance.toml` to `.govern.toml` via `mv` (or `git mv` when the source is tracked).
3. **Summary line.** Report `migrated config: .governance.toml → .govern.toml` in the post-scaffolding output.
