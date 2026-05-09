---
status: done
dependencies: [005-workflows]
---

# 019 — Config-Persisted Decisions

`.govern.toml` is currently a single-purpose pin file: `[pinned] files = [...]` keeps `/govern` from overwriting customized files. Other interactive choices `/govern` makes — most visibly, the per-category workflow recommendation prompts in [005-workflows](../005-workflows/spec.md) — are forgotten the moment the run ends. A user who declines `Linting` workflows is asked again on every subsequent `/govern`, with no way to say "stop offering this."

This feature extends `.govern.toml` from a pin-only file into the project's persisted-decisions store, with declining workflow recommendations as the motivating use case.

## Motivation

Two things are true about `/govern`:

- It is designed to be re-run frequently — every time the framework moves forward, every adopter is encouraged to re-run to pull the changes.
- It surfaces interactive prompts for opt-in scaffolding (workflow recommendations, agent selection) that the adopter has already answered in a previous run.

Without persistence, the second contradicts the first: the more often a user re-runs `/govern`, the more often they are asked the same question. The pragmatic outcome is that adopters either (a) silently accept defaults to make the noise stop, or (b) avoid running `/govern` because the prompts are tedious. Both are bad — the first defeats the explicit-consent design of workflow scaffolding, the second strands adopters on stale framework versions.

Persisted declines fix this: the user answers once, the answer is recorded, and subsequent runs respect it without re-prompting.

## Behavior

Before running the per-category workflow recommendation flow defined in [005-workflows](../005-workflows/spec.md), `/govern` reads `.govern.toml` (if it exists) and collects any categories listed under `[workflows] declined_categories`. For each candidate category in this run's recommendation list, if the category matches a recorded decline (case-insensitive) the prompt is suppressed entirely — no `AskUserQuestion` fires, the matching workflows are not scaffolded, and the post-scaffolding summary emits a `suppressed (workflow): {Category} (declined in .govern.toml)` line for that category.

For categories without a recorded decline, `/govern` presents the per-category prompt with three options instead of two:

1. `Yes, scaffold all in this category` — same as today's accept path; writes nothing to `.govern.toml`.
2. `Skip this run` — same as today's decline path; writes nothing to `.govern.toml`. The user is asked again on the next run.
3. `Skip and don't ask again` — declines this run **and** records the category to `.govern.toml`'s `[workflows] declined_categories` list. If `.govern.toml` does not exist, `/govern` creates it and reports `created .govern.toml to record decline` in the post-scaffolding summary. If the file exists without a `[workflows]` section, the section is added; if the section exists without `declined_categories`, the key is added; if the key exists, the new category is appended (deduplicated).

The user undoes a recorded decline by editing `.govern.toml` and removing the entry — a deliberate, file-edit action with no hidden flag. Recorded declines are permanent: there is no TTL, framework-version trigger, or auto-cleanup. Stale entries (e.g., a category name the framework no longer ships, or a typo) do not error and do not abort — `/govern` reports each unrecognized value once in the summary as `unrecognized workflow decline: "{value}" (in .govern.toml)` and proceeds.

Acceptance is never recorded. An accepted prompt scaffolds files; the existing **Filter out already-scaffolded workflows** step deduplicates on subsequent runs. Only declines need persistence, and only the third prompt option triggers it.

## Schema

`.govern.toml` gains a new top-level `[workflows]` section as a sibling of the existing `[pinned]` section. Domains stay flat at the top level — there is no umbrella `[settings]` or `[decisions]` namespace, and `[pinned]` is unchanged.

```toml
[pinned]
files = [".claude/commands/myapp/implement.md"]

[workflows]
# Categories the user has declined; /govern will not re-prompt for these.
# Match is case-insensitive against the registry-derived category list:
# Linting, Formatting, Testing, Migrations, Code Review, Deployment.
declined_categories = ["Linting", "Formatting"]
```

Future decision domains (agent selection, optional cleanup steps, etc.) are out of scope for this spec; when added, each gets its own top-level section keyed to the thing it governs (e.g., `[agents]`), with internal keys chosen to fit that domain's vocabulary rather than forced into a generic shape.

