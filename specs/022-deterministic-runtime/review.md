---
spec: 022-deterministic-runtime
scenario: dashboard-primitive
reviewed-at: 2026-05-23T13:53:14Z
reviewed-against: 2d9116d2e65432f72af474311b70eeea74549b91
diff-base: 0737133
must-violations: 0
should-violations: 5
low-confidence: 2
skipped-passes: []
---

# Review — 022-deterministic-runtime (dashboard-primitive scenario)

## Summary

Scenario review at HEAD `2d9116d`. The `dashboard` primitive (commits `0737133..2d9116d`) is a clean, read-only addition that fits the existing primitive scaffolding. Stack: text-first markdown + Rust runtime. Loaded rule files: `api-backend.md`, `configuration-cross.md`, `security-backend.md`. Frontend rule files skipped — no frontend surface. No `[[review.disabled-rule-files]]` entries.

This re-review supersedes the prior pass against `c15ae0e`. The one auto-fixable SHOULD from that run (`load_session_target` accepted an unused `&[DashboardSpec]` parameter) has been resolved in commit `2d9116d` — the parameter is removed, the function signature is `fn load_session_target(repo: &Path) -> Result<...>`, and the doc comment now explicitly documents the "echo as-recorded" contract that the scenario mandates. Five SHOULD findings remain, no MUST.

Security posture is solid — no caller-supplied paths cross into filesystem operations (`DashboardArgs` is `{}`), all parse failures route through structured `PrimitiveError::{Yaml, Toml, Json}` variants, the `.govern.toml` `reason` field is correctly excluded from the payload per the scenario's explicit contract, and YAML / TOML / JSON parsing all use safe-by-default modes. The remaining SHOULDs are reuse opportunities: `count_open_questions` and `context_summary` both reimplement section-traversal logic that already lives in `read_spec.rs::section_lines`; pulling a shared helper into `primitives/mod.rs` would prevent future drift between the two implementations' subtle differences in nested-bullet and continuation-line handling.

Five-dimension review of the delta:

- **Security**: 0 findings. The `dashboard` primitive is a hard-coded-path read-only walker — every filesystem operation joins a hard-coded relative (`specs`, `.govern.toml`, `.claude/gov-session.json`) onto the runtime's repo root; no caller input crosses into a path, so `BE-INPUT-004` (canonical-path containment) does not apply. `BE-INPUT-008` (deserialization safety) is satisfied via `serde_yaml::from_str` (safe-by-default in this crate), `serde_json::from_str`, and `toml::from_str` — all data-only, no code-execution surface. Malformed input on any of the three formats routes into a structured `PrimitiveError` variant rather than panicking. The `[[review.disabled-rule-files]]` `reason` field is correctly excluded from the dashboard payload (`DisabledRuleFile` in `runtime/src/primitives/dashboard.rs` declares only `file: String`), matching the scenario's "Reasons are not surfaced" promise. The session-target path is read-only and echoes the recorded slug without validating against the spec list, per the scenario's explicit edge case for stale targets.
- **Reuse**: 3 SHOULD findings (see SHOULD section below). `count_open_questions` and `context_summary` duplicate section-traversal logic from `read_spec.rs`; the dashboard's `count_open_questions` doc comment even claims to "mirror" `read-spec`'s semantics, which is aspirational rather than factual (the two implementations diverge on nested-bullet handling). `is_feature_slug` is the only fresh-but-unique helper; defer extraction until a second caller exists.
- **Quality**: 0 MUST, 0 SHOULD, 2 low-confidence. The prior pass's actionable SHOULD (the unused `specs` parameter, cross-reported with Simplicity) has been resolved in `2d9116d`. The two low-confidence items remaining are a fixture-vs-unit-test coverage gap and a corner-case drift between `count_open_questions` and `read_spec::parse_open_questions` on pathological inputs (specs whose Open Questions section contains the literal placeholder mixed with other text).
- **Efficiency**: 1 SHOULD finding (informational). The walker is two-pass (load specs, then compute `blocked-by`), bounded by repo-committed spec count — operator-controlled, not user-controlled — well within interactive budget on realistic repos (~100 specs ≈ 100 file reads + ~400 stats). Not a `BE-INPUT-006` violation (rule scope is request bodies / page sizes / regex inputs, none of which apply). The two-pass design is intentional: `blocked-by` needs every spec's status known before any spec's `blocked-by` can be computed.
- **Simplicity**: 2 informational findings. The auto-fixable one (the unused parameter on `load_session_target`) is resolved in `2d9116d`. The two remaining (the `GovernConfig`/`ReviewConfig`/`DisabledRuleFile` three-struct shape, and the empty `DashboardArgs {}` struct) are both correct as-is — the TOML nesting requires the struct decomposition for clean serde, and the empty args struct is uniformity-driven and documented inline.

