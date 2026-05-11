---
status: planned
dependencies: [021-runtime-boundary]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 022 — Deterministic Runtime

The runtime is the deterministic execution layer for govern. It interprets slash command procedures, runs the mechanical work each command requires, and invokes the LLM only at named extension points where semantic judgment is genuinely required. Slash command markdown files remain the source of truth and stay LLM-readable; the runtime is an accelerator, not a replacement.

Spec [021-runtime-boundary](../021-runtime-boundary/spec.md) made the runtime constitutionally admissible under [§runtime-boundary](../../framework/constitution.md#runtime-boundary). This spec defines what the runtime actually is.

## Motivation

A `/gov:validate` invocation against a single feature takes 2–4 minutes of LLM wall-clock time today. `/gov:implement`'s per-task gate cycle is the same order of magnitude. Most of that time is spent on work with no semantic content: parsing frontmatter, walking checkbox lists, verifying acceptance criteria one by one, running stuck-detection by reading git log output, deriving the runtime write boundary, checking generator drift via prose instructions that invoke bash scripts. The LLM is doing mechanical orchestration in slow tokens-per-second instead of fast bytes-per-second.

The deterministic share per command is large:

- `/gov:target`, `/gov:status` — 100% deterministic.
- `/gov:validate` — 11 of 13 check sections are mechanical; only per-rule Verification reads require semantic judgment.
- `/gov:implement` — every gate, every checkbox update, every boundary derivation, every status transition is deterministic. The LLM's actual contribution is *writing the code for each task* — that work justifies its cost, but the deterministic scaffold around it is pure overhead.
- `/gov:plan`, `/gov:specify` — file creation, status advancement, template copying is deterministic. Filling in technical decisions and spec body is semantic.
- `/gov:clarify`, `/gov:review`, `/gov:groom` — predominantly semantic, but each carries a deterministic scaffold (read spec, identify open questions, advance status when none remain) currently walked by the LLM.

Across a pipeline cycle, the wall-clock bulk is mechanical work the runtime can absorb. The savings compound: faster per invocation, more invocations per session, lower token cost in aggregate, and — most importantly — interactive instead of "go get coffee while govern runs."

The reliability story from earlier drafts is preserved as a secondary benefit: deterministic execution of deterministic procedures eliminates the probabilistic-miss class of failures that LLM-walked mechanical work occasionally produces. But the headline is wall-clock and cost.

## Architecture

Three pieces, each independently testable.

### Parseable slash command procedures

Slash command markdown files in `framework/commands/*.md` are rewritten so the existing Instructions section IS the procedure — no separate Procedure block, no twin maintenance. The runtime parses the prose directly, using these conventions:

- **Numbered steps** (`1.`, `2.`, ...) are procedure steps. Sub-numbering (`1.1`, `1.2`) marks sub-steps.
- **Backtick-quoted primitive names** inside a step (e.g., `` `read-spec` ``, `` `mark-task` ``, `` `derive-boundary` ``) are primitive calls. The full primitive set is enumerated in §The primitive library below.
- **HTML-comment extension-point markers** (e.g., `<!-- llm:writeCode -->`, `<!-- llm:askClarifyQuestion -->`) mark the seams where the runtime invokes the LLM. The named identifier matches an entry in §LLM extension points.

A markdown-only adopter (no runtime on `PATH`) reads the prose as today — the conventions are unobtrusive and the prose stays human-readable. The runtime reads the same prose and extracts structure. There is no second source of truth to drift from.

A **parseability check** is added to `.github/workflows/markdown-only-pipeline.yml`: every slash command's Instructions section must parse cleanly under these conventions. Parse failure is a CI fail; this catches the only relevant kind of drift (prose convention slipped) at PR time.

### The interpreter

The interpreter is the runtime's procedure-execution engine, invoked by an agent host as a subprocess. Given a slash command name and arguments, it:

1. Parses the command's Instructions section under the structural conventions described above.
2. Walks the steps in order.
3. For each primitive step, calls the corresponding primitive operation in-process.
4. For each LLM extension point, emits a JSON request message to stdout, suspends, and resumes when the agent host writes a JSON response message to stdin.
5. Manages pipeline state — session file, status transitions, gate confirmations, checkbox updates — through primitives.
6. Returns when the procedure completes, halts at a gate, errors, or is cancelled.

The interpreter is stateful within a single subprocess run (in-memory only) and stateless across runs. Every invocation reads state from the markdown source of truth and the session file; nothing persists in the runtime between invocations.

### The primitive library

Primitives are the deterministic operations the interpreter offers procedures. Each primitive has a CLI subcommand (invokable standalone) and an MCP tool (callable by any agent host). The initial primitive set:

- `read-spec` — parse spec frontmatter and body sections.
- `read-tasks` — parse tasks.md into a structured task list with checkboxes.
- `mark-task` / `mark-criterion` — flip checkbox state in a markdown file.
- `set-status` — update spec frontmatter status field.
- `derive-boundary` — compute the runtime write boundary from git diff against the spec dir's first commit.
- `check-stuck` — count commits on tasks.md since status entered `in-progress` and surface cycles.
- `validate-frontmatter` — full frontmatter schema check.
- `resolve-anchor` — verify every `§<anchor>` reference resolves to a `<!-- §anchor -->` marker.
- `traverse-deps` — verify spec dependencies exist as directories and have compatible status.
- `check-rule-ids` — verify cited rule IDs exist in loaded rule files and aren't deprecated.
- `run-generator` — invoke a bash generator in `--dry-run` mode and report drift.
- `lint-markdown` — wrap `npx markdownlint-cli2`.
- `gate-confirm` — surface a pipeline gate to the user through the agent host and await confirmation.

Each primitive is deterministic, independently testable, and has a markdown-only fallback (either a bash script or a prose instruction the LLM can execute).

## Surfaces

The runtime binary exposes two interfaces. Agent hosts pick whichever they integrate with; both are first-class.

### MCP server

Invoked as `runtime mcp`. Exposes every primitive in §The primitive library as an MCP tool. Any MCP-capable agent host (Claude Code, Auggie, future hosts) connects without per-host integration code. The LLM walks the slash command prose as the orchestrator and calls MCP primitives for each operation that has one. Wall-clock improvement is moderate — each primitive call is microseconds instead of LLM-cognitive-seconds — but the LLM's decide-which-primitive-and-read-the-result orchestration loop remains.

This surface satisfies §runtime-boundary principle 5 ("MCP is the seam"). It is the universal-compatibility face of the runtime.

### Subprocess interpreter

Invoked as `runtime exec <command> [args...]`. The agent host spawns the runtime as a subprocess and communicates via newline-delimited JSON over stdio:

- `{"type":"llm-request","extension-point":"<name>","request":{...}}` — runtime suspends; awaits an LLM-driven response from the host.
- `{"type":"llm-response","response":{...}}` — host returns the LLM's structured output; runtime resumes.
- `{"type":"gate-confirm","gate":"<name>","prompt":"..."}` — runtime suspends; awaits user confirmation routed through the host.
- `{"type":"gate-response","confirmed":<bool>}` — host returns the user's decision.
- `{"type":"progress","message":"..."}` — informational, non-blocking.
- `{"type":"complete","result":{...}}` — procedure finished; runtime exits 0.
- `{"type":"error","code":"...","message":"..."}` — procedure halted; runtime exits non-zero.

Stderr is reserved for human-readable logs.

This surface drives the entire procedure end-to-end from the runtime, bypassing the LLM's orchestration loop. Wall-clock improvement is dramatic — minutes to seconds for `/gov:validate` — because the LLM is invoked only at named extension points for genuine semantic work. Adopting this surface is opt-in per-host: each integrating host implements the JSON message protocol (~few hundred lines of code). Claude Code is the first integrator; the protocol is documented and stable for others.

### Surface choice by adopter

- **Markdown-only adopter** (no runtime binary on `PATH`): uses neither surface. LLM walks prose, prose names bash scripts. Per §runtime-boundary principle 3.
- **Auggie or other MCP-only host**: uses the MCP surface. Fast primitives, LLM orchestrates. No host-side integration code.
- **Claude Code or other JSON-protocol-integrated host**: uses the subprocess interpreter surface for slash commands. May also call individual primitives directly via MCP when convenient.

## LLM extension points

An extension point is a named seam in a slash command's Instructions section where semantic work happens. Each extension point has:

- A unique identifier (e.g., `writeCode`, `writeSpecBody`, `askClarifyQuestion`) declared in an HTML-comment marker on the relevant step.
- A schema for the request payload prepared at the seam.
- A schema for the response payload validated at the seam.
- Surrounding prose in the Instructions section describing what should happen — this is what the LLM reads in the markdown-only and MCP-server cases.

How extension points are invoked depends on the surface:

- **Subprocess interpreter**: the runtime emits a JSON `llm-request` message to stdout and suspends. The agent host calls the LLM with the structured request, validates the response against the schema, and writes a `llm-response` message to stdin. The runtime resumes.
- **MCP server**: the LLM is already the orchestrator and walks the surrounding prose itself. The HTML-comment marker is advisory — it tells the LLM "this step is the semantic part, do it yourself, don't try to find a primitive." Surrounding prose still names primitives that may be useful before or after the extension point.
- **Markdown-only**: same as MCP-server — LLM reads the marker as a hint and follows the prose instructions for the semantic work.

The extension point inventory across the initial release and follow-on scenarios:

**Shipped in the initial release** (single-shot — one `llm-request` / `llm-response` round-trip):

- `assessSpecQuality` — `/gov:validate`'s per-rule Verification reads. Request: spec content + a rule's Verification phrase. Response: pass/fail + finding text.
- `writeCode` — `/gov:implement` walk-through-tasks step 4. Request: task description, plan-relevant files, write boundary. Response: list of file edits.
- `writeSpecBody` — `/gov:specify` and `/gov:plan` template-fill moments. Request: template + feature description + section name. Response: filled-in section content.

**Deferred to scenarios on this spec**:

- `askClarifyQuestion` — `/gov:clarify`'s open-question loop. Multi-turn user-mediated: the agent host shows the runtime-prepared question to the user, awaits the user's answer, returns it. Ships in the first follow-on scenario.
- `performReview` — `/gov:review` semantic pass. Ships in the review scenario.
- `routeInboxItem` — `/gov:groom` routing decision. Ships in the groom scenario.

## Markdown-only path

When the runtime is absent from `PATH`, the LLM walks the slash command's prose Instructions as it does today. The structural conventions (numbered steps, backtick-quoted primitive names, HTML-comment extension-point markers) are unobtrusive to a human or LLM reader — primitives read as named operations the LLM should perform, extension points read as labelled markers. The bash scripts under `scripts/` remain authoritative for the primitives that have bash counterparts (`gen-*.sh`, `lint-frontmatter.sh`, `lint-tool-coverage.sh`); the prose names them and the LLM invokes them.

The opt-in invariant from spec 021 — CI proves the markdown-only path completes — fires unchanged. It now proves that the same prose executes correctly under two interpreters (the LLM walking it, and the runtime parsing and executing it), and that the conventions haven't slipped.

## Slash command rewiring

This spec rewrites slash command markdown files in `framework/commands/*.md` so each Instructions section follows the parseable conventions. The initial release covers six commands; the remaining three ship as scenarios on this spec.

### Initial release — six commands

The pipeline backbone:

1. `/gov:status` — 100% deterministic; smallest surface; validates the architecture.
2. `/gov:target` — 100% deterministic; exercises session-file write path and constitution loading.
3. `/gov:validate` — headline wall-clock win. Extension point: `assessSpecQuality` (per-rule Verification reads).
4. `/gov:implement` — largest behavioral surface. Extension point: `writeCode` (per-task work).
5. `/gov:plan` — template-driven plan generation. Extension point: `writeSpecBody` (shared with `/gov:specify`).
6. `/gov:specify` — new feature scaffolding. Extension point: `writeSpecBody`.

Three extension points are designed and shipped in the initial release: `assessSpecQuality`, `writeCode`, `writeSpecBody`. All are single-shot (one `llm-request` / `llm-response` round-trip per invocation) — no multi-turn user-mediated interaction.

### Follow-on scenarios

Each subsequent command lands as a scenario on this spec, in this order:

1. **Scenario: `/gov:clarify`** — first follow-on. Introduces the `askClarifyQuestion` extension point, which is multi-turn and user-mediated (the agent host shows the question, awaits the user's response, returns it to the runtime). The trickier ABI ships after the single-shot pattern proves out in the initial release.
2. **Scenario: `/gov:review`** — introduces `performReview`. Runtime's value-add is small (predominantly LLM work) but ships for pipeline completeness.
3. **Scenario: `/gov:groom`** — introduces `routeInboxItem`. Similar to review: runtime accelerates only the bookkeeping around the inbox walk; routing decisions are LLM-driven.

### Per-rewrite checklist

Each command's rewrite:

- Preserves the command's existing behavior — same primitives invoked, same gates, same order.
- Adopts strict numbered-step format with sub-numbering for sub-steps.
- Quotes every primitive call in backticks, using a name from §The primitive library.
- Marks every LLM seam with an HTML-comment extension-point marker, using a name from §LLM extension points.
- Passes the parseability check after the edit.

## Install policy

The project does not auto-install the runtime — §runtime-boundary principle 3 makes the runtime opt-in for adopters, and 022's existing non-goal forbids auto-install on `/govern` adoption. Discovery of the optional install lives in two places:

- **README** has a Runtime section: one paragraph of rationale ("install for faster slash commands; the markdown-only path works without it"), followed by concise install instructions for whichever channel(s) the distribution-channel question resolves (see Open Questions).
- **`/govern` bootstrap output** mentions the optional runtime install in its completion message — a single line pointing readers at the README Runtime section. One time, at project scaffolding, not on every command.

Slash commands MUST NOT detect the missing runtime and print install nags on each invocation. The markdown-only path is a first-class path per principle 3, not a degraded mode adopters should be upgraded away from. Treating it as degraded would erode the constitutional intent of principle 3 even if it satisfied its letter.

## CI integration

Two workflows coexist after this spec lands:

- `.github/workflows/markdown-only-pipeline.yml` (existing, from spec 021) — proves the markdown-only path completes with the runtime absent from `PATH`. Same five checks as today, plus an added parseability check that every `framework/commands/*.md` Instructions section parses cleanly under the structural conventions.
- `.github/workflows/runtime.yml` (new) — builds the binary, runs its test suite, exercises every primitive against fixture inputs, exercises every slash command's CLI subcommand against a fixture repo, and produces release artifacts. Triggers only on changes to runtime source paths.

The two workflows are independent. The runtime workflow MUST NOT install the binary into the markdown-only workflow's environment.

## Bash script relationships

Stable relationships post-rewrite:

- **`gen-*.sh`** (called by the pre-commit hook) — stay bash. The runtime never replaces them: pre-commit has no LLM in the loop, so they fail the eligibility rule from §runtime-boundary principle 3. The runtime's `run-generator` primitive is a thin wrapper for procedure use; pre-commit continues to call the bash scripts directly.
- **`lint-frontmatter.sh`** — repositioned as the markdown-only fallback for the runtime's `validate-frontmatter` primitive. Same intent, two implementations, both ship; the prose Instructions invoke whichever is available.
- **`lint-tool-coverage.sh`** — stays bash-only with no runtime counterpart. The lint runs exclusively in `markdown-only-pipeline.yml`, which asserts the runtime is absent; a runtime version is unreachable in that workflow.

## Acceptance Criteria

- [ ] A single binary builds from this repo and exposes two surfaces: `runtime mcp` (MCP server) and `runtime exec <command> [args...]` (subprocess interpreter).
- [ ] Every primitive in §The primitive library is exposed as an MCP tool by `runtime mcp`, named under the `gov-rt:<verb>-<noun>` convention.
- [ ] The subprocess interpreter's JSON-over-stdio message protocol is documented in the runtime's docs and stable enough for third-party agent hosts to integrate against.
- [ ] The six initial-release slash commands (`/gov:status`, `/gov:target`, `/gov:validate`, `/gov:implement`, `/gov:plan`, `/gov:specify`) have their prose Instructions sections rewritten to follow the structural conventions and parse cleanly under the runtime parser. The remaining three (`/gov:clarify`, `/gov:review`, `/gov:groom`) are not rewritten in this spec — they ship as scenarios.
- [ ] The three initial-release LLM extension points (`assessSpecQuality`, `writeCode`, `writeSpecBody`) each have a request schema, response schema, and a corresponding HTML-comment marker in the relevant Instructions step.
- [ ] A parseability check is added to `.github/workflows/markdown-only-pipeline.yml` and passes against every slash command file — the six rewritten ones under the new conventions, and the three not-yet-rewritten ones under the legacy prose-walk path (the parser tolerates files that do not yet declare a Procedure-shaped Instructions section).
- [ ] The binary executes each of the six initial-release commands end-to-end against a fixture repo and produces output consistent with the LLM-driven path against the same fixture, within the determinism bounds defined for each command.
- [ ] Median wall-clock time per invocation drops from minutes to seconds for each of the six initial-release commands when the runtime is present.
- [ ] The markdown-only path (no binary on `PATH`) continues to complete every pipeline cycle (greenfield, brownfield, reopen) as it did before this spec.
- [ ] `framework/runtime-tools.txt` is populated with every MCP tool name the binary exposes; `scripts/lint-tool-coverage.sh` passes against the rewritten slash command files.
- [ ] A new CI workflow at `.github/workflows/runtime.yml` builds the binary, runs its test suite, exercises every primitive against fixture inputs, and fails on any test failure.
- [ ] The existing `.github/workflows/markdown-only-pipeline.yml` workflow continues to pass with the runtime binary absent from `PATH`.
- [ ] State-modifying primitives (`mark-task`, `mark-criterion`, `set-status`) use filesystem-atomic writes (write to temp + atomic rename) so a runtime crash mid-write leaves coherent markdown.
- [ ] The runtime self-reports its build-time version via `runtime --version`; parse failures emit a descriptive `error` JSON message that includes the runtime version and notes version-mismatch as a possible cause.
- [ ] The binary is distributed via GitHub release artifacts cross-compiled for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` (and `x86_64-pc-windows-msvc` if cross-compilation is friction-free in CI), with sha256 checksums; adopter-facing install instructions exist in a README Runtime section.
- [ ] The `/govern` bootstrap command's completion output includes a one-line pointer to the README Runtime section, mentioning the runtime is optional.
- [ ] No slash command in `framework/commands/*.md` detects the missing runtime and prints an install nag on invocation; the markdown-only path remains a first-class path per §runtime-boundary principle 3.
- [ ] `/gov:validate` against this spec passes with no hard-fail or blocking findings.
- [ ] `npx markdownlint-cli2` against all rewritten slash command files and new spec files passes.

## Non-Goals

- Replacing the LLM at semantic extension points — that work is by definition outside the runtime.
- Replacing the slash command markdown files with a different source format. Markdown stays the source of truth; structural conventions are layered into the existing prose, not extracted to a separate file or format.
- Rewriting, wrapping, or otherwise interacting with the bash generator scripts (`gen-*.sh`) — pre-commit context, no LLM fallback, not runtime-eligible per §runtime-boundary principle 3.
- Persisting any state outside the markdown source of truth and the session file — runtime state is derived and gitignored per principle 1.
- Auto-installing the runtime on `/govern` adoption — opt-in per principle 3.
- Daemon mode, long-running services, or background processes — per the non-scope list in §runtime-boundary.
- Reimplementing `npx markdownlint-cli2` natively — the runtime wraps it as a primitive.
- Building any non-MCP integration surface (LSP, web UI, REST). Speculative; would need its own spec under §runtime-boundary's eligibility criteria.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Partial-failure semantics** — every primitive distinguishes two outcome categories. (1) Domain outcomes: the primitive completed; result reports success/failure of the check it performed (e.g., `validate-frontmatter` returns "3 issues" or "clean"). These are findings, not errors — procedure continues, findings accumulate. (2) Operational errors: the primitive could not complete (file unreadable, git non-zero, parse failure, OS error). Default: halt the procedure, emit a structured `error` JSON message to the host, exit non-zero. Per-primitive overrides: `mark-task` / `mark-criterion` / `set-status` rollback partial writes before halting (atomicity); `gate-confirm` denial is a clean `complete` outcome, not an error; `run-generator` non-zero exit is a drift finding, not an operational error. The runtime does NOT ask the LLM what to do on error (blurs determinism boundary), does NOT auto-fall-back to prose for the remainder (state-recovery problem), and does NOT continue past operational errors (risks cascading damage). The companion question for when the runtime *process itself* crashes is *Runtime-failure fallback semantics* below.
- **Per-command opt-out** — none in v1. The only opt-out mechanism is binary presence on `PATH` (subprocess interpreter) or MCP client configuration in the agent host (MCP surface). No env var (`GOVERN_RUNTIME_DISABLE=...`), no config-file section, no CLI flag, no per-command annotations. Configuration surface is permanent technical debt and the need is speculative — no concrete case today for trusting the runtime for one command but not another. Real use cases (debugging, side-by-side comparison, CI workflows that need the prose path) are well-served by PATH manipulation; the markdown-only-pipeline workflow already asserts binary absence and doesn't need additional config. If a real need materializes later, a future spec or scenario adds the knob with a motivating example.
- **Distribution channels** — GitHub release artifacts only for v1. Pre-built binaries cross-compiled for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, with sha256 checksums; CI builds and publishes on tag. README Runtime section's install instructions point at the GitHub release page. Windows target shipped if cross-compilation is friction-free in CI; otherwise deferred. Homebrew was struck — `homebrew-core` has popularity gates that the project may not clear early, and maintaining a personal tap adds ongoing cost for marginal benefit. `cargo install govern-runtime` from crates.io was scoped out of v1 to keep release engineering tight; logged to `specs/inbox.md` for revisit after v1 ships and adoption patterns are observable. Linux distro packages and Windows package managers (Scoop, Chocolatey) are community-maintained territory, not project-shipped. Auto-update / self-update inside the binary is out of scope — adopters re-download for new versions.
- **MCP tool naming convention** — `gov-rt:<verb>-<noun>`. Kebab-case under the `gov-rt:` namespace prefix, colon separator. The prefix disambiguates primitives from slash commands (`gov:<command>`) and gives `lint-tool-coverage.sh` an unambiguous string to match against in prose. Verb-noun pattern forces clarity at the naming layer. Forward-compatible if govern ever ships a second MCP server (e.g., `gov-an:` for analytics). Rejected: `gov:` (collides with slash-command pattern); `gov_rt_*` snake_case (less readable); unprefixed names (lint false-positive risk); dotted `gov.runtime.*` (verbose, uncommon in MCP).
- **Versioning enforcement** — rely on schema-evolution discipline plus clear parse errors. No `framework-version:` field added to the constitution; no startup version comparison; no warn-on-mismatch. The binary self-reports its build-time version via `runtime --version` (read from `Cargo.toml` at compile time). On parse failure the binary emits a descriptive `error` JSON message that includes the runtime version and points the adopter at version-mismatch as a possibility. §runtime-boundary's lockstep-versioning policy is the actual protection — adopters who follow tagged releases get matching framework + binary; the parse-error path covers the remaining mismatch cases. Rejected: warn-and-continue (warnings get ignored); explicit refuse-on-mismatch (premature infrastructure for a problem that hasn't materialized); silent best-effort (worst outcome — silently wrong results).
- **Runtime-failure fallback semantics** — surface and fall back. When the runtime *process itself* crashes (segfault, OOM, signal — distinct from a primitive returning a structured error, covered by *Partial-failure semantics* above): (1) host surfaces a user-visible notice including exit code, runtime version, and an issue-reporting URL; (2) host falls back to LLM-walked prose execution for the same command so the user's request still completes. The MCP surface case is already covered by existing fallback discipline (primitive fails → LLM follows the prose fallback marker → bash script or LLM execution). The runtime contributes: filesystem-atomic writes for state-modifying primitives (`mark-task`, `mark-criterion`, `set-status` — write to temp + atomic rename so crashes leave coherent markdown), clear exit-code signaling (0 = clean complete; 1-127 = clean operational error with terminal `error` message; 128+ = signal-killed crash with no terminal message), and version reporting. The runtime does NOT catch its own crashes, persist mid-run state for resume, or auto-retry. Host-side fallback implementation is the expected partner behavior, referenced in the spec but not a runtime AC (Claude Code and Auggie each implement their own integration). Rejected: silent fallback (hides bugs); surface-only halt (bad UX for recoverable cases).
- **Implementation language** — Rust. Reasons: (1) Anthropic's `rmcp` crate is the most mature MCP SDK and is the reference implementation; (2) smallest static binary and fastest cold-start, both of which matter per-invocation; (3) strongest type system for the procedure interpreter's state-machine work; (4) memory safety relevant for parsing user-supplied markdown and JSON; (5) the recent generation of Rust CLI tooling (`ripgrep`, `fd`, `bat`, `eza`, `helix`, `tokei`, `hyperfine`, etc.) has set a high baseline for CLI UX, ecosystem familiarity, and patterns (single static binary, fast cold-start, sensible exit codes) — shipping in that lineage benefits the runtime's adoption. Go was the credible alternative (faster builds, easier cross-compilation, more accessible to newcomer contributors, adequate MCP libraries); rejected because for a foundational tool that's modified rarely and invoked frequently by many adopters, production characteristics outweigh development-velocity characteristics. Other languages (Node/Bun/Deno, Python, C++, Zig, Swift) rejected for distribution complexity, startup overhead, ecosystem fit, or maturity.

- **State management within a procedure run** — in-memory only. Markdown source of truth and `.claude/gov-session.json` are the durable journal; primitives write to them synchronously (e.g., `mark-task` flips the checkbox in the file before moving to the next task). In-memory state holds working copies: position in the procedure, parsed file contents, pending extension-point payloads. Crash recovery is "user re-invokes the slash command; runtime reads markdown to observe what's already done and resumes from the next incomplete step." Observability is stderr (human-readable progress logs the host captures) plus `progress` JSON messages over stdout (structured signals for host UI like "currently walking task 3 of 7"). Rejected: a transient run file (duplicates markdown's journal, adds cleanup obligation, creates drift risk between two state sources); a persistent log file (no out-of-band sink — runtime is a subprocess, its log is its output streams). Consistent with §runtime-boundary principle 1 (markdown is source of truth).
- **Initial scope** — initial release ships six slash commands (`/gov:status`, `/gov:target`, `/gov:validate`, `/gov:implement`, `/gov:plan`, `/gov:specify`) plus the full MCP server. Three follow-on scenarios on this spec land `/gov:clarify`, `/gov:review`, `/gov:groom` in that order. The 6-command cut covers the pipeline backbone and the headline wall-clock wins (`/gov:validate`, `/gov:implement`) — substantial enough to be impactful and worthy of a version-bump release. The three deferred commands all share two properties (smallest acceleration value-add because predominantly LLM-driven; trickier extension-point ABI because user-mediated multi-turn or open-ended semantic), so deferring them lets the single-shot extension-point pattern prove out before the multi-turn pattern ships. Rejected: shipping fewer than six (leaves the headline `/gov:validate` and `/gov:implement` wins on the table); shipping all nine at once (introduces the trickiest extension-point ABI before the simpler one is proven).
- **Extension point ABI** — the runtime exposes two surfaces. (1) MCP server (`runtime mcp`) — every primitive as an MCP tool; the LLM walks slash command prose and calls primitives via MCP. Universal compatibility, zero per-host integration; used by Auggie and any other MCP-capable host. (2) Subprocess interpreter (`runtime exec <command>`) — the runtime drives the procedure end-to-end and uses JSON-over-stdio to invoke the LLM at extension points (`llm-request` / `llm-response` / `gate-confirm` / `gate-response` / `progress` / `complete` / `error` messages). Opt-in per host; Claude Code integrates first. The two surfaces share the same internal primitive implementations; agent hosts pick whichever fits their integration model. Rejected: MCP-with-continuation-tokens (server-side state across calls, protocol complexity, doesn't address the orchestration-tax problem cleanly); in-process library/callback (fragments adoption by language).
- **Install policy (the discovery half of distribution)** — the project does not auto-install the runtime (forbidden by §runtime-boundary principle 3 and 022's non-goal). Install discoverability lives in the README's Runtime section (rationale + concise instructions) and a one-line mention in the `/govern` bootstrap command's completion output pointing readers at the README. Slash commands MUST NOT detect-and-warn about the missing runtime on each invocation; the markdown-only path is a first-class path per principle 3, not a degraded mode. The technical-distribution half (which channels — GitHub release, `cargo install`, Homebrew, etc.) is still open under *Distribution channels*.
- **Procedure format and source** — prose IS the procedure. Slash command Instructions sections are rewritten to follow strict structural conventions (numbered steps, backtick-quoted primitive names, HTML-comment extension-point markers) that the runtime parses directly. No separate Procedure section, no twin maintenance, no drift-between-two-implementations to detect. Rejected: (a) YAML frontmatter (bloats metadata), (b) separate fenced code block (twin maintenance with a CI drift check is real cost for no benefit over a single source), (c) sibling `procedure.md` file (two files per command, drift risk, requires markdown-only adopters to know about both). Rejected source models: hand-authored separate Procedure section (twin maintenance), generated-from-structured-source with prose derived (breaks `framework/constitution.md` §text-first-artifacts principle 1). The cost is a one-time rewrite of every slash command file to match the conventions; the spec already commits to that rewrite. The CI parseability check on Instructions sections is the only drift-detection mechanism needed.
- **Prose-vs-procedure drift check** — superseded by the procedure-format resolution above. Prose is the single source, so there is no second implementation to drift from. The CI parseability check replaces the drift check; failure to parse under the structural conventions is a CI fail.
- **Scope reframing** — earlier drafts proposed wrapping four `/gov:validate` checks (Frontmatter schema, Anchor resolution, Dependency graph, Rule-ID existence). Rejected. The actual problem is the 2–4-minute wall-clock cost of LLM-mechanical orchestration across every command, not a small set of validate checks. The runtime is the procedure interpreter for govern, not a wrapper around individual primitives. Primitives are the runtime's library; the interpreter is the value.
- **Capability eligibility rule** — work is runtime-eligible only when its calling context has an LLM fallback. `/gov:*` commands are LLM-driven and qualify. Bash generators called by the pre-commit hook fail this rule (no LLM in the loop) and stay bash, unwrapped. `scripts/lint-tool-coverage.sh` runs in the workflow that asserts the runtime is absent and fails this rule too. Markdown lint (`npx markdownlint-cli2`) qualifies and is wrapped as the `lint-markdown` primitive.
- **Generator dry-runs as a top-level capability dropped** — wrapping generators behind a top-level runtime tool adds no reliability (bash is already deterministic) and no token cost (the LLM never executed YAML parsing on this path). The `run-generator` primitive remains as a thin wrapper available to procedures, but generator dry-runs are not a headline capability.
