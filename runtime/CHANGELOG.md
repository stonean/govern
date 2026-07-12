# Changelog

All notable changes to the `govern` deterministic runtime are recorded here. The runtime ships in lockstep with the framework per [§runtime-boundary](../framework/constitution.md#runtime-boundary); release tags use the `gvrn-v<MAJOR>.<MINOR>.<PATCH>` scheme distinct from framework tags (was `runtime-v*` before v0.2.0 — see the v0.2.0 rename entry below).

## [0.19.0] — 2026-07-12

Follow-up review of the 0.18.0 runtime (see `specs/022-deterministic-runtime/review.md`). Closes a partially-resolved SSRF finding, a bootstrap parse regression the 0.18.0 parser change introduced, and a set of newly-surfaced input-validation and correctness gaps — plus the full follow-on scenario set the review captured (tasks 64–68 below): six new coverage primitives, the dashboard's rendered pipeline view, the derived writeCode boundary, two retired primitives, and the documented clarify exec scope. One review item is deliberately excluded: MCP unknown-field strictness (022 task 65) rides the rmcp 1.x → 2.x port tracked in `specs/inbox.md` and ships as the next minor, because its strict-params wrapper builds on exactly the rmcp layer that migration reworks.

### Follow-on scenarios (022 tasks 64–68)

- **Parser nested-list continuation (task 64)** — a recognized primitive named in continuation text after a nested ordered list closes (where the parent step was already finalized) now raises `ParseError::Invalid` naming the primitive, instead of silently dropping the dispatch on the exec path. Guarded on being inside the step list, so a primitive named in the Instructions preamble stays a legitimate reference. `scripts/lint-procedure-parseability.sh` catches the class across `framework/commands/*.md` + `framework/bootstrap/*.md`.
- **`remove-inbox-item` primitive (task 67, in progress)** — the complement of `append-inbox`: removes the first `{specs-root}/inbox.md` bullet whose text matches `item` (shared bullet grammar, atomic write, double-blank seam collapsed), reporting the remaining count. A no-match or missing inbox is a clean `removed: false` outcome. `/gov:groom` step 8 now invokes it instead of a host `Edit`, closing the review's highest-value coverage gap (groom's per-item hot loop).
- **`create-plan-artifacts` primitive (task 67, in progress)** — the plan-side mirror of `create-feature`: copies the plan/tasks (and, with `include-data-model`, data-model) templates into an existing feature directory, atomic and mode-preserving, resolving every needed template before the first write. Pre-existing artifacts are never touched by default — each reports `kept`, feeding `/gov:plan`'s keep-or-replace prompt; only `overwrite: true` (the confirmed replace branch) copies over them. `/gov:plan` gains step 3 invoking it, closing the review's template-copy / existing-artifact-detection gap. The template resolver is now a shared `primitives::resolve_template` helper (lifted from `create-feature`); the `plan-basic` parity fixture gains templates and a re-blessed golden (step 3 changes the stream).
- **`check-review-gate` primitive (task 67, in progress)** — evaluates `/gov:implement`'s pre-done review gate in one call, first failure wins: the feature directory's markdown lint (recursive glob through the `lint-markdown` machinery, absorbing the step's raw `npx markdownlint-cli2` invocation), then the spec `review:` block (`not-reviewed` on an absent block or null `last-run`; `must-violations` on `blocking: true`). Returns the verdict plus the canonical blocked message — `/{project}:review` references resolved through the `[host] project` namespace — and the resolve-or-waive guidance. Implement step 13 invokes it, replacing the branch the host re-walked by hand on every completion attempt; a blocked gate is a domain outcome, never an error.
- **`append-question` primitive (task 67, in progress)** — `/gov:amend`'s question-route write, previously primitive-less (asymmetric with the scenario route's `create-scenario` + `append-task`): appends `- {question}` to the target's `## Open Questions` (the spec, or `scenarios/{slug}.md` via `scenario`), creating a missing section per template order and replacing `*None …*` scaffold placeholders. Dedup reuses `read-spec`'s question parser with amend's normalized-whitespace comparison (a match is a clean `appended: false` outcome reporting `duplicate-of`); on a non-`draft` spec target the status reverts to `draft` in the same atomic write — the back-edge that keeps "questions resolved" status claims honest. Amend's question route now invokes it.
- **`diff-cross-spec` primitive (task 67, in progress)** — the cross-spec impact filter `/gov:implement` steps 7 and 12 re-derived by hand per task (step 12 self-declared "no primitive owns this filter yet"): diffs the feature's first spec-dir commit (shared `derive-boundary` walk) against the working tree, scoped to the spec root — sibling-spec paths outside the feature's own dir, plus the inbox's added bullet lines (shared bullet grammar) as the captured issues. Working-tree diffing means the per-task summary sees the run's uncommitted captures; a clean tree equals the documented `first-commit..HEAD -- specs/` form. Implement steps 7/12 now invoke it; review's captured-issues stays on `compute-review-scope` (in-progress window). Resolves the scenario's own-primitive-vs-derive-boundary-mode fork: separate primitive, shared walk helper.
- **`dashboard` rendered-markdown field (task 67, completes the scenario)** — the dashboard result gains `rendered-markdown`: the full `/gov:status` pipeline view pre-rendered as one markdown fragment (preamble, table, counts/callouts, cross-service references readout), absorbing the five LLM-side rendering steps the coverage review counted. The runtime resolves each spec's `references:` index internally for the readout (same classification as `resolve-references`, service `description` appended) and substitutes the `[host] project` namespace into `/{project}:…` texts. Returned data the host may restyle, never stdout printing; the structured payload stays authoritative. `/gov:status` collapses to invoke-dashboard → emit-rendered → target prompt, with the piece-by-piece formats moved to its Rendering reference as the markdown-only path.
- **`substitute-templates` and `merge-claude-md` retired (task 68)** — both were exposed, tested, and permission-listed with no command step invoking either: `substitute-templates`' tree-copy was subsumed by `apply-manifest` (which now owns the shared `apply_substitutions` helper), and `merge-claude-md` was already a compat shim over `merge-managed-block` slated for removal. Six-site removal in reverse: registry (hence `TOOL_NAMES` / `PRIMITIVE_NAMES`), interpreter dispatch, CLI subcommands, MCP `#[tool]` methods, schema Args/Results, `runtime-tools.txt`, and the regenerated configure permission blocks; the exec bootstrap-chain test re-targets `extract-archive` → `apply-manifest` → `merge-managed-block`. Removal lands inside the unreleased 0.19.0, so no released MCP surface breaks.
- **Clarify exec-path scope documented (task 68, completes the scenario)** — `gvrn exec clarify` no-ops steps 7–8 (edge-case enumeration, criterion verification) by design: they cannot fold into `askClarifyQuestion`'s one-question-per-round-trip ABI because they are spec-wide passes that run even on the zero-questions short-circuit. The reduction is now stated in the command's Instructions preamble and the data-model's exec-path note instead of standing as a silent gap; the markdown-only path keeps performing both steps in full.
- **writeCode boundary derivation (task 66)** — `derive-boundary` now emits **directory-zone globs** (`runtime/src/main.rs` → `runtime/src/**`; root-level files stay exact — their zone would be `**`), because the writeCode validator must admit *new* files, which never exact-match a previously-changed path. The walker merges the result into the `write-boundary` enforcement key as a **union** with any session seed (a deliberate grant is never revoked; on a fresh feature the seed admits the first out-of-spec edit; with neither, enforcement stays fail-closed). Previously nothing populated the key — enforcement silently depended on a pre-seeded boundary. The `implement-basic` fixture gains a two-commit fixed-time history and drops its seeded `write-boundary`, so the re-blessed golden's enforcement runs on the derivation alone. Resolves the scenario's boundary-format fork against changing `path_in_boundary` semantics (an exact-path entry that silently grants its directory would make boundary entries lie).

### Security

- **Redirect SSRF (BE-INPUT-007 completion)** — `fetch-archive` now re-runs the full scheme/internal-range screen on **every** redirect hop via a custom `reqwest` redirect policy (capped at 10 hops). Previously the default client followed up to 10 redirects with no re-validation, so a single `302` to `http://169.254.169.254/…` defeated both the https-only rule and the internal-range denial.
- **Path traversal via `feature` (BE-INPUT-001 sibling)** — `set-status`, `mark-task`, `mark-criterion`, `prune-tasks`, `write-review`, and the read-only feature primitives (`read-spec`, `read-tasks`, `check-artifacts`, `check-stuck`, `compute-review-scope`, `traverse-deps`, `derive-boundary`, `process-waivers`, `resolve-references`) now validate the MCP-supplied `feature` argument with `validate_no_traversal`, closing an out-of-repo write/read escape (`feature: "../../other/specs/001-x"`).
- **`write-review` frontmatter injection** — `reviewed-at`, `reviewed-against`, `diff-base`, `feature`, `scenario`, and `skipped-passes` are single-line-validated before being spliced into `review.md` and the spec's `review:` frontmatter block, so a newline can no longer inject a spoofed top-level key (e.g. `status: done`) into the spec.
- **`run-generator` / `lint-markdown` containment** — `run-generator` bounds its `script` argument to the repo (`validate_no_traversal`), and `lint-markdown` rejects a `paths` entry beginning with `-` (which markdownlint-cli2 would parse as a flag such as `--config`, loading arbitrary `customRules` JS).

