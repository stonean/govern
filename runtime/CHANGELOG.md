# Changelog

All notable changes to the `govern` deterministic runtime are recorded here. The runtime ships in lockstep with the framework per [§runtime-boundary](../framework/constitution.md#runtime-boundary); release tags use the `gvrn-v<MAJOR>.<MINOR>.<PATCH>` scheme distinct from framework tags (was `runtime-v*` before v0.2.0 — see the v0.2.0 rename entry below).

## [0.5.0] — 2026-05-17

### Changed (breaking)

- **MCP wire format**: tool names no longer carry the `gov-rt:` prefix. The 23 tools are now registered as bare `<verb>-<noun>` strings (`read-spec`, `read-tasks`, `mark-task`, …) — the same names already used by the `gvrn <subcommand>` CLI surface, so the binary's two surfaces finally agree on identifiers. Server-level namespacing is supplied by the adopter's `.mcp.json` server registration. The canonical server name is **`gvrn`** (was conceptually `gov-rt`), aligning the MCP server name with the binary/crate name. Resulting per-host wire identifiers:
  - Claude Code: `mcp__gvrn__<verb>-<noun>`
  - Auggie: `mcp:gvrn:<verb>-<noun>`

  **Adopter impact**: adopters who previously registered the runtime under the name `gov-rt` in `.mcp.json` must rename it to `gvrn`. Adopters who hand-authored permissions entries referencing `mcp__gov-rt__<tool>` or `mcp:gov-rt:<tool>` must update those entries to use `gvrn`. `framework/bootstrap/configure/{claude,auggie}.md` and the generated `.claude/commands/gov/configure.md` carry the new identifiers; re-running `/gov:configure` after a framework update is sufficient to refresh permission lists. No CLI-level changes — `gvrn <subcommand>` invocations are unchanged.

  **Why now**: the `gov-rt:` namespace was chosen in spec 022 to disambiguate tool names from `/gov:` slash commands at a time when the tool name itself carried the prefix (and a colon, which is not a valid identifier character in Claude Code MCP tool names). Switching to bare names removed the colon; the remaining `gov-rt` token then existed only at the server-name boundary, where it duplicated the `gvrn` binary/crate identity without adding meaning.

### Changed

- `scripts/gen-configure-mcp.sh`: trap-based tempfile cleanup so any early-exit path (set -e, splice failure, signal) releases the per-host block tempfiles instead of leaking them into `$TMPDIR`. Unused `label` parameter dropped from `process()`. SHOULD-tier findings from `/gov:review --fix`.
- `scripts/lint-tool-coverage.sh`: tool references inside a command file's `## Markdown-only reference` section are now skipped — that section *is* the fallback path, so references there do not require a paired fallback marker. Whitespace-strip on manifest lines tightened from "one leading/trailing space" to "any run of `[[:space:]]`". `|| true` added to the section-header lookup so `set -euo pipefail` does not abort when a command file has no markdown-only-reference section. SHOULD-tier findings from `/gov:review --fix`.

## [0.4.1] — 2026-05-16

### Changed

- `create-scenario` and `append-task` now validate caller-supplied path components before any filesystem operation, addressing the four SHOULD findings from `/gov:review` against scenario `022.ask-consolidation`:
  - **BE-INPUT-004 defense-in-depth** — new `validate_slug` and `validate_no_traversal` helpers in `primitives/mod.rs` reject slugs containing path separators or leading dots and reject `feature_path` values that are absolute or contain `..` components. New `PrimitiveError::InvalidSlug { slug, reason }` and `PrimitiveError::InvalidPath { path, reason }` variants surface the rejections as clean operational errors. Defense-in-depth: the existing `is_dir` checks remain, but the new validators close the rule's letter (canonical-path + base-dir check) as well as its spirit.
  - **REUSE** — new shared `iter_numbered_headings(content)` helper in `primitives/mod.rs` yields ATX-2 numbered headings while skipping fenced code blocks. `append-task`'s `next_task_number` is now a one-line `iter_numbered_headings(content).max().unwrap_or(0) + 1`, dropping ~30 lines of duplicate parsing. Available to future primitives that walk `tasks.md` headings.
  - **QUALITY** — `append-task`'s newly-created `tasks.md` now emits `Tasks. Complete in order.` (unlinked) when no `plan.md` exists at the time of creation, and the original `Tasks derived from the [plan](plan.md). Complete in order.` (linked) when `plan.md` is present. Closes the dangling-link case that markdownlint MD051 would flag.
- 19 new unit tests cover the validators, the shared heading-iterator helper, and the conditional intro behavior. Total lib tests grow 203 → 222; full suite 256 passing.

## [0.4.0] — 2026-05-16

### Added

- Two primitives for the `/ask` scenario branch introduced in spec 023, landing via scenario `022.ask-consolidation`:
  - `create-scenario` — write a `scenarios/{slug}.md` file under a feature with `section` frontmatter and Context / Behavior / (optional) Edge Cases body sections. Atomic via tempfile-in-parent + `persist` rename. Creates the `scenarios/` subdirectory if absent. Refuses with `ScenarioConflict` when the destination already exists; refuses with `FeaturePathNotFound` when the feature directory is missing.
  - `append-task` — append a numbered `## N. <title>` block to a feature's `tasks.md`, computing the next number as `max(existing) + 1` so a tasks file with gaps doesn't overwrite existing entries. Creates `tasks.md` with a heading derived from the feature's spec H1 (or a minimal `# Tasks` fallback when the spec is unreadable). Atomic via tempfile-in-parent + `persist` rename. Skips numeric headings inside fenced code blocks.
- New MCP tool names: `gov-rt:create-scenario`, `gov-rt:append-task`. Tool list grows from 21 to 23 entries; both `framework/runtime-tools.txt` and `mcp::server::TOOL_NAMES` carry them.
- New CLI subcommands: `gvrn create-scenario` and `gvrn append-task` (clap-derive args; same JSON envelope on stdout as other write primitives).
- New `PrimitiveError` variants: `ScenarioConflict { path }` and `FeaturePathNotFound { path }`.

## [0.3.1] — 2026-05-12

### Changed

- `enforce-manifest`'s glob compiler now delegates per-character escaping to `regex::escape` (already a transitive dependency via `regex`) instead of maintaining a hand-written metacharacter table. Internal refactor only; the glob → regex translation is byte-for-byte identical, all 14 `enforce_manifest::tests` still pass unchanged (including the metacharacter and bracket-literal coverage). Surfaced by `/gov:review`'s simplicity pass against 022-deterministic-runtime scenario `apply-manifest`.

## [0.3.0] — 2026-05-12

### Added

- Three primitives for strategy-aware bulk install + cleanup (scenario `022.apply-manifest`):
  - `apply-manifest` — strategy-aware bulk substitute + write driven by a typed manifest. Each `ManifestEntry { source, dest, strategy, keep-literals }` records an outcome (`created` / `updated` / `unchanged` / `skipped-exists` / `skipped-pinned` / `source-missing`); aggregate counts roll up across entries. Strategies: `update` (substitute, write only on diff, preserve mtime when unchanged), `create` (substitute, write only when dest absent), `skip-if-conflict` (write verbatim without substitution, only when dest absent). `pinned` short-circuits before any read/write — adopter customizations are never touched. `keep-literals` per entry masks named substitution keys, used by the `govern.md` self-install to keep `{project}` / `{cli-config-dir}` literal for the next adopter's bootstrap.
  - `enforce-manifest` — directory cleanup against an expected file list. Removes files matching `glob-include` (default `*.md`) whose relative path is neither expected nor pinned. `recursive: false` (default) is top-level only; `recursive: true` descends. One call replaces the bootstrap's three legacy cleanup loops (slash-command manifest enforcement, legacy `skills/` removal, legacy workflow filename removal).
  - `merge-managed-block` — generalization of `merge-claude-md` to support configurable marker shapes. `marker-style: "html-comment"` (default) reproduces `merge-claude-md`'s exact behavior; `marker-style: "line-prefix"` uses a `# {marker}` preamble line followed by the block, terminated by a blank line or EOF — matching `.gitignore` and `.gitattributes` conventions.
- New MCP tool names: `gov-rt:apply-manifest`, `gov-rt:enforce-manifest`, `gov-rt:merge-managed-block`. Tool list grows from 14 to 17 entries; both `framework/runtime-tools.txt` and `mcp::server::TOOL_NAMES` carry them, and the MCP integration test exercises every new tool.
- `framework/bootstrap/govern.md` Instructions section rewritten to drive the bootstrap through six primitive calls (`fetch-archive` → `extract-archive` → `apply-manifest` → `merge-managed-block` for `.gitignore` → `enforce-manifest` → `apply-manifest` with `keep-literals` for the `govern.md` self-install) plus a gate-confirm and two prose steps. No host-generated bash walker required; no `python3 -c '...'` substitution fallback; no per-file Edit calls from the host.
- `govern-basic` parity fixture extended to exercise every new strategy + marker style + cleanup path end-to-end. New companion test `govern_basic_post_run_filesystem_state_matches_expectations` walks the post-run target tree and asserts the per-primitive on-disk effects (substitutions applied where expected and NOT where suppressed, pinned file preserved verbatim, keep-literals placeholders kept literal, line-prefix `.gitignore` created, legacy file pruned).

### Changed

- `merge-claude-md` is now a thin compat shim that delegates to `merge-managed-block` with `marker-style: "html-comment"` and `marker: "govern-managed"`. The primitive's public API (`MergeClaudeMdArgs`, `MergeClaudeMdResult`) is unchanged, so existing callers — the bootstrap fixture, parity goldens, and any host scripts — keep working byte-for-byte. Slated for removal in the next major release of `gvrn`.

## [0.2.1] — 2026-05-12

### Changed

- **BREAKING** — `fetch-archive` argument `sha256_url` becomes `Option<String>`. Callers that omit the field download without sidecar verification; the primitive returns the computed sha256 digest and `verified: false` so the host can compare against a known-good value out-of-band. Callers that supply the field keep the verified path verbatim. The result struct grows a `verified: bool` field that reports which path the call took. Motivation: the `/govern` bootstrap operates live-on-main and fetches GitHub's auto-generated source tarballs (`/archive/refs/heads/main.tar.gz`), which ship without sidecars; before 0.2.1 the runtime couldn't drive that fetch and `/govern` always fell back to the markdown-only path.

### Updated

- `framework/bootstrap/govern.md`: step 2 prose acknowledges the sidecar-optional behavior and documents what `verified: false` means for callers that care about integrity.

## [0.2.0] — 2026-05-12

### Added

- Four primitives for the bootstrap procedure (scenario `022.govern-bootstrap`):
  - `fetch-archive` — download an archive plus its sha256 sidecar via reqwest's blocking client and verify the hash before persisting. Adds `reqwest` (blocking, rustls-tls) and `sha2` deps; a 256 MiB per-fetch ceiling caps memory defensively.
  - `extract-archive` — extract `.tar.gz`/`.tgz`/`.zip` in-process (no shell-out) via `flate2` + `tar` and the `zip` crate. Path-traversal protection rejects absolute paths and `..` components before writing.
  - `substitute-templates` — walk a source tree, apply `{key}` → value replacements to text files, copy binary files unchanged, write to a destination tree. Args use `source-dir` / `target-dir` (distinct from extract-archive's `dest` so both primitives can share a single procedure context).
  - `merge-claude-md` — idempotent BEGIN/END marker insert/update for a framework-managed block in CLAUDE.md. Four actions: created / inserted / updated / unchanged; unchanged preserves mtime.
- `gvrn exec` command resolution now considers `framework/bootstrap/<name>.md` as a third candidate after the existing two paths, so the `/govern` bootstrap procedure runs through the runtime.
- `framework/bootstrap/govern.md` gains a parseable `## Instructions` section that exercises the four new primitives plus a gate-confirm for the install approval; the existing 788-line procedure stays in place as the markdown-only reference.

### Changed

- **BREAKING** — package, binary, and library all renamed `runtime` / `govern_runtime` / `govern-runtime` → `gvrn`. Release tag pattern becomes `gvrn-v*` (was `runtime-v*`); release artifacts become `gvrn-<TARGET>.tar.gz` (was `runtime-<TARGET>.tar.gz`).
- **BREAKING** — `substitute-templates` argument names `source` / `dest` → `source-dir` / `target-dir` to avoid clashing with `extract-archive`'s `dest` in shared procedure context.

## [0.1.0] — 2026-05-12

### Added

- Crate scaffold under `runtime/`: `Cargo.toml`, binary entrypoint at `src/main.rs`, library root at `src/lib.rs`, module placeholders for `parser`, `interpreter`, `primitives`, `mcp`, `schema`, and `io`.
- Lint configuration in `Cargo.toml`: `unsafe_code = "forbid"`, `missing_docs = "warn"`, clippy `all` + `pedantic` at warn, plus `unwrap_used` and `expect_used` at warn.
- Pre-commit verification (`.githooks/pre-commit`): when staged changes touch `runtime/`, the hook runs `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`. Set `GOVERN_SKIP_RUNTIME_CHECKS=1` to bypass for a single commit.
- `runtime/bacon.toml` — `bacon` job definitions (`check`, `clippy`, `test`, `fmt`) with `clippy` as the default. Install with `cargo install --locked bacon`.
