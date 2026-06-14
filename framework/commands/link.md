---
description: Register a service so cross-service references resolve to its lifecycle status.
argument-hint: "[alias repo path [--description <text>] | --list]"
parity:
  strict-files:
    - ".govern.toml"
---

# Link

Register a service in `.govern.toml` so cross-service references to it resolve to the linked spec's lifecycle status.

## Purpose

Records a service in the `.govern.toml` `[services]` registry — its canonical repository URL and its local checkout path — so a cross-service reference whose href matches that repo can surface the linked spec's status (see [030 — Cross-Service References](../../specs/030-cross-service-references/spec.md)). The registry is **required for status resolution, optional for referencing**: a reference whose repo matches no `[services]` entry stays a plain navigational link (the `unregistered` outcome). Registration is not derived — the local `path` is machine-local knowledge `govern` cannot infer — so it is captured here rather than harvested.

## Context

This command does not require a session target — it edits the project-level `[services]` registry in `.govern.toml`, not any feature spec. The repo URL is **identity and navigation only**: it is recorded verbatim and is **never fetched**. Status is read later from the local `path`, never over the network.

## Scope Boundaries

- Read and write only `.govern.toml`. Do NOT modify any spec, plan, scenario, source file, or session state.
- Do NOT fetch the repo URL or reach across the network. The local checkout `path` is the only state read at resolution time, and that resolution belongs to `/{project}:status` and `/{project}:analyze`, not to this command.
- Removal and edits of an existing entry stay hand-edits to `.govern.toml`.
- Reference: §text-first-artifacts, §runtime-boundary, and [030 `data-model.md`](../../specs/030-cross-service-references/data-model.md) — the canonical source for the `[services]` schema (constitution loaded by `/{project}:target`; do not re-read).

## Instructions

This command is host-driven: it has no runtime primitive, so the same procedure runs whether or not the `gvrn` runtime is installed — both the deterministic and the markdown-only paths converge on the host's file tools reading and writing `.govern.toml`. The full procedure — argument forms, per-field validation, the additive write, and `--list` — is in the **Procedure** reference below.

<!-- audit:ignore-promotion -->
1. Parse the arguments. With `--list`, run the **List the registry** branch (read-only) and stop. With three positional values (alias, repo, path) plus an optional `--description <text>`, take them as pre-supplied field values, validate them, and prompt only for anything missing or invalid. With no arguments, collect every field interactively.

2. <!-- llm:registerService --> Run the registration flow: prompt for each missing field one at a time — alias, then repo URL, then local path, then an optional description (empty to skip) — validating each value as it is entered (see **Validation**). Then write the `[services.<alias>]` block additively to `.govern.toml`, preserving every other table (see **Additive write**). A duplicate alias is rejected before any write; a duplicate repo is warned but allowed.

<!-- audit:ignore-promotion -->
3. Confirm the result: echo the resulting `[services.<alias>]` block, surface any path-resolution warning, and point the user at `/{project}:status` to see the linked spec's status once the service is checked out.

## Procedure

The host always walks this procedure — there is no runtime shortcut. It is the contract the `registerService` step fulfills.

### Argument forms

- `/{project}:link` (no arguments) — interactive: prompt for every field one at a time.
- `/{project}:link <alias> <repo> <path> [--description <text>]` — shortcut: the positional values pre-supply the fields. They are still validated as if typed; prompt only for anything missing or rejected.
- `/{project}:link --list` — read-only: list the registry and stop. `--list` never combines with field arguments.

### Interactive registration

Prompt one field at a time, the same one-at-a-time interaction `/{project}:clarify` uses. Validate each field before moving to the next; on invalid input, explain the problem and re-prompt the same field. Do not batch the prompts.

1. **alias** — the short, stable handle for the service (e.g., `api`).
2. **repo** — the canonical repository URL.
3. **path** — the local checkout location.
4. **description** (optional) — a one-line note on the service's purpose; empty input skips it.

### Validation

Per field, as entered:

- **alias** — a valid bare TOML key (letters, digits, hyphens, underscores; no whitespace, dots, or quotes) and **not already present** as a `[services.<alias>]` table in `.govern.toml`. A collision is **rejected** with no write — changing an existing entry is a hand-edit. This is the duplicate-alias rejection.
- **repo** — URL-shaped (a scheme such as `https://` and a host). Recorded verbatim; it is identity and navigation only and is never fetched. If the same URL is already registered under another alias, **warn** — a duplicate repo makes the match ambiguous (a registry finding) — but allow the write.
- **path** — the local checkout location, relative to the repo root or absolute, recorded exactly as written. If it does not currently resolve to a directory, **warn** but do **not** block: a missing checkout is the valid `not-checked-out` state, surfaced at resolution time, not a config error.
- **description** — free text, optional, informational only; no behavior depends on it.

### Additive write

- Read `.govern.toml` (create it with just the new block if it is absent).
- Add a `[services.<alias>]` table with `repo`, `path`, and `description` (omit `description` when skipped). Preserve every other table — `[host]`, `[project]`, `[pinned]`, `[migrations]`, `[workflows]`, `[review]`, and any sibling `[services.*]` entries — byte-for-byte. This is the additive discipline already used for `.mcp.json` and permission merges.
- Write atomically (tempfile + rename).

### List the registry

- Read `.govern.toml`. If `[services]` is absent or empty, report that no services are registered and show the register form (`/{project}:link <alias> <repo> <path>`).
- Otherwise, for each registered service show its alias, `repo`, `path`, the `description` when present, and a resolution-health line: **resolves** when the `path` exists as a checkout, **not checked out** when it is missing or unusable. Listing reads only the registry and the existence of each local `path` — it does not read linked specs or touch the network.

### Notes

- The `[services]` schema (`repo`, `path`, optional `description`) is declared canonically in [030 `data-model.md`](../../specs/030-cross-service-references/data-model.md).
- A duplicate `repo` across two aliases is a registry-validation finding (ambiguous match), surfaced on resolution — this command warns at registration time but does not block it.
- Outcome semantics (`ok`, `unregistered`, `not-checked-out`, `broken`, `status-unreadable`) belong to `/{project}:status` and `/{project}:analyze`; this command only records the registry the resolver reads.
