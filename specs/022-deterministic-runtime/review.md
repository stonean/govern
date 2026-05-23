---
spec: 022-deterministic-runtime
scenario: dashboard-primitive
reviewed-at: 2026-05-23T14:01:20Z
reviewed-against: e4b42f1a412571825edf982f6609d8ae84abc163
diff-base: 0737133
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 022-deterministic-runtime (dashboard-primitive scenario)

## Summary

Scenario review at HEAD `e4b42f1`. This is the third pass on the `dashboard` primitive (commits `0737133..e4b42f1`). Stack: text-first markdown + Rust runtime. Loaded rule files: `api-backend.md`, `configuration-cross.md`, `security-backend.md`. Frontend rule files skipped — no frontend surface. No `[[review.disabled-rule-files]]` entries.

The prior passes' findings — one auto-fixable SIMPLICITY/QUALITY SHOULD (unused `specs` parameter on `load_session_target`) and three REUSE SHOULDs (`section_lines` and `is_feature_slug` duplication between `dashboard.rs` and `read_spec.rs`) — have all been resolved:

- `2d9116d` removed the unused `&[DashboardSpec]` parameter from `load_session_target`.
- `e4b42f1` extracted `section_lines` into `primitives/mod.rs` as `pub(crate)`. Both `read_spec::parse_open_questions` and `dashboard::{count_open_questions, context_summary}` now share the iteration helper; the placeholder-strip drift the prior pass flagged as low-confidence is closed by construction.
- `e4b42f1` also promoted `is_feature_slug` to `primitives/mod.rs` alongside `validate_slug` and `validate_no_traversal`. Currently one caller, but the pattern recurs elsewhere; preemptive promotion is cheap.

Six new unit tests in `primitives::tests` cover the extracted helpers directly (section_lines body-until-sibling, absent-heading, deeper-nested-as-body, repeated-heading; is_feature_slug canonical form and non-pattern rejection).

Five-dimension review of the current delta:

- **Security**: 0 findings. Hard-coded-path read-only walker (`DashboardArgs` is `{}`); structured `PrimitiveError::{Yaml, Toml, Json}` variants on parse failure; safe-by-default deserialization for all three formats; `.govern.toml` `reason` field correctly excluded from payload per the scenario's contract; session-target echoed as-recorded per the scenario's stale-target edge case. No security rules apply because every potential surface (`BE-INPUT-001`, `BE-INPUT-002`, `BE-INPUT-004`, `BE-INPUT-006`, `BE-INPUT-008`, `BE-INPUT-011`, `BE-ERR-001`, `BE-LOG-002`) is either inapplicable (no client input, no logging, no error response shaping) or already satisfied via the inherited primitive scaffolding.
- **Reuse**: 0 findings. All three prior REUSE SHOULDs are closed by `e4b42f1`. The `section_lines` and `is_feature_slug` extractions remove the section-traversal duplication between `dashboard.rs` and `read_spec.rs`, with the iteration semantics now living in one place. `count_open_questions` (dashboard) and `parse_open_questions` (read-spec) traverse identically; they differ only in result shape (`u32` count vs. `Vec<OpenQuestion>`).
- **Quality**: 0 findings. `is_feature_slug` correctly accepts `NNN-` and rejects everything else (test coverage in `primitives::tests::is_feature_slug_*`). The shared `section_lines` traversal is exercised by `read_spec`'s and `dashboard`'s consumer tests as well as four new direct tests. The `blocked-by` computation handles nonexistent dependency slugs by returning empty status (test at line 532-544 in dashboard.rs); the scenario-detail reader handles missing `## Context` cleanly. No off-by-one errors in the heading-level comparisons. Prior pass's QUALITY-003 (placeholder-strip drift between dashboard and read-spec on pathological inputs) is closed by the shared traversal.
- **Efficiency**: 0 findings. `load_specs` walks `specs/` once to build the entry list, then makes a second O(N) pass over the list to compute each entry's `blocked-by` from the `status_by_slug` HashMap. The two-pass design is intentional — `blocked-by` requires every spec's status known before any spec's `blocked-by` can be computed — and the combined cost is bounded by repo-committed spec count (operator-controlled, not user-controlled). On realistic repos (~100 specs) the work is ~100 file reads + ~400 stats, well within interactive budget. Not an N+1, not an unbounded user-controlled loop; nothing flagged by `BE-INPUT-006` applies. Prior pass classified this as a SHOULD with "suggested fix: None" — this re-review honestly reclassifies it as a Design note (see below), not a violation. See `framework/rules/security-backend.md` `BE-INPUT-006` and `framework/rules/configuration-cross.md` for the criteria that DON'T apply.
- **Simplicity**: 0 findings. The auto-fixable SIMPLICITY-001 (unused parameter) was resolved in `2d9116d`. The remaining items the prior pass flagged — the `GovernConfig`/`ReviewConfig`/`DisabledRuleFile` three-struct shape and the empty `DashboardArgs {}` struct — are both correct as-is and the prior pass acknowledged this in their "Suggested fix: None" entries. Idiomatic serde for nested TOML tables; clap-derive uniformity for the no-args primitive. Honestly reclassified as Design notes (see below), not violations.

