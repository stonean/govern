---
description: Configure opencode.json with permissions for slash commands.
---

# Configure

Configure the repo-root `opencode.json` `permission` block with the OpenCode permissions needed for the `govern` commands to run without per-call approval.

## Scope Boundaries

- Read and write only the repo-root `opencode.json` (or the adopter's existing `opencode.jsonc` if that is where their config lives). Do NOT modify any other file.
- This is the **same file** OpenCode reads its `mcp` server wiring from; touch only the `permission` region, and preserve `$schema`, `mcp`, and every other top-level key byte-for-byte.
- Add missing entries to the `permission` object; do NOT reorder or rewrite non-canonical entries the user (or `/govern`) previously added beyond the canonical set below.
- Do NOT scan source code, specs, or git history. This command only manages permissions.
- Reference: no constitution sections apply — this command operates on agent-specific permission state, not `govern` artifacts.

## Instructions

> OpenCode's `permission` action map (`{ tool → action }`, where the value is `"allow"` / `"ask"` / `"deny"` or, for `bash` and `external_directory`, a `{ pattern: action }` object) is a fourth permission format, distinct from Claude's `permissions.allow/deny`, Auggie's `toolPermissions[]`, and Antigravity's action grammar. It is **not** served by the `merge-permissions` runtime primitive — OpenCode's `permission` is plain key→action JSON that needs a key-preserving object merge, not allow/deny grammar reconciliation (spec 032 Resolved Q5). Walk the prose below: read the file, install the canonical set additively as a **generic JSON-object merge**, preserve every other key, write atomically.

1. Read the repo-root `opencode.json` (fall back to `opencode.jsonc` if that exists instead; if neither exists, create `opencode.json` with `{ "$schema": "https://opencode.ai/config.json", "permission": {} }`). OpenCode validates config strictly and rejects unknown top-level keys, so preserve `$schema` and only add known keys.

2. Merge the canonical entries below into the `permission` object additively. Preserve `$schema`, `mcp`, and any other top-level key; preserve `permission` entries the adopter added that are not in the canonical set. **Ordering matters** — OpenCode evaluates the **last** matching rule, so within the `bash` object the broad `"*": "ask"` comes first, specific `allow` patterns next, and `deny` patterns last; the top-level `"gvrn*"` allow is placed so no later broad rule shadows it.

3. Canonical `permission` entries:

   **Top-level tool actions:**
   - `"edit": "allow"` — `govern` edits specs, `tasks.md`, `.govern.session.toml`, and config
   - `"webfetch": "allow"`, `"websearch": "allow"` — `/govern` fetches the framework archive; research commands

   **`bash` pattern map (broad `ask` first, allows next, denies last):**

   ```json
   {
     "*": "ask",
     "ls *": "allow",
     "curl *": "allow",
     "tar *": "allow",
     "mktemp *": "allow",
     "chmod +x *": "allow",
     "awk *": "allow",
     "command -v *": "allow",
     "git status *": "allow",
     "git config *": "allow",
     "git rev-parse *": "allow",
     "git diff *": "allow",
     "git ls-files *": "allow",
     "git add *": "allow",
     "git commit *": "allow",
     "git push *": "allow",
     "git log *": "allow",
     "git show *": "allow",
     "gh api *": "allow",
     "make *": "allow",
     "markdownlint *": "allow",
     "markdownlint-cli2 *": "allow",
     "npx markdownlint-cli2 *": "allow",
     "scripts/gen-*": "allow",
     "./scripts/gen-*": "allow",
     "scripts/install-hooks.sh *": "allow",
     "./scripts/install-hooks.sh *": "allow",
     "./.githooks/pre-commit": "allow",
     "git config core.hooksPath *": "allow",
     "rm -rf *": "deny",
     "rm -r *": "deny",
     "rm -fr *": "deny",
     "sudo *": "deny",
     "git push --force *": "deny",
     "git push -f *": "deny",
     "git reset --hard *": "deny",
     "git rm *": "deny",
     "git clean -fd *": "deny",
     "chmod -R 777 *": "deny"
   }
   ```

   **Runtime MCP tools (one glob — there is no dedicated `mcp` permission key; OpenCode matches MCP tools by tool-name pattern, and `"gvrn*"` covers every gvrn tool):**

   <!-- generated:mcp-allow:start -->
   - `"gvrn*": "allow"`
   <!-- generated:mcp-allow:end -->

4. Write the file atomically (tempfile + rename), preserving `$schema`, `mcp`, and all unspecified keys.

5. Confirm what was added.

> **Note (config reload):** OpenCode loads config once at startup and does not hot-reload. After this command writes `opencode.json`, remind the user to quit and restart OpenCode for the new permissions to take effect.
