---
status: draft
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

### Declarative slash command procedures

Slash command markdown files in `framework/commands/*.md` adopt a layered format:

- Existing prose sections (Purpose, Context, Scope Boundaries, Instructions) remain as the LLM-readable specification, unchanged or with minimal edits.
- A new **Procedure** section is added per command, declaring ordered steps and primitive calls in a machine-parseable form (format TBD — see Open Questions).
- Each step that requires semantic judgment names an **LLM extension point** with a request payload, response schema, and a pointer to the prose fallback instruction.

The runtime parses the Procedure section. A markdown-only adopter (no runtime on `PATH`) ignores it and walks the prose Instructions exactly as today. The prose remains authoritative — no semantic content lives in the Procedure section that isn't also in the prose. Drift between the two is caught by a deterministic check added to the markdown-only-pipeline workflow.

### The interpreter

The interpreter is the runtime binary's main entry point. Given a slash command name and arguments, it:

1. Loads the command's Procedure.
2. Walks the steps in order.
3. For each primitive step, calls the corresponding primitive operation.
4. For each LLM extension point, prepares the structured request, surfaces it to the LLM via the agent host, validates the structured response, and continues.
5. Manages pipeline state — session file, status transitions, gate confirmations, checkbox updates — through primitives.
6. Returns when the procedure completes, halts at a gate, errors, or is cancelled.

The interpreter is stateless across invocations. All state lives in the markdown source of truth and the session file.

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

## LLM extension points

An extension point is a named seam in a procedure where the runtime suspends and invokes the LLM with a structured request. Each extension point has:

- A unique identifier (e.g., `writeCode`, `writeSpecBody`, `askClarifyQuestion`).
- A schema for the request payload the runtime prepares.
- A schema for the response payload the LLM produces and the runtime validates.
- A prose instruction in the slash command's Instructions section describing what the LLM should do — this is the markdown-only fallback.

The initial extension point inventory, drawn from the genuinely semantic moments in each command:

- `writeCode` — `/gov:implement` walk-through-tasks step 4. Request: task description, plan-relevant files, write boundary. Response: file edits.
- `writeSpecBody` — `/gov:specify` and `/gov:plan` template-fill moments.
- `askClarifyQuestion` — `/gov:clarify` open-question loop.
- `assessSpecQuality` — `/gov:validate`'s per-rule Verification reads.
- `performReview` — `/gov:review` semantic pass.
- `routeInboxItem` — `/gov:groom` routing decision.

The inventory is non-final — the full set of extension points is part of clarify.

## Markdown-only path

When the runtime is absent from `PATH`, the LLM walks the slash command's prose Instructions as it does today. Procedure sections are ignored. The bash scripts under `scripts/` remain authoritative for the primitives that have bash counterparts (`gen-*.sh`, `lint-frontmatter.sh`, `lint-tool-coverage.sh`); the LLM calls them via the existing prose.

The opt-in invariant from spec 021 — CI proves the markdown-only path completes — fires unchanged. It now proves that the same procedures execute correctly with two interpreters (the LLM and the runtime), and that adding the Procedure sections has not broken the prose path.

## Slash command rewiring

This spec rewires every slash command markdown file in `framework/commands/*.md` to add a Procedure section. Coverage is staged across scenarios on this spec — see Open Questions on initial scope. Each rewire:

- Preserves existing prose Instructions verbatim or with minimal edits.
- Adds the Procedure section parseable by the runtime.
- Names extension points consistently across commands.
- Passes the prose-vs-procedure drift check after the edit.

Commands ordered roughly by deterministic share (highest first): `/gov:status`, `/gov:target`, `/gov:validate`, `/gov:implement`, `/gov:plan`, `/gov:specify`, then the semantic-heavy `/gov:clarify`, `/gov:review`, `/gov:groom`. The first two are the cheapest to ship and validate the architecture; the next two are the largest wall-clock wins; the semantic-heavy commands are the trickiest extension-point design.

## CI integration

Two workflows coexist after this spec lands:

