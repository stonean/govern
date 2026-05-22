# gitignore-marker-rename

**Introduced in:** _set during 027.3 back-fill_
**Summary:** Replace `# Governance` marker in `.gitignore` with `# govern`.

## Procedure

1. **Idempotency check.** If the project's `.gitignore` does not contain a `# Governance` line, or already contains a `# govern` line, exit silently.
2. **Replace.** Replace the first occurrence of `# Governance` with `# govern` in `.gitignore`. The marker check used by the `.gitignore` merge step elsewhere in the bootstrap uses the new spelling, so this rename keeps idempotency intact for that step.
3. **Summary line.** Report `migrated .gitignore marker: # Governance → # govern` in the post-scaffolding output.
