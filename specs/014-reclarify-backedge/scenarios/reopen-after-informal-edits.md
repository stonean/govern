---
section: "/ask status mutation"
---

# Reopen-after-informal-edits

## Context

Spec 014 §`/ask` status mutation defines the `done`-spec branch: `/ask` refuses and reports "Spec is `done`. Run `/{project}:ask` to capture this as a scenario instead." Per [023](../../023-govern-refinement/spec.md) the redirect resolves to `/ask`'s own scenario branch — supplying a scenario-shaped input flips status to `in-progress` via the `done → in-progress` back-edge.

That flow is correct when the user is starting from a fresh observation: they have something to add, they describe it, `/ask` classifies it, the scenario file is created, status flips. The gap surfaces when the agent (acting on the user's behalf during conversation) has *already* informally edited the feature directory — adding scenario files under `scenarios/`, appending lines to `spec.md`, or appending tasks to `tasks.md` — and the spec status is the only remaining inconsistency. The work that justifies the re-open exists on disk; the user just needs the status flipped.

`/ask`'s current contract does not have an entry point for this case. The refinement loop requires an input string to classify, and producing a synthetic input ("re-open because I edited scenarios/foo.md on disk") is friction the user notices: the classifier will either insist on a scenario shape (creating a second scenario file the user does not want) or accept the input as a no-op, neither of which matches intent.

Surfaced during the gvrn 0.10.0 session-consolidation cycle, where scenario edits were made conversationally and the agent then prompted `/gov:ask` to handle the re-open — which led directly to this UX gap.

## Behavior

Two fixes are viable; this scenario is the place to pick one (or accept both) during implementation. Both keep the spec lifecycle invariant intact — `done` is reverted only when work that should be tracked has been added.

### Option A — agent-side: skip `/gov:ask` when re-open is the only intent

The agent (when acting in conversation on the user's behalf) does not prompt `/gov:ask` for a re-open that has no new input to classify. Instead, it invokes the `set-status` MCP primitive directly to flip status from `done` to `in-progress`, then reports the on-disk delta (scenario files newly under `scenarios/`, modified `spec.md` / `tasks.md`) so the user sees what triggered the re-open. The user keeps a single visible action — the conversational request that already added the work.

Trade-off: the re-open path becomes invisible to users who *don't* go through the agent (e.g., scripted hosts, future `/govern`-style automation). They still hit `/gov:ask` with no natural entry point.

### Option B — command-side: `/gov:ask` detects on-disk delta and treats it as an explicit re-open trigger

`/gov:ask`'s refinement loop gains a precondition check on `done` specs: before requiring classifier input, it inspects the feature directory for uncommitted (or simply present) scenario files that the live `spec.md` / `tasks.md` does not yet reference, and for modified-since-`done` edits to those files. If a delta is found, `/gov:ask` offers a re-open branch:

- Display: prior status (`done`), files contributing to the delta (newly-present `scenarios/*.md`, modified `spec.md` or `tasks.md` lines), the timestamp of each.
- Prompt: "Spec is `done` but the feature directory has un-tracked scenario or task edits. Revert status to `in-progress` to reflect the on-disk delta?"
- **Confirm:** revert frontmatter `status` to `in-progress`, no new scenario file created, summary names the on-disk delta. The user runs `/gov:plan` or `/gov:implement` next.
- **Decline:** stop with no changes. The spec remains `done` and the on-disk delta is left alone.

If no delta is found, `/gov:ask` falls through to the existing scenario-classification flow (the redirect today).

Trade-off: the detection logic has to define "delta" carefully — git-uncommitted only? files modified since the `review.last-run` timestamp? files present that are not linked from `spec.md`? Each rule has corner cases (e.g., a scenario file deliberately added during the `done` window for future-spec planning).

### Recommended pick

Option B is the more general fix — it gives every host (agent, scripted, future automation) the same re-open surface and keeps `/gov:ask` as the single status-mutation entry point per the spec 014 ownership model. Option A is a useful agent-behavior tweak even if B lands, since it removes a redundant prompt round-trip; if both are implemented, A handles the conversational case efficiently and B is the durable safety net.

## Edge Cases

- **No on-disk delta, `done` spec** — Option B falls through to today's behavior: classify the user's input as a scenario via the existing `done → in-progress` back-edge. No change.
- **Delta exists but the user intends to add a *new* scenario** — Option B's prompt is offered before classification, so the user can decline the re-open prompt and continue into the scenario branch with the new input. The prompt phrasing must make this opt-out obvious.
- **Delta detection definition picks "files modified since `review.last-run`"** — a spec that has never been reviewed has no `last-run` timestamp. Fall back to the spec's git-blame mtime for the `status: done` frontmatter line, or treat the absence of `last-run` as "no review baseline, no delta detection" and require the user to specify intent explicitly.
- **Delta is a deliberate "future planning" scenario file the user added while leaving the spec `done`** — the user declines the prompt. The file stays, the spec stays `done`. If this pattern recurs, the delta heuristic may need to exclude scenario files whose frontmatter `section:` does not match any current spec section (signaling the scenario is forward-looking, not back-edge work).
- **Option A applied where the user actually wants to add *more* scenario content beyond what's on disk** — the agent should still route to `/gov:ask` for the new input after flipping status. Skipping `/gov:ask` is a re-open shortcut, not a replacement for the scenario-add flow.

## Open Questions

*None — captured during scenario authoring; both options surfaced as alternatives for the implementation pick.*

## Resolved Questions

- **Which option to implement?** Both A and B, per the scenario's "Recommended pick." Option B (command-side delta detection in `framework/commands/ask.md`) is the durable safety net that gives every host the same re-open surface; Option A (agent-side `set-status` shortcut in `AGENTS.md`) removes a redundant prompt round-trip in the conversational case.
- **How to define "delta" for Option B?** `git status --porcelain` scoped to `specs/{feature}/scenarios/`, `specs/{feature}/spec.md`, and `specs/{feature}/tasks.md`. Untracked files under `scenarios/` and modified `spec.md` / `tasks.md` count. Rationale: cheapest signal that doesn't depend on `review.last-run` (which may be unset), matches the scenario's origin story (the gvrn 0.10.0 session where edits were conversational and uncommitted), and the "deliberate future-planning file" edge case is handled by the user declining the prompt. The other candidates (modified-since-`review.last-run`, files-not-linked-from-`spec.md`) were rejected because each requires fallback logic for missing metadata or risks false negatives on legitimately-unreferenced scenarios.
