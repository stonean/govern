# 026 — Framework self-audit (`/audit`) Tasks

Tasks derived from the [plan](plan.md). Complete in order. Phased structure — `/audit` is a non-trivial implementation and the phases keep each landing reviewable on its own.

## Phase A — Scaffolding

### 1. Scaffold `framework/commands/audit.md` skeleton

- [x] Create `framework/commands/audit.md` with frontmatter (`description`, no `argument-hint`, no `parity:` yet) and an empty Instructions section. The full procedure body lands in task 4 after the family scripts exist.
- [x] Add frontmatter description: `Audit framework artifacts for cross-doc, cross-manifest, cross-registry drift. Maintainer-only.`
- [x] Run `scripts/gen-claude-commands.sh` to regenerate `.claude/commands/gov/audit.md`. Verify the mirror exists and matches the source.

- **Done when**: `framework/commands/audit.md` exists with frontmatter + H1 + empty Instructions section; `.claude/commands/gov/audit.md` regenerates without error.

### 2. Create `scripts/audit/` directory and `check-zero.sh` orchestrator

- [x] Create `scripts/audit/` directory with an explanatory `README.md` describing the per-family contract (stdout findings, exit 0/1, read-only, idempotent).
- [x] Create `scripts/audit/check-zero.sh` that invokes the 9 generators/lints in plan order. Print a `check-zero` finding per failing script to stdout. Exit 1 on any failure; exit 0 only when all pass.
- [x] Smoke-test `scripts/audit/check-zero.sh` locally — confirm it exits 0 against the current repo state.

- **Done when**: `scripts/audit/check-zero.sh` exists, runs cleanly against a clean repo, and prints findings against a deliberately broken state (manually re-introduce drift in one file as a smoke test).

## Phase B — Family checks (implement in parallel order; each is independent)

### 3. Implement Family 1 — `scripts/audit/cross-doc-consistency.sh`

- [ ] Sub-check 1a: invoke `scripts/gen-readme-table.sh --check`. Surface non-zero exit as a finding pointing at `README.md`.
- [ ] Sub-check 1b: extract the pipeline-diagram block (delimited by the canonical `draft → clarified → planned → in-progress → done` line and surrounding whitespace) from `framework/constitution.md` §spec-lifecycle, `docs/introduction.md`, and `framework/templates/project/project-readme.md`. Diff pairwise; finding per divergence.
- [ ] Sub-check 1c: extract back-edge sentences from `framework/constitution.md` §spec-lifecycle. Compare against references in `framework/commands/ask.md` and `framework/commands/target.md`'s Status→next-action table. Finding per wording divergence.
- [ ] Unit-test by running against the current repo (should exit 0) and against a deliberately-modified copy with one altered line in `docs/introduction.md` (should exit 1 with one finding).

- **Done when**: script runs, exits 0 on clean state, exits 1 with structured findings on injected drift.

### 4. Implement Family 2 — `scripts/audit/manifest-parity.sh`

- [ ] Sub-check 2a: parse the file manifest from `framework/bootstrap/govern.md`'s scaffold section and from `framework/commands/init.md`'s scaffold section. Extract path lists via section-anchored regex. Diff; finding per asymmetry.
- [ ] Sub-check 2b: extract Claude `permissions.allow` array from `framework/bootstrap/configure/claude.md`'s canonical permission set. Extract Auggie `toolPermissions` array from `framework/bootstrap/configure/auggie.md`. Normalize each entry to `(tool, command-pattern)` per Q6's rule (Claude `Bash(X *)` → `("Bash", "X *")`; Auggie `{toolName: "launch-process", shellInputRegex: "^X "}` → `("Bash", "X *")` after stripping `^` and trailing space, normalizing case). Sort both, diff; finding per asymmetric entry.
- [ ] Smoke-test as in task 3.

- **Done when**: script exits 0 on the current matched configure pair; exits 1 with finding when one configure file is missing a permission the other has.

### 5. Implement Family 3 — `scripts/audit/registry-equivalence.sh`

- [ ] Read `framework/workflows/registry.json` via `jq`. Build a map of `(entry-name → workflow-file)`.
- [ ] Walk `framework/workflows/*.md` (excluding `registry.json`). Build the set of present workflow files.
- [ ] For each registry entry: verify the referenced workflow file exists. For each workflow file: verify it's in the registry.
- [ ] For each pair: extract the workflow file's frontmatter `description:` field; compare against the registry entry's `description` field. Finding per mismatch.

- **Done when**: script exits 0 on the current registry/workflows; exits 1 on a deliberately-introduced asymmetry (e.g., register a non-existent workflow file).

### 6. Implement Family 4 — `scripts/audit/placeholder-roundtrip.sh`

