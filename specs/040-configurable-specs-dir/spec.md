---
status: in-progress
dependencies: [002-project-scaffolding, 003-bootstrap-automation, 017-derive-dont-ask, 022-deterministic-runtime]
review:
  last-run: 2026-06-30T12:06:10Z
  reviewed-against: eb6cd1f562f1fd630c09e1cdaba8f722479bc1c2
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 040 — Configurable spec-root directory name

The top-level directory that holds all govern artifacts is hardcoded to `specs`. This feature adds a single operator-set `.govern.toml` setting for that directory's name, chosen at initial configuration, defaulting to `specs` so existing projects are unaffected. Every command and the optional runtime resolve the spec root from the setting instead of a hardcoded literal.

## Motivation

The name `specs` is baked into the framework in three layers:

- **Constitution and templates** — the §spec-phase layout, rule-file paths (`specs/rules/`), the inbox (`specs/inbox.md`), shared docs (`specs/system.md`, `events.md`, `errors.md`), and the per-feature `specs/{NNN-feature}/` convention. The literal `specs/` appears across roughly three dozen framework markdown files.
- **Command sources** — every pipeline command reads and writes under `specs/`, and the session file records `path = "specs/{NNN-slug}"`.
- **Runtime** — the optional [022-deterministic-runtime](../022-deterministic-runtime/spec.md) primitives hardcode the root (e.g., `repo.join("specs")` in the dashboard primitive, plus boundary derivation and spec reads).

There is no place for an adopter to choose a different name. That is a real problem because `specs/` is one character away from `spec/` — the directory RSpec and many Ruby/Rails projects already own for tests. An adopter integrating govern into a Rails repo cannot adopt without the two trees sitting confusingly side by side, and an adopter who would prefer a clearer name (`governance/`, `design/`, `gov-specs/`) has no supported way to pick one. The collision is not purely cosmetic: tooling, editor search scopes, and CI globs that target `spec/` can sweep in `specs/` content and vice versa.

The fix is one operator-set source of truth for the directory name, set once at initial configuration, honored everywhere, and defaulting to `specs` so no existing adopter changes behavior.

## Setting

A new `.govern.toml` key names the spec-root directory, mirroring the operator-setting pattern established by spec 033's `[rules] surfaces` (see also below):

```toml
[paths]
specs-root = "specs"     # default; an adopter may set e.g. "governance"
```

- **Default preserves today's behavior.** When the key is absent, the effective name is `specs` and every code path behaves exactly as it does now. This is load-bearing: the feature must be a no-op for every project that does not opt in.
- **Well-formed name only.** The value is a single directory-name segment using only the conservative charset `[A-Za-z0-9_-]` (letters, digits, hyphen, underscore) and is non-empty — no path separators, no `.`/`..`, no other punctuation. It names a directory at the repo root, not a nested path. The charset is deliberately strict: the runtime uses the name as a literal path component, but the bash generators interpolate it into `grep`/`awk` regexes, so a `.`, `+`, `(`, … must be excluded to stay regex-safe.
- **Operator-set source of truth.** The name is not derived from the stack or guessed; there is one default (`specs`) and one override (the setting). This keeps with [017-derive-dont-ask](../017-derive-dont-ask/spec.md)'s posture: sensible default, explicit override, prompt confined to the configuration command.

## Configuration behavior

- **Chosen at initial configuration.** During [003-bootstrap-automation](../003-bootstrap-automation/spec.md) (`/govern`), the operator may set the spec-root name; the prompt defaults to `specs` and persists the answer to `.govern.toml`. The prompt lives only in the configuration command — no other command asks for it.
- **Scaffolded under the configured name.** [002-project-scaffolding](../002-project-scaffolding/spec.md) (`/gov:init`) creates the spec-root directory (and its `inbox.md`, `rules/`, shared docs) under the configured name, or `specs` when unset.

### Validation and notices

- **Well-formedness is blocking.** A value that is empty or contains any character outside `[A-Za-z0-9_-]` (path separators, `.`/`..`, or other punctuation) is rejected with a clear message at configuration time — never silently accepted, because such a value breaks path resolution or the generators' regex interpolation.
- **Collision is an advisory.** If the chosen directory already exists on disk and is not a govern spec root (no `inbox.md`, no numbered `NNN-*` subdirs), govern emits a one-line notice naming the directory and proceeds on operator confirmation. The check is driven by what is on disk, not a hardcoded sibling-framework allowlist, so it catches `spec/` full of RSpec tests — the motivating case — and any other pre-existing directory. The operator may deliberately choose an existing name; the choice is honored after the warning.
- **Half-finished-rename notice.** When the configured `specs-root` is absent on disk but a different govern-shaped directory exists, govern emits a one-line notice (a likely interrupted manual rename) rather than silently scaffolding a new empty tree.

