# 007 — Govern Command Tasks

## Tasks

### 1. Add `{cli-config-dir}` placeholder to command templates

Update all `.md` files in `commands/` to replace hardcoded `.claude/` references with `{cli-config-dir}/`. This affects session file paths, settings paths, and command directory references.

**Done when:** Every `.claude/` reference in `commands/*.md` that is CLI-specific uses `{cli-config-dir}/` instead. References to `.claude/` in user-facing prose (e.g., explaining what the setup command does) also use the placeholder.

### 2. Re-derive governance commands from updated templates

Regenerate `.claude/commands/gov/*.md` from the updated `commands/` templates with `{cli-config-dir}` resolved to `.claude` and `{project}` resolved to `gov`.

**Done when:** All governance commands match the updated templates with placeholders resolved. The `init.md` command (governance-specific) is updated to resolve `{cli-config-dir}` alongside `{project}` during scaffolding.

### 3. Create `govern/govern.md` for Claude Code

Write the Claude Code govern command in the `govern/` directory. It contains the full file manifest, pre-flight checks, input collection, fetch logic, placeholder substitution instructions, conflict handling, and post-scaffolding output. All paths target `.claude/`.

**Done when:** `govern/govern.md` exists, passes markdownlint, and contains the complete manifest from the spec with `.claude` as the config directory.

### 4. Create `govern/govern-auggie.md` for Auggie

Write the Auggie govern command in the `govern/` directory. Same structure as `govern.md` but targeting `.augment/` paths. The post-scaffolding next steps omit the `/{project}:setup` step.

**Done when:** `govern/govern-auggie.md` exists, passes markdownlint, and targets `.augment/` paths. Setup step is omitted from next steps.

### 5. Update spec status to `planned`

Set the spec status to `planned` and run markdownlint on all modified files.

**Done when:** Spec status is `planned`, all modified files pass markdownlint.
