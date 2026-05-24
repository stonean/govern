# 022 — Deterministic Runtime Tasks

Tasks derived from the [plan](plan.md). Complete in order. Each task is small enough to complete and verify in a single session; later tasks depend on earlier ones.

## 1. Bootstrap the Rust crate at `runtime/`

- [x] Create `runtime/Cargo.toml` with package metadata (name `govern-runtime`, edition 2024, MSRV declared), the dependencies listed in the plan (`clap`, `serde`, `serde_json`, `serde_yaml`, `tokio`, `rmcp`, `pulldown-cmark`, `tempfile`, `thiserror`, `anyhow`, `regex`, `walkdir`, `git2`, `schemars`).
- [x] Create `runtime/src/main.rs` with a minimal clap-derive CLI exposing `mcp`, `exec`, `parse`, and `--version`. All subcommands print "not yet implemented" and exit 1; `--version` returns `env!("CARGO_PKG_VERSION")`.
- [x] Create `runtime/src/lib.rs` re-exporting `parser`, `interpreter`, `primitives`, `mcp`, `schema`, `io` modules as empty placeholders.
- [x] Create `runtime/.gitignore` excluding `target/`.
- [x] Create `runtime/CHANGELOG.md` with an initial `[Unreleased]` section.
- [x] Commit `runtime/Cargo.lock`.
- **Done when**: `cargo build --release` in `runtime/` succeeds; `runtime/target/release/runtime --version` prints the version from `Cargo.toml`.

## 2. Define schemas in `runtime/src/schema/`

- [x] Create one module per type group from `data-model.md`: `procedure.rs` (AST), `protocol.rs` (envelope + message types), `primitives.rs` (per-primitive args/results), `extensions.rs` (the three extension-point payloads).
- [x] Each type derives `serde::{Serialize, Deserialize}` and `schemars::JsonSchema`.
- [x] Unit tests exercise round-trip serialization for one example of each message type.
- **Done when**: `cargo test --release schema::` passes; running `runtime parse --emit-schema` (a debug subcommand) prints the JSON schema for the protocol envelope.

## 3. Implement read-only primitives

- [x] `read-spec` — parses spec frontmatter via `serde_yaml`, body sections via `pulldown-cmark`, acceptance criteria checkboxes, and open questions. Returns the `read-spec` result shape.
- [x] `read-tasks` — same approach for `tasks.md`.
- [x] `validate-frontmatter` — port of `scripts/lint-frontmatter.sh` semantics with real YAML parsing.
- [x] `resolve-anchor` — scan a markdown file for `§<anchor>` references and `<!-- §anchor -->` markers; cross-reference.
- [x] `traverse-deps` — read frontmatter `dependencies` and verify each named feature directory exists with a compatible `status`.
- [x] `check-rule-ids` — scan rule files for rule IDs, scan the target file for citations, flag missing or deprecated IDs.
- [x] `check-stuck` — use `git2` to count commits on a tasks.md path since the last status change.
- [x] `derive-boundary` — use `git2` to compute `git diff --name-only <first-commit-on-spec-dir>..HEAD` plus the spec dir.
- [x] Each primitive has a `clap`-derive args struct, a pure-Rust function (no I/O on stdout/stderr beyond the JSON result envelope), and a unit test against a fixture file under `runtime/tests/fixtures/primitives/`.
- **Done when**: `cargo test --release primitives::` passes; each primitive is invokable from the CLI surface (e.g., `runtime read-spec --feature 022-deterministic-runtime` prints valid JSON).

## 4. Implement write primitives with atomic semantics

- [x] `mark-task` — flips a single checkbox via tempfile-in-parent + `persist` rename. Returns the previous and current states.
- [x] `mark-criterion` — same approach for acceptance criteria checkboxes.
- [x] `set-status` — updates the `status:` field in spec frontmatter; refuses if the caller's `from` value doesn't match disk state.
- [x] Each write primitive has a unit test that simulates an interrupted write (drops the `NamedTempFile` without `persist`) and asserts the target file is unchanged.
- **Done when**: `cargo test --release primitives::` passes including the interruption tests; on macOS+Linux, the rename is verified atomic by reading the file from a parallel thread mid-write.

## 5. Implement wrapper primitives

- [x] `run-generator` — spawn a bash script with `--dry-run`, capture stdout/stderr/exit. Non-zero exit becomes `drift: true` (not an operational error).
- [x] `lint-markdown` — spawn `npx markdownlint-cli2` with the given paths; capture and parse the output into the `violations` array.
- [x] `gate-confirm` — under the subprocess interpreter, emits the `gate-confirm` JSON message and blocks for a `gate-response`. Under the MCP surface, returns the prompt unchanged and the caller is responsible for routing it (the MCP tool's response shape is `{ "prompt": "...", "gate": "..." }` and the client returns `{ "confirmed": bool }` as a separate tool call — documented in the MCP server's tool description).
- **Done when**: `cargo test --release primitives::` passes; manual smoke test of `runtime run-generator --script scripts/gen-spec-deps.sh` exits 0 and reports `drift: false`.

## 6. Expose primitives as MCP tools

- [x] Wire `rmcp` server in `runtime/src/mcp/server.rs`; expose each primitive as a tool named `gov-rt:<verb>-<noun>` per the resolved naming convention.
- [x] Tool input schemas are derived from `schemars::JsonSchema` derives on the args structs.
- [x] Each tool's handler delegates to the primitive's pure-Rust function and serializes the result.
- [x] Integration test in `runtime/tests/mcp.rs` starts the server in-process, connects an `rmcp` client, lists tools, and invokes each primitive against the per-primitive fixture from task 3 or 4. The test asserts that every tool name in `framework/runtime-tools.txt` is present.
- **Done when**: `runtime mcp` starts and serves the listed tools; the integration test exercises every primitive and passes.

