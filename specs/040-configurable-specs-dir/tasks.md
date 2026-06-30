# 040 — Configurable spec-root directory name Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Runtime config resolver

- [x] Parse `[paths] specs-root` in the runtime config/schema layer
- [x] Add a shared `specs_root(repo)` helper returning the configured name or `specs`
- [x] Unit-test the default branch (key absent/empty → `specs`) and the override branch
- [x] Reject a malformed value (empty, separator, `..`, leading slash) at the parse boundary

Done when: the helper returns the configured value or `specs`, and unit tests cover both branches plus malformed input.

## 2. Runtime primitives resolve the root

- [x] Replace `repo.join("specs")` with the resolver in `read_spec`, `set_status`, `mark_task`, `mark_criterion`, `read_tasks`, `traverse_deps`, `check_stuck`, `derive_boundary`, `resolve_references`, `dashboard`, and `interpreter/payload.rs`
- [x] Confirm full-path primitives (`write_session`, `lint_markdown`, `substitute_templates`) are untouched
- [x] Add a non-`specs` runtime fixture and assert each primitive resolves against it

Done when: every listed site uses the resolver, the existing suite stays green under the default `specs`, and the non-`specs` fixture passes.

## 3. Generators and lints resolve the root

- [x] Resolve `[paths] specs-root` (default `specs`) in `gen-spec-deps.sh` and `gen-cross-service-refs.sh` before walking the tree
- [x] Confirm `lint-rule-ids.sh` (walks `framework/rules/`) and `lint-frontmatter.sh` (govern-CI-only, walks govern's own `specs/`) need no change — neither walks the adopter-configurable spec tree, and neither ships to or runs in an adopter pre-commit
- [x] Add a renamed-root fixture under `scripts/tests/` and assert correct walking

Done when: each script reads the configured root (default `specs`), and a renamed-root fixture is walked correctly without touching adopter wiring beyond these scripts.

## 4. Bootstrap `/govern` prompt, validation, and notices

- [ ] Add the init-time spec-root prompt (default `specs`, persisted to `.govern.toml`), confined to `/govern`
- [ ] Add blocking well-formedness validation with a clear rejection message
- [ ] Add the on-disk collision advisory (warn naming the directory; proceed on confirmation)
- [ ] Add the half-finished-rename notice
- [ ] Scaffold the spec tree under the configured name; author with placeholders

Done when: `framework/bootstrap/govern.md` documents all four behaviors and selective scaffolding via the markdown-only path, using placeholders.

## 5. Init `/gov:init` scaffolds under the configured name

- [ ] Update `.claude/commands/gov/init.md` to scaffold the spec-root dir (`inbox.md`, `rules/`, shared docs) under the configured name or `specs`

Done when: `/gov:init` creates the spec tree under the configured root and falls back to `specs` when unset.

## 6. Command sources resolve the root

- [ ] Resolve the configured root in executable path references across `framework/commands/*.md`
- [ ] Ensure the session-file `path` written by commands uses the resolved root
- [ ] Keep illustrative prose `specs/` as the documented default (no parameterization)

Done when: no command body hardcodes a `specs/` read/write path, and the session `path` reflects the configured root.

## 7. Documentation

- [ ] Add a one-line configurability note to constitution `§spec-phase` (`[paths] specs-root`, default `specs`)
- [ ] Sync the note into the root `constitution.md`
- [ ] Add an adopter-facing mention of `[paths] specs-root` to `README.md`

Done when: both constitutions carry the note and `README.md` documents the key.

## 8. Cross-spec audit (conditional signposts)

- [ ] Audit `002`, `003`, `022` spec bodies for absolute `specs/` claims falsified by this feature
- [ ] Add a back-linked signpost acceptance criterion (reopening that spec) only where a claim is actually falsified; otherwise record N/A

Done when: each of `002`/`003`/`022` is audited and either carries a back-linked signpost or is explicitly recorded as needing none.

## 9. End-to-end and opt-in invariant

- [ ] Run a full pipeline cycle on a non-`specs` fixture (`/gov:specify` → `done`) with no path errors
- [ ] Confirm default-`specs` parity/golden suites are unchanged
- [ ] Confirm the markdown-only opt-in CI stays green

Done when: the non-`specs` cycle reaches `done`, and existing default-`specs` suites and the opt-in CI pass unchanged.