- [ ] `grep -rn` over `framework/commands/*.md` for hardcoded tokens: `.claude/`, `gov:`, `/gov:`.
- [ ] Allowlist mechanism: skip lines preceded by `<!-- audit:ignore-placeholders -->` on the previous line. `framework/commands/audit.md` itself (NOT scaffolded into adopters) is the primary case where literals are correct.
- [ ] Surface remaining hits as findings with file:line and suggested fix (the placeholder form: `{cli-config-dir}/`, `{project}:`, `/{project}:`).

- **Done when**: script exits 0 on the current command set (after audit.md ignore comments are placed); exits 1 if a hardcoded `.claude/` is introduced into any non-allowlisted location.

### 7. Implement Family 5 — `scripts/audit/template-alignment.sh`

- [ ] Parse `framework/commands/analyze.md` for blocking-check sections. Each section names fields it requires (e.g., `Acceptance criteria section exists`, `status field is present and one of: draft, clarified, planned, in-progress, done`).
- [ ] For each named field, verify the corresponding template under `framework/templates/spec/` scaffolds it (e.g., `spec.md` template must include `## Acceptance Criteria`; the frontmatter shape in the template's `---` block must include `status`).
- [ ] Finding per template missing-element. The suggested-fix names the template file and the field to add.

- **Done when**: script exits 0 on the current analyze/templates pair; exits 1 if `## Acceptance Criteria` is removed from `framework/templates/spec/spec.md`.

### 8. Implement Family 6 — `scripts/audit/ssot-invariants.sh`

- [ ] Initial curated list (in the script): open-question counting rule, status state machine, back-edge ownership. Each entry is a struct of `{name, canonical-location, canonical-text-pattern, paraphrase-patterns}`.
- [ ] For each entry: grep all framework artifacts (`framework/`, `specs/`, `docs/`, `README.md`, `AGENTS.md`, `CLAUDE.md`) for the canonical text plus paraphrases.
- [ ] Finding when the canonical text appears in more than one file, or when a paraphrase appears outside the canonical location without a reference to it.
- [ ] Document in the script header: "to add a new SSOT-tracked rule, append a struct to the `SSOT_RULES` array in this file."

- **Done when**: script exits 0 on the current repo state (assumes existing duplicates are tolerated for v1 or fixed before this lands); exits 1 if a tracked rule's canonical text is duplicated.

### 9. Implement Family 7 — `scripts/audit/sibling-coupling.sh`

- [ ] Walk every `specs/NNN-*/spec.md`. Parse `status:` from frontmatter; skip `done` specs.
- [ ] For each non-`done` spec: extract inline markdown links to other `specs/NNN-*/` paths from the body; combine with frontmatter `dependencies:`.
- [ ] For each non-`done` spec: extract Affected Files table rows (first column) from `plan.md` if present.
- [ ] Build pairs `(A, B)` where A and B inline-link each other AND share at least one Affected Files row.
- [ ] For each pair: identify the second-drafted spec (higher NNN prefix). Grep its `## Resolved Questions` for the literal phrase `Why split from {A-slug}:`. If present, skip the pair (suppressed). Otherwise, emit a finding listing both specs, the overlapping rows, and both resolution paths from the plan in the suggested-fix column.
- [ ] Document the literal-phrase contract in the script's stdout output (so copy-paste from the finding produces a working suppression entry).

- **Done when**: script exits 0 on the current spec set (no unsuppressed coupling pairs); exits 1 with a finding when a deliberately-introduced coupling pair (test by creating two stub draft specs that link each other) lacks the suppression phrase.

### 10. Implement Family 8 — `scripts/audit/introducing-drift.sh`

- [ ] Build rename-history catalog: parse `git log --all --pretty=format:"%H %s" -- framework/commands/ scripts/` commit messages for rename indicators. Heuristic patterns: `renamed from X to Y`, `X → Y`, `consolidate X into Y`. Build the set of old-name tokens (e.g., `/capture`, `/elaborate`, `/validate`).
- [ ] For each old-name token: grep all `done` specs' bodies for the token in code-span form (backticked).
- [ ] For each match: examine the surrounding sentence (split at sentence boundaries). Look for current-tense verbs (`is`, `provides`, `exposes`, `creates`, `runs`, `defines`).
- [ ] If a current-tense verb is found near the old-name token: emit a finding with the affected sentence, the spec file:line, and a suggested past-tense rewrite (`is` → `was`, `provides` → `provided`, `exposes` → `exposed`, `creates` → `created`, `runs` → `ran`, `defines` → `defined`).
- [ ] Suggested-fix output names the `/gov:ask` cycle as the resolution path (the maintainer adds a clarify-question on the affected spec, then accepts the past-tense rewrite).

- **Done when**: script runs and produces findings against the documented ~9 specs from spec body's Family 8 background (011, 014, 017, 020, 021, 022, 023, 024, 000). False positives expected; manually verify the findings list is plausible.

## Phase C — Wire the procedure

### 11. Fill `framework/commands/audit.md` Instructions section

- [ ] Replace the empty Instructions section with a numbered procedure: step 1 invokes `run-generator` against `scripts/audit/check-zero.sh`; if it reports drift, halt and exit. Steps 2–9 invoke `run-generator` against each of the eight family scripts; stream output to /audit's stdout under family headers.
- [ ] Each step uses the parseable conventions per spec 022 — numbered, backtick-quoted `run-generator` name, no extension-point markers (every step is deterministic; no LLM extension required).
- [ ] Add an "Output" section to the command body documenting the stdout format and exit-code contract.
- [ ] Add a "Boundary with `/gov:analyze`" section referencing the spec's table.

- **Done when**: `framework/commands/audit.md`'s Instructions section parses cleanly under `scripts/lint-procedure-parseability.sh`; the regenerated `.claude/commands/gov/audit.md` mirror is in sync.

### 12. Verify `framework/commands/audit.md` introduces no new MCP primitives

- [ ] Verify the command's body does NOT introduce any new MCP primitive names (only existing `run-generator` calls). Confirm `scripts/lint-tool-coverage.sh` exits 0.
- [ ] Remove `framework/commands/audit.md` from `runtime/legacy-prose-commands.txt` (it ships parseable from day one).

- **Done when**: lint-tool-coverage passes; lint-procedure-parseability passes against audit.md without it being on the legacy allowlist.

### 13. Wire `/audit` self-invocation

- [ ] Run `./runtime/target/release/gvrn exec audit` against the current repo. Verify it exits 0 (assuming all family checks pass on the current state) or exits 1 with structured findings the maintainer can address.
- [ ] Iterate: any findings surfaced against the current repo state get classified — true drift (fix it now) vs false positive (refine the family script).

- **Done when**: `gvrn exec audit` runs end-to-end and either exits 0, or exits 1 with findings whose resolution path is clear.

## Phase D — CI integration

### 14. Add `/audit` step to `markdown-only-pipeline.yml`

- [ ] Edit `.github/workflows/markdown-only-pipeline.yml`: append a step `Framework self-audit` after the existing lint steps. The step runs `./runtime/target/release/gvrn exec audit`.
- [ ] Verify the workflow's existing runtime-build step provides the binary `/audit` needs.
- [ ] Test locally by running the workflow's bash steps in order; confirm `/audit` runs with the binary available.

- **Done when**: workflow file passes `actionlint`; pushing a draft PR triggers the workflow and includes the `/audit` step.

### 15. Add `/audit` pre-tag gate to `runtime-release.yml`

- [ ] Edit `.github/workflows/runtime-release.yml`: append a pre-matrix step `Framework self-audit (pre-tag gate)` that runs `./runtime/target/release/gvrn exec audit`. A failure aborts the release before any matrix leg fires.
- [ ] Verify the workflow's setup steps build the binary before this new step runs.

- **Done when**: workflow file passes `actionlint`; the pre-tag gate visibly appears in the workflow's job graph.

## Phase E — Validation

### 16. Run `/gov:analyze` against this spec

- [ ] Invoke `/gov:analyze` targeted at `026-framework-self-audit`. Resolve any hard-fail or blocking findings against `spec.md`, `plan.md`, `tasks.md`.
- [ ] Confirm anchor resolution: any §runtime-boundary / §rules / §spec-lifecycle references resolve to constitution markers.
- [ ] Confirm dependency status: 017, 022, 023, 024, 025 are all `done`.

- **Done when**: `/gov:analyze` reports no hard-fail and no blocking findings.

### 17. Final lint sweep

- [ ] `npx markdownlint-cli2 'specs/026-framework-self-audit/**/*.md'` — exit 0.
- [ ] `scripts/lint-procedure-parseability.sh` — exit 0 (audit.md is parseable).
- [ ] `scripts/lint-tool-coverage.sh` — exit 0.
- [ ] `scripts/lint-frontmatter.sh` — exit 0.
- [ ] `gvrn exec audit` — exit 0 against the post-implementation repo state. (The audit auditing itself is the meta-check that closes the loop.)

- **Done when**: every lint exits 0; `/audit` is green against its own implementing repo state.

### 18. Promote spec to `done`

- [ ] After the user confirms `/gov:review` is clean against the four phases' commits, set `specs/026-framework-self-audit/spec.md` status from `in-progress` to `done`.

- **Done when**: spec status is `done`; the runtime CI workflow passes on the final commit.
