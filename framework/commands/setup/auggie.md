# Setup Permissions

Configure `{cli-config-dir}/settings.local.json` with the tool permissions needed for slash commands to run without manual approval.

## Instructions

1. Read `{cli-config-dir}/settings.local.json` (create it if missing, with `{"toolPermissions":[]}`).
2. **Migration:** if the file contains a `permissions` key (Claude Code format), remove the entire `permissions` object. Report: "Removed incompatible `permissions` key (Claude Code format)."
3. Ensure the `toolPermissions` array contains **all** of the following entries. Add any that are missing; do not duplicate existing ones. Match on `toolName` + `shellInputRegex` (when present) to detect duplicates.

   **File operations:**
   - `{ "toolName": "str-replace-editor", "permission": { "type": "allow" } }`
   - `{ "toolName": "save-file", "permission": { "type": "allow" } }`
   - `{ "toolName": "remove-files", "permission": { "type": "deny" } }`

   **Search and read:**
   - `{ "toolName": "view", "permission": { "type": "allow" } }`
   - `{ "toolName": "grep-search", "permission": { "type": "allow" } }`
   - `{ "toolName": "codebase-retrieval", "permission": { "type": "allow" } }`

   **Web access:**
   - `{ "toolName": "web-fetch", "permission": { "type": "allow" } }`
   - `{ "toolName": "web-search", "permission": { "type": "allow" } }`

   **Shell commands — read-only operations:**
   - `{ "toolName": "launch-process", "shellInputRegex": "^ls ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^head ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^cat ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^awk ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^grep ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^for ", "permission": { "type": "allow" } }`

   **Shell commands — git:**
   - `{ "toolName": "launch-process", "shellInputRegex": "^git add ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git commit ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git push ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git log", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git diff", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git status", "permission": { "type": "allow" } }`

   **Shell commands — utility:**
   - `{ "toolName": "launch-process", "shellInputRegex": "^curl ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^gh api ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^mkdir -p ", "permission": { "type": "allow" } }`

   **Shell commands — build / lint:**
   - `{ "toolName": "launch-process", "shellInputRegex": "^make", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^markdownlint", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^npx markdownlint-cli2", "permission": { "type": "allow" } }`

   **Shell commands — denied (destructive):**
   - `{ "toolName": "launch-process", "shellInputRegex": "rm -rf ", "permission": { "type": "deny" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "rm -r ", "permission": { "type": "deny" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "rm -fr ", "permission": { "type": "deny" } }`

   **Shell commands — denied (dangerous git):**
   - `{ "toolName": "launch-process", "shellInputRegex": "^git mv ", "permission": { "type": "deny" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git push --force", "permission": { "type": "deny" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git push -f ", "permission": { "type": "deny" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git reset --hard", "permission": { "type": "deny" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git rm ", "permission": { "type": "deny" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git clean -fd", "permission": { "type": "deny" } }`

   **Shell commands — denied (other dangerous):**
   - `{ "toolName": "launch-process", "shellInputRegex": "^chmod -R 777 ", "permission": { "type": "deny" } }`

4. **Ordering:** deny entries must appear before allow entries in the `toolPermissions` array so that destructive commands are blocked even if a broader allow rule would match. When adding entries, insert deny entries at the top and allow entries after them.

5. Write the updated file and confirm what was added or migrated.
