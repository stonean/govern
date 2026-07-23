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
- The pipeline view is acquired through the single `dashboard` primitive — one runtime call returns the structured payload plus the pre-rendered pipeline view. On the runtime path the procedure does not read individual spec files, list directories, or shell out for that view; the markdown-only fallback (below) necessarily reads the artifacts on disk, but still without shell-pipeline substitution.
- Cross-service reference resolution is read-only and, on the runtime path, folded into that same call: the runtime resolves each spec's `references:` index internally (the same classification the resolve-references primitive exposes) to render the readout. On the markdown-only path the host reads `.govern/config.toml` and the linked specs with host file tools (see the Resolving cross-service references section below). Both paths read only `.govern/config.toml` and the registered local checkouts — the canonical repo URL is never fetched.
- Reference: §text-first-artifacts (the schema is the authoritative source for which fields appear in the payload); [030 — Cross-Service References](../../specs/030-cross-service-references/spec.md) for the reference semantics surfaced here.

## Instructions

> **For agent runtimes**: the Invoke steps below call the MCP tools of the optional gvrn runtime; the host-integration contract — bare↔prefixed tool names, lazy ToolSearch schema fetch, the no-shell-utilities rule, and the two-paths guarantee — lives once in the constitution, §runtime-host-integration. With no gvrn MCP server registered, walk the same prose using the host file-reading tools (Read, Edit, Write).
>
> **For this command specifically**: the single deterministic tool for the pipeline view is `dashboard`. One MCP call returns the session target, the per-spec inventory, the repo-wide tags-union, the `.govern/config.toml` review-state summary, and the pre-rendered pipeline view. Do not substitute shell utilities (no `ls`, no `for` loops over spec directories, no `cat .govern/config.toml`) for that call. Cross-service reference resolution for specs that carry a `references:` index is folded into the same call on the runtime path (the runtime resolves each index internally for the readout); on the markdown-only path, use the host file tools documented in the Resolving cross-service references section. Neither substitutes a shell pipeline.

