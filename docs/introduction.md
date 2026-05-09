# Introduction to Spec-Driven Development

> **This is the long-form pitch.** For normative rules — pipeline gates, spec lifecycle, quality standards — `framework/constitution.md` is authoritative.

## What SDD is

The specification is the primary artifact and source of truth. Code is derived from the spec, not the other way around. A spec describes *what the system does*, not *how it does it*.

### What a spec is not

- Not a PRD — specs are living documents that stay accurate after code ships.
- Not Agile or Scrum — SDD is about what the source of truth is, not how teams organize work.
- Not vibe coding — SDD is the discipline layer that makes AI-assisted coding reliable.
- Not just using AI agents — agents are tools; SDD is what makes them consistent.

## The pipeline

Every feature moves through `spec → plan → tasks → implement`. No code is written without a spec. No implementation begins without a plan.

```text
draft ──/clarify──▶ clarified ──/plan──▶ planned ──/implement──▶ in-progress ──/implement──▶ done
```

The status on each spec — `draft`, `clarified`, `planned`, `in-progress`, `done` — tracks where it is in the pipeline. Two back-edges exist: `/ask` reverts a `clarified`, `planned`, or `in-progress` spec to `draft` when a new open question surfaces (the only state that tolerates open questions), and `/elaborate` reverts a `done` spec to `in-progress` when a new scenario is added. The constitution defines what each transition requires.

## The three cycles

Every spec moves through one of three cycles:

1. **Greenfield** — `/specify` → `/clarify` → `/plan` → `/implement` → `done`. New feature designed from scratch.
2. **Brownfield** — `/capture` (sketch spec) → real work touches the area → `/elaborate` adds a scenario, or `/clarify` resolves open questions → `/implement` → `done`. Existing reality being absorbed into specs incrementally.
3. **Reopen** — a `done` spec is revisited because a bug, edge case, or change request surfaces. `/elaborate` adds a scenario, the spec moves back to `in-progress`, and the next pipeline command resumes from there.

All three converge on the same pipeline. What differs is where the spec enters and how precision accumulates.

## How bugs work

A bug is just an unwritten scenario. Most bugs exist because a situation was never formally described. The fix is not a bug report — it is a scenario added to the spec it belongs to.

**Decision tree (in order):**

1. Does a spec exist for this behavior? If not, write it first.
2. Is the spec ambiguous or incomplete? Correct or enhance it.
3. Is the spec clear but the implementation wrong? Add the missing scenario, then fix the code.

There is no separate bug file. The scenario captures the correct behavior. Git history on that file records when and why it was added. A descriptive commit message covers the rest.

## How brownfield projects adopt this

- Don't frontfill bugs you aren't actively working on.
- Write specs for areas you're actively touching — let adoption spread naturally.
- Use `/log` to drop raw items into `specs/inbox.md` without breaking flow.
- Use `/groom` to walk the inbox and route each item to its proper spec or scenario via the bug decision tree.
- The goal is for `inbox.md` to eventually disappear.

Adoption is incremental by feature area, not a big-bang effort.

## Slash commands

The framework is operationalized through slash commands installed during adoption. All commands are session-aware — run `/target` to set the working feature, then use pipeline commands in context.

| Cluster | Commands |
| --- | --- |
| Pipeline (advance state) | `/specify`, `/clarify`, `/plan`, `/implement`, `/validate` |
| Elaborate (add precision) | `/ask`, `/elaborate` |
| Brownfield (absorb existing reality) | `/capture`, `/log`, `/groom` |
| Orient | `/target`, `/status`, `/help` |
| Bootstrap | `/govern`, `/configure` |

For full descriptions of each command and the rules each enforces, see `framework/constitution.md` and the command sources in `framework/commands/` and `framework/bootstrap/`.

## Key mindset shifts

- A bug is just an unwritten scenario.
- Scenarios are specs at a lower level of abstraction — same format, same discipline.
- The spec absorbs knowledge that ticket systems bury in closed tickets.
- No work begins without a spec or scenario to satisfy.
