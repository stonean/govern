---
spec: 035-groom-session-target
reviewed-at: 2026-06-29T01:10:12Z
reviewed-against: acffa851a0cb284d9766b0006930229955fb1da2
diff-base: 98f859520f2672b58830911d891f6f9eeb14a98e
must-violations: 0
should-violations: 0
low-confidence: 1
captured-issues: 0
skipped-passes: []
---

# Review — 035-groom-session-target

## Summary

Markdown-tier change set: prose edits to one slash-command source
(`framework/commands/groom.md`) and its regenerated `.claude/commands/gov/groom.md`
copy — no application code. This run covers the work window reopened at
`98f8595` (groom added the `reopen-done-spec-on-scenario` scenario and Task 5);
the prior review (`reviewed-against: 43e4ad0`) covered Tasks 1–4 and captured the
very issue Task 5 now resolves — groom Step 4 reopening a `done` spec when it
adds a scenario. No loaded rule's Verification trigger fires against
command-source prose, and the reuse/efficiency/simplicity passes find nothing
actionable: the reopen references the §spec-lifecycle back-edge `/gov:amend`
already performs rather than duplicating logic. The quality pass confirms the
scenario's behavior is fully and consistently specified (Context ↔ Scope
Boundaries ↔ Step 4 ↔ the dedicated subsection ↔ Completion) and surfaces one
low-confidence transparency nuance (below). **0 MUST violations — not blocking;
the spec may advance to `done`.**

Rule-file selection for this run: `[rules] surfaces` unset in govern's own
`.govern.toml`, so step 5 fell back to detected-stack derivation;
`[review] tech-stack-verified = true` skipped the alignment check.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

### quality — done-spec reopen is not named in the routing confirmation (confidence 60)

- **File**: `framework/commands/groom.md` (§Groom each item step 4 confirmation; §Reopening a `done` spec)
- **Finding**: The new subsection states the per-item routing confirmation
  ("Create a scenario under `NNN-slug` and set it as the session target? (Y/n)")
  "is the consent for the reopen as well," but that confirmation names only the
  scenario and the target — not the `done → in-progress` reopen. By contrast,
  `/gov:amend`'s scenario route prompts explicitly ("Revert status to
  `in-progress`...?") before flipping. The mutation is surfaced after the fact in
  the Completion summary (so it is not silent), but a reader consents to the
  reopen without it being named at the prompt.
- **Auto-fixable**: no
- **Suggested fix**: When the matched spec is `done`, have the routing
  confirmation name the reopen too — e.g. _"Create a scenario under `NNN-slug`,
  reopen it to `in-progress`, and set it as the session target? (Y/n)"_. Aligns
  groom's pre-action transparency with `/gov:amend`. Optional follow-up via
  `/gov:amend`; not blocking.

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no inbox additions since diff-base._

## Skipped passes

_None — all five passes ran._
