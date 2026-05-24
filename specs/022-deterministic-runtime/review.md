---
spec: 022-deterministic-runtime
scenario: traverse-deps-cycle-check
reviewed-at: 2026-05-24T02:24:13Z
reviewed-against: a643e4141d6135690887df2bb26cded8d7552561
diff-base: a643e4141d6135690887df2bb26cded8d7552561
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 022-deterministic-runtime (traverse-deps-cycle-check scenario)

## Summary

Scenario review at HEAD `a643e41` (back-edge transition `done → in-progress` plus the scenario implementation are uncommitted; diff base is therefore HEAD and the review walks the working tree). Stack: Rust runtime + text-first markdown. Loaded rule files: `api-backend.md`, `configuration-cross.md`, `security-backend.md`. Frontend rule files skipped — no frontend surface. No `[[review.disabled-rule-files]]` entries.

The scenario adds graph-level cycle detection to `traverse-deps`, complementing spec 017's commit-time `gen-spec-deps.sh` check as defense-in-depth. Delta is contained:

- `runtime/src/primitives/traverse_deps.rs` — reachable-subgraph walker plus Tarjan's SCC, eight new edge-case unit tests covering the scenario's five edge cases plus a 3-node hop.
- `runtime/src/schema/primitives.rs` — additive `cycles: Vec<Vec<String>>` field on `TraverseDepsResult` (`#[serde(default)]`), cycle-bearing round-trip assertion.
- `runtime/tests/mcp.rs` — `traverse_deps_surfaces_two_cycle_via_mcp` plus an empty-cycles assertion on the existing acyclic MCP test.
- `runtime/tests/parity.rs` — `traverse_deps_cycle_check_surfaces_two_cycle_via_cli` exercising the CLI subprocess surface against a hand-built 2-cycle fixture.
- `framework/commands/analyze.md` step 3 prose — names cycles as blocking and explains the defense-in-depth relationship with spec 017's generator-side check.
- `runtime/Cargo.toml`, `runtime/CHANGELOG.md`, `runtime/Cargo.lock` — version bump `0.9.0 → 0.9.1` (patch — defense-in-depth fix, additive schema field; mirrors the 0.7.4 / 0.8.1 patch-for-fix precedent).

Five-dimension review of the delta:

- **Security**: 0 findings. The cycle walker reads frontmatter via the same `read_text` / `split_frontmatter` / `serde_yaml::from_str::<Frontmatter>` chain already used by `read_status` and inherited from every other primitive; `serde_yaml` is a data-only parser with no side-effect deserialization (`BE-INPUT-008` satisfied — same posture as every prior `traverse-deps` review). Path construction at `repo.join("specs").join(node).join("spec.md")` mirrors the existing primitive's pattern that the dashboard / write-session / spec-022 base reviews already cleared against `BE-INPUT-004` — `node` flows from operator-committed frontmatter, not from client input, so the path-traversal threat model does not apply (the `is_feature_slug` guard on the dashboard primitive enforces the same boundary on a different surface). `read_dependencies` swallows IO and YAML errors by returning an empty `Vec`, which is the documented contract for graph degradation to a sink node — not silent error suppression of operationally-meaningful state (the focal-spec read path through `run()` still surfaces these errors as `PrimitiveError::Io` / `PrimitiveError::Yaml` envelopes). No new auth, network, logging, crypto, error-response shaping, or PII surface; no rule from `BE-AUTHN`/`BE-AUTHZ`/`BE-DATA`/`BE-API`/`BE-ERR` triggers against this delta.

- **Reuse**: 0 findings. The local `read_dependencies` helper sits alongside the pre-existing `read_status` helper; both call `read_text → split_frontmatter → serde_yaml::from_str::<Frontmatter>` then extract one field. The duplication is intentional and small (~5 lines): `read_status` propagates errors via `Result<String>` because the focal-spec read must halt the primitive on YAML failure, while `read_dependencies` swallows errors to `Vec<String>` because graph-walker nodes degrade to sinks. Collapsing them would require either a flag parameter (more lines than the current shape) or a generic `read_one_field::<F>` over a `Frontmatter -> F` closure (premature abstraction for two call sites with different error semantics). The `section_lines` / `is_feature_slug` / `validate_slug` helpers in `primitives/mod.rs` are not applicable: `section_lines` traverses body markdown (frontmatter-only here); `is_feature_slug` is a slug-shape validator for the dashboard primitive's specs/ directory walk (this walker reads frontmatter the operator committed and is already paired with `gen-spec-deps.sh`'s commit-time slug check, so re-validating is duplicative). Tarjan's SCC is implemented inline rather than pulling in `petgraph` — adding a graph crate for ~50 lines of well-trodden algorithm on graphs bounded by spec count is over-investment; the recursive form is small enough that lifting it into its own module would obscure rather than clarify.

