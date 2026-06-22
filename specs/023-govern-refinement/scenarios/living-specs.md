---
section: "Follow-on scenarios"
---

# Living-specs

## Context

Spec 023 implemented multiple framework refinements (validate→analyze rename in §4; /capture→/specify and /elaborate→/amend consolidations in §§2–3; spec-and-plan.md removal in §1). To avoid rewriting completed spec bodies during the sweep, 023 relied on the "frozen archaeology" exception in `framework/constitution.md` §drift-prevention and added a `specs/README.md` §Past Renames cross-reference (AC #22) so historical references stayed discoverable.

The result is internally contradictory. `AGENTS.md` line 42 already requires "no dead references in live artifacts" and lists `framework/`, `scripts/`, `runtime/`, `.github/`, `docs/`, `README.md`, and `AGENTS.md` as live — explicitly carving out `specs/NNN-*/` as frozen archaeology. A grep across done specs surfaces ~14 carrying dead references (`/gov:validate`, `/capture`, `/elaborate`, `spec-and-plan`) that the framework ships on purpose. Adopters reading a done spec encounter terminology that no longer matches the installed command files; the Past Renames table is a band-aid that works only if the reader knows to consult it. `AGENTS.md` §Design Principles also forbids designing the framework around author discipline ("remember not to rename things"), but the Past Renames mechanism depends exactly on that.

This scenario removes the carve-out. Spec bodies become living documents representing current state; git history is the historical record of what was written when.

## Behavior

- `framework/constitution.md` §drift-prevention drops the "frozen archaeology" exception. The §spec-lifecycle rule extends: any edit to a done spec body reverts the spec to `in-progress` via the same `/gov:amend` back-edge that already exists for scenarios. The spec advances back to `done` through the normal pipeline.
- `AGENTS.md` line 42's rename rule drops the `specs/NNN-*/` carve-out. The "no dead references in live artifacts" sweep is uniform across all live artifacts, including done-spec bodies.
- `specs/README.md` §Past Renames is deleted. Git log is the historical record of what was written when; the spec body always shows current truth.
- Decision rationale preservation relies on artifacts that already accumulate over a spec's lifetime — Resolved Questions sections, plan.md Trade-offs, review.md findings, and git history. No new artifact tier is introduced.
- As part of this scenario's implementation, sweep the done specs currently carrying dead references and bring them current. Initial grep targets: `/gov:validate` and `/{project}:validate` and `validate.md` (renamed in 023 §4 → `/gov:analyze`, `analyze.md`); `/capture` (consolidated in 023 §2 → `/specify`); `/elaborate` (consolidated in 023 §3 → `/amend`); `spec-and-plan.md` (deleted in 023 §1). The exact file list and rename mapping is finalized during clarify or planning.
- **Mechanical-vs-meaningful boundary** (resolves the back-edge-skip rule): an edit to a done-spec body is **mechanical** (no back-edge) iff every change in the diff is the same find-and-replace token substitution, applied uniformly across all live artifacts per `AGENTS.md` line 42's rename-rule scope (`framework/`, `scripts/`, `runtime/`, `.github/`, `docs/`, `README.md`, `AGENTS.md`, and — after this scenario lands — `specs/NNN-*/`), and the substitution maps a deprecated label (slug, capability, command, identifier, parenthetical descriptor) to its current label. Anything else — new scope, changed semantics, factual corrections, restructuring, edits scoped to a single spec — is a **meaningful edit** and triggers the back-edge via the same `/gov:amend` flow used today for scenarios. The distinction is determinable from the diff alone (a reviewer or future tool can verify uniform-substitution shape), so the rule does not depend on author judgment. Future tooling (a `/gov:analyze` check on done-spec diffs, or a dedicated `/gov:rename old-name new-name` command) could automate enforcement; not in scope for this scenario.

## Edge Cases

- The renaming sweep is independent of pipeline state. A rename is mechanical and does not require a clarify/plan/implement cycle on the swept specs — the back-edge applies only when the spec body is being *meaningfully* edited, per the mechanical-vs-meaningful boundary defined in Behavior.
- 023's own body becomes one of the swept files — its AC #21 cites the "frozen archaeology" exception as the rationale for surviving dead references, and the description text references `/validate`. The new rule applies uniformly, including to the spec that originally introduced the exception. The audit trail of "why was this once an exception?" lives in git history and in 023's plan.md / review.md.
- Commit churn on otherwise-stable spec files is a known cost. The trade-off is accepted: adopters benefit more from current-truth bodies than from immutable historical bodies, especially since git log preserves the historical record at zero additional artifact cost.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

- **Mechanical-vs-meaningful boundary.** Where is the line between an edit that triggers the `done → in-progress` back-edge and a rename sweep that bypasses it? Resolved as: an edit is mechanical (no back-edge) iff every change in the diff is the same find-and-replace token substitution applied uniformly across all live artifacts per the `AGENTS.md` rename rule's scope, and the substitution maps a deprecated label to its current label. Everything else is meaningful and triggers the back-edge. The rule is derived from diff shape, not author judgment — a reviewer or future tool can verify uniform-substitution from the diff alone. Failure-mode is self-correcting: a meaningful edit accidentally tagged as mechanical produces a non-uniform diff caught at PR review; a mechanical sweep accidentally tagged as meaningful just goes through the pipeline unnecessarily (wasteful, not incorrect). The Behavior section was updated to incorporate the rule as a new bullet; future tooling for automation (a `/gov:analyze` check on done-spec diffs, or a `/gov:rename` command) is noted as out-of-scope for this scenario.
