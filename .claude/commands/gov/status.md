---
description: Display the pipeline dashboard for all feature specs.
parity:
  strict-stdout: true
---

# Status

Display the pipeline dashboard for all feature specs.

## Purpose

Read-only overview of every feature's progress through the pipeline. Shows which specs are ready to advance, which are blocked, and what the current session target is.

## Scope Boundaries

- This is a read-only command. Do NOT modify any files.
- For each feature, read only the spec file (`spec.md`) to extract `status`, `dependencies`, `tags`, and open question count. Do NOT read plans, tasks, scenarios, source code, or other artifact contents.
- Check file existence (`plan.md`, `tasks.md`, `data-model.md`, `scenarios/`) without reading them.
- Reference: §text-first-artifacts (the schema is the authoritative source for which fields to read).

## Instructions

> **For agent runtimes**: backticked primitive names in this section map to MCP tools the optional [gvrn runtime](https://crates.io/crates/gvrn) exposes under bare `<primitive>` names (e.g., `read-spec`). Hosts wrap them with a server-name prefix taken from `.mcp.json` (Claude: `mcp__gvrn__read-spec`; Auggie: `mcp:gvrn:read-spec`). When the server is registered for your session, **call the corresponding tool** for each step listed below — that is the deterministic path. When the server is not registered, walk the prose to produce the same result. The two paths share a contract; neither one wraps the other.

Steps 1–2 must complete before any other work. Do NOT read spec directories, list files, or perform any dashboard work until step 2 resolves.

1. The walker context already contains the session target's feature field when `.claude/gov-session.json` exists — the runtime exec subcommand seeds it from that file automatically. In the markdown-only path, read the session file directly to learn the current target (and any scenario it carries).

2. When the session target names a feature, invoke `read-spec` (MCP: `read-spec`) against the targeted feature to load frontmatter, sections, and the open-question count from the body. Route on the loaded status (one of draft, clarified, planned, in-progress, done):
   1. When the status is anything other than done, display the target feature name and status. If a scenario is targeted, also read the scenario file and display the scenario name, the section field (or the legacy spec-ref field for pre-017 scenarios), the context summary, and its open-question count. Then prompt the next pipeline command per the Status → next action table below, and stop. Do not build the full dashboard.
   2. When the status is done, continue to step 3 to build the full dashboard.

3. (Full dashboard path) List directories under `specs/` matching the NNN-feature pattern. For each feature directory, read only its spec file's YAML frontmatter and extract status, dependencies, tags (optional, treat absent as empty), and the open-question count from the body. Otherwise, follow the markdown-only path: open each file with the host and parse the same fields.

4. (Full dashboard path) For each feature, check whether plan.md, tasks.md, and data-model.md exist (without reading them) and count the markdown files in its scenarios subdirectory when one is present.

5. Display the dashboard table with one row per feature. Mark the session target with a leading `>>`. The Scenarios column shows the count of markdown files in the feature's scenarios subdirectory. The Next Action column comes from the Status → next action table below; when the status is in the set {clarified, planned, in-progress} and the open-question count is at least one, the Next Action is `clarify (recovery)` regardless of status — that recovery state usually arises from a manual frontmatter edit.

6. Below the table, show counts per status level, the blocked specs (any feature whose dependencies are not at clarified or later), and any specs in the recovery state. Surface recovery-state specs as a one-line callout: "{N} spec(s) in recovery state: {comma-separated slugs}. Run `/gov:clarify` on each to walk the questions; the spec reverts to draft and advances forward again." If at least one spec has non-empty tags, list the union of tags in use across the repo on one line, comma-separated; skip the line entirely if no spec has tags. If `.govern.toml` exists and contains a non-empty `[[review.disabled-rule-files]]` array (see [`framework/commands/review.md`](review.md) §Inputs), emit a one-line callout naming how many files are excluded and which: `disabled rule files: {N} (.govern.toml) — {comma-separated basenames}`. Skip the line entirely when `.govern.toml` is absent, lacks the section, or the array is empty. Reasons are not surfaced — they live in `.govern.toml`; the dashboard is a glance, not a full pretty-printer.

7. List any non-done specs (excluding the current target) and prompt the user to run `/gov:target` to select one.

## Status → next action

| Status | Next Action |
| --- | --- |
| draft | /gov:clarify |
| clarified | /gov:plan |
| planned | /gov:implement |
| in-progress | /gov:implement |
| done | done (spec is complete) |

When a scenario is targeted and the scenario itself has one or more open questions, the next action is `/gov:clarify` (scenario-targeted, resolves scenario-level open questions regardless of parent spec status).
