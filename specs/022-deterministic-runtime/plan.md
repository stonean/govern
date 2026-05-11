# 022 — Deterministic Runtime Plan

Implements [022 — Deterministic Runtime](spec.md).

## Overview

This spec ships a substantial new artifact: a Rust binary called `runtime` that lives in this repo, plus six rewritten slash command files and two new CI workflows. The spec body already settles the largest design questions in its Resolved Questions section (language, surfaces, ABI, distribution, install policy, MCP naming convention, partial-failure semantics, state management). This plan's job is to make the remaining implementation-concrete choices: project layout, dependency selection, parser strategy, test-fixture shape, ordering, and release engineering. Roughly:

1. **A new Rust crate at `runtime/`** with `clap`-based CLI exposing two top-level subcommands (`runtime mcp`, `runtime exec`), plus per-primitive subcommands so each primitive is invokable standalone.
2. **Thirteen primitive operations** implemented in-process, each with a deterministic core and a thin CLI/MCP/JSON-protocol wrapper. State-modifying primitives use tempfile-plus-rename atomicity.
3. **A markdown procedure parser** for slash command Instructions sections, hand-walked over `pulldown-cmark`'s event stream. The parser produces a typed AST the interpreter walks.
4. **An interpreter** that walks the AST, calls primitives in-process, and round-trips JSON messages over stdio for LLM extension points and gates.
5. **Three extension points** (`assessSpecQuality`, `writeCode`, `writeSpecBody`) with versioned request/response schemas defined in `data-model.md` and serialized via `serde`.
6. **Six rewritten slash command files** under `framework/commands/`, adopting the parseable conventions.
7. **Two new CI workflows**: `runtime.yml` (per-PR build + test on changes to runtime source paths) and a release workflow triggered by tags that cross-compiles and uploads artifacts.
8. **Parseability check** added to the existing `markdown-only-pipeline.yml` — a bash invocation of `runtime parse --check` against every `framework/commands/*.md` file. The check tolerates legacy prose (the three not-yet-rewritten commands) so the workflow doesn't fail on them.
9. **Distribution + discovery**: GitHub release artifacts; README Runtime section; one-line pointer added to the `/govern` bootstrap completion message. No auto-install, no per-invocation install nags.

Implementation proceeds bottom-up: primitives → MCP surface → parser → interpreter → slash command rewrites → CI/distribution. Each layer is testable in isolation before the one above it depends on it.

## Technical Decisions

### Project layout: single Rust crate at `runtime/`

The runtime is a single binary crate at `runtime/Cargo.toml` (sibling to `framework/`, `specs/`, `scripts/`). No workspace, no library/binary split — the binary is the surface, and the primitives and parser are internal modules `lib`-shaped only inasmuch as integration tests need to call them. The crate's targets:

- `runtime/Cargo.toml` — package manifest pinned to a stable Rust edition (2024) and an MSRV declared in the manifest's `rust-version` field.
- `runtime/src/main.rs` — `clap`-derive CLI entrypoint, dispatching to subcommand handlers.
- `runtime/src/lib.rs` — re-exports the modules below so integration tests under `runtime/tests/` can call into them.
- `runtime/src/parser/` — procedure parser.
- `runtime/src/interpreter/` — procedure walker + JSON protocol I/O.
- `runtime/src/primitives/` — one module per primitive, each exposing a pure-Rust function plus `clap`-derive args for the CLI surface.
- `runtime/src/mcp/` — MCP server wiring (uses `rmcp`).
- `runtime/src/schema/` — request/response schemas for primitives, JSON protocol messages, and extension points. Generated `JSONSchema` representations live alongside.
- `runtime/tests/` — integration tests against fixture repos under `runtime/tests/fixtures/`.
- `runtime/CHANGELOG.md` — runtime release notes, maintained in lockstep with framework releases per §runtime-boundary.

`runtime/` is added to the existing `.gitignore`-level inclusion (no exclusion) and shipped as part of the repo. The path `runtime/target/` is added to the root `.gitignore`.

### Dependency selection

Each pick has a one-line rationale; cargo-version pins go into `Cargo.toml`.

