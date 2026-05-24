---
section: "Follow-on scenarios"
---

# Merge-managed-block-multi-subsection-end

## Context

The `merge-managed-block` primitive (line-prefix style) does not reliably detect the end of an existing canonical block when that block contains interior blank lines between subsections. The `.gitignore` template shipped by `/govern` ([framework/templates/project/gitignore](../../../framework/templates/project/gitignore)) is exactly this shape: five subsections (`# Environment and secrets`, `# Claude Code local settings`, `# govern derived views`, `# IDE`, `# OS`), each separated by a blank line. Every `/govern` invocation against an adopter project re-walks the same canonical block; each run leaves a fresh trail of orphan subsection headers below the real block. Over time the file accumulates a tail like:

```text
# Claude Code local settings (keep commands tracked for project-wide access)

# govern derived views — non-markdown caches and indexes generated from specs.
# Markdown artifacts (specs, plans, scenarios) stay in git as the source of truth;
# binary or machine-only views regenerated from them do not. See constitution
# §text-first-artifacts.

# IDE

# OS
```

— canonical-block subsection headers, comment-shaped, with their list bodies dedup'd away. The accumulation reproduces on every adopter that ran `/govern` against a template with multi-subsection blocks.

Root cause is local to [`runtime/src/primitives/merge_managed_block.rs`](../../../runtime/src/primitives/merge_managed_block.rs):

1. `find_line_prefix_block` locates the `# {marker}` line and returns `body_end` as "the next blank line." When the canonical block contains an interior blank, `body_end` is the *first interior blank within the canonical block*, not the block's actual terminator.
2. `merge_line_prefix`'s `Some(...)` arm compares the returned `body` (truncated to the first subsection) against the supplied `block` (the full multi-subsection canonical). The comparison can never succeed for multi-subsection canonicals, so the `unchanged` branch is unreachable — the file is rewritten on every run.
3. The `updated` branch computes `after = &text[body_end..]` using the same wrong `body_end`. `after` therefore contains the rest of the on-disk canonical block (subsections 2..N), which gets concatenated below the freshly written full block.
4. The dedup pass strips non-blank, non-comment adopter-area duplicates, but explicitly preserves comment lines (`# foo`). The subsection headers in the duplicated tail are comments, so they survive — producing the orphan-empty-header trail.

The companion `unchanged`-arm comment already calls out that `find_line_prefix_block`'s "next blank" heuristic only matches when the canonical block has no interior blanks; the `updated` arm inherited the same assumption without the safeguard.

The accompanying regression test `line_prefix_preserves_multi_subsection_block_with_interior_blank_lines` covers the *insert* path (first run, no marker present). It does not cover the *update* path (subsequent runs with the marker present), which is where the bug lives.

## Behavior

- `merge-managed-block` (line-prefix style) MUST identify the existing on-disk canonical block by walking the supplied `block` as a *structural template*, not by finding the next blank line. Specifically: from the position past the marker line, consume up to `block.lines().count()` on-disk lines; an expected blank line (interior subsection separator in the supplied block) matches against an on-disk blank; an expected non-blank line may match any non-blank on-disk content. Terminate early when the expected line is non-blank but the on-disk line is blank — that blank is the end-of-block terminator the previous run wrote.
- When the on-disk body span (the structural-template walk) byte-equals the supplied `block`, the primitive emits `unchanged` and does not rewrite — preserving mtime and idempotency. Multi-subsection canonicals reach `unchanged` on stable reruns, the same way single-subsection canonicals already do.
- When the on-disk body differs from the supplied `block`, the primitive replaces exactly that span and computes `after` from the byte offset immediately following the last consumed body line (including its terminating newline). The post-merge file contains the new managed region followed by the adopter content that originally lived past the block, with no duplicated subsections — provided the on-disk block shared the supplied block's structure (see Edge Cases for the structural-divergence path).
- The `find_line_prefix_block` helper takes the supplied block as an additional parameter; the helper-internal `walk_body_extent` implements the structural-template walk. The previously-used `find_blank_line` helper is removed.
- The cross-boundary dedup pass continues to operate on the corrected block bounds (`block_start..block_end` derived from `managed_region_len` against the new write), so adopter-area duplicates above and below the canonical region are still removed. Canonical-block-wins semantics are unchanged.
- A new unit test under `merge_managed_block::tests` reproduces the bug: it (a) writes a file containing the shipped multi-subsection canonical under the marker, (b) reruns `merge-managed-block` with the same canonical, (c) asserts `action == "unchanged"`, `dedup_removed == Some(0)`, mtime preserved, and file contents byte-equal to the pre-run state.
- A second unit test exercises the same-structure update path — a multi-subsection canonical whose comment wording changes between runs — and asserts the on-disk block is replaced cleanly with each subsection header appearing exactly once (the orphan-tail symptom would surface as duplicated `# IDE` / `# OS` headers).

