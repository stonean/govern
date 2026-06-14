---
description: Configure settings.local.json with permissions for slash commands.
---

# Configure

Configure `{cli-config-dir}/settings.local.json` with the tool permissions needed for slash commands to run without manual approval.

## Instructions

> The `merge-permissions` runtime primitive currently targets the Claude permission shape (`permissions.allow` / `permissions.deny` as string arrays). Auggie's `toolPermissions` array of `{toolName, shellInputRegex, permission}` objects is structurally different and is NOT yet served by a deterministic primitive — whether `merge-permissions` grows a format argument or whether a separate Auggie-format primitive is introduced is a plan-phase decision tracked as an open question on the `framework-list-dedup` scenario (`specs/022-deterministic-runtime/scenarios/framework-list-dedup.md`). Until that lands, Auggie callers walk the prose below: install the canonical set, remove exact-match duplicates by host-side splice.

1. Read `{cli-config-dir}/settings.local.json` (create it if missing, with `{"toolPermissions":[]}`).
2. Ensure the `toolPermissions` array contains **all** of the following entries AND that no exact-match duplicate of an entry (matched on `toolName` + `shellInputRegex` when present) survives the run. Add any canonical entries that are missing; remove duplicates so that each `(toolName, shellInputRegex)` pair appears at most once. First-occurrence wins; later duplicates are removed in place. Do not reorder or rewrite non-duplicate entries beyond the canonical set listed below.

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
   - `{ "toolName": "launch-process", "shellInputRegex": "^chmod \\+x ", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^command -v ", "permission": { "type": "allow" } }`

   **Shell commands — build / lint:**
   - `{ "toolName": "launch-process", "shellInputRegex": "^make", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^markdownlint", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^npx markdownlint-cli2", "permission": { "type": "allow" } }`

   **Shell commands — hooks and generators:**
   - `{ "toolName": "launch-process", "shellInputRegex": "^git config core\\.hooksPath", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^git config --(get|unset) core\\.hooksPath", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^\\./.githooks/pre-commit", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^\\./?scripts/gen-.*\\.sh", "permission": { "type": "allow" } }`
   - `{ "toolName": "launch-process", "shellInputRegex": "^\\./?scripts/install-hooks\\.sh", "permission": { "type": "allow" } }`

   **Runtime MCP tools (`mcp:gvrn:*` — generated from `framework/runtime-tools.txt`):**

   <!-- generated:mcp-allow:start -->
   - `{ "toolName": "mcp:gvrn:read-spec", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:read-tasks", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:mark-task", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:mark-criterion", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:set-status", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:derive-boundary", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:check-stuck", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:validate-frontmatter", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:resolve-anchor", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:resolve-references", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:traverse-deps", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:check-rule-ids", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:run-generator", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:lint-markdown", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:gate-confirm", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:fetch-archive", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:extract-archive", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:substitute-templates", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:merge-claude-md", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:apply-manifest", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:enforce-manifest", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:merge-managed-block", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:merge-permissions", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:migrate-session-file", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:create-scenario", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:append-task", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:dashboard", "permission": { "type": "allow" } }`
   - `{ "toolName": "mcp:gvrn:write-session", "permission": { "type": "allow" } }`
   <!-- generated:mcp-allow:end -->

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
   - `{ "toolName": "launch-process", "shellInputRegex": " > ", "permission": { "type": "deny" } }`

3. **Ordering:** deny entries must appear before allow entries in the `toolPermissions` array so that destructive commands are blocked even if a broader allow rule would match. When adding entries, insert deny entries at the top and allow entries after them.

4. Write the updated file and confirm what was added.
