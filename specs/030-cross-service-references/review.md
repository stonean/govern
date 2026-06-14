---
spec: 030-cross-service-references
reviewed-at: 2026-06-14T23:37:34Z
reviewed-against: 72d54e46cd83117b0184585c67024291407483ee
diff-base: 5cf4ff0d91a5ce838694a70029cf7796886e75bb
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 030-cross-service-references

Rule files loaded: `api-backend.md`, `security-backend.md`,
`configuration-cross.md` (backend stack; the three `*-frontend.md` files were
dropped — `govern` ships no frontend surface, and none are disabled in
`.govern.toml`).

## Summary

Clean across all five passes — **0 MUST, 0 SHOULD, 0 low-confidence; not
blocking.** Spec 030 adds cross-service reference resolution as a deterministic
file-reading MCP primitive (`resolve_references.rs`), a TOML registry schema
(`services.rs`), a bash harvest generator (`gen-cross-service-refs.sh`), and
command prose. The surface has no network, authentication, authorization,
HTTP-API, database, or secret-handling concerns, so the bulk of the loaded
security rules (`BE-AUTHN-*`, `BE-AUTHZ-*`, `BE-API-*`, `BE-SCHEMA-*`,
`BE-PAGE-*`, `BE-IDEMP-*`) have no surface to fire against. Input handling reads
operator- and repo-controlled local files only — never untrusted input over a
network — and the one place a path is composed from harvested data is bounded
by a `NNN-[a-z0-9-]+` slug regex, so no path traversal is reachable. Error
handling cleanly separates operational failures from per-reference outcomes;
the shell generator is hardened (`set -euo pipefail`, fully quoted expansions,
no `eval`, atomic tempfile writes). Reuse and simplicity are good.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /gov:groom)

_None — `specs/inbox.md` had no additions in the review window
(`5cf4ff0..HEAD`)._

## Skipped passes

_None — all five passes ran._

## Pass notes

Scope reviewed: `runtime/src/primitives/resolve_references.rs`,
`runtime/src/schema/services.rs`, the `ResolveReferences*` /
`ResolutionRecord` / `ReferenceOutcome` additions in
`runtime/src/schema/primitives.rs`, the registration glue in
`runtime/src/primitives/mod.rs` and `runtime/src/mcp/server.rs`,
`scripts/gen-cross-service-refs.sh`, and `runtime/tests/cross_service.rs`.

- **Security** — No auth/session/access-control surface (`BE-AUTHN-*`,
  `BE-AUTHZ-*` do not fire). No HTTP request/response surface (`BE-API-*`,
  `BE-SCHEMA-*`, `BE-PAGE-*`, `BE-IDEMP-*` do not fire). `BE-INPUT-*`: the
  primitive's inputs are local `.govern.toml`, the consumer spec, and the
  linked checkout's `spec.md` — all operator/repo-controlled, not untrusted
  network input. The only path composed from harvested data
  (`{checkout}/specs/{spec}/spec.md`) is bounded by the generator's
  `/specs/[0-9][0-9][0-9]-[a-z0-9-]+/` slug regex (no `..`/`/`), and the
  `..`-permitting checkout `path` is documented machine-local config. No
  injection sink (no SQL, no shell exec of content, no `eval`). `BE-DATA-*`:
  no PII, secrets, or sensitive data — only lifecycle statuses are read.
  `BE-LOG-*`: the primitive performs no logging.
- **Reuse** — `resolve_references.rs` reuses `read_text`,
  `split_frontmatter`, and `ALLOWED_STATUSES` (the `validate-frontmatter`
  machinery) rather than re-implementing frontmatter parsing.
  `gen-cross-service-refs.sh` deliberately parallels `gen-spec-deps.sh`
  (the plan's separate-generator trade-off) and never reads or writes
  `dependencies:`. No duplicated logic.
- **Quality** — Operational failures surface as `PrimitiveError`
  (`Result`); per-reference failures are classified outcomes, never errors.
  `read_target_status` swallows read/parse failures via `.ok()?` →
  `status-unreadable`, so a malformed upstream spec cannot panic the run.
  `services.rs::from_toml_str` propagates `toml::de::Error` on malformed
  input (no `unwrap`/`panic` on external data). Edge cases — empty index,
  null service, absent `[services]`, missing checkout, out-of-set status,
  self-reference — are covered by unit tests and the parity fixture.
- **Efficiency** — Resolution is `O(references)` with `O(log n)` registry
  lookups (`BTreeMap`); no N+1, no unbounded loop over untrusted input
  (specs are bounded by the tracked index). The generator is linear in
  spec-file lines.
- **Simplicity** — No premature abstraction or dead branches; the closed
  outcome enum maps one-to-one to the data-model. Hardcoded structural path
  segments (`specs`, `spec.md`, `.govern.toml`) follow the established
  runtime convention (every primitive inlines them) — not configuration, so
  `CFG-CONST-*` does not fire.