- **Quality**: 0 findings. Tarjan's algorithm correctly classifies single-node SCCs: the `scc.len() == 1 && adj[v].contains(&v)` check at `traverse_deps.rs:138-142` separates isolated nodes (no cycle) from self-loops (1-cycle) — covered by `self_cycle_is_reported_as_one_cycle`. The `index_of` early-return in `visit` (`traverse_deps.rs:157-159`) deduplicates the BFS-style walk so revisits don't double-add nodes to `order`; combined with the post-visit `adj[idx].push(dep_idx)`, this correctly records every directed edge even when the dep was already discovered through a different path (covered by `three_node_cycle_via_intermediate_node`). The missing-dep edge case (`missing_node_does_not_close_a_cycle`) confirms the documented "absent closing edge means no cycle" contract — the missing node is added to `order` and gets an empty `adj[idx]` because `read_dependencies` returns `Vec::new()` for the absent file. `read_dependencies`'s error-swallowing is sound by design and not silent: the focal spec's read still propagates IO / YAML errors via `PrimitiveError` (verified by the existing `dependent_resolves_basic_edge` and `missing_dependency_is_incompatible` happy-path tests staying green). The `cycles` field is `Vec<Vec<String>>` rather than `Option<...>` because an empty Vec is the natural "no cycles" sentinel and round-trips through serde with `#[serde(default)]` for older consumers — confirmed by the cycle-bearing round-trip assertion added to `schema::primitives::tests::traverse_deps_round_trip`. All 374 tests green; clippy --release --all-targets -- -D warnings clean; fmt --check clean.

- **Efficiency**: 0 findings. The walker performs one file read per reachable spec (frontmatter parse), then a single Tarjan's pass over the resulting subgraph. For realistic repos (~30 specs, ~50 edges) this is a sub-millisecond pass within the existing `/gov:analyze` budget; the marginal cost over the pre-cycle-detection primitive is reading transitively-reachable specs' frontmatter, which the dashboard primitive already does for the full corpus on every `/gov:status` invocation. The graph is operator-committed, not user-controlled, so `BE-INPUT-006` (input upper bounds) does not apply — there is no DoS surface (the existing primitive already reads each direct dep's frontmatter for status checking; the new code extends to transitive reads). `read_dependencies` returns owned `Vec<String>` rather than borrowing from the file content; the allocations are bounded by the dependency count of each visited spec (typically <5 edges per spec, <200 allocations total on realistic repos). No N+1 pattern (no database), no unbounded loop over user-controlled input.

- **Simplicity**: 0 findings. Tarjan's SCC is the canonical algorithm for "report every cycle in a directed graph"; the recursive form lives in a single ~50-line `Tarjan` struct with the standard `indices` / `lowlinks` / `on_stack` / `stack` / `counter` / `sccs` state. The post-pop `component.reverse()` (`traverse_deps.rs:235`) is documented inline as a deliberate choice to surface slugs in traversal order rather than pop order, matching the scenario's "slugs in traversal order, one entry per cycle" requirement. The `else if … && let Some(w_index) = …` let-chain (`traverse_deps.rs:222-224`) is idiomatic Rust 1.88+ (MSRV is 1.88 per `Cargo.toml`); clippy's `collapsible_if` lint required this exact form. No premature abstraction (`Tarjan` is a struct rather than a free-function only because it carries enough mutable state that threading every field through arguments would be noisier); no dead branches; no configuration that should be a constant.

Test posture: 338 unit + 3 atomic-writes + 5 exec_subprocess + 16 MCP + 10 parity + 2 walker = 374 tests green (up from 352 at the prior `e4b42f1` review; the deltas are the 8 new traverse-deps unit tests + acyclic-empty-cycles assertion on the existing MCP test + 1 new MCP cycle test + 1 new parity cycle test, with a 12-test net increase coming partly from intervening commits between `e4b42f1` and `a643e41`). `cargo clippy --release --all-targets -- -D warnings` clean. `cargo fmt --check` clean. `lint-procedure-parseability`, `lint-tool-coverage`, `lint-frontmatter`, `gen-spec-deps --dry-run` all exit 0. `markdownlint-cli2` on the 022 spec dir + CHANGELOG + `framework/commands/analyze.md` reports 2 errors, both pre-existing on `main` at `analyze.md:60` and `analyze.md:63` (MD029 ordered-list prefix; the `10.` / `11.` step numbers under `1/1/1` style) — verified by `git stash && markdownlint analyze.md` on the unmodified file; not introduced by this scenario's edits.

**Result**: 0 MUST, 0 SHOULD, 0 low-confidence. `blocking: no`. The scenario is free to advance the spec back to `done` once `/gov:implement`'s done-transition gate runs.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._