## 7. Implement the procedure parser

- [x] Walk the `pulldown-cmark` event stream and recognize: numbered list items as steps (with sub-numbering for nested lists), backtick-quoted code spans inside a step matching a primitive name from §The primitive library, HTML comments matching `<!-- llm:<identifier> -->` as extension-point markers.
- [x] Emit the `Procedure` AST defined in `data-model.md`. Parse errors carry `SourceRange`.
- [x] Distinguish two failure modes: `ParseError::LegacyProse` (no parseable structure detected — the file is in the pre-rewrite format) versus `ParseError::Invalid` (structure attempted but malformed).
- [x] Implement `runtime parse <file>` (prints AST as JSON) and `runtime parse --check <file>` (exit 0 if parseable or legacy-allowlisted, exit 1 otherwise).
- [x] Unit tests cover: a well-formed Instructions section parses fully; an empty file is legacy-prose; an Instructions section with a malformed primitive backtick is `Invalid`; a step with both a primitive call and an extension-point marker is allowed (the marker overrides the primitive — extension point wins).
- **Done when**: every existing `framework/commands/*.md` either parses or returns `LegacyProse`; `runtime parse --check` on the current repo exits 0 (with all 14 commands in the legacy allowlist initially).

## 8. Wire the parseability check into `markdown-only-pipeline.yml`

- [x] Create `runtime/legacy-prose-commands.txt` listing all 14 command file paths (one per line). This task's edits remove a path from the file as each command is rewritten in later tasks.
- [x] Create `scripts/lint-procedure-parseability.sh`: builds `runtime/` in release mode (one cargo invocation, cached across runs), invokes `./runtime/target/release/runtime parse --check framework/commands/*.md` honoring the allowlist, exits non-zero on failure.
- [x] Edit `.github/workflows/markdown-only-pipeline.yml` to add step (f) after step (e), invoking the new lint. Add a comment in the workflow explaining that the binary built here is a workflow-local copy (used only for the parseability check) and not on `PATH` for the other steps.
- [x] Verify spec 021's check (a) still passes — the binary is built at `./runtime/target/release/runtime` (relative path), not added to PATH.
- **Done when**: the workflow file passes `actionlint`; locally running the workflow's bash steps produces the same exit codes as before plus the new step (f) exiting 0.

## 9. Implement the interpreter walker

- [x] `runtime/src/interpreter/mod.rs`: a synchronous walker over the parsed `Procedure`. Maintains a `State` struct (position, parsed file contents, pending payloads).
- [x] For each `Step::Primitive`, dispatch to the primitive's pure function. Errors halt the walker and emit an `error` JSON envelope.
- [x] For each `Step::Extension`, emit `llm-request` to stdout with a fresh `request-id`, suspend reading stdin until the matching `llm-response` arrives.
- [x] For each gate (recognized by the prose pattern "Ask the user to approve" — initial implementation; revisited if a more structured marker is needed), emit `gate-confirm` and suspend.
- [x] `Step::Prose` is no-op for the walker (information for the markdown-only path only).
- [x] Integration test under `runtime/tests/walker.rs` walks a fixture procedure that exercises every step type; the test mocks stdin/stdout and asserts the expected JSON sequence.
- **Done when**: `cargo test --release walker::` passes; manual smoke test of `runtime exec status` against a fixture repo produces a JSON message stream.

## 10. Wire `runtime exec <command>` to the interpreter

- [x] In `main.rs`, the `exec` subcommand: locates the slash command file at `framework/commands/<command>.md` (or `.claude/commands/gov/<command>.md` if `framework/` is unavailable — useful for adopting projects), parses it, hands the AST to the interpreter, drives the JSON-over-stdio loop.
- [x] Output stream uses `runtime/src/io.rs` line-framing helpers; every JSON object is flushed immediately after writing.
- [x] Exit codes per the plan: 0 on `complete`, 1-127 on `error`, signal codes pass through to the process exit.
- [x] Integration test starts the runtime as a subprocess from the test harness, pipes stdin/stdout, and exercises a fixture command end-to-end.
- **Done when**: `runtime exec status` against the `runtime/tests/fixtures/status-basic/` fixture produces the expected JSON stream and exits 0.

## 11. Define the three extension-point schemas and add validation

- [x] In `schema/extensions.rs`, finalize the `assessSpecQuality`, `writeCode`, `writeSpecBody` request/response types (already drafted in `data-model.md`).
- [x] The interpreter validates incoming `llm-response` payloads against the schema for the extension point that emitted the request. Validation failures emit `error: schema-mismatch` with the field path and the parsed schema diff.
- [x] For `writeCode`, the interpreter additionally checks that every `edits[].path` is within the `write-boundary` and rejects the response with `error: out-of-boundary-edit` before applying any edit.
- [x] Unit tests under `schema::extensions::tests` cover: missing required field; unexpected enum value; out-of-boundary path in `writeCode`.
- **Done when**: `cargo test --release schema::extensions::` passes; an invalid `llm-response` causes a clean `error` envelope and exit 1.

## 12. Rewrite `/gov:status`

- [x] Edit `framework/commands/status.md`: rewrite the Instructions section to follow the parseable conventions (numbered steps, backtick-quoted primitive names from the runtime library, no extension-point markers — `/gov:status` is fully deterministic).
- [x] Add the `parity: { strict-stdout: true }` frontmatter field.
- [x] Remove `framework/commands/status.md` from `runtime/legacy-prose-commands.txt`.
- [x] Create `runtime/tests/fixtures/status-basic/` with a minimal repo state; create `runtime/tests/golden/status-basic.jsonl` with the expected JSON stream; create `runtime/tests/parity/status/expected.txt` with the captured LLM-driven output.
- [x] Verify `runtime parse --check framework/commands/status.md` passes.
- [x] Verify `runtime exec status` against the fixture produces the golden stream byte-for-byte and the dashboard output matches the parity capture.
- [x] Verify `scripts/lint-tool-coverage.sh` still passes (every primitive reference paired with a fallback marker within 20 lines).
- **Done when**: parseability check green; integration + parity tests for `/gov:status` pass; tool-coverage lint green.

