# 013 — Text-First Artifacts Data Model

The data structure introduced by this feature is the **YAML frontmatter schema** for spec and scenario files. The schema is the canonical metadata format consumers parse; this document declares the structure once so the constitution, templates, and slash commands stay aligned.

The same schema is published in `framework/constitution.md` as a markdown table — the constitution is the authoritative source for adopters; this document is the planning-phase reference and the basis for the validation logic in `/gov:validate`.

## Frontmatter Block

Every spec file (`spec.md`, `spec-and-plan.md`) and scenario file (`scenarios/{slug}.md`) begins with a YAML frontmatter block:

```yaml
---
status: clarified
dependencies: [000-slash-commands, 007-govern-workflow]
tags: [format, migration]
---
```

Delimiters are `---` on the first line and a closing `---` on a subsequent line. The body of the document begins after a blank line following the closing delimiter. The frontmatter MUST be parseable as YAML.

## Schema

### Spec files (`spec.md`, `spec-and-plan.md`)

| Field | Required | Type | Allowed values | Description |
| --- | --- | --- | --- | --- |
| `status` | yes | string | `draft`, `clarified`, `planned`, `in-progress`, `done` | Spec lifecycle state. Read by every pipeline gate; written on transition. |
| `dependencies` | yes | list of strings | spec slugs (e.g., `002-events`, `005-auth`); empty list permitted | Specs this feature depends on. Empty list replaces the bold-prefix `none` convention. |
| `tags` | no | list of strings | free-form; see starter vocabulary | Cross-cutting categories used by graph-view consumers. |

### Scenario files (`scenarios/{slug}.md`)

| Field | Required | Type | Allowed values | Description |
| --- | --- | --- | --- | --- |
| `spec-ref` | yes | string | parent spec ref, conventionally `{NNN-feature} — {Section}` | Identifies the parent spec and section the scenario elaborates. |
| `tags` | no | list of strings | free-form | Scenario-level cross-cutting tags. May overlap with parent spec tags. |

### Open-schema rule

Additional fields beyond those listed above are permitted and ignored by uninterested consumers. This applies uniformly to both spec and scenario files. Examples that adopters or future governance work might add: `owner`, `target_release`, `created_at`, `description`, `aliases`. Consumers MUST NOT error on the presence of unknown fields. `/gov:validate` reports unknown fields as informational findings (not errors).

## Validation Severity

`/gov:validate` checks frontmatter against this schema with the following severity:

| Check | Severity |
| --- | --- |
| Frontmatter block missing on a spec or scenario file | Hard fail |
| Frontmatter YAML malformed (unparseable) | Hard fail |
| `status` missing on a spec | Hard fail |
| `status` value not in the allowed set | Hard fail |
| `dependencies` missing on a spec | Hard fail |
| `dependencies` not a list (e.g., string instead of list) | Hard fail |
| `spec-ref` missing on a scenario | Hard fail |
| `tags` missing or empty on a spec | Advisory |
| Unknown fields present | Informational |
| `dependencies` list contains slugs that do not match an existing spec directory | Advisory (existing cross-reference check) |

Hard fails block the validation pass. Advisory and informational findings are reported but do not block.

## Starter Tag Vocabulary

Published in the constitution as guidance, not enforcement. Adopters and future governance specs MAY introduce new tags as needed; the agent's prompt in `/gov:specify` surfaces existing tags from sibling specs as autocomplete to drive convergence by reuse rather than ceremony.

| Tag | Suggested use |
| --- | --- |
| `cli` | Specs about slash commands or command-line interactions |
| `bootstrap` | Specs about adopting governance, project scaffolding, or initialization |
| `process` | Specs about workflow, lifecycle, or pipeline behavior |
| `templates` | Specs about template files (spec, plan, scenario, project-readme, etc.) |
| `security` | Specs about security rules, authentication, authorization |
| `agent` | Specs about AI-agent behavior, capabilities, or coordination |
| `format` | Specs about artifact formats, schemas, or serialization conventions |
| `pipeline` | Specs about the spec → plan → tasks → implement flow |
| `migration` | Specs that convert existing artifacts to a new format or convention |

## Notes

- **The schema is open by design.** This is the structural representation of "extensibility without coordinated parser changes" — adding a new optional field requires no migration, no parser update, and no breakage for consumers that don't read it.
- **`tags` empty list vs. missing.** The convention is to include `tags: []` rather than omit the key. Templates emit `tags: []`. Validation treats both empty list and missing key as equivalent advisory findings.
- **Scenarios have no `status` field by design** — per the constitution, scenarios are written or not; their completion is tracked through the parent spec's `tasks.md`.
- **Order of keys is not significant** — YAML parsers are insensitive to key order. Templates use the order `status`, `dependencies`, `tags` (specs) or `spec-ref`, `tags` (scenarios) for readability.
