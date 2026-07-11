# 041 — Task Pruning Plan

Implements [041 — Task Pruning](spec.md).

## Overview

Ship `/{project}:prune` as a thin command over a new deterministic runtime
primitive, `prune-tasks`. The primitive does all mechanical work — parse
`tasks.md`, classify each task section as spent or pending, compute the
reduced (keep-pending) or template-state (reset) output, gate `--reset` on
spec status, and perform the atomic write — returning only a compact summary
that never carries the file body. The command owns judgment only: resolve the
session target, show the summary, take the destructive-write confirmation, and
call the primitive's apply pass. Parsing reuses the existing `tasks.md`
machinery so `prune-tasks` sees exactly the task set `read-tasks` and
`mark-task` see. See [data-model.md](data-model.md) for the segmentation,
classification, and request/response schema.

## Technical Decisions

### Reuse the `tasks.md` parser — no new grammar

`prune-tasks` builds its segmentation from the shared helpers in
`runtime/src/primitives/mod.rs`: `detect_tasks_structure` (Flat → task level
2, `## N.`; Phased → task level 3, `### N.` under `## …` containers),
`parse_atx_heading`, `iter_phase_ranges`, and `checkbox::find_checkbox_line`.
A task section's line range terminates at the next heading whose level is
`<= task_level` — the same rule `mark-task`'s `locate_task_range` uses. This
guarantees the section-boundary grammar the spec deferred to the plan matches
every other tasks primitive; no separate parser is introduced.

### Segmentation → classification → rebuild

The `run` function performs three passes: (1) segment the file into a
preamble and an ordered list of phase-heading / task-section blocks; (2)
classify each task section — `Spent` (≥1 checkbox, all checked), `Pending`
(any unchecked), `NoCheckbox` (zero checkboxes, always preserved); (3) rebuild
the output per mode. Classification counts checkboxes via
`checkbox::find_checkbox_line`, which already skips `- **Done when**:` lines.
The classification table and block model are specified in
[data-model.md](data-model.md).

### One primitive, two modes, an `apply` flag

`prune-tasks` takes `feature`, `reset: bool`, `force: bool`, `apply: bool`.
`apply: false` is a pure preview — it computes the segmentation, classification,
and gate decision and returns the compact summary (counts, per-section
classification, size before/after) **without writing and without ever
returning the file body**. `apply: true` recomputes and writes the reduced
`tasks.md` via the shared `write_atomic` (tempfile + rename). Keeping the body
inside the runtime on both passes is the token-reduction contract that
motivated making the primitive do its own write (resolved runtime-eligibility
question). The command calls preview → renders summary → confirms → calls
apply; the confirmation gate is preserved without round-tripping the file
through model context.

### keep-pending vs reset output

- **keep-pending** (`reset: false`): emit the preamble verbatim, keep every
  `Pending`/`NoCheckbox` section verbatim, drop every `Spent` section, and in
  phased files drop a `## …` phase container that has no surviving task
  section (no empty phases linger). Seams normalize to a single blank line
  with one trailing newline so the result is `markdownlint`-clean. When no
  section is spent the output equals the input → `nothing-to-prune: true`, no
  write.
- **reset** (`reset: true`): emit the file's existing first `# …` heading
  followed by a canonical empty-tasks body — a constant equal to
  `framework/templates/spec/tasks.md` with its own H1 removed (the intro line
  and guidance comment). A unit test asserts the constant matches that template
  body so they never drift. A file with no `# …` heading fails
  `malformed-tasks` and writes nothing.

### Status gate for `--reset`

When `reset` is true the primitive reads `spec.md` frontmatter `status`.
`status == done` (or `force: true`) → `gate: "allowed"`, proceed.
Otherwise → `gate: "blocked-needs-force"`, `applied: false`, no write — a
**domain outcome** carried in the result, not an operational error (matching
the `mod.rs` convention that domain results ride the struct). keep-pending
never reads `spec.md` (`status: null`, `gate: "not-applicable"`). The command
surfaces a blocked reset by naming the status, pointing at the keep-pending
default, and mentioning `--reset --force`.

### Command layer — judgment and single-artifact scope

