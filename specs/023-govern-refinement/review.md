---
spec: 023-govern-refinement
reviewed-at: 2026-05-16T14:50:00Z
reviewed-against: 670a3181acbfefbf3dab31eb7df84f5442672e8a
diff-base: 7283e2ca3af69039f08643fec77e6b3c4b6a93b4
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 023-govern-refinement

## Summary

Spec 023 is a framework-shape refactor with no application code under review. The implementation scope is:

- **One new bash script** (`scripts/gen-configure-mcp.sh`, ~110 lines) — the MCP allow-list generator. Reads `framework/runtime-tools.txt`, emits per-host permission entries into managed blocks in `framework/bootstrap/configure/claude.md` and `framework/bootstrap/configure/auggie.md`.
- **Prose rewrites** across `framework/commands/*.md` (specify, ask, analyze, target, status, clarify, plan, implement, review, validate→analyze, help, groom), `framework/constitution.md`, `framework/bootstrap/govern.md`, `framework/templates/spec/spec.md`, `framework/templates/project/{agents,project-readme}.md`, `docs/introduction.md`, `README.md`, `AGENTS.md`, `specs/README.md`.
- **One generator-script update** (`scripts/gen-help-tables.sh`) — `commands-elaborate` marker → `commands-refine`; drop `/elaborate` and `/capture` rows; rename `validate.md` → `analyze.md` in the pipeline-table builder.
- **One generator-script comment update** (`scripts/lint-frontmatter.sh`) — `/gov:validate` → `/gov:analyze`.
- **Two pre-commit hook updates** (`.githooks/pre-commit`, `framework/bootstrap/hooks/govern-pre-commit`) — register `gen-configure-mcp.sh`; drop `spec-and-plan.md` from stage-loop globs.
- **One CI template comment update** (`framework/templates/ci/adopter-generators.yml`) — `/gov:validate` → `/gov:analyze`; `find` clause drops `spec-and-plan.md`.

Three file deletions: `framework/commands/{capture,elaborate,validate}.md` and `framework/templates/spec/spec-and-plan.md`. (The validate→analyze case is a content-preserving rename rather than a true deletion; the file lives at `framework/commands/analyze.md`.)

The Rust code that landed under Phase A (the `create-scenario` and `append-task` primitives in `runtime/src/`, plus the version-bump and CHANGELOG entries for `gvrn 0.4.0` and `gvrn 0.4.1`) belongs to spec [022](../022-deterministic-runtime/spec.md) per [§cross-spec-impact](../../framework/constitution.md#cross-spec-impact). That code was reviewed under [specs/022-deterministic-runtime/review.md](../022-deterministic-runtime/review.md) and returned 0 MUST / 0 SHOULD / 1 low-confidence; the gvrn 0.4.1 patch addressed all four SHOULDs from the initial 0.4.0 pass.

## Rule coverage

Loaded:

- `framework/rules/security-backend.md` (70 rules) — backend security
- `framework/rules/security-frontend.md` (31 rules) — frontend security
- `framework/rules/configuration.md` (11 rules) — config constants + env vars

The 112-rule set targets application-code patterns (authentication, authorization, input validation, deserialization, XSS, CSRF, dependency pinning, env-var defaults, etc.). The 023 scope contains:

- One bash script with deterministic, file-driven I/O — no untrusted input, no shell exec on user-supplied values, no env-var defaults, no constants violations.
- Prose changes — no code patterns to evaluate.

No rule's Verification trigger fires against this scope. Each pass below records 0 findings as a function of code+rules per the derive-don't-ask invariant; this is not an evaluation skip.

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

## Pass detail

### Security pass

`scripts/gen-configure-mcp.sh` reads a trusted source file (`framework/runtime-tools.txt`, repo-internal), iterates known-format `gov-rt:<verb>-<noun>` entries, and emits well-formed YAML-array-style bullet items into managed blocks of two markdown files. No external input, no shell exec on user-supplied values, no path traversal surface. The atomic write pattern uses `splice` + `mv` — same idiom as the other `scripts/gen-*.sh` generators. Clean.

### Reuse pass

The new generator uses the same shape as `scripts/gen-readme-table.sh`, `scripts/gen-help-tables.sh`, and `scripts/gen-spec-deps.sh` — `--dry-run` flag handling, marker-pair splice, `cmp -s` for idempotency. No duplicated logic worth extracting; each generator's marker shape and content rules differ enough to keep them independent.

### Quality pass

The bash script handles bash 3.2 (macOS default) — uses `while IFS= read` rather than `mapfile` (caught during initial implementation when `mapfile` was used and failed on macOS). Comment trimming handles trailing whitespace. Empty `runtime-tools.txt` exits non-zero with a clear message. Marker missing → exit non-zero with descriptive error.

The prose rewrites preserve the canonical pipeline semantics (status transitions, gate enforcement, three-tier rule model). The classifier heuristic in `ask.md` documents an inherently fuzzy decision but always defers to the user via the `flip` override at the refinement-approval gate, so a classification error has cost ≤ 1 keystroke.

### Efficiency pass

Linear scan of `runtime-tools.txt` (≤ 25 lines). Single `mktemp` for output staging. No N+1 patterns. Clean.

### Simplicity pass

The generator is shaped to match the existing `gen-*.sh` pattern; deviating from that shape would be the simplicity concern. The `printf` templates for Claude and Auggie are inline (3-line block per host) rather than abstracted — appropriate for two host-formats with no shared structure beyond the JSON object wrap on the Auggie side.

The `/ask` classifier heuristic is documented as prose (per the resolved question) rather than as a new LLM extension point — the simpler design choice. No premature schema for a binary classification.

## Verification artifacts

- `gen-help-tables.sh --dry-run` reports "No changes (help.md in sync)"
- `gen-spec-deps.sh --dry-run` reports "No changes (all specs in sync)"
- `gen-readme-table.sh --dry-run` reports "No changes (README in sync)"
- `gen-configure-mcp.sh --dry-run` reports "No changes (mcp-allow blocks in sync)"
- `scripts/lint-tool-coverage.sh` exits 0
- `scripts/lint-procedure-parseability.sh` exits 0
- `scripts/lint-frontmatter.sh` exits 0
- `markdownlint-cli2` returns 0 errors across the 48 markdown files touched by this spec

CI workflows on `670a318`: `markdown-only-pipeline` green, `generators` green, `runtime` green.
