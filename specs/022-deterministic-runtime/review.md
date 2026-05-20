---
spec: 022-deterministic-runtime
scenario: framework-list-dedup
reviewed-at: 2026-05-19T00:00:00Z
reviewed-against: dc146f52c5974a506654202c9a6d2dd924f74d8f
diff-base: dc146f52c5974a506654202c9a6d2dd924f74d8f
must-violations: 0
should-violations: 0
low-confidence: 1
skipped-passes: []
---

# Review — 022-deterministic-runtime (framework-list-dedup scenario)

## Summary

Re-bless after the SHOULD findings from the prior review were resolved:

- **REUSE-003** (local `read_text` shadow) — fixed. `merge_permissions.rs` now imports `read_text` from `crate::primitives` alongside the existing `PrimitiveError, Result, write_atomic` imports, matching every other path-reading primitive in the module.
- **SIMPLICITY-001** (unreachable fallback in `serialize_pretty`) — fixed. The `unwrap_or_else(|_| "{}".to_string())` defensive branch is replaced with `.expect("serde_json::Value serializes infallibly")`, which documents the invariant and panics loudly if the assumption is ever violated by an upstream change. `clippy::expect_used` is allowed at the module level following the established pattern in `resolve_anchor.rs:4` and `check_rule_ids.rs:4`.

Scope unchanged from the prior review (tasks #31 and #32 — the `check-stuck-read-blob-reuse` REUSE refactor and the `framework-list-dedup` primitive work). Stack: text-first markdown + Rust runtime. Loaded rule files: `configuration-cross.md`, `security-backend.md`, `api-backend.md`. None of the BE-API or BE-AUTHN/AUTHZ/etc. triggers fire against the diff (no HTTP/RPC endpoints, no credentials, no sessions, no user-input boundaries beyond CLI procedure args). `CFG-CONST-002` satisfied — `DEFAULT_PATH` is module-local.

Test posture: 299 tests pass (`cargo test --release`); `cargo clippy --release --all-targets -- -D warnings` clean; `cargo fmt --check` clean; `scripts/lint-tool-coverage.sh` clean.

**Result**: 0 MUST, 0 SHOULD, 1 low-confidence (EFFICIENCY — unchanged from prior review, deferred). `blocking: no`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

### LOW-CONFIDENCE (62): EFFICIENCY — O(n²) dedup via `Vec::contains`

- **File**:
  - `runtime/src/primitives/merge_permissions.rs:246-263` (the dedup pass uses `seen.contains(&s_owned)` and `seen.contains(entry)` on a growing `Vec<String>`)
  - `runtime/src/primitives/merge_managed_block.rs:369-372` (the dedup pass uses `canonical_lines.contains(&line)` for every adopter-area line)
- **Rule**: efficiency pass — prefer data structures with the right complexity for the operation
- **Finding**: Both dedup paths do linear scans per lookup, giving O(n×m) overall where n = entries to scan and m = the seen/canonical set. For typical inputs — a `.claude/settings.local.json` with ~30 canonical entries and ≤50 user-added entries, or a `.gitignore` managed block with ~5-20 canonical lines — this is well below the noise floor. The cost only matters if an adopter accidentally accumulates thousands of duplicate entries; in that case the bigger issue is the file shape, not the dedup speed. Routine cleanup would substitute `HashSet<String>` for `Vec<String>`, but the change has no observable impact at realistic scale.
- **Confidence**: 62 (low — impact is theoretical at expected file sizes; promote to SHOULD if a real adopter file exceeds 1k entries)
- **Auto-fixable**: yes (mechanical substitution)
- **Suggested fix**: replace the `seen: Vec<String>` and `canonical_lines: &[&str]` lookups with `HashSet`-backed equivalents. Deferred until a concrete case justifies it.

## Waived findings

_None._

## Skipped passes

_None._
