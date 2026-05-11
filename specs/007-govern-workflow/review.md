---
spec: 007-govern-workflow
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 007-govern-workflow

## Summary

The `/govern` installer (`govern/govern.md`, later unified by spec 012) plus a sweeping `.claude/` → `{cli-config-dir}/` substitution across every command source. Pure markdown — security rules do not apply at the framework level. The bootstrap installer itself is interpreted by an AI agent at adoption time and was subsequently rewritten by specs 012 and 015. All five passes ran; no findings. `blocking: no`.

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

The `/govern` installer issues fetch operations from the adopter's machine — those operations are described in the markdown, executed by the AI agent on the operator's behalf. Concrete fetch semantics (curl, tarball, integrity) were tightened by spec 015 and audited there.

### Reuse

The `{cli-config-dir}` placeholder is the canonical substitution token reused by `scripts/gen-claude-commands.sh` and every later command edit. The unified `govern.md` model (post-012) is the single bootstrap entry point.

### Quality

Substitution is exhaustive across the listed command files; later regenerations of `.claude/commands/gov/*.md` re-derive deterministically.

### Efficiency

N/A.

### Simplicity

One placeholder, one substitution table — no per-agent special-casing in command sources.