- `.github/workflows/markdown-only-pipeline.yml` (existing, from spec 021) — proves the markdown-only path completes with the runtime absent from `PATH`. Same five checks as today, plus an added prose-vs-procedure drift check across slash command files.
- `.github/workflows/runtime.yml` (new) — builds the binary, runs its test suite, exercises every primitive against fixture inputs, exercises every slash command's CLI subcommand against a fixture repo, and produces release artifacts. Triggers only on changes to runtime source paths.

The two workflows are independent. The runtime workflow MUST NOT install the binary into the markdown-only workflow's environment.

## Bash script relationships

Stable relationships post-rewrite:

- **`gen-*.sh`** (called by the pre-commit hook) — stay bash. The runtime never replaces them: pre-commit has no LLM in the loop, so they fail the eligibility rule from §runtime-boundary principle 3. The runtime's `run-generator` primitive is a thin wrapper for procedure use; pre-commit continues to call the bash scripts directly.
- **`lint-frontmatter.sh`** — repositioned as the markdown-only fallback for the runtime's `validate-frontmatter` primitive. Same intent, two implementations, both ship; the prose Instructions invoke whichever is available.
- **`lint-tool-coverage.sh`** — stays bash-only with no runtime counterpart. The lint runs exclusively in `markdown-only-pipeline.yml`, which asserts the runtime is absent; a runtime version is unreachable in that workflow.

## Acceptance Criteria

- [ ] A single binary builds and runs from this repo, providing a CLI subcommand for each primitive and each slash command.
- [ ] Every primitive is exposed as an MCP tool, named per the convention resolved in clarify.
- [ ] Every slash command exposed by govern has a Procedure section parseable by the runtime; the existing prose Instructions are preserved as the markdown-only specification.
- [ ] LLM extension points are named consistently across commands; each has a request schema, response schema, and a prose fallback instruction in the command's Instructions section.
- [ ] A prose-vs-procedure drift check is added to `.github/workflows/markdown-only-pipeline.yml` and passes against every slash command file.
- [ ] The binary executes `/gov:status`, `/gov:target`, and `/gov:validate` end-to-end against a fixture repo and produces output consistent with the LLM-driven path against the same fixture, within the determinism bounds defined for each command.
- [ ] Median wall-clock time per `/gov:validate` invocation against a target spec drops from minutes to seconds when the runtime is present.
- [ ] The markdown-only path (no binary on `PATH`) continues to complete every pipeline cycle (greenfield, brownfield, reopen) as it did before this spec.
- [ ] `framework/runtime-tools.txt` is populated with every MCP tool name the binary exposes; `scripts/lint-tool-coverage.sh` passes against the rewritten slash command files.
- [ ] A new CI workflow at `.github/workflows/runtime.yml` builds the binary, runs its test suite, exercises every primitive against fixture inputs, and fails on any test failure.
- [ ] The existing `.github/workflows/markdown-only-pipeline.yml` workflow continues to pass with the runtime binary absent from `PATH`.
- [ ] When a runtime primitive crashes mid-procedure, the slash command falls back per the semantics resolved in clarify.
- [ ] The binary's version-coordination behavior matches the policy resolved in clarify.
- [ ] The binary is distributed per the channel(s) resolved in clarify; adopter-facing install instructions exist in the README.
- [ ] `/gov:validate` against this spec passes with no hard-fail or blocking findings.
- [ ] `npx markdownlint-cli2` against all rewritten slash command files and new spec files passes.

## Non-Goals

- Replacing the LLM at semantic extension points — that work is by definition outside the runtime.
- Replacing the slash command markdown files with a different source format. Markdown stays the source of truth; Procedure sections are added alongside prose, not in place of it.
- Rewriting, wrapping, or otherwise interacting with the bash generator scripts (`gen-*.sh`) — pre-commit context, no LLM fallback, not runtime-eligible per §runtime-boundary principle 3.
- Persisting any state outside the markdown source of truth and the session file — runtime state is derived and gitignored per principle 1.
- Auto-installing the runtime on `/govern` adoption — opt-in per principle 3.
- Daemon mode, long-running services, or background processes — per the non-scope list in §runtime-boundary.
- Reimplementing `npx markdownlint-cli2` natively — the runtime wraps it as a primitive.
- Building any non-MCP integration surface (LSP, web UI, REST). Speculative; would need its own spec under §runtime-boundary's eligibility criteria.

