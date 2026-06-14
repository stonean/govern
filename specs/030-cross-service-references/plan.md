# 030 — Cross-Service References Plan

Implements [030 — Cross-Service References](spec.md).

## Overview

Cross-service references are informative links from a spec to a spec in another service, surfaced with the linked spec's lifecycle status. Implementation spans five surfaces, all landing inside this feature:

1. **Registry** — a `.govern.toml [services]` table mapping a service alias to its canonical repo and local checkout path.
2. **Harvest** — a generator that extracts cross-service URL links from a spec body into a derived `references:` frontmatter index, kept strictly distinct from `dependencies:`.
3. **Runtime** — a new `gvrn` primitive that resolves each reference through the registry, reads the linked spec's `status` from the local checkout, and classifies the outcome. The markdown-only path performs the same work with host file tools; the primitive is the fast path, never a prerequisite.
4. **Commands** — `/{project}:status` surfaces the resolved status; `/{project}:analyze` reports a provably-broken reference as a finding.
5. **Constitution** — a §spec-lifecycle carve-out so reference edits don't reopen a `done` spec, plus a `references:` row in the §text-first-artifacts frontmatter schema.

Comprehensive tests are a first-class deliverable (per the planning decision): Rust unit tests per outcome, generator tests, markdown-only↔runtime parity with golden output, and the no-runtime CI job exercising the fallback.

## Technical Decisions

### D1 — Registry: `.govern.toml [services]`

A `[services.<alias>]` table with two string fields: `repo` (canonical URL, the identity matched against body-link hrefs) and `path` (local checkout, relative to repo root or absolute). The table is optional — absent means no cross-service resolution, and a single-service adopter writes nothing.

`.govern.toml` is the shared adopter-side database (per `AGENTS.md`); `[services]` is a new table documented in this spec's `data-model.md`. Adding it does **not** generate a §cross-spec-impact signpost on spec 019. The schema is declared canonically in `data-model.md` and the runtime reads per that schema (§runtime-boundary principle 4). Entries are added with the `/{project}:link` command (D6), not derived — `path` is machine-local knowledge `govern` cannot infer. An optional `description` annotates an entry with the service's purpose; it is **informational only** (surfaced for orientation, never branched on), which keeps it clear of the no-human-diligence principle that rejects optional *load-bearing* inputs.

### D2 — Reference syntax and harvesting

A reference is a standard markdown link whose href is the linked spec's canonical repo URL (settled in the spec). A **new** generator, `scripts/gen-cross-service-refs.sh`, harvests body links whose host+repo matches a registered `[services]` entry into a derived `references:` frontmatter field, keyed `{service, spec-slug}`, ignoring the URL's branch ref. It honors the same exclusions as `gen-spec-deps.sh` (fenced blocks, blockquotes, `## See also`).

A **separate** generator (not an extension of `gen-spec-deps.sh`) keeps informative references cleanly partitioned from the blocking sibling-dependency graph — they must never share an index or enter the cycle check. Each generator edits only its own frontmatter field; neither clobbers the other.

