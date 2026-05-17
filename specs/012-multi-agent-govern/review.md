---
spec: 012-multi-agent-govern
reviewed-at: 2026-05-17T22:30:00Z
reviewed-against: d904430
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 012-multi-agent-govern

## Summary

Unified `/govern` installer plus the post-task-10 audit of `settings_template` Bash patterns: both Agent Registry rows now cover `Bash(git status *)`, `Bash(git config *)`, `Bash(chmod *)`, and `Bash(awk *)` (Claude format) and the mirrored `launch-process` regexes (Auggie format). The change is pure data in markdown table cells — no application code added. Tech-stack alignment skipped via `.govern.toml` `[review] tech-stack-verified = true`. Loaded rule files: `configuration-cross.md` (the only file whose suffix selects for govern's text-first stack — no backend/frontend code). All five passes ran; no findings. `blocking: no`.

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

The added Bash patterns follow the scenario's narrow-pattern guidance — `Bash(git status *)` and `Bash(git config *)` instead of `Bash(git *)` so risky operations (`git push`, `git reset --hard`) remain outside the bootstrap allowlist. `Bash(chmod *)` and `Bash(awk *)` are broad but consistent with the pre-existing `Bash(curl *)` / `Bash(tar *)` convention and scoped to the bootstrap permission set only — the full permission set is applied later by `/{project}:configure`. No security rule files apply at this scope (no backend/frontend code).

### Reuse

The change uses the existing canonical `settings_template` JSON field on each row; no parallel structure introduced.

### Quality

JSON validity verified on both rows (`node -e JSON.parse(...)` parses cleanly — 14 entries in Claude's `permissions.allow`, 8 in Auggie's `toolPermissions`). Parity verified: every new Claude `Bash(X *)` pattern has a matching Auggie `^X␣` (trailing-space) regex with identical command-token granularity. The scenario's parity contract holds.

### Efficiency

N/A — static configuration data.

### Simplicity

Could the four new entries be consolidated into a broader pattern? Explicitly no — the scenario's "Pattern over-broadness" edge case rejects `Bash(git *)` and the same reasoning applies to other broad patterns. The four narrow entries are the simplest form that satisfies the audit contract without granting unintended commands.
