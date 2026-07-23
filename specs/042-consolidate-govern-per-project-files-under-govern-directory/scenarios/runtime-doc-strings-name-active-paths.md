---
section: "Documentation and canonical sources"
---

# Runtime-doc-strings-name-active-paths

## Context

Spec 042 moved the per-project config and session files to `.govern/config.toml` / `.govern/session.toml`, with reads and writes resolving the **active** file (`session_path_for_write` and the config resolver). The runtime's user-visible self-documentation still names the legacy root paths as the target: `gvrn write-session --help` says "Atomically rewrite `.govern.session.toml`", and the same doc comments surface as MCP tool descriptions and schema argument docs.

## Behavior

- The runtime's user-visible doc surfaces — clap `--help` output for the `main.rs` Command variants, MCP tool descriptions, and schema argument docs — describe config/session access as targeting the active file, naming `.govern/config.toml` / `.govern/session.toml` as the canonical locations.
- No user-visible doc string names `.govern.toml` or `.govern.session.toml` as *the* target path; legacy names appear only where the fallback or migration behavior is being described.

## Edge Cases

- Doc strings that legitimately document the legacy fallback or a migration (e.g., `migrate-session-file`) keep naming the legacy path — the sweep distinguishes "describes the fallback" from a stale canonical claim.
- Internal code comments are out of scope; the sweep covers what clap and the schema/MCP layer emit to users.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
