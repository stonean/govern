---
section: "Follow-on scenarios"
---

# Framework-list-dedup

## Context

Two framework-managed files in adopted projects accumulate duplicate list entries over time, by different mechanisms but the same root cause: multiple commands and agents append to them across many runs, and the existing primitives don't dedup.

- **`.gitignore`** — the originating observation. `/govern`'s bootstrap invokes `merge-managed-block` with `marker-style: "line-prefix"` and `marker: "govern"` to install or update a managed block of ignore patterns (e.g., `.claude/`, `*.sqlite`). The primitive replaces the block's body wholesale on each run, so duplicates inside the managed block are impossible. Duplicates appear *across the marker boundary*: an adopter (or another command) pasted `.claude/` above or below the `# govern` line, and the canonical block also contains `.claude/`. Both survive because `merge-managed-block` deliberately doesn't touch adopter-owned territory.

- **`.claude/settings.local.json`** — the surface where this conversation surfaced. `/configure` writes a canonical permission allow/deny set into JSON arrays, but the file has no managed-block delimiter — every entry lives in the same array whether canonical, user-added, or appended by another command. Prior `/configure` runs, manual edits, and other appenders have produced exact-match duplicates that never get cleaned up.

The behavior contract for the JSON surface is committed in spec 023's `configure-dedup-permissions` scenario. This scenario lands the runtime primitives both surfaces depend on — one new primitive and one behavior extension to an existing primitive — under a single dedup contract: the framework's canonical block wins, and duplicates of canonical entries get removed wherever they appear. Keeping `govern` settings grouped under their canonical block (and not redundantly scattered through the file) is the visible payoff.

This mirrors the `ask-consolidation` scenario's precedent of landing more than one primitive delivery in a single scenario when they share a coherent contract.

## Behavior

### Part A — `merge-permissions` (new primitive, JSON surface)

- **Name and surfaces.** Named `merge-permissions`. Ships as the CLI subcommand `gvrn merge-permissions` and the MCP tool exposed under the bare name `merge-permissions` (Claude: `mcp__gvrn__merge-permissions`; Auggie: `mcp:gvrn:merge-permissions`).
- **Inputs.** A target JSON file path (default `.claude/settings.local.json`, but accepts any path so other host bootstraps can reuse it); a canonical `allow` set (array of strings); a canonical `deny` set (array of strings); optional additional fields the canonical set wants to ensure under `permissions` (e.g., `additionalDirectories`).
- **Output envelope.** Standard primitive envelope: action (`created` / `updated` / `unchanged`), path written, per-array counts of entries added vs. duplicates removed.
- **File does not exist.** Create it with `{ "permissions": { "allow": [...canonical-allow], "deny": [...canonical-deny] } }` and emit `created`.
- **File exists.** Parse it as JSON. Refuse with a `parse-error` envelope on malformed JSON; do not write.
- **Canonical-presence pass.** Ensure every canonical entry is present in its respective array. Append missing entries at the end, preserving prior order.
- **Dedup pass.** Remove exact-match duplicates from `permissions.allow` and `permissions.deny` — exact string-equality, no normalization. First occurrence wins; later duplicates are removed in place.
- **Preservation.** Untouched fields preserved byte-for-byte: `additionalDirectories`, `defaultMode`, any other top-level keys, and unspecified keys under `permissions`. Field order under `permissions` matches the existing file's order; new fields added by this run land at the end of `permissions`.
- **Atomic write.** Tempfile + rename semantics, matching the other state-modifying primitives.
- **Idempotency / mtime preservation.** When the parsed file already matches the post-merge content (after canonical re-serialization with the primitive's formatter), emit `unchanged` and do not rewrite — preserves mtime for build-tool idempotency, matching `merge-managed-block`'s contract.
- **Registry update.** `framework/runtime-tools.txt` adds the `merge-permissions` entry so the configure-source generator (per spec 023 §6) picks it up on the next bootstrap.

### Part B — `merge-managed-block` cross-boundary dedup (existing primitive, line-prefix style)

- **Scope.** The new dedup behavior applies *only* when `marker-style: "line-prefix"`. The `html-comment` callsites (e.g., the `CLAUDE.md` framework block) keep their current passive behavior: the managed region is prose paragraphs, not a list, so cross-boundary dedup isn't a coherent operation there.
- **Dedup pass (line-prefix only).** After the existing install-or-update of the managed block, the primitive scans adopter-owned territory (everything outside the marker preamble line and its terminating blank line) for lines that string-equal any line inside the canonical block. Each matching line in adopter-owned territory is removed. **Canonical-block wins** — the canonical line stays inside the marker; the duplicate copy outside is the one removed. This keeps `govern` settings grouped under the `# govern` block rather than redundantly scattered through the file.
- **Comparison.** Exact string-equality on the trimmed line content (excluding the trailing newline). No glob expansion, no path normalization. `.claude/` and `.claude/*` are distinct, and both survive if both are present outside the marker.
- **Blank lines and comments preserved.** Adopter-owned blank lines and comment lines (`# foo` lines other than the marker line itself) are never removed by the dedup pass — the dedup operates on non-blank, non-comment ignore-pattern lines.
- **Output envelope addition.** The existing `merge-managed-block` result envelope grows two new fields on `line-prefix` invocations: `dedup-removed` (count of adopter-area lines removed) and a list of the removed line contents for telemetry / debug. The `html-comment` envelope shape is unchanged.
- **Atomic write and idempotency.** Same semantics as the primitive's existing contract — a run where the canonical block matches and no cross-boundary duplicates exist emits `unchanged` and does not rewrite.
- **No registry update.** `framework/runtime-tools.txt` already lists `merge-managed-block`; the behavior extension is transparent to the registry.

## Edge Cases

- **`merge-permissions` — missing `permissions` object.** Valid JSON but no top-level `permissions` field: add it with the canonical `allow` / `deny` arrays. Other top-level fields are preserved.
- **`merge-permissions` — missing `allow` or `deny` array.** `permissions` exists but one array absent: seed with the canonical set for that array; the other array's existing contents are untouched apart from dedup.
- **`merge-permissions` — non-array `allow` / `deny`.** Field exists but is not an array (null, object, string): refuse with a `schema-error` envelope; do not silently coerce.
- **`merge-permissions` — duplicate across canonical/non-canonical.** A user-added entry that string-equals a canonical entry is a duplicate. First occurrence wins regardless of which set the survivor came from; the canonical-presence pass does not re-append a canonical entry when an equal user-added entry is already present.
- **Auggie permission format.** Auggie's permission entries use a different shape (objects with `toolName` / `permission` fields, per spec 023 §6's host-specific note). Whether `merge-permissions` serves both host shapes via a format argument or whether a separate Auggie-format primitive is introduced is a plan-phase decision recorded as an open question on this scenario.
- **`merge-managed-block` — duplicate appears multiple times outside the marker.** All adopter-area duplicates of a canonical line are removed, not just the first. The canonical block remains the single source of that entry.
- **`merge-managed-block` — duplicate line appears in two separate managed blocks.** Out of scope. The primitive supports one managed block per file per marker; the cross-boundary scan only considers the single block's contents against everything outside it.
- **`merge-managed-block` — adopter line with trailing whitespace differing from canonical.** Treated as distinct (exact string-equality on trimmed line content; trailing whitespace inside the trim doesn't apply, but a difference in pattern body — even one space — is preserved).
- **Concurrent writers.** Both primitives' atomic-rename semantics guarantee no partial-write state on disk; concurrent invocations resolve last-writer-wins with file-system-atomicity for the rename. Neither primitive holds a lock.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
