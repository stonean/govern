---
section: "Follow-on scenarios"
---

# Dashboard-primitive

## Context

The [`/gov:status`](../../../framework/commands/status.md) full-dashboard path (steps 3, 4, 6 of the prose Instructions) has no deterministic single-call surface in the runtime today. The existing primitive set offers `read-spec` per spec, but no `list-specs`, no artifact-existence check, no `.govern.toml` reader. An agent walking the prescribed deterministic path pays ~3N+ MCP round-trips for N specs (one `read-spec` each, plus N existence checks the runtime doesn't expose at all, plus a config read it also doesn't expose).

Faced with that cost, agents reliably fall back to shell — `ls -d specs/[0-9][0-9][0-9]-*`, a `for` loop over `plan.md` / `tasks.md` / `data-model.md` / `scenarios/`, and `cat .govern.toml` — which the §Instructions preamble explicitly forbids ("do not substitute shell utilities (`awk`, `sed`, `grep` pipelines, `for` loops over files) for the prescribed file reads"). The ban has no positive target to point at, so the violation is also the only path that completes in interactive time.

Observed 2026-05-22 against this repo: a `/gov:status` invocation with the session target at `done` triggered the full-dashboard path; the agent executed `ls`, a shell `for` loop over every spec dir, and a `cat .govern.toml` rather than ~30+ MCP round-trips.

Spec 022 establishes the command-specific-primitive pattern (`check-stuck` for `/gov:implement`, `derive-boundary` for `/gov:plan`, `check-rule-ids` for `/gov:review`, `enforce-manifest` for `/govern`). The dashboard is the same shape: a `/gov:status`-specific deterministic kernel that the markdown can hand off to in one call.

## Behavior

New primitive: `dashboard`. MCP tool name `dashboard`; CLI subcommand `runtime dashboard`.

Inputs: none beyond the standard repo-root context every primitive operates on. The primitive reads `.govern.toml` (committed config) and `.govern.session.toml` (gitignored session state) at the repo root; both paths are constants, neither is caller-supplied.

Returns one structured payload:

- `session-target` — `{feature: string, scenario: string|null, scenario-detail: {section, context-summary, open-question-count}|null}` when `<repo-root>/.govern.session.toml` exists, otherwise `null`. The `scenario-detail` field is populated when `scenario` is non-null, giving callers everything they need to render the scenario header line without a separate read. Lets MCP callers skip a separate session-file read; the subprocess-interpreter surface continues to seed walker context from the same file.
- `specs` — array, one entry per `NNN-feature` directory under `specs/`, in directory-name order:
  - `slug` — the directory basename
  - `status` — frontmatter status (`draft` | `clarified` | `planned` | `in-progress` | `done`)
  - `dependencies` — frontmatter `dependencies` array (empty when absent)
  - `tags` — frontmatter `tags` array (empty when absent)
  - `open-question-count` — body count of unresolved questions, matching `read-spec`'s existing semantics
  - `has-plan`, `has-tasks`, `has-data-model` — booleans for file existence (no read)
  - `scenarios-count` — count of `*.md` files under `scenarios/` when the directory exists, `0` otherwise
  - `blocked-by` — array of dependency slugs from this spec's `dependencies` whose own `status` is below `clarified`; empty when the spec is unblocked. The caller renders the "blocked specs" callout straight from a non-empty `blocked-by`.
- `tags-union` — top-level sorted, deduplicated union of every spec's `tags` arrays; empty when no spec has tags. The caller suppresses the "tags in use" callout exactly when this is empty.
- `config` — object describing `.govern.toml` review state:
  - `present` — boolean; `true` when `.govern.toml` exists at repo root, `false` otherwise
  - `disabled-rule-files` — array of basenames from `[[review.disabled-rule-files]]`; empty when the section is absent or the array is empty

Reasons (the human-readable `reason` field on each disabled-rule-files entry) are not surfaced — the dashboard is a glance, not a pretty-printer; `.govern.toml` remains the source of truth for full context, matching the existing prose rule.

The status command collapses to a single path that always invokes `dashboard` once and renders the preamble + table + counts + callouts from the returned payload. The prior "short-circuit when target is non-`done`" branch is removed — a glance at the full pipeline with the target row marked is more useful than a target-only view, and the preamble line preserves the immediate "what next" signal. Step 5 (table render), step 6's callout formatting, and step 7 (non-done spec prompt) stay in the prose — they are presentation, not data acquisition.

