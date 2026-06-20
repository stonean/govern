---
spec: 022-deterministic-runtime
reviewed-at: 2026-06-20T18:12:31Z
reviewed-against: 5fb4be3
diff-base: 5fb4be3
must-violations: 0
should-violations: 1
low-confidence: 2
captured-issues: 0
skipped-passes: []
notes: "Scoped to the uncommitted gvrn 0.13.0 runtime changes (git diff HEAD -- runtime/), layered on 5fb4be3 — the code under review is the working tree, not a committed sha. Supersedes the prior commands-dir-parameterization scenario review."
---

# Review — 022-deterministic-runtime

## Summary

Scoped to the uncommitted gvrn 0.13.0 runtime changes (`git diff HEAD -- runtime/src runtime/tests`, layered on `5fb4be3`): the `merge-managed-block` group-alignment rewrite, the `cli-config-dir` relocation (`host.rs` read order, `write-session` merge-writer, `dashboard` optional feature), and OpenCode command resolution (`command_file_candidates`). **No MUST violations — not blocking.** The headline behaviors verified correct: the merge-writer preserves `cli-config-dir` on a target write and the target block on a host-config write; `host.rs` precedence (session → legacy `[host]` → default) is sound; `command_file_candidates` plural-first ordering is backward-compatible; the two-pointer walk cannot infinite-loop or panic on byte offsets / multibyte UTF-8 (slicing only at `\n`/string-end; `ci` strictly increases, bounded by `canon.len()`). No security-rule violations: `cli-config-dir` is config-file input (not a boundary request), resolved paths are `repo.join`ed and existence-checked, TOML parse errors are handled, and the atomic tempfile+rename write is preserved. One advisory SHOULD (a pre-existing, documented `walk_body_extent` edge case) and two low-confidence findings remain; one low-confidence simplicity finding (dead `Default` derives) was fixed inline during the review.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

### SHOULD: none — `walk_body_extent` consumes adopter content when a canonical subsection is appended at the very end

- **File**: `runtime/src/primitives/merge_managed_block.rs:358-388` (`walk_body_extent`)
- **Rule**: none (correctness — data-loss edge case; maps to no rule ID)
- **Finding**: When the new canonical **appends** a subsection at the _end_ of the managed block AND the adopter has their own content immediately after the block, the trailing adopter group shares no pattern with the leftover canonical group and there is no _later_ canonical group to realign against, so it falls into the "full rewrite" branch (`ci += 1`) and is consumed into `block_end` — then silently dropped when `merge_line_prefix` replaces the block. The mid-insert direction (what the shipped gitignore template actually does — agents are inserted before the stable `# IDE`/`# OS` trailer) is handled correctly, which is why the regression test passes.
- **Severity rationale**: Advisory, not blocking, because (a) it is **pre-existing** — the prior line-shape walk had the same behavior; this rewrite neither introduced nor worsened it; (b) it is **documented** as a known edge case in [`scenarios/merge-managed-block-subsection-insertion.md`](scenarios/merge-managed-block-subsection-insertion.md) ("Subsection appended at the end + adopter content follows"); and (c) it is **not reachable through current framework usage** (the template never appends after adopter territory).
- **Auto-fixable**: no
- **Suggested fix**: The instinctive fix (break instead of consume when `ci` is the last canonical group and the on-disk group matches nothing) is **unsafe** — it would break the full-replacement case (`line_prefix_updates_in_place_preserving_surrounding_content`: `.old/` → `.claude/`), which is locally indistinguishable from an adopter tail. The sound fix is to give the line-prefix managed block an explicit END sentinel (a valid `#`-comment terminator) so the block is unambiguously bounded regardless of structural change. That is a larger design change requiring a one-time migration for existing adopters and is deferred — tracked as the END-sentinel direction in the subsection-insertion scenario.

## Low-confidence findings

### quality (confidence 70) — removed middle subsection left as orphan adopter content

- **File**: `runtime/src/primitives/merge_managed_block.rs:358-388`
- **Finding**: If a future canonical _removes_ a middle subsection the on-disk block still has, that subsection is left below the managed block as orphan content rather than dropped. Low impact: the shipped template only ever adds agent subsections, never removes. Documented as the "Subsection removed between runs" edge case in the subsection-insertion scenario.
- **Auto-fixable**: no

### reuse (confidence 55) — residual candidate-list duplication across the two resolution callsites

- **File**: `runtime/src/main.rs:211-230`, `runtime/src/interpreter/payload.rs:255-270`
- **Finding**: Both callsites assemble the same 3-segment candidate list (`framework/commands/<name>.md`, then `host.command_file_candidates(...)`, then `framework/bootstrap/<name>.md`), differing only in `PathBuf`/`.exists()` vs `String`/`.is_file()`. The _variable_ middle is correctly centralized in the new helper (the real reuse win); only the two fixed framework segments and the ordering remain duplicated. Not worth the churn unless these callsites are touched again.
- **Auto-fixable**: no

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no additions to `specs/inbox.md` in the review window._

## Skipped passes

_None — all five passes (security, reuse, quality, efficiency, simplicity) ran._

## Applied during review

- **simplicity (was low-confidence): dead `Default` derives removed.** `SessionHost` (`runtime/src/host.rs`) and `SessionFile` (`runtime/src/primitives/dashboard.rs`) derived `Default` but never used it (both deserialize via `.ok()?` / `map_err`, not `default()`). Removed both derives; `ExistingSession`'s `Default` is genuinely used (`unwrap_or_default`) and was left intact. Build, clippy (`-D warnings`), and the host/dashboard unit tests stay green.