- `clap` (derive feature) — CLI parsing. Industry standard, good UX, derive-based ergonomics fit the per-primitive subcommand pattern.
- `serde`, `serde_json` — JSON protocol messages and primitive payloads.
- `tokio` (rt-multi-thread, io-util, signal features) — async runtime required by `rmcp`. The non-MCP subprocess interpreter path stays mostly synchronous but shares the runtime to avoid mode-switching.
- `rmcp` — Anthropic's reference MCP SDK. Mandated by the spec's resolution on implementation language.
- `pulldown-cmark` — markdown event parser. The procedure parser walks its event stream rather than re-tokenizing markdown by hand. Chosen for correctness on edge cases (escapes, nested lists, fenced blocks) and to avoid maintaining a markdown lexer.
- `tempfile` — atomic-write tempfile creation in the target file's parent directory (so the subsequent rename stays on the same filesystem).
- `thiserror` — typed error definitions per primitive and interpreter surface.
- `anyhow` — `main()` error bubbling only; not used in primitive APIs.
- `regex` — limited use in lints (e.g., anchor reference matching). The bulk of "matching" is structural via `pulldown-cmark` events.
- `serde_yaml` — frontmatter parsing in `read-spec` and `validate-frontmatter`. Chosen over hand-rolled because frontmatter is real YAML.
- `walkdir` — `derive-boundary` and lint primitives need recursive directory walks.
- `git2` — `derive-boundary` and `check-stuck` need access to git history. Pure-Rust libgit2 binding; no shell-out to `git`, which would couple the runtime to system git on PATH.

No `tracing`, no `log`, no async logging crates. Human-readable stderr output uses `eprintln!`; structured `progress` JSON messages over stdout are emitted by the interpreter directly. Observability is a single function in `src/io.rs` that writes one JSON object per line. Keeping the dependency surface tight matters because every new crate is supply-chain surface for a tool that runs in every adopter's environment.

### Parser strategy: pulldown-cmark events + hand-walked structural recognizer

The procedure parser reads a markdown file with `pulldown-cmark`, walks its event stream, and emits a typed AST defined in `data-model.md`. Two reasons for this layering:

1. `pulldown-cmark` handles the surface lexing (code spans, fenced blocks, lists, headings, HTML blocks) correctly. Hand-rolling that is bug-prone.
2. The structural conventions (numbered steps, backtick-quoted primitive names, HTML-comment extension-point markers) are recognized at the event level by a small state machine. This is what the parser owns.

The parser exposes two public entry points: `parse(source: &str) -> Result<Procedure, ParseError>` and `check(source: &str) -> Result<(), ParseError>` — the latter is what the CI parseability check invokes. Parse errors carry line/column ranges (`pulldown-cmark` provides byte offsets; the parser converts to line/col on demand).

**Backward-compatibility provision**: when an Instructions section does not match the parseable conventions, `parse` returns `Err(ParseError::LegacyProse)`. The CI check treats this variant as a non-failure for the three legacy commands (`/gov:clarify`, `/gov:review`, `/gov:groom`) by maintaining an allowlist file `runtime/legacy-prose-commands.txt`. Once each legacy command is rewritten (in its scenario), it is removed from the allowlist. The allowlist is also the discoverable surface that lists what hasn't been ported yet.

### Interpreter: synchronous walk + stdio JSON for asynchronous seams

The interpreter is a synchronous walker over the parsed AST. When it reaches a primitive step, it calls the primitive in-process and gets back a result. When it reaches an LLM extension point or a gate-confirm, it serializes a JSON request to stdout, blocks reading stdin one line at a time until it gets a parseable response, then resumes. No tasks, no futures, no concurrency in the walker itself — the surface is request/response.

The async runtime (tokio) is started for the MCP surface; the subprocess interpreter runs on `block_on` of a small main-task that drives the synchronous walker. This keeps both paths in the same binary with a single async-init cost.

### JSON-over-stdio framing: line-delimited JSON, one message per line

Newline-delimited JSON (`\n`-terminated). Each line is one complete JSON object. The protocol uses an envelope with a `type` discriminator (`llm-request`, `llm-response`, `gate-confirm`, `gate-response`, `progress`, `complete`, `error`) per the spec body. The set of message types is closed; adding one requires a versioned protocol bump per §runtime-boundary's lockstep-versioning rule. Stderr is human-readable text (no JSON), reserved for logs the host may surface to its own log streams.