The index lives as a **committed, derived `references:` frontmatter field**, parallel to `dependencies:`. It is reconstructable from body links (so it satisfies §runtime-boundary principle 1 — the runtime owns no state the markdown can't rebuild) and stays glanceable on GitHub. An on-demand harvest with no persisted field was considered and rejected: the spec commits to an *index*, and a persisted field keeps `/{project}:status` fast and mirrors the `dependencies:` precedent.

### D3 — Runtime primitive: `resolve-references`

A new primitive in `runtime/src/primitives/resolve_references.rs` following the established pattern (`run(args, repo) -> Result<…>`, outcomes as data, `PrimitiveError` reserved for operational failures). Input: the consumer feature plus repo root. For each entry in the consumer's `references:` index it resolves the service via `[services]` and emits a resolution record `{ service, spec, outcome, status? }`.

The outcome is decided by deterministic predicates — no prose read for intent:

| Outcome | Predicate |
| --- | --- |
| `ok` | service registered, checkout path exists, target `spec.md` exists, `status` present and in the allowed set → record the status |
| `unregistered` | href repo matches no `[services]` entry |
| `not-checked-out` | registered, but `path` missing / not a usable checkout |
| `broken` | registered + reachable, but the target spec does not exist (renamed/deleted/mistyped) or the URL is malformed |
| `status-unreadable` | target file exists but `status` is missing / malformed YAML / out of the allowed set / a scenario (no status) |

It reuses the `validate-frontmatter` machinery (`read_text`, `split_frontmatter`, `ALLOWED_STATUSES`) against the resolved external path. Registered in `primitives/mod.rs`, exposed in `mcp/server.rs`, types in `schema/primitives.rs`. The markdown-only fallback (D4) performs the identical classification with host file tools.

### D4 — Command integration and the markdown-only fallback

- `/{project}:status` gains a reference-status readout per spec: each reference shows its outcome and, on `ok`, the linked lifecycle status. The command prose carries the markdown-only procedure (read `.govern.toml`, resolve path, read linked frontmatter `status`, classify) as the runtime-absent path; the `resolve-references` primitive is the runtime path. Neither wraps the other.
- `/{project}:analyze` reports a `broken` outcome as an **Advisory** finding — surfaced on every run, but non-blocking, because references are informative and non-load-bearing. (`unregistered` / `not-checked-out` are *not* findings; they are informational unknowns — the can't-check vs. provably-broken line from the spec.)

### D5 — Constitution and frontmatter schema

- **§text-first-artifacts (Frontmatter Schema)** — add a `references:` row to the spec-file schema: generator-managed, derived from body links, distinct from `dependencies:`, not hand-authored.
- **§spec-lifecycle** — add an explicit carve-out: an edit whose entire diff adds/removes/changes cross-service reference links is mechanical-class (non-reopening); a `done` spec stays `done`. Worded so the exemption is determinable from the diff alone (the changed link's target resolves to a registered cross-service reference), consistent with the existing mechanical-vs-meaningful test.
- **§drift-prevention (Canonical sources)** — add a row naming this spec's `data-model.md` as the canonical source for the `[services]` registry schema.
- `AGENTS.md` mirrors the §spec-lifecycle interaction and the new generator in the contributor-side notes.

Template-rule alignment holds without a template change: a freshly scaffolded spec has no references, so the new `/{project}:analyze` broken-reference check passes by default.

### D6 — Registration command `/{project}:link`

A new slash command registers a service in `[services]` — chosen over hand-edit-only for two reasons: it surfaces the capability in `/{project}:help` and the README (an adopter discovers it; a `.govern.toml` table they would not), and it guarantees well-formed TOML rather than leaving structure to the author. Registration is *not* derived — `path` is machine-local knowledge `govern` cannot infer.

- **Flow:** prompts for each field one at a time — alias, then repo URL, then local path, then an optional `description` (enter to skip) — validating as it goes, the same one-at-a-time interaction `/{project}:clarify` uses. Inline args (`/{project}:link <alias> <repo> <path> [--description <text>]`) are accepted as an optional shortcut.
- **Validation (per field, as entered):** alias is a valid, unique TOML key (no clobber of an existing entry); `repo` is URL-shaped; `path` is recorded as written but *warns* — does not block — when it does not currently resolve, since `not-checked-out` is a valid state.
- **Write:** additive — adds the `[services.<alias>]` block and leaves every other `.govern.toml` table intact (the additive discipline already used for `.mcp.json` and permission merges). The markdown-only path edits `.govern.toml` with host file tools; a small deterministic write primitive may serve as the fast path, but `gvrn` is never required.
- **`--list`:** shows registered services and, when reachable, their resolution health. Removal stays a hand-edit.
- **Docs:** the command's row lives in the README **Orient** section and `/{project}:help`; the `unregistered` outcome (D4) points the user at `/{project}:link`.

The command adds surface: a source under `framework/commands/`, wiring through the command / help-table / permission generators, per-agent permission entries, the command manifest, and tests.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `specs/030-cross-service-references/data-model.md` | Create | Registry, reference, index, and outcome-enum schema |
| `scripts/gen-cross-service-refs.sh` | Create | Harvest cross-service URL links → `references:` frontmatter |
| `runtime/src/primitives/resolve_references.rs` | Create | `resolve-references` primitive |
| `runtime/src/primitives/mod.rs` | Modify | Register the primitive |
| `runtime/src/mcp/server.rs` | Modify | Expose the MCP tool |
| `runtime/src/schema/primitives.rs` | Modify | Args/Result types + outcome enum |
| `runtime/src/schema/extensions.rs` | Modify | `[services]` table type |
| `framework/commands/link.md` | Create | `/{project}:link` registration command source |
| `framework/commands/status.md` | Modify | Surface reference status (both paths) |
| `framework/commands/analyze.md` | Modify | Broken-reference Advisory finding |
| `framework/bootstrap/configure/*.md` | Modify | Per-agent permission entries for `/{project}:link` |
| `framework/constitution.md` | Modify | §text-first-artifacts schema row, §spec-lifecycle carve-out, §drift-prevention canonical-source row |
| `framework/runtime-tools.txt` | Modify | Register the new tool name for the opt-in check |
| `.github/workflows/markdown-only-pipeline.yml` | Modify | Run the new generator `--dry-run`; exercise the fallback |
| `.github/workflows/generators.yml` | Modify | Run the new generator |
| `scripts/install-hooks.sh` | Modify | Wire the new generator into the pre-commit hook |
| `runtime/tests/fixtures/cross-service-*` | Create | Consumer spec + fake registered checkouts per outcome |
| `runtime/tests/parity/cross-service/*` | Create | Markdown-only↔runtime parity |
| `runtime/tests/golden/cross-service-*.jsonl` | Create | Golden resolution records |
| `README.md` | Modify | Document `[services]`, the `/{project}:link` command (Orient section), and cross-service references |
| `AGENTS.md` | Modify | Contributor-side notes (generator, §spec-lifecycle interaction) |

## Trade-offs

- **Separate generator vs. extending `gen-spec-deps.sh`** — chose separate, so informative references can never leak into the blocking dependency/cycle graph. Cost: a second generator to wire into the hook and CI.
- **Committed `references:` frontmatter vs. gitignored cache** — chose committed frontmatter for glanceability and GitHub visibility, consistent with `dependencies:`. Cost: a little frontmatter noise; benefit: reviewable and reconstructable.
- **Broken-reference severity: Advisory vs. Blocking** — chose Advisory. References are informative and non-gating, so a broken one is surfaced every run but never blocks pipeline advancement. Limitation: a broken reference can linger; mitigated by repeated surfacing.
- **Local-checkout-only (no fetch)** — a reference to an unchecked-out service shows `not-checked-out` rather than resolving. Accepted per the spec's Non-Goals; keeps resolution deterministic and CI simple.
- **Self-reference** — a URL pointing at the consumer's own repo resolves like any other registered service (minimal special-casing); documented as a limitation rather than a guarded error.
- **Slash command vs. hand-edit-only registration** — chose `/{project}:link`. Hand-editing `.govern.toml` works but is undiscoverable and error-prone; a command surfaces in `/{project}:help` / README and guarantees formatting. Registration stays non-derived (`path` is machine-local). Cost: a new command's full generator/permission/test surface.
