---
status: draft
---

# 020 — `/gov:review` Plan

Implements [020 — `/gov:review` code review command with blocking gate](spec.md).

## Overview

`/gov:review` ships as a new markdown slash-command file (`framework/commands/review.md`) following the same shape as `/gov:validate`, `/gov:plan`, and the other pipeline commands — no new code, no new runtime. Each invocation is interpreted by the AI agent against the loaded rules. The blocking gate is enforced by three lightweight, mutually reinforcing edits to `framework/commands/implement.md`, `framework/commands/validate.md`, and `framework/templates/ci/adopter-generators.yml`. Templates and the constitution are updated alongside so newly-created specs ship with the `review:` frontmatter block and the gate is documented in the pipeline section. The scenario file at `scenarios/waiver-expiry.md` captures the subtlest behavior (rule/file-anchored waiver expiry) at the situational tier.

The clarify pass added three behaviors that this plan must propagate: tech-stack alignment as a hard pre-flight gate (with `.govern.toml [review] tech-stack-verified` opt-out), an empty-scope short-circuit, and cross-pass dedupe. These all live in the embedded `framework/commands/review.md` artifact in the spec; the plan's job is to ensure each shipped file picks them up correctly.

## Technical Decisions

### Markdown-instructed implementation, no new code

`/gov:review` is a slash-command file. The agent reads it on invocation and follows the instructions — there is no Go/TypeScript/Python module to write. This matches every other `/gov:*` command and keeps the framework's "text-first artifacts" invariant intact. Trade-off: the agent's review quality is the agent's quality; we cannot bolt on a deterministic linter without changing the framework's shape. Acceptable because the existing security rules in `framework/rules/` are already authoritative natural-language rules read by the agent, not regex patterns.

### Three-mechanism gate composes

The gate fires in three places — `/gov:implement` halt, `/gov:validate` drift check, CI template — because each closes a different failure window:

- **`/gov:implement` halt** catches the local case (operator forgot to run `/gov:review` before completing).
- **`/gov:validate` drift check** catches the desync case (frontmatter says `done` but `review.blocking: true`, or `review.last-run` is missing entirely on a `done` spec).
- **CI template** catches the bypass case (someone edited frontmatter directly to set `done` without running either of the above).

Each mechanism is small and reads the same `review:` frontmatter block — adding a fourth would not strengthen the gate and would multiply maintenance.

### Tech-stack alignment is an agent judgment, not a parser

The alignment check (added during clarify) reads `AGENTS.md` `Tech Stack` and the file scope, then asks the agent whether they appear consistent. Implementation is a paragraph of natural-language instructions, not a polyglot file-extension classifier. The `.govern.toml [review] tech-stack-verified` opt-out exists precisely so adopters with unusual layouts (vendored code, polyglot repos) can bypass false negatives — the LLM judgment is the cheap path; the bypass is the escape valve.

### `.govern.toml` follows the shared-database convention

Per AGENTS.md (Workflow): `.govern.toml` is shared adopter-side state, not a schema owned by any one spec. The new `[review]` section with `tech-stack-verified = true` is documented in this spec's body (under Inputs and Behavior) and in the embedded `framework/commands/review.md` artifact. No signpost or edit on spec 019 is required; that policy is now codified in AGENTS.md.

### Frontmatter `review:` block ships in templates

The block is added to both `framework/templates/spec/spec.md` and `framework/templates/spec/spec-and-plan.md` so every newly-created spec ships with the field shapes pre-populated to safe defaults (`last-run: null`, `must-violations: 0`, `blocking: false`). Existing specs in adopter projects will not have the block until `/gov:review` runs on them; this is intentional — the field is created lazily on first review, and `/gov:validate`'s drift check tolerates `review.last-run: null` until the spec reaches `status: done`.

### Idempotency invariant is a property of inputs, not state

`review.md` is regenerated wholesale on every run from (code in scope) + (loaded rules) + (spec acceptance criteria + scenarios). The only fields permitted to vary across identical-input runs are `reviewed-at` and `reviewed-against` (timestamp and HEAD SHA). Waivers are part of the input set — they are read from spec frontmatter and produce identical `Waived findings` sections across runs as long as the anchor is intact. This makes the AC-6 idempotency check deterministic for a CI snapshot test.

### CI template stays minimal