`/{project}:prune` resolves the session target (erroring to `/{project}:target`
when none), maps `--reset`/`--force` to the primitive args, calls preview,
renders the summary, routes the confirmation through `gate-confirm`, then calls
apply. Its write surface is `tasks.md` only — never plan, spec, scenarios, or
status (resolved single-artifact boundary). Missing `tasks.md` surfaces as
`tasks-file-missing`, which the command translates to a "run
`/{project}:plan`" directive.

### Error taxonomy

Two new `PrimitiveError` variants in `runtime/src/primitives/mod.rs`:
`TasksFileMissing { root, feature }` and `MalformedTasks { path, reason }`.
`FeatureNotFound` is reused for a missing feature dir; `MissingSpecFile` /
`StatusFieldMissing` are reused when a `--reset` cannot read status. Each
error writes nothing.

### Runtime wiring (fully-wired primitive)

`prune-tasks` is wired through all seven registration sites so it is callable
from the CLI, MCP, and the `gvrn exec` walker:

1. `schema/primitives.rs` — `PruneTasksArgs` / `PruneTasksResult` / section
   record / classification+mode enums (kebab-case serde, `clap::Args` on the
   args struct) + a `prune_tasks_round_trip` serde test.
2. `primitives/prune_tasks.rs` — the `run` function + inline `#[cfg(test)]`.
3. `primitives/mod.rs` — `pub mod prune_tasks;` and the two new error variants.
4. `main.rs` — args import, `PruneTasks(PruneTasksArgs)` `Command` arm, dispatch.
5. `mcp/server.rs` — `"prune-tasks"` in `TOOL_NAMES` + a `#[tool(name =
   "prune-tasks", …)]` async method.
6. `interpreter/mod.rs` — `"prune-tasks" => call!(PruneTasksArgs, prune_tasks)`.
7. `parser/mod.rs` — `"prune-tasks"` in `PRIMITIVE_NAMES`.

Then the canonical manifest and generated config: add `prune-tasks` to
`framework/runtime-tools.txt` and regenerate the per-agent MCP allow-blocks via
`scripts/gen-configure-mcp.sh` (and `scripts/gen-claude-commands.sh` for the
installed command). The `runtime/tests/mcp.rs` parity test enforces
`TOOL_NAMES` ↔ `runtime-tools.txt` ↔ served-tools agreement, so the manifest
line is mandatory. Finish with a `runtime/CHANGELOG.md` `### Added` entry and a
minor `runtime/Cargo.toml` version bump (lockstep versioning).

### Command authoring and registration

