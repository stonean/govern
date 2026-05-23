---
section: "The primitive library"
---

# Write-session-primitive

## Context

The session JSON at `.claude/gov-session.json` (Claude) is read by the `dashboard` primitive (`runtime/src/primitives/dashboard.rs:267-294`) but written only by the host. [`/gov:target`](../../../framework/commands/target.md) step 7 and [`/gov:ask`](../../../framework/commands/ask.md)'s scenario-route step 4 both invoke the host's file-writing tool for the session file, with prose specifying tempfile+rename atomic-write semantics.

On Claude Code this triggers a per-invocation `Write({cli-config-dir}/{project}-session.json)` permission prompt that the documented `Write(...)` permissions entries in `framework/bootstrap/configure/claude.md` have not been able to suppress reliably across sessions. MCP tool calls go through a separate permission lane, so once a user allows `mcp__gvrn__write-session` the prompt is gone for the lifetime of the project.

The "host responsibility — the runtime exposes no session-shaped primitive" wording in `framework/commands/target.md` and `framework/commands/ask.md` is descriptive of the current state, not a constitutional constraint. Walking the §runtime-boundary eligibility criteria for the write path: deterministic (JSON write with tempfile+rename is fully mechanical); currently mechanical (the prose already prescribes the exact atomic-write semantics); degradation-not-failure when removed (the markdown-only path keeps the host write). All three criteria pass, and the read path is already in the runtime — the write path is the asymmetry.

Spec 022 establishes the runtime's primitive library and the rule that state-modifying primitives use tempfile+rename. The session file is one of two durable journals named by the spec (markdown + `.claude/gov-session.json`, per [plan.md §No data persistence outside session file + markdown](../plan.md#no-data-persistence-outside-session-file--markdown)) — the runtime's write surface ought to cover both.

## Behavior

New primitive: `write-session`. MCP tool name `write-session`; CLI subcommand `runtime write-session`.

Inputs (kebab-case CLI args; camelCase JSON fields on the wire):

- `feature` — required string, feature slug (e.g., `022-deterministic-runtime`).
- `path` — required string, repo-relative spec directory (e.g., `specs/022-deterministic-runtime`). Named `path` to match the existing session-JSON convention used by every fixture under `runtime/tests/fixtures/*/.claude/gov-session.json` and by host-written sessions in adopter repos.
- `scenario` — optional string, scenario slug. Both `scenario` and `scenario-path` MUST be supplied together or both omitted; supplying one without the other is a `PrimitiveError::MissingArgument` naming the absent field. Omitting both clears any previously set scenario.
- `scenario-path` — optional string, repo-relative scenario file path (e.g., `specs/022-deterministic-runtime/scenarios/write-session-primitive.md`). Required iff `scenario` is set.

Returns:

- `path` — repo-relative path of the written session JSON.
- `created` — boolean; `true` when the file did not exist before this call, `false` when an existing file was overwritten.

Semantics:

- Resolves the session path the same way `dashboard` does: `<repo-root>/.claude/gov-session.json`. The primitive creates the `.claude/` directory if absent.
- Writes a JSON object whose top-level fields are exactly `feature`, `path`, optional `scenario`, optional `scenarioPath`, and `setAt` (ISO 8601 UTC, the primitive's own clock — matches the field name already used by every fixture session and by the host-written form). The pair `scenario` + `scenarioPath` appears only when both inputs are present; their absence means "no scenario targeted." `scenarioPath` is camelCase on the wire to match the dashboard primitive's reader (`runtime/src/primitives/dashboard.rs:258`).
- Uses the same tempfile-in-target-directory + atomic rename pattern the runtime's other state-modifying primitives (`mark-task`, `mark-criterion`, `set-status`) already share.
- JSON encoding: two-space indent via `serde_json::to_string_pretty`, with a single trailing newline appended. Field order matches the order shown above (feature, path, scenario, scenarioPath, setAt) — emitted by using an `IndexMap`-style serialization or an explicit struct with `serde`'s field order, so the parity byte-equality check on `.claude/gov-session.json` is stable.

[`framework/commands/target.md`](../../../framework/commands/target.md) step 7 is rewritten to invoke this primitive on the deterministic path. [`framework/commands/ask.md`](../../../framework/commands/ask.md) scenario-route step 4 of the record block is rewritten the same way. Both keep a markdown-only fallback that does the JSON write by hand with the same atomic-write semantics.

[`framework/runtime-tools.txt`](../../../framework/runtime-tools.txt) gains the `write-session` line. The runtime version bumps from `0.8.1` to `0.9.0` (minor — additive tool surface, mirroring the precedent set when `dashboard` was added). Parity coverage extends to the target command's deterministic path so the byte-equality check on `.claude/gov-session.json` is satisfied by the runtime-written file.

## Edge Cases

- **`.claude/` directory absent.** Primitive creates it before writing. Mirrors the way other write primitives handle missing parent directories under the repo root.
- **Existing session file with different shape (extra fields).** Overwritten in full. The primitive does not merge — the session file is owned by `/gov:target` and the scenario branch of `/gov:ask`, both of which write the complete record.
- **`scenario` provided without `scenario-path`, or vice versa.** `PrimitiveError::MissingArgument` with a message naming the absent field and the primitive (`write-session`). The pair is atomic by design.
- **Scenario file at `scenario-path` does not exist.** Primitive writes the path regardless; staleness is the caller's concern, matching `dashboard`'s tolerance for stale scenario paths in `load_session_target` (`runtime/src/primitives/dashboard.rs:278-282`).
- **Repo-root resolution.** Same conventions as every other primitive — the runtime resolves the repo root from the invocation context; the primitive does not accept an alternate root argument.
- **Concurrent invocations from the same session.** Tempfile+rename gives atomicity; last-writer-wins is acceptable. The session file is per-user, per-project; contention is not a real model.
- **Filesystem refuses the rename (cross-device, permission).** Surfaces as `PrimitiveError::Io`, halting the call. The prior host-write path had the same failure mode; nothing degraded.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