<!-- audit:ignore-promotion -->
1. Invoke `dashboard` to load the full pipeline state in one call. Alongside the structured payload (session-target, specs, tags-union, config), the result carries a `rendered-markdown` field: the preamble, dashboard table, counts and callouts, and cross-service references readout pre-rendered as one markdown fragment (the runtime resolves each spec's `references:` index internally for the readout, so no separate per-spec call is needed on this path).

<!-- audit:ignore-promotion -->
1. Emit the `rendered-markdown` fragment as the pipeline view. It is returned data the host may restyle — do not recompute the underlying facts from the structured payload when the fragment is present. On the markdown-only path, derive the same facts with host file tools and render the four pieces by hand per the Rendering reference below.

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

### Rendering reference

The `rendered-markdown` field carries these four pieces pre-rendered, in this order, blocks separated by blank lines. Without the runtime, derive the same facts with the host file tools (read each spec's frontmatter and body, `.govern/session.toml`, and `.govern/config.toml` directly — no shell-pipeline substitution; the field names below refer to the `dashboard` payload schema, whose values the markdown-only walk derives from the artifacts on disk) and render each piece as follows. Both paths produce the same view.

1. Render the **preamble line** above the table from the session-target field. When session-target is present: `Target: {feature} / {status} / next: {next-action}`. The `{status}` and `{next-action}` come from the matching per-spec entry in the `specs` array; the next action follows the Status → next action table above, with `clarify (recovery)` overriding when the status is in {clarified, planned, in-progress} and the open-question count is at least one. When a scenario is also targeted, append a second preamble line: `Scenario: {scenario} ({section}) — open-questions: {open-question-count}`, with the `{section}` and `{open-question-count}` taken from `session-target.scenario-detail`; when the scenario has at least one unresolved question, the next action is `/gov:clarify` (scenario-targeted) regardless of the parent spec's next action. When session-target is null, render `No session target. Run /gov:target to select one.`

2. Render the **dashboard table** with one row per entry in the `specs` array. Columns: Feature, Status, Plan, Tasks, Data-model, Scenarios, Dependencies, Next Action. Mark the row matching the session target by wrapping its Feature cell in bold (`| **{slug}** | … |`); when session-target is null, no row is bolded. The Plan/Tasks/Data-model cells show `✓` when the artifact exists and `—` otherwise. The Scenarios column shows scenarios-count. The Dependencies column shows the comma-separated three-digit NNN prefixes from `specs[].dependencies`, sorted ascending — `—` when the array is empty. The Next Action column comes from the Status → next action table above; when the status is in {clarified, planned, in-progress} and open-question-count is at least one, the Next Action is `clarify (recovery)` regardless of status — that recovery state usually arises from a manual frontmatter edit.

3. Below the table, render **counts and callouts**. Show counts per status level on one line, in lifecycle order (group the `specs` array by `status`): `Counts: {status} {N} · …`. Show the blocked specs callout when any entry has a non-empty blocked-by — `Blocked: {N} spec(s) — {comma-separated slugs}` (the slugs come from each blocked spec's `slug`, not its blocked-by). Show the recovery-state callout when any entry's status is in {clarified, planned, in-progress} with open-question-count ≥ 1: `{N} spec(s) in recovery state: {comma-separated slugs}. Run /gov:clarify on each to walk the questions; the spec reverts to draft and advances forward again.` Show the tags-in-use line when tags-union is non-empty: `tags: {comma-separated}`; skip the line entirely otherwise. Show the disabled-rule-files line when `config.present` is true and `config.disabled-rule-files` is non-empty: `disabled rule files: {N} ({config-file}) — {comma-separated basenames}`, where `{config-file}` is the repo-relative resolved config file (`.govern/config.toml`, or the legacy root `.govern.toml` on a pre-migration layout); skip the line entirely when `config.present` is false or the array is empty. Reasons are not surfaced — they live in `.govern/config.toml`; the dashboard is a glance, not a full pretty-printer.

4. Render the **cross-service references readout** for every spec whose derived `references` index is non-empty. Resolve each reference with the procedure in the Resolving cross-service references section below (folded into the `dashboard` call on the runtime path; host file tools on the markdown-only path), then list each reference beneath its spec as `{service}/{spec}` with its resolution outcome: on **ok**, the linked lifecycle status (e.g., `api/003-user → done`); on **unregistered**, `status not attempted` plus a pointer to `/gov:link` to register the service; on **not checked out** or **status unreadable**, `unknown` with the distinguishing reason; on **broken**, the broken-reference notice that `/gov:analyze` also reports. Append the matched service's `description` for orientation when one is present. Omit the readout entirely when no spec declares references — a single-service adopter sees no change.

### Resolving cross-service references

A spec's derived `references:` frontmatter index records each cross-service reference as a `{service, spec}` pair (see [030 — Cross-Service References](../../specs/030-cross-service-references/spec.md)). On the runtime path the `resolve-references` primitive classifies each entry; when the runtime is unavailable, classify each entry with the host's file tools — read `.govern/config.toml` and the linked spec directly, with **no shell-pipeline substitution**. The repo URL is identity and navigation only and is **never fetched**; status is read from the local checkout.

For each `{service, spec}` entry, in index order, decide the outcome by what can be proven:

1. **`unregistered`** — the entry's `service` is null, or names an alias absent from `.govern/config.toml` `[services]`. Status not attempted; a plain navigational link, not an error. (Surface it with a pointer to `/gov:link` to register the service.)
2. Otherwise read the matched `[services.<alias>]` entry's local `path` (relative to the repo root, or absolute):
   - **`not-checked-out`** — `path` is missing or is not a directory. Status `unknown`; informational, never reported as broken — nothing can be proven without a checkout.
   - Otherwise resolve the target spec at `{path}/{linked-specs-root}/{spec}/spec.md`, where `{linked-specs-root}` is the **linked service's own** `[paths] specs-root` (default `specs`) read from the checkout's `.govern/config.toml` — each service may configure its own spec root, and `resolve-references` honors the linked repo's setting, so the markdown-only path must read it too rather than assuming `specs/`:
     - **`broken`** — the target file does not exist (renamed, moved, deleted, or mistyped upstream). A provable defect, surfaced by `/gov:analyze`.
     - Otherwise read the target's frontmatter `status`:
       - **`status-unreadable`** — the file has no frontmatter, its frontmatter is not valid YAML, or `status` is missing or not one of `draft` / `clarified` / `planned` / `in-progress` / `done`. Status `unknown`; surfaced, the defect is upstream's.
       - **`ok`** — `status` is present and in that allowed set. The record carries that lifecycle status.

This host-tools procedure produces the same resolution records — `{service, spec, outcome, status}` — as the `resolve-references` primitive for the same inputs; the two paths share a contract and neither wraps the other.

**Surfacing in `/gov:status`.** The readout render step lists one line per reference beneath its spec: the resolution `outcome` and, on `ok`, the linked lifecycle `status`. The matched `[services.<alias>]` entry's `description` — read from the same `.govern/config.toml` the resolution consults — is appended for orientation when present (informational only; no rendering branches on it, so a blank description degrades to nothing). An `unregistered` reference is surfaced with a pointer to `/gov:link` so the user can register the service; `not-checked-out` and `status-unreadable` surface as `unknown` with their distinguishing reason; a `broken` reference is surfaced here and is the same provable defect `/gov:analyze` reports as an Advisory finding.
