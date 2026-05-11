---
spec: 015-tarball-fetch
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 015-tarball-fetch

## Summary

Single-file edit to `framework/bootstrap/govern.md` replacing per-file fetches with an archive-fetch / extract / resolve flow. The instructions describe `curl -fsSL` + `tar -xzf` + `mktemp -d` operations executed by the AI agent on the operator's machine during `/govern` adoption. The fetch surface is the only network touchpoint in the entire framework. All five passes ran; no findings. `blocking: no`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._

## Pass notes

### Security

The fetch is described with `curl -fsSL` (fails on HTTP error, silent progress, follow redirects). The archive URL is a documented HTTPS endpoint at `github.com/.../govern/archive/refs/heads/main.tar.gz`; the integrity check verifies the extracted `govern-main/` directory exists before any per-file copy. Failure of any of {curl, tar, missing dir} triggers an explicit abort message rather than silent fallback. No `eval`, no `bash <(curl …)`-style pipe-to-shell. Temp directory is via `mktemp -d -t govern-XXXXXX` (cross-platform safe between `$TMPDIR` and `/tmp`).

### Reuse

The fetch-and-extract pattern is the documented single path for adopter file acquisition — no per-file fallback alongside it (the previous per-file flow was fully replaced, not paralleled).

### Quality

Three failure modes (curl fail, tar fail, missing extraction dir) all converge on the same abort message — operator gets a single, consistent error surface. Edge Cases section documents fetch failure as a hard abort, not a partial-write state.

### Efficiency

Archive fetch replaces O(N) per-file curls with O(1) tarball — direct improvement to bootstrap latency. The `--maxdepth`/sed/grep operations in the per-file resolution step are bounded by the tarball contents (fixed shape).

### Simplicity

Three-step flow (fetch → extract → resolve-per-file) is described inline without indirection. The generator is not involved (govern.md is not in `gen-claude-commands.sh`'s input set), keeping the install path direct.
