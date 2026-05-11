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

## 14. Rewrite `/gov:validate`

- [x] Rewrite `framework/commands/validate.md` to invoke the mechanical primitives (`validate-frontmatter`, `resolve-anchor`, `traverse-deps`, `check-rule-ids`, `run-generator`, `lint-markdown`) for the deterministic checks, and an `<!-- llm:assessSpecQuality -->` marker on the per-rule Verification step.
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
- [ ] Manually push the first tag `runtime-v0.1.0` (or whatever version the `Cargo.toml` declares) once the workflow file lands; verify all matrix legs produce artifacts.
- **Done when**: a tag push produces a GitHub release with the six (or five, if Windows defers) tarballs/zips and checksums; each release asset's binary runs `--version` cleanly on its target platform.

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

## 24. Run `/gov:validate` against this spec and fix findings

- [ ] Run `/gov:validate` targeted at `022-deterministic-runtime` and resolve any hard-fail or blocking findings on spec, plan, tasks, and data-model files.
- [ ] Confirm anchor resolution: `§runtime-boundary` references in this spec resolve to the marker in `framework/constitution.md`.
- [ ] Confirm dependency status: `021-runtime-boundary` is `done`.
- **Done when**: `/gov:validate` reports no hard-fail and no blocking findings.

## 25. Run `npx markdownlint-cli2` and final sweep

- [x] Lint all rewritten command files under `framework/commands/`, all spec files under `specs/022-deterministic-runtime/`, the root `README.md`, and `framework/bootstrap/govern.md`.
- [x] Verify the existing `scripts/lint-frontmatter.sh` and `scripts/lint-tool-coverage.sh` still exit 0.
- [x] Run the full `markdown-only-pipeline.yml` workflow locally (manually executing the bash steps); confirm steps (a)–(f) all pass with the runtime binary not on `PATH` (a workflow-local build exists for step (f) only).
- **Done when**: every lint exits 0; the markdown-only workflow is green; this spec is ready to advance to `done`.

> Confirmed 2026-05-11: markdownlint-cli2 on the 022 spec dir + README.md + bootstrap/govern.md reports 0 errors. lint-frontmatter, lint-tool-coverage, lint-procedure-parseability, gen-spec-deps --dry-run, gen-readme-table --dry-run, gen-help-tables --dry-run all exit 0. Workflow steps (a)–(f) execute green locally.
