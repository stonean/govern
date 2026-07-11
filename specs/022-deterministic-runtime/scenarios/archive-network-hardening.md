---
section: "Follow-on scenarios"
---

# Archive-network-hardening

## Context

The 2026-07-11 runtime review found network/archive gaps in the bootstrap-path primitives:

- `fetch-archive` calls reqwest's blocking client directly from the async MCP tool handler — a documented panic in debug builds and a blocked tokio worker thread in release; it silently truncates response bodies at the 256 MiB cap and writes the truncated archive with a matching computed sha (corruption surfaces later in `extract-archive`); and its module doc claims the archive is written on checksum mismatch while the code errors before writing (the error's `path` names a file that never exists).
- `extract-archive` skips tar symlink entries but materializes zip symlink entries as regular files whose content is the link-target path, with the S_IFLNK-stripped mode applied; and its 0o7777 mode mask preserves setuid/setgid/sticky bits from untrusted downloaded archives.

## Behavior

- The MCP seam wraps `fetch-archive`'s blocking HTTP work in `tokio::task::spawn_blocking` (or uses the async client), so the tool cannot panic in debug builds or starve the runtime in release.
- A response body exceeding the fetch cap is an operational error naming the cap, never a silent truncation; the archive is not written.
- The checksum-mismatch doc/behavior drift is resolved in favor of the code: the error is documented as fail-before-write and no longer reports a path that does not exist.
- Zip symlink entries are detected via the entry's Unix mode (`S_IFMT == S_IFLNK`) and skipped, matching the tar path and the module doc.
- Extracted file modes are masked to 0o777 — setuid/setgid/sticky bits from archive headers are never applied.

## Edge Cases

- CLI invocation of `fetch-archive` (no tokio runtime) is unaffected by the spawn_blocking seam.
- A body of exactly the cap size still succeeds.
- Executable bits within 0o777 are still preserved (the `apply_unix_mode` exec-bit behavior that `apply-manifest` depends on is unchanged).

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
