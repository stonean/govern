# 035 — Groom sets the session target from the routed item Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Set the target in the routing branches

- [x] In `framework/commands/groom.md` Context, replace the "a target is not required" note with: groom operates across all specs and does not *require* a target, but it now *sets* the session target when it routes an item to an existing spec (so a follow-on `/gov:amend` / `/gov:implement` needs no manual `/gov:target`).
- [x] In Step 3 (spec edit), after the routing is confirmed, set `.govern.session.toml` to the matched feature (`feature` + `path`), preserving any existing `cli-config-dir` (read first, carry forward; tempfile + rename), per the markdown-only session-write pattern in `specify.md` / `amend.md`.
- [x] In Step 4 durable-requirement branch (scenario creation), after the scenario + task are written, set the target to the matched feature **plus the new scenario** (`feature` + `path` + `scenario` + `scenario-path`), same preservation semantics.
- [x] State explicitly that Step 1 (rule item), Step 2 (new spec → `/gov:specify`), and the Step 4 chore branch set **no** target. (New `### Setting the session target` subsection; Scope Boundaries updated to note the `.govern.session.toml` write.)
- Done when: both existing-spec branches write the target with cli-config-dir preserved; the non-spec branches write nothing.

## 2. Name the target in the confirmation and report it at completion

- [x] Reword the per-item routing confirmation so it names the target it will set (e.g. *"Create a scenario under `NNN-slug` and set it as the session target? (Y/n)"*); add no separate target prompt.
- [x] Add a Completion line naming the final session target, or "session target unchanged" when no groomed item set one.
- Done when: the confirmation names the target and the completion summary reports it.

## 3. Validate

- [x] Regenerate `.claude/commands/gov/groom.md` (`gen-claude-commands.sh`) cleanly.
- [x] `npx markdownlint-cli2`, `scripts/lint-*.sh` (incl. `lint-procedure-parseability.sh`), and `scripts/audit/*` pass.
- Done when: all lints/audits green and the generated copy is in sync.

## 4. Review and complete

- [x] Run `/gov:review` over the change set; resolve any MUST findings. (0 MUST / 0 SHOULD; 1 incidental issue captured to the inbox — groom not reopening done specs on scenario add — see `review.md`.)
- Done when: `/gov:review` reports no blocking violations and the spec can advance to `done`.

## 5. Reopen a done spec to in-progress when groom adds a scenario (Step 4)

- [ ] Implement the behavior described in `scenarios/reopen-done-spec-on-scenario.md`
- Done when: `framework/commands/groom.md` Step 4 durable-requirement branch performs a `set-status` `done → in-progress` on the matched spec when its status is `done`, alongside creating the scenario, appending the task, and setting the session target; specs not in `done` are left unchanged. The behavior mirrors `/gov:amend`'s scenario route and §spec-lifecycle's "Backward via new scenario" edge, and the generated `.claude/commands/gov/groom.md` copy regenerates cleanly.
