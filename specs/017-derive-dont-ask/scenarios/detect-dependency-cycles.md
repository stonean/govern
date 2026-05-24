---
section: "Generators and Hooks"
---

# Detect-dependency-cycles

## Context

`scripts/gen-spec-deps.sh` rewrites every spec's frontmatter `dependencies` from body links on every commit (Q7 / AC23). The generator currently treats the resulting edges as the truth of the dep graph but never checks the graph for cycles. A cycle is a structural defect — `traverse-deps` cannot order such a graph, `/anvil:status` ordering by `blocked-by` becomes unstable, and the dashboard's blocked-by callout reports nonsense.

The most common path to a cycle is sibling [skip-prose-cross-references](skip-prose-cross-references.md): bidirectional navigational links between two specs make each one depend on the other. The opt-out scenario fixes the *source* of cycles, but the generator should still detect cycles in the *output* — both as a safety net for adopters who don't adopt the opt-out and as protection against any other path to a cycle (deliberate or accidental).

In one adopter project the silent cycle count reached 9 direct 2-cycles plus longer cycles composed of them, none of which surfaced until a manual dep-graph audit. The generator owns the artifact; it should own the integrity check.

## Behavior

- After computing each spec's `dependencies`, `gen-spec-deps.sh` MUST run a cycle check across the full graph (every `specs/NNN-*/spec.md` and the edges among them).
- If the graph contains a cycle, the generator exits non-zero and writes to stderr a structured report naming the strongly connected component(s) — at minimum a line per cycle listing the spec slugs in traversal order (e.g., `cycle: 001-system-spec-templates -> 012-multi-agent-govern -> 001-system-spec-templates`).
- The check runs *after* the frontmatter rewrite so the diff is visible in the working tree even when the run fails — the author sees both the offending edges and the cycle report. The pre-commit hook surfaces the failure and blocks the commit.
- The check is unconditional in the govern repo (CI catches drift) and in adopter projects (the shipped pre-commit hook propagates the failure).
- Fixtures under the existing `gen-spec-deps` test surface cover: (a) a 2-cycle triggers the detector with both slugs in the message; (b) a 3-cycle is reported as a single SCC, not three 2-cycles; (c) a graph with a cycle and an acyclic subgraph reports only the cycle; (d) the acyclic happy path exits 0 with no cycle output.

## Edge Cases

- **Self-cycle** (spec A's body contains a link to itself): reported as a 1-cycle. Almost certainly a typo, but the generator does not silently strip self-references.
- **Multiple disjoint cycles** in one graph: every SCC is reported; one report line per cycle. The author sees the full picture in one run rather than fixing-and-rerunning.
- **Cycle that crosses a spec at a status the dep is permitted to skip** (e.g., a cycle entirely among `done` specs): still reported. The opinion is structural, not status-aware — a cycle in the artifact is a defect regardless of the operational state of the participants.
- **Cycle introduced mid-commit**: the pre-commit hook detects it and blocks the commit; the author either edits the body to break the cycle (typically via [skip-prose-cross-references](skip-prose-cross-references.md)'s opt-out) or, if the cycle is deliberate, has to argue for an override mechanism that does not exist today.

## Open Questions

*None — Q1 resolved during scenario implementation.*

## Resolved Questions

- **Q1: Cycle-check failure mode in the existing pre-commit hook.** Confirmed during implementation: no hook edit required. Both the govern-repo hook (`.githooks/pre-commit`) and the shipped adopter hooks (`framework/bootstrap/hooks/pre-commit` wrapper and `framework/bootstrap/hooks/govern-pre-commit` managed inner) all run under `set -euo pipefail`, so a non-zero exit from `scripts/gen-spec-deps.sh` propagates unswallowed and blocks the commit. Naming note: the scenario originally referred to `.githooks/govern-pre-commit`; the govern repo actually uses a single `.githooks/pre-commit` (the `govern-pre-commit` split exists only on the adopter side, where the outer `pre-commit` wraps the managed `govern-pre-commit`).