`framework/templates/ci/adopter-generators.yml` only checks frontmatter state on `done` specs (`review.blocking: true` or `review.last-run` missing → fail). It does **not** invoke `/gov:review` itself in CI. Reason: `/gov:review` is an interactive AI-assisted command; running it in CI would require an AI runtime and a stable API budget. Adopters who want CI-side review run the command locally (or via a pre-merge agent run) and commit the resulting `review.md` and frontmatter. The CI template's job is the bypass-detection backstop, not the primary gate.

### Waiver-expiry warrants a scenario file

Of the 13 acceptance criteria, AC 8's waiver auto-expiry has the subtlest behavior — the framework drops the waiver when the file is renamed or the rule is no longer triggered, and the underlying finding re-blocks. This is exactly the situational tier (specific condition, concrete behavior) the constitution carves out for scenarios. `scenarios/waiver-expiry.md` documents the expected behavior for at least the four edge cases: file renamed, file deleted, rule renamed/removed, and the same rule firing at a different location. Other ACs are general behavior, covered by the spec body.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/commands/review.md` | Create | Source for `/gov:review`; embedded artifact in spec.md is the canonical content |
| `framework/commands/implement.md` | Edit | Pre-`done` review gate (halts when `review.blocking: true` or `review.last-run` missing) |
| `framework/commands/validate.md` | Edit | Add review-drift check on `done` specs; integrate `--fix` to revert to `in-progress` with notice |
| `framework/templates/spec/spec.md` | Edit | Add `review:` block to frontmatter schema |
| `framework/templates/spec/spec-and-plan.md` | Edit | Add `review:` block to frontmatter schema |
| `framework/templates/ci/adopter-generators.yml` | Edit | Add a step that fails when any `done` spec has `review.blocking: true` or missing `review.last-run` |
| `framework/constitution.md` | Edit | Reference the review gate in the pipeline section between `/gov:implement` and `done` |
| `README.md` | Edit | Add `/gov:review` row to Pipeline (advance state) table; add Waivers subsection; update pipeline diagrams |
| `.claude/commands/gov/review.md` | Generated | Produced by the regeneration script — not hand-edited |
| `specs/020-code-review/scenarios/waiver-expiry.md` | Create | Scenario capturing the rule/file-anchored waiver auto-expiry behavior |
| `specs/020-code-review/data-model.md` | Create | Consolidate the data structures introduced (frontmatter block, review.md, waiver records, .govern.toml section) |

## Trade-offs

### Considered and rejected

- **Tunable confidence threshold via `.govern.toml`** — rejected during clarify (Q2). The 80 cutoff is a framework-calibration opinion, not a project decision; tunability would let teams effectively waive the gate by raising the threshold to 100.
- **Required `co-waived-by` field on MUST waivers** — rejected during clarify (Q4). The framework cannot enforce a "different person" guarantee; encoding the requirement in frontmatter would be performative. Adopters with two-author policy can layer fields on the open-schema waiver record and gate them in their own CI.
- **Hash-based auto-reset of `tech-stack-verified`** — rejected during clarify (tech-stack refinement). Adds machinery for a corner case the operator can resolve with one keystroke; the flag is an efficiency optimization, not a correctness invariant.
- **`--all` covering only `in-progress` specs** — rejected during clarify (Q1). Excluding `done` would make the blocking gate retroactively blind to MUST rules added after a feature shipped.
- **Cross-spec signpost on spec 019 for the new `[review]` section** — rejected per AGENTS.md Workflow policy (codified during clarify): `.govern.toml` is a shared adopter-side database; new sections/keys are documented in the adding spec, not in 019.
- **Running `/gov:review` itself in CI** — rejected. Requires an AI runtime in the CI environment and turns review into an external dependency. The CI template stays as a frontmatter-state backstop; adopters run review locally.

### Known limitations

- The agent-judgment tech-stack alignment will occasionally misfire on polyglot or vendored repos. Bypass via `.govern.toml [review] tech-stack-verified = true` is the documented path; documented in the blocking-error message.
- `/gov:review`'s output quality is the agent's output quality. There is no deterministic linter substitute. Mitigated by the loaded rule files being the authoritative source — the agent's judgment is bounded by the rules, not free-form.
- Existing `done` specs in adopter projects (pre-`/gov:review`) lack `review.last-run` entirely. The first `/gov:validate` run after adoption will flag them. This is the intended behavior — adopters re-review or waive on adoption.
- CI gate uses awk-parsed YAML; will be replaced by a deterministic runtime check in v2 (see [§runtime-boundary](../../framework/constitution.md#runtime-boundary) once landed).
