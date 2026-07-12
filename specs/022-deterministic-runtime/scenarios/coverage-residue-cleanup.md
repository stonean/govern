---
section: "Follow-on scenarios"
---

# Coverage-residue-cleanup

## Context

Two smaller findings from the 2026-07-11 coverage review, distinct from the new-primitive work in [coverage-expansion-primitives](coverage-expansion-primitives.md):

- **Two primitives are dead weight**: `substitute-templates` and `merge-claude-md` are exposed as MCP tools, listed in the configure permission blocks, and tracked by `runtime-tools.txt` / `lint-tool-coverage`, but no command prose invokes either as a step — both were folded into `apply-manifest` (which takes a substitutions map and handles `CLAUDE.md` with `strategy: skip`). They are carried, tested, and permission-listed for nothing.
- **The exec path silently narrows semantic work**: `clarify.md` steps 7–8 (edge-case enumeration, acceptance-criterion verification) are work the markdown-only path performs, but they carry no `<!-- llm:… -->` marker, so `gvrn exec clarify` no-ops them. The "two paths, one contract" guarantee is quietly narrower under exec, with nothing signalling the reduction.
- **Noted, not urgent**: `merge-permissions` serves only the Claude permission shape; the Auggie / Antigravity / OpenCode formats each carry an explicit "walk the prose" banner and do the merge by hand.

## Behavior

- `substitute-templates` and `merge-claude-md` are either wired into a command step that genuinely needs them, or retired — removed from `TOOL_NAMES`, `PRIMITIVE_NAMES`, the interpreter dispatch, the CLI enum, `runtime-tools.txt`, the data-model, and the configure permission blocks (the six-site rule in reverse), so the exposed surface matches what commands actually use.
- `clarify.md`'s exec-path scope matches its markdown-only scope: steps 7–8 either gain `llm:` markers (folding into the `askClarifyQuestion` loop) so the exec path performs the same semantic work, or the exec-path scope reduction is documented explicitly in the command and data-model so it is not a silent gap.
- The `merge-permissions` single-format limitation is recorded (here) as the trigger for a future per-format merge primitive if that path becomes hot — not built speculatively.

## Edge Cases

- Retiring a primitive must pass `lint-tool-coverage` (no command references the retired name) and the MCP `TOOL_NAMES ⊇ runtime-tools.txt` test afterward.
- If a real future caller for `substitute-templates` / `merge-claude-md` is identified, wiring it in is preferred over retiring — the decision is per-primitive, not blanket.
- The markdown-only path for clarify keeps performing steps 7–8 regardless of which exec-path option is chosen.

## Open Questions

- Wire or retire `substitute-templates` / `merge-claude-md`: is there a near-term command that would use either, or are they genuinely subsumed by `apply-manifest`?
- Markers versus documented reduction for clarify steps 7–8: does folding them into `askClarifyQuestion` fit the single-round-trip ABI, or is a documented scope reduction the honest answer?

## Resolved Questions

*None yet.*
