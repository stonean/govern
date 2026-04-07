# 007 — Govern Command Tasks

## Tasks

- [x] Add `{cli-config-dir}` placeholder to command templates — Update all `.md` files in `commands/` to replace hardcoded `.claude/` references with `{cli-config-dir}/`. Done when: every `.claude/` reference in `commands/*.md` that is CLI-specific uses `{cli-config-dir}/` instead.

- [x] Re-derive governance commands from updated templates — Regenerate `.claude/commands/gov/*.md` from the updated `commands/` templates with `{cli-config-dir}` resolved to `.claude` and `{project}` resolved to `gov`. Done when: all governance commands match the updated templates with placeholders resolved.

- [x] Create `govern/govern.md` for Claude Code — Write the Claude Code govern command in the `govern/` directory with full file manifest, pre-flight checks, input collection, fetch logic, placeholder substitution, conflict handling, and post-scaffolding output. Done when: `govern/govern.md` exists, passes markdownlint, and contains the complete manifest with `.claude` as the config directory.

- [x] Create `govern/govern-auggie.md` for Auggie — Same structure as `govern.md` but targeting `.augment/` paths, with setup step omitted from next steps. Done when: `govern/govern-auggie.md` exists, passes markdownlint, and targets `.augment/` paths.

- [x] Update spec status to `planned` — Set the spec status to `planned` and run markdownlint on all modified files. Done when: spec status is `planned`, all modified files pass markdownlint.
