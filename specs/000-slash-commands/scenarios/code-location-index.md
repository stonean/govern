---
spec-ref: "000-slash-commands — Command Set / implement"
tags: []
---

# Code Location Index

## Note

This scenario was clarified through the lens of [013-text-first-artifacts](../../013-text-first-artifacts/spec.md). The Resolved Questions below capture how 013's principle shaped each decision: location and maintenance derive from the "structured derived view, regenerated on demand" framing; granularity was decided on consumer-fit grounds; the consumer question was reframed as "the agent itself reads this on subsequent sessions, and humans read the diff at PR time" — sufficient to justify building the producer by default, with programmatic consumers (`/gov:validate`, `/gov:status`, `/gov:capture`) deferred until each is independently specced.

## Context

`/gov:implement` walks tasks, writes code, and verifies acceptance criteria, but does not record *where* the implementation landed. After a feature reaches `done`, there is no machine-readable mapping from acceptance criteria (or the feature itself) to the files, functions, or modules that satisfy them. This forces every downstream consumer — `/gov:validate` doing traceability, `/gov:status` answering "what surface area does this feature own?", `/gov:capture` and 011's brownfield triage answering "where does this bug live?" — to re-derive the mapping by reading code or asking the user.

A code-location index produced and maintained by `/gov:implement` would close that gap. The index becomes an authoritative artifact tying spec language to source-tree positions, surviving the implementation session that produced it.

## Behavior

- As `/gov:implement` works through tasks, it accumulates an in-memory `Map<AC, Set<file>>` recording the source files it creates or modifies in service of each acceptance criterion.
- At the end of each `/gov:implement` run, the map is regenerated as `specs/{NNN-feature}/code-locations.md` with deterministic ordering: `## AC: {criterion text}` headings appear in spec order; files within each AC are alphabetical. Stable diffs, no spurious churn.
- The artifact is committed to git so the diff is reviewable in PRs (humans get review/onboarding/refactoring-impact value immediately) and so `/gov:implement` itself can read prior sessions' state when resuming a feature mid-stream.
- Programmatic consumers (`/gov:validate` for dead-criterion detection, `/gov:status` for surface-area summaries, `/gov:capture` for "where does this bug live") are deferred until each is independently specced. The data is available the moment any of them adopts the index.
- The index format is plain markdown — no new tooling, no JSON schema, no parsing dependency beyond what governance already requires.

## Edge Cases

- Code that is moved, renamed, or deleted after `done` — the index becomes stale unless something rewrites it. The maintenance question (regenerate vs. incremental) determines whether staleness is detected, accepted, or auto-healed.
- Acceptance criteria satisfied by configuration, infrastructure, or external artifacts (not source files) — the index needs a way to express "implemented by X" where X is not a code path.
- Brownfield features where `/gov:implement` was never run — the index must be backfillable by some other path (likely 011's triage flow) so brownfield specs are not permanently second-class.

## Open Questions

*All open questions resolved. See Resolved Questions below.*

## Resolved Questions

- **Location** — per-spec at `specs/{NNN-feature}/code-locations.md`, treated as a structured derived view per 013's text-first artifacts principle. Listed consumers (`/gov:validate`, `/gov:status`, `/gov:capture`, 011's brownfield triage) are all session-target-scoped, so each opens a known per-spec path rather than filtering a top-level index. Matches the rest of the artifact layout (every feature artifact lives inside `specs/{NNN-feature}/`), survives spec moves and archival, and avoids the write contention a single top-level index would create with parallel feature work. Cross-feature queries are not a stated consumer; if one emerges, walking `specs/*/code-locations.md` produces a combined view on demand — itself another structured derived view.
- **Granularity** — per-acceptance-criterion. Files are grouped under the AC they serve. Per-task is too noisy (overlapping file lists, write churn for the same file across many tasks); per-feature is too coarse (loses the AC-to-files mapping that `/gov:validate` needs for dead-criterion detection and `/gov:capture` needs for "where does this bug live"). Three of the four listed consumers benefit from per-criterion granularity; `/gov:status` is indifferent (it can collapse per-criterion into a unique-files set). The artifact format is `## AC: {criterion text}` headings with file bullets underneath; each AC's file list deduplicates internally. `/gov:implement` writes this incrementally as it works through tasks tied to ACs. Idempotent — re-running implement without new tasks produces no change.
- **Maintenance** — regenerated by `/gov:implement` on every run from an in-memory `Map<AC, Set<file>>` accumulated as the run walks tasks. No incremental appends. Aligns with 013's "structured derived views are regenerated on demand" principle: the canonical source is the running task state plus actual file edits; `code-locations.md` is the regenerated derived view. Staleness is impossible — if a previously-recorded file is renamed, moved, or deleted, the next implement run simply omits it from the regenerated output, and `git diff` shows the removal. History lives in git (`git log -- specs/{feature}/code-locations.md`) rather than embedded in the artifact, avoiding duplication of git's job. Deterministic ordering (AC order matches spec order; files alphabetical within each AC) ensures stable diffs and no spurious churn. Idempotent: re-running implement with no new task progress produces an identical file.
- **Consumers** — build the producer by default; defer programmatic consumers until each is independently specced. The agent itself is a consumer on subsequent `/gov:implement` sessions of the same feature (a structured prior-state map for resuming work), and humans are consumers at PR review, onboarding, and refactoring-impact time — those consumers exist now and justify the producer's existence on their own. The four candidate programmatic consumers from the original scenario (`/gov:validate` dead-criterion detection, `/gov:status` surface-area summaries, `/gov:capture` "where does this bug live", 011's brownfield triage) all stay deferred — building any of them requires a dedicated spec that picks up this scenario's resolutions as the producer-side design. The artifact is **committed to git** (not gitignored), because as a markdown derived view it diffs cleanly in PRs and provides immediate human value — distinct from binary or structured-noise derived views (SQLite caches, JSON indexes) that 013's principle gitignores. This distinction warrants a small clarification to 013 / the constitution: markdown derived views may be committed when their diffs are valuable to humans; non-markdown derived views must remain gitignored.