## 13. Rewrite `/gov:target`

- [x] Same shape as task 12, against `framework/commands/target.md`. Includes session-file write through `mark-task`-equivalent atomic-write semantics (but the session file is JSON, not markdown — implementation note: the runtime uses the same tempfile+rename pattern for any state-modifying primitive regardless of target file shape).
- [x] `parity: { strict-files: [".claude/gov-session.json"] }`.
- [x] Fixture under `runtime/tests/fixtures/target-basic/`; golden + parity captures.
- **Done when**: parseability + integration + parity green for `/gov:target`; tool-coverage lint green.

## 14. Rewrite `/gov:analyze`

- [x] Rewrite `framework/commands/analyze.md` to invoke the mechanical primitives (`validate-frontmatter`, `resolve-anchor`, `traverse-deps`, `check-rule-ids`, `run-generator`, `lint-markdown`) for the deterministic checks, and an `<!-- llm:assessSpecQuality -->` marker on the per-rule Verification step.
- [x] `parity: { semantic-fields: ["findings[].message"], strict-fields: ["findings[].rule-id", "findings[].severity"] }`.
- [x] Fixture under `runtime/tests/fixtures/validate-basic/`; golden stream + parity capture.
- [x] The fixture exercises at least one MUST-tier finding and one SHOULD-tier finding so the extension point's response routing is covered.
- **Done when**: parseability + integration + parity green; the parity check on findings is set-equality on (rule-id, severity, file, line); tool-coverage lint green.

## 15. Rewrite `/gov:implement`

- [x] Rewrite `framework/commands/implement.md` to invoke `read-tasks`, `derive-boundary`, `check-stuck`, `mark-task`, `set-status`, `gate-confirm`, and an `<!-- llm:writeCode -->` marker on the per-task work step.
- [x] `parity: { strict-fields: ["task-checkbox-state"], strict-files: ["specs/.../tasks.md"], semantic-fields: ["code-edits[].content"] }`.
- [x] Fixture under `runtime/tests/fixtures/implement-basic/` with a feature ready for implementation (status: planned, one task pending).
- [x] The fixture exercises the write-boundary check: a malicious `writeCode` response that edits a file outside the boundary is rejected.
- **Done when**: parseability + integration + parity green; out-of-boundary rejection test passes; tool-coverage lint green.

## 16. Rewrite `/gov:plan`

- [x] Rewrite `framework/commands/plan.md` to invoke `read-spec`, `set-status`, `lint-markdown`, `gate-confirm`, and `<!-- llm:writeSpecBody -->` markers on the plan-creation step (one per plan section to fill: Technical Decisions, Affected Files, Trade-offs).
- [x] `parity: { strict-fields: ["status-transition"], semantic-fields: ["plan-body"] }`.
- [x] Fixture under `runtime/tests/fixtures/plan-basic/` with a clarified spec.
- **Done when**: parseability + integration + parity green; the status transition `clarified → planned` is strict-equal; tool-coverage lint green.

## 17. Rewrite `/gov:specify`

- [x] Rewrite `framework/commands/specify.md` to invoke `lint-markdown`, `gate-confirm`, and `<!-- llm:writeSpecBody -->` markers on the new-feature-spec creation step.
- [x] `parity: { strict-fields: ["frontmatter"], strict-files: ["specs/<NNN>-<slug>/spec.md (path-only)"], semantic-fields: ["spec-body"] }`.
- [x] Fixture under `runtime/tests/fixtures/specify-basic/` with an empty `specs/` directory.
- **Done when**: parseability + integration + parity green; the new feature directory is created at the right `NNN-slug` path with valid frontmatter; tool-coverage lint green.

## 18. Populate `framework/runtime-tools.txt`

- [x] Replace the file body with the 14 MCP tool names from the plan's manifest section, one per line, preserving the comment header from spec 021.
- [x] Verify `scripts/lint-tool-coverage.sh` exits 0 after the rewrites in tasks 12-17 (every reference in the six rewritten commands is paired with a fallback marker within 20 lines).
- [x] Verify spec 021's CI check (a) — `command -v <name>` returns non-zero for each entry — still passes; none of the 14 names should collide with any real binary on a stock Ubuntu runner.
- **Done when**: the manifest matches the plan; `scripts/lint-tool-coverage.sh` exits 0.

## 19. Create `.github/workflows/runtime.yml`

- [x] Workflow `runtime` with `paths` filter on `runtime/**` and `framework/commands/*.md`. Triggers on `pull_request` and `push` to `main`.
- [x] Single job on `ubuntu-latest`: checkout, install Rust toolchain (`dtolnay/rust-toolchain@stable`), `cargo build --release`, `cargo test --release`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- [x] Cache cargo registry and target directory via `actions/cache` keyed on `Cargo.lock`.
- **Done when**: workflow file passes `actionlint`; pushing a PR triggers the job and it runs to completion locally via `act` or in real CI.

## 20. Create `.github/workflows/runtime-release.yml`