The §Instructions preamble's shell-utility ban gains a positive target: when `dashboard` is available via MCP, that IS the deterministic path; when the host's MCP schema is loaded lazily (Claude Code's deferred-tool reminder), the existing `ToolSearch` instruction applies — fetch the schema and call the tool, do not bail to shell.

The primitive is read-only and pure with respect to filesystem state (no atomic-write concerns, no rollback). Standard partial-failure semantics apply: malformed frontmatter on any single spec is an operational error, halts the call, emits structured `error`.

## Edge Cases

- **Empty `specs/` directory.** Returns `specs: []`, `config` populated normally. No error — a fresh project with no specs yet is a valid state.
- **`NNN-feature` directory missing `spec.md`.** Treat as an operational error (the directory naming convention promises a spec); halt with structured `error` naming the offending directory. Matches `read-spec`'s behavior on missing files and avoids silently hiding broken state from a diagnostic command.
- **Spec directory whose name doesn't match the `NNN-feature` pattern.** Skip silently (not part of the dashboard inventory). `specs/inbox.md`, `specs/templates/`, ad-hoc notes, etc., are out of scope.
- **`.govern.toml` absent.** Returns `config: {present: false, disabled-rule-files: []}`. The caller distinguishes "no config" from "config with empty array" via the `present` flag to drive the markdown's callout-suppression rule.
- **`.govern.toml` present but `[[review.disabled-rule-files]]` section absent or its array empty.** Returns `config: {present: true, disabled-rule-files: []}`. Caller suppresses the callout per the existing prose rule.
- **`.govern.toml` parse failure.** Operational error, halts the call. Consistent with other primitives' handling of malformed-input files at the repo root.
- **`scenarios/` exists but contains non-markdown files.** `scenarios-count` reflects only `*.md` files, matching the existing prose ("count the markdown files in its scenarios subdirectory").
- **`.govern.session.toml` absent.** `session-target: null`. No error — a fresh project with no `/{project}:target` invocation yet is a valid state.
- **`.govern.session.toml` present but malformed.** Operational error (`PrimitiveError::Toml`), halts the call. Consistent with `.govern.toml` parse failures.
- **Legacy `.claude/{project}-session.json` on disk.** Ignored — `dashboard` reads `.govern.session.toml` only. Adopters who haven't yet run the post-0.10.0 `/govern` bootstrap migration see "no target" until they re-`/{project}:target` or the migration translates the legacy file.
- **Session file present but its `feature` field names a directory that doesn't exist.** Return the session-target field as-recorded; do not validate against the `specs` array. The caller already handles this — `/{project}:target` is the corrective action, not `/{project}:status`.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Primitive name: `dashboard` vs `list-specs` vs `read-pipeline-state`?** **`dashboard`.** Rationale: matches the runtime's existing command-specific-primitive pattern (`check-stuck` for `/gov:implement`, `derive-boundary` for `/gov:plan`, `check-rule-ids` for `/gov:review`, `enforce-manifest` for `/govern`) — the primitive is the deterministic kernel of `/gov:status`'s dashboard step, and its payload shape mirrors the rendered dashboard table 1:1. `list-specs` undersells the `.govern.toml` and `session-target` fields (the payload is more than an inventory). `read-pipeline-state` overstates scope — pipeline state is broader (gates, in-flight tasks, last-review timestamps) and none of that lives in this primitive. The verb-noun convention from spec 022's primitive-naming question stands: `dashboard` reads as a noun, paralleling `gate-confirm`'s noun form already in the manifest.
- **Blocked-specs computation: in the primitive or in the caller?** **In the primitive.** Per-spec `blocked-by` is an array of dependency slugs whose `status` is below `clarified`, empty when the spec is unblocked. Rationale: the rule has no semantic content — it is a fixed predicate over fields the primitive already returns. Pushing it to the caller would put deterministic work on the LLM, exactly the orchestration loop spec 022's motivation eliminates. The earlier "rule might evolve" framing was speculative future-proofing of the kind §design-principles rejects; today's rule is stable and a future change ships as a runtime patch on the same cadence as every other command-specific primitive. The caller renders the "blocked" callout straight from a non-empty `blocked-by`; the callout text quotes the array.
- **`tags` union for the per-repo callout: primitive or caller?** **In the primitive.** Same shape and same reasoning as `blocked-by`: pure deterministic fold over per-spec `tags` arrays, no semantic content, no reason to walk it from the LLM. The payload adds a top-level `tags-union` field — the sorted, deduplicated union across every spec's `tags` array, empty when no spec has tags. The caller suppresses the "tags in use" callout exactly when `tags-union` is empty.
- **Should `dashboard` also be invoked on the short-circuit path (session target not `done`)?** **Yes — invoke unconditionally; the short-circuit path is removed.** Rationale: `/gov:status` collapses to one path that always renders the full dashboard, with the session target's row marked `>>` and a short preamble line above the table showing the target and its next action. A glance at the full pipeline is more useful than the truncated target-only view; users who only want the target line already get it from the preamble. The single-path design also eliminates the branching prose (steps 2.1 vs 2.2 in the current command) and matches the runtime's broader direction of collapsing per-command logic into one deterministic call. The primitive returns enough about the session target (via the existing per-spec entry plus a new `session-target.scenario-detail` field when a scenario is targeted) that the caller renders both the preamble and the full table from a single response.
- **Version bump: patch or minor?** **Minor — `gvrn 0.8.0`.** Rationale: the change adds a new MCP tool name to the runtime surface — `dashboard` joins the canonical `TOOL_NAMES` list, the `framework/runtime-tools.txt` manifest, the parity-test matrix, and ships an additional CLI subcommand. None of those touch an existing primitive's JSON schema, so this isn't a breaking change — but it is additive surface, which is what minor bumps signal in this runtime. Precedent: `gvrn 0.7.0` was the minor bump for the writeCode payload-bundling work (added fields, no breakage); the same logic applies here at a slightly larger scale (a whole new tool name). Patch bumps in this runtime have been reserved for behavior-tightening fixes that leave the tool surface unchanged (`gvrn 0.5.2` for `check-stuck-tasks-md-advancement`, `gvrn 0.7.3` for `writecode-payload-canonicalize-paths`). The Q4 framework-side rewrite of `framework/commands/status.md` is a prose change, not a runtime ABI change, so it doesn't push the bump higher.
