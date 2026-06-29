---
spec: 000-slash-commands
scenario: implement-skips-planned-prompt
reviewed-at: 2026-06-28T00:00:00Z
reviewed-against: 98f859520f2672b58830911d891f6f9eeb14a98e
diff-base: 98f859520f2672b58830911d891f6f9eeb14a98e
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 000-slash-commands (scenario: implement-skips-planned-prompt)

## Summary

Clean across all five passes — 0 MUST, 0 SHOULD, 0 low-confidence. The change
removes the planned → in-progress confirmation gate from `/gov:implement`: the
prose gate trigger in `framework/commands/implement.md` step 4 is gone (merged
into the `set-status` step), the `--auto` carve-out now lists only
`in-progress → done`, the generated `.claude/commands/gov/implement.md` mirror
is regenerated, the runtime golden/fixtures that encoded the old gate are
re-blessed, and a §cross-spec-impact signpost is recorded on
`010-agent-autonomy`. Loaded rules (configuration-cross, security-backend,
api-backend, performance-backend) target application code — constants/env-vars,
HTTP API design, auth/input security, query performance — none of which this
change introduces; the in-scope artifacts are command-prose markdown, Rust
**test** fixtures, and one test-file edit. The full runtime test suite passes
(391 lib + 16 + 10 + 7 + 3 + 2 + 1, zero failures) and all touched markdown
lints clean. Not blocking.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — no issues were appended to `specs/inbox.md` during this work._

## Skipped passes

_None._ All five passes ran.

## Pass notes

### Security

No auth, input-handling, HTTP, persistence, or crypto surface in the diff — the
shipped `security-backend` rules find nothing to flag. Notably, the edited
out-of-boundary parity test (`runtime/tests/parity.rs`) still feeds a writeCode
edit that escapes the write boundary and still asserts the `out-of-boundary-edit`
rejection; only the now-absent gate-response line was removed and the request id
shifted to `req-1`. The write-boundary enforcement coverage is intact.

### Reuse

No duplicated logic. The removed gate step folds into the existing `set-status`
step rather than introducing a parallel path; the regenerated Claude mirror
flows through the canonical `scripts/gen-claude-commands.sh` generator, not a
hand edit.

### Quality

Correct and consistent. Step renumbering (4–7) leaves no dangling
cross-references: the carve-out's "see step 4" resolves to the `set-status`
step, and "step 2" (Scope Boundaries) still resolves to `derive-boundary`. The
re-blessed golden shows the `gate-confirm` envelope and its progress line
removed, steps flowing 1–6, and writeCode at `req-1`; the parity test asserts it
byte-for-byte and passes. `stdin.jsonl` and the fixture spec's pipeline
description were aligned before re-blessing. Full suite green.

### Efficiency

N/A — prose and test-data edits; no loops, queries, or hot paths.

### Simplicity

A net simplification: one fewer procedure step and one fewer runtime gate. The
carve-out parenthetical is concise and the rationale is stated inline. No new
flags, args, config keys, or state shapes.

### Out of scope (informational)

The planned → in-progress `set-status` guard (`from: planned`) and the
already-`in-progress` resume path are unchanged by this work; the scenario
documents resume behavior as unchanged. Not a finding against this change.
