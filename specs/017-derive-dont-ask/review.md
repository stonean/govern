---
spec: 017-derive-dont-ask
reviewed-at: 2026-05-10T00:00:00Z
reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
diff-base: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 017-derive-dont-ask

## Summary

Largest cross-cutting change in the project: removes discipline-dependent frontmatter (`title`, `tags`, `[simple]`), introduces four `scripts/gen-*.sh` generators (`gen-readme-table`, `gen-help-tables`, `gen-spec-deps`, `install-hooks`), `.githooks/pre-commit` hook orchestration, the `framework/rules/configuration.md` rule file, CI workflow (`generators.yml`), and migration of dogfood specs. The generator scripts and CI workflow constitute the only "real code" in govern; all were audited globally. All five passes ran; no findings. `blocking: no`.

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

Each generator uses `set -euo pipefail`, no `eval`, no network, no user-controlled input — they iterate spec frontmatter from the repo itself. `install-hooks.sh` modifies `git config core.hooksPath .githooks`, which is local to the repo; `chmod +x` is scoped to `.githooks/pre-commit`. CI workflow uses `actions/checkout@v4` with `contents: read` permissions. The shipped adopter pre-commit hook is split (per 018) so adopters own the outer file and govern owns the inner — no silent overwrites.

### Reuse

The `gen-*.sh` script convention establishes the pattern reused by spec 021's `lint-*.sh` scripts. The marker-comment splice pattern (`<!-- generated:...:start --><!-- generated:...:end -->`) is shared across `gen-help-tables.sh` and `gen-readme-table.sh`.

### Quality

Generators handle the missing-marker case explicitly (non-zero exit with named error). `gen-spec-deps.sh` correctly skips fenced code blocks and blockquote-prefixed lines when extracting body links — preventing forward-pointer signposts on done specs from polluting the derived dependency list. The "frozen archaeology" rule on done-spec frontmatter (no migration of dogfood specs' title/tags) prevents unnecessary churn on historic state.

### Efficiency

Insertion sort in `gen-spec-deps.sh` is appropriate for n<30 specs. Generators run in dry-run mode in CI; the pre-commit hook stages outputs so commits are self-contained.

### Simplicity

This spec embodies the §design-principles "never depend on human diligence" rule — it removed every input that required the author to remember to fill it in (title, tags, `[simple]`, frontmatter dependencies as the source of truth). Replaced each with a derived signal: filename → title, body inline links → dependencies, body content → README and help tables.