## Resolution behavior

- **No hardcoded `specs/` in executable paths.** Every pipeline command resolves the spec root from the setting before reading or writing any artifact. The session file's `path` field uses the configured root (`{root}/{NNN-slug}`).
- **All sibling artifacts move with the root.** Rule files resolve under `{root}/rules/`, the inbox at `{root}/inbox.md`, and shared docs (`system.md`, `events.md`, `errors.md`) at `{root}/`.
- **Runtime agrees with the markdown-only path.** Split by primitive shape: primitives that take a full spec *path* argument (`write-session`, `lint-markdown`, `substitute-templates`) need no change because the host bakes the resolved root into that path. Primitives that take a bare *feature name* and join it under the root internally (`read-spec`, `set-status`, `mark-task`, `mark-criterion`, `read-tasks`, `traverse-deps`, `check-stuck`, `derive-boundary`, `resolve-references`) and tree-enumerating primitives (`dashboard`) currently hardcode `repo.join("specs")`; they resolve the root from `[paths] specs-root` in the committed `.govern.toml` (default `specs`) through one shared helper. That default matches the markdown-only path, so the two resolutions provably agree. Per the runtime-boundary principles, `.govern.toml` is git-tracked source of truth the runtime already parses — the runtime reads the resolved root, it does not own it.
- **Prose default vs. executable resolution.** The constitution and templates keep `specs/` as the documented default name in illustrative paths; the configurability fact is stated once at its canonical home (the §spec-phase directory-layout block) with a back-pointer from the `.govern.toml` schema docs. Only command bodies and the runtime resolve the configured root.

## Acceptance Criteria

- [x] `.govern.toml` accepts `[paths] specs-root`; when it is unset, the effective name is `specs` and no command or runtime behavior changes for existing adopters.
- [x] At initial configuration (`/govern`), the operator can choose a different spec-root name; the prompt defaults to `specs` and the choice is persisted to `.govern.toml`. No command other than `/govern` prompts for it.
- [x] A malformed value (empty, or containing any character outside `[A-Za-z0-9_-]` — path separators, `.`/`..`, or other punctuation) is rejected with a clear message at configuration time rather than silently accepted.
- [x] When the chosen directory already exists on disk and is not a govern spec root (no `inbox.md`, no numbered `NNN-*` subdirs), configuration emits a one-line notice naming the directory and proceeds on operator confirmation; the choice is honored after the warning.
- [x] When the configured `specs-root` is absent on disk but a different govern-shaped directory exists, govern emits a one-line half-finished-rename notice instead of silently scaffolding a new empty tree.
- [x] `/gov:init` scaffolds the spec-root directory — including `inbox.md`, `rules/`, and shared docs — under the configured name, or under `specs` when the setting is unset.
- [x] No pipeline command reads or writes a hardcoded `specs/` path; each resolves the spec root from the setting, and a project configured with a non-`specs` name shows no stray `specs/` directory after running the pipeline.
- [x] The session file's `path` field uses the configured spec root (e.g., `governance/040-...` when the setting is `governance`), and self-corrects on the next `/gov:target` / `/gov:specify` write after a manual rename.
- [x] Rule files, the inbox, and shared docs (`system.md`, `events.md`, `errors.md`) resolve under the configured spec root.
- [x] The runtime resolves the spec root consistently with the markdown-only path: full-path primitives consume the root from their `path` argument unchanged, while every primitive that joins a bare feature name under the root or enumerates the tree resolves `[paths] specs-root` from `.govern.toml` (default `specs`) through one shared helper.
- [x] The constitution's §spec-phase directory-layout block carries a single one-line note that the spec-root name is configurable via `[paths] specs-root` (default `specs`), with a back-pointer from the `.govern.toml` schema docs; no other prose is parameterized.
- [x] A project configured with a non-`specs` spec-root name completes a full pipeline cycle (`/gov:specify` → … → `done`) with no path errors.
- [x] Runtime error messages that name a spec artifact reflect the configured spec-root (e.g. `governance/040-foo`), with no hardcoded `specs/` prefix, so they stay accurate under a renamed root.

