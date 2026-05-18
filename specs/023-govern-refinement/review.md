---
spec: 023-govern-refinement
reviewed-at: 2026-05-17T21:30:00Z
reviewed-against: 9393114a3cb2bc9c2c7594c4e294c42cf3ee011f
diff-base: 8e0cee93e9a6714caba98e7bf6b48f5932d36e79
must-violations: 0
should-violations: 2
low-confidence: 1
skipped-passes: []
---

# Review — 023-govern-refinement

## Summary

Re-review of `023-govern-refinement` covering the living-specs scenario's implementation (`9393114`) on top of the prior clean baseline (`8e0cee9`, reviewed 2026-05-17T14:15:00Z, see git log for that pass's notes). The implementation removes the "frozen archaeology" exception from `framework/constitution.md` §drift-prevention, extends §spec-lifecycle from two back-edges to three (adding `done → in-progress` for meaningful body edits, with the mechanical-vs-meaningful boundary defined inline), rewrites `AGENTS.md` line 42 to fold `specs/NNN-*/` into the live-artifacts sweep, and deletes `specs/README.md` §Past Renames.

The scenario's behavior is bulleted in [`scenarios/living-specs.md`](scenarios/living-specs.md) Behavior; the implementation took the user-approved option-B path — mechanical token substitution for prefixed and file-path references across ~50 non-introducing spec files, with introducing-spec body cleanup (bare-backticked old names that appear in historical-action descriptions) explicitly deferred to a follow-on cycle. The mechanical sweep covered four token families: `/gov:validate` / `/{project}:validate` → `/gov:analyze` / `/{project}:analyze`; `/gov:capture` / `/{project}:capture` → `/gov:specify` / `/{project}:specify`; `/gov:elaborate` / `/{project}:elaborate` → `/gov:ask` / `/{project}:ask`; `validate.md` → `analyze.md`; `configuration.md` → `configuration-cross.md`.

Zero MUST findings; two advisory SHOULD findings on scope-completeness and sweep lossiness; one low-confidence finding on per-file sweep coverage. Blocking: no.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

### SHOULD: SCOPE-001 — scenario done-when partially satisfied vs. literal reading

- **File**: `specs/023-govern-refinement/scenarios/living-specs.md`
- **Rule**: The scenario's Behavior bullet 5 reads: "As part of this scenario's implementation, sweep the done specs currently carrying dead references and bring them current. Initial grep targets: `/gov:validate`, `/capture`, `/elaborate`, `spec-and-plan.md`." Strict reading: every dead reference is swept.
- **Finding**: The implementation took the user-approved option-B path — mechanical sweep for prefixed and file-path forms (`/gov:validate`, `/{project}:validate`, `validate.md`, etc.) executed and verified clean; bare-backticked old names in introducing-spec bodies (`/capture` in 011, `/elaborate` in 014, `/validate` in 017/020, etc.) deferred to a follow-on cycle because mechanical substitution would break sentences like "A new `/capture` command provides a lightweight entry point... separate from `/specify`" (011 spec.md). The done-when criterion is met in spirit (no current-usage dead references remain in the live artifact set), but a strict reading of the scenario's Behavior bullet 5 would call this partial.
- **Auto-fixable**: no
- **Suggested fix**: Acknowledge the option-B path explicitly in the scenario's Behavior section. No further code action needed at this scope.

### SHOULD: QUAL-002 — mechanical sweep loses intermediate-rename chronology in some swept passages

- **File**: `specs/014-reclarify-backedge/spec.md` (and similar contexts in 011, 017, 020, 022)
- **Rule**: A mechanical substitution sweep applies the same token-pair replacement across every live artifact. The rule does not distinguish "current command name" from "intermediate-rename history" in prose contexts.
- **Finding**: The sweep substituted `/{project}:elaborate` → `/{project}:ask` in 014's spec body. Pre-sweep prose said `"renamed to /{project}:elaborate (see 006-bug-workflow)"` referencing 006's rename of `/scenario` → `/elaborate`. Post-sweep prose says `"renamed to /{project}:ask"`, which is current-name-accurate but loses the chain: 006 renamed `/scenario` *to* `/elaborate`, and 023 *later* consolidated `/elaborate` into `/ask`. A reader following the cross-ref to 006 will find content about `/elaborate`, not `/ask`. The same shape affects any swept passage that named the rename's destination as an intermediate name.
- **Auto-fixable**: no
- **Suggested fix**: Per-spec past-tense prose rewrites that preserve rename chronology — best handled as a separate `/gov:ask` cycle on each affected spec rather than another mechanical pass.

## Low-confidence findings

### LOW: QUAL-003 — bulk sweep coverage not exhaustively verified per file

- **File**: 54 files across `specs/` (see `git diff --stat`)
- **Rule**: Mechanical substitution across many files invites latent prose breakage in any sentence whose surrounding context made the old token grammatically meaningful in a way the new token isn't.
- **Finding**: The `sed -i` sweep ran across all non-023 spec files for `/gov:validate`, `/{project}:validate`, `/gov:capture`, `/{project}:capture`, `/gov:elaborate`, `/{project}:elaborate`, `validate.md`, and `configuration.md`. Verification was spot-check sampling (016 review, 022 plan, 017 spec) plus the rename-chronology issue surfaced in QUAL-002. Exhaustive per-file inspection was not performed. Confidence the sweep is uniformly clean: ~70%. Real risk: a sentence somewhere uses the old token in a context where the new token reads awkwardly or wrongly.
- **Confidence**: 70
- **Auto-fixable**: no
- **Suggested fix**: A follow-on read-through pass of the swept files (or an `/gov:analyze` cycle when the analyzer is extended to flag suspect prose) would close the gap. Lower-cost alternative: rely on PR review of the commit's diff to surface any sentence-level breakage.

