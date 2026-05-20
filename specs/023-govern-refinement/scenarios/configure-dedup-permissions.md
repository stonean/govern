---
section: "5–6. /configure canonical allow-set"
---

# Configure-dedup-permissions

## Context

The originating observation was about `.gitignore`: this project's editing patterns (multiple commands and agents appending over time) have produced duplicate entries. `.gitignore`'s framework-managed region is already well-controlled — `merge-managed-block` with the `# govern` line-prefix marker regenerates the block wholesale on every run — so duplicates there are bounded to the case of an adopter pasting a canonical entry outside the marker, which is adopter-owned territory the framework intentionally does not touch.

`.claude/settings.local.json` is the surface where the same dedup concern actually bites. Spec 023 §§5–6 defined `/configure`'s canonical allow-set semantics: explicit per-path entries for govern-owned state files (§5) and an unconditional MCP-tool permission block sourced from `framework/runtime-tools.txt` (§6). Both sections describe additions — what gets added — but say nothing about cleanup of what's already there. The command source (`framework/bootstrap/configure/claude.md` and the generated `.claude/commands/gov/configure.md`) reinforces this with an explicit instruction: "Add missing entries; do NOT remove, deduplicate, reorder, or rewrite entries the user (or another command) added beyond the canonical set listed below."

The intent of that line is sound — protect user-added entries from being clobbered — but it has a side effect: duplicates introduced by prior `/configure` runs, manual edits, or other commands appending to `permissions.allow` / `permissions.deny` never get cleaned up, and the file grows noisy over time. Unlike `.gitignore`, there is no managed-block delimiter to separate framework-owned from adopter-owned entries — every entry lives in the same JSON array. The dedup work is mechanical (string-equal comparison) and the writer is the framework itself, so this is a deterministic action the `gvrn` runtime should handle rather than agent prose.

## Behavior

- `/configure` invokes a new deterministic `gvrn` primitive (introduced separately on spec 022) that performs an idempotent merge of the canonical allow/deny set into `.claude/settings.local.json`. The merge ensures every canonical entry is present, removes exact-match duplicates from `permissions.allow` and `permissions.deny` (including non-canonical entries the user or other commands added), preserves first-occurrence order, and leaves untouched fields (`additionalDirectories`, `defaultMode`, top-level keys other than `permissions`) byte-for-byte unchanged.
- Exact-match equality is string-equal on the rendered permission pattern. No semantic normalization: `Bash(git status)` and `Bash(git status *)` are distinct, and both are preserved if both are present.
- The dedup pass is mandatory on every `/configure` invocation, not a separate flag — the file's invariant is "no duplicate entries in `permissions.allow` / `permissions.deny`."
- `framework/bootstrap/configure/claude.md` and `framework/bootstrap/configure/auggie.md` update their inline instruction: the "do NOT … deduplicate, reorder, or rewrite" prohibition becomes "do NOT reorder or rewrite non-duplicate entries; the primitive deduplicates exact-match entries automatically." The generated `.claude/commands/gov/configure.md` follows from the source rewrite via the normal command generator; no separate edit required.
- The markdown-only path (runtime absent) is the existing agent-driven splice: read the JSON, ensure canonical entries are present, then perform the dedup pass as a final step before writing.
- `.gitignore` is out of scope for this scenario. Its framework-managed region is already handled deterministically by `merge-managed-block`. If the broader dedup contract should extend to `.gitignore`'s managed block (or across the marker boundary), that is a separate scenario on spec 022 against the `merge-managed-block` primitive — not a `/configure` behavior.

## Edge Cases

- Empty file or missing `permissions.allow` / `permissions.deny` array: the primitive seeds the array with the canonical entries; no dedup is required.
- Malformed JSON: the primitive surfaces a parse error and refuses to write; the existing JSON-corruption path applies.
- Duplicates introduced by another command between `/configure` runs (e.g., `/govern` or a future command appending to `permissions.allow`): cleaned up on the next `/configure` invocation. `/configure` is the single owner of the dedup invariant on this file.
- The Auggie permission format uses a different rendering (per spec 023 §6's host-specific note); dedup applies independently per host's array shape.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
