---
status: draft
dependencies: [020-code-review, 024-rule-loader]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 025 — Rule-file opt-out via `.govern.toml`

Add a narrow `.govern.toml` `[[review.disabled-rule-files]]` opt-out so an adopter whose stack matches a rule file's surface — but whose project is not yet ready to enforce that file's rules — can exclude the file from `/gov:review` loading with a recorded reason. Derivation per [024 — stack-aware rule-file loader](../024-rule-loader/spec.md) remains the default; this spec only adds the explicit override.

## Motivation

[Spec 024](../024-rule-loader/spec.md) makes rule-file selection derived: a project with frontend code automatically gets [`accessibility-frontend.md`](../../framework/rules/accessibility-frontend.md) and [`performance-frontend.md`](../../framework/rules/performance-frontend.md) enforced; a project with a backend API automatically gets [`api-backend.md`](../../framework/rules/api-backend.md) enforced. That default is correct — the alternative (every rule file is opt-in) is the silent author-discipline failure mode AGENTS.md:58 exists to prevent.

But derivation can't see intent. A team building an internal admin UI may not be ready to enforce WCAG AA. An early MVP may not have published an OpenAPI schema yet. An adopter migrating onto `govern` mid-project will hit a backlog of rules they cannot triage all at once.

Without an override, those teams have three bad options:

1. **Pin the file in `.govern.toml [pinned] files` and delete its content locally.** Defeats the purpose of shipping rules — the next `govern` update can't push improvements to a file the adopter is now maintaining.
2. **Add a waiver for every finding.** Waivers are per-`(rule, file)` and intended for narrow, justified exceptions — using them to suppress an entire rule file means dozens of waiver entries that obscure the small number of waivers that actually mean something.
3. **Ignore `/gov:review`'s `blocking: true` output.** Detaches the team from the gate the framework provides and removes the protection from rules they DO care about.

The fix: a deliberate, recorded, file-level opt-out. `.govern.toml` already houses adopter-side database state (see [`framework/constitution.md`](../../framework/constitution.md) and `AGENTS.md` line 42). A new `[[review.disabled-rule-files]]` array-of-tables key lets adopters say "we know this file applies; we're not enforcing it yet; here's why." The reason is mandatory — without one, the entry is silently typo-able to a meaningless on/off switch.

## Acceptance Criteria

- [ ] `.govern.toml` accepts `[[review.disabled-rule-files]]` as an array-of-tables. Each entry has two required fields:
  - `file` — the basename of a file in `framework/rules/` (e.g., `"accessibility-frontend.md"`)
  - `reason` — a free-text justification (non-empty; trimmed length ≥ 16 characters to discourage placeholder text)
- [ ] `/gov:review` reads `[[review.disabled-rule-files]]` during rule-file selection and skips any listed file regardless of stack detection. Skipped files emit a one-line stdout notice at the start of the run:

  ```text
  disabled-rule-file: <filename> — <reason> (.govern.toml)
  ```

  The notice is the point: silent skipping is forbidden.
- [ ] An entry whose `file` does not exist in `framework/rules/` produces a one-line warning (`unknown disabled-rule-file: <filename> (no such file in framework/rules/)`) but is not a fatal error — operators may temporarily list a file that has been renamed or moved.
- [ ] An entry missing `file` or `reason`, or whose `reason` fails the minimum-length check, is **skipped with a warning** (same pattern as malformed waivers, per [`framework/commands/review.md`](../../framework/commands/review.md) §Malformed and duplicate waivers). The entry is NOT auto-removed; the operator must clean it up. Same reasoning the existing waiver design uses: malformed entries are operator-authored state, not garbage for the framework to collect.
- [ ] Duplicate entries (same `file` listed twice) emit a warning and only the first applies — same pattern as duplicate waivers.
- [ ] `/gov:status` surfaces the disabled list (when present) in the pipeline dashboard, so the override is visible at-a-glance and doesn't hide in `.govern.toml`.
- [ ] `/gov:analyze` does NOT error on the new key. The key is a `.govern.toml` extension owned by this spec, not a spec-frontmatter change.
- [ ] The mechanism is uniform across all rule files. Adopters CAN disable [`security-backend.md`](../../framework/rules/security-backend.md) or [`security-frontend.md`](../../framework/rules/security-frontend.md) — the reason field is the audit trail. The framework does not enforce a "security files cannot be disabled" carve-out: enforcing it would require a hardcoded list of "real security" files that drifts from reality, and dropping security rules is a high-stakes decision that the reason field already makes visible. PR review and the operator's own policy are the safeguards, not the framework.
- [ ] Documentation: the `[review]` section schema in the relevant config-decisions spec (or its successor) is updated to describe the new key. Reference: AGENTS.md line 42 — `.govern.toml` keys are documented in the spec that owns them, not retro-added to earlier specs.

