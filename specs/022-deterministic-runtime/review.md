---
spec: 022-deterministic-runtime
reviewed-at: 2026-05-24T22:55:00Z
reviewed-against: working-tree (uncommitted; diff base 1ce8666 docs(022): advance status to done)
diff-base: 1ce8666
must-violations: 0
should-violations: 0
low-confidence: 1
skipped-passes: []
notes: "Two SHOULD findings from the initial review (AGENTS-WORKFLOW-NO-DEAD-REFS and AGENTS-GOTCHA-OBSOLETE) were resolved by landing the mechanical sweep across AGENTS.md, runtime/tests/, and 7 done-spec bodies in the same change set, per the §spec-lifecycle mechanical-sweep clause."
---

# Review — 022-deterministic-runtime (task 40 session-file consolidation)

## Summary

The session-file consolidation work (`.claude/gov-session.json` → `.govern.session.toml`) is implementation-clean: every primitive contract updated, every test green (379/379), every lint clean, the audit passes. The five review passes turned up **0 MUST violations** and **2 SHOULD violations**, both in the doc-sweep dimension rather than the runtime code — the mechanical rename across live artifacts was applied to the framework command sources and most spec bodies but missed `AGENTS.md` (3 hits), several pre-022 done-spec bodies (6 files, 13 hits), the parity test's TODO scaffolding file (1 file, 2 hits), and one fixture's spec body (1 file). One low-confidence reuse note on `SESSION_FILE`'s placement is recorded for completeness.

**Blocking status: no.** The runtime correctness review is clean. The doc sweep is a mechanical cleanup that the framework's own "No dead references" workflow rule (`AGENTS.md` Workflow §3) explicitly classifies as not requiring a `done → in-progress` back-edge on the touched specs — it's bundled here under the same change set rather than a separate ticket.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

### SHOULD: AGENTS-WORKFLOW-NO-DEAD-REFS — Incomplete mechanical sweep for `.claude/gov-session.json` → `.govern.session.toml`

- **File**: 11 files across `AGENTS.md`, `runtime/tests/`, and `specs/NNN-*/` done-spec bodies.
- **Rule**: From `AGENTS.md` Workflow — *"No dead references in live artifacts. When renaming or removing a name (spec slug, capability, command, identifier, parenthetical descriptor, etc.), update every reference across **live artifacts**: `framework/`, `scripts/`, `runtime/` (including `tests/fixtures/`, `tests/golden/`, `tests/parity/`), `.github/`, `docs/`, `README.md`, `AGENTS.md`, and `specs/NNN-*/` (including done-spec bodies)."* The rule treats partial sweeps as a quality failure because "a reader following a forward-pointer or back-reference in live artifacts must never land on an outdated name."
- **Finding**: Task 40 swept `framework/commands/*`, `framework/bootstrap/govern.md`, `framework/bootstrap/configure/claude.md`, `framework/constitution.md`, `framework/templates/project/gitignore`, the runtime crate, and spec 022's own bodies. It did NOT sweep:
  - `AGENTS.md:44` — "Use the `Write` tool, not Bash redirects, for `.claude/gov-session.json`" — the rule still applies but the target path is now `.govern.session.toml`.
  - `AGENTS.md:45` — example "`.claude/gov-session.json`" as a repo-relative path; outdated.
  - `AGENTS.md:53` — see the separate finding below; this is a special case (the gotcha is now obsolete, not merely outdated).
  - `runtime/tests/parity/target/expected.txt:3,5` — TODO scaffolding describing how to capture the strict-files target.
  - `runtime/tests/fixtures/target-basic/specs/002-target/spec.md:24` — fixture spec body mentions the strict-files path.
  - `specs/023-govern-refinement/spec.md:90` — motivation prose names the file.
  - `specs/007-govern-workflow/spec.md:112` — registry table column listing per-agent session paths.
  - `specs/005-workflows/spec.md:104` — design rationale cites the file as a JSON format precedent (which is now stale on two counts: the file is TOML and the path changed).
  - `specs/003-bootstrap-automation/spec.md:90`, `plan.md:31`, `tasks.md:14` — done-when AC and plan prose.
  - `specs/010-agent-autonomy/spec.md:69,90,147` — resolved-questions prose discussing session-state shape.
  - `specs/000-slash-commands/review.md:46`, `scenarios/target-clear-flag.md:9,13,19` — review notes and the `--clear` flag scenario body.
