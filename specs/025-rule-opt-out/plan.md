# 025 — Rule-file opt-out via `.govern.toml` Plan

Implements [025 — Rule-file opt-out via `.govern.toml`](spec.md).

## Overview

The feature is a narrow, markdown-only change. There is no new module, no new parser, no new CLI surface — `.govern.toml` is read ad-hoc by the agent runtime in each command that needs it, following the precedent established by [spec 020's](../020-code-review/spec.md) handling of `[review] tech-stack-verified`. This spec adds one new array-of-tables key (`[[review.disabled-rule-files]]`) and threads its consultation into `/gov:review`'s rule-file selection step (§Behavior step 5), with discoverability notices on stdout and a one-line callout in `/gov:status`. Two files in the spec's original Affected list are dropped after planning (see Trade-offs).

## Technical Decisions

### Where the disabled list is consulted

`framework/commands/review.md` §Behavior step 5 is the single read site. The sequence after this spec lands:

1. Discover `framework/rules/*.md` by directory walk.
2. Classify each file by suffix; filter the recognized set by the detected stack (existing 024 logic).
3. **New:** read `.govern.toml` `[[review.disabled-rule-files]]`. For each entry:
   - If `file` matches a file in the post-stack-filter set, drop it and emit the `disabled-rule-file: <name> — <reason> (.govern.toml)` notice.
   - If `file` matches a `framework/rules/` basename that was NOT selected by stack detection, emit the `disabled-rule-file (no-op): <name> not selected by stack detection` notice (no drop — there was nothing to drop).
   - If `file` does not match any `framework/rules/` basename, emit the `unknown disabled-rule-file: <name> (no such file in framework/rules/)` warning (already covered by AC3).
   - Malformed / duplicate entries follow the warn-and-skip pattern documented in §Malformed and duplicate waivers (review.md lines 358–375).
4. Emit the existing `loading rule files: <list>` notice with disabled files excluded.

Reading order matters: the disabled-rule-file notices fire **before** the `loading rule files` notice so the stdout transcript reads top-down as "what got excluded, then what's loading."

### Notice ordering and whitespace handling

The single-line notice format is fixed by AC2. Internal whitespace (including newlines from TOML multi-line strings) in `reason` is collapsed to single spaces when emitting the stdout line — TOML `"""..."""` strings are valid input but the notice is rendered single-line. Implemented as: `reason.split_whitespace().join(' ')`-equivalent in the agent runtime, or the markdown-instruction equivalent.

### Exit code invariance

Warnings (malformed, duplicate, unknown, length-fail) emit to stdout but never set the exit code. `/gov:review`'s exit code remains `1` iff `must-violations > 0` (per existing §Output line 426). This is enforced by the AC4 wording and called out again in the §Output section of the updated review.md.

### `/gov:status` integration

`framework/commands/status.md` step 6 already lists below-the-table callouts (blocked specs, recovery-state specs, tags-in-use). Add a fourth callout when `.govern.toml` `[[review.disabled-rule-files]]` is non-empty:

```text
disabled rule files: <N> (.govern.toml) — <comma-separated basenames>
```

Single line; no verbose mode. Adopters who need the reasons read `.govern.toml` directly. Surfacing the list at all is the AC6 contract — the dashboard is the discoverability surface, not a full pretty-printer.

### Constitution edit

`framework/constitution.md` §rules has a subsection on filename-suffix (lines 285–295, added by spec 024). Append a short paragraph after that subsection — not a new subsection — naming the file-level opt-out and pointing at `framework/commands/review.md` §Inputs. The constitution describes the framework's contracts; the implementation details live in the command file.

### Example TOML block

The canonical `.govern.toml` schema example lives in `framework/bootstrap/govern.md` lines 246–262 (showing `[pinned]` and `[workflows]`). Add a commented-out `[[review.disabled-rule-files]]` block alongside, so adopters running through bootstrap see the schema at the same place they see the others. This replaces the spec body's reference to a `framework/templates/project/govern-toml.md` file that does not exist.

### `framework/commands/analyze.md` — no edit needed

Verified during planning: `framework/commands/analyze.md` does not read `.govern.toml` at all (grep returns no matches). AC7 ("`/gov:analyze` does NOT error on the new key") is structurally satisfied by analyze never seeing the file. The Affected files row from spec.md is dropped — there is nothing to extend.

### `scripts/lint-govern-toml.sh` — out of scope

Verified during planning: the script does not exist. The spec body Affected files row carries an `(if it exists)` qualifier, and the Q3 resolution treats `.govern.toml` hygiene as a separate single-purpose tool. Adding the script as part of 025 is scope creep — runtime warnings (stdout) already cover the operator-feedback path. Defer to a future spec when demand emerges.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/commands/review.md` | edit | §Inputs (add `[[review.disabled-rule-files]]` Config bullet); §Behavior step 5 (apply disabled-files filter after stack filter, emit per-entry notices); §Output (document the new notices); §Notes for adopters (one bullet on the override and a link to `.govern.toml`'s `[[review.disabled-rule-files]]`) |
| `framework/commands/status.md` | edit | Step 6 — add a fourth below-the-table callout when the disabled list is non-empty |
| `framework/constitution.md` | edit | §rules — append a brief paragraph after the filename-suffix subsection mentioning the file-level opt-out |
| `framework/bootstrap/govern.md` | edit | Example TOML block (lines 246–262) — add a commented-out `[[review.disabled-rule-files]]` example alongside `[pinned]` and `[workflows]` |
| `specs/025-rule-opt-out/spec.md` | edit | Mark all 9 AC checkboxes after implementation is verified; record the analyze/lint-script drops in the frontmatter `review` block via `/gov:review` |

## Trade-offs

### Considered and rejected

- **Surface `.govern.toml` parsing in a shared TOML reader module.** Govern has no shared parser — each command's markdown instructions tell the agent to read the file ad-hoc. Following the precedent set by [spec 020's](../020-code-review/spec.md) handling of `[review] tech-stack-verified` keeps the change shape uniform; introducing a shared module for one new key is premature.
- **Extend `framework/commands/analyze.md` to validate `.govern.toml`.** Analyze reads markdown spec artifacts, not `.govern.toml`. Adding a `.govern.toml` validator to analyze would couple two unrelated concerns (artifact-vs-artifact audits vs. config hygiene); single-purpose `scripts/lint-govern-toml.sh` is the right home if and when it exists.
- **Create `scripts/lint-govern-toml.sh` as part of this spec.** Spec body line 68's `(if it exists)` qualifier and the Q3 resolution both treat the lint as a separate concern. The feature works without it — malformed entries warn at runtime via stdout. Deferring keeps 025 narrow.
- **Verbose `/gov:status` listing of disabled files with reasons.** The dashboard is meant to be glanceable; full reasons live in `.govern.toml`. One-line callout with basenames is the AC6 contract.
- **Auto-removing malformed disabled entries.** Same posture as malformed waivers (review.md lines 358–365): operator-authored state is not framework-collected garbage. Warn and skip; the operator cleans it up.

### Known limitations

- The disabled list is consulted once per `/gov:review` invocation, at step 5. There is no in-process rescan or hot-reload — consistent with the rest of `.govern.toml` reads in the codebase.
- The no-op notice (`disabled-rule-file (no-op): ...`) depends on stack detection having produced a definite result. The existing tech-stack alignment gate (§Behavior step 4) guarantees this either via `tech-stack-verified = true` or a fresh check; if both fail the run halts before step 5, so the no-op branch is never reached without a stack.
- Adopters who rename a rule file in their fork (e.g., `accessibility-frontend.md` → `a11y-frontend.md`) and forget to update `.govern.toml` get the `unknown disabled-rule-file` warning and the rule file stays enforced. This is the AC3 design — quiet re-enablement would be a worse failure mode.
