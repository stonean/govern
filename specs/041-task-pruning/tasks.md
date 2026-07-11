# 041 — Task Pruning Tasks

Tasks derived from the [plan](plan.md). Complete in order. Phase A ships the
`prune-tasks` runtime primitive; Phase B adds the `/{project}:prune` command on
top and registers it. The primitive (Phase A) must build and pass its parity
test before the command (Phase B) wires to it.

## Phase A — `prune-tasks` runtime primitive

### 1. Schema types

- [x] Add `PruneTasksArgs`, `PruneTasksResult`, the per-section record, and the `PruneMode` / `Classification` enums to `runtime/src/schema/primitives.rs`, following the kebab-case serde + `clap::Args` pattern (`feature`, `reset`, `force`, `apply` flags; the result excludes the file body).
- [x] Add a `prune_tasks_round_trip` serde test alongside the existing round-trip tests.
- **Done when**: `cargo build` compiles the new types and the round-trip test passes.

### 2. Segmentation, classification, and rebuild

- [x] Create `runtime/src/primitives/prune_tasks.rs` with `run(&PruneTasksArgs, &Path) -> Result<PruneTasksResult>`: segment via `detect_tasks_structure` / `parse_atx_heading` / `iter_phase_ranges`, and classify each task section `Spent` / `Pending` / `NoCheckbox` using `checkbox::find_checkbox_line`.
- [x] Implement keep-pending rebuild: drop `Spent` sections, drop phase containers with no surviving task section, preserve the preamble verbatim, normalize seams to one blank line, and set `nothing-to-prune` when nothing is spent.
- [x] Implement reset rebuild: existing first `# …` heading + a `CANONICAL_EMPTY_TASKS_BODY` constant, with a test asserting the constant equals `framework/templates/spec/tasks.md` minus its H1.
- [x] Implement the `--reset` status gate (read `spec.md` status; `allowed` / `blocked-needs-force`; `force` override; domain-outcome, not error) and the `apply` write via `write_atomic`.
- [x] Add `TasksFileMissing` and `MalformedTasks` variants to `PrimitiveError` and `pub mod prune_tasks;` in `runtime/src/primitives/mod.rs`.
- [x] Add inline `#[cfg(test)]` tests + fixtures under `runtime/tests/fixtures/primitives/` covering flat and phased spent-section removal, pending / no-checkbox preservation, empty-phase drop, keep-pending no-op, reset target, reset gate (done vs non-done vs `force`), missing `tasks.md`, and malformed `tasks.md`.
- **Done when**: all `prune_tasks` unit tests pass and preview output carries no file body.

### 3. Wire the primitive through CLI, MCP, interpreter, parser

- [x] `runtime/src/main.rs`: import the args, add `PruneTasks(PruneTasksArgs)` to `Command`, add the dispatch arm.
- [x] `runtime/src/mcp/server.rs`: add `"prune-tasks"` to `TOOL_NAMES` and a `#[tool(name = "prune-tasks", …)]` method.
- [x] `runtime/src/interpreter/mod.rs`: add `"prune-tasks" => call!(PruneTasksArgs, prune_tasks)`.
- [x] `runtime/src/parser/mod.rs`: add `"prune-tasks"` to `PRIMITIVE_NAMES`.
- **Done when**: the CLI subcommand, the MCP tool, and a `gvrn exec` procedure step all resolve the primitive.

### 4. Canonical manifest, generated config, release metadata

- [x] Add `prune-tasks` to `framework/runtime-tools.txt`.
- [x] Run `scripts/gen-configure-mcp.sh` and `scripts/gen-claude-commands.sh`; commit the regenerated configure allow-blocks.
- [x] Add a `runtime/CHANGELOG.md` `### Added` entry (new tool; list grows N→N+1) and bump `runtime/Cargo.toml` to the next minor version.
- **Done when**: the `runtime/tests/mcp.rs` parity test passes (`TOOL_NAMES` ↔ `runtime-tools.txt` ↔ served) and the generators report no drift.

### 5. Green gate

