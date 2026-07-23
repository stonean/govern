---
description: Set the working feature (and optionally scenario) for this session.
argument-hint: "[feature[/scenario] | --clear]"
parity:
  strict-files:
    - ".govern.session.toml"
---

# Target

Set the working feature (and optionally scenario) for this session.

## Purpose

Establishes which feature spec all subsequent `/gov:*` commands operate on. Optionally targets a specific scenario within the feature for scenario-aware commands. Must be run before any pipeline command. Remains active for the session unless changed by running `/gov:target` again.

## Scope Boundaries

- Read `.govern/constitution.md` once per session and the targeted feature's `spec.md` frontmatter and open-question count. Read the targeted scenario file only when one is specified.
- Do NOT read plan files, tasks, source code, test files, or unrelated specs' bodies.
- Do NOT modify any spec, plan, scenario, or source file. The only file written is the session file (`.govern/session.toml`). Status transitions belong to the pipeline commands (`/gov:clarify`, `/gov:plan`, `/gov:implement`) and to `/gov:amend` (the documented back-edges: `clarified|planned|in-progress → draft` on a new question, and `done → in-progress` on a new scenario).
- Reference: §spec-lifecycle, §scenarios, §concurrent-features, §text-first-artifacts.

## Instructions

> **For agent runtimes**: the Invoke steps below call the MCP tools of the optional gvrn runtime; the host-integration contract — bare↔prefixed tool names, lazy ToolSearch schema fetch, the no-shell-utilities rule, and the two-paths guarantee — lives once in the constitution, §runtime-host-integration. With no gvrn MCP server registered, walk the same prose using the host file-reading tools (Read, Edit, Write).

<!-- audit:ignore-promotion -->
1. When the invocation has no argument (whitespace or empty), read the session file — `.govern/session.toml` when it exists, else the legacy root `.govern.session.toml` (the parity strict-files frontmatter above names the legacy path, matching the legacy-layout parity fixtures) — to display the current target. If the file is empty or absent, report no target set; otherwise display the feature name and status, the scenario detail when one is targeted (scenario name, the section field or legacy spec-ref field, and the context summary), and the artifacts list. Then stop — the steps below only apply when a feature argument is supplied. Treat `0`, `00`, or any other non-whitespace string as a valid feature identifier.

<!-- audit:ignore-promotion -->
2. When the invocation argument is exactly `--clear`, clear the session target through the write-session primitive's clear mode: it removes the target block (feature / path / scenario / scenario-path / set-at) while preserving any cli-config-dir (the per-contributor agent identity written by `/govern`) so `gvrn exec` keeps resolving command files. On the markdown-only path, reach the same reset state by hand: if the session file records a cli-config-dir, rewrite it to contain only that key via the tempfile + rename pattern; otherwise delete the file. Either way no `feature` remains, so the dashboard's documented "session file → session-target: null" reset state holds. Emit `Session cleared. Run /gov:target to set a new target.` and stop — the steps below only apply when a feature argument is supplied. `--clear` combined with a feature argument or a scenario suffix halts with `/gov:target: --clear cannot be combined with a feature argument` (no session mutation). When the session file is already absent, this is a no-op that still emits the confirmation line.

3. Parse the argument: when the value contains a slash, split into a feature-part and a scenario-slug; otherwise treat the value as a feature-part with no scenario. Invoke `resolve-feature` with the feature-part as the identifier — it scans the configured specs root and matches by exact directory name, feature number (zero-padded or not), or unique case-insensitive partial slug, returning the directory name, path, and status. Ambiguity and no-match are domain outcomes the host mediates: on `ambiguous`, list the returned candidates and ask the user to choose; on `not-found`, report the feature does not exist and list available features (the `not-found` result carries no candidate list — enumerate them from the dashboard payload's `specs[].slug`, or a specs-directory listing on the markdown-only path). Otherwise, fall back to the markdown-only path: search the specs directory for a matching name by hand.

<!-- audit:ignore-promotion -->
4. Load the constitution file once per session to make its sections available for subsequent commands. (Host responsibility — no primitive reads the constitution; otherwise, fall back to the markdown-only path.)

5. Invoke `run-generator` against the dependency generator as a safety net (dry-run only). When it reports drift, the `dependencies:` frontmatter is stale from uncommitted body edits — surface that and recommend committing (the pre-commit hook syncs it) or running `.govern/scripts/gen-spec-deps.sh` manually. Do **not** run the generator for real from `/gov:target`: this command writes only the session file (see Scope Boundaries), while the generator rewrites `dependencies:` across every spec. On the markdown-only path, run `.govern/scripts/gen-spec-deps.sh --dry-run` by hand and surface a diff the same way.

6. Invoke `read-spec` against the resolved feature to load frontmatter, sections, and the open-question count from the body. The frontmatter status is normally one of draft, clarified, planned, in-progress, or done — `read-spec` returns it verbatim, and `/gov:analyze` (through its frontmatter-validation step) owns flagging an out-of-set value.

7. When a scenario was provided, invoke `resolve-feature` again with the scenario slug as the scenario argument: the result's scenario block reports the scenario file's path, whether it exists, and its section frontmatter field (falling back to the legacy spec-ref field for pre-017 scenarios). Capture the context summary from the scenario body with host file tools — the summary is not a primitive result. If the scenario does not exist, list available scenarios and ask the user to choose (host-mediated domain outcome). Otherwise, fall back to the markdown-only path: locate the scenario file under the feature's scenarios subdirectory and read its frontmatter and body by hand.

8. Invoke `write-session` with the feature slug as the feature argument, the repo-relative spec directory — under the configured `[paths] specs-root` (default `specs`; spec 040) — as the path argument, and the scenario slug plus its file path as the scenario and scenario-path arguments when one is targeted (omit both to clear any previously set scenario). This is a *target write*: the primitive sets feature/path/(scenario) and stamps a fresh set-at while **preserving** any cli-config-dir already in the file (the per-contributor agent identity written by `/govern`), at `.govern/session.toml` (repo root; gitignored; same path for every adopter regardless of AI CLI or project name), and applies tempfile + rename atomic-write semantics. On the markdown-only path (no runtime on `PATH`), the host first reads any existing `.govern/session.toml` to capture its cli-config-dir, then writes the TOML directly — top-level keys feature, path, optional scenario, optional scenario-path, set-at (ISO 8601 UTC), then the preserved cli-config-dir (when present) — through the same tempfile + rename pattern.

<!-- audit:ignore-promotion -->
9. Display the resolved target: feature name and current status, scenario detail when present, the artifacts list (which of spec.md, plan.md, tasks.md, and data-model.md exist), the dependency status from step 5, the open-question count, and the next pipeline step per the Status → next action table below.

## Status → next action

| Status | Open Questions | Next pipeline step |
| --- | --- | --- |
| draft | any | /gov:clarify |
| clarified | 0 | /gov:plan |
| planned | 0 | /gov:implement |
| in-progress | 0 | /gov:implement |
| done | any | confirm complete; run /gov:amend to record a scenario and reopen |

When the status is clarified, planned, or in-progress AND the open-question count is at least one, the next step is `/gov:clarify` (recovery). This state usually arises from a manual frontmatter edit; the normal back-edge via `/gov:amend` keeps status and open-question presence in sync.