The interpreter `panic`s if it reads a malformed JSON line on stdin — that is a host-implementation bug, not a recoverable runtime condition.

### MCP surface: one tool per primitive, no orchestration tools

`runtime mcp` starts an `rmcp` server exposing one tool per primitive in §The primitive library, with tool names following the `gov-rt:<verb>-<noun>` convention from the resolved-questions section. Tool input schemas are derived from each primitive's args struct via `schemars`. No orchestration tools (`run-procedure`, `walk-tasks`, etc.) are exposed via MCP — orchestration is the subprocess interpreter's job, not the MCP server's. This keeps the MCP surface flat and stable.

### Atomic writes: tempfile in same directory, then `std::fs::rename`

State-modifying primitives (`mark-task`, `mark-criterion`, `set-status`) write to a tempfile created via `tempfile::NamedTempFile::new_in(parent_dir)`. The tempfile is written, flushed, fsynced, then renamed over the target with `tempfile::NamedTempFile::persist`. POSIX rename is atomic within the same filesystem; placing the tempfile in the same directory guarantees that property. Windows rename semantics are weaker but acceptable for this use case — adopters running on Windows accept the small atomicity gap; partial-write recovery via `git checkout` is documented in the README Runtime section.

### CLI surface shape

`clap` derive-based, with subcommands flat under the top-level binary:

```text
runtime mcp                          # start MCP server (long-running)
runtime exec <command> [args...]     # subprocess interpreter
runtime parse <file>                 # parser (debug/manual use)
runtime parse --check <file>         # parseability check (CI)
runtime <primitive-name> [args...]   # invoke a primitive standalone
runtime --version                    # build-time version, from CARGO_PKG_VERSION
runtime --help                       # rendered by clap
```

Primitive subcommand names match their MCP tool names without the `gov-rt:` prefix (e.g., `runtime read-spec --feature 022-deterministic-runtime`). This makes the CLI surface a 1:1 mirror of the MCP surface, which is what `framework/runtime-tools.txt` ultimately enumerates.

### Versioning: built-in from `Cargo.toml`

`runtime --version` reads `env!("CARGO_PKG_VERSION")` baked at compile time. Every `error` JSON message includes the runtime version so adopters reporting issues can attach it without an extra invocation. Per the resolved-questions section, there is no startup framework-version comparison; parse failures and primitive errors carry version metadata so mismatch is diagnosable when it arises.

### Fixture-based integration testing

Integration tests live under `runtime/tests/` and exercise the runtime against fixture repos checked into `runtime/tests/fixtures/<name>/`. Each fixture is a minimal git-tracked directory tree resembling a real `govern`-adopting project: a `specs/<feature>/` with spec.md, plan.md, tasks.md as appropriate; a `framework/constitution.md`; a `.claude/gov-session.json`. Fixtures are committed real files — no setup scripts — so they can be inspected and diffed by reviewers.

Each fixture exercises one slash command end-to-end. The test asserts:

1. The runtime's output stream matches an expected sequence of JSON messages (golden file under `runtime/tests/golden/<fixture>.jsonl`).
2. The fixture's on-disk state after the run matches a captured "after" tree (`runtime/tests/expected/<fixture>/`).

Updating goldens uses a `RUNTIME_TEST_UPDATE_GOLDEN=1` env var that rewrites the expected files. CI runs without the env var; failures show a diff in the test output.

The fixture repo is the integration substrate. Per-primitive unit tests under each `src/primitives/<name>.rs` cover narrower behaviors (frontmatter shapes, anchor mismatches, atomic-write crash points via interrupted writes).

### Per-command parity testing: LLM-output equivalence is approximate, not byte-equal

The spec's acceptance criterion "produces output consistent with the LLM-driven path against the same fixture, within the determinism bounds defined for each command" requires per-command "determinism bounds." For each of the six commands, the parity bound is stated in the rewritten command's frontmatter (a new `parity:` field with sub-keys):

