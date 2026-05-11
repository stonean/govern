---
spec: 020-code-review
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 020-code-review

## Summary

The `/gov:review` command itself: `framework/commands/review.md`, three reinforcing edits to `implement.md` / `validate.md` / `adopter-generators.yml` that implement the three-mechanism blocking gate, frontmatter schema additions to two templates, `waiver-expiry` scenario, and `data-model.md`. Self-review — `/gov:review` is the command being defined here. All five passes ran; no findings. `blocking: no`.

This re-run regenerates the review.md that was first produced at the bootstrap of 020 (precedent review file dated 2026-05-10T22:31:48Z). Per the idempotency invariant, the body content is identical to that bootstrap review modulo `reviewed-at` and `reviewed-against`. The earlier review noted that `AGENTS.md` had no `Tech Stack` section and the alignment check was bypassed by inspection; that section now exists (added 2026-05-10) so the alignment check succeeds normally on this run.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._ All five passes ran.

## Pass notes

### Security

No security-sensitive code introduced. The bash steps in `framework/templates/ci/adopter-generators.yml` use `find -maxdepth 2` with explicit predicates over spec frontmatter from the repo itself — no user-controlled input, no `eval`, no curl. Loaded security rules (`security-backend.md`, `security-frontend.md`) do not apply: no HTTP, no auth, no DOM.

### Reuse

Implementation reuses existing patterns: frontmatter schema (per spec 013), `.govern.toml` adopter-side storage (per spec 019), command-file structure (matching `/gov:validate`, `/gov:plan`, et al.), and the three-mechanism gate composability (each mechanism reads the same `review:` block rather than maintaining parallel state).

### Quality

The three reinforcing checks (implement halt, validate drift, CI gate) read the same frontmatter shape with consistent grandfather logic — a `done` spec with no `review:` block is exempt. Edge cases addressed during clarify: empty scope, missing AGENTS.md Tech Stack, cross-pass dedupe, waiver expiry on rule/file changes. Idempotency is structurally enforced.

### Efficiency

No performance concerns. `/gov:review` runs once per invocation; CI file-scan operations use bounded `find` predicates. The `.govern.toml [review] tech-stack-verified` opt-out exists to skip the agent-judgment alignment check on routine runs.

### Simplicity

Considered and rejected during clarify (per `plan.md`): tunable confidence threshold, required `co-waived-by` field, hash-based auto-reset of `tech-stack-verified`, separate `/gov:review` invocation in CI. All four rejections documented under §"Considered and rejected". Final design is minimal: one new command file, three edits, two template touches, one CI step, one `.govern.toml` key.

## Notes

- The waiver-processing additions (per-run apply/expire/no-extend, malformed/duplicate warnings) are operator-state handling, not new findings to track.
- Future `/gov:review` runs against an unchanged target reproduce this report modulo timestamps and `reviewed-against`.