Test posture: 312 unit tests pass (15 new in `dashboard.rs::tests`); 5 integration tests; 3 atomic-writes tests; 15 MCP tests; 9 parity tests; 2 walker tests — 346 total green. `cargo clippy --release --all-targets -- -D warnings` clean. `cargo fmt --check` clean. `lint-procedure-parseability`, `lint-tool-coverage`, `lint-frontmatter`, `placeholder-roundtrip` (Family 4 of `scripts/audit/run-all.sh`), and `markdownlint-cli2` over the 022 spec dir + CHANGELOG + `framework/commands/status.md` all clean.

**Result**: 0 MUST, 5 SHOULD, 2 low-confidence. `blocking: no`. The spec is free to advance to `done`; the remaining SHOULD findings are advisory (reuse opportunities, plus two informational items the source code already documents as intentional).

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

### SHOULD: REUSE-001 — `count_open_questions` duplicates `read_spec::parse_open_questions` + `section_lines`

- **File**: `runtime/src/primitives/dashboard.rs:187-214`
- **Rule**: Reuse pass (no enumerated rule ID; project convention from `AGENTS.md` Boundaries and the implicit DRY norm). The dashboard's `count_open_questions` doc comment explicitly claims to "mirror `read-spec`'s open-question semantics" — this is the canonical "extract a helper" signal.
- **Finding**: `count_open_questions` re-implements section traversal (find `## Open Questions`, walk lines until next heading at same-or-shallower level, count top-level a `-` bullet items, treat the canonical placeholder as zero) that already lives in `read_spec.rs::parse_open_questions` + `section_lines` (lines 101-127 + 151-171). The two implementations have subtly different rules for continuation lines and nested bullets — the dashboard counts every a `-` bullet line in the section, while `read_spec` collapses blank-line-separated continuations into one question. Result is identical on canonical specs, but diverges on questions with nested sub-bullets or malformed bodies.
- **Auto-fixable**: no (requires a small refactor: extract `section_lines` into `primitives/mod.rs`, then have both consumers use it).
- **Suggested fix**: Extract `section_lines(body, heading)` from `read_spec.rs` into `primitives/mod.rs` as a `pub(crate)` helper. Reuse it from both `parse_open_questions` (read-spec) and `count_open_questions` (dashboard). The placeholder-stripping logic can become a small helper too. Drop the "mirrors `read-spec`'s open-question semantics" claim from the dashboard's doc comment once the implementations actually share the section-walk.

### SHOULD: REUSE-002 — `context_summary` walks ATX sections inline

- **File**: `runtime/src/primitives/dashboard.rs:358-381`
- **Rule**: Reuse pass (same convention as REUSE-001).
- **Finding**: `context_summary` walks the scenario body's `## Context` section to extract a first-non-blank-line summary, duplicating the section-iteration shape used by `read_spec.rs::section_lines`. Once REUSE-001's `section_lines` extraction lands, this function becomes a thin "first non-blank line" filter on top of the shared iterator.
- **Auto-fixable**: no.
- **Suggested fix**: Bundle with REUSE-001 — both findings dissolve once `section_lines` is shared.

### SHOULD: REUSE-003 — `is_feature_slug` lives inline; future callers will want a shared helper

