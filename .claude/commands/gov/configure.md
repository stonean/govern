---
description: Configure settings.local.json with permissions for slash commands.
---

# Configure

Configure `.claude/settings.local.json` with the permissions needed for slash commands to run without manual approval.

## Scope Boundaries

- Read and write only `.claude/settings.local.json`. Do NOT modify any other file.
- Add missing entries; do NOT remove, deduplicate, reorder, or rewrite entries the user (or another command) added beyond the canonical set listed below.
- Do NOT scan source code, specs, or git history. This command only manages permissions.
- Reference: no constitution sections apply — this command operates on agent-specific permission state, not `govern` artifacts.

## Instructions

1. Read `.claude/settings.local.json` (create it if missing, with `{"permissions":{"allow":[],"deny":[]}}`).
2. Ensure the `permissions.allow` array contains **all** of the following entries. Add any that are missing; do not duplicate existing ones:

   **File operations:**
   - `Edit`
   - `Write`

   **Govern state files (no per-write confirmation):**
   - `Edit(.claude/gov-session.json)`
   - `Write(.claude/gov-session.json)`

   **Web access:**
   - `WebFetch`
   - `WebSearch`

   **Bash commands used by workflows (read-only shell operations):**
   - `Bash(ls *)`
   - `Bash(for *)`
   - `Bash(head *)`
   - `Bash(cat *)`
   - `Bash(awk *)`
   - `Bash(grep *)`

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

   **Runtime MCP tools (`gov-rt:` namespace — generated from `framework/runtime-tools.txt`):**

   <!-- generated:mcp-allow:start -->
   - `mcp__gov-rt__read-spec`
   - `mcp__gov-rt__read-tasks`
   - `mcp__gov-rt__mark-task`
   - `mcp__gov-rt__mark-criterion`
   - `mcp__gov-rt__set-status`
   - `mcp__gov-rt__derive-boundary`
   - `mcp__gov-rt__check-stuck`
   - `mcp__gov-rt__validate-frontmatter`
   - `mcp__gov-rt__resolve-anchor`
   - `mcp__gov-rt__traverse-deps`
   - `mcp__gov-rt__check-rule-ids`
   - `mcp__gov-rt__run-generator`
   - `mcp__gov-rt__lint-markdown`
   - `mcp__gov-rt__gate-confirm`
   - `mcp__gov-rt__fetch-archive`
   - `mcp__gov-rt__extract-archive`
   - `mcp__gov-rt__substitute-templates`
   - `mcp__gov-rt__merge-claude-md`
   - `mcp__gov-rt__apply-manifest`
   - `mcp__gov-rt__enforce-manifest`
   - `mcp__gov-rt__merge-managed-block`
   - `mcp__gov-rt__create-scenario`
   - `mcp__gov-rt__append-task`
   <!-- generated:mcp-allow:end -->

3. Ensure the `permissions.deny` array contains **all** of the following entries. Add any that are missing:

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

4. Ensure `permissions.additionalDirectories` contains:
   - The `specs/` directory (absolute path)
   - The `.claude/commands/gov/` directory (absolute path)

5. Write the updated file and confirm what was added.
