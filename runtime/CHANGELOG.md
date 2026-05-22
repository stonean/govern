# Changelog

All notable changes to the `govern` deterministic runtime are recorded here. The runtime ships in lockstep with the framework per [§runtime-boundary](../framework/constitution.md#runtime-boundary); release tags use the `gvrn-v<MAJOR>.<MINOR>.<PATCH>` scheme distinct from framework tags (was `runtime-v*` before v0.2.0 — see the v0.2.0 rename entry below).

## [0.7.2] — 2026-05-21

### Changed

- **`enforce-manifest` contract trimmed to slash-command manifest enforcement only.** The module docstring previously claimed the primitive replaced three bootstrap cleanup loops (slash-command manifest enforcement, legacy `skills/` directory removal, legacy workflow filename removal). Adopter cleanup of historical conventions has moved out of the primitive's contract and into the registry-driven `## Pre-run Migrations` loop introduced by spec [027 — Bootstrap Migration Registry](../specs/027-bootstrap-migration-registry/spec.md); per-entry procedures live at `framework/migrations/{id}.md` and are dispatched by the bootstrap loop, not by `enforce-manifest`. The primitive itself is unchanged — same `expected` / `pinned` inputs, same `removed` / `kept` / `pinned-kept` outputs, byte-identical behavior — but it is now the slash-command directory's enforcer only, and the docstring says so. The `govern-basic` parity fixture grows one pre-seeded `framework/skills/old-skill.md` plus a `runtime/tests/parity.rs` assertion that the file survives the bootstrap, locking the contract trim in place against regression.

## [0.7.1] — 2026-05-21

### Changed

- **Direct dependencies refreshed to latest majors.** `git2` 0.20 → 0.21, `reqwest` 0.12 → 0.13, `rmcp` 0.8 → 1.7, `sha2` 0.10 → 0.11, `zip` 5 → 8. Plus the transitive bumps cargo picked up (`digest` 0.10 → 0.11, `pulldown-cmark` 0.13.3 → 0.13.4, `tar` 0.4.45 → 0.4.46, `tower-http` 0.6.10 → 0.6.11, etc.). Hygiene-driven; no bug pushed for the bumps. The runtime had no driver to update before, but staleness compounds — clearing the backlog while the runtime is quiescent is cheaper than absorbing the migrations one CVE at a time.

  Migration touched two API surfaces. `reqwest` 0.13 renamed the rustls feature flag (`rustls-tls` → `rustls`), so `Cargo.toml` updated. `rmcp` 1.x made `ServerInfo` and `CallToolRequestParam` (renamed `CallToolRequestParams`) `#[non_exhaustive]`, so the construction sites in `src/mcp/server.rs` (`ServerInfo::new(caps).with_instructions(...)` builder) and `tests/mcp.rs` (`CallToolRequestParams::new(name).with_arguments(args)`) were rewritten through the new builder paths. One `#[allow(dead_code)]` annotation on `GovRuntimeServer::tool_router` because rustc's dead-code analysis doesn't see through the `#[tool_router]` macro — the field is required structurally even though rustc thinks it's unread.

  No behavior change visible at the protocol surface. All 325 tests pass; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean. Two parity goldens unchanged.

- **MSRV bumped from 1.85 to 1.88.** `zip` 8 requires Rust 1.88. Adopters installing on a toolchain older than 1.88 will get a clean cargo rejection rather than a confusing build error. The release matrix runs on `stable`, which is well past 1.88. Side effect: clippy 1.95's `collapsible_if` now suggests let-chains (stabilized in 1.88) for `if let X { if Y { … } }` patterns. Six call sites in `interpreter::payload`, `primitives::append_task`, `primitives::read_spec`, `primitives::mod`, and `main` were rewritten to use `if let X && Y` — REUSE only, no behavior change.

## [0.7.0] — 2026-05-20

### Added

- **`writeCode` request bundling.** The interpreter now populates the typed `WriteCodeRequest` shape end-to-end before emitting an `llm-request` envelope. Three fields previously left empty are now filled from disk: `plan-relevant-files` (entries parsed from the targeted feature's `plan.md` Affected Files table, each inlined as `{path, content}`; planned-new files absent on disk are omitted, not errored), `constitution-excerpts` (sections resolved from the running command file's `Reference: §<anchor>, …` line under Scope Boundaries, each anchor's body extracted from `framework/constitution.md`), and `task` (the targeted feature's tasks.md entry matching `task-number`, with `number`, `heading`, and `subtasks[].text` all populated). The legacy context-dump fields are appended after the typed prefix for backward compatibility with hosts that already parse them.

- **`writeSpecBody` re-run state.** When `/gov:plan` or `/gov:specify` re-runs against a partially-filled spec or plan section, the interpreter reads the existing section body from disk and emits it in the request's `existing-content` field. Empty sections elide the field. Section identification is heuristic-driven for v1 — matches `Fill the <name> section` in the step prose.

- **Secret-exfiltration guard for `plan-relevant-files`.** A new read-side guard refuses to inline files matching `.env`, `.env.*`, `*-secrets.*`, or `credentials*`, plus paths that match the repo's `.gitignore`. The first match halts the procedure with a structured `secret-exfiltration-blocked` error envelope and an unambiguous remediation hint (rename or remove the entry from `plan.md`'s Affected Files). No override flag in v1 — the plan author resolves by editing the plan.

