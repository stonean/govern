---
spec: 022-deterministic-runtime
scenario: check-stuck-tasks-md-advancement
reviewed-at: 2026-05-18T02:30:00Z
reviewed-against: 109befe
diff-base: 109befe983c7298482bafe2f751e9da41c2d51d4
must-violations: 0
should-violations: 1
low-confidence: 0
skipped-passes: []
---

# Review — 022-deterministic-runtime (check-stuck-tasks-md-advancement scenario)

## Summary

Reviewed the task-30 implementation: `check-stuck`'s second-condition fix. The change adds 4 helper functions (`first_incomplete_index_unchanged`, `read_blob_at_head`, `read_blob_at_commit`, `read_blob_from_tree`, `first_incomplete_subtask_index`), modifies `run()` to require both `count >= threshold` AND the new condition, and adds one regression test. CHANGELOG entry and version bump (0.5.1 → 0.5.2). All edits under `runtime/src/primitives/check_stuck.rs` plus metadata files.

Stack: text-first markdown + Rust runtime. Loaded rule files: `configuration-cross.md` — none of its CFG-* triggers fire against the diff (no env-var lookups, operator-tunable constants, or shared cross-module values introduced). 0 MUST, 1 SHOULD (low-impact reuse opportunity in `find_in_progress_commit`), 0 low-confidence. `blocking: no`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

### SHOULD: REUSE-002 — `find_in_progress_commit` duplicates blob-read logic

- **File**: `runtime/src/primitives/check_stuck.rs:137-141` (the inline blob lookup inside `find_in_progress_commit`).
- **Rule**: AGENTS.md design principles + DRY — when a helper exists, prefer it over inline duplication.
- **Finding**: This diff adds `read_blob_from_tree(repo, tree, path)` as a clean helper, but the pre-existing `find_in_progress_commit` keeps its own inline version:

  ```rust
  let status = match tree.get_path(Path::new(spec_rel)).ok() {
      Some(entry) => {
          let blob = repo.find_blob(entry.id())?;
          extract_status(std::str::from_utf8(blob.content()).unwrap_or(""))
              .map(str::to_string)
      }
      None => None,
  };
  ```

  Could be:

  ```rust
  let status = read_blob_from_tree(repo, &tree, spec_rel)?
      .and_then(|c| extract_status(&c).map(str::to_string));
  ```

- **Auto-fixable**: yes (mechanical refactor; tests exercise the function so behavior is verifiable)
- **Suggested fix**: refactor `find_in_progress_commit` to use `read_blob_from_tree` for its tree-read. Drop the inline `unwrap_or("")` in favor of the helper's `Option<String>` return. Keep `extract_status` as-is.
- **Trade-off**: kept the inline version unchanged in this commit to avoid coupling the bug fix with a refactor — the existing inline code is correct and tested. The reuse cleanup is a follow-up commit; flagging here so it lands deliberately rather than accumulating.

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._

## Pass notes

### Security

No security rules apply at the framework level. The new code reads git blob content via libgit2's safe Rust bindings, parses UTF-8 with `from_utf8(...).ok()` (silently returns None on invalid encoding — defensible because tasks.md is required to be UTF-8 per the framework's text-first contract; a corrupted blob yields `stuck: false`, which is the safer default than panicking).

The `git2::Oid::from_str(sha)?` parse can fail if a malformed SHA is passed. Since `since` comes from `find_in_progress_commit`'s `oid.to_string()` (always 40-char hex), this is unreachable in practice. The `?` propagates as `PrimitiveError::Git` if it ever did fire — consistent with the rest of the file.

### Reuse

One SHOULD finding (REUSE-002 above) — `find_in_progress_commit` could use the new `read_blob_from_tree` helper to replace its inline blob lookup. Deferred to a follow-up commit per the "don't couple bug fix with refactor" preference.

`first_incomplete_subtask_index`'s fenced-code-block skip pattern mirrors `iter_task_numbers_at_levels` in `primitives/mod.rs`. The shared pattern could promote to a helper (`is_fence_open(line)` + `walk_outside_fences(content)` iterator); both consumers care about slightly different line semantics though, so the shared abstraction would need a closure parameter. Not worth the abstraction for two callers in v1; promote later if a third consumer surfaces.

### Quality

The vacuous-false cases match the scenario's edge case list:

- `tasks.md` missing at `since-sha` → `since_idx = None` → return `false` → `stuck = false`. ✓
- All subtasks complete at HEAD → `head_idx = None` → return `false` → `stuck = false` (completion is the opposite of stuck per scenario). ✓
- Phased structure → `first_incomplete_subtask_index` walks lines and matches `- [ ]` regardless of containing heading level. ✓
- No commits → count=0 → existing behavior, stuck=false. ✓

The new regression test (`stuck_false_when_checkboxes_flipped_across_threshold_commits`) exercises the false-positive case directly — 4 commits, each flipping a different subtask, asserts `stuck: false` with `commit_count: 4`. The test passes both before this change is reverted (sanity check) and after — the assertion is on the post-fix expectation.

All 5 existing tests still pass — each flips no checkboxes between commits so the new condition holds and `stuck` correctly fires when it should.

### Efficiency

Per `check-stuck` invocation, two extra git tree lookups (`since-sha` and HEAD) plus two content reads plus two line walks. Each operation is O(tasks.md size) and the tree lookups are constant in libgit2's object cache. Negligible compared to the existing commit walk for `count_commits_touching`. No findings.

### Simplicity

The `match (head_idx, since_idx)` exhaustive-on-Option pattern is clean. Three free helper functions match the existing module style (free functions, not methods). The new code is ~50 lines vs the existing 130 lines of `check_stuck.rs` — proportionate to the behavior change.

Considered alternative: thread `since_sha` and HEAD content through `count_commits_touching` to avoid the second tree walk. Rejected — `count_commits_touching` doesn't need the content; threading it would couple unrelated concerns. The current shape (one helper per concern) is simpler.

The position-based equality (per Q1 resolution) keeps the helper's logic linear: walk the file, return the first `- [ ]` line index. Heading-text equality would have required parsing the structure (which level-2/3 heading owns each subtask) and is appropriately deferred to v2.
