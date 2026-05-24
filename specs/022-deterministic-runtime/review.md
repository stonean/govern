---
spec: 022-deterministic-runtime
scenario: merge-managed-block-multi-subsection-end
reviewed-at: 2026-05-24T03:14:50Z
reviewed-against: 75fbc6dd3895ba6f23207a7947a4a98ce7963f0a
diff-base: 8e616453b3d5d40fc0a9acdf0e3fa1db4c6f78ec
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 022-deterministic-runtime (merge-managed-block-multi-subsection-end scenario)

## Summary

Scenario review at HEAD `75fbc6d` (diff base `8e61645` — the commit immediately before the scenario landed; the spec's `done → in-progress` back-edge and the implementation commit are inside the diff window). Stack: Rust runtime + text-first markdown. Loaded rule files: `api-backend.md`, `configuration-cross.md`, `security-backend.md`. Frontend rule files skipped — no frontend surface. No `[[review.disabled-rule-files]]` entries.

The scenario fixes a `merge-managed-block` (line-prefix style) end-of-block detection bug that caused `/govern` reruns to accumulate orphan subsection-header comments below the managed region whenever the canonical block contained interior blank lines (the shipped `.gitignore` template is shaped this way). Delta is contained:

- `runtime/src/primitives/merge_managed_block.rs` — `find_line_prefix_block` gains an `expected_block: &str` parameter; the next-blank `find_blank_line` helper is replaced with a new `walk_body_extent` that walks up to `block.lines().count()` lines from the marker using the supplied block as a *structural* template (expected blanks must align with on-disk blanks; an unexpected blank — non-blank expected, blank actual — signals the end-of-block terminator). Two new unit tests cover the stable-rerun (`unchanged`, mtime preserved) and same-structure-content-change (clean replacement, each subsection header appears exactly once) paths. The stale unchanged-arm comment is removed; the module-level doc paragraph for the `line-prefix` style is rewritten to describe the new walk.
- `runtime/CHANGELOG.md`, `runtime/Cargo.toml`, `runtime/Cargo.lock` — version bump `0.9.1 → 0.9.2` (patch — bug fix, no schema or wire-protocol change; mirrors the 0.5.2 / 0.7.3 / 0.8.1 patch-for-fix precedent).
- `specs/022-deterministic-runtime/scenarios/merge-managed-block-multi-subsection-end.md` — new scenario authoring the contract: structural-template walk; stable-rerun, same-structure update, single-subsection, grow, shrink/divergent-structure, adopter-edit, last-line-blank, marker-as-tail-comment edge cases.
- `specs/022-deterministic-runtime/spec.md` — `status: done → in-progress` (back-edge for the new scenario; review block left as-is per the project's reopen convention — it's refreshed by this run).
- `specs/022-deterministic-runtime/tasks.md` — new `## 39.` task entry pointing at the scenario; the single subtask was flipped to `[x]` by `mark-task` after the implementation landed.
- `README.md` — single-line touch by the help-table generator (auto-staged by the pre-commit hook; no human edit).

Five-dimension review of the delta:

- **Security**: 0 findings. The diff is pure internal byte-string walking inside a primitive that operates on framework-managed files in a deterministic runtime context. No new I/O, no network egress, no authentication, no authorization, no parsing of untrusted input across a system boundary, no logging, no error responses, no schema, no new dependencies. Every BE-* rule category was considered and found N/A: BE-AUTHN / BE-AUTHZ (no auth surface); BE-INPUT-001..015 (the `text` argument flows from `read_text(&path)` in `run()` against a framework-managed file path; the `marker` and `expected_block` arguments are caller-supplied within the runtime; no SQL/NoSQL/shell/template/LDAP/XML interpreters; no path construction in changed code; no URL fetch; no deserialization beyond the existing `serde_yaml` consumers untouched by this diff); BE-DATA / BE-API (no PII, no API surface); BE-ERR (the new helper returns `usize` directly — error propagation is the caller's contract, unchanged by this diff); BE-LOG (no log emission); BE-DEPS (no new dependencies — `Cargo.lock` delta is just the version bump). The structural-template walk's iteration bound is `expected_block.lines().count()`, which is caller-controlled within the runtime — not crossing an external trust boundary — so BE-INPUT-006 (resource-consumption bounds) does not trigger; this is the same posture as every other `merge_managed_block` helper that walks `text` line by line.
- **Reuse**: 0 findings. The new `walk_body_extent` shares a shape with the line-walking loops in `find_line_prefix_block` (marker discovery), `merge_html_comment` (no — uses `str::find`), and `dedup_outside_block` (line iteration over the post-merge content). Each call site has subtly different needs — `dedup_outside_block` needs raw + trimmed line + `has_newline` separately to reconstruct output; `find_line_prefix_block` needs equality against the header string at each line; `walk_body_extent` needs only blank-vs-non-blank classification and offset advancement. Extracting a single line-iterator helper would be premature DRYing — the per-call subset of "advance offset / detect line end / strip CR" is only 4 lines and inlining preserves the locality each site needs. Verdict: keep as-is.
- **Quality**: 0 findings. The behavior contract is explicit (`scenarios/merge-managed-block-multi-subsection-end.md`'s Behavior and Edge Cases sections), and both passing tests (`line_prefix_multi_subsection_rerun_is_unchanged_and_preserves_mtime` and `line_prefix_multi_subsection_update_replaces_cleanly_without_duplicated_tail`) lock the two cases the scenario commits to. Walked edge-case correctness against the existing test surface: CRLF handling preserved (`line_prefix_block_with_crlf_line_endings_in_existing_file` passes — `walk_body_extent`'s `trim_end_matches('\r')` on `actual_line` matches the legacy helper's behavior); single-subsection canonicals preserved (every existing single-subsection test still passes — when no interior blanks exist, the walk consumes all `block.lines().count()` lines and produces the same body_end the legacy helper computed); empty-block case (`block` normalized to empty string → `expected_block.lines().count() == 0` → walk consumes zero lines → body_end = body_start, body slice empty, body == block, `unchanged` — consistent with the legacy behavior on the same input); EOF before all expected lines consumed (the loop's `body_offset >= text.len()` guard breaks cleanly, body_end = text.len(), body != block, update path runs). The divergent-structure case (where the supplied block's blank-line positions don't align with the on-disk block's) is explicitly out of scope per the scenario's edge-case bullet — the failure mode is locally visible (orphan headers in the diff) and the operator recovers by hand. All 27 `merge_managed_block::tests` pass; full runtime suite (376 tests) passes; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean.
- **Efficiency**: 0 findings. Worst-case complexity of `walk_body_extent` is O(text.len() × expected_block.lines().count()) — each iteration does a `rest.find('\n')` linear scan from `body_offset` to the next newline. The legacy `find_blank_line` had the same per-line shape (linear scan to next newline, no whole-text rescans), so the asymptotic cost is unchanged. For realistic .gitignore files (text under 10 KB, blocks under 100 lines), this is sub-millisecond and never on a hot path — `merge-managed-block` runs once per `/govern` invocation against the `.gitignore` only.
- **Simplicity**: 0 findings. Net line-count delta in the production helpers is small (the new `walk_body_extent` is 18 lines; the removed `find_blank_line` was 14 lines). The structural-template invariant has a one-line statement (`if !expected_line.is_empty() && actual_line.is_empty() { break; }`) accompanied by a doc-comment paragraph explaining the WHY (the previous-run terminator signal) — matches the constitution's guidance for comments. The unchanged-arm stale comment that asserted "body has no interior blanks" was correctly removed (its assumption no longer holds and keeping it would mislead future readers). The `find_line_prefix_block` signature change (added `expected_block` parameter) is the minimum-viable plumbing; there is one call site, updated in the same diff.

No `--fix` action applied (the `/gov:review` invocation in this turn ran without `--fix`).

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

*None.*

## Waived findings

*None.*

## Skipped passes

*None.*