## Edge Cases

- **Stable canonical, repeated runs (the bug case)** — the on-disk body matches the supplied block byte-for-byte after a `created` or `inserted` run. The structural walk consumes all `block.lines().count()` lines, the body slice byte-equals `block`, and the primitive emits `unchanged` without rewriting. Mtime is preserved. This is the case that accumulates orphan headers under the broken next-blank heuristic; the fix must make it perfectly idempotent.
- **Same-structure content change** — supplied `block` and on-disk block share line count and blank-line positions (the realistic update path when the framework template tweaks comment wording or adds an ignore pattern within an existing subsection). The structural walk consumes all lines, the body slice differs from `block`, and the *updated* arm replaces exactly the on-disk span. Adopter content past the block — and its single separating blank line — moves to `after` and is preserved.
- **Canonical block with no interior blank lines** — the previously broken case stays correct under the new rule. The structural walk degenerates to "consume all `block.lines().count()` lines" with no interior-blank handling needed; behavior is unchanged for single-subsection canonicals and all existing tests pass.
- **Canonical block grew between runs** (more lines than on-disk wrote) — at the first position where supplied `block` expects non-blank but on-disk has a blank (the previous run's end-of-block terminator), the structural walk stops early. `body_end` lands at the end of the old canonical's last consumed line; `after = &text[body_end..]` correctly starts at the separator blank. The replacement writes the new full canonical; adopter content after the separator is preserved.
- **Canonical block shrank or structurally diverged between runs** — supplied `block` has fewer lines than the on-disk old canonical, or its blank-line positions don't align with the old block's. The structural walk consumes up to `block.lines().count()` lines; whatever remains of the old canonical past `body_end` becomes adopter territory. Cross-boundary dedup strips non-comment body lines that match the new canonical; leftover comment-shaped subsection headers from the obsolete subsections persist as orphan headers. Operators clean these up by hand on the one-time transition. Acceptable because (a) the *common* path — stable canonical, repeated reruns — is the case that must be perfectly idempotent (and now is), (b) structural template changes are rare and coordinated, and (c) the failure mode is locally visible (orphan headers in the diff) and recoverable (delete the headers).
- **Adopter edited inside the canonical region** — out of scope. The primitive's contract has always been "the managed region is framework-owned; don't edit inside it." If an adopter inserted a line inside the block, the structural walk may include their edit in the body slice, the body comparison fails, and the next run overwrites. This matches the existing `html-comment` style's behavior.
- **Canonical block where the *last* line is blank** — `block.lines()` does not yield an empty trailing line after a final newline, but does yield an empty line for an interior blank, so `block.lines().count()` returns the correct expected-line budget. The structural walk respects the same yielding behavior on the on-disk side, so the body span captures exactly the line set that was written.
- **Marker appears in adopter content as a literal `# {marker}` line outside the managed region** — already handled by the marker-line scan returning the first match. The behavior is unchanged: the first `# {marker}` line wins; the rest of the file (including any second occurrence) is adopter territory.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
