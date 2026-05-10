---
section: "Waivers"
---

# Waiver expiry

## Context

A MUST violation found by `/gov:review` has been waived via `--waive <rule-id> --reason "<text>"`. The waiver record is anchored to a specific `(rule, file)` pair in the spec's `review.waivers` frontmatter list. On subsequent `/gov:review` runs the system must decide whether each waiver still applies — the rule, the file, and the offending code may all have changed since the waiver was recorded.

Anchoring on `(rule, file)` is intentional. A waiver is a statement about *this specific spot* in the codebase being an acceptable violation; it is not a project-wide pardon for the rule. When the anchor moves or disappears, the justification no longer attaches to a specific location, so the waiver expires and the framework returns to its default position of blocking on MUST violations.

## Behavior

For each waiver in `review.waivers` at the start of every `/gov:review` run, before counting findings into `must-violations`:

1. **File still exists at the anchored path** and the rule still fires there → the waiver applies. The finding is recorded under `## Waived findings` in `review.md` with the waiver's `reason`, and excluded from the `must-violations` count.
2. **File no longer exists at the anchored path** (renamed, deleted, or moved) → the waiver is dropped from `review.waivers` on the next write of the spec frontmatter. The framework does not chase renames — the operator explicitly anchored to that path, and a path change is a meaningful event worth re-evaluating.
3. **Rule still fires at the same file but the code has moved within the file** (e.g., a different line range) → the waiver applies. Line numbers are not part of the anchor; rule + file is the contract.
4. **Rule no longer fires at the anchored path** (offending code was fixed or moved away from that file) → the waiver is dropped from `review.waivers` on the next frontmatter write. There is nothing to waive at that location.
5. **The same rule fires at a *different* file in scope** → the waiver does NOT extend to the new location. A waiver is a per-location decision. If the violation at the new file is also intentional, the operator records a separate `--waive` for that file.

When step 2 or step 4 applies and the same rule still fires *anywhere in scope* (including the same file when the rule's anchor was lost via path rename), the underlying finding re-counts toward `must-violations` and `review.blocking` flips back to `true` if it was previously `false`. The spec returns to the blocking state until either the violation is fixed or a fresh waiver is recorded.

Every drop emits a one-line notice on stdout so the operator notices the lost coverage: `waiver expired: rule {rule-id} at {file} ({reason})`. Silent expiry would let waivers quietly evaporate — the notice is the point.

## Edge Cases

- **File renamed to a path also covered by an existing waiver for the same rule** — the renamed file does not inherit the existing waiver. Each waiver is independent; the framework does not merge waivers across path collisions. The pre-rename waiver expires (path no longer exists); the new path's waiver continues to apply only to its original anchor.
- **Rule removed from the framework entirely** — every waiver referencing the removed rule expires on the next run. The expiry notice still emits for visibility; subsequent runs no longer process the rule.
- **Rule renamed (ID changed)** — a new ID is a different rule by `govern`'s rules-tier conventions (IDs are permanent per `specs/008-security-rules/data-model.md`). Existing waivers referencing the old ID expire; new waivers must be recorded against the new ID if the violation persists.
- **Waiver record malformed** (missing `rule`, `file`, `reason`, `waived-at`, or `waived-by`) — `/gov:review` skips the malformed waiver, emits a one-line warning naming the offending entry, and proceeds. The malformed entry is not auto-removed; the operator must clean it up to prevent the warning from recurring.
- **Multiple waivers for the same `(rule, file)` pair** — only the first applies; duplicates emit a warning on the run that observes them and are not auto-pruned. The framework treats duplicates as operator-authored noise to investigate, not silent state to clean up.

## Open Questions

*None — all resolved at the parent spec's clarify pass.*

## Resolved Questions

*None.*
