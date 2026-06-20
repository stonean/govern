---
section: "Follow-on scenarios"
---

# Merge-managed-block-subsection-insertion

## Context

The fix captured in [merge-managed-block-multi-subsection-end](merge-managed-block-multi-subsection-end.md) made `merge-managed-block` (line-prefix style) idempotent for multi-subsection canonicals by bounding the on-disk block with a *structural template* walk (`walk_body_extent`): from past the marker, consume up to `block.lines().count()` on-disk lines, matching expected-blank against on-disk-blank and terminating when an expected non-blank line meets an on-disk blank. That walk is correct only when the on-disk block and the new canonical share the same **shape** — same line count, same blank-line positions. That sibling scenario's *Edge Cases* already named two divergent paths: "Canonical block grew between runs" (predicted to stop early but land cleanly) and "Canonical block shrank or structurally diverged" (declared to leave orphan headers, "Acceptable… structural template changes are rare and coordinated").

Adding a new agent's gitignore subsection is exactly a structure-changing canonical edit, and it is **not** rare — it recurs every time the framework adopts an agent. [framework/templates/project/gitignore](../../../framework/templates/project/gitignore) is `claude-style` for both Claude (`.claude/*` + `!.claude/commands/`) and Auggie (`.augment/*` + `!.augment/commands/`), with Antigravity (`.agents/*` + `!.agents/skills/`) and the `# govern derived views` / `# govern session state` / `# IDE` / `# OS` subsections after. Inserting the Auggie subsection between Claude and Antigravity makes the new canonical four lines longer than what an existing adopter has on disk.

The "grew between runs" Edge Case predicted this lands cleanly. It does not. Reproduced against the live primitive (old on-disk block: Claude → Antigravity, no Auggie; new canonical with Auggie inserted), the structural walk drifts by the inserted line count: by the time it reaches the tail, a non-blank expected line (`# govern session state`) lines up with an on-disk blank, so it terminates ~4 lines early. `body_end` lands mid-block; the old block's tail subsections spill into `after` and are concatenated below the freshly written canonical. The dedup pass strips their pattern bodies but preserves their comment headers, leaving the same orphan trail the prior scenario fixed for the stable-rerun case:

```text
# OS
.DS_Store
Thumbs.db

# govern session state — per-user, ephemeral; managed by /{project}:target.

# IDE

# OS
```

The merge converges (a second run reaches `unchanged` — no infinite accumulation), so the damage is a one-time injection of permanent orphan comment headers, not unbounded growth. New adopters (no marker on disk) are unaffected: the `inserted` path writes the new canonical verbatim. Only existing adopters re-running `/govern` hit it.

Root cause is the structural-template assumption in `walk_body_extent` ([`runtime/src/primitives/merge_managed_block.rs`](../../../runtime/src/primitives/merge_managed_block.rs)): line-shape is not a stable identity for the block across a subsection insertion. Comment wording and blank positions drift between releases; the **pattern lines** (non-blank, non-comment gitignore globs) are the stable identity.

## Behavior

- `walk_body_extent` MUST bound the on-disk block by **group alignment**, not a line-shape walk. It splits both the supplied `block` and the on-disk region (from past the marker) into blank-line-delimited subsections, reduces each subsection to its pattern lines (non-blank, non-comment), and aligns on-disk groups against canonical groups with a two-pointer walk.
- An on-disk group is part of the managed block when, against the current canonical group, it: **shares a pattern** (a structure-preserving edit such as a comment-wording tweak — consume it); or **shares a pattern with a *later* canonical group** (the canonical inserted one or more subsections not present on disk — skip past the inserted groups, then consume this on-disk group against the group it matches); or **shares no pattern while canonical groups remain** (a full rewrite of the current group — consume it).
- The block ends at the first on-disk group reached after the canonical's groups are exhausted; that group and everything after it is adopter territory and is preserved. `body_end` is the byte offset immediately past the last in-block group's final line, so `after` begins at the separator blank, exactly as the same-structure update path requires.
- A subsection-insertion edit (the Auggie case) MUST replace the on-disk block cleanly: the new subsection appears exactly once, every subsection header appears exactly once (no orphan tail), and adopter language sections appended after the block survive verbatim. A second run with the same canonical reaches `unchanged`.
- Existing guarantees are preserved unchanged: stable-canonical reruns reach `unchanged` (mtime preserved); same-structure content changes (comment-wording tweaks) replace cleanly; full-content replacement of a single-group block replaces cleanly; cross-boundary dedup operates on the corrected `block_start..block_end` bounds with canonical-block-wins semantics. The v0.12.0 regression tests pass unchanged.
- A new unit test (`line_prefix_multi_subsection_inserts_new_subsection_without_orphan_tail`) reproduces the insertion path: write the old multi-subsection canonical under the marker with an adopter `# Rust` tail, rerun with the new canonical that inserts a subsection in the middle, and assert `action == "updated"`, the inserted subsection present once, no duplicated headers, the adopter tail preserved, and an idempotent second run (`unchanged`).

## Edge Cases

- **Subsection inserted in the middle (the realistic case)** — the new canonical has one or more subsections the on-disk block lacks. The two-pointer walk detects each insertion by the "shares a pattern with a later canonical group" rule, skips the inserted canonical group(s), and consumes the on-disk groups against the canonical groups they match. The block is bounded at the true end; no orphan tail.
- **Subsection appended at the end + adopter content follows** — ambiguous without a block delimiter: the trailing canonical group and the first adopter group are indistinguishable by pattern alone, and the walk may consume one adopter group as a rewrite of the appended canonical group. This matches the pre-existing structural-walk behavior (it consumed the separator blank plus the next adopter line) and is not regressed. The framework mitigates it by inserting new agent subsections in the middle (before the stable `# IDE` / `# OS` trailer), never appending after adopter territory.
- **Full-content replacement of a single-group block** — the on-disk group shares no pattern with the canonical group and no later canonical group exists; the "full rewrite of the current group" rule consumes it, and the next on-disk group (adopter) stops the walk because the canonical groups are exhausted. Preserved unchanged from the prior behavior.
- **Subsection removed between runs** — the on-disk block has a group whose patterns match no remaining canonical group. With canonical groups still pending, it is consumed as a full rewrite of the current canonical group, which shifts the alignment; the removed subsection's content is replaced rather than orphaned. Agent removal is not a supported workflow, so this path is incidental; the failure mode (if any) is locally visible in the diff.
- **Every pattern in a subsection renamed simultaneously** — that subsection shares no pattern with its canonical counterpart and is treated as a full rewrite (correct) unless it is also not the last canonical group and a later group coincidentally shares a pattern; in practice agent subsections carry two stable patterns (a dir glob plus its carve-out), so a simultaneous rename of both is implausible.
- **Adopter edited inside the canonical region** — out of scope, unchanged: the managed region is framework-owned. An adopter line inserted inside the block may be captured into the body slice; the body comparison fails and the next run overwrites.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
