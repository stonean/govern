---
spec: 003-bootstrap-automation
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 003-bootstrap-automation

## Summary

Generated `.claude/commands/gov/*.md` instances plus the hand-maintained `init.md`. The shipped command instances are mechanical substitutions of `framework/commands/*.md` via `scripts/gen-claude-commands.sh`, which was reviewed at the global code-surface audit. Security rules do not apply to markdown command instances. All five passes ran; no findings. `blocking: no`.

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

`scripts/gen-claude-commands.sh` was audited globally: `set -euo pipefail`, no `eval`, no network, no user-controlled input, deterministic `sed` substitutions. The generated command instances inherit the safety profile of their `framework/commands/*.md` sources. The hand-maintained `init.md` is interpreted by an agent at adoption time; no executable runtime.

### Reuse

Generator-script convention is reused by every other `scripts/gen-*.sh` introduced later (017). Substitution table is the single source of truth for `{project}` → `gov` and `{cli-config-dir}` → `.claude`.

### Quality

Generator handles the "obsolete generated file" pruning case (lines 44–55) and preserves `init.md` as the documented exception. No contract drift between command source and generated output observed at HEAD.

### Efficiency

`ls "$DEST"/*.md | wc -l` at line 57 is informational output, not a hot path; fine for the small file count involved (<30).

### Simplicity

Generator is ~58 lines with no configuration surface — the substitution table is hardcoded. Appropriate for the closed set of placeholders.