## Open Questions

<!-- All open questions resolved — see Resolved Questions below. -->

*None — all resolved.*

## Resolved Questions

- **Config key and section.** Resolved: the setting is `[paths] specs-root`, defaulting to `"specs"`. A `[paths]` section is the natural home and leaves room for future path settings; `specs-root` names the role rather than a literal value (it stays accurate when set to `governance`) and reads unambiguously as a single directory name, not a nested path. Rejected `[layout] spec-root` because `[paths]` is the more conventional section name and bare `root` risks confusion with the repo root.
- **Set-once vs. changeable later.** Resolved: the name is set at initial configuration; changing it after the tree is populated is an operator-driven manual rename (`git mv specs governance` for deterministic history, then update `[paths] specs-root`), not an automated govern migration. The session `path` self-corrects on the next `/gov:target` / `/gov:specify` write. Govern owns no file-mover: moving a populated tree touches git history, the session file, and external references — `git mv` is the right tool, and an in-framework mover would add a destructive code path for a rare operation (mirroring 033 leaving removed-surface files in place rather than pruning). As a cheap safety check — not a migration — govern emits a one-line notice when the configured `specs-root` is absent on disk but a different govern-shaped directory exists (a likely half-finished rename), rather than silently scaffolding an empty tree.
- **Prose parameterization.** Resolved: keep `specs/` as the documented default value in prose; do not parameterize illustrative paths. The default is exactly `specs`, so the ~35 constitution/template/README references stay literally correct for every adopter who does not opt in, and `{specs-root}` placeholders would degrade a human-read document for accuracy it already has. Per the *canonical sources* discipline, the configurability fact is stated once at its natural home — the §spec-phase directory-layout block — with a back-pointer from the `.govern.toml` schema docs; all other prose references `specs/` as the default without repeating the caveat. Only command bodies and the runtime resolve the configured root, mirroring how the framework treats `{project}` (illustrative default in prose, resolved at runtime).
- **Collision validation depth.** Resolved: well-formedness is a hard, blocking validation (a name that is empty or contains any character outside `[A-Za-z0-9_-]` — separators, `.`/`..`, other punctuation — is rejected with a clear message at configuration time; the strict charset keeps the name regex-safe for the bash generators, per the 040 review); collision with an existing directory is a non-blocking advisory. The collision check is driven by what is actually on disk, not a hardcoded framework allowlist: if the chosen directory already exists and is not a govern spec root (no `inbox.md`, no numbered `NNN-*` subdirs), govern emits a one-line notice naming the directory and proceeds on operator confirmation. This catches `spec/` full of RSpec tests — the motivating case — and any other pre-existing directory, without govern needing to enumerate sibling frameworks. An operator may deliberately pick an existing name, so the choice is honored after the warning (consistent with the framework's "no silent mismatch" posture and 033's contradiction notice); blocking would override a legitimate choice and silence would reproduce the confusion this feature exists to prevent.
- **How the runtime obtains the root.** Resolved: split by primitive shape. Primitives that take a full spec *path* argument (`write-session`, `lint-markdown`, `substitute-templates`) need no change — the host bakes the resolved root into that path, so no new runtime-owned state is introduced. Primitives that take a bare *feature name* and join it under the root internally (`read-spec`, `set-status`, `mark-task`, `mark-criterion`, `read-tasks`, `traverse-deps`, `check-stuck`, `derive-boundary`, `resolve-references`) and tree-enumerating primitives (`dashboard`) currently hardcode `repo.join("specs")`; they must discover the root and resolve it by reading `[paths] specs-root` from the committed `.govern.toml`, defaulting to `specs` when absent. Reading `.govern.toml` honors the runtime-boundary rule — it is git-tracked source-of-truth config the runtime already parses (`[rules] surfaces`, `[services]`, `[[review.disabled-rule-files]]`), so resolving the root from it adds no state the markdown cannot reconstruct (principle 1) and keeps the runtime reading a schema the constitution declares (principle 4). The default-`specs` fallback is identical to the markdown-only path, so the two paths provably agree. Rejected uniform host-passes-root-to-every-call: it would churn every primitive signature for a value most already carry in their path argument.

## See also

- [033-rule-surface-setting](../033-rule-surface-setting/spec.md) — analogous `.govern.toml` operator setting with a `/govern` prompt and a derive-by-default posture; a close structural model for this feature.
