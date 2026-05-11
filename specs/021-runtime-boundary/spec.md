---
status: in-progress
dependencies: [020-code-review]
review:
  last-run: 2026-05-10T00:00:00Z
  reviewed-against: 3d7c50beb1aa9e82783cb2a7f9ed5b0540068625
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 021 — Runtime Boundary

Establish the constitutional scope, eligibility criteria, and opt-in invariant for an optional deterministic runtime that adopters may install alongside the markdown framework. This spec ships the constitutional amendment and the CI invariant that makes it enforceable. No binary, no MCP tools, no slash-command rewiring — those are scoped to a follow-up spec (022).

## Motivation

`govern`'s constitution declares text-first artifacts load-bearing: "adopting `govern` requires no bootstrap tooling beyond the AI agent itself" (§text-first-artifacts). That principle is correct as stated for the markdown framework but forecloses an emerging need: deterministic execution of mechanical checks and fixes that today are LLM-executed at meaningful token cost and probabilistic reliability.

The motivating evidence is concrete. [020-code-review](../020-code-review/plan.md)'s plan acknowledges directly: "we cannot bolt on a deterministic linter without changing the framework's shape." The CI gate it ships uses `awk`-based YAML parsing because no binary exists; the same blocking check is implemented three times (in `/gov:implement`, in `/gov:validate`, and in CI bash) because there is no single deterministic enforcer; cross-pass dedupe and idempotency hashing are punted to LLM judgment despite being purely mechanical.

This spec changes the constitution's shape — deliberately, with bounded scope — to permit an optional runtime that absorbs deterministic work while preserving the load-bearing properties: markdown is source of truth, the agent's write path stays simple, PRs review glanceably, and adopters who install nothing still complete every pipeline cycle.

## Constitutional Amendment

### §text-first-artifacts opening paragraph (edit)

The opening paragraph is amended to acknowledge the optional runtime without weakening the markdown source-of-truth claim. Current load-bearing properties (Edit-driven write path, glanceable PRs, rare merge conflicts, markdown-as-source-of-truth) are preserved verbatim. The clause "adopting `govern` requires no bootstrap tooling beyond the AI agent itself" is replaced by language that distinguishes the markdown framework (standalone, no tooling) from the optional runtime (opt-in, see §runtime-boundary).

### §runtime-boundary (new subsection)

A new subsection containing five elements:

**Five principles** establishing what the runtime can and cannot do, stated in RFC 2119 keywords consistent with the rule format declared in §rules:

1. **Markdown is source of truth** — the runtime MUST NOT own state the markdown cannot reconstruct. Runtime-owned data (caches, indexes, parsed graphs) is derived and gitignored, per the existing rule on structured derived views.
2. **Determinism only** — the runtime MUST NOT call an LLM. Work requiring semantic judgment (content quality, `/clarify` resolution, `/capture` sketching, per-rule Verification reads, `/groom` routing) stays in slash commands.
3. **Opt-in for adopters** — the runtime MUST NOT be a prerequisite for any pipeline gate. A markdown-only adopter — agent + `Edit`, no binary on `PATH` — must complete every cycle (greenfield, brownfield, reopen) and reach `done` on every spec.
4. **Schema follows the constitution** — the runtime reads frontmatter and artifact structure according to the schemas declared in this document. Schema changes ship through the constitution; the runtime updates to match. The constitution does not import runtime types.
5. **MCP is the seam** — the runtime exposes its capabilities as MCP tools. Slash commands call those tools when they want determinism, keeping the runtime accessible to any agent host and preventing `govern`-specific coupling.

**Three eligibility criteria** for moving a capability into the runtime — a capability is runtime-eligible only when **all three** hold:

1. **Deterministic** — no semantic judgment required; the same inputs always produce the same outputs.
2. **Currently mechanical** — already either (a) executed by an LLM following procedural instructions in a slash command body, or (b) implemented as a bash script invoked by `govern` workflows.
3. **Degradation, not failure, when removed** — without the runtime, the work still completes correctly via the markdown-only path; only speed, cost, or reliability degrades.

**Opt-in invariant** — the repository's CI MUST include a job that exercises a representative pipeline cycle end-to-end with the runtime binary absent from `PATH`. A change that causes this job to fail — i.e., a slash command that silently requires the runtime — is a constitution violation, not a feature.

**Versioning** — the runtime ships in lockstep with the framework. A `govern` release includes the binary built against the schemas in that release; an adopter's `govern` version pins their compatible runtime version, eliminating schema/runtime drift as a failure mode.

**Explicit non-scope** — the runtime is NOT a spec authoring tool, NOT a workflow orchestrator, NOT a long-running service, NOT a storage layer. Lifting any of these exclusions requires a constitutional amendment.

### §drift-prevention canonical sources (edit)

A row is added to the canonical sources table in §drift-prevention naming §runtime-boundary as the source of truth for runtime contract questions, so future work that touches the runtime knows where the binding statements live.

## Acceptance Criteria

