---
spec: 022-deterministic-runtime
scenario: writecode-payload-canonicalize-paths
reviewed-at: 2026-05-22T01:00:00Z
reviewed-against: b1a855e4478b6b4fbf61d03e1b1d7f1ab6c2ee2b
diff-base: ae011c180d737b31ec7895e0d2ccb8748ad1977a
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 022-deterministic-runtime (writecode-payload-canonicalize-paths scenario)

## Summary

Scenario review at HEAD `b1a855e`. The canonicalization-and-case-fold work in task #35 closes the BE-INPUT-004 SHOULD finding recorded in the prior whole-spec review at `2873aad`. `load_plan_relevant_files` now canonicalizes both `repo` and each candidate `abs` before reading, rejecting paths whose canonical form escapes `canon_repo` with `PayloadError::SecretExfiltration { pattern: "out-of-repo" }`. `secret_pattern` lowercases the basename before its glob checks, so `.ENV` / `Credentials.json` / `DB-Secrets.yaml` cannot bypass the guard on case-insensitive filesystems. Five scenarios are covered by tests: four new (relative escape, absolute escape, in-repo happy path, case-fold bypass) plus the pre-existing planned-new test that exercises the canonicalize-fails-continues branch.

Stack: text-first markdown + Rust runtime. Loaded rule files: `api-backend.md`, `configuration-cross.md`, `security-backend.md`. Frontend rule files skipped — no frontend surface. No `[[review.disabled-rule-files]]` entries.

Five-dimension review of the delta:

- **Security**: The fix is precisely the BE-INPUT-004 defense-in-depth tightening the prior review prescribed. The canonical-containment check operates on resolved paths, so symlink-to-outside is caught for free (canonicalize follows symlinks; the target's canonical form fails `starts_with(canon_repo)`). Case-fold normalization uses `to_ascii_lowercase` — sufficient because secret patterns are ASCII; non-ASCII case-fold edge cases (Cyrillic homoglyphs, full Unicode case folding) are out of scope since the on-disk patterns are themselves ASCII and the filesystem cannot resolve a non-ASCII spelling to an ASCII file. `repo.canonicalize()` failure returns `Ok(Vec::new())`, matching the pre-existing "no plan, no files" posture for `plan_path` not found. Threat model the fix closes (compromised `/gov:plan` LLM or malicious plan author bypassing PR review) is fully addressed by the structural canonical check rather than enumerable-pattern matching, which is the right tier of defense for this class.
- **Reuse**: The two new `match X.canonicalize() { ... }` blocks are isolated to one function (`load_plan_relevant_files`) and follow the existing `let Ok(...) = ... else { ... };` patterns already in the function. No abstraction opportunity — the canonical check is a single mechanism applied at one entry point, exactly where the rule mandates.
- **Quality**: Error propagation is consistent — the new `"out-of-repo"` pattern label rides the existing `PayloadError::SecretExfiltration` variant rather than introducing a new error type, so callers and the `error` envelope's `code` field stay stable. The `to_ascii_lowercase()` call returns an owned `String`, which the function captures by binding `basename` to the owned value; subsequent `starts_with` / `==` / `rsplit_once` calls operate on `&str` correctly. Doc-comment on `PayloadError::SecretExfiltration` was updated to enumerate the new label, matching the function's new behavior. The atomic-write semantics of state-modifying primitives are not touched — this fix is a pure read-side guard.
- **Efficiency**: Each plan entry now performs one additional `canonicalize()` syscall (stat-equivalent). For plans with ~10 entries (the typical shape) the overhead is microseconds and dwarfed by the subsequent `read_to_string` cost. No N+1; no unbounded loop introduced.
- **Simplicity**: The diff is ~20 lines of behavior change in `load_plan_relevant_files` + ~80 lines of tests + 1 line in `secret_pattern` + 6 lines of doc-comment updates. No new module, no new trait, no new abstraction. The function reads top-to-bottom in the same shape as before, just with one extra check per entry.

Test posture: 296 unit tests pass (up from 268 at the prior review; the dep-major refresh and four new canonicalization tests account for the delta). `cargo clippy --release --all-targets -- -D warnings` clean. `cargo fmt --check` clean. `lint-procedure-parseability`, `lint-tool-coverage`, `lint-frontmatter`, `markdownlint-cli2` over the 022 spec dir + CHANGELOG all clean.

**Result**: 0 MUST, 0 SHOULD, 0 low-confidence. `blocking: no`. The prior whole-spec SHOULD has been closed by this work.

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
