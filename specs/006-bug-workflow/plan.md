# 006 — Bug Workflow Plan

## Overview

Add scenario support, a bug decision tree, and brownfield triage to the governance framework. This involves creating two new templates, two new slash commands (`/gov:scenario` and `/gov:triage`), updating four existing commands and their templates, and updating the constitution and README. All artifacts are markdown files — no application code, no persistence.

## Technical Decisions

### Scenarios live alongside their parent spec

Scenario files are placed in `specs/{NNN-feature}/scenarios/{slug}.md`. This co-locates scenarios with the spec they elaborate, making them discoverable without a separate index. The `scenarios/` subdirectory keeps the feature directory uncluttered.

### Scenario template uses plain sections, not Given/When/Then

The spec explicitly states Given/When/Then is not required. The template uses spec-ref, Context, Behavior, and Edge Cases sections. This matches the governance preference for plain language over formal syntax.

### `/gov:scenario` creates both the scenario file and a task entry

When a scenario is created, `/gov:scenario` also appends a task to the parent spec's `tasks.md`. If `tasks.md` does not exist, it creates one. This ensures every scenario has a corresponding implementation task that carries completion status.

### `/gov:triage` operates on a flat `specs/triage.md` file

Triage is a temporary inbox — a flat markdown list, not a directory structure. Each item is walked through the decision tree and migrated to the appropriate spec or scenario. Items are removed from `triage.md` as they are resolved. When `triage.md` is empty, the command reports triage is clean. The file is kept to preserve git history.

### Command templates updated in both `commands/` and `.claude/commands/gov/`

Template changes go into `commands/` (the source of truth for adopting projects). The governance-specific copies in `.claude/commands/gov/` are then re-derived by copying the template and replacing `{project}` with `gov`. This maintains the dogfooding principle from spec 003.

### Constitution updates are additive

New sections are added to the constitution for bug handling and scenario lifecycle. The existing spec phase file structure is extended to include `scenarios/`. No existing sections are removed or restructured.

### README table updated to reflect current status

The README feature table is updated with the correct status for 006-bug-workflow (currently listed as `draft`, will become `planned`). Specs 004 and 005 are also in the table if present.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `templates/scenario.md` | Create | Scenario document starter with spec-ref, Context, Behavior, Edge Cases |
| `templates/triage.md` | Create | Triage inbox format with migration rules |
| `templates/spec.md` | Modify | Add reference to scenarios directory convention |
| `constitution.md` | Modify | Add bug handling section with decision tree, scenario lifecycle, scenario directory convention |
| `commands/scenario.md` | Create | `/scenario` command template for creating scenario files |
| `commands/triage.md` | Create | `/triage` command template for reviewing and migrating triage items |
| `commands/about.md` | Modify | Add `/scenario` and `/triage` to command tables, add scenario concepts |
| `commands/status.md` | Modify | Add scenario counts per spec to dashboard |
| `commands/next.md` | Modify | Add `/scenario` as a suggested next action |
| `commands/validate.md` | Modify | Add scenario-linked task completeness check |
| `.claude/commands/gov/scenario.md` | Create | Governance-specific copy with `gov` replacing `{project}` |
| `.claude/commands/gov/triage.md` | Create | Governance-specific copy with `gov` replacing `{project}` |
| `.claude/commands/gov/about.md` | Modify | Re-derive from updated template |
| `.claude/commands/gov/status.md` | Modify | Re-derive from updated template |
| `.claude/commands/gov/next.md` | Modify | Re-derive from updated template |
| `.claude/commands/gov/validate.md` | Modify | Re-derive from updated template |
| `README.md` | Modify | Document bug workflow and scenario conventions |

## Trade-offs

### Considered: scenario files with their own status field

Rejected. Scenarios are permanent requirement documents. Adding a status field would create a parallel lifecycle that conflicts with the spec lifecycle. The task in `tasks.md` carries the completion status instead.

### Considered: bug files as a standard artifact

Rejected for the default case. The spec explicitly states bug files are rarely needed. Scenarios are the primary artifact. Bug files are documented as an exception for complex root causes, reproduction context, or deferred workarounds.

### Considered: triage as a directory of individual files

Rejected. A flat markdown file is simpler for a temporary inbox. Individual files add filesystem overhead for items that should be migrated quickly.

### Considered: updating only `commands/` templates and not `.claude/commands/gov/`

Rejected. The governance repo must dogfood its own commands. Per spec 003, the gov copies are re-derived from templates after any template change.

## Open Questions Resolved

All open questions were resolved during clarification. See spec.md — the Open Questions section confirms all resolved.
