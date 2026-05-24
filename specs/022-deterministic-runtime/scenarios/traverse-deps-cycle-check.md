---
section: "The primitive library"
---

# Traverse-deps-cycle-check

## Context

Spec 022 declares the `traverse-deps` primitive (spec.md:76) as "verify spec dependencies exist as directories and have compatible status." Today the primitive checks (a) existence of each declared dependency directory and (b) status compatibility along single edges, but it does not check the dep graph for acyclicity. `/anvil:analyze` consumes `traverse-deps` (analyze.md:45) and inherits that gap — a graph with a cycle passes `traverse-deps` and `/anvil:analyze` reports no finding.

The upstream fix lives in spec 017's [detect-dependency-cycles](../../017-derive-dont-ask/scenarios/detect-dependency-cycles.md) — `gen-spec-deps.sh` will fail when its generated graph has a cycle, blocking the commit. That covers the common case (cycle introduced during normal author flow). It does *not* cover:

- adopter projects on an older shipped `gen-spec-deps.sh` (the script ships with `create` strategy and is not auto-updated);
- projects where the pre-commit hook was skipped or never installed;
- frontmatter `dependencies` lists that have drifted from body links (uncommitted edits, hand-edited frontmatter on a one-off basis);
- any future path that produces a cycle outside the generator's purview.

`traverse-deps` is the read-side primitive `/anvil:analyze` and other commands rely on. Cycle detection belongs in the primitive as a defense-in-depth check independent of how the graph was assembled.

## Behavior

- `traverse-deps` MUST detect cycles in the dep graph it walks and emit a structured finding when one is present. The finding names the strongly connected component(s) — slugs in traversal order, one entry per cycle.
- The finding is at the same severity level as the existing dependency-existence and status-compatibility findings (blocking). `/anvil:analyze` treats it as a finding that fails the analyze gate, consistent with how the existing `traverse-deps` findings are surfaced.
- Cycle detection runs even when other findings (missing-dependency, status-mismatch) are present. The primitive reports the full set; the cycle is not masked by other defects.
- The primitive's `success` result shape is unchanged when the graph is acyclic — cycle detection adds findings, it does not restructure the output.
- Parity tests under `runtime/tests/parity/` exercise `/anvil:analyze` against a fixture containing a 2-cycle and assert both the markdown-only walker and the runtime walker surface an equivalent finding.

## Edge Cases

- **Cycle entirely among `done` specs**: still reported. `traverse-deps` is the structural read primitive; the cycle is a defect in the artifact regardless of the operational state of the participants. Aligns with [detect-dependency-cycles](../../017-derive-dont-ask/scenarios/detect-dependency-cycles.md)'s same-status posture for the generator-side check.
- **Self-cycle in `dependencies`** (a spec listing itself): reported as a 1-cycle.
- **Multiple disjoint cycles**: each SCC is reported as its own finding; the analyze report lists all of them in one pass.
- **Graph with a missing dependency that would close a cycle if present**: the missing-dependency finding fires; the cycle finding does not (the closing edge is absent in the actual graph the primitive walks).
- **Cycle introduced via stale frontmatter** (frontmatter `dependencies` lists an edge no longer in the body): `traverse-deps` walks frontmatter, so it sees and reports the cycle. The author resolves by re-running `gen-spec-deps.sh` (which removes the stale edge and either clears the cycle or surfaces a real one).

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
