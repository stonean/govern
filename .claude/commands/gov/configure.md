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

   **Build / lint:**
   - `Bash(make *)`
   - `Bash(markdownlint *)`
   - `Bash(markdownlint-cli2 *)`
   - `Bash(npx markdownlint-cli2 *)`

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
