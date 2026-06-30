# 040 — Configurable spec-root directory name Plan

Implements [040 — Configurable spec-root directory name](spec.md).

## Overview

Introduce one operator-set `.govern.toml` key — `[paths] specs-root`, default `"specs"` — as the single source of truth for the spec-root directory name, and teach every executable code path to resolve the root from it instead of hardcoding `specs/`. The work spans three surfaces:

1. **Runtime (`gvrn`)** — a single shared resolver reads `[paths] specs-root` (default `specs`), and every primitive that today calls `repo.join("specs")` calls the resolver instead.
2. **Bootstrap and commands (markdown)** — `/govern` gains the init-time prompt, validation, and notices; `/gov:init` scaffolds under the configured name; command bodies resolve the root in executable path references while illustrative prose keeps `specs/` as the documented default.
3. **Generators (`scripts/`)** — the tree-walking generators and lints resolve the configured root before walking.

The load-bearing invariant is **default-`specs` everywhere**: an adopter who never sets the key sees byte-for-byte identical behavior, so existing fixtures, golden tests, and the markdown-only opt-in CI all keep passing unchanged.

## Technical Decisions

### Config schema: `[paths] specs-root`, default `specs`

A new TOML table `[paths]` carries a scalar key `specs-root`. Resolution rule everywhere: read the key; when absent or empty, fall back to `specs`. The key is documented in this spec's body — per the project convention that `.govern.toml` is a shared adopter-side database whose new "tables" are documented in the adding spec, not signposted onto the prior `.govern.toml` specs (`017`/`019`). Well-formedness (non-empty, single segment, no separators, no `..`, no leading slash) is validated when `/govern` writes the value.

### One shared runtime resolver

Add a single helper in the runtime's config/schema layer (the same layer that already parses `[services]` and `[rules]`):

```rust
// resolve the spec-root directory name for `repo`, defaulting to "specs"
fn specs_root(repo: &Path) -> PathBuf  // reads [paths] specs-root from <repo>/.govern.toml
```

Every `repo.join("specs")` site is rewritten to `repo.join(specs_root(repo))`. From the discovery sweep, the sites are: `primitives/{read_spec, set_status, mark_task, mark_criterion, read_tasks, traverse_deps, check_stuck, derive_boundary, resolve_references, dashboard}.rs` and `interpreter/payload.rs`. Full-path primitives (`write_session`, `lint_markdown`, `substitute_templates`) are untouched because the host bakes the resolved root into the path argument they receive.

One resolver (not a `.govern.toml` read duplicated per primitive) keeps the default and the parsing in one place — the drift-prevention discipline applied to code. This is an edit to existing primitives' path construction plus a config-read helper; it does **not** add a new primitive, so the new-primitive wiring checklist (schema/parser/interpreter/main/server/`runtime-tools.txt`) does not apply.

### Bootstrap (`/govern`) gains the prompt, validation, and notices

In `framework/bootstrap/govern.md`:

- **Prompt at init** for the spec-root name, defaulting to `specs`, persisted to `.govern.toml`. Confined to `/govern` — no other command prompts (mirrors `033`'s surface-prompt pattern).
- **Well-formedness rejection** (blocking) on malformed values.
- **On-disk collision advisory** — if the chosen directory already exists and is not a govern spec root (no `inbox.md`, no numbered `NNN-*` subdirs), emit a one-line notice and proceed on confirmation.
- **Half-finished-rename notice** — configured root absent on disk but a different govern-shaped directory present.
- **Selective scaffolding** under the configured name.

Authored with placeholders (`{project}`, `{cli-config-dir}`) per the source-editing convention.

### Init (`/gov:init`) scaffolds under the configured name

`.claude/commands/gov/init.md` is the hand-maintained, govern-specific init command (no generator source counterpart). It scaffolds the spec-root directory — `inbox.md`, `rules/`, and shared docs — under the configured name, or `specs` when unset.

### Command sources resolve the root; prose keeps `specs/`

In `framework/commands/*.md`, executable path references (where a command reads/writes under the spec tree, and where the session-file `path` is written) resolve the configured root. Illustrative prose keeps `specs/` as the documented default. The configurability fact is stated once at its canonical home — the constitution `§spec-phase` directory-layout block — with an adopter-facing mention in `README.md`; no other prose is parameterized.

### Generators resolve the root

`scripts/gen-spec-deps.sh` and `scripts/gen-cross-service-refs.sh` walk the spec tree to derive frontmatter; `scripts/lint-rule-ids.sh` and `scripts/lint-frontmatter.sh` walk it to lint. Each resolves `[paths] specs-root` (default `specs`) from the repo's `.govern.toml` before walking. Because these ship to adopter repos and run from the adopter pre-commit hook, they must read the adopter's `.govern.toml` at run time. No new generators are introduced, so the three-site generator-wiring rule does not apply — the existing generators are taught to resolve the root.

### No data model

The feature adds one scalar config key, not a domain entity or data structure, so no `data-model.md` is created (the readiness-check data-model item is N/A).

### Backward compatibility is the test strategy

Default-`specs` means every existing test, fixture, and golden file keeps passing with no edits. New coverage adds a non-`specs` fixture exercised end-to-end (runtime parity + a markdown-only cycle) to prove resolution, plus a unit test for the resolver's default and override branches.

## Affected Files

<!-- Planning aid only — /gov:implement derives the real write boundary from git. -->

| File | Action | Purpose |
| --- | --- | --- |
| `runtime/src/schema/` (config layer) | Modify | Parse `[paths] specs-root`; add shared `specs_root(repo)` resolver (default `specs`) |
| `runtime/src/primitives/read_spec.rs` | Modify | Resolve root instead of `repo.join("specs")` |
| `runtime/src/primitives/set_status.rs` | Modify | Resolve root |
| `runtime/src/primitives/mark_task.rs` | Modify | Resolve root |
| `runtime/src/primitives/mark_criterion.rs` | Modify | Resolve root |
| `runtime/src/primitives/read_tasks.rs` | Modify | Resolve root |
| `runtime/src/primitives/traverse_deps.rs` | Modify | Resolve root (feature dir + dep dirs) |
| `runtime/src/primitives/check_stuck.rs` | Modify | Resolve root |
| `runtime/src/primitives/derive_boundary.rs` | Modify | Resolve root |
| `runtime/src/primitives/resolve_references.rs` | Modify | Resolve root |
| `runtime/src/primitives/dashboard.rs` | Modify | Resolve enumerated root |
| `runtime/src/interpreter/payload.rs` | Modify | Resolve root for plan/spec path construction |
| `runtime/tests/` (fixtures + parity/golden) | Create/Modify | Non-`specs` fixture; default-`specs` parity unchanged |
| `scripts/gen-spec-deps.sh` | Modify | Resolve root before walking |
| `scripts/gen-cross-service-refs.sh` | Modify | Resolve root before walking |
| `scripts/lint-rule-ids.sh` | Modify | Resolve root |
| `scripts/lint-frontmatter.sh` | Modify | Resolve root |
| `scripts/tests/` | Create/Modify | Renamed-root generator coverage |
| `framework/bootstrap/govern.md` | Modify | Init prompt, validation, collision/half-rename notices, scaffold under root |
| `.claude/commands/gov/init.md` | Modify | Scaffold spec-root dir under configured name (hand-maintained) |
| `framework/commands/*.md` | Modify | Resolve configured root in executable path refs; session `path` uses resolved root |
| `framework/constitution.md` (`§spec-phase`) | Modify | One-line configurability note |
| `constitution.md` (root) | Modify | Sync target of framework constitution |
| `README.md` | Modify | Adopter-facing mention of `[paths] specs-root` |
| `specs/002-…`, `specs/003-…`, `specs/022-…/spec.md` | Modify (conditional) | Back-linked signpost AC only where an absolute `specs/` claim is falsified |

## Trade-offs

- **One shared resolver vs. per-primitive config read** — chose one helper. Rejected duplicating `.govern.toml` parsing in each primitive: it would scatter the default value and re-parse config per call.
- **Set-once + manual rename vs. govern-owned mover** — chose manual `git mv` plus a divergence notice. Rejected an in-framework mover: a destructive code path for a rare operation that `git mv` already handles deterministically.
- **Prose keeps `specs/` vs. full parameterization** — chose documented-default plus one canonical note. Rejected `{specs-root}` placeholders across ~35 files: it degrades a human-read document for accuracy the default already provides.
- **040 as canonical owner vs. restating in 002/003/022** — chose single-owner with conditional signposts. Rejected restating "spec-root is configurable" in each touched spec: it scatters the requirement and over-reopens done specs. Signposts land only where a prior spec makes a now-false absolute claim.
- **Known limitations** — (1) Renaming after the tree is populated is a manual operator step; govern detects divergence but does not migrate. (2) Adopter CI globs or editor scopes that hardcode `specs/` must be updated by the adopter. (3) Cross-service reference URLs that encode `specs/` in another repo's path are unaffected here (they target the *other* repo's layout).