### Changed

- **`WriteCodeRequest` field order is now cache-anchored.** Struct fields reorder to `constitution-excerpts`, `plan-relevant-files`, `write-boundary`, `task` (was `task`, `plan-relevant-files`, `write-boundary`, `constitution-excerpts`). The stable prefix — three fields that do not vary between tasks in the same `/gov:implement` walk — is contiguous and front; the per-task variable (`task`) is last. Hosts implementing prompt caching SHOULD place a cache anchor between `write-boundary` and `task` per the new contract documented in spec 022's `LLM extension points` section. A `serialize-order-lock` test in `schema::extensions::tests` enforces the new layout. Two parity goldens re-blessed (`implement-basic`, `plan-basic`) for the new field order and bundled-payload contents.

### Tests

- 12 new unit tests under `interpreter::payload::tests` cover the new readers and the secret guard (`parse_affected_files`, `parse_command_references`, `extract_anchor_body`, `extract_section_body`, `extract_section_name`, `secret_pattern` for each pattern family, gitignore matching via libgit2, `load_plan_relevant_files` happy and rejection paths, `build_write_code_request` field-order lock, `build_write_spec_body_request` existing-content inlining). The fixture `runtime/tests/fixtures/implement-basic/` grows a small `plan.md` and `framework/constitution.md` exercising the populated bundle; `runtime/tests/fixtures/plan-basic/` grows a partially-filled `plan.md` exercising the writeSpecBody re-run state.

  Origin: spec 022 scenario `writecode-payload-bundling`. Bumps the minor (additive bundling + field reorder — no host wire-format breakage thanks to backward-compat merge).

## [0.6.1] — 2026-05-19

### Fixed

