---
spec: 040-configurable-specs-dir
reviewed-at: 2026-06-30T12:06:10Z
reviewed-against: eb6cd1f562f1fd630c09e1cdaba8f722479bc1c2
diff-base: cfc1023ace626499343fc23a823905708e151955
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 1
skipped-passes: []
---

# Review — 040-configurable-specs-dir

## Summary

Re-review after the fixes in `eb6cd1f`. **0 MUST, 0 SHOULD — clean, not blocking.** All three advisory findings from the prior run (`41d25f6`) are resolved: the redundant per-call `.govern.toml` read is gone — `dashboard::load_specs` and `traverse_deps::run` each resolve the root exactly once and thread it through `load_one_spec` / `visit`; `validate_specs_root` (and the mirrored bash check) now enforce the conservative, regex-safe `[A-Za-z0-9_-]` charset, with the spec's well-formedness rule, AC, and the `/govern`+`/gov:init` operator messages updated to match; and `resolve_specs_root` is extracted to `scripts/lib/specs-root.sh`, sourced by both generators (one definition) and shipped via the Shared Files manifest. The fixes introduce no new findings — re-reviewed adversarially across all five passes. Verification: `cargo test` 412/0, clippy clean, both generator suites green through the shared lib, framework self-audit (incl. manifest-parity) 0 findings, markdown-only opt-in lints pass. Security/performance/reliability rules remain N/A — a CLI runtime plus bash generators with no network, datastore, or credential surface.

One non-blocking, out-of-scope note (not a counted finding): the sibling enumeration helpers `list_specs` / `staged_specs` remain duplicated across the two generators. That duplication predates 040 (the feature only modified them to resolve the configured root, it did not introduce them); now that `scripts/lib/specs-root.sh` exists, they are natural candidates to move into the same lib in a future hygiene pass.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /gov:groom)

- Cross-service reference resolution assumes the _referenced_ service uses `specs/` — `gen-cross-service-refs.sh`'s URL matcher (`/specs/NNN-slug/`) targets another repo's layout, which this project's `[paths] specs-root` does not govern. Deferred from 040's scope (a referenced service that renamed _its_ root is a cross-repo concern); run `/gov:groom` to route.

## Skipped passes

_None._