- **File**: `runtime/src/primitives/dashboard.rs:114-121`
- **Rule**: Reuse pass.
- **Finding**: The hand-rolled `NNN-` pattern check is the only one of its kind in the runtime today (no duplicate to consolidate), but the pattern recurs across `framework/commands/*.md` and `scripts/audit/*.sh`. When a second Rust caller materializes (audit script ports, a future `list-specs` primitive shim, etc.) the function belongs in `primitives/mod.rs` alongside `validate_slug` / `validate_no_traversal`.
- **Auto-fixable**: no.
- **Suggested fix**: Defer. Promote to a shared helper when the second caller arrives.

### SHOULD: EFFICIENCY-001 — two-pass `load_specs` (informational)

- **File**: `runtime/src/primitives/dashboard.rs:57-107`
- **Rule**: Efficiency pass.
- **Finding**: `load_specs` walks `specs/` once to build the entry list, then makes a second pass over the list to compute each entry's `blocked-by` from the `status_by_slug` HashMap. Combined cost is bounded by the on-disk spec count (operator-committed, not user-supplied). On a 100-spec repo: ~100 file reads + ~400 stats — well within interactive budget. Not a `BE-INPUT-006` violation; the inefficiency is informational, not blocking.
- **Auto-fixable**: no.
- **Suggested fix**: None. The two-pass design is intentional: `blocked-by` requires every spec's status known before any spec's `blocked-by` can be computed. One-pass alternatives (lazy resolution, callback-based status lookup) would add complexity without measurable wall-clock benefit at realistic scales.

### SHOULD: SIMPLICITY-002 — `GovernConfig` / `ReviewConfig` / `DisabledRuleFile` shape (informational)

- **File**: `runtime/src/primitives/dashboard.rs:230-245`
- **Rule**: Simplicity pass.
- **Finding**: Three nested structs (`GovernConfig` → `ReviewConfig` → `DisabledRuleFile`) for what is functionally one path: `review.disabled-rule-files[].file`. Each layer is required by TOML's nested-table semantics — `GovernConfig` carries `[review]`, `ReviewConfig` carries `[[review.disabled-rule-files]]`, `DisabledRuleFile` carries the `file = "..."` entry.
- **Auto-fixable**: no.
- **Suggested fix**: None. Collapsing would require a custom `Deserialize` impl that's strictly more code than the current shape. The three-struct decomposition is the idiomatic serde pattern for nested TOML tables.

## Low-confidence findings

### Low-confidence: QUALITY-002 — fixture vs. unit-test coverage gap (confidence 75)

- **File**: `runtime/tests/fixtures/status-basic/specs/000-blocker/spec.md`
- **Finding**: The `000-blocker` fixture declares an unresolved Open Questions entry ("One unresolved item to give this spec a non-zero open-question count") used by the parity test at `runtime/tests/parity/status/expected.txt`, but the unit tests in `dashboard.rs::tests` (lines 407-671) use synthetic `TempDir` data for their assertions. The fixture exercises the open-question count via the parity path only — not redundant coverage, but a documentation-vs-test asymmetry.
- **Suggested fix**: None required. The parity test exists explicitly to exercise the fixture; the unit tests are designed to be standalone and synthetic. Low confidence because the "gap" may be intentional design rather than a defect.

### Low-confidence: QUALITY-003 — placeholder-strip semantic drift on malformed inputs (confidence 60)

- **File**: `runtime/src/primitives/dashboard.rs:187-214` vs. `runtime/src/primitives/read_spec.rs:101-127`
- **Finding**: On a malformed `## Open Questions` section that contains the literal `*None — all resolved.*` followed by additional text in the same line (e.g., `*None — all resolved.* — but actually here is one`), the two implementations may disagree on the count. Dashboard counts the leading a `-` bullet (if present) and then sees the placeholder; read_spec walks continuation lines into the current question. Practical impact is nil because canonical specs use the placeholder verbatim and on its own line. Confidence 60.
- **Suggested fix**: None required at current scope. Subsumed by REUSE-001's resolution.

## Waived findings

_None._

## Skipped passes

_None._
