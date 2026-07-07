---
section: "Follow-on scenarios"
---

# Review-exec-wiring — thread primitive results so `/gov:review` runs under `runtime exec`

## Context

The [review-runtime-acceleration](review-runtime-acceleration.md) scenario landed the four review primitives (`discover-rule-files`, `process-waivers`, `compute-review-scope`, `write-review`), the `performReview` extension point, and the `/gov:review` command rewrite. These work end-to-end on the **MCP/host path**, where the host threads each primitive's result into the next step.

The **`runtime exec` path is not yet wired**: the interpreter walker discards primitive results (`dispatch_primitive` returns a value, but `handle_primitive` maps `Ok(_) => Ok(None)` in `runtime/src/interpreter/mod.rs`). So on `runtime exec review`, the `scope`/`diff-base` from `compute-review-scope` and the `selected`/`rules-dir` from `discover-rule-files` never reach `build_perform_review_request` or `write-review` — each pass would receive an empty scope and empty rule set, and `write-review` would receive no scope. Only `performReview` findings accumulate today (the 45e walker change), because that path was added explicitly. Surfaced during task 45i (integration + release prep) and deferred here as its own unit so the cross-command golden re-bless is deliberate rather than a tail-of-45i cram.

## Behavior

- **Thread primitive results into the walker context.** After a primitive step dispatches, merge the structured result's top-level keys into the walker context, so a later step's payload builder and later primitives can read prior results — `compute-review-scope`'s `scope` and `diff-base`, and `discover-rule-files`'s `selected` and `rules-dir`, feed `build_perform_review_request`; `write-review` reads `diff-base` plus the accumulated `findings`. Define the merge policy explicitly: which keys merge, and whether a primitive result may overwrite a session-seeded key (default: preserve seeded keys such as `write-boundary`, or namespace results to avoid the collision entirely).
- **`/gov:review` exec golden / parity fixture.** Add a git-backed fixture repo (a spec, rule files, a plan with `Affected Files`, and a `stdin.jsonl` of `performReview` responses) plus a parity golden asserting the emitted stream: `compute-review-scope` → `discover-rule-files` → `process-waivers` → five `performReview` requests carrying the populated scope + rules → `write-review` → `complete`. Mirror the `implement-basic` / `status-basic` fixture shape and the `{{runtime-version}}` placeholder.
- **Re-bless the existing exec goldens intentionally.** Merging results changes the request payload for every command whose Instructions place a primitive step before an extension or a later primitive (implement, status, target, analyze, plan, specify, govern). Re-bless each affected golden and confirm every diff is an expected *added context field* — never a semantic change.

## Edge Cases

- **Result-key collisions.** Two primitives (or a primitive and a session-seeded value) sharing a top-level key — e.g. a primitive `path` result vs. the session `path`, or a primitive `feature` vs. the seeded `feature`. Pick last-write-wins or a per-primitive namespace and state it; never silently clobber a load-bearing seeded key like `write-boundary`.
- **Conditional execution stays host-side.** The empty-scope skip and the dimension-flag pass-skipping are host behaviors per review-runtime-acceleration's Edge Cases; the exec walker runs all nine steps unconditionally, so an exec review on an empty scope still emits five `performReview` requests. The fixture accounts for this. Adding walker-level conditional-step support (so exec matches the host path's skipping) is a separate follow-on, out of scope here.
- **Golden churn is bounded and inspected.** Quantify the set of goldens that change and assert — by reviewing each diff, not by blanket `BLESS=1` — that none changes semantically; only added context fields appear.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