`framework/commands/prune.md` is the single hand-authored source, written with
`{project}:` / `{cli-config-dir}/` placeholders (the Family-4 placeholder
round-trip audit). Its `description:` frontmatter feeds the generated help
table. Each `## Instructions` step carries a backticked primitive name
(`prune-tasks`, `gate-confirm`), an `<!-- llm:* -->` marker, or an
`<!-- audit:ignore-promotion -->` annotation (Family-9 promotion audit), and the
file references §runtime-host-integration once so the tool-coverage lint passes.
The command is authored to parse under `gvrn parse --check`; if the
preview→confirm→apply prose resists the parser, `framework/commands/prune.md`
is added to `runtime/legacy-prose-commands.txt` as the documented escape hatch.
Registration is generator-driven: add the row to `scripts/gen-help-tables.sh`,
then run the generators (or the pre-commit hook) to regenerate
`framework/commands/help.md` and materialize `.claude/commands/gov/prune.md`.
Adopter materialization (`/govern` bootstrap, `gov:init`) enumerates
`framework/commands/*.md` dynamically — no per-command edit there. Slash
commands are not individually permission-gated, so no `settings` entry is
needed beyond the `prune-tasks` MCP allow-block that already flows from
`runtime-tools.txt`. The whole rollout is verified by
`scripts/audit/run-all.sh` (check-zero), which fails on a stale help table, a
missing materialized command, an unparseable/unlisted command, or a placeholder
violation.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `specs/041-task-pruning/data-model.md` | Create | Segmentation, classification, and `prune-tasks` schema (done) |
| `runtime/src/schema/primitives.rs` | Edit | `PruneTasksArgs`/`PruneTasksResult`/section+mode types + round-trip test |
| `runtime/src/primitives/prune_tasks.rs` | Create | The `prune-tasks` `run` function + unit tests |
| `runtime/src/primitives/mod.rs` | Edit | `pub mod prune_tasks;` + `TasksFileMissing`/`MalformedTasks` error variants |
| `runtime/src/main.rs` | Edit | CLI subcommand: import, `Command` arm, dispatch |
| `runtime/src/mcp/server.rs` | Edit | `TOOL_NAMES` entry + `#[tool]` method |
| `runtime/src/interpreter/mod.rs` | Edit | `dispatch_primitive` match arm |
| `runtime/src/parser/mod.rs` | Edit | `PRIMITIVE_NAMES` allowlist entry |
| `framework/runtime-tools.txt` | Edit | Canonical manifest entry `prune-tasks` |
| `framework/bootstrap/configure/claude.md` | Regenerate | MCP allow-block (via `gen-configure-mcp.sh`) |
| `framework/bootstrap/configure/auggie.md` | Regenerate | MCP allow-block (via `gen-configure-mcp.sh`) |
| `runtime/tests/mcp.rs` | Edit | Behavioral integration test (parity test auto-covers registration) |
| `runtime/CHANGELOG.md` | Edit | `### Added` entry (new tool; list grows N→N+1) |
| `runtime/Cargo.toml` | Edit | Minor version bump (lockstep) |
| `framework/commands/prune.md` | Create | Authoritative `/{project}:prune` command source (placeholders, step annotations) |
| `runtime/legacy-prose-commands.txt` | Edit (conditional) | Escape hatch if `prune.md` doesn't parse as a Procedure |
| `scripts/gen-help-tables.sh` | Edit | Add the prune row to a command-group table |
| `framework/commands/help.md` | Regenerate | Help table (via `gen-help-tables.sh`) |
| `.claude/commands/gov/prune.md` | Regenerate | Materialized command (via `gen-claude-commands.sh`) |
| `README.md` | Edit | Add `/prune` to the hand-maintained Commands section |

## Trade-offs

- **Section is the atomic unit; interiors are never edited.** Rejected
  stripping completed checkboxes out of still-pending sections: it would force
  renumbering and intra-section reference rewrites, dragging judgment into a
  mechanical primitive, and would discard the working context a half-done
  section's checked items provide. Cost: a long-lived section with many
  completed boxes and one straggler stays large until that box is checked.
  Accepted — that residue is small and self-clearing.
- **Primitive writes; preview never returns the body.** Rejected a
  preview-only primitive that hands the proposed content back for the agent to
  write: it round-trips the whole file through model context twice, defeating
  the runtime's token-reduction purpose. Cost: the primitive owns a write path
  (more surface than a pure computation) and the confirmation becomes a
  two-call preview→apply dance. Accepted — the summary-only contract is the
  point.
- **Reset re-emits a canonical body constant.** Rejected reading the tasks
  template at runtime (adds a template-path dependency and a missing-template
  failure mode) and rejected preserving the working preamble verbatim (a
  normal filled file has already stripped the guidance comment, so it wouldn't
  satisfy "restores … heading plus guidance comment"). Cost: a constant that
  must track the template, guarded by a drift test.
- **Empty phase containers are dropped in keep-pending.** Rejected preserving
  them (leaves noisy empty `## Phase …` headings). Cost: a phase the user
  intends to refill loses its heading; re-planning re-adds it. Accepted —
  keep-pending targets a lean working set.
- **Single-artifact scope.** Rejected reconciling `plan.md` task references
  (interpretive, cross-artifact, judgment-heavy). Cost: a plan may mention
  tasks prune removed; `/{project}:analyze` surfaces genuine drift as advisory.

## Known limitations

- Prune classifies by checkbox state alone; a section left fully checked but
  not actually merged (a mis-checked box) is treated as spent. This matches
  every other tasks primitive's trust of checkbox state.
- `--reset --force` on a live spec can strand it with no runnable tasks until
  `/{project}:plan` repopulates; this is the deliberate, explicit escape hatch,
  not silent behavior.
