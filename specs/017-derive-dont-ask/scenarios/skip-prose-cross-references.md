---
section: "Generators and Hooks"
---

# Skip-prose-cross-references

## Context

`scripts/gen-spec-deps.sh` (per Q7 / AC23) is authoritative — the pre-commit hook regenerates each spec's frontmatter `dependencies` from inline markdown links to sibling spec directories in the body (links matching `](../NNN-slug/...)` or `](specs/NNN-slug/...)` outside fenced code blocks; script lines 39–70). The design assumes every body link expresses a dependency.

In practice authors write two kinds of cross-references: (1) **dependency** links — "this spec needs the rule loader from [024-rule-loader](../024-rule-loader/spec.md)" — which the generator's current behavior is built for; and (2) **navigational** links — "`/jobs` was added by [018-scheduled-jobs](../018-scheduled-jobs/spec.md)" — which are informational and don't make the citing spec depend on the referenced one. The generator silently promotes (2) to (1).

When bidirectional navigational links exist between two specs, both ends gain a dependency on the other, producing a cycle. In one adopter project (anvil), bidirectional prose references produced 9 direct 2-cycles (e.g., 001↔012, 001↔018, 005↔009, 006↔010). None blocked work — every involved spec was already past the gate — but the dep graph surfaced by `/anvil:status`, the dashboard's `blocked-by` callout, and `traverse-deps` becomes unreliable.

Root cause: no syntactic distinction in the markdown source between "depends on" and "see also."

## Behavior

- `gen-spec-deps.sh` MUST provide an author-controlled opt-out so navigational links can stay rich without inducing dep edges. The opt-out is syntactic — visible in the markdown source — not configuration-driven.
- The opt-out MUST compose with the existing fenced-code exclusion: a link inside a code fence continues to produce no edge regardless of any opt-out markers around it (current behavior unchanged).
- Running `gen-spec-deps.sh` twice on a tree containing opt-outs produces no diff on the second run — same idempotence invariant as the current generator.
- Fixtures under the existing `gen-spec-deps` test surface MUST cover: (a) a link in the opt-out region produces no edge; (b) a link outside any opt-out region produces an edge as today; (c) a link inside a code fence still produces no edge (regression).
- Constitution / AGENTS.md / spec 017 prose that explains the "body links are authoritative" rule gains a one-line carve-out describing the opt-out form chosen below.

## Edge Cases

- **Adopter projects on an older shipped `gen-spec-deps.sh`** (the script ships to adopters with `update` strategy per AC23 — every `/govern` run refreshes it from upstream): adopters pick up the opt-out automatically on the next `/govern` run. Adopters who have pinned the script in `.govern.toml` `pinned.files` keep their existing copy and see no behavior change until they unpin or hand-merge.
- **Migration of existing specs in this repo**: any inline link currently treated as a dep that the author wants to demote becomes a one-shot edit (move under the opt-out marker), then the next pre-commit run rewrites the frontmatter. No data migration; the body is authoritative.
- **Empty `## See also` section (or whatever opt-out form is chosen)**: produces no edges and no error.

## Open Questions

*None — Q1 resolved during scenario implementation.*

## Resolved Questions

- **Q1: Opt-out form.** **Resolution: section-based, marker is `## See also` only.** Links under a level-2 heading whose text (case-insensitively) is exactly `See also` are skipped; the opt-out ends at the next heading at level 2 or shallower (deeper subheadings inherit the opt-out). `## References` is **not** an opt-out — task 29's migration deliberately uses `## References` as the formal location for body-authored dependency links in 13 specs, and edges under it must continue to flow into frontmatter. The split mirrors conventional markdown semantics (References = formal citations / deps, See also = navigational pointers). Section-based is idiomatic and noise-free; comment-based markers (`<!-- gen-spec-deps:ignore -->`) are deferred — finer-grained but add visible markup, revisit if a real in-prose case surfaces. The opt-out composes with the existing fenced-code and blockquote-line exclusions: any of the three suppresses an edge.
