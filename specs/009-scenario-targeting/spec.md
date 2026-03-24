# 009 — Scenario Targeting

**Status:** done
**Dependencies:** 006-bug-workflow

Promote scenarios to first-class targets in the governance pipeline. Currently, the session target is always a feature — commands operate on the feature's spec, plan, and tasks. This spec extends targeting so that individual scenarios within a feature can be targeted, allowing commands like `question`, `clarify`, `status`, and `implement` to operate at scenario granularity.

The motivation is context management: as specs grow, loading the entire spec to work on a single scenario wastes agent context. Scenario-level targeting keeps the agent focused on a bounded artifact.

## Session Target Extension

The session file gains an optional `scenario` field. When present, commands that support scenario-level operation use the scenario file as their primary context instead of the feature spec.

Session file with feature-only target (current behavior, unchanged):

```text
feature: "{NNN-feature-name}"
path: "specs/{NNN-feature-name}"
setAt: "{ISO 8601 timestamp}"
```

Session file with scenario target:

```text
feature: "{NNN-feature-name}"
path: "specs/{NNN-feature-name}"
scenario: "{scenario-slug}"
scenarioPath: "specs/{NNN-feature-name}/scenarios/{scenario-slug}.md"
setAt: "{ISO 8601 timestamp}"
```

When a scenario is targeted, the feature context is always available — the scenario refines which artifact within the feature is the primary focus.

## Target Command Changes

The `target` command accepts an extended syntax to target scenarios:

- `target` (no arguments) — displays the current target, including scenario if set. Informs the user they can run `target {feature}` or `target {feature}/{scenario-slug}` to change focus.
- `target {feature}` — targets the feature, clears any scenario (current behavior)
- `target {feature}/{scenario-slug}` — targets the feature and a specific scenario within it

When targeting a scenario, the command validates that the scenario file exists under the feature's `scenarios/` directory. If the `scenarios/` directory does not exist, it reports "No scenarios exist for this feature. Run `/gov:scenario` to create one." If the directory exists but the slug does not match a file, it lists available scenarios and asks the user to choose.

When targeting a feature that does not exist, it reports "Feature `{feature}` does not exist."

The target display includes scenario information when one is targeted: scenario name, which spec section it references, and its context summary.

## Scenario Template Changes

The scenario template gains `## Open Questions` and `## Resolved Questions` sections. This allows questions to be captured and resolved directly against the scenario rather than bubbling up to the parent spec — the same pattern specs use.

```text
# {Scenario Name}

**spec-ref:** {NNN-feature-name} — {Section name}

## Context

## Behavior

## Edge Cases

## Open Questions

## Resolved Questions
```

## Command Behavior by Target Level

Commands fall into three categories based on how they respond to scenario targeting:

### Scenario-aware commands

These commands change behavior when a scenario is targeted:

- **question** — reads the scenario file for context, appends the refined question to the scenario's Open Questions section instead of the spec's
- **clarify** — resolves open questions in the targeted scenario file; when no scenario is targeted, operates on the spec as today
- **status** — displays scenario-level detail (open questions, spec-ref, context summary) when a scenario is targeted
- **implement** — scopes implementation context to the targeted scenario when one is set

### Feature-only commands

These commands always operate at the feature level regardless of scenario targeting:

- **specify** — creates features, not scenarios
- **plan** — plans are feature-level artifacts
- **validate** — validates the feature spec and all its artifacts

### Scenario-creating commands

- **scenario** — after creating a scenario file, sets it as the session target (both feature and scenario) without prompting for confirmation

## Clarify Behavior for Scenarios

When `clarify` is run with a scenario targeted:

- Resolve open questions in the scenario file
- Enumerate edge cases specific to the scenario's behavior
- Verify the scenario's behavior section is unambiguous
- The scenario does not have its own status field — resolution is complete when all open questions are removed

When `clarify` is run with only a feature targeted (no scenario):

- Existing behavior is unchanged — resolves spec-level open questions
- Scenario-level open questions are not surfaced — spec-level and scenario-level questions are independent concerns; the user must target the scenario to resolve those

## Acceptance Criteria

- [ ] Session file supports an optional `scenario` and `scenarioPath` field
- [ ] `target` command accepts `{feature}/{scenario-slug}` syntax
- [ ] `target` command validates scenario existence and lists alternatives on mismatch
- [ ] `target` command displays scenario detail when one is targeted
- [ ] Scenario template includes `## Open Questions` and `## Resolved Questions` sections
- [ ] `question` command appends to the scenario's Open Questions when a scenario is targeted
- [ ] `question` command appends to the spec's Open Questions when no scenario is targeted
- [ ] `clarify` command resolves scenario-level open questions when a scenario is targeted
- [ ] `clarify` command resolves spec-level open questions when no scenario is targeted (unchanged)
- [ ] `scenario` command sets the newly created scenario as the session target
- [ ] `status` command shows scenario detail when a scenario is targeted
- [ ] Feature-only commands (specify, plan, validate) ignore the scenario field
- [ ] `target` command with no arguments displays the current target including scenario when set
- [ ] `target` command with no arguments informs user how to change focus
- [ ] `target` command reports no scenarios exist when the feature has no `scenarios/` directory
- [ ] `target` command reports feature not found when the feature does not exist
- [ ] Command file parity maintained between `commands/` and `.claude/commands/gov/`
- [ ] Govern file parity maintained across `govern/` variants

## Open Questions

*None — all resolved during clarification.*

## Resolved Questions

- **Feature-level clarify and scenario questions:** Stay silent. Spec-level and scenario-level open questions are independent concerns; the user must target the scenario to resolve those.
- **Target with no arguments:** Display the current target (including scenario) and inform the user how to change focus.
- **Scenario command target switch confirmation:** Just do it — no confirmation prompt. The user explicitly asked to create the scenario, so switching is the obvious next step.