## Non-goals

- **Per-rule-id disabling.** The unit is the rule file. Finer granularity invites cherry-picking individual MUST violations, which is what waivers already exist for. Two mechanisms for the same problem create confusion about which to reach for.
- **Per-spec exemption (e.g., "skip `accessibility-frontend.md` for spec 042").** Defeats the gate's purpose — a rule that doesn't apply to spec 042 also doesn't apply to spec 043 next sprint. Project-wide disablement is honest; per-spec is laundering.
- **A "until-date" auto-expire.** Tempting but failure-mode-asymmetric — if the date is in the past, does `/gov:review` re-enable enforcement silently (breaks a build the team wasn't expecting) or warn-only (everyone learns to ignore the warning)? Both options reintroduce the disciplines this spec is trying to remove. Operators who want a sunset commitment write it into the `reason` and remove the entry when ready. PR review enforces it.
- **Encoding "approval" or "second-author" requirements.** Same reasoning as the waiver design ([spec 020](../020-code-review/spec.md) Resolved questions): govern has no runtime that can verify a second person approved; encoding it in the toml would be performative. Adopters whose policy requires co-authorization layer their own fields onto the entry — the §text-first-artifacts open-schema rule guarantees `/gov:review` will not error on unknown fields.

## Affected files

| File | Change | Strategy |
| --- | --- | --- |
| `framework/commands/review.md` | edit — §Inputs (add the new config key), §Behavior step 5 (consult the disabled list during selection), §Output (the new stdout notice) | update |
| `framework/commands/status.md` | edit — surface the disabled list in the pipeline dashboard | update |
| `framework/commands/analyze.md` | edit — extend `.govern.toml` validation tolerance to accept the new key | update |
| `framework/constitution.md` | edit — §rules anchor (or wherever rule-file loading is described) gets a brief mention of the override mechanism | update |
| `framework/templates/project/govern-toml.md` (or equivalent template/example) | edit — add a commented-out example block showing the schema | update |
| `scripts/lint-govern-toml.sh` (if it exists) | edit — accept the new key | update |

## Open Questions

- The 16-character minimum on `reason` is arbitrary — long enough to discourage `"todo"` or `"later"`, short enough to not block a legitimate `"WCAG deferred to Q3"`. Confirm during `/gov:clarify` whether this is the right threshold or whether the framework should be looser (just non-empty) and let PR review enforce quality.
- Does this key go under `[review]` (matching the existing `[review] tech-stack-verified`) or under a new `[rules]` section? `[review]` keeps the related keys together; `[rules]` is more semantically accurate but fragments adopter-side state. Lean: `[review]`.
- Should `/gov:review` exit non-zero if any disabled file lists `reason: ""` or fails the minimum check, even when the rest of the run is clean? Lean: no — warnings stay warnings, only MUST violations block. Matches the waiver malformed-entry behavior.
- Should there be a CLI shortcut for adding a disabled entry (e.g., `/gov:review --disable accessibility-frontend.md --reason "..."`)? Symmetric with `--waive`, but the use case is rarer and editing `.govern.toml` directly is fine. Defer to a follow-up spec if demand emerges.
