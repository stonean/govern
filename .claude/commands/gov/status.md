---
description: Display the pipeline view for all feature specs.
parity:
  strict-stdout: true
---

# Status

Display the pipeline view for all feature specs.

## Purpose

Read-only overview of every feature's progress through the pipeline. Shows which specs are ready to advance, which are blocked, and what the current session target is.

## Scope Boundaries

- This is a read-only command. Do NOT modify any files.
- All data acquisition happens through the `dashboard` primitive — the procedure makes one runtime call and renders from its payload. The procedure itself does not read individual spec files, list directories, or shell out.
- Reference: §text-first-artifacts (the schema is the authoritative source for which fields appear in the payload).

## Instructions

> **For agent runtimes**: backticked primitive names in this section map to MCP tools the optional [gvrn runtime](https://crates.io/crates/gvrn) exposes under bare `<primitive>` names (e.g., `dashboard`). Hosts wrap them with a server-name prefix taken from `.mcp.json` (Claude: `mcp__gvrn__dashboard`; Auggie: `mcp:gvrn:dashboard`). When the server is registered for your session, **call the corresponding tool** for each step listed below — that is the deterministic path. If your host loads MCP tool schemas lazily (e.g., Claude Code lists tool names in a deferred-tool system reminder before exposing their schemas), the runtime is still registered: fetch the schema via the host's mechanism (`ToolSearch` on Claude Code) and call the tool — do not bail to the markdown-only fallback. When no `gvrn` MCP server is configured, walk the prose using the host's file-reading tool (e.g., `Read`) to produce the same result; do **not** substitute shell utilities (`awk`, `sed`, `grep` pipelines, `for` loops over files) for the prescribed file reads. The two paths share a contract; neither one wraps the other.
>
> **For this command specifically**: the single deterministic tool is `dashboard`. One MCP call returns the session target, the per-spec inventory, the repo-wide tags-union, and the `.govern.toml` review-state summary. Do not substitute shell utilities (no `ls`, no `for` loops over spec directories, no `cat .govern.toml`) for that call.

<!-- audit:ignore-promotion -->
1. Invoke `dashboard` (MCP: `dashboard`) to load the full pipeline state in one call. Otherwise, follow the markdown-only path: read `.govern.session.toml` for the session target, walk `specs/` for the `NNN-feature` directories, parse each spec's frontmatter and open-question count, check artifact existence (`plan.md` / `tasks.md` / `data-model.md`), count `*.md` files under `scenarios/`, compute each spec's blocked-by from the per-spec `dependencies` and statuses, fold the tags-union across every spec, and read `.govern.toml` for the `[[review.disabled-rule-files]]` summary — but only when the runtime MCP server is genuinely unavailable.

<!-- audit:ignore-promotion -->
1. Render the **preamble line** above the table from the session-target field. When session-target is present: `Target: {feature} / {status} / next: {next-action}`. The `{status}` and `{next-action}` come from the matching per-spec entry in the `specs` array; the next action follows the Status → next action table below, with `clarify (recovery)` overriding when the status is in {clarified, planned, in-progress} and the open-question count is at least one. When a scenario is also targeted, append a second preamble line: `Scenario: {scenario} ({section}) — open-questions: {open-question-count}`, with the `{section}` and `{open-question-count}` taken from `session-target.scenario-detail`; when the scenario has at least one unresolved question, the scenario's next action is `/gov:clarify` (scenario-targeted) regardless of the parent spec's next action. When session-target is null, render `No session target. Run /gov:target to select one.`

<!-- audit:ignore-promotion -->
1. Render the **dashboard table** with one row per entry in the `specs` array. Columns: Feature, Status, Plan, Tasks, Data-model, Scenarios, Dependencies, Next Action. Mark the row matching the session target by wrapping its Feature cell in bold (`| **{slug}** | … |`); when session-target is null, no row is bolded. The Scenarios column shows scenarios-count. The Dependencies column shows the comma-separated three-digit NNN prefixes from `specs[].dependencies`, sorted ascending — `—` when the array is empty. The Next Action column comes from the Status → next action table below; when the status is in {clarified, planned, in-progress} and open-question-count is at least one, the Next Action is `clarify (recovery)` regardless of status — that recovery state usually arises from a manual frontmatter edit.

<!-- audit:ignore-promotion -->
1. Below the table, render **counts and callouts** from the payload. Show counts per status level (group the `specs` array by `status`). Show the blocked specs callout when any entry has a non-empty blocked-by — `Blocked: {N} spec(s) — {comma-separated slugs}` (the slugs come from each blocked spec's `slug`, not its blocked-by). Show the recovery-state callout when any entry's status is in {clarified, planned, in-progress} with open-question-count ≥ 1: `{N} spec(s) in recovery state: {comma-separated slugs}. Run /gov:clarify on each to walk the questions; the spec reverts to draft and advances forward again.` Show the tags-in-use line when tags-union is non-empty: `tags: {comma-separated}`; skip the line entirely otherwise. Show the disabled-rule-files line when `config.present` is true and `config.disabled-rule-files` is non-empty: `disabled rule files: {N} (.govern.toml) — {comma-separated basenames}`; skip the line entirely when `config.present` is false or the array is empty. Reasons are not surfaced — they live in `.govern.toml`; the dashboard is a glance, not a full pretty-printer.

<!-- audit:ignore-promotion -->
1. List any non-done specs (excluding the current target, if any) and prompt the user to run `/gov:target` to select one.

## Status → next action

| Status | Next Action |
| --- | --- |
| draft | /gov:clarify |
| clarified | /gov:plan |
| planned | /gov:implement |
| in-progress | /gov:implement |
| done | done (spec is complete) |

When a scenario is targeted and the scenario itself has one or more open questions, the next action is `/gov:clarify` (scenario-targeted, resolves scenario-level open questions regardless of parent spec status).

## Markdown-only reference

### Resolving cross-service references

A spec's derived `references:` frontmatter index records each cross-service reference as a `{service, spec}` pair (see [030 — Cross-Service References](../../specs/030-cross-service-references/spec.md)). On the runtime path the `resolve-references` primitive classifies each entry; when the runtime is unavailable, classify each entry with the host's file tools — read `.govern.toml` and the linked spec directly, with **no shell-pipeline substitution**. The repo URL is identity and navigation only and is **never fetched**; status is read from the local checkout.

For each `{service, spec}` entry, in index order, decide the outcome by what can be proven:

1. **`unregistered`** — the entry's `service` is null, or names an alias absent from `.govern.toml` `[services]`. Status not attempted; a plain navigational link, not an error. (Surface it with a pointer to `/gov:link` to register the service.)
2. Otherwise read the matched `[services.<alias>]` entry's local `path` (relative to the repo root, or absolute):
   - **`not-checked-out`** — `path` is missing or is not a directory. Status `unknown`; informational, never reported as broken — nothing can be proven without a checkout.
   - Otherwise resolve the target spec at `{path}/specs/{spec}/spec.md`:
     - **`broken`** — the target file does not exist (renamed, moved, deleted, or mistyped upstream). A provable defect, surfaced by `/gov:analyze`.
     - Otherwise read the target's frontmatter `status`:
       - **`status-unreadable`** — the file has no frontmatter, its frontmatter is not valid YAML, or `status` is missing or not one of `draft` / `clarified` / `planned` / `in-progress` / `done`. Status `unknown`; surfaced, the defect is upstream's.
       - **`ok`** — `status` is present and in that allowed set. The record carries that lifecycle status.

This host-tools procedure produces the same resolution records — `{service, spec, outcome, status}` — as the `resolve-references` primitive for the same inputs; the two paths share a contract and neither wraps the other.