- **`mark-task` ignored phased `tasks.md` files.** The primitive only matched task headings at level 2 (`## N. ...`), so phased files (`## Phase X — … / ### N. Task`, the shape `read-tasks` learned to handle in 0.5.1) returned `task '{N}' not found` for every task. Surfaced 2026-05-19 during `/gov:implement` on spec 023 task #19 — the heading happened to contain backticks (`### 19. Dedup `/configure` permission entries`), so the bug initially looked like a backtick-parser issue, but inline-code spans were never the root cause; the parser handled them correctly via `parse_atx_heading` already.

  Resolution: `mark-task` now calls `detect_tasks_structure` before splitting the file into lines and walks the appropriate task level (2 for flat, 3 for phased), exactly the way `read_tasks.rs` already does. `locate_task_range` takes a new `task_level` parameter; its terminator condition relaxes from the hardcoded `level <= 2` to `level <= task_level` so a phased task's range correctly ends at the next sibling `### N.` heading OR the next `## …` phase container, whichever comes first.

  REUSE-only: `read-tasks`'s observable behavior is unchanged; `mark-task` and `read-tasks` now consume the same `detect_tasks_structure` helper, eliminating the structure-detection drift that caused this bug. Future heading-shape edge cases fix once, propagate to both primitives.

  Four new regression tests cover the previously-broken path: `flips_subtask_in_phased_tasks_md` (basic phased success), `resolves_phased_task_with_backticks_in_heading` (the exact symptom from spec 023 task #19), `phased_task_range_terminates_at_next_phase_container` (range-termination correctness), and `phased_task_set_matches_read_tasks` (cross-primitive agreement — `read-tasks` and `mark-task` recognize the same set of tasks on a phased fixture, the contract named in the scenario's done-when). The 6 existing tests still pass; lib total 268 → 269.

  Origin: spec 022 scenario `mark-task-backtick-headings`, routed from `specs/inbox.md` via `/gov:groom`.

## [0.6.0] — 2026-05-19

### Added

- **`merge-permissions` primitive.** Idempotently merges a canonical permission allow/deny set into a JSON file (default `.claude/settings.local.json`) with exact-match dedup across `permissions.allow` and `permissions.deny`. Inputs: optional `path` (defaults to `.claude/settings.local.json`), `allow: Vec<String>`, `deny: Vec<String>`. Behavior: creates the file with `{"permissions":{"allow":[...],"deny":[...]}}` when absent (`created`); on an existing file, removes exact-match duplicates from each array (first-occurrence wins), ensures every canonical entry is present (appended at end if absent), preserves untouched top-level keys and unspecified keys under `permissions` byte-for-byte, and writes atomically via tempfile + rename. When the parsed file already equals the post-merge value structurally, emits `unchanged` and does not rewrite — preserves mtime for build-tool idempotency, matching `merge-managed-block`'s contract. Result envelope reports per-array counts of entries added vs. duplicates removed. Refuses with a `Json` parse error on malformed JSON, with a `JsonSchema` error when `permissions.allow` / `permissions.deny` exists but is not an array, or when the top-level value is not a JSON object. New `PrimitiveError::Json` and `PrimitiveError::JsonSchema` variants. 15 unit tests cover every edge case.

  Origin: spec 022 scenario `framework-list-dedup` (consumed by spec 023 `configure-dedup-permissions` to land the `/configure` dedup invariant). Registered as both the CLI subcommand `gvrn merge-permissions` and the MCP tool exposed under the bare name `merge-permissions` (Claude: `mcp__gvrn__merge-permissions`; Auggie: `mcp:gvrn:merge-permissions`). `framework/runtime-tools.txt` updated.

### Changed

- **`merge-managed-block` cross-boundary dedup (line-prefix style only).** After the existing block install/update pass, the primitive scans adopter-owned territory (everything outside the `# {marker}` preamble line and its blank-line terminator) for lines that string-equal a non-blank, non-comment line inside the canonical block. Matching adopter-area lines are removed in place — canonical-block wins. Adopter-owned blank lines and comment lines (`#` lines other than the marker itself) are preserved untouched even when their content matches a canonical line. Comparison is exact string-equality after stripping trailing `\r`; no glob expansion, no path normalization (`.claude/` and `.claude/*` are distinct). The result envelope grows two new fields on `line-prefix` invocations: `dedup-removed` (count of removed lines) and `dedup-removed-lines` (verbatim removed lines in source order). The `html-comment` style is unchanged — `dedup-removed` and `dedup-removed-lines` are `None` and elided from the JSON envelope when serialized (`skip_serializing_if = "Option::is_none"`). 10 new unit tests cover the line-prefix dedup paths; the 13 existing tests still pass.

  Motivating use case: `.gitignore` managed via `merge-managed-block` accumulated duplicates outside the `# govern` marker when adopters pasted a canonical pattern (e.g., `.claude/`) into adopter-owned territory. With cross-boundary dedup the canonical block stays the single source of those entries.

- **`check-stuck` `find_in_progress_commit` REUSE refactor.** Inline `tree.get_path(...).find_blob(...).content()` chain replaced with the existing `read_blob_from_tree` helper (introduced for `check-stuck-tasks-md-advancement` in 0.5.2). REUSE-only; observable behavior unchanged. Origin: spec 022 scenario `check-stuck-read-blob-reuse`.

- **`serde_json` `preserve_order` feature.** Enabled so user-supplied JSON key order in `.claude/settings.local.json` survives `merge-permissions` round-trips. Side effect: every JSON `Value` serialized by the runtime now preserves insertion order rather than alphabetizing keys. Three parity goldens re-blessed (`analyze-basic`, `implement-basic`, `plan-basic`) for the new key order in `llm-request` envelopes. New `BLESS=1` env-var path in `runtime/tests/parity.rs` enables future bulk re-blessing of the corpus.

### Tests

- 25 new unit tests added (15 for `merge-permissions`, 10 for `merge-managed-block` cross-boundary dedup). Total: 299 passing (`cargo test --release`); clippy clean across `--all-targets`; fmt clean.

## [0.5.2] — 2026-05-18

### Fixed

- **`check-stuck` over-reported false positives.** The primitive set `stuck = count >= threshold` based purely on the commit count of `tasks.md` since the most-recent `in-progress` transition. `/gov:implement`'s contract specifies a second condition that was not enforced: `stuck: true` should only fire when the same task is still the first incomplete one (no checkbox flipped to `- [x]` between commits in the window). Once 3+ commits landed on `tasks.md` — even when each flipped a different subtask — `stuck: true` fired on every subsequent run for the remainder of the feature, training operators to dismiss the warning.

  Resolution: the new `first_incomplete_index_unchanged` helper reads `tasks.md` at both `since-sha` and HEAD, finds the line index of the first `- [ ]` group in each (skipping fenced code blocks), and returns `true` only when both indices exist and match. `stuck` now requires `count >= threshold AND first_incomplete_index_unchanged`. Vacuous-false cases (no `tasks.md` at `since-sha`; all subtasks complete at HEAD) leave `stuck: false` — completion is the opposite of stuck.

  Subtask-identity equality is position-based for v1 (per scenario `check-stuck-tasks-md-advancement` Q1 resolution): matches how `/gov:implement` already walks tasks; reordering during implementation is rare and breaks the implicit ordering contract anyway. Heading-text equality is a future enhancement if reorder churn surfaces.

  New regression test `stuck_false_when_checkboxes_flipped_across_threshold_commits` exercises the false-positive case (4 commits, each flipping a different subtask). The five existing tests still pass — they each flip no checkboxes between commits, so the new condition holds and `stuck` correctly fires.

  No schema changes; `CheckStuckArgs` and `CheckStuckResult` JSON shapes are unchanged. Lib tests 238 → 239; full crate suite still passes.

  Reported 2026-05-17 from anvil/017-pagination (second occurrence). Inbox-routed via `/gov:groom`.

## [0.5.1] — 2026-05-17

### Fixed

Four structural bugs in `tasks.md` primitives surfaced during spec 023's living-specs work, resolved by the `runtime-primitive-structural-bugs` scenario on spec 022:

- **`append-task`'s default body line used the title as the slug** — a title like `Implement scenarios/living-specs.md` produced `scenarios/scenarios/living-specs.md.md`, doubled prefix and extension. Resolution: new explicit `slug` argument (`AppendTaskArgs.slug: Option<String>`). When `body` is omitted, `slug` is required; the primitive refuses with the new `PrimitiveError::MissingArgument` variant if both are omitted. When `body` is supplied, `slug` is ignored. The buggy heuristic that derived the slug from the title is removed entirely.
- **`append-task` numbering hardcoded to `## N.` top-level** — on phased `tasks.md` files (`## Phase A — … / ### N. Task` shape), the primitive found no `## N.` matches and fell back to `## 1.` at the file's bottom, colliding with the existing `### 1.` task and breaking the file's H3 convention. Resolution: new `TasksStructure` enum (`Flat` / `Phased`) detected by presence of any `### N.` heading. New `AppendTaskArgs.parent_heading: Option<String>` lets the caller name the phase to extend; refuses with the new `PrimitiveError::ParentHeadingNotFound` variant when the supplied heading does not match. When `parent_heading` is omitted, the primitive extends an existing `Phase X — Follow-on scenarios` phase if one is present, otherwise creates `Phase {next-letter} — Follow-on scenarios` with the auto-computed next letter. Phase containers explicitly exclude `## N.` numeric headings, so mixed-structure files keep their phase set clean.
- **`read-tasks` returned empty on phased files** — the parser only matched `## N.` level-2 headings and blinded `/gov:implement` on every phased spec. Resolution: structure-aware task-level dispatch — phased files walk `### N.` at level 3, flat files keep walking `## N.` at level 2. New `Task.phase: Option<String>` carries the heading text of the containing phase for phased tasks; the field is absent from JSON output for flat tasks (`skip_serializing_if = "Option::is_none"`) so pre-existing consumers parse unchanged. Mixed-structure files walk only the phased layer per the scenario's edge case.
- **`check-stuck` reopen regression coverage** — investigation showed the topological-reverse revwalk already tracked the most-recent `in-progress` transition correctly (the bug had been resolved in earlier 022 work without closing the scenario task). Added three regression tests under `primitives::check_stuck::tests` to lock the correct behavior in place: `reopen_measures_from_most_recent_in_progress_transition`, `first_in_progress_works_when_never_reopened`, and `mechanical_sweeps_do_not_disturb_since_sha`.

### Changed

- New shared helpers in `primitives::mod`: `TasksStructure`, `detect_tasks_structure`, `iter_task_numbers_at_levels`, `iter_phase_ranges`, `PhaseRange`. Used by both `append-task` (Phase 2) and `read-tasks` (Phase 3) to keep phased-structure detection in one place. The deprecated single-purpose `iter_numbered_headings` wrapper is removed; callers in tests now invoke `iter_task_numbers_at_levels(content, &[2])` directly.

### Tests

- 26 new unit tests across `append_task`, `read_tasks`, and `check_stuck` covering the four bug fixes and their edge cases. Total lib tests: 235 → 238; full crate suite: 269 passing.

## [0.5.0] — 2026-05-17

### Changed (breaking)

- **MCP wire format**: tool names no longer carry the `gov-rt:` prefix. The 23 tools are now registered as bare `<verb>-<noun>` strings (`read-spec`, `read-tasks`, `mark-task`, …) — the same names already used by the `gvrn <subcommand>` CLI surface, so the binary's two surfaces finally agree on identifiers. Server-level namespacing is supplied by the adopter's `.mcp.json` server registration. The canonical server name is **`gvrn`** (was conceptually `gov-rt`), aligning the MCP server name with the binary/crate name. Resulting per-host wire identifiers:
  - Claude Code: `mcp__gvrn__<verb>-<noun>`
  - Auggie: `mcp:gvrn:<verb>-<noun>`

  **Adopter impact**: adopters who previously registered the runtime under the name `gov-rt` in `.mcp.json` must rename it to `gvrn`. Adopters who hand-authored permissions entries referencing `mcp__gov-rt__<tool>` or `mcp:gov-rt:<tool>` must update those entries to use `gvrn`. `framework/bootstrap/configure/{claude,auggie}.md` and the generated `.claude/commands/gov/configure.md` carry the new identifiers; re-running `/gov:configure` after a framework update is sufficient to refresh permission lists. No CLI-level changes — `gvrn <subcommand>` invocations are unchanged.

  **Why now**: the `gov-rt:` namespace was chosen in spec 022 to disambiguate tool names from `/gov:` slash commands at a time when the tool name itself carried the prefix (and a colon, which is not a valid identifier character in Claude Code MCP tool names). Switching to bare names removed the colon; the remaining `gov-rt` token then existed only at the server-name boundary, where it duplicated the `gvrn` binary/crate identity without adding meaning.

### Changed

- `scripts/gen-configure-mcp.sh`: trap-based tempfile cleanup so any early-exit path (set -e, splice failure, signal) releases the per-host block tempfiles instead of leaking them into `$TMPDIR`. Unused `label` parameter dropped from `process()`. SHOULD-tier findings from `/gov:review --fix`.
- `scripts/lint-tool-coverage.sh`: tool references inside a command file's `## Markdown-only reference` section are now skipped — that section *is* the fallback path, so references there do not require a paired fallback marker. Whitespace-strip on manifest lines tightened from "one leading/trailing space" to "any run of `[[:space:]]`". `|| true` added to the section-header lookup so `set -euo pipefail` does not abort when a command file has no markdown-only-reference section. SHOULD-tier findings from `/gov:review --fix`.

## [0.4.1] — 2026-05-16

### Changed

- `create-scenario` and `append-task` now validate caller-supplied path components before any filesystem operation, addressing the four SHOULD findings from `/gov:review` against scenario `022.ask-consolidation`:
  - **BE-INPUT-004 defense-in-depth** — new `validate_slug` and `validate_no_traversal` helpers in `primitives/mod.rs` reject slugs containing path separators or leading dots and reject `feature_path` values that are absolute or contain `..` components. New `PrimitiveError::InvalidSlug { slug, reason }` and `PrimitiveError::InvalidPath { path, reason }` variants surface the rejections as clean operational errors. Defense-in-depth: the existing `is_dir` checks remain, but the new validators close the rule's letter (canonical-path + base-dir check) as well as its spirit.
  - **REUSE** — new shared `iter_numbered_headings(content)` helper in `primitives/mod.rs` yields ATX-2 numbered headings while skipping fenced code blocks. `append-task`'s `next_task_number` is now a one-line `iter_numbered_headings(content).max().unwrap_or(0) + 1`, dropping ~30 lines of duplicate parsing. Available to future primitives that walk `tasks.md` headings.
  - **QUALITY** — `append-task`'s newly-created `tasks.md` now emits `Tasks. Complete in order.` (unlinked) when no `plan.md` exists at the time of creation, and the original `Tasks derived from the [plan](plan.md). Complete in order.` (linked) when `plan.md` is present. Closes the dangling-link case that markdownlint MD051 would flag.
- 19 new unit tests cover the validators, the shared heading-iterator helper, and the conditional intro behavior. Total lib tests grow 203 → 222; full suite 256 passing.

## [0.4.0] — 2026-05-16

### Added

- Two primitives for the `/ask` scenario branch introduced in spec 023, landing via scenario `022.ask-consolidation`:
  - `create-scenario` — write a `scenarios/{slug}.md` file under a feature with `section` frontmatter and Context / Behavior / (optional) Edge Cases body sections. Atomic via tempfile-in-parent + `persist` rename. Creates the `scenarios/` subdirectory if absent. Refuses with `ScenarioConflict` when the destination already exists; refuses with `FeaturePathNotFound` when the feature directory is missing.
  - `append-task` — append a numbered `## N. <title>` block to a feature's `tasks.md`, computing the next number as `max(existing) + 1` so a tasks file with gaps doesn't overwrite existing entries. Creates `tasks.md` with a heading derived from the feature's spec H1 (or a minimal `# Tasks` fallback when the spec is unreadable). Atomic via tempfile-in-parent + `persist` rename. Skips numeric headings inside fenced code blocks.
- New MCP tool names: `gov-rt:create-scenario`, `gov-rt:append-task`. Tool list grows from 21 to 23 entries; both `framework/runtime-tools.txt` and `mcp::server::TOOL_NAMES` carry them.
- New CLI subcommands: `gvrn create-scenario` and `gvrn append-task` (clap-derive args; same JSON envelope on stdout as other write primitives).
- New `PrimitiveError` variants: `ScenarioConflict { path }` and `FeaturePathNotFound { path }`.

## [0.3.1] — 2026-05-12

### Changed

- `enforce-manifest`'s glob compiler now delegates per-character escaping to `regex::escape` (already a transitive dependency via `regex`) instead of maintaining a hand-written metacharacter table. Internal refactor only; the glob → regex translation is byte-for-byte identical, all 14 `enforce_manifest::tests` still pass unchanged (including the metacharacter and bracket-literal coverage). Surfaced by `/gov:review`'s simplicity pass against 022-deterministic-runtime scenario `apply-manifest`.

## [0.3.0] — 2026-05-12

### Added

- Three primitives for strategy-aware bulk install + cleanup (scenario `022.apply-manifest`):
  - `apply-manifest` — strategy-aware bulk substitute + write driven by a typed manifest. Each `ManifestEntry { source, dest, strategy, keep-literals }` records an outcome (`created` / `updated` / `unchanged` / `skipped-exists` / `skipped-pinned` / `source-missing`); aggregate counts roll up across entries. Strategies: `update` (substitute, write only on diff, preserve mtime when unchanged), `create` (substitute, write only when dest absent), `skip-if-conflict` (write verbatim without substitution, only when dest absent). `pinned` short-circuits before any read/write — adopter customizations are never touched. `keep-literals` per entry masks named substitution keys, used by the `govern.md` self-install to keep `{project}` / `{cli-config-dir}` literal for the next adopter's bootstrap.
  - `enforce-manifest` — directory cleanup against an expected file list. Removes files matching `glob-include` (default `*.md`) whose relative path is neither expected nor pinned. `recursive: false` (default) is top-level only; `recursive: true` descends. One call replaces the bootstrap's three legacy cleanup loops (slash-command manifest enforcement, legacy `skills/` removal, legacy workflow filename removal).
  - `merge-managed-block` — generalization of `merge-claude-md` to support configurable marker shapes. `marker-style: "html-comment"` (default) reproduces `merge-claude-md`'s exact behavior; `marker-style: "line-prefix"` uses a `# {marker}` preamble line followed by the block, terminated by a blank line or EOF — matching `.gitignore` and `.gitattributes` conventions.
- New MCP tool names: `gov-rt:apply-manifest`, `gov-rt:enforce-manifest`, `gov-rt:merge-managed-block`. Tool list grows from 14 to 17 entries; both `framework/runtime-tools.txt` and `mcp::server::TOOL_NAMES` carry them, and the MCP integration test exercises every new tool.
- `framework/bootstrap/govern.md` Instructions section rewritten to drive the bootstrap through six primitive calls (`fetch-archive` → `extract-archive` → `apply-manifest` → `merge-managed-block` for `.gitignore` → `enforce-manifest` → `apply-manifest` with `keep-literals` for the `govern.md` self-install) plus a gate-confirm and two prose steps. No host-generated bash walker required; no `python3 -c '...'` substitution fallback; no per-file Edit calls from the host.
- `govern-basic` parity fixture extended to exercise every new strategy + marker style + cleanup path end-to-end. New companion test `govern_basic_post_run_filesystem_state_matches_expectations` walks the post-run target tree and asserts the per-primitive on-disk effects (substitutions applied where expected and NOT where suppressed, pinned file preserved verbatim, keep-literals placeholders kept literal, line-prefix `.gitignore` created, legacy file pruned).

### Changed

- `merge-claude-md` is now a thin compat shim that delegates to `merge-managed-block` with `marker-style: "html-comment"` and `marker: "govern-managed"`. The primitive's public API (`MergeClaudeMdArgs`, `MergeClaudeMdResult`) is unchanged, so existing callers — the bootstrap fixture, parity goldens, and any host scripts — keep working byte-for-byte. Slated for removal in the next major release of `gvrn`.

## [0.2.1] — 2026-05-12

### Changed

- **BREAKING** — `fetch-archive` argument `sha256_url` becomes `Option<String>`. Callers that omit the field download without sidecar verification; the primitive returns the computed sha256 digest and `verified: false` so the host can compare against a known-good value out-of-band. Callers that supply the field keep the verified path verbatim. The result struct grows a `verified: bool` field that reports which path the call took. Motivation: the `/govern` bootstrap operates live-on-main and fetches GitHub's auto-generated source tarballs (`/archive/refs/heads/main.tar.gz`), which ship without sidecars; before 0.2.1 the runtime couldn't drive that fetch and `/govern` always fell back to the markdown-only path.

### Updated

- `framework/bootstrap/govern.md`: step 2 prose acknowledges the sidecar-optional behavior and documents what `verified: false` means for callers that care about integrity.

## [0.2.0] — 2026-05-12

### Added

- Four primitives for the bootstrap procedure (scenario `022.govern-bootstrap`):
  - `fetch-archive` — download an archive plus its sha256 sidecar via reqwest's blocking client and verify the hash before persisting. Adds `reqwest` (blocking, rustls-tls) and `sha2` deps; a 256 MiB per-fetch ceiling caps memory defensively.
  - `extract-archive` — extract `.tar.gz`/`.tgz`/`.zip` in-process (no shell-out) via `flate2` + `tar` and the `zip` crate. Path-traversal protection rejects absolute paths and `..` components before writing.
  - `substitute-templates` — walk a source tree, apply `{key}` → value replacements to text files, copy binary files unchanged, write to a destination tree. Args use `source-dir` / `target-dir` (distinct from extract-archive's `dest` so both primitives can share a single procedure context).
  - `merge-claude-md` — idempotent BEGIN/END marker insert/update for a framework-managed block in CLAUDE.md. Four actions: created / inserted / updated / unchanged; unchanged preserves mtime.
- `gvrn exec` command resolution now considers `framework/bootstrap/<name>.md` as a third candidate after the existing two paths, so the `/govern` bootstrap procedure runs through the runtime.
- `framework/bootstrap/govern.md` gains a parseable `## Instructions` section that exercises the four new primitives plus a gate-confirm for the install approval; the existing 788-line procedure stays in place as the markdown-only reference.

### Changed

- **BREAKING** — package, binary, and library all renamed `runtime` / `govern_runtime` / `govern-runtime` → `gvrn`. Release tag pattern becomes `gvrn-v*` (was `runtime-v*`); release artifacts become `gvrn-<TARGET>.tar.gz` (was `runtime-<TARGET>.tar.gz`).
- **BREAKING** — `substitute-templates` argument names `source` / `dest` → `source-dir` / `target-dir` to avoid clashing with `extract-archive`'s `dest` in shared procedure context.

## [0.1.0] — 2026-05-12

### Added

- Crate scaffold under `runtime/`: `Cargo.toml`, binary entrypoint at `src/main.rs`, library root at `src/lib.rs`, module placeholders for `parser`, `interpreter`, `primitives`, `mcp`, `schema`, and `io`.
- Lint configuration in `Cargo.toml`: `unsafe_code = "forbid"`, `missing_docs = "warn"`, clippy `all` + `pedantic` at warn, plus `unwrap_used` and `expect_used` at warn.
- Pre-commit verification (`.githooks/pre-commit`): when staged changes touch `runtime/`, the hook runs `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`. Set `GOVERN_SKIP_RUNTIME_CHECKS=1` to bypass for a single commit.
- `runtime/bacon.toml` — `bacon` job definitions (`check`, `clippy`, `test`, `fmt`) with `clippy` as the default. Install with `cargo install --locked bacon`.
