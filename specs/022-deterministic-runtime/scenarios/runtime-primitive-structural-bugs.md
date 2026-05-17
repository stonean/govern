---
section: "Follow-on scenarios"
---

# Runtime-primitive-structural-bugs

## Context

Three MCP primitives shipped under spec 022's runtime — `append-task` and `create-scenario` from the ask-consolidation scenario, plus `read-tasks` and `check-stuck` from the broader 022 primitive set — share a root cause: hardcoded structural assumptions about `tasks.md` shape and the in-progress history that don't survive contact with real specs.

All four bugs surfaced 2026-05-17 during the living-specs scenario work on 023. In each case the primitive's "atomic-write" guarantee meant the broken state landed on disk before the operator could inspect; the operator then manually corrected the state, which defeats the deterministic-runtime promise that primitives are predictable mechanical operators.

The fixes live in the runtime crate (`runtime/src/primitives/append_task.rs`, the `read-tasks` parser, the `check-stuck` git-log walker). The shape established by 022's earlier scenarios (govern-bootstrap, apply-manifest, ask-consolidation) applies: primitives ship as Rust + MCP tool pairs with parity tests; each fix gets at least one fixture-based unit test plus a parity-test entry exercising the corrected behavior.

## Behavior

Four bugs, all rooted in the same family of structural assumptions:

### Bug 1: `append-task` default body uses `title` where it expects a slug

When the `body` argument is omitted, the primitive's documented behavior emits a single default checkbox `- [ ] Implement the behavior described in scenarios/{slug}.md` "derived from the title." The actual implementation substitutes the *full title* into the `{slug}` slot. A title of `"Implement scenarios/living-specs.md"` produces `- [ ] Implement the behavior described in scenarios/scenarios/living-specs.md.md` — doubled prefix, doubled extension.

**Fix.** Add an explicit `slug` argument to the primitive (preferred over deriving from title, because the caller knows the slug it just used in `create-scenario`). When `body` is omitted, the default checkbox uses `{slug}.md` cleanly. The `title` field stays free-form; it is the heading text, not the slug source.

### Bug 2: `append-task` numbering hardcoded to `## N.` top-level

When `tasks.md` uses a phased structure — e.g., 023's `## Phase A —` / `## Phase B —` / `## Phase C —` with `### N. Task` headers inside each phase — the primitive finds no `## N.` matches and falls back to `## 1.` at the file's bottom. That collides with the existing `### 1.` task in Phase A AND is structurally wrong (h2 vs the file's h3 task convention).

**Fix.** Detect the existing structure by scanning for `## Phase X` headers. When phased structure is detected, the next task is appended under a phase the caller specifies via a new `parent-heading` argument (or under a default `## Phase C — Follow-on` heading the primitive creates if no phase argument is given and no follow-on phase exists yet). The task heading level matches the existing convention: `### N.` inside a phase, `## N.` flat.

### Bug 3: `read-tasks` returns empty when tasks.md is phased

Same root cause as bug 2. 023's tasks.md uses `## Phase A — ... / ### N. Task` shape; the primitive's parser only matches `## N.` top-level numbered headers and returns `tasks: []`, even though 18 tasks exist. Downstream consumers — most notably the `/gov:implement` walker that relies on `read-tasks` to identify the next incomplete subtask — go blind on specs with non-flat `tasks.md`.

**Fix.** Extend the parser to walk `### N.` headers nested under `## Phase X` containers and return the flattened list with optional `phase` metadata on each task (so callers can render phase context if useful). For flat tasks.md files, the existing behavior is preserved — the parser returns the same shape it does today.

### Bug 4: `check-stuck` measures from the wrong baseline

The primitive computes `since-sha` from the *first* time the spec entered `in-progress` in git history, rather than the most recent reopen. A spec that went `in-progress → done → in-progress` via `/gov:ask`'s back-edge inherits every commit from the original implementation window. 023's first commit attempt on the living-specs task fired `stuck: true` with `commit-count: 8` because the original tasks-1-through-17 commits still counted, even though task 18 was brand-new and hadn't been touched.

**Fix.** Identify the most recent `in-progress` transition by walking `git log -p -- specs/{feature}/spec.md` backwards and finding the most recent commit whose diff includes a `+status: in-progress` line. Count commits on `tasks.md` since that SHA. This handles the reopen case correctly without changing the threshold or the false-positive shape.

## Edge Cases

- **A `tasks.md` with mixed structure** — some `## N.` top-level tasks plus some `### N.` tasks nested under `## Phase X`. Treat the file as phased: any `## Phase X` header anywhere in the file signals phased structure; the parser walks the appropriate headers per section.
- **`append-task` called with `parent-heading` that doesn't exist** — refuse with a clean operational error rather than silently creating a new phase or appending to file bottom. The caller fixes the argument and retries.
- **`check-stuck` on a spec that has never been `done`** — no reopen has occurred; the existing behavior (count from first `in-progress`) is already correct. The fix only matters for specs that have at least one `done → in-progress` cycle in their history.
- **`check-stuck` on a spec where `spec.md` has been touched by mechanical sweeps** between the most recent `in-progress` transition and HEAD. The sweep edits change `spec.md`'s diff but not the `status:` line; the walker's match criterion (`+status: in-progress` in the commit diff) correctly identifies status-flipping commits and ignores sweep commits.
- **`read-tasks` on a tasks.md whose phase headers use a different label** (e.g., `## Stage 1 —` instead of `## Phase A —`). Treat any `## ...` header above the first `### N.` task as a phase container; the label is informational, not load-bearing. Document the detection rule so future tasks.md authors can predict whether their structure will be parsed as phased.

## Open Questions

- **`append-task` slug derivation when no slug is supplied.** If the caller omits `slug` AND omits `body`, can the primitive derive the slug from the most-recently-created scenario file in the same feature directory? That coupling between two primitives feels fragile. Lean: require explicit `slug` when `body` is omitted, refuse with a clean operational error otherwise. Resolve during clarify.
- **Phase-default heading text.** When `append-task` creates the default follow-on phase (no `parent-heading` argument, phased structure detected), what label does it use? Existing precedent in 023 is `## Phase C — Follow-on scenarios`. Should the primitive hardcode `Follow-on` or accept a label argument? Lean: hardcode `Follow-on scenarios`, override with the argument. Resolve during clarify.

## Resolved Questions

*None yet.*
