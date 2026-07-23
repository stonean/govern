---
spec: 042-consolidate-govern-per-project-files-under-govern-directory
reviewed-at: 2026-07-23T00:01:01Z
reviewed-against: 3d135ca0da936dfbb8b3c78014a5f296638c8769
diff-base: 790ebd9d03c2c1061f7289d179f8d2a7f931a212
must-violations: 0
should-violations: 1
low-confidence: 1
captured-issues: 3
skipped-passes: []
---

# Review — 042-consolidate-govern-per-project-files-under-govern-directory

## Summary

Spec 042 relocates govern's per-project files under .govern/ — a path-resolution and prose-sweep feature with no new attack surface: no user input reaches the new path constructions (fixed constants joined to the repo root; the specs-root charset allowlist guards the one regex interpolation), writes remain atomic tempfile+rename, and constants stay centralized in schema/paths.rs. 0 MUST violations across all five passes; 1 SHOULD (reuse: the active-file rule is restated inline at ~7 prose sites where a pointer to the canonical §Project Configuration statement would prevent drift) and 1 low-confidence note (theoretical probe-to-use race against a concurrently running /govern migration, mitigated by the serial-pipeline design and atomic writes). Not blocking.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

### SHOULD: QUAL-REUSE — active-file resolution rule restated inline at ~7 prose sites

- **File**: `framework/bootstrap/govern.md`
- **Rule**: Reuse pass: identify logic that duplicates existing utilities or that should be extracted into shared code (review.md §Reuse pass); constitution §drift-prevention: shared knowledge has one canonical source.
- **Finding**: The three-branch active-file rule (`.govern/` when it exists, else legacy root when that exists, else `.govern/`) is restated near-verbatim at govern.md step 6, §Inputs intro, §Pre-run Migrations step 2, the workflow-decline step, §Project Configuration's write-policy paragraph, review.md steps 1 and 4, and link.md's Scope Boundaries and Additive write. The canonical statements already exist (§Project Configuration prose; runtime `config_path_for_write`); each inline restatement is a future drift site if the rule ever changes.
- **Auto-fixable**: no
- **Suggested fix**: Where a site only needs the rule (not walker-context detail), replace the inline three-branch parenthetical with a short pointer, e.g. "the active config file (per §Project Configuration's write policy)". Keep the full statement in §Project Configuration and the runtime resolvers.

## Low-confidence findings

### LOW-CONFIDENCE: BE-RACE-001 — resolver existence-probe → use window races a concurrent migration

- **File**: `runtime/src/schema/paths.rs:64-109`
- **Rule**: Shared mutable state reachable from more than one concurrent execution context MUST be protected by a synchronization mechanism — a lock, an atomic primitive, single-owner/actor confinement, or serialized access; unsynchronized concurrent read-write is a data race.
- **Finding**: config_path/session_path/active_path probe existence and return a PathBuf the caller later opens; a /govern migration moving the file between probe and use yields a missing-file read (benign: callers treat as absent → defaults/fallback) or, worst case, a session write landing on the legacy path concurrently with the migration's move, losing that write. Mitigated by design: the pipeline is serial per constitution §concurrent-features, the migration runs only inside /govern, and all writes are atomic tempfile+rename — recorded low-confidence for visibility, not as a confirmed defect.
- **Auto-fixable**: no

## Waived findings

*None.*

## Captured issues

- [ ] Test infra: runtime/tests/exec_subprocess.rs + parity.rs `ensure_binary_built()` only builds `target/release/gvrn` when absent, so subprocess-based integration tests silently run a STALE release binary and don't reflect current src changes (discovered during spec 042 — new-layout exec seed appeared to fail until `cargo build --release` was run manually). Consider always rebuilding (or using CARGO_BIN_EXE) so exec/parity tests validate current code.
- [ ] Display-literal drift (spec 042 follow-up): runtime-emitted provenance tags still say `(.govern.toml)` — `discover_rule_files.rs:273` (`disabled-rule-file: … (.govern.toml)`) and `dashboard.rs:234` (`disabled rule files: {N} (.govern.toml) — …`) — mirrored verbatim in `review.md:197` and `status.md:59`. Update runtime literals + both doc mirrors + any parity goldens together (ideally name the resolved config path instead of a hardcoded one); doc-only edits would break doc↔runtime message parity.
- [ ] Stale CLI doc strings (spec 042 follow-up): `gvrn write-session --help` says "Atomically rewrite `.govern.session.toml`" — the write targets the active file (`.govern/session.toml` post-migration) via session_path_for_write. Sweep the runtime's user-visible doc comments/help strings (main.rs Command variants, schema arg docs) for legacy `.govern.session.toml` / `.govern.toml` mentions.

## Skipped passes

*None.*
