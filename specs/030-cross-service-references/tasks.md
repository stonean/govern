# 030 — Cross-Service References Tasks

Tasks derived from the [plan](plan.md). Complete in order. Tests are first-class — each implementation task carries its own verification.

## 1. Registry schema (`[services]`)

- [x] Add the `[services.<alias>]` type (`repo`, `path`, optional `description`) in a new `runtime/src/schema/services.rs` module, parsed from `.govern.toml`; absent table → empty set.
- [x] Confirm `data-model.md` matches the implemented schema (`repo` / `path` / optional `description`). (The §drift-prevention canonical-source row is Task 8.)
- [x] **Done when:** `[services]` parses, a duplicate `repo` is detectable, and a missing table is a no-op; unit tests cover present/absent/duplicate.

## 2. `/{project}:link` registration command

- [x] Create `framework/commands/link.md`: register a service in `[services]` — **prompt for each field one at a time** (alias → repo → path → optional description), validating as entered (unique TOML key, URL-shaped repo, not-checked-out warning on the path); inline positional args remain an optional shortcut; additive write that preserves other `.govern.toml` tables; `--list` shows registered services + resolution health.
- [x] Wire through the command/help-table/permission generators and add per-agent permission entries in `framework/bootstrap/configure/*.md`; register in the command manifest.
- [x] **Done when:** `/{project}:link` adds a well-formed `[services]` block without disturbing other tables, rejects a duplicate alias, warns on an unresolved path, lists services with `--list`, and appears in `/{project}:help`; command tests pass.

## 3. Harvest generator

- [x] Create `scripts/gen-cross-service-refs.sh`: harvest body links whose repo matches a registered `[services]` entry into the `references:` frontmatter field; ignore the branch ref; honor the fenced-block / blockquote / `## See also` exclusions; never touch `dependencies:`.
- [x] Wire it into `scripts/install-hooks.sh` (pre-commit) and `.github/workflows/generators.yml`.
- [x] **Done when:** `--dry-run` is clean on the repo; shell tests cover a matching link (harvested), an unregistered link (recorded with null service), a `## See also` link (excluded), a branch-ref variation (same identity), and confirm `dependencies:` is untouched.

## 4. `resolve-references` primitive + unit tests

- [x] Add `runtime/src/primitives/resolve_references.rs` with `run(args, repo)`; reuse `read_text` / `split_frontmatter` / `ALLOWED_STATUSES`; classify each reference into the closed outcome enum.
- [x] Add Args/Result/outcome types to `runtime/src/schema/primitives.rs`; register in `runtime/src/primitives/mod.rs` and expose in `runtime/src/mcp/server.rs`.
- [x] **Done when:** the crate builds and Rust unit tests (tempdir fixtures with fake registered checkouts) cover all five outcomes — `ok`, `unregistered`, `not-checked-out`, `broken` (missing target + malformed URL), `status-unreadable` (no frontmatter / malformed YAML / out-of-set / scenario target) — plus a self-reference.

## 5. Markdown-only fallback

- [x] Write the runtime-absent procedure into the command prose (Tasks 6–7): read `.govern.toml`, resolve `path`, read the linked frontmatter `status`, classify — using host file tools only, no shell-pipeline substitution.
- [x] **Done when:** the prose path and the primitive produce identical resolution records for the same fixtures; `lint-procedure-parseability.sh` passes.

## 6. `/{project}:status` integration

- [ ] Update `framework/commands/status.md` so the status payload includes per-reference resolution records and the dashboard surfaces outcome + linked status, plus the service `description` for orientation when present. The `unregistered` outcome points the user at `/{project}:link`.
- [ ] **Done when:** `status.md` documents both paths; the `status-basic` parity fixture/golden are extended with a reference and pass.

## 7. `/{project}:analyze` broken-reference finding

- [ ] Update `framework/commands/analyze.md`: a `broken` outcome is an Advisory finding (distinct from the informational unknowns `unregistered` / `not-checked-out`); the `unregistered` surfacing suggests `/{project}:link`.
- [ ] **Done when:** `analyze.md` documents the check; the `analyze-basic` parity fixture/golden cover a broken reference and a clean reference and pass.

## 8. Constitution + frontmatter schema

- [ ] Add the `references:` row to constitution §text-first-artifacts (generator-managed, derived, distinct from `dependencies:`).
- [ ] Add the §spec-lifecycle carve-out: a diff that only adds/removes/changes cross-service reference links is mechanical-class (non-reopening); a `done` spec stays `done`. Word it so the exemption is diff-determinable.
- [ ] Add the §drift-prevention canonical-source row pointing at this spec's `data-model.md`.
- [ ] Mirror the §spec-lifecycle interaction and the new generator in `AGENTS.md`.
- [ ] **Done when:** anchors resolve, the canonical-source/back-link audits pass, and `scripts/audit/run-all.sh` is clean.

## 9. Parity, golden, and fixtures

- [ ] Add `runtime/tests/fixtures/cross-service-*` (a consumer spec plus fake registered checkouts exercising each outcome) and `runtime/tests/parity/cross-service/*` with `runtime/tests/golden/cross-service-*.jsonl`.
- [ ] **Done when:** markdown-only and runtime paths produce byte-identical golden records; the runtime test suite is green.

## 10. CI opt-in invariant

- [ ] Add the `resolve-references` tool name to `framework/runtime-tools.txt` so step (a) verifies it is absent from PATH.
- [ ] Add `bash scripts/gen-cross-service-refs.sh --dry-run` to `markdown-only-pipeline.yml` step (b), and ensure the markdown-only job completes status resolution with no runtime present.
- [ ] **Done when:** `markdown-only-pipeline.yml` passes with the runtime binary absent — the fallback is exercised end-to-end.

## 11. Documentation

- [ ] Update `README.md`: document `[services]` and cross-service reference resolution (local-checkout requirement, outcome semantics), and add the `/{project}:link` row to the **Orient** command table.
- [ ] **Done when:** README reflects the shipped behavior, the Orient table includes `/{project}:link`, and `npx markdownlint-cli2` is clean.

## 12. Full validation sweep

- [ ] **Done when:** `npx markdownlint-cli2`, the full runtime test suite (`cargo test`), `scripts/audit/run-all.sh`, and every generator `--dry-run` are all green, and `/{project}:review` passes on the implementation.
