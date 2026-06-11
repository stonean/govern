---
description: Configure .agents/settings.json with permissions for slash commands.
---

# Configure

Configure `{cli-config-dir}/settings.json` with the Antigravity tool permissions needed for the `govern` skills to run without per-call approval.

## Scope Boundaries

- Read and write only `{cli-config-dir}/settings.json`. Do NOT modify any other file.
- Add missing entries and remove exact-match duplicates from `permissions.allow`, `permissions.deny`, and `permissions.ask`; do NOT reorder or rewrite non-duplicate entries the user (or another command) added beyond the canonical set listed below.
- Do NOT scan source code, specs, or git history. This command only manages permissions.
- Reference: no constitution sections apply — this command operates on agent-specific permission state, not `govern` artifacts.

## Instructions

> Antigravity's `{ "permissions": { "allow": [], "deny": [], "ask": [] } }` shape is structurally distinct from Claude's `permissions.allow/deny` and Auggie's `toolPermissions[]`, and is **not yet** served by the `merge-permissions` runtime primitive (whether the primitive grows a third format is a plan-phase decision tracked on spec 022). Until that lands, walk the prose below: read the file, install the canonical set additively, remove exact-match duplicates, write atomically.

1. Read `{cli-config-dir}/settings.json` (create it if missing, with `{ "permissions": { "allow": [], "deny": [], "ask": [] } }`).
2. Ensure `permissions.allow`, `permissions.deny`, and `permissions.ask` contain all of the canonical entries below. Add any that are missing; remove exact-match duplicates so each `action(target)` string appears at most once per array. Do not reorder or rewrite non-duplicate entries beyond the canonical set. Preserve any other top-level keys and unspecified keys under `permissions` byte-for-byte.

   Antigravity auto-allows reads and writes of files **inside the workspace** by default, so `read_file`/`write_file` entries are intentionally omitted — `govern`'s edits to specs, `.govern.session.toml`, and config all fall under the workspace auto-allow. Only out-of-workspace and non-file actions need explicit grants.

3. Canonical `permissions.allow` entries:

   **Web access (`govern` fetches the framework archive and gitignore patterns from GitHub):**
   - `read_url(github.com)`
   - `read_url(githubusercontent.com)`

   **Shell — read-only and utility:**
   - `command(ls)`
   - `command(curl)`
   - `command(gh api)`
   - `command(mkdir -p)`
   - `command(chmod +x)`
   - `command(which)`

   **Shell — git:**
   - `command(git add)`
   - `command(git commit)`
   - `command(git push)`
   - `command(git log)`
   - `command(git diff)`
   - `command(git status)`
   - `command(git show)`
   - `command(git config)`

   **Shell — build / lint:**
   - `command(make)`
   - `command(markdownlint)`
   - `command(markdownlint-cli2)`
   - `command(npx markdownlint-cli2)`

   **Shell — hooks and generators (`govern`'s pre-commit pipeline):**
   - `command(scripts/gen-.*)`
   - `command(./scripts/gen-.*)`
   - `command(./.githooks/pre-commit)`
   - `command(scripts/install-hooks.sh)`
   - `command(./scripts/install-hooks.sh)`

   **Runtime MCP tools (`mcp(gvrn/...)` — generated from `framework/runtime-tools.txt`):**

   <!-- generated:mcp-allow:start -->
   - `mcp(gvrn/*)`
   <!-- generated:mcp-allow:end -->

4. Canonical `permissions.deny` entries:

   **Destructive file operations:**
   - `command(rm -rf)`
   - `command(rm -r)`
   - `command(rm -fr)`
   - `command(sudo)`

   **Dangerous git operations:**
   - `command(git mv)`
   - `command(git push --force)`
   - `command(git push -f)`
   - `command(git reset --hard)`
   - `command(git rm)`
   - `command(git clean -fd)`

   **Other dangerous commands:**
   - `command(chmod -R 777)`

5. Canonical `permissions.ask` entries: none by default — leave `ask` as `[]` unless the adopter has added their own. Antigravity already defaults un-granted `command`/`read_url`/`mcp` actions to Ask, so no explicit `ask` entries are required for safe operation.

6. Confirm what was added.

> **Note (`-C` worktree git):** the `claude-style` configure sets allow `git -C <path> …` variants. They are intentionally omitted here — Antigravity's token-prefix `command()` matching would over-broaden them, and `git -C` is rare in the pipeline; those invocations fall through to Antigravity's default Ask prompt.