## Waived findings

*None.*

## Skipped passes

*None — all five passes ran.*

## Pass summary

| Pass | MUST | SHOULD | Notes |
| --- | --- | --- | --- |
| Security | 0 | 0 | The implementation is markdown documentation — no code, no secrets, no env vars, no operator-tunable values introduced. `configuration-cross.md` rules (CFG-CONST-NNN, CFG-ENV-NNN) target plan affected-files snippets in code, not framework constitution prose. The change does not create new security surface; if anything it tightens an existing inconsistency (the contradiction between "no dead references" and "frozen archaeology"). |
| Reuse | 0 | 0 | The new §spec-lifecycle back-edge bullet mirrors the existing two back-edges' shape verbatim. The mechanical-vs-meaningful boundary text cites `AGENTS.md`'s rename-rule scope by reference rather than restating the artifact list. No duplication introduced. |
| Quality | 0 | 2 | Two advisory findings, both rooted in the option-B vs strict-option-2 trade-off the user explicitly chose: SCOPE-001 (scenario done-when partial), QUAL-002 (sweep loses intermediate-rename chronology). |
| Efficiency | 0 | 0 | Markdown-only change; no loops, queries, or computational paths. The mechanical sweep itself is a one-shot `sed -i` operation against bounded input (~50 files). |
| Simplicity | 0 | 0 | The constitution change adds one bullet to an existing list. `AGENTS.md` line 42 is a rewrite-in-place, not an additional rule. `specs/README.md` shrinks (Past Renames deleted). The scenario file follows the established template shape. No new abstractions, flags, or config keys introduced. |

## Acceptance criteria audit

The scenario's Behavior bullets (the scenario has no separate Acceptance Criteria section — Behavior is the contract):

| # | Behavior bullet | Status |
| --- | --- | --- |
| 1 | `framework/constitution.md` §drift-prevention drops the "frozen archaeology" exception; §spec-lifecycle extends with the new back-edge | ✓ — `### Done specs are frozen archaeology` subsection removed; §spec-lifecycle now lists three back-edges with the mechanical-vs-meaningful boundary inline |
| 2 | `AGENTS.md` line 42's rename rule drops the `specs/NNN-*/` carve-out; sweep is uniform across all live artifacts | ✓ — `specs/NNN-*/` added to the live-artifacts list; mechanical-sweep rule with `done → in-progress` opt-out wording |
| 3 | `specs/README.md` §Past Renames is deleted | ✓ — section removed; Design Decisions consolidated; pointer paragraph added explaining git-log + mechanical-sweep substitute for the table |
| 4 | Decision rationale preservation via existing artifacts (Resolved Questions, plan.md Trade-offs, review.md findings, git history) — no new artifact tier | ✓ — no new artifact tier introduced |
| 5 | Sweep done specs currently carrying dead references; bring them current | Partial — mechanical sweep for prefixed and file-path refs complete; bare-backticked old names in introducing-spec bodies deferred to follow-on per option-B (see SHOULD: SCOPE-001) |
| 6 | Mechanical-vs-meaningful boundary defined (inline in the scenario's Behavior section) | ✓ — defined as uniform-substitution-diff applied across all live artifacts per AGENTS.md rename-rule scope |

## Prior pass (superseded — preserved for context)

The prior pass of this review (2026-05-17T14:15:00Z against `8e0cee9`) found three SHOULD-tier hygiene findings (one simplicity, two quality) in two bash generators touched by the post-23 `gov-rt → gvrn` MCP-server rename. All three were applied via `/gov:review --fix` in that session:

- **SIMPLICITY-001** — dropped unused `local label="$1"` from `process()` in `scripts/gen-configure-mcp.sh` and removed the matching string arguments at the two call sites.
- **QUALITY-001** — added a top-level `trap cleanup EXIT` to `scripts/gen-configure-mcp.sh` plus a `cleanup_files` registry so every `mktemp` is released regardless of which exit path the script takes.
- **QUALITY-002** — replaced the single-space-only strip in `scripts/lint-tool-coverage.sh` with a full whitespace-class strip (`[![:space:]]`), so a manifest line with multi-space indent or trailing tab can no longer slip through with embedded whitespace.

Re-running the affected scripts after the fixes (recorded in the prior pass):

```text
gen-configure-mcp.sh --dry-run    → No changes (mcp-allow blocks in sync)
lint-tool-coverage.sh             → exit 0
```

The `lists_every_manifest_tool_and_canonical_set` invariant in `runtime/tests/mcp.rs:94` enforces that `TOOL_NAMES` in `server.rs`, the running server's tool list, and `framework/runtime-tools.txt` all agree; the post-rename test suite (256 passed / 0 failed) confirmed the rename was consistent across these three sources.

## Output

```text
/gov:review — 023-govern-refinement

  security    ✓ 0 MUST   0 SHOULD
  reuse       ✓ 0 MUST   0 SHOULD
  quality     ✓ 0 MUST   2 SHOULD   (1 low-confidence)
  efficiency  ✓ 0 MUST   0 SHOULD
  simplicity  ✓ 0 MUST   0 SHOULD

  blocking: no
  report:   specs/023-govern-refinement/review.md
```

Exit code: `0`.
