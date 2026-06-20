---
spec: 032-opencode-agent
reviewed-at: 2026-06-20T16:55:57Z
reviewed-against: a65c021bbcdf3bdd96dc970b486d51b454ddac85
diff-base: 5fdd739099194ab0eb746599e63c1cbbf66b1145
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 032-opencode-agent

## Summary

Clean review: **0 MUST, 0 SHOULD, 0 low-confidence** across all five passes.
The change adds OpenCode as a fourth agent (third `layout` profile) entirely in
markdown framework files, shell (`install.sh`, `gen-configure-mcp.sh`, two audit
scripts, one test), and embedded JSON config blobs. Rule selection: the stack is
markdown/bash (`tech-stack-verified = true`), neither backend nor frontend, so
only `configuration-cross.md` (the `*-cross.md` file) loads; the backend /
frontend / api / accessibility / performance rule files do not apply to a
markdown/bash framework surface. No rule in the loaded set is violated. The
change follows established repo patterns (the OpenCode layout reuses the
claude-style command flow; `configure/opencode.md` mirrors `antigravity.md`'s
prose-walk; the generator's OpenCode block mirrors Antigravity's constant block),
and is covered by the generator test (check D), both parity audits, the full
17-family audit gate (green), and a live `opencode 1.17.8` integration check
(`opencode mcp list` → `✓ gvrn connected` on a scaffolded sample). Not blocking.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

*None.*

## Waived findings

*None.*

## Captured issues (pending /gov:groom)

*None — no incidental issues were appended to `specs/inbox.md` during this work.*

## Skipped passes

*None — all five passes ran.*

## Pass notes

- **Security.** `configuration-cross.md` is the only loaded rule file; it
  introduces no env vars or operator-tunable constants in this change, so its
  CFG-CONST / CFG-ENV rules do not fire. No secrets, auth, DB, or network
  surface is added. The change is security-positive: both the `install.sh`
  OpenCode seed and `configure/opencode.md` carry `deny` entries for destructive
  shell (`rm -rf`, `sudo`, `git push --force`, `git reset --hard`, …).
- **Reuse.** The OpenCode layout deliberately reuses the claude-style command
  flow (verbatim copy, byte-compare self-update, `# govern` integrity check)
  rather than duplicating logic; the divergence is isolated to the `command/`
  directory name, `/{project}/<name>` invocation, and the single-file
  `opencode.json` config. The settings-template JSON is duplicated between the
  registry row and the `install.sh` seed — this is the **existing, intentional**
  pattern for every agent, guarded against drift by
  `installer-registry-parity.sh` direction 3 (order-insensitive JSON compare),
  so it is not a new reuse defect.
- **Quality (correctness).** The per-layout branches compose consistently:
  the `command/` (singular) directory, `/{project}/<name>` invocation, root
  `opencode.json` settings/MCP target, and verbatim `govern` installer are
  applied uniformly across §Derived values, §Per-Agent Scaffolding,
  §Permission Setup, self-update, integrity, placeholder, and directory-creation.
  The State-B write-file branch is generalized to OpenCode's `mcp` shape; the
  `bash` permission map orders `*: ask` → allows → denies to satisfy OpenCode's
  last-match-wins semantics. `installer-registry-parity` (install path + seed
  JSON), `runtime-probe-parity` (probe), and the generator test all pass, and
  the live scaffolded sample loaded gvrn and registered the namespaced command.
- **Efficiency.** No performance-sensitive code; the generator's OpenCode block
  is a constant built outside the per-tool loop. Nothing to flag.
- **Simplicity.** A third `layout` was warranted (verified: the single committed
  `opencode.json` spanning MCP + permissions matches neither existing layout);
  `configure/opencode.md` stays a host-side prose-walk rather than introducing a
  new runtime primitive (spec Resolved Q5). No premature abstraction.
