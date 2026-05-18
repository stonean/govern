---
section: "Follow-on scenarios"
---

# Family-9-annotations-and-promotions

## Context

[`scripts/audit/primitive-promotion-candidates.sh`](../../../scripts/audit/primitive-promotion-candidates.sh) (Family 9) flags numbered Instructions steps in the six rewritten pipeline commands ([`analyze`](../../../framework/commands/analyze.md), [`implement`](../../../framework/commands/implement.md), [`plan`](../../../framework/commands/plan.md), [`specify`](../../../framework/commands/specify.md), [`status`](../../../framework/commands/status.md), [`target`](../../../framework/commands/target.md)) that contain neither a backticked runtime-primitive name nor an `<!-- llm:* -->` extension-point marker. Each flagged step is one of two categories:

1. **Host-responsibility step.** Logic that legitimately belongs to the host (e.g., interactive prompts, ambiguity resolution, directory traversal that has no runtime primitive). These need an `<!-- audit:ignore-promotion -->` annotation on the preceding content line so Family 9 stops flagging them.
2. **Primitive-promotion candidate.** Deterministic logic encoded as prose that should become a new gvrn primitive. These expand the runtime surface and require schema + implementation in `runtime/` plus a new entry in [`framework/runtime-tools.txt`](../../../framework/runtime-tools.txt).

Origin: spec 026's `/audit` v1 advisory findings, 2026-05-18. Captured via the inbox.

## Behavior

Family 9 resolution ships in two passes:

- **Pass 1 — annotation sweep.** Walk each Family 9 finding and decide category by inspection. For host-responsibility steps, insert `<!-- audit:ignore-promotion -->` on the preceding content line of the affected step in the framework/commands source. Regenerate `.claude/commands/gov/*.md`. Family 9 advisory count drops to only the genuine primitive-promotion candidates.
- **Pass 2 — primitive promotions.** For each remaining flagged step, design the corresponding primitive (args/result schemas under `runtime/src/schema/primitives.rs`; implementation under `runtime/src/primitives/`; MCP tool registration; new entry in `framework/runtime-tools.txt`). Replace the prose step with a backticked invocation of the new primitive. `gvrn` ships a minor version bump per new primitive.

Pass 1 is mechanical; Pass 2 expands the runtime contract and should land per-primitive (each primitive its own commit) so review can scope to one schema at a time.

## Edge Cases

- **A flagged step is genuinely ambiguous** (could be host or runtime depending on framing). Default to Pass 1 (host annotation) for the conservative path; promote later if a real adopter need surfaces. Promoting prematurely expands the runtime contract without payoff.
- **A flagged step crosses host/runtime boundary** (some sub-steps are deterministic, others interactive). Split the numbered step into two numbered steps in the source command file; annotate the host sub-step, promote the deterministic sub-step.
- **A primitive promotion needs cross-spec dependencies** (e.g., new schema types in `data-model.md`). Promotion blocks on the schema work; track via this scenario's tasks below.

## Open Questions

- **Total count of Family 9 advisories at start of Pass 1.** Run `bash scripts/audit/primitive-promotion-candidates.sh` against the current tree to enumerate; the count drives whether Pass 2 ships as one batch or multiple per-primitive commits.

## Resolved Questions

*None.*
