---
section: "The primitive library"
---

# Write-session-primitive

## Context

The session state file at `<repo>/.govern.session.toml` (repo root, gitignored, TOML) is read by the `dashboard` primitive (`runtime/src/primitives/dashboard.rs`) but written only by the host. [`/{project}:target`](../../../framework/commands/target.md) step 7 and [`/{project}:amend`](../../../framework/commands/amend.md)'s scenario-route step 4 both invoke the host's file-writing tool for the session file, with prose specifying tempfile+rename atomic-write semantics.

On Claude Code this triggers a per-invocation `Write(.govern.session.toml)` permission prompt that documented `Write(...)` permissions entries in `framework/bootstrap/configure/claude.md` have not been able to suppress reliably across sessions. MCP tool calls go through a separate permission lane, so once a user allows `mcp__gvrn__write-session` the prompt is gone for the lifetime of the project.

The "host responsibility — the runtime exposes no session-shaped primitive" wording in `framework/commands/target.md` and `framework/commands/amend.md` is descriptive of the pre-0.9.0 state, not a constitutional constraint. Walking the §runtime-boundary eligibility criteria for the write path: deterministic (atomic-write of a tiny TOML doc is fully mechanical); currently mechanical (the prose already prescribes the exact atomic-write semantics); degradation-not-failure when removed (the markdown-only path keeps the host write). All three criteria pass, and the read path is already in the runtime — the write path is the asymmetry.

Spec 022 establishes the runtime's primitive library and the rule that state-modifying primitives use tempfile+rename. The session file is one of two durable journals named by the spec (markdown + `.govern.session.toml`, per [plan.md §No data persistence outside session file + markdown](../plan.md#no-data-persistence-outside-session-file--markdown)) — the runtime's write surface ought to cover both.

> **History note.** The first ship of `write-session` (gvrn 0.9.0) wrote `<repo>/.claude/gov-session.json` — a hardcoded path that baked in both the AI CLI's config directory (`.claude/` for Claude Code) and the adopting project's name (`gov-session.json` for this repo). That choice broke adopters whose project or AI CLI didn't match the runtime's baked-in constants (observed against `anvil` on Claude, whose session would have been `.claude/anvil-session.json`). The 0.10.0 consolidation moves the file to `.govern.session.toml` at the repo root: one path for every adopter, no `{cli-config-dir}` or `{project}` resolution.

## Behavior

New primitive: `write-session`. MCP tool name `write-session`; CLI subcommand `runtime write-session`.

Inputs (kebab-case CLI args; kebab-case TOML keys on disk):

- `feature` — required string, feature slug (e.g., `022-deterministic-runtime`).
- `path` — required string, repo-relative spec directory (e.g., `specs/022-deterministic-runtime`). Named `path` to match the existing convention.
- `scenario` — optional string, scenario slug. Both `scenario` and `scenario-path` MUST be supplied together or both omitted; supplying one without the other is a `PrimitiveError::MissingArgument` naming the absent field. Omitting both clears any previously set scenario.
- `scenario-path` — optional string, repo-relative scenario file path (e.g., `specs/022-deterministic-runtime/scenarios/write-session-primitive.md`). Required iff `scenario` is set.

Returns:

- `path` — repo-relative path of the written session file. Always `.govern.session.toml` post-0.10.0; kept on the result for symmetry with other write primitives.
- `created` — boolean; `true` when the file did not exist before this call, `false` when an existing file was overwritten.

Semantics:

