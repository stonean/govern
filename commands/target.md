# Target

Set the working feature (and optionally scenario) for this session.

## Purpose

Establishes which feature spec all subsequent `/{project}:*` commands operate on. Optionally targets a specific scenario within the feature for scenario-aware commands. Must be run before any pipeline command. Remains active for the session unless changed by running `/{project}:target` again.

## Instructions

### No arguments — display current target

If `$ARGUMENTS` is empty:

1. Read `{cli-config-dir}/{project}-session.json`. If the file does not exist or is empty, report: "No target set. Run `/{project}:target {feature}` to set one."
2. Display the current target:
   - Feature name and current status
   - Scenario name, spec-ref, and context summary (if a scenario is targeted)
   - Artifacts present
3. Inform the user how to change focus:
   - `/{project}:target {feature}` — target a feature
   - `/{project}:target {feature}/{scenario-slug}` — target a specific scenario
4. Stop here.

### With arguments — set target

1. Parse `$ARGUMENTS`:
   - If it contains a `/`, split into `{feature-part}` and `{scenario-slug}`.
   - Otherwise, treat the entire argument as `{feature-part}` with no scenario.

2. **Resolve feature:** Accept `{feature-part}` as a feature number (e.g., `001`), partial name (e.g., `api-versioning`), or full directory name (e.g., `001-api-versioning`). Search `specs/` for a matching directory.
   - If ambiguous, list matches and ask the user to choose.
   - If no match, report: "Feature `{feature-part}` does not exist." List available features.

3. Read `constitution.md` to load governance rules for the session. Subsequent commands reference specific §sections from this read — do not re-read the constitution unless the session is new.

4. Determine which spec file exists: `spec.md` or `spec-and-plan.md`. Read it and extract status, dependencies, and open question count.

5. Check which artifacts exist: `spec.md` (or `spec-and-plan.md`), `plan.md`, `tasks.md`, `data-model.md`.

6. **Resolve scenario (if provided):**
   - Check if `specs/{feature}/scenarios/` directory exists. If not, report: "No scenarios exist for this feature. Run `/{project}:scenario` to create one."
   - List `.md` files in `specs/{feature}/scenarios/`.
   - Match `{scenario-slug}` against filenames (without `.md` extension). If no match, list available scenarios and ask the user to choose.
   - Read the scenario file to extract spec-ref and context summary.

7. Write `{cli-config-dir}/{project}-session.json`:

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
     - `draft` → `/{project}:clarify`
     - `clarified` → `/{project}:plan`
     - `planned` → `/{project}:implement`
     - `in-progress` → `/{project}:implement`
     - `done` → ask the user if they want to reopen this spec. If yes, update the spec status to `in-progress` and suggest `/{project}:scenario` to capture the change. If no, confirm the spec is complete.
