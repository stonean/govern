---
section: "Follow-on scenarios"
---

# Settings-template-bash-allowlist

## Context

The `settings_template` field in `framework/bootstrap/govern.md`'s agent registry (lines 46–48) seeds the per-adopter `{config_dir}/settings.local.json` at first `/govern` run and gets merged into existing files on subsequent runs (see Behavior steps 1–3 in the same doc). The template's `permissions.allow` array determines which Bash command patterns run *during bootstrap* without prompting the operator.

The current Claude row's allow list covers `Bash(curl *)`, `Bash(ls *)`, `Bash(tar *)`, `Bash(mktemp *)`, plus several `Read(...)` entries for temp directories. Spec 015's smoke test surfaced that `/govern` actually invokes more Bash commands than this list covers — observed prompts during a routine bootstrap run included `Bash(git status *)` (used by Frontmatter Migration's precheck) and `Bash(awk '{print $5, $9}')` (used by the post-scaffolding summary). Adopters accept these prompts and they accumulate in `settings.local.json`, creating per-adopter drift from what the framework would have shipped if the gaps weren't there.

This is pre-existing — not caused by 015 — but 015's smoke test was the first time we exercised the full bootstrap path against a fresh adopter and saw the prompt accumulation pattern end-to-end.

## Behavior

Audit the bootstrap allowlist against actual `/govern` runtime behavior and extend the canonical `settings_template` in both agent registry rows to cover every Bash command pattern a routine bootstrap exercises.

- **Audit pass.** Trace through a full `/govern` run (greenfield bootstrap of a fresh adopter project) and enumerate every distinct Bash command pattern the procedure invokes. Sources: `framework/bootstrap/govern.md` body (the Instructions section), plus any sub-procedures it composes (e.g., Frontmatter Migration, post-scaffolding summary, workflow scaffolding).
- **Confirmed gaps to add.** At minimum: `Bash(git status *)`, `Bash(awk *)`. These are confirmed missing per 015's smoke test.
- **Suspected gaps to verify.** `Bash(grep *)`, `Bash(head *)`, `Bash(cat *)` — common patterns that may also prompt. Add the ones that prompt during the audit pass; leave the others out.
- **Both rows in sync.** Every pattern added to the Claude row's `settings_template` JSON must have a semantically equivalent entry in the Auggie row's `settings_template` in Auggie's native format. The "parity across configure formats" contract from spec 012's body governs this — the audit covers both rows in one pass.
- **No regressions.** Existing entries stay. The merge behavior (Behavior step 2 in `framework/bootstrap/govern.md`) ADDs missing entries without deduplicating or reordering, so the canonical list grows monotonically.

## Edge Cases

- **Adopter has accumulated entries from prompts.** The merge logic adds missing entries without touching what's already there, so adopters who previously accepted the prompts and now have `Bash(git status *)` in their local file are no-ops — their entry stays, the framework's entry is recognized as already present.
- **Auggie's allow shape differs structurally.** Auggie's `settings.local.json` uses `toolPermissions` with `shellInputRegex` per spec 012 (see the relevant Behavior section). The equivalent of `Bash(git status *)` is a `launch-process` permission with the matching regex. The audit must produce both forms; a single Bash pattern in the Claude row that has no Auggie equivalent is a finding to resolve before landing.
- **Pattern over-broadness.** `Bash(git *)` would cover `git status`, `git diff`, `git push`, `git reset`, etc. `git push` and `git reset --hard` are operations spec 012 (and AGENTS.md) treats as risky/externally-visible. Prefer narrower patterns (`Bash(git status *)`, `Bash(git log *)`, `Bash(git diff *)`) over `Bash(git *)`. The audit captures only what `/govern` actually invokes; risky git operations are not in `/govern`'s path, so they shouldn't end up in the list.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