- [x] Tag-triggered workflow on `runtime-v*`. Matrix across target triples: `aarch64-apple-darwin`, `x86_64-apple-darwin` (on `macos-latest`), `x86_64-unknown-linux-gnu` (on `ubuntu-latest`), `aarch64-unknown-linux-gnu` (cross-compiled via `cargo-zigbuild` on `ubuntu-latest`), and `x86_64-pc-windows-msvc` (on `windows-latest`) — Windows entry is best-effort per the spec's resolved Distribution channels question.
- [x] Each matrix entry: build, strip, tarball or zip with the binary plus a `sha256sum` file, upload as a release asset via `softprops/action-gh-release`.
- [x] Workflow includes a smoke test step on each platform: the built binary is invoked with `--version` after build to catch obvious link-time failures.
- [x] Manually push the first tag `runtime-v0.1.0` (or whatever version the `Cargo.toml` declares) once the workflow file lands; verify all matrix legs produce artifacts.
- **Done when**: a tag push produces a GitHub release with the six (or five, if Windows defers) tarballs/zips and checksums; each release asset's binary runs `--version` cleanly on its target platform.

> Confirmed 2026-05-12: tag `runtime-v0.1.0` pushed; all 5 matrix legs green (aarch64/x86_64 macOS, x86_64/aarch64 Linux, x86_64 Windows). Release at <https://github.com/stonean/govern/releases/tag/runtime-v0.1.0> ships 5 archives + 5 sha256 sidecars. The release workflow's per-platform `--version` smoke test ran inside each matrix job before the upload step.

## 21. Add the Runtime section to root `README.md`

- [x] After the "Feature Specs" section and before the closing material, add a `## Runtime` section: one paragraph of rationale (opt-in, faster slash commands, markdown-only path still works), a fenced bash block with install instructions (curl against the GitHub release artifact URL pattern, with sha256 verification), and a "When to install" paragraph (recommended for adopters who run slash commands frequently; skip if usage is occasional).
- [x] No edit to `framework/templates/project/project-readme.md` — the install surface is this repo's README, not the adopted project's.
- **Done when**: README renders cleanly; the install snippet runs end-to-end against a real release artifact (validated after task 20).

## 22. Add the bootstrap completion-message pointer

- [x] Edit `framework/bootstrap/govern.md` in both completion-message blocks (first-run, lines ~750-763, and update-mode, the parallel block below). Append one line to the "Next steps" list pointing readers at the README Runtime section: "Optional: install the deterministic runtime for faster slash commands — see [Runtime](https://github.com/<owner>/<repo>#runtime)."
- [x] No detect-and-warn anywhere else — spot-check that no slash command source references the runtime binary path or checks `command -v` for any name in `framework/runtime-tools.txt`.
- **Done when**: bootstrap output includes the line in both modes; no slash command source nags about missing runtime.

## 23. Cross-spec impact sweep

- [x] Re-read inline links in `spec.md`, `plan.md`, and `data-model.md` and confirm whether any sibling spec needs an update. The expected references are: 020-code-review (motivating evidence only), 021-runtime-boundary (constitutional precondition, no update needed because 021 already commits to a forward reference here). The constitution's §runtime-boundary subsection is referenced read-only; this spec does not introduce a constitution amendment.
- [x] If any §cross-spec-impact rule fires, record the affected change in the sibling spec with a back-link to this spec before proceeding.
- **Done when**: confirmed in writing that no §cross-spec-impact action was triggered, or each triggered change is recorded in its target spec.

> Confirmed 2026-05-11: no `§cross-spec-impact` action triggered. 020 and 021 are referenced read-only; no sibling spec body cites 022. Constitution `§runtime-boundary` anchor resolves at line 400.

## 24. Run `/gov:analyze` against this spec and fix findings

- [x] Run `/gov:analyze` targeted at `022-deterministic-runtime` and resolve any hard-fail or blocking findings on spec, plan, tasks, and data-model files.
- [x] Confirm anchor resolution: `§runtime-boundary` references in this spec resolve to the marker in `framework/constitution.md`.
- [x] Confirm dependency status: `021-runtime-boundary` is `done`.
- **Done when**: `/gov:analyze` reports no hard-fail and no blocking findings.

> Confirmed 2026-05-11: validate-frontmatter clean; traverse-deps compatible (021 at `done`); `§runtime-boundary` resolves to 2 markers in `framework/constitution.md`; gen-spec-deps reports no drift; markdownlint-cli2 over `specs/022-deterministic-runtime/` reports 0 errors. Advisory anchor mismatches in `spec.md` (`§LLM`, `§The`, `§runtime-boundary`, `§text-first-artifacts`) are pre-existing cross-file or multi-word references the primitive's regex doesn't span; they are advisory, not blocking.

## 25. Run `npx markdownlint-cli2` and final sweep

- [x] Lint all rewritten command files under `framework/commands/`, all spec files under `specs/022-deterministic-runtime/`, the root `README.md`, and `framework/bootstrap/govern.md`.
- [x] Verify the existing `scripts/lint-frontmatter.sh` and `scripts/lint-tool-coverage.sh` still exit 0.
- [x] Run the full `markdown-only-pipeline.yml` workflow locally (manually executing the bash steps); confirm steps (a)–(f) all pass with the runtime binary not on `PATH` (a workflow-local build exists for step (f) only).
- **Done when**: every lint exits 0; the markdown-only workflow is green; this spec is ready to advance to `done`.

> Confirmed 2026-05-11: markdownlint-cli2 on the 022 spec dir + README.md + bootstrap/govern.md reports 0 errors. lint-frontmatter, lint-tool-coverage, lint-procedure-parseability, gen-spec-deps --dry-run, gen-readme-table --dry-run, gen-help-tables --dry-run all exit 0. Workflow steps (a)–(f) execute green locally.

## 26. Implement scenario: govern-bootstrap