Test posture: 318 unit + 5 integration + 3 atomic-writes + 15 MCP + 9 parity + 2 walker = 352 tests green (up from 346 on `2d9116d` and 312 on `c15ae0e`). `cargo clippy --release --all-targets -- -D warnings` clean. `cargo fmt --check` clean. `lint-procedure-parseability`, `lint-tool-coverage`, `lint-frontmatter`, `placeholder-roundtrip` (Family 4 of `scripts/audit/run-all.sh`), and `markdownlint-cli2` over the 022 spec dir + CHANGELOG + `framework/commands/status.md` all clean.

**Result**: 0 MUST, 0 SHOULD, 0 low-confidence. `blocking: no`. The spec is free to advance to `done`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Design notes (intentional, not violations)

These observations from the prior review (`reviewed-at: 2026-05-23T13:53:14Z`) were classified as SHOULDs but each had "Suggested fix: None" — they describe intentional design, not violations. This pass reclassifies them honestly as design notes for traceability:

### Two-pass `load_specs` is intentional

- **File**: `runtime/src/primitives/dashboard.rs:57-107`
- **Note**: `load_specs` walks `specs/` once to build the entry list, then again to fill in `blocked-by`. The two-pass design is mandated by the dependency-graph semantics — `blocked-by` cannot be computed for any spec until every spec's status is known. Combined cost is O(N) where N is the operator-committed spec count; ~100 file reads + ~400 stats on realistic repos. Not an N+1, not user-controlled, not a DoS surface.

### `GovernConfig` / `ReviewConfig` / `DisabledRuleFile` three-struct shape is required by serde

- **File**: `runtime/src/primitives/dashboard.rs:230-245`
- **Note**: The three-struct decomposition (`GovernConfig` → `ReviewConfig` → `DisabledRuleFile`) mirrors the TOML's nested-table structure: `[review]` is the section, `[[review.disabled-rule-files]]` is the array-of-tables, each entry has `file = "..."`. Collapsing the decomposition would require a custom `Deserialize` impl that is strictly more code. The current shape is idiomatic serde for nested TOML.

### Empty `DashboardArgs {}` struct is required by clap-derive uniformity

- **File**: `runtime/src/schema/primitives.rs:406-408`
- **Note**: `DashboardArgs` is an empty unit struct preserved so the primitive's `run(&args, repo)` shape matches every other primitive. The CLI surface auto-derives a no-flag subcommand; the MCP surface accepts an empty object payload. Documented inline in the struct's doc comment.

## Waived findings

_None._

## Skipped passes

_None._
