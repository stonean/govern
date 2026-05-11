---
description: Set the working feature (and optionally scenario) for this session.
argument-hint: "[feature[/scenario]]"
parity:
  strict-files:
    - .claude/gov-session.json
---

# Target

Set the working feature (and optionally scenario) for this session.

## Purpose

Establishes which feature spec all subsequent `/{project}:*` commands operate on. Optionally targets a specific scenario within the feature for scenario-aware commands. Must be run before any pipeline command. Remains active for the session unless changed by running `/{project}:target` again.

## Scope Boundaries

- Read `constitution.md` once per session and the targeted feature's `spec.md` (or `spec-and-plan.md`) frontmatter and open-question count. Read the targeted scenario file only when one is specified.
- Do NOT read plan files, tasks, source code, test files, or unrelated specs' bodies.
- Do NOT modify any spec, plan, scenario, or source file. The only file written is the session JSON. Status transitions belong to the pipeline commands (`/{project}:clarify`, `/{project}:plan`, `/{project}:implement`) and to `/{project}:elaborate` (the documented `done → in-progress` back-edge).
- Reference: §spec-lifecycle, §scenarios, §concurrent-features, §text-first-artifacts.

## Instructions

1. When the invocation has no argument (whitespace or empty), read the session JSON to display the current target. If the file is empty or absent, report no target set; otherwise display the feature name and status, the scenario detail when one is targeted (scenario name, the section field or legacy spec-ref field, and the context summary), and the artifacts list. Then stop — the steps below only apply when an argument is supplied. Treat `0`, `00`, or any other non-whitespace string as a valid feature identifier.

2. Parse the argument: when the value contains a slash, split into a feature-part and a scenario-slug; otherwise treat the value as a feature-part with no scenario. Resolve the feature-part by accepting a feature number, a partial name, or a full directory name; search the specs directory for a matching name. If ambiguous, list matches and ask the user to choose. If no match, report the feature does not exist and list available features. (Host responsibility — no runtime primitive iterates the specs directory; otherwise, fall back to the markdown-only path.)

3. Load the constitution file once per session to make its §sections available for subsequent commands. (Host responsibility — no primitive reads the constitution; otherwise, fall back to the markdown-only path.)

4. Recompute dependencies as a safety net by running scripts/gen-spec-deps.sh as a dry run; if the dry run reports a diff, run it for real to sync the frontmatter dependencies from body inline links. The pre-commit hook normally keeps this in sync; this step catches uncommitted body edits. (Host responsibility today; the runtime exposes an equivalent procedural wrapper used by other commands. Otherwise, follow the markdown-only path.)

5. Invoke `read-spec` against the resolved feature to load frontmatter, sections, and the open-question count from the body. The frontmatter status is one of draft, clarified, planned, in-progress, or done.

6. When a scenario was provided, locate the scenario file under the feature's scenarios subdirectory and read it: extract the section field from frontmatter (or the legacy spec-ref field for pre-017 scenarios) and capture the context summary from the body. If the scenario does not exist, list available scenarios and ask the user to choose. (Host responsibility — the runtime does not expose a scenario primitive; otherwise, fall back to the markdown-only path.)

7. Write the session JSON at its canonical path with the feature name, the repo-relative spec directory path, the scenario slug and scenario path when present (omit both fields when targeting a feature without a scenario — clears any previously set scenario), and the current ISO 8601 timestamp. The host applies tempfile + rename atomic-write semantics analogous to the runtime's write primitives. Otherwise (markdown-only path), the host writes through the same conventions directly.

8. Display the resolved target: feature name and current status, scenario detail when present, the artifacts list (which of spec.md or spec-and-plan.md, plan.md, tasks.md, and data-model.md exist), the dependency status from step 4, the open-question count, and the next pipeline step per the Status → next action table below.

## Status → next action

| Status | Open Questions | Next pipeline step |
| --- | --- | --- |
| draft | any | /{project}:clarify |
| clarified | 0 | /{project}:plan |
| planned | 0 | /{project}:implement |
| in-progress | 0 | /{project}:implement |
| done | any | confirm complete; run /{project}:elaborate to reopen via a scenario |

When the status is clarified, planned, or in-progress AND the open-question count is at least one, the next step is `/{project}:clarify` (recovery). This state usually arises from a manual frontmatter edit; the normal back-edge via `/{project}:ask` keeps status and open-question presence in sync.