## Acceptance Criteria

- [x] `framework/bootstrap/govern.md` documents `[workflows] declined_categories` as a sibling top-level section to `[pinned]`, with the schema example and the case-insensitive matching rule.
- [x] Before the workflow recommendation flow's per-category prompts fire, `/govern` reads `.govern.toml` (if present) and collects entries in `[workflows] declined_categories`. For each candidate category in this run, if the category matches an entry case-insensitively, no `AskUserQuestion` fires and no workflows in that category are scaffolded.
- [x] The per-category workflow prompt has three options: `Yes, scaffold all in this category`, `Skip this run`, `Skip and don't ask again`. Picking the third option appends the category to `[workflows] declined_categories` in `.govern.toml`. Picking the first or second writes nothing to `.govern.toml`.
- [x] When the third option fires and `.govern.toml` does not exist, `/govern` creates it. When `.govern.toml` exists without a `[workflows]` section or `declined_categories` key, `/govern` adds them. Repeated declines of the same category do not produce duplicate entries.
- [x] The post-scaffolding summary emits `suppressed (workflow): {Category} (declined in .govern.toml)` once per category that was suppressed by a recorded decline this run (i.e., a candidate category that matched an entry). The summary also emits `created .govern.toml to record decline` when the file was created during this run.
- [x] Entries in `[workflows] declined_categories` that do not match any registry-derived category name (case-insensitive) are reported once each in the summary as `unrecognized workflow decline: "{value}" (in .govern.toml)`. Unrecognized entries do not abort the run and do not affect prompts.
- [x] Removing a category from `[workflows] declined_categories` (or deleting the section entirely) causes the next `/govern` run to prompt for that category again. There is no separate "unforget" command.
- [x] An adopter project with no `.govern.toml`, or one that has only `[pinned]` and no `[workflows]` section, runs through the workflow recommendation flow exactly as it does today — every category gets prompted, with the new three-option set instead of the prior two-option set.
- [x] If `.govern.toml` is malformed (TOML parse error), `/govern` aborts with a clear error message — same fail-loud posture as today.
- [x] The README's "Pinning files with .govern.toml" section is renamed (e.g., to "Configuring `.govern.toml`") and expanded to document `[workflows] declined_categories` alongside the existing `[pinned]` example, including how to remove an entry to re-enable prompting.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Validation rules.** Soft validation. At decline-check time, `/govern` matches recorded category names case-insensitively against the registry-derived category list (`Linting`, `Formatting`, `Testing`, `Migrations`, `Code Review`, `Deployment`). Matching entries suppress prompts as designed; non-matching entries (typos, removed categories, free-form notes) do not error and do not abort the run — they are reported once each in the post-scaffolding summary as `unrecognized workflow decline: "{value}" (in .govern.toml)`. Rationale: the file is user-editable; aborting on a typo would punish drift the adopter can fix in seconds, while silent-ignore would hide the drift entirely. The summary is the visible-but-non-blocking middle ground, and aligns with the framework's existing posture for `.govern.toml` (no commit hook, no validate rule, summary-only enforcement). Case-insensitive matching absorbs capitalization variance for free. (TOML parse errors remain a hard abort — that path is unchanged.)
- **Interaction with `/govern` on a fresh `.govern.toml`.** Yes — `/govern` creates `.govern.toml` if it doesn't exist when the user picks `Skip and don't ask again`, and lazily creates the `[workflows]` section if `.govern.toml` exists without it. The user explicitly chose the persisted-decline option; refusing to persist would silently downgrade that choice to `Skip this run`, which is the silent-degradation pattern the framework's design principles forbid — the option label promises "won't ask again," and the implementation has to honor it on the first decline as much as the hundredth. The file's appearance is reported in the post-scaffolding summary (e.g., `created .govern.toml to record decline`). Adopters who don't want the file in version control can `.gitignore` it; both committed (durable across clones) and ignored (per-clone) are coherent outcomes.
- **Decline summary verbosity.** Always list, as one line per suppressed category. Format: `suppressed (workflow): {Category} (declined in .govern.toml)`. Suppressed lines only fire when a recorded decline actually suppresses a prompt — a run with no declines emits no extra output. Rationale: the whole point of persisted declines is to remove cognitive load; a hidden-by-default flag would reintroduce it ("what *did* I decline?"). The `(declined in .govern.toml)` suffix names the source file so the user knows where to edit to undo. Distinguishes from the same-run `Skip this run` path: only persistence-suppressed categories surface here, not categories the user was prompted-and-declined live.
- **Schema shape.** Top-level section per domain, with domain-specific keys. The motivating example: `[workflows] declined_categories = ["Linting", "Formatting"]`. Mirrors the existing `[pinned] files = [...]` structure — top-level section keyed by the thing it governs, contents shaped to that thing's vocabulary. Each domain chooses its own keys (a future `[agents]` section might use `excluded` or `defaults` depending on what its decision actually is) rather than forcing a generic `declined = [...]` shape on every domain. Rejected: a `[decisions.*]` umbrella misclassifies `[pinned]` (which is configuration, not a decision), and a `[settings.*]` umbrella commits to expanding into non-decision config without a current need. Additive — future domains nest at the top level without disturbing existing sections.
- **Existing `[pinned]` relationship.** `[pinned]` stays as a top-level section, unchanged. Resolved by the schema-shape answer above: there is no umbrella to migrate under. `[pinned]` and `[workflows]` are siblings, not nested. This avoids any breaking change to existing `.govern.toml` files.
- **Scope on first delivery.** Workflows-only. The motivating pain is the per-category workflow prompts; other `/govern` prompts (agent selection, project-name confirmation, cleanup confirmations) are one-shot or structural and don't share the same re-prompt-fatigue character. Constraining first delivery to one domain reduces the risk of generalizing the schema from a single example. The section-per-domain schema (see next question) is additive — future domains like `[agents]` can be added later in their own follow-on spec without breaking entries written today. This spec deliberately does not try to anticipate future domains.
- **TTL or permanence.** Permanent until manually edited. Matches the existing `[pinned]` model (no expiration in the same file). Time-based expiration ("re-ask after N runs/days") would re-introduce the prompt fatigue the feature exists to eliminate. Version-based expiration is impractical because `/govern` runs continuously on `main` — there is no clean version boundary that wouldn't degenerate into "re-prompt on every framework commit." The genuine reasons to reconsider a decline are user-driven (team adopts a tool, language changes), not calendar-driven; those cases are exactly when an adopter would naturally edit `.govern.toml`. Tradeoff accepted: stale entries can rot in `.govern.toml` indefinitely; the post-scaffolding summary surfaces what is being honored so the user can spot stale entries.
- **Auto-write vs manual record.** Extend the per-category prompt with a third option rather than chaining a follow-up question. The prompt becomes: `Yes, scaffold all in this category` / `Skip this run` / `Skip and don't ask again`. Picking the third option is what writes the decline to `.govern.toml`; the second option preserves today's no-write decline behavior. Rationale: avoids the "two prompts in a row" tax of an opt-in follow-up while making persistence visible in the option label (no auto-write surprise). The third option is the new affordance this feature delivers; the first two map directly onto today's accept/decline.
- **Granularity of workflow declines.** Per-category only. The prompt the user actually answers is per-category ("Scaffold these Linting workflows for Claude Code?" — Yes/No across the whole category list); recording at that same granularity mirrors the answered question. Per-template adds a fine-tuning surface that has no corresponding prompt today, and the rare "accept the category but reject one tool" case is already handled by the **Filter out already-scaffolded workflows** step (scaffold once, delete the unwanted file). The schema remains additive — a future per-template need can extend without breaking existing entries. Tradeoff accepted: an adopter who declines a category wholesale is not re-prompted when a new tool joins that category in their stack; they remove the category from `.govern.toml` to re-enable prompting, consistent with the "deliberate file-edit to undo" model.