- **`/gov:status`** — strict byte-equality on the dashboard output.
- **`/gov:target`** — strict byte-equality on `.claude/gov-session.json` after the run.
- **`/gov:validate`** — set-equality on the list of findings (each finding's rule ID, severity, file, line), but not on the per-finding prose (semantic extension point varies wording).
- **`/gov:implement`** — set-equality on the set of files modified per task; checkbox state strict-equal; code content is the LLM extension point's responsibility, not the runtime's.
- **`/gov:plan`** — frontmatter status transition strict-equal; plan/tasks body content is semantic (extension point).
- **`/gov:specify`** — new feature directory created at the right path; spec frontmatter strict-equal; spec body is semantic.

The `parity:` frontmatter field is read by the parity-test harness in `runtime/tests/parity.rs`. The LLM-driven path is recorded once (manually, by a maintainer running the legacy prose-walked command and capturing output) and committed under `runtime/tests/parity/<command>/` as the "expected LLM output." Subsequent test runs compare the runtime's output against that capture under the declared bound.

This approach makes parity testing concrete and reviewable without requiring CI to invoke an LLM (the parity captures are committed, not regenerated).

### Slash command rewrites: order and rewrite shape

Order matches the spec's resolved Initial Scope:

1. `/gov:status` first — smallest surface, 100% deterministic, validates the architecture end-to-end.
2. `/gov:target` next — exercises session-file write and constitution loading.
3. `/gov:validate` — first command with a real extension point (`assessSpecQuality`).
4. `/gov:implement` — largest behavioral surface, exercises `writeCode` extension point and atomic write primitives in anger.
5. `/gov:plan` — exercises `writeSpecBody`.
6. `/gov:specify` — second `writeSpecBody` user; validates the extension point against two callers.

Each rewrite follows the spec's Per-rewrite checklist verbatim and adds nothing new beyond it. Rewrites do not change command behavior or argument shape; the runtime executes the same procedure the LLM walked before.

### Slash command frontmatter: add `parity:` and `runtime:` fields

Two new optional frontmatter fields:

- `runtime: <minimum-version>` — if set, the runtime refuses to execute the command at a lower version. This is the only enforcement mechanism for runtime/framework version skew; spec resolution covers the broader policy.
- `parity: { strict-files?: [...], strict-stdout?: bool, semantic-fields?: [...] }` — per the parity testing section above.

Both fields are unknown to the markdown-only path (the LLM ignores fields it doesn't recognize). They are unknown to spec 021's parseability convention (which only constrains the Instructions section). No constitution edit needed.

### Parseability check: new step in `markdown-only-pipeline.yml`

A new step (f) is added to the existing workflow, after step (e). It runs `bash scripts/lint-procedure-parseability.sh`, which:

1. Builds the runtime binary in `--release` mode locally in the workflow runner. This is the *only* place the workflow has a Rust toolchain — confined to this lint, not poisoning the markdown-only assertion that the binary is absent at command-execution time.
2. Invokes `runtime parse --check framework/commands/*.md`, allowing failures for files listed in `runtime/legacy-prose-commands.txt`.
3. Exits non-zero on any other failure.

Wait — this contradicts spec 021's check (a) "no runtime binary on PATH." The parseability check needs the binary; check (a) demands its absence. Resolution: the parseability check runs the binary as `./runtime/target/release/runtime` directly (relative path, not PATH lookup). Spec 021's check (a) is unchanged — it greps PATH for names in `framework/runtime-tools.txt`, which is about whether the binary is *available to slash commands*, not whether the workflow runner has compiled a copy locally for its own lint. This distinction is annotated in `lint-procedure-parseability.sh` and surfaced in a comment in the workflow file so future readers don't trip on it.

Alternative considered: ship a tiny standalone parser in bash. Rejected — the parser is the single source of truth for what "parseable" means, and a bash reimplementation would drift.

### `runtime.yml` CI workflow

Triggers on PR and push to `main`, with `paths` filter covering `runtime/**` and any `framework/commands/*.md` (rewrites must keep parseability). Steps:

1. Checkout.
2. Setup Rust toolchain (`actions-rust-lang/setup-rust-toolchain` or `dtolnay/rust-toolchain`).
3. `cargo build --release` in `runtime/`.
4. `cargo test --release` in `runtime/`.
5. `cargo clippy -- -D warnings`.
6. `cargo fmt --check`.

No matrix on OS in the per-PR workflow — Linux only. The release workflow is where cross-platform matters; PR CI is a smoke test.

### Release workflow

A separate workflow at `.github/workflows/runtime-release.yml`, triggered on tag pushes matching `runtime-v*`. Steps:

1. Matrix across the target triples in the acceptance criteria: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc` if cross-compilation is friction-free.
2. Build via `cargo-zigbuild` for Linux ARM (cross-compilation), native build for macOS targets on macOS runners, and `cargo build` on a Windows runner for the Windows target.
3. Tarball/zip each binary with a `sha256sum` checksum file.
4. Upload to a GitHub release created from the tag, using `softprops/action-gh-release`.

Tag scheme is `runtime-v<MAJOR>.<MINOR>.<PATCH>`, distinct from any framework tag so framework releases and runtime releases evolve in lockstep but are independently traceable.

The release workflow is documented in `runtime/CHANGELOG.md` (release process appendix) and exercised once before this spec is marked `done` — the first release tag (e.g., `runtime-v0.1.0`) ships in this spec to validate the workflow end-to-end.

### README placement: root `README.md` Runtime section

The Runtime section lands in the root `README.md`, placed after the existing "Feature Specs" section and before the closing TL;DR-like material. The section:

1. One paragraph of rationale: opt-in, faster slash commands, markdown-only path still works.
2. Install instructions: a fenced bash block invoking `curl` against the GitHub release artifact URL pattern, with shasum verification.
3. A "When to install" guidance paragraph (basically: install if you adopt govern and run slash commands frequently; skip if you only use govern occasionally).

No edits to `framework/templates/project/project-readme.md` — the template ships to adopters, who do not redistribute the runtime themselves. Adopters install the runtime in their own dev environment; the section in *this* repo's README is the install-instructions surface.

### Bootstrap completion-message pointer

One additional line is appended to the "Next steps" list in both First-run and Update-mode blocks of `framework/bootstrap/govern.md` (lines 750-763 and the matching update-mode block). The line reads roughly: *"Optional: install the deterministic runtime for faster slash commands — see [Runtime](https://github.com/...#runtime) in the govern README."* Wording finalized at implementation time.

The line is the only place the bootstrap output mentions the runtime; per the spec's install-policy resolution, no slash command may detect-and-warn about the missing binary.

### Generator scripts stay untouched

Per the spec's non-goals and §runtime-boundary principle 3, the `gen-*.sh` scripts under `scripts/` are not wrapped, modified, or replaced by the runtime. The pre-commit hook continues to call them directly. The runtime's `run-generator` primitive is a thin wrapper for procedure use (invokes the same bash script with `--dry-run` and surfaces drift as a finding), but the pre-commit path never goes through it.

### `framework/runtime-tools.txt` is the manifest

Populated in this spec with the 13 MCP tool names following the `gov-rt:<verb>-<noun>` convention. Each name on its own line. Comment header preserved from spec 021. After population:

```text
gov-rt:read-spec
gov-rt:read-tasks
gov-rt:mark-task
gov-rt:mark-criterion
gov-rt:set-status
gov-rt:derive-boundary
gov-rt:check-stuck
gov-rt:validate-frontmatter
gov-rt:resolve-anchor
gov-rt:traverse-deps
gov-rt:check-rule-ids
gov-rt:run-generator
gov-rt:lint-markdown
gov-rt:gate-confirm
```

That is 14, not 13 — `gate-confirm` is in the primitive library list in the spec and gets a tool name like every other primitive. The spec's "initial primitive set" enumeration has 14 entries when counted; `mark-task` / `mark-criterion` are listed together with a slash but are two distinct primitives. The plan reads them as two; the manifest reflects that.

`scripts/lint-tool-coverage.sh` (from spec 021) verifies that every reference to any name in this manifest appears within 20 lines of a fallback marker. Each runtime-tool reference in a rewritten command will be paired with prose like "Otherwise, follow the procedure described above." to keep the lint green.

### No data persistence outside session file + markdown

State management is the spec's already-resolved decision: in-memory within a run, markdown + `.claude/gov-session.json` are the durable journal. The plan reaffirms with concrete implementation: the interpreter holds parsed AST + walker position + pending payload in `interpreter::State`, a plain `struct` with no `Drop`-time side effects. Process death loses this state without consequence; the user re-invokes the slash command and the runtime re-derives position from the markdown.

### Error semantics and exit codes

Already resolved in the spec; concrete mapping:

- Exit code `0` — `complete` JSON message written; clean run.
- Exit codes `1-127` — `error` JSON message written; primitive- or interpreter-level failure. Specific codes:
  - `1` — generic operational error.
  - `2` — parse error.
  - `3` — gate-confirm denied (clean denial, not an error per spec; mapped to `complete` with a `confirmed: false` payload — exit `0`. This code is reserved but unused for now; documented for forward compatibility).
  - `64-78` — `sysexits.h` ranges for I/O, permission, OS errors.
- Exit codes `128+` — signal-killed (no terminal message). Host fallback applies.

### Trade-offs

Considered and rejected:

- **Hand-rolled markdown lexer instead of `pulldown-cmark`** — rejected. Brings the maintenance burden of a markdown spec edge-case-tracker into this repo for no benefit. `pulldown-cmark` is widely used, well-tested, and the event stream is the right level of abstraction for the structural recognizer.
- **Separate `procedure` parser surface separate from `Instructions`** — rejected at clarify-time (spec resolution). Plan reaffirms: prose IS the procedure.
- **Cargo workspace with `runtime-core` + `runtime-cli`** — rejected. A workspace is premature factoring for a single binary with no other consumer; the boundary between primitive logic and CLI args is already small. If a second consumer ever appears, splitting is a small refactor.
- **`tracing` for structured logs** — rejected. Tracing's value is multi-target output sinks and async-aware spans; this runtime is short-lived and single-threaded for the walker. `eprintln!` on stderr is the floor; structured JSON `progress` messages on stdout are the structured channel. Adding tracing would increase dep surface and binary size for no observability win.
- **`reqwest` for any HTTP** — runtime makes zero outbound HTTP calls. No HTTP client dep.
- **Daemon / persistent mode** — explicit non-goal in the spec.
- **In-process LLM client** — explicit non-goal (`Determinism only` principle).
- **Build the runtime as part of the markdown-only-pipeline workflow PR-by-PR** — rejected as default; only the parseability check builds it locally. The markdown-only assertion remains intact.
- **Cross-platform parity testing in CI** — out of scope for this spec. The release workflow builds cross-platform artifacts; parity testing is Linux-only in `runtime.yml`. macOS/Windows binary correctness is covered by the release workflow's smoke test (each platform's binary runs `runtime --version` after build).
- **Embedding a markdown linter natively** — explicit non-goal in spec; runtime wraps `npx markdownlint-cli2` via the `lint-markdown` primitive.

### Known limitations

- **No retry / no resume**: a runtime crash mid-procedure loses position; the user re-invokes the command and the runtime walks the markdown to figure out where to resume. This is acceptable per the spec's state-management resolution but means long procedures with many tasks re-do all the boilerplate (re-reads files, re-parses) on resume. Mitigation: primitives are cheap enough that re-doing them is sub-second.
- **`pulldown-cmark` edge cases**: certain markdown extensions (tables, footnotes, definition lists) are off by default in `pulldown-cmark`. The procedure parser opts into tables; other extensions are off. If a slash command body relies on an extension we don't opt into, the parser will silently misread it. Mitigation: the parseability check on every PR catches this at the earliest opportunity.
- **Windows atomic-write semantics**: weaker than POSIX. A crash on Windows between write and rename can leave a partial tempfile in the parent directory (not the target file, which is unchanged). Cleanup is a manual `rm` operation; documented in the README.
- **Fixture maintenance cost**: every command rewrite needs a fixture under `runtime/tests/fixtures/`. The first fixture (for `/gov:status`) costs the most to build; subsequent fixtures fork from it. The cost is bounded but real.
- **Parity captures are manual**: the LLM-output captures under `runtime/tests/parity/` are taken once per command by a maintainer and committed. If the LLM-driven path's output changes (because constitution updates change what `/gov:validate` flags, for instance), the captures must be re-taken. There is no automatic regeneration; that would require an LLM in CI, which the spec rules out.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `runtime/Cargo.toml` | Create | Crate manifest, dependencies, MSRV |
| `runtime/Cargo.lock` | Create | Lockfile, committed |
| `runtime/src/main.rs` | Create | CLI entrypoint, clap dispatch |
| `runtime/src/lib.rs` | Create | Module re-exports for integration tests |
| `runtime/src/parser/` | Create | Procedure parser over `pulldown-cmark` events |
| `runtime/src/interpreter/` | Create | Procedure walker, JSON protocol I/O |
| `runtime/src/primitives/` | Create | One module per primitive (14 modules) |
| `runtime/src/mcp/` | Create | `rmcp`-based MCP server |
| `runtime/src/schema/` | Create | Typed schemas for primitives, protocol, extension points |
| `runtime/src/io.rs` | Create | Stdio framing for protocol messages and progress |
| `runtime/tests/` | Create | Integration tests + golden output comparisons |
| `runtime/tests/fixtures/` | Create | Per-command fixture repos |
| `runtime/tests/golden/` | Create | Expected JSON message streams |
| `runtime/tests/parity/` | Create | Captured LLM-driven outputs for parity comparison |
| `runtime/legacy-prose-commands.txt` | Create | Allowlist of not-yet-rewritten command files |
| `runtime/CHANGELOG.md` | Create | Runtime release notes |
| `runtime/.gitignore` | Create | Exclude `target/` |
| `framework/runtime-tools.txt` | Edit | Populate with the 14 MCP tool names |
| `framework/commands/status.md` | Edit | Rewrite Instructions to parseable conventions |
| `framework/commands/target.md` | Edit | Rewrite Instructions to parseable conventions |
| `framework/commands/validate.md` | Edit | Rewrite Instructions to parseable conventions; add `assessSpecQuality` extension point marker |
| `framework/commands/implement.md` | Edit | Rewrite Instructions to parseable conventions; add `writeCode` extension point marker |
| `framework/commands/plan.md` | Edit | Rewrite Instructions to parseable conventions; add `writeSpecBody` extension point marker |
| `framework/commands/specify.md` | Edit | Rewrite Instructions to parseable conventions; add `writeSpecBody` extension point marker |
| `scripts/lint-procedure-parseability.sh` | Create | Bash wrapper that builds runtime and invokes `runtime parse --check` |
| `.github/workflows/markdown-only-pipeline.yml` | Edit | Add step (f) parseability check; preserve existing checks (a)–(e) |
| `.github/workflows/runtime.yml` | Create | Per-PR build + test + clippy + fmt |
| `.github/workflows/runtime-release.yml` | Create | Tag-triggered cross-compile + release upload |
| `framework/bootstrap/govern.md` | Edit | One-line pointer to runtime in completion message (first-run and update-mode blocks) |
| `README.md` | Edit | New Runtime section: rationale, install, when to install |
| `specs/022-deterministic-runtime/plan.md` | Create | This file |
| `specs/022-deterministic-runtime/tasks.md` | Create | Task breakdown |
| `specs/022-deterministic-runtime/data-model.md` | Create | Procedure AST, JSON protocol, primitive and extension-point schemas |

## Cross-Spec Validation

- **`framework/constitution.md`** read; plan is consistent with §runtime-boundary (5 principles, 3 eligibility criteria, opt-in invariant, lockstep versioning, non-scope). The plan introduces no spec-authoring tooling, no workflow orchestration, no long-running services, no storage layer. Schemas live in this spec body and `data-model.md`; the constitution does not import them.
- **`specs/021-runtime-boundary/spec.md`** read; plan implements the runtime that 021 makes constitutionally admissible. The CI parseability check coexists with 021's opt-in invariant checks; the resolution above explains why the binary's presence in the parseability step does not violate check (a) in 021's workflow.
- **`specs/events.md` / `specs/errors.md`** — not present in this project (project-level cross-cutting files). No event or error-code coordination needed.
- No sibling spec data models exist that conflict with this feature's data model. The runtime data structures are internal to the binary and the JSON protocol; they do not bleed into other specs.

## Trade-offs

See the Considered and rejected and Known limitations subsections in Technical Decisions above. The major trade-offs are: dependency surface (kept tight); Windows atomicity (accepted as weaker); fixture maintenance cost (accepted as bounded); parity captures committed-not-regenerated (accepted to keep LLMs out of CI).
