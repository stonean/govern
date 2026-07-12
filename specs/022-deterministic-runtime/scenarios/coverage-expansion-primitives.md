---
section: "Follow-on scenarios"
---

# Coverage-expansion-primitives

## Context

The 2026-07-11 coverage review found ~22 of 100 parsed command steps are deterministic-but-still-prose — mechanical work the LLM still performs by hand. The residue concentrates in three shapes: output rendering from primitive payloads, conditionals/loops the interpreter cannot express, and hot loops with hand work. Several map to a shipped-but-unused primitive or a missing one, each passing the runtime-eligibility criteria (deterministic, currently mechanical, degradation-not-failure when absent):

- **groom step 8** removes an item from `specs/inbox.md` with a host `Edit` — the only groom write not primitive-backed (the shipped `append-inbox` appends only).
- **implement steps 7 and 12** re-derive, per task, a `git diff` against the spec dir's first commit filtered to paths outside `specs/{feature}/` (the cross-spec impact); step 12's prose self-declares "no primitive owns this filter yet." The same diff feeds `/gov:review`'s captured-issues section.
- **implement step 13** re-states the `review:` block gate branch by hand on every completion attempt, and invokes `npx markdownlint-cli2` directly instead of the existing `lint-markdown` primitive.
- **amend's question route** appends to `## Open Questions` with a normalized-whitespace dedup and a same-write status back-edge, with no primitive (asymmetric with the scenario route's `create-scenario` + `append-task`) — the blocker to de-legacying `amend.md`.
- **plan.md's** template-copy and existing-artifact detection have no primitive (`create-feature` covers only `spec.md`).
- **status.md** spends five of its six steps on LLM-side rendering of the `dashboard` payload (preamble, table, counts/callouts, references readout).

## Behavior

New primitives, each wired at every site per the AGENTS.md six-site rule (schema Args/Result, primitive module + `mod.rs`, MCP server + `TOOL_NAMES`, `PRIMITIVE_NAMES`, interpreter dispatch, CLI enum, `runtime-tools.txt`, data-model entry, regenerated configure permission blocks):

- `remove-inbox-item` — dedup-aware removal of one bullet from `specs/inbox.md` (the complement of `append-inbox`, same atomic-write contract); groom step 8 invokes it.
- `diff-cross-spec` (or a mode on `derive-boundary`) — the `git diff` against the spec dir's first commit filtered to paths outside `specs/{feature}/`, plus the inbox-window additions; implement steps 7/12 and the review captured-issues section invoke it.
- `check-review-gate` — evaluates the spec `review:` block plus the feature-dir markdown lint into a verdict and the canonical blocked message; implement's completion gate invokes it, replacing both the hand-walked branch and the raw `npx markdownlint-cli2` call (which becomes `lint-markdown`).
- `append-question` — appends to `## Open Questions` with normalized-whitespace dedup and the same-write `done|clarified|planned|in-progress → draft` back-edge; amend's question route invokes it.
- `create-plan-artifacts` — copies the plan/tasks/data-model templates and reports pre-existing files (mirroring `create-feature`, atomic and mode-preserving); plan.md invokes it.
- `dashboard` gains a rendered-markdown field (preamble + table + counts/callouts + references) the host may restyle, absorbing status.md's rendering steps.

## Edge Cases

- Each rewritten step keeps its current prose as the documented markdown-only fallback.
- The `dashboard` render field is returned data the host may restyle, never stdout printing — it stays inside the runtime boundary (no user-facing rendering owned by the runtime).
- `remove-inbox-item` reports a not-found removal as a domain outcome (no error), mirroring `append-inbox`'s `deduped`.
- `append-question`'s dedup uses the same normalized-whitespace comparison amend already specifies, so the runtime and markdown-only paths agree.

## Open Questions

*None — all resolved.*

## Resolved Questions

- `diff-cross-spec` as its own primitive versus a mode on `derive-boundary`? **Own primitive, shared walk.** The two results share only the diff base — boundary globs versus sibling-spec paths + inbox bullet lines — so a mode would be a result-shape union for no gain; the expensive piece (the first-commit revwalk) is shared as the `pub(crate)` `first_commit_for_prefix` helper instead. `/gov:review`'s captured-issues section stays on `compute-review-scope`, whose window starts at the in-progress transition (review wants the current work window, not the feature's whole history); `diff-cross-spec` diffs against the working tree so implement's per-task summary sees uncommitted captures.
- All six in one pass, or an interpreter loop/conditional scenario first? **Incremental, no new construct needed.** The six landed one primitive per commit (remove-inbox-item, create-plan-artifacts, check-review-gate, append-question, diff-cross-spec, the dashboard `rendered-markdown` field). Per-step conditionals stayed host judgment on primitive results — the `check-stuck` precedent: the walker dispatches and merges; the host halts, prompts, or re-invokes (e.g. `create-plan-artifacts` with `overwrite: true` on the confirmed replace branch). No interpreter loop/conditional construct was required.