- [x] `framework/constitution.md` §text-first-artifacts opening paragraph is amended to reference an optional runtime; the load-bearing properties (Edit, glanceable PRs, rare conflicts, markdown source-of-truth) remain in the paragraph verbatim.
- [x] `framework/constitution.md` contains a new `§runtime-boundary` subsection with the five principles, the three eligibility criteria, the opt-in invariant, the versioning rule, the non-scope list, and a one-line forward pointer to spec 022.
- [x] The five principles use RFC 2119 keywords (MUST / MUST NOT) consistent with the rule format declared in §rules.
- [x] The non-scope list uses MUST NOT for each excluded item, in the same RFC 2119 register as the five principles.
- [x] An anchor marker `<!-- §runtime-boundary -->` exists at the new subsection so `/gov:validate`'s anchor-resolution check resolves references to it.
- [x] `framework/constitution.md` §drift-prevention canonical sources table contains a row whose Fact column names "Runtime contract / boundary," pointing at `framework/constitution.md` §runtime-boundary.
- [x] A CI workflow exists in this repo that asserts: (a) no runtime binary on `PATH`, (b) all bash generator scripts (`gen-spec-deps.sh`, `gen-readme-table.sh`, `gen-help-tables.sh`) run clean in `--dry-run`, (c) `npx markdownlint-cli2` passes, (d) a slash-command-runtime-fallback lint scans `framework/commands/*.md` and verifies every reference to a runtime tool is wrapped in a graceful-fallback pattern, and (e) frontmatter integrity (status enum and dependencies-list shape) holds across all spec and scenario files.
- [x] The CI workflow above runs on every PR that modifies `framework/`, `specs/`, or `.claude/commands/`.
- [x] The existing principle bullets in §text-first-artifacts (markdown by default, frontmatter for metadata, relative links not wiki-links, derived views gitignored, exceptions require amendment) are unchanged — only the opening paragraph is edited.
- [x] `/gov:validate` against this spec passes (no hard-fail or blocking findings) once acceptance criteria are met.
- [x] `npx markdownlint-cli2` against `framework/constitution.md` and the new spec files passes after the edits.

## Non-Goals

- Building the runtime binary — deferred to spec 022.
- Moving any existing `/gov:validate` deterministic checks into a runtime — deferred to spec 022.
- Defining MCP tool schemas — deferred to spec 022.
- Choosing the runtime's implementation language (Rust, Go, etc.) — project-level decision, not constitutional.
- Release engineering, distribution, or version-coordination tooling for the binary — project-level.
- Editing slash command files to call runtime tools — deferred to spec 022.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **§runtime-boundary placement** — subsection within §text-first-artifacts (not a sibling top-level section). The runtime boundary is a clause of the text-first principle ("text is source of truth; runtime is a derived view"), not a standalone load-bearing concept. Subsection placement mirrors the existing structure (Principles, Frontmatter Schema, Validation Severity, then Runtime Boundary) and makes the dependency on text-first explicit. Future amendments can lift the subsection out if scope demands it; that is a small mechanical edit when the need materializes.
- **Opt-in CI invariant — composition of the "representative pipeline cycle"** — the invariant runs as a deterministic check job, not an LLM-driven cycle. Concretely the CI job asserts: (1) the runtime binary is absent from `PATH`, (2) all bash generator scripts (`gen-spec-deps.sh`, `gen-readme-table.sh`, `gen-help-tables.sh`) run clean in `--dry-run`, (3) `npx markdownlint-cli2` passes, (4) a slash-command-runtime-fallback lint scans `framework/commands/*.md` for references to runtime tools and verifies each is wrapped in a graceful-fallback pattern, and (5) frontmatter integrity (status enum + dependencies list shape) is checked programmatically. LLM-driven commands (`/gov:specify`, `/gov:clarify`, `/gov:plan`, etc.) are deliberately excluded — they're LLM-driven by design and the runtime cannot subsume them; their fallback safety is enforced structurally by check (4) at the source-file level. The invariant is a tripwire, not a smoke test: vacuously satisfied today, it fails the moment spec 022 wires a slash command to a runtime tool without a fallback. Precedent: spec 020's plan rejected running `/gov:review` in CI for the same reason — LLM-in-CI introduces API budget and non-determinism.
- **"What the runtime is not" wording** — normative, using MUST NOT, to match the RFC 2119 register of the five principles in the same subsection. Final phrasing: *"to prevent scope creep, the runtime MUST NOT be a spec authoring tool, MUST NOT be a workflow orchestrator, MUST NOT be a long-running service, and MUST NOT be a storage layer. Lifting any of these exclusions requires a constitutional amendment."* Rationale: descriptive prose would create an inconsistency within the subsection (the five principles are MUST/MUST NOT) and would make the existing "Lifting requires amendment" sentence incoherent — "lifting" is meaningless against descriptive language.
- **Opt-in CI invariant — when it runs** — every PR that modifies `framework/`, `specs/`, or `.claude/commands/`, starting now (not deferred until the binary exists). The invariant is a tripwire that must already be running before the first runtime-touching PR so that PR is the one that fails. It is vacuously satisfied today (no command references a runtime tool yet) at the cost of trivial workflow-runner minutes. The path filter ensures docs-only or release-only PRs skip the workflow.
- **Capability listing in §runtime-boundary** — capability-agnostic, with a single forward pointer. §runtime-boundary defines the boundary; actual capabilities live in their introducing specs (beginning with spec 022). Mirrors how §rules works: the constitution defines the rules tier and schema; actual rules live in `specs/{rule-set}.md` and are not enumerated in the constitution. A one-line pointer at the end of the subsection — *"Specific capabilities are introduced through their own feature specs, beginning with spec 022 (runtime v0)."* — anchors the reader without pinning content. Reader-side concreteness already lives in this spec's Motivation section (awk YAML parsing, three-place gate, cross-pass dedupe); the constitution does not need to repeat it.
- **Runtime-eligibility note requirement on slash commands** — no requirement. A hand-authored "deterministic vs. semantic" note on each slash command source is exactly the anti-pattern the Design Principles section in `AGENTS.md` forbids: it depends on author diligence and fails silently when an author forgets. The substantive invariant — that any runtime-tool reference in a slash command has a graceful fallback — is already enforced by the CI fallback lint (check 4 from the opt-in invariant resolution above), which is derived from the command source rather than from an author-supplied marker. If richer signals are wanted later, they are extracted by static analysis, not by hand.
