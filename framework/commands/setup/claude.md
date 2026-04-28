# Setup Permissions

Configure `{cli-config-dir}/settings.local.json` with the permissions needed for slash commands to run without manual approval.

## Instructions

1. Read `{cli-config-dir}/settings.local.json` (create it if missing, with `{"permissions":{"allow":[],"deny":[]}}`).
2. Ensure the `permissions.allow` array contains **all** of the following entries. Add any that are missing; do not duplicate existing ones:

   **File operations:**
   - `Edit`
   - `Write`

   **Web access:**
   - `WebFetch`
   - `WebSearch`

   **Bash commands used by skills (read-only shell operations):**
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

   **Other dangerous commands:**
   - `Bash(chmod -R 777 *)`
   - `Bash(> *)`

4. Ensure `permissions.additionalDirectories` contains:
   - The `specs/` directory (absolute path)
   - The `{cli-config-dir}/commands/{project}/` directory (absolute path)

5. Write the updated file and confirm what was added.
