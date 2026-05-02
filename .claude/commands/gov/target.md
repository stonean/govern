---
description: Set the working feature (and optionally scenario) for this session.
argument-hint: "[feature[/scenario]]"
---

# Target

Set the working feature (and optionally scenario) for this session.

## Purpose

Establishes which feature spec all subsequent `/gov:*` commands operate on. Optionally targets a specific scenario within the feature for scenario-aware commands. Must be run before any pipeline command. Remains active for the session unless changed by running `/gov:target` again.

## Scope Boundaries

- Read `constitution.md` once per session and the targeted feature's `spec.md` (or `spec-and-plan.md`) frontmatter and open-question count. Read the targeted scenario file only when one is specified.
- Do NOT read plan files, tasks, source code, test files, or unrelated specs' bodies.
- Do NOT modify any spec, plan, scenario, or source file. The only file written is the session JSON. Status transitions belong to the pipeline commands (`/gov:clarify`, `/gov:plan`, `/gov:implement`) and to `/gov:elaborate` (the documented `done → in-progress` back-edge).
- Reference: §spec-lifecycle, §scenarios, §text-first-artifacts.

## Instructions

### No arguments — display current target

If `$ARGUMENTS` is empty or contains only whitespace (note: `0`, `00`, `000`, or any other valid string is NOT empty — treat any non-whitespace value as a feature identifier):

1. Read `.claude/gov-session.json`. If the file does not exist or is empty, report: "No target set. Run `/gov:target {feature}` to set one."
2. Display the current target:
   - Feature name and current status
   - Scenario name, spec-ref, and context summary (if a scenario is targeted)
   - Artifacts present
3. Inform the user how to change focus:
   - `/gov:target {feature}` — target a feature
   - `/gov:target {feature}/{scenario-slug}` — target a specific scenario
4. Stop here.

### With arguments — set target

1. Parse `$ARGUMENTS`:
   - If it contains a `/`, split into `{feature-part}` and `{scenario-slug}`.
   - Otherwise, treat the entire argument as `{feature-part}` with no scenario.

2. **Resolve feature:** Accept `{feature-part}` as a feature number (e.g., `001`), partial name (e.g., `api-versioning`), or full directory name (e.g., `001-api-versioning`). Search `specs/` for a matching directory.
   - If ambiguous, list matches and ask the user to choose.
   - If no match, report: "Feature `{feature-part}` does not exist." List available features.

3. Read `constitution.md` to load governance rules for the session. Subsequent commands reference specific §sections from this read — do not re-read the constitution unless the session is new.

4. Determine which spec file exists: `spec.md` or `spec-and-plan.md`. Parse the YAML frontmatter block at the top of the file and extract `status`, `dependencies`, and `tags`. Count open questions in the body's `## Open Questions` section.

5. Check which artifacts exist: `spec.md` (or `spec-and-plan.md`), `plan.md`, `tasks.md`, `data-model.md`.

6. **Resolve scenario (if provided):**
   - Check if `specs/{feature}/scenarios/` directory exists. If not, report: "No scenarios exist for this feature. Run `/gov:elaborate` to create one."
   - List `.md` files in `specs/{feature}/scenarios/`.
   - Match `{scenario-slug}` against filenames (without `.md` extension). If no match, list available scenarios and ask the user to choose.
   - Read the scenario file: extract `spec-ref` from the YAML frontmatter and capture the context summary from the body's `## Context` section.

7. Write `.claude/gov-session.json`:

   Feature-only target:

   ```json
   {
     "feature": "{NNN-feature-name}",
     "path": "specs/{NNN-feature-name}",
     "setAt": "{ISO 8601 timestamp}"
   }
   ```

   Feature + scenario target:

   ```json
   {
     "feature": "{NNN-feature-name}",
     "path": "specs/{NNN-feature-name}",
     "scenario": "{scenario-slug}",
     "scenarioPath": "specs/{NNN-feature-name}/scenarios/{scenario-slug}.md",
     "setAt": "{ISO 8601 timestamp}"
   }
   ```

   When targeting a feature without a scenario, omit the `scenario` and `scenarioPath` fields (clearing any previously set scenario).

8. Display:
   - Feature name and current status
   - Scenario name, spec-ref, and context summary (if a scenario is targeted)
   - Artifacts present
   - Dependency status
   - Open question count
   - Next pipeline step (based on status):
     - `draft` → `/gov:clarify`
     - `clarified` → `/gov:plan`
     - `planned` → `/gov:implement`
     - `in-progress` → `/gov:implement`
     - `done` → confirm the spec is complete. To reopen it, run `/gov:elaborate` to add a scenario — that command performs the documented `done → in-progress` back-edge.