- Writes to `<repo-root>/.govern.session.toml`. The path is hardcoded — there is no AI-CLI or project-name variability to parameterize. No parent directory is created (the repo root always exists when the runtime is invoked from inside the repo).
- Writes a TOML document whose top-level keys are exactly `feature`, `path`, optional `scenario`, optional `scenario-path`, and `set-at` (ISO 8601 UTC, the primitive's own clock). The pair `scenario` + `scenario-path` appears only when both inputs are present; their absence means "no scenario targeted." Keys are kebab-case to align with `.govern.toml`'s on-disk format.
- Uses the same tempfile-in-target-directory + atomic rename pattern the runtime's other state-modifying primitives (`mark-task`, `mark-criterion`, `set-status`) already share.
- TOML encoding: `toml::to_string` with field order matching the struct (feature, path, scenario, scenario-path, set-at) so the parity byte-equality check on `.govern.session.toml` is stable.

[`framework/commands/target.md`](../../../framework/commands/target.md) step 7 invokes this primitive on the deterministic path. [`framework/commands/amend.md`](../../../framework/commands/amend.md) scenario-route step 4 of the record block invokes it the same way. Both keep a markdown-only fallback that writes the TOML document by hand with the same atomic-write semantics.

[`framework/runtime-tools.txt`](../../../framework/runtime-tools.txt) carries the `write-session` line. The runtime version bumps to `0.10.0` for the consolidation (breaking: removes the `session-path` capability the runtime never exposed; changes the on-disk format from JSON to TOML; changes the file location). Parity coverage extends to the target command's deterministic path so the byte-equality check on `.govern.session.toml` is satisfied by the runtime-written file.

## Edge Cases

- **Existing session file with different shape (extra keys).** Overwritten in full. The primitive does not merge — the session file is owned by `/{project}:target` and the scenario branch of `/{project}:amend`, both of which write the complete record.
- **`scenario` provided without `scenario-path`, or vice versa.** `PrimitiveError::MissingArgument` with a message naming the absent field and the primitive (`write-session`). The pair is atomic by design.
- **Scenario file at `scenario-path` does not exist.** Primitive writes the path regardless; staleness is the caller's concern, matching `dashboard`'s tolerance for stale scenario paths in `load_session_target`.
- **Legacy `.claude/{project}-session.json` on disk.** Ignored by the primitive — `write-session` writes to `.govern.session.toml` only. The bootstrap migration (`session-file-consolidate`) translates and deletes legacy files on the next `/govern` run; adopters who skip the bootstrap will see a stale legacy file alongside the live `.govern.session.toml` until they delete it by hand.
- **Repo-root resolution.** Same conventions as every other primitive — the runtime resolves the repo root from the invocation context; the primitive does not accept an alternate root argument.
- **Concurrent invocations from the same session.** Tempfile+rename gives atomicity; last-writer-wins is acceptable. The session file is per-user, per-project; contention is not a real model.
- **Filesystem refuses the rename (cross-device, permission).** Surfaces as `PrimitiveError::Io`, halting the call. The prior host-write path had the same failure mode; nothing degraded.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

- **Session file location: parameterize via `session-path` arg, or consolidate onto a single repo-root file?** **Consolidate.** Rationale: the parameterized-path approach (caller passes `{cli-config-dir}/{project}-session.json` to `write-session` and `dashboard`) bakes the host-/project-specific variability into both primitive contracts and every command source that uses them. The host has to remember the path; the wire contract grows a required arg; doc-comments and MCP tool descriptions have to be careful not to commit to specific values. The consolidation has none of that surface area — the path is a constant the runtime knows, the wire contract stays small, and the file is uniform across every adopter. The cost is a one-time migration (`framework/migrations/session-file-consolidate.md`) for adopters who had a pre-0.10.0 session file; the bootstrap handles it on the next `/govern` run. Rejected: keeping the path host-specific (the original bug); a `[session]` section inside `.govern.toml` (mixes committed config with per-user state in one file, complicates gitignore).
- **TOML or JSON?** **TOML.** Rationale: matches `.govern.toml`'s on-disk format, so adopters see one config language at the project root rather than two. Kebab-case keys (`scenario-path`, `set-at`) align with `.govern.toml`'s convention rather than the legacy camelCase (`scenarioPath`, `setAt`).
