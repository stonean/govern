# 010 — Agent Autonomy

**Status:** draft
**Dependencies:** 000-slash-commands

Evaluate capabilities found in autonomous agent orchestration tools (e.g., GSD-2) and determine which can be adopted within governance's constraints: zero dependencies, markdown-only artifacts, platform-agnostic, and human-in-the-loop pipeline gates.

Each capability is evaluated independently. The outcome for each is one of: **adopt** (add to governance), **adapt** (modify the concept to fit governance's model), or **decline** (not a fit).

## Capabilities Under Evaluation

### Skills system

Composable, context-specific instruction sets loaded based on task type. Currently governance uses a single monolithic `AGENTS.md` per project. A skills system would split this into reusable skill files that commands selectively reference.

- Current state: `AGENTS.md` covers all conventions in one file
- GSD-2 approach: 16 bundled skill profiles loaded by task context
- Governance opportunity: skill files in a `skills/` directory, selectively `@import`-ed by commands or CLAUDE.md based on task type

### Complexity routing

Classify tasks by complexity to inform model selection and batching. Currently governance treats all tasks uniformly regardless of scope.

- Current state: lightweight-vs-standard track decision at spec level only
- GSD-2 approach: simple/standard/complex classification with automatic model routing
- Governance opportunity: complexity field in `tasks.md` entries; `implement` command uses this to suggest model or batch simple tasks

### Stuck detection

Detect when an agent is cycling on the same task without progress. Currently governance has no mechanism to identify repeated failed attempts.

- Current state: no detection — user must notice manually
- GSD-2 approach: sliding-window analysis of dispatch history catches cycles
- Governance opportunity: append-only execution log per spec; `implement` command checks for repeated attempts and suggests decomposition

### Autonomous execution

Chain task execution without pausing for user approval between individual tasks within a phase. Currently every status change requires explicit approval.

- Current state: user approves every transition
- GSD-2 approach: full auto mode — agent loops until milestone is complete
- Governance opportunity: auto-advance between tasks within a phase while preserving approval gates at phase transitions (planned→done)

### Parallel milestones

Work on multiple features concurrently with isolated git state. Currently governance targets one feature at a time via a single session target.

- Current state: single-feature session (`session.json` holds one target)
- GSD-2 approach: multiple worktrees running concurrently with file-based coordination
- Governance opportunity: multi-target session, `--feature` flag on commands, guidance for worktree-based isolation

### Cost controls

Track token usage and enforce budget limits. Currently governance has no cost awareness.

- Current state: no cost tracking — delegated entirely to the AI platform
- GSD-2 approach: per-task token/cost metrics, budget ceilings with warnings/pauses/halts
- Governance opportunity: unclear — governance has no runtime to instrument; this may remain the platform's responsibility

## Acceptance Criteria

- [ ] Each capability has a recommendation: adopt, adapt, or decline
- [ ] Adopted/adapted capabilities have a clear description of what changes to governance artifacts
- [ ] Declined capabilities have a rationale explaining why they don't fit
- [ ] No capability introduces a runtime dependency or requires a specific AI platform
- [ ] Changes respect command file parity (commands/ and .claude/commands/gov/)
- [ ] Changes respect govern file parity (govern/ variants stay in sync)

## Open Questions

- Should the skills system replace `AGENTS.md` entirely, or layer on top of it (base conventions in AGENTS.md, specialized skills in separate files)?
- For complexity routing, who assigns the complexity — the `plan` command automatically, or the user during task creation?
- Is an execution log worth maintaining if it requires every `implement` invocation to append state? Does this create noise in git history?
- For autonomous execution, should auto-advance within a phase be opt-in (flag) or the default behavior?
- For parallel milestones, does governance need to prescribe worktree management, or is it sufficient to support multiple targets and let the platform handle isolation?
- Is there any meaningful cost guidance governance can provide beyond "use your platform's cost controls"?
