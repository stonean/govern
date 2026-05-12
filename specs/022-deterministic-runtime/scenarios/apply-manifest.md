---
section: "Follow-on scenarios"
---

# Apply-manifest — strategy-aware bulk substitute + write for the bootstrap

## Context

After the `govern-bootstrap` scenario landed, `/govern` successfully uses `fetch-archive` and `extract-archive` through the runtime. But the substitute + write phase still requires the host to generate a bash walker script (~200 lines, observed in the wild as `govern-walk.sh` during an anvil bootstrap) because `substitute-templates` has one strategy — overwrite every file at the destination — and the bootstrap actually needs three:

1. **update** (overwrite if different, honor pinned-file list) — the bulk of framework files: constitution, rules, templates, hooks.
2. **create** (write only if destination doesn't already exist) — adopter-seedable files: system.md, errors.md, events.md, scripts.
3. **skip-if-conflict** — adopter-owned files: AGENTS.md, CLAUDE.md.

Plus three orthogonal concerns the current primitives don't address:

- **Pinned-file exemption**: `.govern.toml`'s `[pinned] files` block lists adopter customizations the bootstrap must never touch, regardless of strategy.
- **Manifest enforcement**: as the framework evolves, files get renamed or dropped; the bootstrap must remove no-longer-shipped files from the adopter's directories so they don't accumulate (legacy `skills/`, legacy workflow filenames, dropped slash commands).
- **Template-literal preservation**: when installing `govern.md` itself, the bootstrap MUST keep `{project}` and `{cli-config-dir}` placeholders literal so the next `/govern` run substitutes them per the next adopter — not per this one.

Because none of those fit `substitute-templates`'s shape, the host falls back to scripting. The runtime's value-add then drops from "drives the whole bootstrap" to "drives the fetch/extract phase; the rest reverts to bash." Worse, the observed walker uses `python3 -c '...'` for safe substitution because the host knows bare bash is unreliable when the description string contains quotes or shell metacharacters — a sign the primitives layer is the right home for this work and the bash layer can't shoulder it correctly.

## Behavior

Three new primitives. Each is thin enough that the bootstrap procedure shrinks to roughly half a dozen primitive calls plus two prose steps for input-gathering and the completion message.

1. **`apply-manifest`** — strategy-aware bulk substitute + write.
   - Args: a `source-root` (typically the staging directory from a prior `extract-archive`), a `target-root` (the adopter project), an `entries` list where each entry is `{ source: String, dest: String, strategy: "update" | "create" | "skip-if-conflict", keep-literals: Option<Vec<String>> }`, a `pinned: Vec<String>` of dest paths the primitive must not touch, and a `substitutions: BTreeMap<String, String>` for placeholder replacement.
   - For each entry: resolve `source` against `source-root`, resolve `dest` against `target-root`. Apply the entry's strategy:
     - **update**: substitute placeholders, compare against the existing destination, write only when the result differs (preserve mtime when unchanged), record `created` / `updated` / `unchanged`.
     - **create**: substitute placeholders, write only when the destination is absent, record `created` / `skipped-exists`.
     - **skip-if-conflict**: write only when the destination is absent; substitution is not applied (these are adopter-owned templates the framework seeds but never edits afterward), record `created` / `skipped-exists`.
   - Pinned entries short-circuit before any read or write and record `skipped-pinned`.
   - `keep-literals` lets a single entry exclude named substitution keys (e.g., `govern.md` self-install with `keep-literals: ["project", "cli-config-dir"]` keeps those placeholders intact for the next adopter's bootstrap).
   - Result: `entries: Vec<ManifestEntryResult>` listing the per-entry action, plus aggregate `created` / `updated` / `unchanged` / `skipped-exists` / `skipped-pinned` / `source-missing` counts.

2. **`enforce-manifest`** — directory cleanup against an expected file list.
   - Args: `directory: String`, `expected: Vec<String>` (filenames relative to `directory`), `pinned: Vec<String>` (dest paths, relative), `recursive: bool` (default `false` — top-level only, matching the bootstrap's slash-command cleanup behavior), and `glob-include: Option<String>` (default `*.md`).
   - For each file in `directory` matching `glob-include`: if it's in `expected`, keep; if it's in `pinned`, keep with a `pinned-kept` label; otherwise remove.
   - Result: `removed: Vec<String>`, `kept: Vec<String>`, `pinned-kept: Vec<String>`.
   - One primitive call replaces the bootstrap's three cleanup loops (slash-command manifest enforcement, legacy `skills/` directory removal, legacy workflow filename removal).

3. **`merge-managed-block`** — generalize `merge-claude-md` to arbitrary file types with configurable marker shape.
   - Args: `path: String`, `block: String`, `marker: Option<String>` (default `govern-managed`), `marker-style: Option<"html-comment" | "line-prefix">` (default `html-comment`).
   - `html-comment` style: `<!-- BEGIN {marker} -->` / `<!-- END {marker} -->` (current `merge-claude-md` behavior, unchanged).
   - `line-prefix` style: a single line `# {marker}` marker followed by the block, terminated by a blank line or EOF (matches `.gitignore` and `.gitattributes` conventions; the bootstrap's `.gitignore` merge currently uses a `# govern` line marker that the host hand-greps for).
   - Same four actions as `merge-claude-md`: `created` / `inserted` / `updated` / `unchanged`. mtime preserved on the unchanged path.
   - `merge-claude-md` becomes a thin compat wrapper that delegates to `merge-managed-block` with `marker-style: html-comment` and `marker: govern-managed`; the existing fixture and procedure callers keep working unchanged. Drop the alias on the next major version.

The bootstrap procedure (`framework/bootstrap/govern.md`) rewrites its Instructions section to invoke these three primitives instead of relying on a host-generated walker:

```text
1. (prose) context note — host pre-gathers inputs
2. fetch-archive
3. extract-archive
4. (gate) approve install
5. apply-manifest               # replaces ~20 update/create entries
6. merge-managed-block (.gitignore)   # replaces the inline grep check
7. enforce-manifest             # replaces three cleanup loops
8. apply-manifest (govern.md self-install with keep-literals)
9. (prose) completion message
```

Six primitive calls plus two prose steps. The host's remaining job: gather inputs, compute the manifest from the staged tree, invoke the procedure, render the completion message. No bash walker, no `python3 -c '...'` substitution fallback, no per-file Edit tool calls from the host.

## Edge Cases

- **Cross-platform path separators** — manifest entry paths use forward slashes; the primitive normalizes to the host OS at write time. Pinned-file matching is case-sensitive on Unix, case-insensitive on Windows (matching NTFS semantics).
- **Pinned file with a strategy of `update`** — `skipped-pinned` wins; the file is not touched. The result records the entry with a clear label so the host can surface it in the completion message ("3 files kept due to pinning").
- **Manifest entry whose source the staging tree doesn't contain** — `apply-manifest` returns a `source-missing` action for that entry rather than silently skipping or erroring the whole walk. The host surfaces this so the operator can diagnose the upstream archive (likely a deleted-from-main path).
- **`enforce-manifest` against an empty or missing directory** — succeeds with zero removals. The primitive does NOT create the directory if absent (that's `apply-manifest`'s job for its `dest` paths).
- **`merge-managed-block` line-prefix style with no trailing newline in the existing file** — the primitive normalizes by ensuring the block is followed by a single blank line before any subsequent content; the file ends with exactly one trailing newline regardless of marker style.
- **`keep-literals` interacts with the manifest's `substitutions` map** — listed keys are simply not substituted for that entry. Listed keys absent from the substitutions map are no-ops (no error). Listing every key in the map is equivalent to passing an empty map for that entry.
- **Re-running the bootstrap is idempotent** — every primitive call is a no-op when inputs haven't changed: `apply-manifest` records `unchanged` per entry, `enforce-manifest` records zero removals (assuming no upstream rename happened), `merge-managed-block` records `unchanged`. mtimes are preserved on the unchanged path so build tools don't see spurious touches.
- **Mixing pinned with `skip-if-conflict`** — both checks fire; whichever short-circuits first wins (pinned is checked first since it's the absolute "do not touch" signal). The result label reflects which check fired (`skipped-pinned` over `skipped-exists`).

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

- **Should we instead add per-file strategy support to the existing `substitute-templates` primitive?** No. `substitute-templates`'s contract is "overwrite the destination from a source tree with substitutions applied" — a single-strategy bulk copy that's useful in narrower contexts than the bootstrap (e.g., scaffolding a new sub-tree from a template directory). Conflating it with the bootstrap's three-strategy manifest would muddy the abstraction. `apply-manifest` is the more specific primitive that names what it's for; `substitute-templates` stays as-is for the simpler use case.
- **Why not just keep generating bash walker scripts?** That's the current state; the scenario exists because the host writing bash defeats the runtime's value proposition. Bash also has known pitfalls (sed quoting hazards for user-provided substitution values — the observed walker reaches for `python3 -c '...'` to dodge them) that a native Rust primitive avoids by construction. Substitution determinism, idempotency, and atomicity all want to live below the host, not above it.
- **Should `merge-managed-block` and the `merge-claude-md` alias both ship in the same release?** Yes. The alias is a zero-effort compat shim; deprecating it later is cheaper than breaking the existing `merge-claude-md` callers (the bootstrap fixture, the parity goldens, future host scripts). Drop the alias on the next major release of `gvrn`.
- **Why ship `enforce-manifest` separately from `apply-manifest` rather than folding "remove not-in-manifest" into the latter?** Single responsibility. `apply-manifest` is "write the things you said to write" — it knows the expected files but doesn't own the existing files. `enforce-manifest` is "delete the things not on the list" — it operates against directory state, not against manifest input. Splitting them keeps each primitive's failure modes narrow and lets callers compose them (apply-then-enforce for first-run; just-apply for incremental scaffolding).
