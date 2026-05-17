---
spec: 022-deterministic-runtime
scenario: ask-consolidation
reviewed-at: 2026-05-16T14:35:00Z
reviewed-against: 27a0a7ec001694d540c30de33bb3a4bcdee89fe7
diff-base: 7283e2ca3af69039f08643fec77e6b3c4b6a93b4
must-violations: 0
should-violations: 0
low-confidence: 1
skipped-passes: []
---

# Review — 022-deterministic-runtime (ask-consolidation scenario)

## Summary

Final review of the `ask-consolidation` scenario after `gvrn 0.4.1` cleared the four SHOULDs from the initial 0.4.0 pass and `27a0a7e` updated the framework rule set (15 fixes + 25 new rules across `framework/rules/`). Re-walking the scope under the updated rule set: 0 MUST, 0 SHOULD, 1 low-confidence. Blocking: no.

**Scope.** `runtime/src/primitives/create_scenario.rs`, `runtime/src/primitives/append_task.rs`, helper additions in `runtime/src/primitives/mod.rs` (validators + shared `iter_numbered_headings` iterator), wiring across `parser/mod.rs`, `interpreter/mod.rs`, `mcp/server.rs`, `main.rs`, schema additions in `schema/primitives.rs`, plus `framework/runtime-tools.txt` and the CHANGELOG / Cargo.toml release metadata.

**Rule coverage walked.** `framework/rules/security-backend.md` (70 rules, post-update), `framework/rules/configuration-cross.md` (11 rules). `framework/rules/security-frontend.md` not in scope — no frontend code touched.

**Security pass.** BE-INPUT-004 (path canonicalization) is the only rule that fires structurally against the new primitives. The 0.4.1 fix wires `validate_no_traversal` against `feature_path` and `validate_slug` against `slug` before any filesystem operation. Defense-in-depth satisfied — caller-supplied path components are rejected before they reach `repo.join(...)`. The 25 new rules added in `27a0a7e` (MFA, JWT, OAuth/OIDC, CSPRNG, log injection, CSV injection, HTTP smuggling, GraphQL, LDAP, etc.) do not apply to gvrn's pure-filesystem primitives — no auth, no HTTP, no logging, no crypto in scope.

**Reuse pass.** `iter_numbered_headings` is the canonical ATX-2 numbered-heading walker; `append-task::next_task_number` is now a one-line consumer. No remaining duplicate parsing logic in the new code. Future primitives walking `tasks.md` headings have the same helper available.

**Quality pass.** Atomic writes via tempfile-in-parent + `persist` on every state-modifying operation. Conditional `[plan](plan.md)` link addressed (only emitted when `plan.md` exists at creation time). Fixture tests cover happy path + each failure mode for both primitives + the new validators + the shared iterator. 256 total tests passing.

**Efficiency pass.** O(N) line walks for `tasks.md` parsing. No unbounded loops, no repeated work. Single-file atomic writes. No concerns.

**Simplicity pass.** Helper functions appropriately scoped — `validate_slug`, `validate_no_traversal`, `iter_numbered_headings`, `title_from_slug`, `derive_tasks_heading`, `slug_from_title` each have a focused responsibility. No premature abstraction; no dead branches under the current spec.

**Lints (verifiers).** `cargo fmt --check` clean, `cargo clippy --all-targets --all-features -- -D warnings` clean, `scripts/lint-tool-coverage.sh` passes, `scripts/lint-procedure-parseability.sh` passes, `scripts/lint-frontmatter.sh` passes, `npx markdownlint-cli2` clean on the 022 spec dir + CHANGELOG.

**Release artifacts.** `gvrn-v0.4.0` and `gvrn-v0.4.1` both live on GitHub releases (5-leg matrix green for each) and crates.io.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

### QUALITY (confidence 70): `derive_tasks_heading` may produce a heading that includes the entire H1 verbatim

- **File**: `runtime/src/primitives/append_task.rs` (`derive_tasks_heading`)
- **Finding**: `derive_tasks_heading` reads the feature's `spec.md`, finds the first ATX-1 heading, and emits `# {text} Tasks`. For a spec whose H1 is `"042 — Foo Bar"`, the derived heading becomes `"# 042 — Foo Bar Tasks"` — matches the existing convention across 22 prior specs. If a spec author wrote a verbose H1 like `"042 — Foo Bar (deprecated; superseded by 043)"`, the tasks-heading inherits the parenthetical noise. Low confidence because the convention has held in practice; flagged only because a single counter-example would surface as markdownlint MD024 (duplicate-heading) if "Tasks" already appeared in the H1 by coincidence. Not blocking.

## Waived findings

*None.*

## Skipped passes

*None.*
