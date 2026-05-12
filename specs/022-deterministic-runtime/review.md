---
spec: 022-deterministic-runtime
scenario: apply-manifest
reviewed-at: 2026-05-12T14:30:00Z
reviewed-against: f8d4008
diff-base: 6fc0acc
must-violations: 0
should-violations: 3
low-confidence: 2
skipped-passes: []
---

# Review — 022-deterministic-runtime (scenario: apply-manifest)

## Summary

Scenario `apply-manifest` adds three primitives
(`apply-manifest`, `enforce-manifest`, `merge-managed-block`),
refactors `merge-claude-md` into a thin compat shim delegating to
`merge-managed-block`, wires the new primitives across every runtime
entry point (parser names, walker dispatch, MCP `TOOL_NAMES`,
`framework/runtime-tools.txt`), rewrites the `/govern` bootstrap
procedure to a six-primitive shape (eliminating the host-generated
bash walker that the live anvil bootstrap had to fall back to), and
extends the `govern-basic` parity fixture to exercise every strategy
+ marker style + cleanup path end-to-end. `gvrn` bumps 0.2.1 → 0.3.0
and ships to crates.io plus GitHub releases.

No MUST violations: the shipped rule catalogs (`security-backend.md`,
`security-frontend.md`, `configuration.md`) target web-app patterns
(auth/sessions, XSS, cross-module config) that don't fire against
CLI/MCP primitive code. The single rule with surface contact is
`BE-INPUT-004` (path-traversal defense for filesystem ops on
user-supplied values); under the runtime's threat model the host
that constructs manifest entries IS the operator (no privilege
boundary crossed), so the rule's strict applicability is bounded —
recorded as SHOULD with explicit threat-model context rather than
silenced.

