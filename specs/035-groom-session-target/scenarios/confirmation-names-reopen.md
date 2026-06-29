---
section: "Behavior"
---

# Confirmation-names-reopen

## Context

035 makes `/{project}:groom` set the session target when it routes an item to an existing spec, and the `reopen-done-spec-on-scenario` scenario makes groom's Step 4 reopen a matched `done` spec `done → in-progress` when it adds a scenario. The per-item routing confirmation groom already requires names the scenario and the session target — e.g. "Create a scenario under `NNN-slug` and set it as the session target? (Y/n)" — but it does **not** name the reopen, even though that single confirmation is treated as the consent for the reopen. `/{project}:amend`'s scenario route, by contrast, prompts explicitly before flipping a `done` spec ("Revert status to `in-progress`...?"). As written, groom's `done → in-progress` mutation is surfaced to the operator only afterward, in the Completion summary.

## Behavior

When groom's Step 4 routes a scenario to a matched spec whose status is `done`, the per-item routing confirmation names the reopen alongside the scenario and target — e.g. "Create a scenario under `NNN-slug`, reopen it to `in-progress`, and set it as the session target? (Y/n)". The operator therefore consents to the `done → in-progress` mutation before it happens, rather than only seeing it reported in the Completion summary afterward. The reopen stays part of that single confirmation — no separate prompt is added, preserving groom's prompt count (procedural-fidelity) while matching `/{project}:amend`'s practice of naming the reopen at the consent point.

## Edge Cases

- When the matched spec is **not** `done` (`draft`, `clarified`, `planned`, or `in-progress`), there is no reopen, so the confirmation is unchanged — it names only the scenario and the target.
- A Step 3 spec-edit route never reopens (it carries its own §spec-lifecycle back-edge), so its confirmation never names a reopen.
- The Completion summary still names any reopened spec — naming the reopen in the confirmation is additive pre-action transparency, not a replacement for the after-the-fact report.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