- [x] 26.1 Add `fetch-archive` primitive — download a URL to a local path, fetch its sha256 sidecar, verify the hash; pure-Rust function with unit tests for the verification helper. Adds `reqwest` (blocking, rustls-tls) and `sha2` deps.
- [x] 26.2 Add `extract-archive` primitive — untar/unzip a local archive into a staging directory; tar.gz on Unix and zip everywhere. Adds the `zip` crate (in-process; no shell-out).
- [x] 26.3 Add `substitute-templates` primitive — walk a staging tree, apply a `{key}` → value substitution map, write to a target tree; unit test on a small staging tree.
- [x] 26.4 Add `merge-claude-md` primitive — idempotent block insert/update; unit tests for first-run, update-mode, and no-op cases.
- [x] 26.5 Extend `gvrn exec`'s command resolution to also look at `framework/bootstrap/<name>.md` after the existing two candidates; integration test in `runtime/tests/exec_subprocess.rs`.
- [x] 26.6 Wire the four new primitives into the walker dispatcher and the MCP server tool list; update `framework/runtime-tools.txt`.
- [x] 26.7 Rewrite `framework/bootstrap/govern.md` Instructions section under the parseable conventions, keeping the existing prose as a `## Markdown-only reference` block; add `parity:` frontmatter.
- [x] 26.8 Create fixture `runtime/tests/fixtures/govern-basic/` (adopter-project skeleton plus a tiny archive asset) and the golden JSONL stream; add the parity test case.

  > Scope adjusted 2026-05-12: parity-test coverage of the full bootstrap procedure requires mock-HTTP infrastructure inside the parity harness (fetch-archive needs an HTTP server). That mock layer is deferred. End-to-end coverage of the back half (extract → substitute → merge sharing context) ships as `exec_chains_bootstrap_primitives_extract_substitute_merge` in `runtime/tests/exec_subprocess.rs`. The full govern-basic parity fixture remains a follow-up once mock-HTTP support lands.
  >
  > Resolved 2026-05-12 (post-/gov:review): mock-HTTP support landed in `runtime/tests/parity.rs` (a minimal `MockHttp` server binds to 127.0.0.1:0 and serves the test-time-built tarball + sidecar on dynamic routes; the harness substitutes `{MOCK_HTTP}` in the staged session JSON with the server URL before launch). The `govern-basic` fixture under `runtime/tests/fixtures/govern-basic/` now exercises `/install` (a fixture-local stand-in for the production `/govern` procedure) end-to-end through all four bootstrap primitives plus the gate-confirm, and the golden + parity-capture artifacts ship alongside the other six commands.
- [x] 26.9 Add CHANGELOG entry; bump `gvrn` to 0.2.0; re-run every lint (cargo test, clippy, fmt, lint-procedure-parseability, lint-tool-coverage, markdownlint).

  > Confirmed 2026-05-12: gvrn 0.2.0 builds; cargo test reports 172 OK across all targets; clippy --all-targets -- -D warnings clean; cargo fmt --check clean; lint-procedure-parseability, lint-tool-coverage, lint-frontmatter all exit 0; markdownlint-cli2 on the 022 spec dir + CHANGELOG + bootstrap reports 0 errors.
- **Done when**: the scenario's described behavior is correctly implemented and tested.

## 27. Implement scenario: apply-manifest

- [x] 27.1 Add `apply-manifest` primitive — `ManifestEntry { source, dest, strategy: "update"|"create"|"skip-if-conflict", keep-literals: Option<Vec<String>> }` plus `ManifestEntryResult` enum (`created` / `updated` / `unchanged` / `skipped-exists` / `skipped-pinned` / `source-missing`). Pure-Rust `run()` resolves sources against `source-root`, applies the per-entry strategy with pinned-exemption short-circuit and per-entry `keep-literals` masking of the substitutions map, returns aggregate counts. Unit tests cover each strategy, the pinned path, keep-literals on a govern.md-style entry, and the source-missing branch.
- [x] 27.2 Add `enforce-manifest` primitive — `directory: String`, `expected: Vec<String>`, `pinned: Vec<String>`, `recursive: bool` (default false), `glob-include: Option<String>` (default `*.md`). `run()` walks the directory, removes files not in `expected` and not pinned, returns `removed` / `kept` / `pinned-kept` lists. Unit tests: top-level cleanup, recursive cleanup, pinned exemption, missing directory (zero-removal success), non-default glob.
- [x] 27.3 Refactor `merge-claude-md` into `merge-managed-block` — add `marker-style: "html-comment" | "line-prefix"` (default `html-comment`); extract the BEGIN/END merge logic into a marker-style-aware shared core. `line-prefix` style: single `# {marker}` line preamble followed by the block, terminated by a blank line or EOF. The `merge-claude-md` primitive becomes a thin compat shim that delegates with `marker-style: html-comment` and `marker: govern-managed`. All existing `merge-claude-md` unit tests, parity fixtures, and goldens keep passing unchanged. New unit tests cover the line-prefix style with `.gitignore`-shaped fixtures.
- [x] 27.4 Wire the three new primitives — add to `parser::PRIMITIVE_NAMES`, the walker's `dispatch_primitive` match, the MCP `TOOL_NAMES` list with per-primitive `#[tool]` handlers, and `framework/runtime-tools.txt`. Verify `scripts/lint-tool-coverage.sh` exits 0 (the existing fallback markers in `framework/bootstrap/govern.md` extend to the new tool names once the procedure rewrite in 27.5 lands).
- [x] 27.5 Rewrite `framework/bootstrap/govern.md` Instructions section to use the new primitives — six primitive calls (`fetch-archive` → `extract-archive` → `apply-manifest` → `merge-managed-block` for `.gitignore` → `enforce-manifest` → `apply-manifest` with `keep-literals` for the govern.md self-install) plus two prose steps (context note, completion message) plus the gate-confirm for the install approval. Update the `(MCP: gov-rt:*)` bridge annotations on every new primitive reference. Drop the host-side bash walker guidance from the markdown-only reference (the walker is no longer needed; the markdown-only path now describes the same six logical steps as host-driven file operations).
- [x] 27.6 Extend the `govern-basic` parity fixture — grow the `mock-http/staging/` tree to include files exercising every strategy (one `update`, one `create`, one `skip-if-conflict`, one pinned, one keep-literals govern.md analog) plus a directory the `enforce-manifest` step cleans up. Update `runtime/tests/fixtures/govern-basic/.claude/gov-session.json` to seed the new manifest entries. Regenerate `runtime/tests/golden/govern-basic.jsonl` against the rewritten procedure (the envelope sequence grows by the three new primitive dispatches).
- [x] 27.7 Add CHANGELOG entry; bump `gvrn` to 0.3.0 (additive primitives + `merge-claude-md` becomes a compat shim — same minor-bump convention as 0.1 → 0.2); re-run every lint (cargo test, clippy --all-targets -- -D warnings, fmt --check, lint-procedure-parseability, lint-tool-coverage, lint-frontmatter, markdownlint-cli2 over the 022 spec dir + CHANGELOG + bootstrap).

  > Confirmed 2026-05-12: gvrn 0.3.0 builds; cargo test reports 220 OK across all targets (187 lib + 3 atomic_writes + 5 exec_subprocess + 15 mcp + 9 parity + 2 walker); clippy --all-targets --release -- -D warnings clean; cargo fmt --check clean; lint-procedure-parseability, lint-tool-coverage, lint-frontmatter all exit 0; markdownlint-cli2 on the 022 spec dir + CHANGELOG + bootstrap/govern.md reports 0 errors.
