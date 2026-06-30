---
spec: 040-configurable-specs-dir
reviewed-at: 2026-06-30T02:41:41Z
reviewed-against: 41d25f6b81397527d701ff73933bc782758cc8cd
diff-base: cfc1023ace626499343fc23a823905708e151955
must-violations: 0
should-violations: 3
low-confidence: 0
captured-issues: 1
skipped-passes: []
---

# Review — 040-configurable-specs-dir

## Summary

Implementation reviewed across all five passes against the backend + cross-cutting rule files (frontend rules excluded — no frontend surface in scope). **0 MUST violations — the spec is not blocked from `done`.** Three advisory (SHOULD) findings: a redundant per-call `.govern.toml` read in the two tree-enumerating primitives, an over-permissive `specs-root` validator whose accepted characters can mis-behave when interpolated into the bash generators' regexes, and the deliberate duplication of the resolver across the two generators. The default-`specs` invariant is well-covered by tests (every pre-existing suite unchanged; a dedicated non-`specs` integration suite plus renamed-root unit/generator tests). Most security/performance/reliability rules (auth, DB pools, retries, env vars, HTTP) are N/A — the change is a CLI runtime plus bash generators with no network, datastore, or credential surface.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

### SHOULD: efficiency — redundant `.govern.toml` read+parse in tree-enumerating primitives

- **File**: `runtime/src/primitives/dashboard.rs:115-117`, `runtime/src/primitives/traverse_deps.rs:165`
- **Rule**: Efficiency pass — "repeated work." (No specific rule ID; `performance-backend` rules govern DB/cache/pool design-time commitments, not file I/O.)
- **Finding**: `paths::specs_dir(repo)` / `Paths::load(repo)` reads and TOML-parses `<repo>/.govern.toml` on every call. `dashboard::load_one_spec` is invoked once per spec inside `load_specs`'s loop, and `traverse_deps::visit` recurses once per reachable node — so each re-parses the same `.govern.toml` O(N) times where the pre-040 code (`repo.join("specs")`) did zero I/O. Absolute cost is small (µs-scale parse of a tiny file) and never affects correctness, but it is avoidable repeated work in the two walk paths.
- **Auto-fixable**: no
- **Suggested fix**: Resolve the root once per primitive run and thread it: in `dashboard`, resolve in `load_specs` and pass the resolved dir to `load_one_spec`; in `traverse_deps`, resolve in `run` and pass the spec-root into `visit` (replacing its `repo` param's `specs_dir` call). Single-call primitives (`read_spec`, `set_status`, etc.) already load once and need no change.

### SHOULD: quality/robustness — `validate_specs_root` accepts characters that are unsafe in the generators' regex interpolation

- **File**: `runtime/src/schema/paths.rs:40-68` (`validate_specs_root`), `scripts/gen-spec-deps.sh` / `scripts/gen-cross-service-refs.sh` (`resolve_specs_root` + `grep -E "^$SPECS_ROOT/…"` and the awk dynamic regex)
- **Rule**: Quality pass — robustness / contract consistency. (`quality-cross` QUAL-STUB-001 is adjacent — the generators' `… || true` would swallow a regex _syntax_ error as "no match," silently skipping a spec — but the swallow predates this change.)
- **Finding**: The validator (and the bash `case "$name" in "" | */* | *..*`) reject only empty / path-separator / `..` / leading-slash, so they accept regex metacharacters (`.`, `+`, `*`, `[`, `(`, …) and a lone `.`. The runtime uses the name only as a literal path component (safe), but the bash generators interpolate `$SPECS_ROOT` **unescaped** into `grep -E` and an awk regex. A "valid" name like `v1.0` or `spec+s` would over-match; an unbalanced char like `spec(` would make the regex invalid and — via `… || true` — silently drop that spec from enumeration. A lone `.` resolves the spec-root to the repo root (degenerate). Low real-world likelihood (uncommon names), but a genuine gap between the stated well-formedness contract and safe interpolation.
- **Auto-fixable**: no
- **Suggested fix**: Tighten `validate_specs_root` (and the mirrored bash check) to a conservative filename charset — e.g. require `^[A-Za-z0-9_-]+$` and reject a lone `.` — which closes both the regex-metachar and degenerate-`.` cases and keeps the runtime and bash validators in agreement. This widens the spec's §Setting well-formedness rule ("no separators, no `..`, no leading slash"), so it warrants a small spec update (route via `/gov:amend`) rather than a silent code-only change.

### SHOULD: reuse — `resolve_specs_root` duplicated across the two generators (accepted trade-off)

- **File**: `scripts/gen-spec-deps.sh`, `scripts/gen-cross-service-refs.sh` (identical `resolve_specs_root`)
- **Rule**: Reuse pass; constitution §drift-prevention (manifest/code duplication can drift).
- **Finding**: The ~20-line `resolve_specs_root` is copy-pasted into both generators, so a future fix to one can drift from the other.
- **Auto-fixable**: no
- **Suggested fix**: None required now. This deliberately follows the established local convention — `list_specs` / `staged_specs` are already duplicated across the same two generators because each must run standalone in an adopter pre-commit, and a shared sourced lib would add adopter-shipping wiring plus a sourcing-path failure mode (the AGENTS.md three-site generator-wiring lesson). Recorded for visibility; revisit only if a third generator needs the resolver, at which point a shipped `scripts/lib/` helper earns its keep.

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /gov:groom)

- Cross-service reference resolution assumes the _referenced_ service uses `specs/` — `gen-cross-service-refs.sh`'s URL matcher (`/specs/NNN-slug/`) targets another repo's layout, which this project's `[paths] specs-root` does not govern. Deferred from 040's scope (a referenced service that renamed _its_ root is a cross-repo concern); run `/gov:groom` to route.

## Skipped passes

_None._
