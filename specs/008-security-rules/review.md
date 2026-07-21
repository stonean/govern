---
spec: 008-security-rules
reviewed-at: 2026-07-21T17:24:17Z
reviewed-against: ba807cc50336165b183c5d8f6182a4935c9e87c6
diff-base: ba807cc50336165b183c5d8f6182a4935c9e87c6
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 008-security-rules

## Summary

Re-review triggered by a reopen: `FE-DEPS-005` (dependency-originated network egress) was added to `framework/rules/security-frontend.md` after `FE-DEPS-004`, before `## FE-PII`. The gap was surfaced upstream from an adopter — every existing `FE-DEPS` rule governs *what code is loaded and from where* (vulnerability scan, integrity, injection, pinning); none governed *what a dependency does on the network once running*. The motivating case was `@mui/x-telemetry` posting to a vendor endpoint on dev-server reloads while silent in production builds. No `data-model.md` change was required — `FE`/`DEPS` is already in the category table and the file carries ID grammar, not a per-ID manifest.

Tech-stack alignment was skipped via `.govern.toml [review] tech-stack-verified = true`. The project's own code surface is markdown + bash + YAML — no backend service, no frontend application — so the security-frontend file is *shipped content*, not an enforceable constraint on the framework's own source. The change is a single rule addition to that shipped content; all five passes ran clean against the modified scope.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

*None.*

## Waived findings

*None.*

## Skipped passes

*None.*

## Pass notes

### Security

The added rule is itself a security constraint (dependency egress), not code that could violate one. It follows the established `FE-DEPS` format — Statement (RFC 2119 MUST/MUST NOT), Rationale, Verification, Source — and cites OWASP A08:2021. No new constant, env var, or configuration value is introduced. No findings.

### Reuse

`FE-DEPS-005` occupies distinct semantic ground from the four existing `FE-DEPS` rules (runtime egress vs. load-time provenance, vulnerability, and pinning) and from `FE-CSP-*` (a build-time obligation vs. a browser-enforced backstop). No overlap or duplication.

### Quality

ID grammar holds (`FE-DEPS-005`, `[A-Z][A-Z0-9]*` category, zero-padded sequence, next free number in the family). No duplicate IDs (`grep '^### ' … | sort | uniq -d` clean). Insertion preserves family ordering; markdownlint clean.

### Efficiency

N/A — a rule-content addition.

### Simplicity

The rule is intentionally independent of `FE-CSP-*` rather than folded into it, which keeps each concern citable on its own terms. One rule, no indirection.

## Rule-file selection

```text
loading rule files: configuration-cross.md
```

Backend / frontend rule files were not loaded against the framework's own source — the implementation is markdown + bash + YAML, not application code. Those files are shipped to adopters; this project does not enforce them against itself.