- [x] `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are all clean.
- **Done when**: the runtime workspace is green.

## Phase B — `/{project}:prune` command

### 6. Author the command source

- [x] Create `framework/commands/prune.md` with `description:` frontmatter and `{project}:` / `{cli-config-dir}/` placeholders. Instructions: resolve the session target; call `prune-tasks` preview; render the compact summary; route confirmation through `gate-confirm`; call `prune-tasks` apply; surface `blocked-needs-force`, `tasks-file-missing` (→ `/{project}:plan`), and no-target (→ `/{project}:target`). Annotate each step with a primitive name, an `<!-- llm:* -->` marker, or `<!-- audit:ignore-promotion -->`, and reference §runtime-host-integration once.
- **Done when**: the command documents both the runtime and markdown-only paths and states the single-artifact (`tasks.md`-only) scope.

### 7. Parseability

- [x] Run `gvrn parse --check framework/commands/prune.md`. If it parses cleanly, leave it; otherwise add `framework/commands/prune.md` to `runtime/legacy-prose-commands.txt`.
- **Done when**: `scripts/lint-procedure-parseability.sh` passes for `prune.md`.

### 8. Register and regenerate

- [x] Add the `'/{project}:prune' "$CMD_DIR/prune.md"` row to the appropriate group in `scripts/gen-help-tables.sh` (add a new `generated:commands-<group>` marker pair in `help.md` and a splice-loop entry only if prune warrants its own group).
- [x] Run the generators (or the pre-commit hook) to regenerate `framework/commands/help.md` and materialize `.claude/commands/gov/prune.md`; commit both.
- **Done when**: `gen-help-tables.sh --dry-run` and `gen-claude-commands.sh --check` report in-sync.

### 9. Docs

- [x] Add a `/prune` row to the hand-maintained `## Commands` section of `README.md`.
- **Done when**: the README lists prune in the correct group.

### 10. Full audit gate

- [x] Run `scripts/audit/run-all.sh` (check-zero + all families) and resolve any findings.
- **Done when**: the audit reports zero findings and the CI-equivalent checks pass.

## Phase C — Framework consistency (tasks.md is ephemeral tracking)

Surfaced by the pre-`done` durability review: `tasks.md` must be treated as disposable tracking end to end, not durable information.

### 11. Make the shared `tasks.md` parsers ignore HTML comments

- [x] In `runtime/src/primitives/mod.rs`, teach the shared line-walkers to skip content inside `<!-- … -->` HTML comments (single- and multi-line) exactly as they skip fenced blocks: `iter_task_numbers_at_levels`, `iter_phase_ranges`, and `section_lines` (so `detect_tasks_structure` follows).
- [x] Apply the same comment-skipping in `read_tasks.rs`, `mark_task.rs`, `check_stuck.rs`, and `prune_tasks.rs::segment` so every tasks parser agrees.
- [x] Add a regression test proving a reset (template-state) `tasks.md` parses to zero tasks and `append-task` returns number 1.
- **Done when**: `gvrn read-tasks` on a `--reset` file returns 0 tasks; `cargo test` / clippy / fmt clean.

### 12. Codify `tasks.md` as an ephemeral tracking artifact in the constitution

- [ ] Add a canonical statement (in `framework/constitution.md`, §tasks-phase or §text-first-artifacts) classifying `tasks.md` as an ephemeral work-tracking artifact — a view of what is left to do, safe to prune — distinct from the durable spec / scenarios / rules, with `plan.md` / `data-model.md` as design records.
- [ ] Update `framework/commands/prune.md` and this spec to cite that classification directly rather than by analogy to §bug-handling; reconcile the AGENTS.md artifact grouping so `tasks` is not read as a durable source of truth.
- **Done when**: the constitution names tasks.md's durability class explicitly; `resolve-anchor` and the framework audit are clean.

### 13. Relax `/{project}:analyze` scenario-consistency for pruned tasks

- [ ] Update `framework/commands/analyze.md` so the scenario-consistency check does not require a scenario's implementing task to persist in `tasks.md` after the scenario is implemented (a `done` spec with pruned scenario tasks is not a drift finding); regenerate the materialized command.
- **Done when**: analyze reports no false scenario-consistency finding for a `done` spec whose scenario tasks were pruned; the framework audit is clean.
