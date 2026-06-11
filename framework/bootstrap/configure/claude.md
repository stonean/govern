---
description: Configure settings.local.json with permissions for slash commands.
---

# Configure

Configure `{cli-config-dir}/settings.local.json` with the permissions needed for slash commands to run without manual approval.

## Scope Boundaries

- Read and write only `{cli-config-dir}/settings.local.json`. Do NOT modify any other file.
- Add missing entries and remove exact-match duplicates from `permissions.allow` and `permissions.deny`; do NOT reorder or rewrite non-duplicate entries the user (or another command) added beyond the canonical set listed below. The `merge-permissions` primitive performs the canonical-presence + dedup passes automatically; only `additionalDirectories` is handled outside the primitive (it has no duplication problem — entries are presence-checked, not deduped).
- Do NOT scan source code, specs, or git history. This command only manages permissions.
- Reference: no constitution sections apply — this command operates on agent-specific permission state, not `govern` artifacts.

## Instructions

1. Invoke `merge-permissions` (MCP: `merge-permissions`) to install the canonical `permissions.allow` and `permissions.deny` sets into `{cli-config-dir}/settings.local.json` and dedup exact-match entries from both arrays. The primitive creates the file if missing (with `{"permissions":{"allow":[],"deny":[]}}`), reads it otherwise, and writes atomically (tempfile + rename). It preserves untouched top-level keys and unspecified keys under `permissions` byte-for-byte; the action emitted is `created`, `updated`, or `unchanged` with per-array counts of entries added vs. duplicates removed. Otherwise (markdown-only path), the host walks the canonical sets below: read the file, ensure every canonical entry is present, remove exact-match duplicates from `permissions.allow` and `permissions.deny`, write atomically.

2. Canonical `permissions.allow` entries:

   **File operations:**
   - `Edit`
   - `Write`

   **Govern state files (no per-write confirmation):**
   - `Edit(.govern.session.toml)`
   - `Write(.govern.session.toml)`

   **Web access:**
   - `WebFetch`
   - `WebSearch`

   **Bash commands (read-only shell operations):**
   - `Bash(ls *)`

   File-content parsers (`awk`, `grep`, `cat`, `head`, `for` loops over files) are intentionally **not** in the canonical set. The runtime primitives and the host's `Read` / `Grep` / `Glob` tools cover those reads on the deterministic and markdown-only paths respectively; shell pipelines are not a sanctioned third path. See `framework/constitution.md` §runtime-boundary.

   **Git commands:**
   - `Bash(git add *)`
   - `Bash(git commit *)`
   - `Bash(git push *)`
   - `Bash(git log *)`
   - `Bash(git diff *)`
   - `Bash(git status *)`
   - `Bash(git show *)`

   **Git commands targeting another working tree (`-C <path>`):**
   - `Bash(git -C * add *)`
   - `Bash(git -C * commit *)`
   - `Bash(git -C * push *)`
   - `Bash(git -C * log *)`
   - `Bash(git -C * diff *)`
   - `Bash(git -C * status *)`
   - `Bash(git -C * show *)`

   **Utility:**
   - `Bash(curl *)`
   - `Bash(gh api *)`
   - `Bash(mkdir -p *)`
   - `Bash(chmod +x *)`
   - `Bash(command -v *)`

   **Build / lint:**
   - `Bash(make *)`
   - `Bash(markdownlint *)`
   - `Bash(markdownlint-cli2 *)`
   - `Bash(npx markdownlint-cli2 *)`

   **Hooks and generators (govern's pre-commit pipeline):**
   - `Bash(git config core.hooksPath *)`
   - `Bash(git config --get core.hooksPath)`
   - `Bash(git config --unset core.hooksPath)`
   - `Bash(./.githooks/pre-commit)`
   - `Bash(scripts/gen-*.sh)`
   - `Bash(./scripts/gen-*.sh)`
   - `Bash(scripts/install-hooks.sh)`
   - `Bash(./scripts/install-hooks.sh)`

   **Runtime MCP tools (`mcp__gvrn__*` — generated from `framework/runtime-tools.txt`):**

   <!-- generated:mcp-allow:start -->
   - `mcp__gvrn__read-spec`
   - `mcp__gvrn__read-tasks`
   - `mcp__gvrn__mark-task`
   - `mcp__gvrn__mark-criterion`
   - `mcp__gvrn__set-status`
   - `mcp__gvrn__derive-boundary`
   - `mcp__gvrn__check-stuck`
   - `mcp__gvrn__validate-frontmatter`
   - `mcp__gvrn__resolve-anchor`
   - `mcp__gvrn__traverse-deps`
   - `mcp__gvrn__check-rule-ids`
   - `mcp__gvrn__run-generator`
   - `mcp__gvrn__lint-markdown`
   - `mcp__gvrn__gate-confirm`
   - `mcp__gvrn__fetch-archive`
   - `mcp__gvrn__extract-archive`
   - `mcp__gvrn__substitute-templates`
   - `mcp__gvrn__merge-claude-md`
   - `mcp__gvrn__apply-manifest`
   - `mcp__gvrn__enforce-manifest`
   - `mcp__gvrn__merge-managed-block`
   - `mcp__gvrn__merge-permissions`
   - `mcp__gvrn__migrate-session-file`
   - `mcp__gvrn__create-scenario`
   - `mcp__gvrn__append-task`
   - `mcp__gvrn__dashboard`
   - `mcp__gvrn__write-session`
   <!-- generated:mcp-allow:end -->

3. Canonical `permissions.deny` entries:

   **Destructive file operations:**
   - `Bash(rm -rf *)`
   - `Bash(rm -r *)`
   - `Bash(rm -fr *)`
   - `Bash(*rm -rf *)`
   - `Bash(*rm -r *)`
   - `Bash(*rm -fr *)`

   **Dangerous git operations:**
   - `Bash(git mv *)`
   - `Bash(git push --force *)`
   - `Bash(git push -f *)`
   - `Bash(git reset --hard *)`
   - `Bash(git rm *)`
   - `Bash(git clean -fd *)`
   - `Bash(git -C * mv *)`
   - `Bash(git -C * push --force *)`
   - `Bash(git -C * push -f *)`
   - `Bash(git -C * reset --hard *)`
   - `Bash(git -C * rm *)`
   - `Bash(git -C * clean -fd *)`

   **Other dangerous commands:**
   - `Bash(chmod -R 777 *)`
   - `Bash(> *)`

4. Ensure `permissions.additionalDirectories` contains (host-side; not handled by `merge-permissions` — this field has no duplication problem, entries are presence-checked):
   - The `specs/` directory (absolute path)
   - The `{cli-config-dir}/commands/{project}/` directory (absolute path)

   Read the file (post-`merge-permissions` write), add any missing absolute paths to `additionalDirectories`, and write atomically.

5. Confirm what was added.