## Open Questions

- **Procedure format and source** — three viable shapes: (a) YAML block in frontmatter, (b) a structured fenced code block in the body, (c) a sibling `procedure.md` file per command. Tradeoffs across LLM readability, parser complexity, and drift detection. A related question: is the Procedure section hand-authored (twin maintenance), generated from prose by a deterministic parser (parser brittleness), or generated from a structured source with prose derived (breaks markdown-as-source-of-truth)? Resolution shapes most of the rest of this spec.
- **Prose-vs-procedure drift check** — what is the deterministic similarity test that catches divergence between prose Instructions and Procedure steps? Step-name alignment? Heading-to-step match? Open pending the procedure format decision.
- **Extension point ABI** — JSON over stdio? Structured tool calls via MCP? An in-process callback the agent host provides? Affects which agent hosts can use the runtime and how cleanly it embeds in Claude Code vs other surfaces.
- **Initial scope** — which slash commands ship in the first release and which are added incrementally via scenarios on this spec? `/gov:status` and `/gov:target` are the cheapest to validate the architecture; `/gov:validate` is the largest wall-clock win; semantic-heavy commands (`/gov:clarify`, `/gov:review`) are the trickiest extension-point design.
- **State management within a procedure run** — the runtime is stateless across invocations, but within a single procedure run it tracks position, gates passed, and partial results. In-memory only, or written to a transient run file? Affects crash recovery and observability.
- **Partial-failure semantics** — when a primitive fails (file unreadable, git command non-zero, gate denied, network blip during MCP), does the procedure halt, ask the LLM via an extension point, or fall back to the prose Instructions for the remaining steps? Likely a per-primitive policy, not a single global rule.
- **Per-command opt-out** — should an adopter be able to disable runtime acceleration for a specific command while keeping it for others (e.g., trust the runtime for `/gov:status` but not yet for `/gov:implement`)? Or is it all-or-nothing per session?
- **Implementation language** — Rust, Go, or other? Rust has the most mature MCP library ecosystem and produces small static binaries; Go has faster build times and simpler cross-compilation. Both fit §runtime-boundary's eligibility criteria. Project-level decision.
- **MCP tool naming convention** — `gov-rt:<verb>-<noun>` or some other shape? Affects `framework/runtime-tools.txt` content and `scripts/lint-tool-coverage.sh` matching.
- **Distribution channel** — GitHub release artifact only? Add `cargo install` / `go install` / Homebrew? Adopter install path is separate from the markdown-only opt-in invariant.
- **Versioning enforcement** — how does the binary verify it matches the framework's schemas? Refuse on mismatch, warn and continue, or rely on schema-evolution discipline? Tied to the lockstep-versioning policy from §runtime-boundary.
- **Runtime-failure fallback semantics** — when a runtime tool is invoked but crashes (not absent — actually fails), does the slash command silently fall back to LLM execution, surface the error and halt, or both? Distinct from the absent-binary case the opt-in invariant already covers.

## Resolved Questions

- **Scope reframing** — earlier drafts proposed wrapping four `/gov:validate` checks (Frontmatter schema, Anchor resolution, Dependency graph, Rule-ID existence). Rejected. The actual problem is the 2–4-minute wall-clock cost of LLM-mechanical orchestration across every command, not a small set of validate checks. The runtime is the procedure interpreter for govern, not a wrapper around individual primitives. Primitives are the runtime's library; the interpreter is the value.
- **Capability eligibility rule** — work is runtime-eligible only when its calling context has an LLM fallback. `/gov:*` commands are LLM-driven and qualify. Bash generators called by the pre-commit hook fail this rule (no LLM in the loop) and stay bash, unwrapped. `scripts/lint-tool-coverage.sh` runs in the workflow that asserts the runtime is absent and fails this rule too. Markdown lint (`npx markdownlint-cli2`) qualifies and is wrapped as the `lint-markdown` primitive.
- **Generator dry-runs as a top-level capability dropped** — wrapping generators behind a top-level runtime tool adds no reliability (bash is already deterministic) and no token cost (the LLM never executed YAML parsing on this path). The `run-generator` primitive remains as a thin wrapper available to procedures, but generator dry-runs are not a headline capability.
