---
spec: 021-runtime-boundary
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 021-runtime-boundary

## Summary

Constitutional amendment (§text-first-artifacts opening, new §runtime-boundary subsection, drift-prevention table row) plus the deterministic CI tripwire that enforces the markdown-only opt-in invariant: `framework/runtime-tools.txt` (empty manifest), `scripts/lint-runtime-fallback.sh` (proximity-scan for graceful-fallback markers near runtime-tool references), `scripts/lint-frontmatter.sh` (shape-only frontmatter integrity check), `.github/workflows/markdown-only-pipeline.yml` (the five-check workflow). All scripts and CI YAML audited globally. Spec is `in-progress`; no findings block its advance to `done`. All five passes ran; no findings. `blocking: no`.

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

`lint-runtime-fallback.sh` and `lint-frontmatter.sh` both use `set -euo pipefail`, no `eval`, no network, no user-controlled input — they walk repo files and emit text. The CI workflow pins `actions/checkout@v4` and `actions/setup-node@v4`, runs with `contents: read`, and gates the workflow via `paths:` filter. The runtime-tools manifest is empty (populated by spec 022); the lint exits 0 on an empty manifest, which is the correct no-op behavior.

### Reuse

`lint-*.sh` reuses the convention established by `gen-*.sh` (017): single-purpose bash script, `--help` flag, structured exit codes. The frontmatter integrity check is intentionally shape-only — the rigorous parser is `/gov:validate`'s hard-fail tier; this lint is a CI smoke test, not a duplicate.

### Quality

Five deterministic checks (a–e in the workflow) plus the runtime-tool absence check on PATH compose a tripwire: a future PR that silently introduces `python`, `go run`, etc. without a graceful fallback in `framework/commands/*.md` fails CI. The 20-line proximity window is documented in the script header; the marker enum (`Otherwise|Fallback|If unavailable|markdown-only path`) is the documented contract. The fallback-marker check trades false-positive risk for a fully derived signal — consistent with §design-principles "never depend on human diligence" (no author-supplied marker to forget).

### Efficiency

Lint scripts iterate the repo once per invocation. The CI workflow path filter prevents runs on unrelated PRs. Insertion sort and bounded glob patterns appropriate for n<30 specs.

### Simplicity

No new runtime introduced — the spec ships the constitutional scope for a future opt-in runtime and the tripwire that protects the markdown-only path until then. Runtime implementation is explicitly deferred to spec 022. The runtime-tools manifest is empty by design — populated when 022 lands, not earlier.