Three SHOULD findings concern code-reuse drift (the `resolve_path` /
forward-slash-normalization helpers continue to be duplicated
per-primitive, now across 7 modules) and a defense-in-depth note on
path-traversal handling. The simplicity-pass `is_regex_meta` finding
that was flagged on the initial review run was applied via `--fix`
(see [Auto-fix applied](#auto-fix-applied) below) and dropped from
the count. Two low-confidence findings flag operational edge cases
worth a second look before the runtime hits real-world bootstrap
loads. **Spec remains non-blocking and may advance to `done` after
`/gov:validate`.**

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

### SHOULD: BE-INPUT-004 — defense-in-depth path-traversal check on host-supplied entry paths

- **File**: `runtime/src/primitives/apply_manifest.rs:147,159`,
  `runtime/src/primitives/merge_managed_block.rs:63`
- **Rule**: BE-INPUT-004 — _"User-supplied values MUST NOT be used
  directly in filesystem paths. Filesystem operations MUST resolve
  the canonical path and verify it falls within the expected base
  directory before opening the file."_
- **Finding**: `apply-manifest`'s `process_entry` joins
  `source_root` / `target_root` with `entry.source` / `entry.dest`
  via `Path::join` and writes / reads without canonicalizing and
  re-checking containment. An absolute path in `entry.source`
  (`"/etc/passwd"`) silently overrides `source_root` per Rust's
  `PathBuf` semantics; a relative path containing `..` could escape
  `target_root` after canonicalization. `merge-managed-block` has
  the same shape — `args.path` is host-supplied and writes happen
  without a base-directory check. Under the runtime's actual threat
  model the host (LLM orchestrator / operator script) already runs
  with the operator's filesystem privileges, so this isn't a
  privilege boundary — recorded at SHOULD because BE-INPUT-004 was
  written for web-service contexts where the user IS untrusted, and
  the CLI threat model puts the trust boundary at the operator
  level. The same shape exists in pre-027 primitives
  (`extract-archive` has explicit path-traversal protection for
  archive entries, but `substitute-templates`'s host-supplied
  source/target dirs do not; `merge-claude-md`'s `path` arg does
  not).
- **Auto-fixable**: no — defense-in-depth design choice; whether
  to harden the CLI's args boundary is a framework-level decision,
  not a mechanical fix.
- **Suggested fix**: if the runtime's threat model evolves to
  treat the MCP/JSON host as untrusted (e.g., third-party MCP
  clients invoking `gov-rt:*` tools without operator review),
  add a canonical-path + base-directory check helper to
  `primitives::mod` and call it from every primitive that writes a
  host-supplied path. Until then, leave the contract explicit:
  primitives trust the host, and the host is responsible for the
  paths it supplies.

### SHOULD: reuse — `resolve_path` helper duplicated across 7 primitive modules

- **File**: `runtime/src/primitives/apply_manifest.rs:112-119`,
  `runtime/src/primitives/enforce_manifest.rs:112-119`,
  `runtime/src/primitives/merge_managed_block.rs:106-113`,
  plus the four pre-existing copies in `extract_archive.rs`,
  `substitute_templates.rs`, `run_generator.rs`, `fetch_archive.rs`.
- **Rule**: AGENTS.md `Boundaries` + general reuse — duplicated
  utility functions drift over time.
- **Finding**: the apply-manifest scenario adds three new copies of
  the same six-line `resolve_path(repo, p)` helper (absolute path →
  as-is, relative path → `repo.join`). The pattern is now in 7
  primitive modules and the bodies are byte-identical. A single
  shared helper in `primitives::mod` would eliminate the drift
  surface — and provide the natural home for the
  canonicalize-and-base-check escalation if the threat model
  changes (see BE-INPUT-004 above).
- **Auto-fixable**: no — affects all 7 call sites; clean refactor
  belongs in a follow-up commit rather than a `--fix` mechanical
  pass.
- **Suggested fix**: promote `resolve_path` to
  `runtime/src/primitives/mod.rs` (alongside `read_text`, `rel_path`,
  `write_atomic`, `write_atomic_bytes`); delete the per-module
  copies. The rename surface is contained; tests already cover the
  behavior.

### SHOULD: reuse — forward-slash path normalization duplicated

- **File**: `runtime/src/primitives/apply_manifest.rs:121-123` (`normalize_dest_path`),
  `runtime/src/primitives/enforce_manifest.rs:121-127` (`normalize` + `path_to_forward_slash`).
- **Rule**: same reuse concern as above; smaller surface.
- **Finding**: both new primitives need to normalize host-supplied
  paths to forward-slash form for portable comparison (Windows
  semantics) and to render `Path` values back to JSON-stable
  strings. The helpers are slightly different (one accepts `&str`,
  the other accepts `&Path`) but cover the same intent.
  Promoting a small `forward_slash` helper alongside `rel_path` in
  `primitives::mod` removes the duplication.
- **Auto-fixable**: no — minor refactor; better landed as a
  follow-up.
- **Suggested fix**: add `pub(crate) fn forward_slash(p: impl AsRef<Path>) -> String`
  to `primitives::mod`; collapse the per-primitive callers.

## Auto-fix applied

The simplicity finding `enforce_manifest::is_regex_meta` →
`regex::escape` was applied via `/gov:review` `--fix` on the same
HEAD as this review (`f8d4008`). The `is_regex_meta` table was
deleted; `compile_glob` now delegates per-character escaping to
`regex::escape` (already in the crate's deps). All 14
`enforce_manifest::tests` still pass byte-for-byte against the new
implementation, including
`glob_escapes_regex_metacharacters` (verifies `legacy.md` does NOT
match `legacyXmd` — i.e. `.` is treated as a literal, not a regex
`any`) and `regex_metacharacter_inside_glob_is_treated_as_a_literal`
(verifies `[bracket].md` matches its literal form). The pre-fix
behavior is preserved by `regex::escape`'s contract.

## Low-confidence findings

### LOW-CONFIDENCE: quality — `apply-manifest` reads the entire destination file to detect `unchanged`

- **File**: `runtime/src/primitives/apply_manifest.rs:178-201`
  (`apply_update`).
- **Confidence**: 60 — depends on real-world file sizes the
  bootstrap touches. Framework files are small (`<10 KiB`); not yet
  exercised against larger payloads.
- **Finding**: when `dest_exists`, `apply_update` reads the entire
  destination into memory and compares against the post-substitution
  bytes. For small files this is fine; for a `unchanged` decision
  across a large staging tree the cost is `O(sum-of-file-sizes)`
  per run. A streaming compare (read N bytes at a time, short-circuit
  on first mismatch) would bound the memory cost.
- **Auto-fixable**: no — optimization that's only worth doing when
  use cases prove it's needed.
- **Suggested fix**: defer until profiling on a real bootstrap
  identifies a hot spot. Document the current behavior in the
  primitive's module docs.

### LOW-CONFIDENCE: quality — `find_line_prefix_block` byte-offset arithmetic is dense

- **File**: `runtime/src/primitives/merge_managed_block.rs:220-240`.
- **Confidence**: 70 — unit tests cover the surface (created /
  inserted / updated / unchanged across line-prefix shapes,
  including CRLF and EOF-no-newline edge cases), but the manual
  byte-offset math (`offset + line_end + usize::from(line_end < rest.len())`)
  is harder to audit than line-iterator-based scanning would be.
- **Finding**: the implementation walks the file by repeatedly
  finding `\n` in a tail slice, computing the next offset by
  conditionally adding 1 for the consumed newline. The
  `usize::from(bool)` idiom is correct but non-obvious; a reader
  has to verify the loop variant manually. Refactoring to use
  `text.lines()` plus a separate byte-offset tracker (or
  `text.split_inclusive('\n').enumerate()`) would make the loop
  invariant explicit.
- **Auto-fixable**: no — refactor pending a clearer reading-by-line
  shape.
- **Suggested fix**: defer until the function needs to be touched
  for another reason; current behavior is correct and tested.

## Waived findings

*None.*

## Skipped passes

*None — all five passes ran against the apply-manifest scope.*
