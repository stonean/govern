# session-file-consolidate

**Introduced in:** gvrn 0.10.0
**Summary:** Consolidate the session state file from `{config_dir}/{project}-session.json` (host- and project-name-specific) onto `.govern.session.toml` at the repo root.

## Background

Pre-0.10.0 the session lived at `{cli-config-dir}/{project}-session.json`, e.g. `.claude/gov-session.json` for an adopter using Claude Code with project `gov`, `.claude/anvil-session.json` for one using Claude with `anvil`, or `.augment/anvil-session.json` for the same adopter on Auggie. The path baked in the AI CLI's config directory and the adopter's project name, which broke whenever those didn't match the runtime's hardcoded constant (`.claude/gov-session.json`).

The consolidation moves the file to `.govern.session.toml` at the repo root: gitignored, host-agnostic, project-name-agnostic, and uniform across every adopter. TOML replaces JSON to align with `.govern.toml`'s on-disk format. Keys are kebab-case (`scenario-path`, `set-at`) rather than the legacy camelCase (`scenarioPath`, `setAt`).

## Procedure

1. **Idempotency check.** If no `{config_dir}/{project}-session.json` exists for any selected agent (i.e., no legacy session file is present in the project), exit silently — adopters who never used `/gov:target` or who already migrated have nothing to do.

2. **Locate the legacy file.** Iterate every selected agent's `config_dir`. The legacy filename is `{config_dir}/{project}-session.json` after the bootstrap's placeholder substitution. The most-recently-modified existing legacy file wins; ties on mtime fall back to lexicographic order on the path. Other legacy files are deleted in step 4.

3. **Translate to `.govern.session.toml`.** If `.govern.session.toml` already exists at the repo root, leave it alone — the user has already targeted post-consolidation and the new file is authoritative. Otherwise, parse the legacy JSON and write `.govern.session.toml` with these key renames:

   - `feature` → `feature` (unchanged)
   - `path` → `path` (unchanged)
   - `scenario` → `scenario` (unchanged)
   - `scenarioPath` → `scenario-path`
   - `setAt` → `set-at`

   Any other top-level keys present in the legacy JSON (e.g., walker-context-seed fields from non-standard adopter usage) are preserved as top-level TOML keys with the same name. Use `toml::to_string` for deterministic output; finish with the standard tempfile + rename atomic-write pattern.

4. **Delete the legacy file(s).** After `.govern.session.toml` is written (or already existed), delete every `{config_dir}/{project}-session.json` for every selected agent. Use `git rm` when the file is tracked, plain `rm` otherwise.

5. **Summary line.** Report `migrated session state: {legacy-paths-joined-with-commas} → .govern.session.toml` in the post-scaffolding output. When `.govern.session.toml` was preserved as already-current, append `(merged into existing .govern.session.toml? no — the existing file was kept)`.

## Notes

- The migration is one-way. There is no reverse path; adopters who pin to a pre-0.10.0 framework version and re-run a newer `/govern` will not have a legacy file recreated.
- The migration does not regenerate the gitignore entry for `.govern.session.toml` — that ships via the framework-managed gitignore block which the `apply-manifest` step rewrites on every `/govern` run.
