---
spec: 008-security-rules
reviewed-at: 2026-05-18T00:00:00Z
reviewed-against: 2c44696447300ddf12ad395e2c64b1dd17a81949
diff-base: 2c44696447300ddf12ad395e2c64b1dd17a81949
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 008-security-rules

## Summary

Re-review triggered by two advisory fixes landed against the spec's artifacts: `data-model.md` (line 44, category schema relaxed from "Short uppercase abbreviation" to `[A-Z][A-Z0-9]*` to admit alphanumeric tokens like `A11YFORM`, `A11YMEDIA`, `VITALS`, etc., introduced by the later rule files) and `tasks.md` (line 56, `FE-DEP-002` typo corrected to `FE-DEPS-002`). Both are doc-only edits; the rule files this spec creates (`framework/rules/security-{backend,frontend}.md`) and the bootstrap/analyze wiring are unchanged since the 2026-05-10 review.

Tech-stack alignment was skipped via `.govern.toml [review] tech-stack-verified = true`. The project's own code surface is markdown + bash + YAML — no backend service, no frontend application — so only the cross-cutting rule file applies; the security-backend/security-frontend files are *shipped content*, not enforceable constraints on the framework's own source. All five passes ran clean against the modified scope.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._

## Pass notes

### Security

`configuration-cross.md` CFG-CONST-* and CFG-ENV-* triggers fire on plans/specs that introduce operator-tunable constants or env vars. The data-model.md change tightens a category-token regex; the tasks.md change fixes a typo. Neither introduces a new constant, env var, or configuration value. No findings.

### Reuse

The schema clarification is one cell of one table; the typo fix is a single token. No duplication.

### Quality

The relaxed regex `[A-Z][A-Z0-9]*` matches every legacy ID (`AUTHN`, `XSS`, `CSRF`, etc.) and the extended set (`A11YFORM`, `A11YMEDIA`, `SCHEMA`, `APIVER`, `VITALS`, `BUNDLE`, `IMAGE`, `FONT`, `LOAD`, `KBD`, `ARIA`, `IDEMP`, `COMPAT`, `SEMHTML`, `CONTRAST`, `ERRENV`, `STATUS`, `PAGE`). It does not match lowercase or symbol-bearing tokens, preserving the original intent. No regression.

### Efficiency

N/A — doc edits.

### Simplicity

The phrase "drawn from the per-surface set below or extended by a later rule-introducing spec" makes the open-extension semantics explicit, replacing the prior implicit "Short uppercase abbreviation" closed-list reading. One sentence, no indirection.

## Rule-file selection

```text
loading rule files: configuration-cross.md
```

Backend / frontend rule files were not loaded — the framework's own implementation is markdown + bash + YAML, not application code. Those files are shipped to adopters; this project does not enforce them against itself.