### Fixed

- **`gvrn exec govern` parse regression** — `framework/bootstrap/govern.md` step 6 (which named two primitives, `merge-managed-block` + `write-session`) is split into two steps, so the bootstrap procedure parses again under the 0.18.0 two-primitive hard-error. `scripts/lint-procedure-parseability.sh` now also parses `framework/bootstrap/*.md`, so this class can no longer slip through CI.
- **`gvrn exec target <feature>` retarget** — a `resolve-feature` `resolved` result now overrides the session-seeded `feature`/`path` on the `target` command, so switching the target actually writes the new feature instead of rewriting the stale one with a fresh timestamp.
- **Numbered-heading grammar** — `mark-task` and `read-tasks` now require the trailing `.` on a task heading (`## N.`), matching `append-task`/`prune-tasks`, so a prose heading like `## 3 quick wins` is no longer treated as task 3.
- **`writeCode` edit contract** — `validate_response` now rejects a `create` edit with no `content` and an `edit` edit with neither `patch` nor `content` (`ValidationError::EditContent`), instead of admitting an undefined edit the schema alone allowed.
- **`discover-rule-files` surfaces** — the MCP-boundary `detected-surfaces` argument is validated like a `[rules] surfaces` config value; an unrecognized member (e.g. `"Backend"`) fails fast instead of silently loading only `-cross.md` rules.
- **Exec operational-error envelopes** — the command-not-found, unreadable-file, and walker-I/O exit paths now emit a terminal `error` protocol message on stdout (with the runtime version), honoring the 1–127 clean-band contract that lets a host distinguish a clean operational error from a signal-killed crash.
- **Parser robustness** — a heading that emits no text (empty, or a code-span only like `` ## `gvrn` ``) no longer opens the Instructions section over the real one; an extension marker on its own line between steps is attached to the next step rather than dropped; and a marker merely quoted in a code span is no longer mistaken for a live seam.
- **CRLF idempotency** — `merge-managed-block`'s `html-comment` style normalizes `\r` before its unchanged-compare, so `merge-claude-md` no longer rewrites `CLAUDE.md` on every run on a CRLF checkout.
- **`merge-managed-block` marker** — the `marker` argument is rejected when it contains a newline or `-->`, which would corrupt the managed-region delimiters.

### Changed

- **`write_atomic` mode preservation (Unix)** — an in-place rewrite now re-applies the destination file's prior permissions after the tempfile rename, so `set-status` / `mark-task` / `write-review` / etc. no longer narrow an existing `0644` file to `0600`.
- Four no-op `matches!(…)` statement tests are wrapped in `assert!(…)`; stale doc comments (`dashboard` `session-path`, `prune-tasks` `size-after`, the `ParseDiagnostics` data-model note) are corrected.

### Not fixed (documented)

- The `fetch-archive` DNS-rebinding TOCTOU (guard resolves, reqwest re-resolves) remains as logged in `specs/inbox.md`; the redirect fix above is orthogonal to it.
- On the `/gov:implement` exec path, `derive-boundary`'s computed boundary does not auto-populate the `write-boundary` key the writeCode validator enforces on (that key is a seeded input); enforcement is fail-closed without a seed. Auto-binding it is a design decision (seed-vs-derived precedence) that also needs a multi-commit `implement-basic` fixture, so it is deferred rather than changed here.
- `deny_unknown_fields` was not added to the primitive `Args` structs: the exec interpreter binds every primitive's args from a clone of the *entire* walker context (a deliberate superset), so rejecting unknown fields would break all primitives on the exec path. A misspelled kebab-case field still silently defaults; closing this needs a per-primitive field allowlist, not a blanket attribute.

## [0.18.0] — 2026-07-11

Remediation of the nine MUST and twenty-one SHOULD findings from the 0.17.0 `/gov:review` (see `specs/022-deterministic-runtime/review.md`).

### Security

- **writeCode boundary (BE-INPUT-004)** — the edit-path segment screen now splits on both `/` and `\`, closing a Windows-only traversal escape (`runtime/a\..\..\x` no longer satisfies `runtime/**`).
- **Archive extraction (BE-INPUT-006)** — `extract-archive` now bounds cumulative decompressed bytes (`MAX_EXTRACT_BYTES` = 2 GiB) and entry count (`MAX_EXTRACT_ENTRIES` = 100000), erroring with the cap named before writing past it, closing a decompression-bomb DoS through the unverified bootstrap path.
- **Outbound fetch (BE-INPUT-007)** — `fetch-archive` enforces an https-only scheme allowlist and denies hosts resolving to loopback / link-local / RFC-1918 / unique-local / cloud-metadata ranges (SSRF). An opt-in `GVRN_FETCH_ALLOW_INSECURE_HOSTS` allowlist (empty by default) exempts named hosts for internal mirrors and local testing.
- **Slug validation (BE-INPUT-001/002)** — `validate_slug` is now an allowlist (`^[a-z0-9]+(?:-[a-z0-9]+)*$`), rejecting newlines and control characters that previously reached written filenames and headings; `append-task` validates its `slug` before interpolating it into `tasks.md`.

### Fixed

- **Command exec-path correctness** — `/gov:clarify` step 2 no longer collapses `read-spec` and the recovery-branch `set-status` into one step (the parser bound only the last), `/gov:clarify`'s draft→clarified flip is gated by a dedicated `gate-confirm` step, and `/gov:groom`'s scenario route no longer drops `create-scenario`. Root cause fixed structurally: the parser now raises `ParseError::Invalid` when one numbered step names two distinct primitives, so this class fails CI instead of silently dropping a dispatch.
- **`gvrn exec specify`** binds the newly-created feature into the session write (a `create-feature` `created: true` result overrides the stale session-seeded `feature`/`path`), instead of rewriting the session to the old target.
- **`verifyCriteria`** verdicts now gate `mark-criterion` on the exec path — a criterion the response does not affirm `met: true` is left unchecked.
- **`read-spec`** folds wrapped acceptance-criterion continuation lines into the criterion text (no more mid-sentence truncation reaching `verifyCriteria`), preserving the checkbox-index contract.

### Performance

- `check-stuck` memoizes status parses by the spec's blob OID (one parse per distinct spec version, not per commit); the git-walk MCP tools (`check-stuck`, `compute-review-scope`, `derive-boundary`) run under `spawn_blocking`; `is-gitignored` opens the repo once per plan-file loop; `first-rule-with-verification` is a single pass.

### Internal

- Consolidation (behavior-preserving): shared `list_feature_dirs`, `feature_number`, `read_scenario_section`, `list_scenario_files`, `template_candidates`, and `frontmatter_status` helpers replace ~nine hand-rolled copies across the new primitives and payload builders; `UNBLOCKING_STATUSES`/`PLANNED_OR_LATER` derive from `schema::status`; `append-inbox` and `check-artifacts` reuse the shared checkbox grammar and case-insensitive scenario walk (closing two latent divergences); the seven extension-request builders share `typed_with_legacy_context`/`typed_only`. Command-prose restatements now point at the markdown-only reference rather than duplicating it.

### Notes

- `cargo test` (760), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `scripts/audit/run-all.sh`, and the ubuntu/macos/windows CI matrix are clean. Ships in lockstep with the framework per [§runtime-boundary](../framework/constitution.md#runtime-boundary).

## [0.17.0] — 2026-07-11

The backlog burn-down release: every SHOULD-tier and coverage finding from the 2026-07-11 full-runtime review, implemented as nine scenarios on spec 022 (`primitive-robustness-hardening`, `archive-network-hardening`, `parser-walker-conventions`, `extension-request-hygiene`, `implement-completion-gate`, `clarify-command-acceleration`, `groom-command-acceleration`, `scaffolding-primitives`, `analyze-artifact-checks`) plus the CI/README/consolidation chores. The minor bump is driven by the four new primitives and the parser/walker convention changes below.

### Added

- **`resolve-feature`** — resolves a feature by exact directory name, bare or zero-padded number, or unique partial slug under the configured specs root; reports ambiguity/not-found as domain outcomes and optionally a scenario file's existence + `section`. Adopted by `/gov:target`.
- **`create-feature`** — next-NNN computation, kebab slug derivation, directory creation, and mode-mirrored template copy (installed template wins over the framework source); refuses existing directories. Adopted by `/gov:specify`.
- **`append-inbox`** — atomic bullet append to `specs/inbox.md` with create-from-template and optional dedup-by-prefix. Adopted by `/gov:log` and named by `/gov:implement`'s auto-capture rule.
- **`check-artifacts`** — the residual deterministic `/gov:analyze` families: artifact completeness per status, task numbering/done-when consistency, scenario→task mapping (prune-aware per constitution §tasks-phase), and review-state drift, with severities mirroring the command reference. Adopted by `/gov:analyze` step 8.
- **`write-session` clear mode** — removes the session target while preserving the per-contributor `cli-config-dir`; `/gov:target --clear` and `/gov:specify` now route through the primitive (the stale "no session-shaped primitive" claim is gone).
- **`verifyCriteria` extension point** — typed request/response for `/gov:implement`'s per-criterion verification; the completion gate is now numbered parseable steps (`read-tasks` tally, `read-spec`, per-criterion `mark-criterion`, review-gate reads, `gate-confirm`, `set-status`), giving `mark-criterion` its first prose consumer.
- **Typed `askClarifyQuestion` / `routeInboxItem` builders** — the two long-deferred extension points now have schema types, data-model entries, and typed request construction; unknown extension identifiers are an error, never a context dump.
- `/gov:clarify`, `/gov:groom`, and `/gov:log` are rewritten to the parseable conventions and leave `legacy-prose-commands.txt` (now just `amend.md` and `help.md`); `/gov:plan` and `/gov:specify` approval gates use `gate-confirm`; raw `gen-spec-deps.sh` references route through `run-generator`.
- The tool list grows from 33 to **37**; claude/auggie configure permission blocks regenerated.

### Changed

- **Parser step numbering** honors the document: ordered-list `start` seeds the counter, lists separated only by HTML comments continue the sequence, and nested bullets no longer become phantom steps — `progress.step` and gate names sent to hosts now match the literal step numbers in every command file.
- **Gate semantics**: a step invoking `gate-confirm` blocks by virtue of the primitive, phrase or no phrase; the "ask the user to approve" prose trigger applies only to prose steps; a primitive step never silently drops its dispatch.
- **Span heuristic**: only backticked spans in invoking position (or within edit distance 2 of a primitive name) fail parseability — ordinary kebab-case vocabulary parses; `prune.md` and `govern.md` now parse cleanly. `gvrn parse --check` distinguishes legacy-prose (rc 2) from invalid (rc 1), and the parseability lint rejects invalid even for allowlisted files.
- **Extension-request hygiene**: walker accumulator keys (`llm:*`, `findings`) are filtered from every request payload; `writeSpecBody` populates `template-path`/`template-content`/`section`/`feature-description` with command-aware existing-section reads.
- **MCP seam**: `fetch-archive`, `extract-archive`, `run-generator`, and `lint-markdown` run under `spawn_blocking` (no more blocking-client panic in debug builds or pinned tokio workers).
- Path validation is uniform: `enforce-manifest` requires the target directory inside the repo, `apply-manifest` validates every manifest entry, `merge-managed-block`/`merge-permissions` require repo-relative paths. On Windows, `validate_no_traversal` also rejects rooted (`\foo`), drive-relative (`C:foo`), and UNC-prefixed paths that `is_absolute()` alone missed — surfaced by the first windows-latest CI run.
- `check-stuck` tolerates branchy history (parent-blob transition detection, reachability-based counting, CRLF-aware frontmatter); `write-review` computes both outputs before writing either; `substitute-templates` mirrors source file modes; `create-scenario` YAML-escapes `section`; `append-task` rejects embedded newlines; `dashboard` degrades to a detail-less target on malformed scenario frontmatter; `check-rule-ids` scopes deprecation to the rule's own section; `set-status` validates lifecycle membership; `fetch-archive` errors on over-cap bodies instead of truncating; `extract-archive` skips zip symlinks and masks modes to `0o777`.

### Internal

- Consolidation (behavior-preserving, goldens byte-exact): one shared `resolve_path` (nine private copies deleted), one checkbox grammar for the read and mark sides of the criterion/task index contract, a canonical lifecycle-status constant in `schema/status.rs` with the compatibility subset derived from it, one shared frontmatter-status reader, and a single `PRIMITIVE_REGISTRY` from which the parser's and MCP server's name lists are both defined — with tests pinning the interpreter dispatch and `runtime-tools.txt` to set-equality against it.

### Notes

- CI now tests on ubuntu/macos/windows with `--locked` everywhere and a pinned toolchain; a repo-wide `.gitattributes` pins LF. Ships in lockstep with the framework per [§runtime-boundary](../framework/constitution.md#runtime-boundary). `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `scripts/audit/run-all.sh` are clean.

## [0.16.1] — 2026-07-11

Bug-fix release for the eleven MUST-level defects surfaced by the 2026-07-11 full-runtime accuracy review, recorded as six scenarios on spec 022 (`spec-side-parser-hardening`, `host-protocol-conformance`, `write-boundary-path-normalization`, `merge-managed-block-trailing-append`, `resolve-references-cli-exec-wiring`, `waiver-processing-order`).

### Fixed

- **`merge-managed-block`** — the group-alignment walk consumed the first adopter section *after* the managed block as a "rewrite" whenever the new canonical block appended trailing subsection(s), deleting adopter content on every `/govern` update. Unmatched trailing canonical groups are now pure insertions; adopter content beyond the block is preserved verbatim (including when an adopter heading collides with an appended canonical heading).
- **`validate-frontmatter`** — a spec missing `status` or `dependencies` entirely returned `clean: true`; both now produce blocking findings per the constitution's hard-fail tier. A present-but-empty frontmatter block (`---`/`---`) is treated as an empty mapping (both missing-field findings) rather than a `MissingFrontmatter` operational error — a small widening that applies to every frontmatter consumer.
- **`read-spec` / `mark-criterion`** — the acceptance-criteria walkers had no HTML-comment/fence awareness, so the spec template's comment-embedded example checkbox counted as a phantom criterion (and was flippable). Both now share a single `SkipScanner`-aware section walker, keeping their index contract aligned by construction; template-state specs report zero criteria.
- **`set-status`** — the frontmatter splice offset hardcoded the 4-byte `---` + LF opener; on CRLF spec files every transition wrote one byte early and corrupted the frontmatter. The offset now comes from the opener actually matched.
- **`check-rule-ids`** — the deprecation scan sliced the rule file at a raw byte offset and panicked when the window edge landed mid-UTF-8 character (em-dash-dense rule files made this reachable); the slice now backs up to a char boundary.
- **`writeCode` boundary validation** — edit paths were prefix-matched without normalization, so `runtime/../framework/x` satisfied `runtime/**`; absolute paths and `.`/`..`/empty segments are now rejected before pattern matching.
- **`assessSpecQuality`** — the extension point sent a raw walker-context dump instead of its documented request; it now builds the typed `spec-path` / `spec-content` / `rule{id, verification, severity}` shape (analyze golden re-blessed).
- **`gvrn exec` parse failures** — exited 2 with no terminal message, violating the exit-code contract; they now emit a terminal `error` envelope carrying the runtime version and a version-mismatch note.
- **stdio robustness** — a stray envelope, mismatched request-id, malformed line, or blank keepalive on stdin halted the walk (or surfaced a raw I/O error); per the data model these are now logged to stderr and ignored while the runtime keeps waiting. Stdin EOF remains an operational error.
- **`resolve-references`** — was reachable only over MCP; now wired as a CLI subcommand, an interpreter dispatch arm, and a `PRIMITIVE_NAMES` entry, with a registry-equality test so the name lists cannot silently diverge again.
- **`/gov:review` waiver ordering** — the procedure classified waivers *before* the passes produced findings, mass-expiring every valid waiver on the exec path; `process-waivers` now runs after the five passes (binding the accumulated findings to `fired`) and immediately before `write-review` (review golden re-blessed).

### Notes

- Ships in lockstep with the framework per [§runtime-boundary](../framework/constitution.md#runtime-boundary). `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `scripts/audit/run-all.sh` are clean.

## [0.16.0] — 2026-07-10

Adds the `prune-tasks` primitive backing the new `/gov:prune` command (spec 041).

### Added

- **`prune-tasks`** — reduces a feature's `tasks.md`. The default keep-pending prune drops every *spent* (≥ 1 checkbox, all checked) task section and any phase container left with no surviving task section, preserving the preamble and every pending / no-checkbox section verbatim; `--reset` rewrites the file to the template's initial state (the existing `# …` heading plus the canonical empty-tasks body). Reset is gated on spec status — permitted only when the spec is `done`, unless `--force` is supplied. A two-mode `apply` flag keeps the file body out of the caller's context: `apply: false` returns a compact summary (per-section classification, removed/kept counts, size before/after) and writes nothing; `apply: true` writes atomically. New MCP tool `prune-tasks`; new CLI subcommand `gvrn prune-tasks`; the tool list grows from 32 to 33.

### Notes

- Ships in lockstep with the framework per [§runtime-boundary](../framework/constitution.md#runtime-boundary). `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` are clean.

## [0.15.0] — 2026-07-06

The **review-runtime-acceleration** series (spec 022 scenario `review-runtime-acceleration`): the deterministic bookkeeping around `/gov:review`'s five semantic passes moves into primitives, and the passes themselves become a new `performReview` extension point. The bump to `0.15.0` is driven by the breaking `create-scenario` arg-shape change below.

### Added

- **`discover-rule-files`** — owns `/gov:review` rule-file selection: rule-dir listing, suffix classification (`-backend` / `-frontend` / `-cross` / unrecognized), `[rules] surfaces` selection (valid list / `[]` cross-only / unset derive-from-stack / degenerate fail-fast), and the `[[review.disabled-rule-files]]` filter — returning the selected set plus the ordered stdout notices verbatim.
- **`process-waivers`** — per-run classification of a spec's `review.waivers` against the currently-firing `(rule, file)` findings: apply / expire (with the `waiver expired: …` notice) / do-not-extend / malformed (skip-and-warn, never prune) / duplicate (first applies, dup warns). The anchor is the `(rule, file)` pair, so code moving within a file does not expire a waiver.
- **`compute-review-scope`** — resolves the `diff-base` (the status-to-`in-progress` commit, or a `--since` override), the file scope (plan `Affected Files` unioned with files modified since `diff-base`, larger set wins), and the inbox additions in the window, using `git2`.
- **`write-review`** — renders `specs/NNN/review.md` (frontmatter + fixed skeleton) and updates the spec `review:` frontmatter block. Applies the deterministic cross-pass dedup (highest-severity-wins on rule-id + file + overlapping range) before counting, buckets survivors into MUST / SHOULD / low-confidence / waived (applied waivers drop out of the counts), prunes expired waivers from `review.waivers`, and emits the 0-findings / `blocking: false` report for empty scope. `blocking` is true exactly when `must-violations` exceeds zero; both writes are atomic.
- **`performReview` extension point** — the LLM seam for each review pass. `PerformReviewRequest` carries the in-scope files (the cache-stable prefix) plus the rules loaded for the pass; `PerformReviewResponse` returns findings in the shape `write-review` consumes. The interpreter emits one `llm-request` per pass step and accumulates each pass's findings into the shared `findings` context key so a later `write-review` step consumes the union across all passes.

### Changed

- **BREAKING — `create-scenario` takes a single `body` argument** instead of separate `context` / `behavior` / `edge-cases` params. LLM-authored content now crosses the runtime boundary as one payload (the content-ingestion convention): the host assembles the `## Context` … `## Edge Cases` markdown in-context and hands it over whole, and the primitive keeps framing it with the `section:` frontmatter, the H1-from-slug, the atomic write, the slug-conflict refusal, and the auto-appended Open / Resolved Questions scaffolding. This removes the host MCP-encoder failure mode where one of several large sibling string params is silently dropped. `framework/commands/amend.md`'s scenario-branch prose is updated to match.
- **The procedure parser recognizes the four review primitives.** `discover-rule-files`, `process-waivers`, `compute-review-scope`, and `write-review` are added to `parser::PRIMITIVE_NAMES`, so the rewritten `/gov:review` Instructions resolve their backticked `Invoke` steps instead of raising `ParseError::Invalid`.

### Notes

- **Framework, shipping in this tagged archive (no runtime-crate behavior change):** `framework/commands/review.md` is rewritten from 604 lines of prose into a parseable nine-step procedure — `compute-review-scope` → `discover-rule-files` → `process-waivers` → five `performReview` passes → `write-review` — with the mechanical bookkeeping delegated to the primitives; `review.md` is removed from `runtime/legacy-prose-commands.txt`. The `#3/#4` prose tightening moves the ~1.1 KB "For agent runtimes" host-integration blockquote into `framework/constitution.md` §runtime-host-integration once and replaces it in every command with a one-line pointer, dropping the redundant `(MCP: …)` parentheticals and "Otherwise, follow the markdown-only path" tails; `scripts/lint-tool-coverage.sh` now exempts a command that carries the pointer from the per-reference proximity check.
- Ships in lockstep with the framework per [§runtime-boundary](../framework/constitution.md#runtime-boundary). `cargo test` (475 lib tests + the integration suites, including new MCP coverage for the four review primitives and a review-command parse-parity test), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, the parseability / tool-coverage lints, the framework self-audit, and markdownlint are clean.

## [0.14.2] — 2026-07-01

### Fixed

- **MCP tool schemas no longer carry non-standard numeric `format` hints.** `schemars` stamps OpenAPI-style `format` values (`uint32`, `uint64`, `uint8`) onto every Rust integer/float field, so every tool's input/output schema exposed over MCP annotated its numeric properties (`task-number`, `created`, `updated`, `unchanged`, `commit-count`, `line`, `bytes`, …) with a `format` JSON Schema defines no meaning for. Strict clients validate the served schemas and log a `unknown format "uint32" ignored in schema at path …` warning for each occurrence — opencode emits dozens on connect. `GovRuntimeServer::new` now walks every tool's input and output schema at construction and drops `format` from any node typed `integer`/`number`, leaving string formats (`date-time`, `uri`, …) untouched. This required pointing the `#[tool_handler]` at the stored, sanitized `self.tool_router` (`router = self.tool_router`) instead of its default `Self::tool_router()`, which regenerated a fresh, unsanitized router on every `list_tools`/`call_tool`. New `tests/mcp.rs` regression `no_tool_schema_carries_a_nonstandard_numeric_format` asserts no served schema exposes a numeric `format`. Schema-only change: no primitive behavior or parity golden is affected.

## [0.14.1] — 2026-06-30

### Fixed

- **`apply-manifest` preserves the source file's executable bit on the destination.** Every primitive write goes through `write_atomic_bytes`, which materializes the destination from a fresh `NamedTempFile` (mode `0600` on Unix) and renames it into place — so the written file always lost its executable bit. Invisible for the markdown/JSON the manifest mostly ships (git tracks only the exec bit), but fatal for the generator scripts: the **Shared Files** manifest ships `scripts/gen-spec-deps.sh` and `scripts/gen-cross-service-refs.sh` with `update` strategy, and the `govern-pre-commit` hook execs them. A `/govern` run therefore rewrote those generators to `0600`, failing the adopter's pre-commit hook; worse, a fresh adopter's first run hit the `created` path and emitted a non-executable generator from the start. `apply-manifest` now mirrors the source file's `Permissions` onto the destination after each write (`mirror_source_mode`, `cp -p` semantics) on the `created` / `updated` / `skip-if-conflict`-created paths; the `unchanged` path still never touches the file, preserving its mtime and its already-correct mode. This makes `apply-manifest` consistent with `extract-archive`, which already preserves archive-header modes (`apply_unix_mode`) so the staging source carries `+x` for the mirror to copy. New Unix-only regression tests `source_executable_bit_propagates_to_dest_on_create_and_update` and `skip_if_conflict_propagates_source_executable_bit`.

## [0.14.0] — 2026-06-30

### Added

- **Configurable spec-root directory name (spec 040).** The directory holding all govern artifacts is no longer hardcoded to `specs`. A new optional `[paths] specs-root` key in the committed `.govern.toml` names it, defaulting to `specs` so every existing adopter is unaffected. New `runtime/src/schema/paths.rs` parses and validates the value: a single directory-name segment in the conservative `[A-Za-z0-9_-]` charset (no path separators, no `.`/`..`, no regex metacharacters), so the name is safe both as a literal path component and when interpolated into the generators' regexes; an empty or out-of-charset value falls back to `specs`. Per [§runtime-boundary](../framework/constitution.md#runtime-boundary) the runtime reads this git-tracked source of truth — it does not own it. New `runtime/tests/specs_root_override.rs` covers the default, a non-`specs` override, and rejection of malformed values end-to-end.

### Changed

- **Every spec-root-joining primitive resolves the configured root through one shared resolver.** The primitives that take a bare *feature name* and join it under the root internally — `read-spec`, `set-status`, `mark-task`, `mark-criterion`, `read-tasks`, `traverse-deps`, `check-stuck`, `derive-boundary`, `resolve-references` — and the tree-enumerating `dashboard` previously hardcoded `repo.join("specs")`; they now resolve `[paths] specs-root` from `.govern.toml` (default `specs`) through a single helper in `primitives/mod.rs`, so the two resolutions cannot drift and the default path stays byte-identical to before. Primitives that take a full spec *path* argument (`write-session`, `lint-markdown`, `substitute-templates`) are unchanged — the host bakes the resolved root into that path. Runtime error messages that name a spec artifact now reflect the configured root (e.g. `governance/040-foo`) with no hardcoded `specs/` prefix.
- **`/gov:implement` no longer emits the `planned → in-progress` gate-confirm (spec 000).** Invoking the command is itself the user's approval to start work, so the implement walk drops the gate-confirm step and `writeCode` becomes the first request; the `implement-basic` golden, fixtures, and parity expectation are re-blessed to match.

### Fixed

- **`check-rule-ids` harvests digit-bearing rule-ID categories and matches the canonical deprecation label.** The rule-ID regex is widened to the schema grammar `[A-Z][A-Z0-9]+`, so digit-bearing categories (e.g. `FE-A11YFORM-*`, `FE-A11YMEDIA-*`) are harvested instead of being reported missing; `is_deprecated` now matches the canonical `**DEPRECATED in {version}:**` label rather than a bare `**Deprecated**` / `[DEPRECATED]`.

### Notes

- The `/ask` slash command was renamed to `/amend`; the runtime's mechanical references (the legacy-prose-command list, primitive doc strings and comments) were updated in lockstep — no behavior change.
- **Framework, shipping in this tagged archive (no runtime-crate impact):** `gen-cross-service-refs.sh` is now root-aware — it resolves a *referenced* service's own `[paths] specs-root` (spec 030 task 13, scenario `referenced-service-spec-root`) instead of assuming `specs/`, so a referenced service that renamed its spec root still harvests. `lib/specs-root.sh` gained a reusable `specs_root_of <toml>` helper.
- Ships in lockstep with the framework per [§runtime-boundary](../framework/constitution.md#runtime-boundary). `cargo test` (412 tests + the integration suites), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, both generator suites, the framework self-audit, and markdownlint are clean.

## [0.13.0] — 2026-06-20

### Added

- **`gvrn exec` resolves the `opencode` layout's singular `command/` directory (spec 032 runtime follow-up).** OpenCode installs its slash-command files under a *singular* `.opencode/command/{project}/<name>.md`, unlike claude-style's plural `.opencode/commands/…` (Claude, Auggie). The two command-resolution callsites (`main::run_exec` and `interpreter::payload::locate_command_file`) baked in `commands/` (plural), so `gvrn exec <name>` could never find an OpenCode adopter's command file — OpenCode shipped on the markdown-only path only (spec 032 Decision 10 deferred this to a 022 follow-up). The plural/singular knowledge is now factored into one new public method, `Host::command_file_candidates` (`runtime/src/host.rs`), which returns both flat-namespaced layout paths in order (plural first, then singular); both callsites consume it, so they can no longer drift. The two candidates are mutually exclusive per adopter (each agent's `cli_config_dir` and `layout` select exactly one), so trying both resolves any supported layout without the runtime knowing which agent wrote the file. Existing claude-style adopters are unaffected — the plural form is tried first and resolves exactly as before. Layout detection needs no new host-config field (the layouts are told apart by trying both candidates); where `cli-config-dir` is read from is the separate concern handled by the relocation below. New unit test `command_file_candidates_cover_both_layouts_plural_first` and a new `exec-opencode` parity fixture + subprocess test (`exec_resolves_command_via_opencode_singular_command_dir`) cover it.

### Changed

- **`cli-config-dir` relocated from committed `.govern.toml` to the per-contributor `.govern.session.toml`.** The `[host]` block committed both `cli-config-dir` and `project`, but `cli-config-dir` names the contributor's agent config dir (`.claude` / `.augment` / `.opencode` / `.agents`) — a per-contributor choice, since teammates on one project may use different agents. Committing it baked one contributor's agent into shared config, so everyone else resolved command files against the wrong directory. `project` (the shared slash-command namespace) stays committed; `cli-config-dir` moves to the gitignored, per-contributor `.govern.session.toml`. `crate::host::Host::load` now reads `cli_config_dir` from the session file → legacy `.govern.toml` `[host]` (so existing adopters keep working) → default `.claude`, and `project` from `.govern.toml`. `write-session` becomes a **merge-writer** over the session file (all on-disk keys optional): a *target write* (`feature`+`path`) sets the target and preserves `cli-config-dir`; a *host-config write* (`cli-config-dir`, no `feature`) sets the agent identity and preserves the target — so `WriteSessionArgs.feature`/`path` are now optional and a new `cli-config-dir` arg is accepted (existing callers that always pass `feature`+`path` are unaffected). `dashboard`'s session reader treats `feature` as optional (a session file with only `cli-config-dir` reports no target rather than erroring). On the framework side, `/govern` (`framework/bootstrap/govern.md` §Instructions step 6) writes `project` to `.govern.toml` `[host]` and `cli-config-dir` to the session file; `/{project}:target` (`framework/commands/target.md`) preserves `cli-config-dir` across a target switch and on `--clear`. Migration is self-healing — no dedicated primitive: the legacy fallback keeps adopters resolving until their next `/govern`, which drops `cli-config-dir` from the committed managed block and records it in the session file. New unit tests cover the resolution order, the merge-writer's target/host-config writes and validations, and the dashboard's optional-feature path.

### Fixed

- **`merge-managed-block` (line-prefix) now absorbs a *structure-changing* canonical edit without leaving an orphan tail.** The previous fix (v0.12.0, see below) made `find_line_prefix_block`'s `walk_body_extent` bound the on-disk block by walking `block.lines().count()` lines using the new canonical as a *structural* template — expected-blank matches on-disk-blank, expected-non-blank-but-on-disk-blank terminates. That works only when the on-disk block and the new canonical share the same shape (same line count, same blank-line positions): a comment-wording tweak, the realistic case it was written for. It does **not** hold when the canonical gains or loses a subsection — exactly what happens when a new agent's gitignore block is added (e.g. Auggie's `.augment/*` + `!.augment/commands/` inserted between the Claude and Antigravity subsections in [`framework/templates/project/gitignore`](../framework/templates/project/gitignore)). The new canonical is then longer than what an existing adopter has on disk; the structural walk drifts by the inserted line count and terminates ~4 lines early, so the old block's tail subsections (`# govern session state` / `# IDE` / `# OS`) spilled past `body_end` into `after`. The cross-boundary dedup pass then stripped their pattern bodies but preserved their comment headers, leaving a trail of orphan headers below the freshly written block. The merge converged on the second run (no infinite accumulation) but the orphan comment lines were permanent. `walk_body_extent` now bounds the on-disk block by **group alignment** instead of a line-shape walk: it splits both the canonical and the on-disk region into blank-line-delimited subsections, reduces each to its pattern lines (non-blank, non-comment — the stable identity across wording drift), and aligns them with a two-pointer walk. An on-disk group is in the block when it shares a pattern with the current canonical group (structure-preserving edit), shares a pattern with a *later* canonical group (the canonical inserted a subsection — skip past it), or shares no pattern while canonical groups remain (a full rewrite of that group); the block ends at the first on-disk group reached after the canonical's groups are exhausted (adopter territory, preserved). Full-content replacement of a single-group block and adopter language sections appended after the block are both preserved unchanged. New regression test `line_prefix_multi_subsection_inserts_new_subsection_without_orphan_tail` exercises the inserted-subsection case (clean replacement, Auggie present once, no duplicated headers, adopter `# Rust` tail preserved, idempotent rerun). All 28 `merge_managed_block` unit tests pass, including the v0.12.0 structure-preserving update test unchanged.

- **`.augment/` (Auggie) added to the framework-managed `.gitignore` block.** [`framework/templates/project/gitignore`](../framework/templates/project/gitignore) gained a `.augment/*` + `!.augment/commands/` subsection mirroring the Claude `commands` carve-out — Auggie is a `claude-style` agent (per the Agent Registry in `framework/bootstrap/govern.md`), so its config dir tracks `commands/` exactly as `.claude` does. The block had `.claude` and `.agents` (Antigravity) subsections but Auggie's was never added when the agent was introduced. The two prose enumerations of the managed block in `framework/bootstrap/govern.md` now list `.augment/` alongside `.claude/` and `.agents/`.

## [0.12.1] — 2026-06-17

### Fixed

- **`gen-cross-service-refs.sh` was never distributed to adopters (spec 030 follow-up).** The cross-service-references generator shipped wired only into this repo's own `.githooks/pre-commit`, so it ran on `govern`'s own commits but a `/govern` update in an adopter project never installed or ran it — the derived `references:` index silently never materialized downstream. The generator is now a row in the **Shared Files** manifest (`framework/bootstrap/govern.md`, `update` strategy) so `/govern` copies it, and is invoked by the adopter pre-commit hook (`framework/bootstrap/hooks/govern-pre-commit`) so commits regenerate `references:`.

### Changed

- **Adopter `govern-pre-commit` scopes generators to staged specs (`--staged`).** Both `gen-spec-deps.sh` and `gen-cross-service-refs.sh` gained a `--staged` flag, and the adopter hook now passes it: committing one spec only rewrites the specs in that commit, never dirtying or restaging unrelated tracked specs. `gen-spec-deps.sh` still builds the dependency-cycle graph from the **full** spec set (read all, write staged), so a staged edge that closes a cycle through an unstaged spec is still caught. This repo's own `.githooks/pre-commit` continues to run the generators unscoped, keeping the whole tree in sync.

- **Cross-service-reference authoring is now documented.** A new README "Documenting a reference in a spec" subsection and a spec-template comment show the concrete inline-link shape, the dependency-vs-reference distinction (sibling `../NNN-slug/` links stay dependencies; absolute service URLs become references), and the harvest opt-outs. The template's example link is backtick-wrapped so it is never harvested into an adopter's frontmatter if the comment is left in place.

### Notes

- No runtime crate code changes — the version is bumped in lockstep ([§runtime-boundary](../framework/constitution.md#runtime-boundary)) to ship the framework fixes via the tagged archive. The 416 runtime tests, both generator test suites (with new `--staged` coverage), the framework self-audit, and markdownlint are clean.

## [0.12.0] — 2026-06-14

### Added

- **`resolve-references` primitive + `[services]` registry — cross-service reference resolution (spec 030).** A spec can link a spec in another service by its canonical repo URL; when that service is registered in `.govern.toml` `[services]` (alias → `repo` / `path` / optional `description`) and checked out locally, `govern` resolves the linked spec's lifecycle `status` from the local checkout. The new `resolve-references` MCP tool (`runtime/src/primitives/resolve_references.rs`) classifies each entry of a consumer spec's derived `references:` index into a closed outcome enum — `ok` (status surfaced), `unregistered`, `not-checked-out`, `broken`, or `status-unreadable` — by deterministic predicates, reusing the `validate-frontmatter` machinery (`read_text` / `split_frontmatter` / `ALLOWED_STATUSES`). The canonical repo URL is identity and navigation only and is **never fetched**; resolution reads only `.govern.toml` and the registered local checkouts. References are informative, never dependencies — kept strictly distinct from `dependencies:` and never entering the blocking dependency graph.

- **`[services]` registry schema (`runtime/src/schema/services.rs`).** A new public `Services` / `ServiceEntry` type plus a `from_toml_str` parser and a `duplicate_repos` detector for the `.govern.toml` `[services]` table. An absent table is an empty registry (single-service adopters write nothing); a missing required field surfaces as a parse error rather than a silent default.

- **MCP exposure and command wiring.** `resolve-references` joins `TOOL_NAMES` and gains a `#[tool]` handler in `runtime/src/mcp/server.rs`; the `ResolveReferencesArgs` / `ResolveReferencesResult` / `ResolutionRecord` / `ReferenceOutcome` types land in `runtime/src/schema/primitives.rs`. The tool is registered in `framework/runtime-tools.txt` for the markdown-only-pipeline opt-in invariant and is referenced (each with a graceful markdown-only fallback) by `/gov:status` and `/gov:analyze`.

### Notes

- Backward-compatible, additive release: no existing tool, type, or behavior changes. Adopters with no `[services]` table and no cross-service references see no difference. The markdown-only path resolves references identically via host file tools, and the no-runtime CI job exercises that fallback end-to-end. All 416 runtime tests pass, including the new `runtime/tests/cross_service.rs` golden/parity test asserting the runtime and markdown-only paths produce byte-identical resolution records.

## [0.11.3] — 2026-06-09

### Fixed

- **`lint-markdown` silently dropped every violation.** markdownlint-cli2 v0.22.1 emits a severity token (`error`/`warning`) between the location and the rule ID — `path:line error MD028/alias message` — but `parse_violation_line`'s regex expected the rule immediately after the location (`path:line MD028 …`). No real output line matched, so `violations` came back empty even when markdownlint exited non-zero. The regex now accepts an optional `(?:error|warning)` token before the rule. Both the `gvrn lint-markdown` CLI and the `lint-markdown` MCP tool share the same `run()`, so both paths are fixed.

- **`clean` ignored the exit code.** It was derived solely from `violations.is_empty()`, contradicting the module's documented contract ("exit code 1 and 2+ both flow through as `clean: false`"). A parse miss — or any config/runtime error that produced no recognizable violation lines — therefore reported `clean: true` against a non-zero exit. `clean` is now `violations.is_empty() && exit_code == 0`, mirroring `run-generator`'s exit-code-derived `drift`, so a non-zero exit can never be reported as clean even if output-format drift defeats the parser again. Two parser tests cover the severity-token form (with and without a column).

## [0.11.2] — 2026-05-24

### Changed

- **Bump `toml` 0.8 → 1.x (post-1.0 stable line).** The `toml` crate's first stable major (1.1.2, spec TOML 1.1.0) is now the runtime's TOML parser. The session-file TOML→JSON bridge in `runtime/src/main.rs::run_exec` now deserializes directly into `serde_json::Value` (`toml::from_str::<Value>(&text)`) rather than parsing into `toml::Value` first and re-serializing — the indirect path tripped on `toml::Value`'s reworked Serialize impl in 1.x (table values were nested in a wrapper rather than flattened). The direct deserialization is also a simplification — fewer hops, one less crate-internal contract to track. No public API changes; the `PrimitiveError::Toml` variant's source type updates from `toml v0.8`'s `toml::de::Error` to the structurally-identical `toml v1.x` equivalent.

- **20 in-range dependency refreshes via `cargo update`.** Brings every direct-dep within its current Cargo.toml version range to the latest published version (mostly transitive bumps: `icu_*` 2.1 → 2.2, `wasm-bindgen` 0.2.121 → 0.2.122, `wit-bindgen` 0.46 → 0.57, `web-sys` 0.3.98 → 0.3.99). One direct-dep increment: `serde_json` 1.0.149 → 1.0.150. No code changes required; all 359 lib + 6 + 16 + 10 + 3 + 2 tests pass against the refreshed graph.

### Verified current

`cargo update --dry-run` reports zero pending updates after this release. Every direct dependency declared in `runtime/Cargo.toml` matches the latest published version compatible with its version range (`anyhow`, `clap`, `flate2`, `git2`, `pulldown-cmark`, `regex`, `reqwest`, `rmcp`, `schemars`, `serde`, `serde_json`, `serde_norway`, `sha2`, `tar`, `tempfile`, `thiserror`, `toml`, `tokio`, `walkdir`, `zip`). The only major bump deferred is `zip 8 → 9.0.0-pre2`, held back because 9.x has not yet shipped a stable release.

## [0.11.1] — 2026-05-24

### Changed

- **Swap deprecated `serde_yaml` for the actively-maintained `serde_norway` fork.** Frontmatter parsing (5 callsites under `runtime/src/primitives/{dashboard,read_spec,traverse_deps,validate_frontmatter}.rs` plus the shared `PrimitiveError::Yaml` variant in `runtime/src/primitives/mod.rs`, plus one test-side parse in `runtime/tests/parity.rs`) moves from `serde_yaml v0.9.34+deprecated` (last release March 2024; dtolnay no longer maintaining) to `serde_norway v0.9.42`. The fork is API-compatible — pure `serde_yaml::` → `serde_norway::` mechanical swap, no behavior changes — and currently the most actively-maintained option in the post-deprecation YAML-for-Rust landscape. All 359 lib + 6 exec_subprocess + 16 parity + 10 atomic + 3 mcp + 2 walker tests pass against the new dep unchanged.

## [0.11.0] — 2026-05-24

### Added

- **`Host` config loader and `.govern.toml` `[host]` block — parameterized command-file resolution.** The runtime previously resolved slash-command files via three hardcoded candidate paths, the middle of which baked in both Claude Code's config-dir name (`.claude`) and this repo's slash-command namespace (`gov/`). That combination broke adopters whose layout matched neither default: an Auggie adopter named `anvil` has commands under `.augment/commands/anvil/*.md` and the runtime's lookup never reached them. A new `gvrn::host::Host` public type and its `Host::load(repo: &Path) -> Self` constructor read the host's values from `.govern.toml`'s `[host]` block (`cli-config-dir` and `project` keys); both command-resolution callsites (`gvrn exec`'s `run_exec` in `runtime/src/main.rs` and the anchor-extractor's `locate_command_file` in `runtime/src/interpreter/payload.rs`) now construct the middle candidate as `{host.cli_config_dir}/commands/{host.project}/{name}.md`. When the block is absent the loader falls back to `.claude` / repo directory basename — preserving this repo's behavior unchanged — via two module-level consts (`DEFAULT_CLI_CONFIG_DIR`, `FALLBACK_PROJECT`). Six unit tests under `runtime/src/host::tests` cover missing file, empty file, block absent, full override, partial override (per-field defaults), and malformed-TOML fall-soft. An integration test under `runtime/tests/exec_subprocess.rs` (`exec_resolves_command_via_parameterized_host_block`) exercises the parameterized lookup against an Auggie-shaped fixture at `runtime/tests/fixtures/exec-auggie/` (no `framework/commands/` tree, command file at `.augment/commands/anvil/smoke.md`, `.govern.toml` declaring the override). Closes spec 022's `commands-dir-parameterization` scenario.

- **Bootstrap procedure writes the `[host]` block on every `/govern` run.** `framework/bootstrap/govern.md` gains a new step 6 (between the existing `.gitignore` merge and `enforce-manifest` cleanup) that invokes `merge-managed-block` against `.govern.toml` with `marker-style: "line-prefix"` and `marker: "govern (host)"`. First-run creates the file with just the managed block; subsequent runs update the values in place under the `# govern (host)` preamble line, preserving every other section (`[pinned]`, `[workflows]`, `[migrations]`, `[review]`) byte-for-byte. The §Project Configuration section's example TOML and per-key reference now document the `[host]` schema; step 1's host-context list picks up the new `host-block` item; subsequent steps are renumbered (prior 6/7/8 → 7/8/9).

- **Audit Family 13 — runtime hardcoded paths (`scripts/audit/runtime-hardcoded-paths.sh`).** `git grep`-scans `runtime/src/**` for the literal `.claude/commands/gov/` string. Any match signals a regression to the host/project hardcode the parameterization removed, with a SUGGESTED-FIX pointing at `runtime/src/host.rs::Host::load`. Specs, scenarios, migration bodies, tests, and fixtures are out of scope (the prior path appears in them legitimately as historical context). Wired into `scripts/audit/run-all.sh` after Families 1–12.

### Fixed

- **Auggie / Anvil / non-default-layout adopters can resolve their command files.** Before this release, an adopter whose `cli-config-dir` was not `.claude` or whose project namespace was not `gov` would invoke `gvrn exec <name>` and get `runtime exec: command file not found` — because the runtime's second candidate path was the literal `.claude/commands/gov/<name>.md` regardless of the adopter's actual layout. With the parameterized lookup, the runtime reads the adopter's values from `.govern.toml`'s `[host]` block and resolves the correct path on the first try.

### Changed

- **`framework/commands/analyze.md` and `framework/migrations/spec-and-plan-sunset.md` no longer cite the dropped "frozen archaeology" rule.** Spec 023's `living-specs` scenario removed the exception from constitution §drift-prevention; two live framework files still referenced it. `analyze.md`'s scenario-frontmatter check is reframed as backward compatibility (pre-017 scenarios written before the `section` field existed may still carry `spec-ref`). `spec-and-plan-sunset.md`'s done-spec rename is reframed as a §spec-lifecycle mechanical edit (uniform body-identical change, no back-edge). The generated `.claude/commands/gov/analyze.md` mirror was regenerated by `scripts/gen-claude-commands.sh`.

### Documentation

- `AGENTS.md` Workflow section gains a new entry codifying the "never use frozen-archaeology phrasing" convention (rule + **Why:** + **How to apply:**), so the dropped term doesn't sneak back into future work.

### Test infrastructure

- **`runtime/tests/common/mod.rs` — shared helpers for integration test crates.** Each `tests/*.rs` compiles as its own integration-test binary, so previously-shared logic (`copy_dir_recursive`) had been duplicated across `parity.rs` and `exec_subprocess.rs`. The new sub-module path is the idiomatic Rust workaround: `tests/common/mod.rs` is NOT auto-built as a test binary (a `tests/foo.rs` file would be), and each integration test declares `mod common;` to bring the helper into scope. Both call sites now import `common::copy_dir_recursive` from the canonical implementation.

## [0.10.0] — 2026-05-24

### Added

- **`migrate-session-file` primitive — backs the `session-file-consolidate` bootstrap migration with tested code.** New MCP tool and CLI subcommand that reads a pre-0.10.0 legacy session JSON (`.claude/{project}-session.json`), applies the camelCase→kebab-case key renames (`scenarioPath`→`scenario-path`, `setAt`→`set-at`), writes the result to `.govern.session.toml` via tempfile+rename, and deletes the legacy file. Idempotent: returns `action: "no-legacy"` when no legacy file exists, `action: "kept-existing"` when `.govern.session.toml` is already present (legacy still deleted), `action: "migrated"` on a fresh translation. The destination path is sourced from `write_session::SESSION_FILE`, so the migration cannot drift from the runtime's read/write path. 10 new unit tests under `runtime/src/primitives/migrate_session_file.rs` cover happy path, idempotency, preserve-existing, non-standard-key preservation, malformed-input rejection, path-traversal rejection, and (critically) a compile-time assertion that the primitive's destination equals `SESSION_FILE`. The `framework/migrations/session-file-consolidate.md` body now invokes the primitive on the runtime path with a markdown-only fallback for adopters without `gvrn` on `PATH`.

- **Audit Family 11 — consolidation-pair drift (`scripts/audit/consolidation-pair.sh`).** Verifies the consolidated session-file path agrees across every artifact that references it: the runtime `SESSION_FILE` constant, the migration body's destination prose, the framework gitignore template, and the Claude configure-permission file. Also asserts the migration body names both legacy keys (`scenarioPath`, `setAt`) AND their kebab-case replacements (`scenario-path`, `set-at`) so the rename contract is auditable — a silently dropped rename would leave adopters with camelCase keys the runtime ignores. Verified against synthetic drift in both axes.

- **Audit Family 12 — fixture session-file shape (`scripts/audit/fixture-session-shape.sh`).** Verifies every `runtime/tests/fixtures/*/.govern.session.toml` (a) parses cleanly as TOML and (b) does not use the legacy camelCase keys. Test-data complement to Family 11. Verified against synthetic drift.

### Changed

- **Session state consolidated onto `.govern.session.toml` at the repo root.** `write-session` and `dashboard` previously read/wrote `.claude/gov-session.json` — a hardcoded path that baked in both the AI CLI's config directory (`.claude/` for Claude Code) and the adopting project's name (`gov-session.json` for this repo). That broke adopters whose project name or AI CLI didn't match the runtime's baked-in constants (observed against an adopter named `anvil`, whose canonical session would have been `.claude/anvil-session.json`): `/{project}:target` wrote the gov-shaped filename while every downstream consumer read the bootstrap-substituted one, and the session never round-tripped. The fix is consolidation, not parameterization — both primitives now read/write `<repo>/.govern.session.toml`, a single location with no `{cli-config-dir}` or `{project}` variability. The new file sits alongside `.govern.toml` at the repo root, is gitignored (per-user, ephemeral state), and uses TOML to align with `.govern.toml`'s on-disk format. Keys are kebab-case (`scenario-path`, `set-at`) rather than the legacy camelCase (`scenarioPath`, `setAt`). The runtime CLI's walker-context seed in `gvrn exec` reads the same file via TOML→JSON bridging so parity fixtures keep working. Closes spec 022's reopened consolidation scope.

- **`merge-permissions`'s `path` is now required.** The previous `DEFAULT_PATH = ".claude/settings.local.json"` constant silently routed non-Claude hosts to a Claude-shaped destination. The bootstrap procedure already passes the path explicitly via `{cli-config-dir}/settings.local.json`, so the default was unused on every supported invocation path; removing it makes a missing path fail loudly instead of corrupting an Auggie adopter's settings.

### Migration

- **Bootstrap migration `session-file-consolidate`.** Added to `framework/migrations.toml` (introduced in 0.10.0). On the next `/govern` run, the bootstrap detects any legacy `{config_dir}/{project}-session.json` files (across every selected agent's config dir), translates the most recent one into `.govern.session.toml` with kebab-case key renames (`scenarioPath` → `scenario-path`, `setAt` → `set-at`), and deletes the legacy file(s). Adopters who skip the bootstrap can also clear the legacy file by re-running `/{project}:target` once after upgrade.

- **`.govern.session.toml` added to the framework-managed `.gitignore` block.** The next `/govern` run rewrites `.gitignore` via `merge-managed-block` and the new entry lands automatically.

### Schema (wire-compatible, no breaking deserialization)

- `MergePermissionsArgs.path` changed from `Option<String>` to `String`. CLI callers that omitted `--path` now get a clap error instead of a silent default; the bootstrap procedure already passes the path on every call.
- `DashboardArgs` reverts to an empty struct. Session-target reads use the constant `.govern.session.toml` path; callers do not pass a `session-path` arg.
- `WriteSessionArgs.session_path` was *not* added (intermediate path-parameterization design rejected). The TOML on-disk shape uses kebab-case keys (`scenario-path`, `set-at`); the JSON wire shape for CLI/MCP args is unchanged from 0.9.x.

### Documentation

- Framework command sources (`framework/commands/*.md`), the bootstrap procedure (`framework/bootstrap/govern.md`), the configure permission file (`framework/bootstrap/configure/claude.md`), and `framework/constitution.md` reference `.govern.session.toml` exclusively. The "Session JSON path" derived-value row in the bootstrap's per-agent registry table is removed (the session file is no longer per-agent).

## [0.9.2] — 2026-05-23

### Fixed

- **`merge-managed-block` (line-prefix) detects the end of a multi-subsection canonical correctly.** The primitive's `find_line_prefix_block` helper previously used a "next blank line is the terminator" heuristic to bound the on-disk canonical block. That heuristic mis-truncated canonicals containing interior blank lines between subsections — the shipped `.gitignore` template ([`framework/templates/project/gitignore`](../framework/templates/project/gitignore)) is exactly this shape — so the `body == block` comparison could never succeed on multi-subsection canonicals and the *updated* arm's `after = &text[body_end..]` re-emitted the tail subsections as adopter-area content. The cross-boundary dedup pass stripped non-comment body lines but preserved subsection-header comment lines, so each `/govern` rerun left an orphan trail of empty `# Environment and secrets` / `# Claude Code …` / `# IDE` / `# OS` headers below the real block. The helper now walks up to `block.lines().count()` lines from the marker using the supplied block as a *structural* template: expected blank lines (interior subsection separators) match against on-disk blanks; an unexpected blank (non-blank expected, blank found) signals the end-of-block terminator. Two new unit tests under `runtime/src/primitives/merge_managed_block.rs::tests` cover the regression — a stable-rerun assertion (`action == "unchanged"`, `dedup_removed == 0`, mtime preserved) and a content-changed-update assertion (clean replacement, each subsection header appears exactly once). All 27 existing `merge_managed_block` unit tests pass unchanged. Closes spec 022's `merge-managed-block-multi-subsection-end` scenario.

## [0.9.1] — 2026-05-23

### Fixed

- **`traverse-deps` now detects cycles in the reachable dep subgraph.** The primitive previously checked dependency existence and per-edge status compatibility but ignored graph-level acyclicity, so a 2-cycle (`A → B → A`), a self-cycle (`A → A`), or any deeper SCC slipped through silently. Spec 017's `gen-spec-deps.sh` blocks cycles at commit time, but that check does not cover adopters on an older shipped script, skipped pre-commit hooks, or stale frontmatter edits that drift from the body links — all paths where a cycle can re-enter the artifact tree without the generator firing. The primitive now runs Tarjan's strongly-connected-components algorithm over the subgraph it walks and reports every non-trivial SCC (size ≥ 2, or size 1 with a self-edge) in a new `cycles: Vec<Vec<String>>` result field; each entry names the participating slugs in traversal order. Any non-empty `cycles` flips `compatible` to `false`, so `/gov:analyze` step 3 fails its gate without further wiring. Eight new unit tests under `runtime/src/primitives/traverse_deps.rs` cover the scenario's five edge cases (cycle among `done` specs still reported; self-cycle as 1-cycle; multiple disjoint SCCs; missing-node-doesn't-close-a-cycle; stale-frontmatter cycle visible) plus a 3-node hop case; an MCP integration test in `runtime/tests/mcp.rs` and a CLI-subprocess parity test in `runtime/tests/parity.rs` exercise both surfaces against a hand-built 2-cycle fixture so the markdown-only walker (agent + MCP) and the runtime walker (`gvrn traverse-deps`) surface equivalent findings. Closes spec 022's `traverse-deps-cycle-check` scenario.

### Changed

- **`TraverseDepsResult` schema gains a `cycles` field.** Additive and `#[serde(default)]` — adopters reading the JSON envelope on `0.9.0` continue to deserialize cleanly; consumers reading `0.9.1` see an empty array on every acyclic invocation. `framework/commands/analyze.md` step 3 prose now names cycles as blocking and explains the defense-in-depth relationship with spec 017's generator-side check.

## [0.9.0] — 2026-05-23

### Added

- **`write-session` primitive — atomic rewrite of `.claude/gov-session.json`.** New MCP tool and CLI subcommand that writes the session-target record (feature, path, optional scenario + scenarioPath, setAt) through the same tempfile + rename pattern every other state-modifying primitive uses. Pairs with `dashboard`'s read of the same file: spec 022 already listed the session file as one of two durable journals (markdown + `.claude/gov-session.json`), and the read path was in the runtime since 0.8.0; the write path closes the asymmetry. On Claude Code, routing the write through MCP moves consent from the per-invocation `Write({cli-config-dir}/{project}-session.json)` permission prompt — which existing `Write(...)` allow entries did not reliably suppress across sessions — into the MCP tool-permission lane, where a single allow covers every subsequent `/gov:target` and `/gov:ask` scenario-switch. 13 new unit tests under `runtime/src/primitives/write_session.rs` cover the happy paths (with/without scenario, fresh-vs-overwrite, directory creation), error paths (mismatched scenario pair, parent-component path, absolute scenario path), and the atomic-write contract (dropped tempfile leaves destination unchanged).

### Changed

- **`framework/commands/target.md` step 7 now invokes `write-session`.** The host-write prose is replaced with the primitive call; the markdown-only fallback still writes the same JSON shape directly with the same tempfile + rename semantics. Step 1 additionally names `{cli-config-dir}/{project}-session.json` inline (with the Claude resolution to `~/.claude/gov-session.json`) so hosts no longer have to derive the path from the parity `strict-files` frontmatter.

- **`framework/commands/ask.md` scenario-route step 4 now invokes `write-session`.** Same migration as target.md: the "host responsibility — the runtime exposes no session-shaped primitive" wording is removed; the markdown-only fallback remains.

- **`framework/runtime-tools.txt` gains the `write-session` line.** Matched by the parser's `PRIMITIVE_NAMES` and the MCP server's `TOOL_NAMES`.

- **`runtime/tests/golden/target-basic.jsonl` updated.** The byte-stream now includes the `write-session` dispatch envelope between `read-spec` and `complete`. Re-blessed via `BLESS=1 cargo test target_basic`.

## [0.8.1] — 2026-05-23

### Changed

- **Internal: `section_lines` extracted to `primitives/mod.rs`.** Both `read_spec::parse_open_questions` and `dashboard::{count_open_questions, context_summary}` now share the section-traversal helper via `primitives::section_lines` (new `pub(crate)` function). The iteration semantics are the single source of truth; consumers diverge only in how they fold the yielded lines into their result shape. Closes the `count_open_questions` / `parse_open_questions` semantic-drift surface the `/gov:review` pass against `c15ae0e` flagged on pathological inputs. Six new direct unit tests in `primitives::tests` cover the extracted helper.

- **Internal: `is_feature_slug` promoted to `primitives/mod.rs`.** The `NNN-feature` pattern check moves from `dashboard.rs` to `primitives/mod.rs` as `pub(crate)`, alongside `validate_slug` and `validate_no_traversal`. Currently one caller, but the pattern recurs across the codebase and the helper is small enough to promote ahead of demand.

- **Internal: `load_session_target` no longer accepts an unused `&[DashboardSpec]` parameter.** The dashboard scenario's last edge case explicitly forbids the session-target validation that parameter was prospective for ("Return the session-target field as-recorded; do not validate against the `specs` array"). The parameter existed for a use case the scenario contract rules out; removing it tightens the signature without changing behavior.

No behavior changes, no schema changes, no public surface changes. CLI subcommands, MCP tool shapes, and protocol envelopes are byte-identical to `0.8.0`. Patch bump per the runtime's convention for internal cleanups that leave the wire contract unchanged (precedent: `0.5.2`, `0.7.3`).

## [0.8.0] — 2026-05-23

### Added

- **`dashboard` primitive — single-call surface for `/gov:status`.** New MCP tool and CLI subcommand returning the per-spec inventory (slug / status / dependencies / tags / open-question-count / has-plan / has-tasks / has-data-model / scenarios-count / blocked-by), the repo-wide `tags-union`, the `.govern.toml` review-state summary (`{present, disabled-rule-files}`), and the optional session target (with `scenario-detail` populated when a scenario is targeted) in one call. Collapses the previous `/gov:status` "list specs + N read-spec + shell for-loop + cat .govern.toml" dance — which the §Instructions preamble already forbade as a fallback substitute — into a single MCP round-trip. `blocked-by` is computed in-primitive as the subset of `dependencies` whose own status is below `clarified`; `tags-union` is the sorted, deduplicated fold across every spec's `tags` array. 15 new unit tests under `runtime/src/primitives/dashboard.rs` cover the happy path plus every edge case enumerated in the scenario (empty `specs/`, missing `spec.md`, non-pattern dirs, `.govern.toml` absent / present-empty / parse-failure, scenarios with non-md files, session absent, session targeting a stale scenario, blocked-by computation, open-question continuation lines).

### Changed

- **`framework/commands/status.md` collapses to a single path.** The short-circuit branch (steps 2.1 / 2.2 — "stop after read-spec when target is not `done`") is removed. The procedure now invokes `dashboard` unconditionally and renders a one-line preamble above the table that surfaces the targeted feature (and scenario, when present) plus its next action. The §Instructions preamble names `dashboard` as the deterministic target for the status command so the shell-utility ban has a positive callout.

- **`Frontmatter` schema gains a serde-default `tags` field.** Backwards-compatible: specs that omit `tags:` continue to deserialize with an empty `Vec<String>`. Existing primitives (`read-spec`, `traverse-deps`) see the new field but do not surface it; `dashboard` is the first consumer.

- **Two new `PrimitiveError` variants.** `Toml` wraps `toml::de::Error` for `.govern.toml` parse failures; `MissingSpecFile` surfaces when an `NNN-feature` directory under `specs/` lacks the expected `spec.md`. Both surface as operational errors that halt the procedure with structured envelopes, consistent with the partial-failure semantics resolved in spec 022.

### Dependencies

- **`toml = "0.8"`.** New dependency used by the `.govern.toml` reader inside the `dashboard` primitive. Small, well-maintained crate; standard choice for TOML parsing.

## [0.7.4] — 2026-05-22

### Fixed

- **`merge-managed-block` cross-boundary dedup no longer destroys canonical content past the first interior blank line.** The dedup pass in `runtime/src/primitives/merge_managed_block.rs` previously re-derived the managed block's bounds by calling `find_line_prefix_block` on the post-merge content, which terminates at the first blank line. Canonical blocks shipped by the framework (notably `framework/templates/project/gitignore`) contain blank-line-separated subsections, so every canonical line past the first subsection was flagged `!in_block` and removed as an "adopter duplicate," leaving section comment headers with no patterns under them. `merge_line_prefix` now computes `block_start` and `block_end` directly from what it writes — `header.len() + 1 + block.len() + 1` past the start offset — and passes them as parameters to `dedup_outside_block`, which no longer re-scans for marker bounds. The contract for canonical blocks (string-equal-line removal in adopter territory, canonical-block wins) is unchanged; only the bounds computation moved from a fragile blank-line walk to a direct measurement. New regression test `line_prefix_preserves_multi_subsection_block_with_interior_blank_lines` exercises a multi-subsection block mirroring the shipped `.gitignore` template.

## [0.7.3] — 2026-05-22

### Changed

- **`writeCode` payload bundling now canonicalizes plan-relevant paths and case-folds the secret-pattern guard.** `load_plan_relevant_files` (in `runtime/src/interpreter/payload.rs`) previously joined each Affected-Files entry under `repo` and read it without verifying the resolved path stayed under the repo root. A plan entry of `../../etc/passwd` or `/etc/hosts` bypassed the basename-only secret-pattern check and exfiltrated through the outbound `writeCode` payload. The function now canonicalizes both `repo` and each candidate `abs` and rejects entries whose canonical form does not `starts_with(canon_repo)`, emitting `PayloadError::SecretExfiltration { pattern: "out-of-repo" }` so the existing `secret-exfiltration-blocked` error envelope stays the single surface for the whole class. Missing files (planned-new) still skip cleanly via the canonicalize-fails-continues branch — the existing happy path is preserved. `secret_pattern` also lowercases the basename before pattern matching so `.ENV` on macOS APFS cannot bypass the guard. Four new regression tests cover relative escape, absolute escape, in-repo happy path (positive), and case-fold bypass; the existing planned-new test continues to exercise the canonicalize-skip path for a fifth scenario. Closes the BE-INPUT-004 SHOULD finding recorded in `specs/022-deterministic-runtime/review.md`.

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

- **`mark-task` ignored phased `tasks.md` files.** The primitive only matched task headings at level 2 (`## N. ...`), so phased files (`## Phase X — … / ### N. Task`, the shape `read-tasks` learned to handle in 0.5.1) returned `task '{N}' not found` for every task. Surfaced 2026-05-19 during `/gov:implement` on spec 023 task #19 — the heading happened to contain backticks (``### 19. Dedup `/configure` permission entries``), so the bug initially looked like a backtick-parser issue, but inline-code spans were never the root cause; the parser handled them correctly via `parse_atx_heading` already.

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
