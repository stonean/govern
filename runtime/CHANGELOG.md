# Changelog

All notable changes to the `govern` deterministic runtime are recorded here. The runtime ships in lockstep with the framework per [§runtime-boundary](../framework/constitution.md#runtime-boundary); release tags use the `gvrn-v<MAJOR>.<MINOR>.<PATCH>` scheme distinct from framework tags (was `runtime-v*` before v0.2.0 — see the v0.2.0 rename entry below).

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
