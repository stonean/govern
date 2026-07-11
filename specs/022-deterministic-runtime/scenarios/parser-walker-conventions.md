---
section: "Follow-on scenarios"
---

# Parser-walker-conventions

## Context

Three convention-layer defects from the 2026-07-11 review, bundled because they share the parser/walker seam:

- **Step numbering**: the parser discards ordered-list `start` values and resets its counter on every new depth-1 list, and `&lt;!-- audit:ignore-promotion --&gt;` markers between items split one markdown list into several — so all rewritten commands parse with wrong step labels (status.md's six steps all parse as step "1"), `progress.step` and gate names sent to hosts are wrong or duplicated, and the data model's `StepNumber` contract is not honored.
- **Gate convention**: a step blocks only when its prose contains the literal phrase "ask the user to approve" — a convention documented nowhere (the spec lists only numbered steps, backticked primitives, and HTML markers) — and a `gate-confirm` primitive step without the phrase (prune.md step 4's shape) dispatches non-blocking, so the exec path would skip confirmation before a destructive write. Conversely the phrase check pre-empts step type, so a primitive step containing the phrase silently drops its dispatch.
- **Span heuristic**: `looks_like_primitive` flags every lowercase kebab-case code span, so ordinary vocabulary (`no-checkbox`, `cli-config-dir`, `keep-pending`) hard-fails parseability. The functionally rewritten prune.md sits on `legacy-prose-commands.txt` as a result — and the allowlist lint tolerates `Invalid` (not just `LegacyProse`), weakening the CI signal.

## Behavior

- The parser honors document numbering: ordered-list `start` seeds the step counter, lists separated only by HTML comments/blank lines continue the previous numbering, and parsed step numbers match the literal numbers in all rewritten command files. `progress.step` and gate names reflect real steps.
- A step that invokes `gate-confirm` is a blocking gate by virtue of the primitive, phrase or no phrase; the prose-phrase trigger remains as a fallback for gates without the primitive and is documented as a structural convention alongside the other three. A primitive step is never silently converted to a phrase-triggered gate that drops its dispatch.
- Backticked kebab-case spans that are not in `PRIMITIVE_NAMES` do not fail parsing; the strict check applies only to spans in primitive-invoking position (e.g. "Invoke `X`") or within edit distance of a known primitive name. prune.md parses cleanly and leaves the legacy allowlist; the parseability lint accepts only `LegacyProse` for allowlisted files, never `Invalid`.

## Edge Cases

- A file whose lists genuinely restart at 1 mid-Instructions (two separate procedures) is not a supported shape; the parser follows the numbering it sees.
- The phrase fallback and the primitive-based gate never both fire for one step (primitive wins).
- Removing prune.md from the allowlist is part of this scenario; help.md, amend.md, log.md, clarify.md, and groom.md remain allowlisted until their own rewrites land.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