- [x] 27.8 Tag-push `gvrn-v0.3.0` (triggers the release workflow's 5-leg matrix); after all matrix legs report success, `cargo publish` from `runtime/` to upload `gvrn 0.3.0` to crates.io. Both steps require user authorization (externally visible).

  > Confirmed 2026-05-12: tag `gvrn-v0.3.0` pushed; release run 25739997145 reports all 5 matrix legs green (aarch64/x86_64 macOS, x86_64/aarch64 Linux, x86_64 Windows). Release at <https://github.com/stonean/govern/releases/tag/gvrn-v0.3.0> ships 5 archives + 5 sha256 sidecars. `cargo publish` from `runtime/` uploaded `gvrn 0.3.0` to crates.io.
- **Done when**: the scenario's described behavior is correctly implemented and tested; `gvrn-v0.3.0` is live on GitHub releases and crates.io; `/govern` against a real adopter project drives the full bootstrap through `gov-rt:*` MCP tools with no host-generated bash walker observed.

## 28. Implement scenario: ask-consolidation

Adds two new primitives — `create-scenario` and `append-task` — that the `/ask` scenario branch (introduced in spec [023 — `govern` Refinement](../023-govern-refinement/spec.md)) invokes when classifying an input as a scenario.

- [x] 28.1 Add `create-scenario` primitive — args (`feature-path`, `slug`, `section`, `context`, `behavior`, optional `edge-cases`); resolves the scenario template at `framework/templates/spec/scenario.md`, substitutes the supplied values, writes `{feature-path}/scenarios/{slug}.md` atomically via tempfile-in-parent + `persist` rename. Creates the scenarios subdirectory if absent. Refuses on slug conflict with a clean operational error. Unit tests cover: happy path, scenarios directory absent, slug conflict, feature path absent, optional edge-cases omitted.
- [x] 28.2 Add `append-task` primitive — args (`feature-path`, `title`, `done-when`, optional `body`); reads existing `tasks.md` to compute next task number from `max(existing) + 1` (not `count + 1`); appends a new section block atomically. Creates `tasks.md` with a derived heading when absent. Unit tests cover: empty `tasks.md`, existing tasks (sequential numbering), skip-value numbering, missing `tasks.md`, atomic-write semantics on simulated crash mid-write.
- [x] 28.3 Wire the two new primitives — add to `parser::PRIMITIVE_NAMES`, the walker's `dispatch_primitive` match, the MCP `TOOL_NAMES` list with per-primitive `#[tool]` handlers, and `framework/runtime-tools.txt`. Verify `scripts/lint-tool-coverage.sh` exits 0. Verify the pre-commit run of `scripts/gen-configure-mcp.sh` (added in spec 023 task 1) flows the two new tool names into both `framework/bootstrap/configure/claude.md` and `framework/bootstrap/configure/auggie.md` in the same commit.
- [x] 28.4 Add CHANGELOG entry; bump `gvrn` to 0.4.0 (additive primitives — same minor-bump convention as the apply-manifest scenario); re-run every lint (`cargo test`, `clippy --all-targets -- -D warnings`, `fmt --check`, `lint-procedure-parseability`, `lint-tool-coverage`, `lint-frontmatter`, `markdownlint-cli2` over the 022 spec dir + CHANGELOG).
- [x] 28.5 Tag-push `gvrn-v0.4.0` (triggers the release workflow's 5-leg matrix); after all matrix legs report success, `cargo publish` from `runtime/` to upload `gvrn 0.4.0` to crates.io. Both steps require user authorization (externally visible).

  > Confirmed 2026-05-16: tag `gvrn-v0.4.0` pushed; release run 25962899207 reports all 5 matrix legs green (aarch64/x86_64 macOS, x86_64/aarch64 Linux, x86_64 Windows). Release at <https://github.com/stonean/govern/releases/tag/gvrn-v0.4.0> ships 5 archives + 5 sha256 sidecars. `cargo publish` from `runtime/` uploaded `gvrn 0.4.0` to crates.io.
- **Done when**: the scenario's described behavior is correctly implemented and tested; `gvrn-v0.4.0` is live on GitHub releases and crates.io; spec 023's Phase B can begin (the `framework/commands/ask.md` rewrite calls the new primitives).

## 29. Implement scenario: runtime-primitive-structural-bugs

- [x] - [ ] Implement the behavior described in [`scenarios/runtime-primitive-structural-bugs.md`](scenarios/runtime-primitive-structural-bugs.md).

- **Done when**: All four primitive bug fixes ship: `append-task` accepts an explicit `slug` argument and detects phased vs. flat tasks.md structure; `read-tasks` parses phased tasks.md correctly and returns the flattened list with phase metadata; `check-stuck` measures from the most recent `in-progress` transition, not the first. Each fix has fixture-based unit tests plus a parity-test entry; `gvrn` ships a new patch or minor version.

## 30. Implement scenario: check-stuck-tasks-md-advancement

- [x] Implement the behavior described in [`scenarios/check-stuck-tasks-md-advancement.md`](scenarios/check-stuck-tasks-md-advancement.md).

- **Done when**: `check-stuck`'s second condition is enforced — `stuck` only fires when both `commit_count >= threshold` AND the first incomplete subtask in `tasks.md` has not advanced across the walked commit window. New regression test in `runtime/src/primitives/check_stuck.rs::tests` asserts `stuck: false` when threshold-count commits flipped intervening checkboxes. `gvrn` ships a patch version bump.

## 31. Implement scenario: check-stuck-read-blob-reuse

- [x] Implement the behavior described in [`scenarios/check-stuck-read-blob-reuse.md`](scenarios/check-stuck-read-blob-reuse.md).

- **Done when**: `find_in_progress_commit` in `runtime/src/primitives/check_stuck.rs` uses the `read_blob_from_tree` helper instead of the inline `tree.get_path(...).find_blob(...).content()` chain. Existing `check_stuck` tests pass unchanged; no `gvrn` version bump required (REUSE-only, no behavior change).

## 32. Implement scenario: framework-list-dedup

- [x] Implement the behavior described in `scenarios/framework-list-dedup.md`

- **Done when**: `merge-permissions` ships as a new CLI subcommand and MCP tool with the canonical-presence + dedup contract described in the scenario, registered in `framework/runtime-tools.txt`. `merge-managed-block` grows cross-boundary dedup behavior gated on `marker-style: "line-prefix"` (canonical-block wins; html-comment callsites unchanged), with the envelope additions described. Both deliveries have unit tests covering the happy paths and edge cases. The scenario's described behavior is correctly implemented and tested.

## 33. Implement scenario: mark-task-backtick-headings

- [x] Implement the behavior described in `scenarios/mark-task-backtick-headings.md`

- **Done when**: `mark-task` and `read-tasks` recognize the same set of task headings on every `tasks.md`, including headings containing inline-code (backtick-quoted) spans. The shared `primitives::mod::parse_atx_heading` helper is the single source of truth for both primitives' heading parsing. A regression test exercises a heading like ``### N. Dedup `/configure` permission entries`` and asserts both `mark-task` and `read-tasks` resolve task `N` identically. No version bump beyond a patch (REUSE-only, no behavior change for `read-tasks`).

## 34. Implement scenario: writecode-payload-bundling

- [x] 34.1 Wire `writeCode.plan-relevant-files` — parse the targeted feature's `plan.md` Affected Files table, read each listed repo-relative file from disk, inline as `{path, content}` in the request payload. Files listed but absent from disk (planned-new files) are omitted, not errored. Unit tests for the Affected Files parser and the file-loader.
- [x] 34.2 Wire `writeCode.constitution-excerpts` — parse the running command file's `Reference: §<anchor>, §<anchor>` line under Scope Boundaries, resolve each anchor via the existing `resolve-anchor` primitive, inline each section body as a string. Command files with no `Reference:` line yield an empty array. Unit tests.
- [x] 34.3 Add read-side secret-exfiltration guard for `plan-relevant-files` — refuse files matching `.env`, `.env.*`, `*-secrets.*`, `credentials*`; respect `.gitignore`. Matched paths halt the procedure with a structured `secret-exfiltration-blocked` error envelope. Unit tests cover each pattern and a `.gitignore`-driven match.
- [x] 34.4 Reorder `WriteCodeRequest` struct fields to `constitution-excerpts`, `plan-relevant-files`, `write-boundary`, `task` so the stable prefix is contiguous and front. Existing round-trip tests stay green; add a serialization-order assertion to lock the new order.
- [x] 34.5 Wire `writeSpecBody.existing-content` — read the current section body from disk on re-runs of `/gov:specify` or `/gov:plan`, emit in the `existing-content` field. Empty sections emit `None`. Unit tests.
- [x] 34.6 Update spec 022's `## LLM extension points` section with the cache-breakpoint contract — one paragraph stating hosts SHOULD place a prompt-cache anchor between `write-boundary` and `task` in serialized `writeCode` request payloads. SHOULD, not MUST.
- [x] 34.7 Add parity tests under `runtime/tests/parity/` for `/gov:implement` and `/gov:plan` exercising the new bundled fields against fixtures with realistic plan tables, command `Reference:` lines, and `writeSpecBody` re-run states. Markdown-only walker and runtime walker produce equivalent state mutations.
- [x] 34.8 Add CHANGELOG entry; bump `gvrn` to 0.7.0 (feature-level addition); re-run every lint (`cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `lint-procedure-parseability`, `lint-tool-coverage`, `lint-frontmatter`, `markdownlint-cli2` over the 022 spec dir + CHANGELOG).
- [x] 34.9 Tag-push `gvrn-v0.7.0` (triggers the 5-leg release matrix); after all matrix legs report success, `cargo publish` from `runtime/` to upload `gvrn 0.7.0` to crates.io. Both steps require user authorization (externally visible).

- **Done when**: every subtask above is checked; the scenario's described behavior is correctly implemented and tested; `gvrn-v0.7.0` is live on GitHub releases and crates.io.

## 35. Implement scenario: writecode-payload-canonicalize-paths

- [x] Implement the behavior described in `scenarios/writecode-payload-canonicalize-paths.md`

- **Done when**: `load_plan_relevant_files` canonicalizes every candidate path and rejects out-of-repo escapes with a structured error envelope; `secret_pattern` matches case-insensitively on the basename; the scenario's five scenarios (relative escape, absolute escape, in-repo happy path, planned-new file, case-fold bypass) are covered by tests; `gvrn` ships a patch bump (`0.7.3`; `0.7.2` was already claimed by 027.5).

## 36. Implement scenario: dashboard-primitive

- [x] 36.1 Add `dashboard` primitive — args struct + result schema (per-spec `slug` / `status` / `dependencies` / `tags` / `open-question-count` / `has-plan` / `has-tasks` / `has-data-model` / `scenarios-count` / `blocked-by`, top-level `tags-union`, `config: {present, disabled-rule-files}`, optional `session-target` with `scenario-detail`). Pure-Rust `run()` walks `specs/` honoring the `NNN-feature` pattern, parses frontmatter via existing helpers, computes `blocked-by` from each spec's `dependencies` (a dep is "blocking" when its own status is below `clarified`), folds `tags-union` across every spec's `tags` array, reads `.govern.toml` for the `[[review.disabled-rule-files]]` section, and reads `.claude/gov-session.json` (plus the targeted scenario file when `scenario` is non-null) to populate `session-target`. Unit tests cover the happy path plus every edge case enumerated in the scenario: empty `specs/`, `NNN-feature` directory missing `spec.md` (operational error), non-pattern directory (skipped silently), `.govern.toml` absent / present-empty / parse-failure, `scenarios/` with non-markdown files, session file absent, session targeting a nonexistent feature.
- [x] 36.2 Wire `dashboard` as CLI subcommand + MCP tool — register in `parser::PRIMITIVE_NAMES`, walker `dispatch_primitive` match, MCP `TOOL_NAMES` with a `#[tool]` handler, and `framework/runtime-tools.txt`. Verify `scripts/lint-tool-coverage.sh` exits 0.
- [x] 36.3 Rewrite `framework/commands/status.md` — collapse to a single path that always invokes `dashboard` once; remove the short-circuit branch (steps 2.1 / 2.2); add a preamble line above the table that surfaces the targeted feature (and scenario, when present) plus its next action; update the §Instructions preamble to name `dashboard` as the deterministic target for the status command so the shell-utility ban has a positive callout. `scripts/lint-procedure-parseability.sh` and `scripts/lint-tool-coverage.sh` both exit 0.
- [x] 36.4 Refresh `runtime/tests/fixtures/status-basic/` to multi-spec coverage exercising `blocked-by` / `tags-union` / `.govern.toml` aggregation and a scenario-targeted session; regenerate `runtime/tests/golden/status-basic.jsonl` and `runtime/tests/parity/status/expected.txt` against the rewritten procedure. Parity test green.
- [x] 36.5 Add CHANGELOG entry; bump `gvrn` to 0.8.0; re-run every lint (`cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `lint-procedure-parseability`, `lint-tool-coverage`, `lint-frontmatter`, `markdownlint-cli2` over the 022 spec dir + CHANGELOG + `framework/commands/status.md`).
- [x] 36.6 Tag-push `gvrn-v0.8.0` (triggers the 5-leg release matrix); after all matrix legs report success, `cargo publish` from `runtime/` to upload `gvrn 0.8.0` to crates.io. Both steps require user authorization (externally visible).

- **Done when**: every subtask above is checked; the scenario's described behavior is correctly implemented and tested; `gvrn-v0.8.0` is live on GitHub releases and crates.io.

## 37. Implement the write-session primitive

- [x] Implement the behavior described in `scenarios/write-session-primitive.md`

- **Done when**: the scenario's described behavior is correctly implemented and tested.

## 38. Implement scenario: traverse-deps-cycle-check

- [x] Implement the behavior described in `scenarios/traverse-deps-cycle-check.md`

- **Done when**: the scenario's described behavior is correctly implemented and tested. `traverse-deps` detects cycles in the dep graph it walks and emits a blocking finding naming the SCC(s); `/anvil:analyze` surfaces the finding and fails its gate; parity tests under `runtime/tests/parity/` cover the cycle-detection path against a 2-cycle fixture and the existing acyclic happy path stays green; coordinates with sibling [detect-dependency-cycles](../017-derive-dont-ask/scenarios/detect-dependency-cycles.md) as defense-in-depth (this fires when the upstream generator-side check was bypassed or the adopter is on an older shipped script).

## 39. Implement scenario: merge-managed-block-multi-subsection-end

- [x] Implement the behavior described in `scenarios/merge-managed-block-multi-subsection-end.md`

- **Done when**: `merge-managed-block` (line-prefix style) identifies the existing on-disk canonical block by line count rather than the next-blank heuristic, so multi-subsection canonicals (e.g., the shipped `.gitignore` template) reach `unchanged` on stable reruns and update cleanly without leaving orphan subsection-header tails. Two new unit tests cover the multi-subsection update path: a stable-rerun test asserting `action == "unchanged"` / `dedup_removed == 0` / mtime preserved, and a body-changed test asserting clean replacement with no duplicated tail. All existing `merge_managed_block::tests` pass unchanged.
