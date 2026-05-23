---
description: Set the working feature (and optionally scenario) for this session.
argument-hint: "[feature[/scenario] | --clear]"
parity:
  strict-files:
    - "{cli-config-dir}/{project}-session.json"
---

# Target

Set the working feature (and optionally scenario) for this session.

## Purpose

Establishes which feature spec all subsequent `/{project}:*` commands operate on. Optionally targets a specific scenario within the feature for scenario-aware commands. Must be run before any pipeline command. Remains active for the session unless changed by running `/{project}:target` again.

## Scope Boundaries

- Read `constitution.md` once per session and the targeted feature's `spec.md` frontmatter and open-question count. Read the targeted scenario file only when one is specified.
- Do NOT read plan files, tasks, source code, test files, or unrelated specs' bodies.
- Do NOT modify any spec, plan, scenario, or source file. The only file written is the session JSON. Status transitions belong to the pipeline commands (`/{project}:clarify`, `/{project}:plan`, `/{project}:implement`) and to `/{project}:ask` (the documented back-edges: `clarified|planned|in-progress → draft` on a new question, and `done → in-progress` on a new scenario).
- Reference: §spec-lifecycle, §scenarios, §concurrent-features, §text-first-artifacts.

## Instructions

> **For agent runtimes**: backticked primitive names in this section map to MCP tools the optional [gvrn runtime](https://crates.io/crates/gvrn) exposes under bare `<primitive>` names (e.g., `read-spec`). Hosts wrap them with a server-name prefix taken from `.mcp.json` (Claude: `mcp__gvrn__read-spec`; Auggie: `mcp:gvrn:read-spec`). When the server is registered for your session, **call the corresponding tool** for each step listed below — that is the deterministic path. If your host loads MCP tool schemas lazily (e.g., Claude Code lists tool names in a deferred-tool system reminder before exposing their schemas), the runtime is still registered: fetch the schema via the host's mechanism (`ToolSearch` on Claude Code) and call the tool — do not bail to the markdown-only fallback. When no `gvrn` MCP server is configured, walk the prose using the host's file-reading tool (e.g., `Read`) to produce the same result; do **not** substitute shell utilities (`awk`, `sed`, `grep` pipelines, `for` loops over files) for the prescribed file reads. The two paths share a contract; neither one wraps the other.

<!-- audit:ignore-promotion -->
1. When the invocation has no argument (whitespace or empty), read the session JSON at `{cli-config-dir}/{project}-session.json` (the parity strict-files frontmatter above names this exact path) to display the current target. If the file is empty or absent, report no target set; otherwise display the feature name and status, the scenario detail when one is targeted (scenario name, the section field or legacy spec-ref field, and the context summary), and the artifacts list. Then stop — the steps below only apply when a feature argument is supplied. Treat `0`, `00`, or any other non-whitespace string as a valid feature identifier.

<!-- audit:ignore-promotion -->
2. When the invocation argument is exactly `--clear`, remove the session JSON at `{cli-config-dir}/{project}-session.json` (delete the file; the `dashboard` primitive's documented "session file absent → session-target: null" behavior is the reset state). Emit `Session cleared. Run /{project}:target to set a new target.` and stop — the steps below only apply when a feature argument is supplied. `--clear` combined with a feature argument or a scenario suffix halts with `/{project}:target: --clear cannot be combined with a feature argument` (no session mutation). When the session file is already absent, the delete is a no-op that still emits the confirmation line.

<!-- audit:ignore-promotion -->
3. Parse the argument: when the value contains a slash, split into a feature-part and a scenario-slug; otherwise treat the value as a feature-part with no scenario. Resolve the feature-part by accepting a feature number, a partial name, or a full directory name; search the specs directory for a matching name. If ambiguous, list matches and ask the user to choose. If no match, report the feature does not exist and list available features. (Host responsibility — no runtime primitive iterates the specs directory; otherwise, fall back to the markdown-only path.)

<!-- audit:ignore-promotion -->
4. Load the constitution file once per session to make its sections available for subsequent commands. (Host responsibility — no primitive reads the constitution; otherwise, fall back to the markdown-only path.)

<!-- audit:ignore-promotion -->
5. Recompute dependencies as a safety net by running scripts/gen-spec-deps.sh as a dry run; if the dry run reports a diff, run it for real to sync the frontmatter dependencies from body inline links. The pre-commit hook normally keeps this in sync; this step catches uncommitted body edits. (Host responsibility today; the runtime exposes an equivalent procedural wrapper used by other commands. Otherwise, follow the markdown-only path.)

6. Invoke `read-spec` (MCP: `read-spec`) against the resolved feature to load frontmatter, sections, and the open-question count from the body. The frontmatter status is one of draft, clarified, planned, in-progress, or done.

<!-- audit:ignore-promotion -->
7. When a scenario was provided, locate the scenario file under the feature's scenarios subdirectory and read it: extract the section field from frontmatter (or the legacy spec-ref field for pre-017 scenarios) and capture the context summary from the body. If the scenario does not exist, list available scenarios and ask the user to choose. (Host responsibility — the runtime does not expose a scenario primitive; otherwise, fall back to the markdown-only path.)

8. Invoke `write-session` (MCP: `write-session`) with the feature slug as the feature argument, the repo-relative spec directory as the path argument, and the scenario slug plus its file path as the scenario and scenario-path arguments when one is targeted (omit both to clear any previously set scenario). The primitive writes the canonical session JSON at `{cli-config-dir}/{project}-session.json`, stamps the `setAt` field from the runtime's clock, and applies tempfile + rename atomic-write semantics. On the markdown-only path (no runtime on `PATH`), the host writes the same JSON shape directly — top-level fields feature, path, optional scenario, optional scenarioPath, setAt (ISO 8601 UTC) in that order — through the same tempfile + rename pattern.

<!-- audit:ignore-promotion -->
9. Display the resolved target: feature name and current status, scenario detail when present, the artifacts list (which of spec.md, plan.md, tasks.md, and data-model.md exist), the dependency status from step 5, the open-question count, and the next pipeline step per the Status → next action table below.

## Status → next action

| Status | Open Questions | Next pipeline step |
| --- | --- | --- |
| draft | any | /{project}:clarify |
| clarified | 0 | /{project}:plan |
| planned | 0 | /{project}:implement |
| in-progress | 0 | /{project}:implement |
| done | any | confirm complete; run /{project}:ask to record a scenario and reopen |

When the status is clarified, planned, or in-progress AND the open-question count is at least one, the next step is `/{project}:clarify` (recovery). This state usually arises from a manual frontmatter edit; the normal back-edge via `/{project}:ask` keeps status and open-question presence in sync.