- **Auto-fixable**: yes (mechanical find-and-replace where the reference is a live forward-pointer; verify each hit's framing before replacing — some entries in the consolidation scenario, migration body, and CHANGELOG are explicit legacy references and MUST be preserved).
- **Suggested fix**: Run a single-commit sweep that updates the 11 files listed above. The rule's own enforcement clause ("**mechanical sweep** under §spec-lifecycle and does NOT trigger the done → in-progress back-edge") means the done specs touched here stay `done`. Recommended grep:

  ```bash
  grep -rln "\.claude/gov-session\.json\|\bgov-session\.json\b" \
    AGENTS.md runtime/tests/ specs/ \
    | grep -v "022-deterministic-runtime\|session-file-consolidate\|CHANGELOG"
  ```

  Each remaining hit gets `.claude/gov-session.json` → `.govern.session.toml` (and bare `gov-session.json` → `.govern.session.toml`). In the registry-table case (`007-govern-workflow/spec.md:112`), the row should be removed entirely — the file is no longer per-agent. In the JSON-precedent case (`005-workflows/spec.md:104`), the prose needs a non-trivial rewrite (the precedent argument no longer holds for either format or path) or the citation should be dropped.

### SHOULD: AGENTS-GOTCHA-OBSOLETE — `AGENTS.md` line 53 gotcha is resolved by the consolidation and should be retired

- **File**: `AGENTS.md:53`.
- **Rule**: Same "No dead references" rule, plus the §design-principles rule against documentation that pushes false guidance (a stale gotcha actively misleads).
- **Finding**: The line-53 gotcha — *"Claude Code prompts on `.claude/gov-session.json` writes despite the per-path allowlist"* — describes a workaround for the Claude Code harness's built-in `.claude/` directory protection. The gotcha's own "escape hatches" section even names the fix that the consolidation just implemented: *"(b) move `gov-session.json` out of `.claude/` (e.g., `.govern/session.json`) via a new spec amending 023's session-file path."* Now that `0.10.0` moves the file to `.govern.session.toml` at the repo root, the entire workaround is obsolete — and a reader following the gotcha will be confused by guidance that doesn't match the current code.
- **Auto-fixable**: no (judgment call: outright remove vs. rewrite as historical context). The cleaner option is removal — the consolidation is now the canonical state, and a brief CHANGELOG entry already documents the migration.
- **Suggested fix**: Remove the bullet at `AGENTS.md:53` entirely. The corresponding `framework/` change at `framework/bootstrap/configure/claude.md` (the `Edit/Write` permission entries) already points at `.govern.session.toml`; no replacement guidance is needed because the harness's `.claude/`-directory protection no longer fires on this file.

## Low-confidence findings

### Low-confidence (CFG-CONST-001): `SESSION_FILE` cross-module constant placement

- **File**: `runtime/src/primitives/write_session.rs:39` (defined `pub(crate)`); `runtime/src/primitives/dashboard.rs:19` (imported).
- **Rule**: `CFG-CONST-001` — *"Shared constants — values used across multiple modules — MUST live in a centralized location idiomatic to the project's language (e.g., `shared/constants/` in JavaScript/TypeScript, `internal/constants/` in Go, a top-level constants module in Python) rather than being duplicated across modules."*
- **Finding**: `SESSION_FILE = ".govern.session.toml"` is defined in `write_session.rs` and consumed by `dashboard.rs`, so the *value* is centralized (no duplicate literals across modules, which is the rule's primary failure mode). But the *location* — inside the primitive that owns the write half — isn't a top-level shared-constants module. Rust idiom here is contested: some projects keep file-format constants alongside the owning primitive (the "this primitive is the source of truth for the file's name" interpretation), others extract them into a top-level `constants.rs` (the strict CFG-CONST-001 reading). The existing pattern in this crate is the former (e.g., `merge_managed_block::DEFAULT_MARKER`, `merge_permissions`'s former `DEFAULT_PATH`), so the placement matches established convention.
- **Confidence**: 55 (the rule is satisfied on the no-duplication test, contested on the location test, and the codebase already follows the same pattern for `merge_managed_block`). Recorded for visibility; not blocking.
- **Suggested fix** (if pursued): move `SESSION_FILE` to a new `runtime/src/constants.rs` exposing `pub const SESSION_FILE: &str` and re-export from `lib.rs`. Update both call sites. Skip the change unless the codebase adopts a top-level constants module for other cross-module values too — fixing one without the others would create inconsistency.

## Waived findings

*None.*

## Skipped passes

*None.*

## Pass summary

| Pass | MUST | SHOULD | Low-confidence |
| --- | --- | --- | --- |
| Security | 0 | 0 | 0 |
| Reuse | 0 | 0 | 1 |
| Quality | 0 | 2 | 0 |
| Efficiency | 0 | 0 | 0 |
| Simplicity | 0 | 0 | 0 |

**Security**: The diff touches no auth, persistence, network, or input-validation surfaces. `dashboard.rs::load_session_target` reads a constant repo-root path — no traversal surface (BE-INPUT-004 N/A). `write_session.rs` retains its `validate_no_traversal` checks on the `path` and `scenario-path` args, which still carry user-controlled values. `merge_permissions.rs::path` becoming required removes a footgun where a non-Claude host would have silently written to a Claude-shaped path — net security improvement, not a regression. No security rules fire.

**Reuse**: One low-confidence note on `SESSION_FILE` placement (above). Otherwise clean — the consolidation removed duplicated path literals from `write_session` and `dashboard` (and `main.rs::run_exec`) in favor of a single constant import, which is the *opposite* of a CFG-CONST violation.

**Quality**: Two SHOULD findings, both in the documentation-sweep dimension (above). Runtime code, tests, and fixtures are clean — every callsite updated, every test green (343 lib + 3 atomic-writes + 5 exec-subprocess + 16 mcp + 10 parity + 2 walker = 379 tests), parity goldens re-blessed, MCP descriptions updated, gitignore entry shipped, bootstrap migration entry written, CHANGELOG entry added.

**Efficiency**: No regressions. The TOML parse/serialize on session-file I/O is microseconds for a sub-1KB document — equivalent to the prior JSON path. The `dashboard.rs::load_session_target` read still short-circuits on `is_file()` before touching the parser. `main.rs::run_exec`'s walker-context seed goes via `serde_json::to_value(toml::Value)` once per `gvrn exec` invocation — same complexity class as the prior `serde_json::from_str` against the JSON file.

**Simplicity**: The consolidation is a net simplification. Removed: `DashboardArgs.session_path: Option<String>` (intermediate parameterization), `merge_permissions::DEFAULT_PATH` constant + `resolve_path` helper. The runtime no longer carries per-host knowledge for the session file. Per-fixture `.claude/` directories deleted (7 fixtures). The `WriteSessionArgs` struct surface shrank from the path-parameterization design back to its 0.9.x shape (with the path-encoded keys renamed kebab-case on disk).

## Process notes

- Re-bless rationale captured in CHANGELOG 0.10.0 entry. Parity goldens' field-ordering change (TOML's `BTreeMap` iteration → alphabetized JSON output) is deterministic per-run and consistent with the spec's `writeCode` cache-breakpoint contract (the first four fields' order is preserved; the rest is unspecified).
- The `gvrn exec` walker-context seed reads `.govern.session.toml` via `toml::Value` → `serde_json::Value` bridging so nested fixture context (arrays-of-tables for `entries`, sub-tables for `substitutions`) survives the round-trip. This is the secondary use of the file beyond session-target storage; documented in the plan.md fixture-testing section and exercised by `runtime/tests/parity.rs::govern_basic_*`.
