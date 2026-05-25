---
spec: 022-deterministic-runtime
scenario: commands-dir-parameterization
reviewed-at: 2026-05-24T23:30:00Z
reviewed-against: 8d42f50
diff-base: 36461bd
must-violations: 0
should-violations: 0
low-confidence: 2
waived-violations: 0
skipped-passes: []
notes: "Both SHOULDs from the initial pass (CFG-CONST-003 const extraction in Host::defaults and copy_dir_recursive duplicated across integration test crates) were resolved in the same change set — landed as follow-up commits before this review was finalized. The findings are retained below for the audit trail."
---

# Review — 022-deterministic-runtime (scenario: commands-dir-parameterization)

## Summary

Scenario-targeted review of the `commands-dir-parameterization` scenario
(task 41). Scope: 7 commits from `36461bd..8d42f50` covering the new `Host`
config loader (`runtime/src/host.rs`), the two parameterized callsites in
`run_exec` and `locate_command_file`, the bootstrap procedure's new
`merge-managed-block` step, the Auggie-shaped fixture + integration test,
Family 13 of the audit suite, and a small terminology sweep in two live
framework files.

Initial pass surfaced two SHOULDs and two low-confidence notes. Both
SHOULDs were resolved in-session (see **Resolved SHOULDs** below); two
low-confidence notes remain (pre-existing test-helper hazard and
defensive input validation). Zero MUST violations either way. The
scenario's described behavior is correctly implemented and verified by
unit tests, integration tests, and parity goldens — all green.

`blocking: false`. The spec is clear to advance to `done`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None remaining — both SHOULDs from the initial pass were resolved in-session._

## Resolved SHOULDs

### RESOLVED: CFG-CONST-003 — extract `Host::defaults` literals to module-level `const`

- **File**: `runtime/src/host.rs`
- **Rule**: `framework/rules/configuration-cross.md` §CFG-CONST-003 —
  "Operator-tunable values ... MUST be backed by a named constant or
  an environment variable. They MUST NOT appear as bare literals in
  business logic."
- **Finding**: `Host::defaults` inlined two operator-tunable defaults
  as string literals (`".claude"` for `cli_config_dir`, `"gov"` for
  the `project` fallback). The codebase convention is to extract
  module-level defaults as `const` (e.g., `merge_managed_block.rs`'s
  `DEFAULT_MARKER`).
- **Resolution**: Extracted `const DEFAULT_CLI_CONFIG_DIR: &str =
  ".claude"` and `const FALLBACK_PROJECT: &str = "gov"` at module
  scope with doc-comments explaining each. `Host::defaults` now
  references the consts. All 6 host loader unit tests still pass; the
  test assertions deliberately retain the literal values to catch any
  accidental change to the const definitions.

### RESOLVED: REUSE — `copy_dir_recursive` duplicated across integration test crates

- **File**: `runtime/tests/exec_subprocess.rs` (had a freshly-added
  duplicate) and `runtime/tests/parity.rs` (pre-existing canonical
  copy).
- **Rule**: `AGENTS.md` "no dead references in live artifacts" implies
  one canonical source for shared logic.
- **Finding**: Task 41's integration test added a second
  `copy_dir_recursive` helper alongside the one in `parity.rs`. Rust's
  integration-test model (each `tests/*.rs` is its own compilation
  unit) makes inline duplication tempting; the idiomatic fix is a
  `tests/common/mod.rs` shared sub-module that each integration test
  declares with `mod common;`.
- **Resolution**: Created `runtime/tests/common/mod.rs` with the
  shared `copy_dir_recursive`. Both `parity.rs` and `exec_subprocess.rs`
  now declare `mod common;` and import `common::copy_dir_recursive`.
  The canonical implementation is the parity.rs shape (more defensive:
  creates `dst` upfront and handles missing parents for nested files).
  All 16 parity tests + 6 exec_subprocess tests pass against the
  consolidated helper.

## Low-confidence findings

### LOW: ensure_binary_built only rebuilds when the binary is absent (pre-existing)

- **File**: `runtime/tests/exec_subprocess.rs:27-43` (pre-existing helper;
  inherited by my new test)
- **Confidence**: 65
- **Finding**: `ensure_binary_built` short-circuits when
  `runtime/target/release/gvrn` exists, regardless of whether the
  source has changed. During this task's development, the helper
  silently ran my new test against a stale binary that pre-dated the
  `Host::load` wiring — the test failed with "command file not found"
  until I manually invoked `cargo build --release`. The fix is to
  always invoke `cargo build --release` (cargo's incremental
  compilation is fast enough that "the build is no-op when nothing
  changed" remains true). This is pre-existing scaffolding, not a
  regression from my work, but my new test inherits the hazard.
- **Suggested fix**: Drop the `if binary.exists()` early-return in
  `ensure_binary_built` and let cargo decide whether to rebuild.

### LOW: BE-INPUT-004 — `.govern.toml` `[host]` values flow into filesystem paths without canonicalization

- **File**: `runtime/src/host.rs:38-60`, `runtime/src/main.rs:209-220`,
  `runtime/src/interpreter/payload.rs:378-393`
- **Confidence**: 40
- **Finding**: `Host::load` reads `cli_config_dir` and `project` from
  `.govern.toml` and joins them into command-resolution paths without
  validating that neither contains `..`, path separators, or other
  traversal-shaped components. The current threat model treats
  `.govern.toml` as trusted local config (the adopter authored it; if
  they wanted to read random files, they could edit the source
  directly), so this is not a live security concern. The defensive
  posture would be to require each value to be a single
  path-component without traversal tokens — closes the door if the
  runtime is ever used in a context where `.govern.toml` is
  untrusted (e.g., CI running against PR-submitted config). Not
  flagged as MUST because the rule's "user-supplied values" language
  targets HTTP/RPC request input, not local config files.
- **Suggested fix**: Add a `validate(&self)` method to `Host` that
  rejects values containing `..` or path separators, called from
  `Host::load` before returning; surface validation failures as a
  warning and fall back to defaults.

## Waived findings

_None._

## Skipped passes

_None._

## Pass-by-pass results

Final counts (after the in-session SHOULD resolutions):

| Pass | MUST | SHOULD | Low-confidence | Notes |
| --- | --- | --- | --- | --- |
| Security | 0 | 0 | 1 | BE-INPUT-004 (defensive) — see above |
| Reuse | 0 | 0 | 0 | `copy_dir_recursive` consolidated to `tests/common/mod.rs` |
| Quality | 0 | 0 | 1 | `ensure_binary_built` pre-existing hazard |
| Efficiency | 0 | 0 | 0 | `Host::load` once per `gvrn exec`; not in a hot path |
| Simplicity | 0 | 0 | 0 | `Host::defaults` literals extracted to module-level `const`s |
