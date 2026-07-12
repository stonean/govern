---
section: "Follow-on scenarios"
---

# Write-review-known-field-quoting

## Context

`write-review` renders a waiver's *known* fields (`rule`, `file`, `reason`, `waived-at`, `waived-by`) through `yaml_scalar`, which emits the value bare — it does not quote a string that would re-parse as a non-string (a bare `1234`, `true`, or `null`). The open-schema *extra* waiver fields were hardened to quote such values via `yaml_string` in gvrn 0.20.0, but the known-field path kept the old `yaml_scalar` call, so the two rendering paths are inconsistent.

A waiver whose `reason` (or another known field) were a bare-numeric or bool-like string would render unquoted and then fail the next `RawWaiver` parse — a loud error, not silent corruption. It is practically unreachable (reasons are justification text, `waived-by` is an email, `rule` / `file` are IDs and paths), but the inconsistency sits directly beside the extras path that was just fixed. Surfaced 2026-07-12 during the command-runtime alignment review (gvrn 0.20.0).

## Behavior

- `write-review` renders every known waiver field through the same string-quoting path as the extra fields (`yaml_string`), so a known field whose value would otherwise re-parse as a number, bool, or null is quoted and round-trips through `RawWaiver` unchanged. Known-field and extra-field rendering share one quoting rule.
- A rendered `review.md` re-parses into the same waiver it was written from, with no field silently retyped.

## Edge Cases

- Timestamp-shaped values (`waived-at`, ISO 8601) render without golden-fixture churn — confirm the switch to `yaml_string` leaves their output unchanged before landing it.
- Ordinary string values (justification prose, emails, rule IDs, paths) render identically to today.
- A known field carrying a bare `true` / `1234` / `null`-like string is quoted, where `yaml_scalar` would have emitted it bare and broken the re-parse.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
