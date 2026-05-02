# 005 — Workflows Data Model

## Registry file

**Location in framework:** `framework/workflows/registry.json`
**Location in adopted project (after `/govern` sync):** `workflows/registry.json`

The registry is a single JSON document containing a top-level array of registry entries:

```json
[
  { "name": "...", "category": "...", "trigger": { "field": "...", "value": "..." }, "template": "...", "description": "..." }
]
```

The file uses a top-level array (not an object with a `version` or `entries` wrapper) to keep the document grep-friendly and minimal. Schema evolution is managed by adding optional fields to entries; breaking changes would coordinate via the framework's regular update path.

## Registry entry

| Field | Type | Required | Constraints |
| --- | --- | --- | --- |
| `name` | string | yes | Human-readable workflow name (e.g., `"ESLint"`, `"pytest"`). Used in the present-and-accept UI. |
| `category` | string | yes | Must equal one of the fixed categories: `Testing`, `Linting`, `Formatting`, `Migrations`, `Code Review`, `Deployment`. Case-sensitive. |
| `trigger` | object | yes | Single field/value pair; see below. |
| `trigger.field` | string | yes | Tech stack key. Must equal one of: `project_type`, `backend_language`, `backend_framework`, `database`, `messaging`, `backend_test_runner`, `frontend_language`, `frontend_framework`, `css_ui`, `frontend_test_runner`. |
| `trigger.value` | string | yes | Value compared (case-insensitively) against the user's selection for `trigger.field`. |
| `template` | string | yes | Path to the workflow file relative to `framework/workflows/`. Must end in `.md`. The field is named `template` (rather than `file` or `workflow`) because the file contains placeholders that get substituted at scaffold time. |
| `description` | string | yes | One-line explanation of what the workflow does. Shown beside `name` in the present-and-accept UI. |

### Validation rules

Init and govern validate each entry at read time. An entry that fails validation is logged as a warning and skipped; the surrounding flow continues. Validation failures:

- Missing required field
- `category` not in the fixed set
- `trigger.field` not in the recognized set
- `template` path does not end in `.md`
- `template` file is not found in `framework/workflows/` (warned at scaffold time, not at registry-load time, so a registry can ship ahead of workflow files being added)

If the registry file itself is missing or unparseable JSON, init/govern emit a single warning (`Workflow registry not found or invalid, skipping workflow recommendations`) and continue without the workflow step. The pipeline does not abort.

## Trigger / tech stack mapping

Tech stack keys correspond to the questions asked by init step 4 (sourced from spec 004) and to the rows of the AGENTS.md Tech Stack table:

| Key | Init question | AGENTS.md layer |
| --- | --- | --- |
| `project_type` | 4a | (not in table — used to gate match scope) |
| `backend_language` | 4b | Backend language / Language |
| `backend_framework` | 4b | Backend framework |
| `database` | 4b | Database |
| `messaging` | 4b | Messaging |
| `backend_test_runner` | 4b | Backend test runner |
| `frontend_language` | 4c | Frontend language / Language |
| `frontend_framework` | 4c | Frontend framework |
| `css_ui` | 4c | CSS/UI |
| `frontend_test_runner` | 4c | Frontend test runner |

When matching against an AGENTS.md table (govern's path), the **Language** layer maps to either `backend_language` or `frontend_language` based on the existing layer label rules from 004 (use `Language` for backend-only or frontend-only projects, and the disambiguated `Backend language` / `Frontend language` for fullstack).

## Categories

Fixed enum (per resolved question #4):

- `Testing`
- `Linting`
- `Formatting`
- `Migrations`
- `Code Review`
- `Deployment`

Adding a new category requires a registry entry that uses it **plus** an update to the constitution-side category list (currently captured in this data model and the spec). This keeps the UI grouping consistent.

## Workflow file

Each workflow file is a `.md` file at `framework/workflows/{filename}` matching an entry's `template` path. The directory is flat — no inner `templates/` subdirectory.

**Naming convention:** `{workflow}-{language}-{tool}.md` (e.g., `lint-typescript-eslint.md`).

**Format:** the same prompt-and-instructions format as `framework/commands/*.md`. Workflow files use the standard placeholders:

- `{project}` — replaced with the adopting project's slug at scaffold time
- `{cli-config-dir}` — replaced with the agent's `config_dir` (e.g., `.claude`)

**Scaffolded destination:** `{config_dir}/commands/{project}/workflows/{filename}`. The scaffold copy preserves the file stem; e.g., `lint-typescript-eslint.md` is scaffolded as `lint-typescript-eslint.md` under the project's `workflows/` subdirectory.

Workflow files are not synced into adopted projects on `/govern` runs. They are fetched on demand from upstream at scaffold time using the same URL pattern as other governance file fetches.

## Project-level state

**`{config_dir}/commands/{project}/workflows/`** — the directory that holds scaffolded workflow files in an adopted project. Existence of a file inside this directory means the corresponding workflow has already been scaffolded and is treated as "owned" by the project (not overwritten on subsequent govern runs). Removing a file from this directory makes the workflow eligible to be re-offered on the next govern run.

**`workflows/registry.json`** — the project's local copy of the framework registry, written by govern's manifest sync (`update` strategy). Provides a manifest of available workflows for inspection and is the source govern reads at recommendation time within a single run.
