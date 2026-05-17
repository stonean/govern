# 021 — Runtime Boundary Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Amend `framework/constitution.md` §text-first-artifacts opening paragraph

- [x] Replace the existing opening paragraph of §text-first-artifacts. New text retains all four load-bearing properties (Edit-driven write path, glanceable PRs, rare merge conflicts, markdown as source of truth) verbatim. The "no bootstrap tooling beyond the AI agent" clause is replaced by language distinguishing the markdown framework (standalone, no tooling) from the optional runtime (opt-in, with an inline link to `[§runtime-boundary](#runtime-boundary)`).
- [x] Verify the existing principle bullets immediately below the opening paragraph (markdown by default, frontmatter for metadata, relative links not wiki-links, derived views gitignored, exceptions require amendment) are unchanged.
- **Done when**: `git diff framework/constitution.md` shows the opening paragraph replaced and the principle bullets untouched.

## 2. Add the §runtime-boundary subsection to `framework/constitution.md`

- [x] Add the `<!-- §runtime-boundary -->` anchor marker on a line of its own, followed by the `### Runtime Boundary` heading. Place after the existing "Validation Severity" subsection and before §drift-prevention.
- [x] Write the subsection body with: five principles (each MUST or MUST NOT, RFC 2119 register), three eligibility criteria (Deterministic, Currently mechanical, Degradation-not-failure), opt-in invariant (CI MUST exercise the cycle with the runtime binary absent), versioning rule (lockstep), non-scope statement (MUST NOT for each of: spec authoring tool, workflow orchestrator, long-running service, storage layer; closes with "Lifting any of these exclusions requires a constitutional amendment").
- [x] Add the one-line forward pointer: *"Specific capabilities are introduced through their own feature specs, beginning with spec 022 (deterministic runtime)."*
- **Done when**: `grep -c '<!-- §runtime-boundary -->' framework/constitution.md` returns 1; the subsection contains the literal strings "MUST NOT" appearing in both the principles and the non-scope list; the forward-pointer sentence is present.

## 3. Add the §drift-prevention canonical sources row

- [x] In `framework/constitution.md` §drift-prevention canonical sources table, add a row whose Fact column reads "Runtime contract / boundary" and whose Canonical-source column points at `framework/constitution.md` §runtime-boundary, placed in alphabetical-by-fact position (after "Rules artifact tier definition," before "Security rule file format and ID conventions").
- **Done when**: the row exists in the table; `grep -c '§runtime-boundary' framework/constitution.md` returns at least 2 (the anchor and the table entry).

## 4. Create `framework/runtime-tools.txt`

- [x] Create the file with a comment header explaining its purpose and linking to §runtime-boundary in `framework/constitution.md`. No tool entries — spec 022 populates this.
- **Done when**: the file exists with only comment lines (each starting `#`) and is referenced from the tool-coverage lint script.

## 5. Create `scripts/lint-tool-coverage.sh`

- [x] Bash script that reads `framework/runtime-tools.txt` (skipping blank lines and `#` comments), iterates over `framework/commands/*.md`, finds each match of each tool name, and for each match scans forward 20 lines for any case-insensitive occurrence of: `Otherwise`, `Fallback`, `If unavailable`, `markdown-only path`.
- [x] On match-without-fallback, print `path:line: missing fallback for tool '<name>'`. Exit 1 if any error; exit 0 if all references have a fallback OR the manifest is empty.
- [x] `chmod +x` and verify it runs with the empty manifest (passing trivially).
- **Done when**: `bash scripts/lint-tool-coverage.sh` exits 0 against the current repo; a manual test with a temporary tool name in the manifest and a contrived violation in a command file exits 1 with a clear error.

## 6. Create `scripts/lint-frontmatter.sh`

- [x] Bash script that finds all `specs/**/spec.md`, `specs/**/spec-and-plan.md`, and `specs/**/scenarios/*.md` files, then for each: confirms a `---` delimited frontmatter block exists at the top; verifies `status:` value is one of `draft`, `clarified`, `planned`, `in-progress`, `done` (when the field is present); verifies `dependencies:` parses as either an inline bracketed list (`dependencies: [a, b]`) or an empty list (`dependencies: []`).
- [x] On failure, print `path: <reason>`. Exit 1 if any file fails; 0 if all pass.
- [x] `chmod +x` and verify it passes against the current repo.
- **Done when**: `bash scripts/lint-frontmatter.sh` exits 0 against every existing spec and scenario file.

## 7. Create `.github/workflows/markdown-only-pipeline.yml`

- [x] Workflow file with name `markdown-only-pipeline`, single job `markdown-only` on `ubuntu-latest`, triggered by `pull_request` (paths: `framework/**`, `specs/**`, `.claude/commands/**`) and by `push` to `main` (same paths).
- [x] Job steps: checkout; setup Node (for `npx markdownlint-cli2`); step (a) assert each name in `framework/runtime-tools.txt` is not on PATH; step (b) run all three `scripts/gen-*.sh --dry-run` and assert clean; step (c) `npx markdownlint-cli2`; step (d) `bash scripts/lint-tool-coverage.sh`; step (e) `bash scripts/lint-frontmatter.sh`.
- [x] Each step has a `name:` that names the corresponding spec check (a/b/c/d/e) so failures map directly to spec acceptance criteria.
- **Done when**: the workflow file passes `actionlint` (if installed) or `yq` parse; pushing a branch with the file triggers the job (verified via GitHub Actions UI on PR).

## 8. Run `/gov:analyze` against this spec

- [x] Run `/gov:analyze` targeted at `021-runtime-boundary` and resolve any hard-fail or blocking findings.
- [x] Anchor-resolution check passes for `§runtime-boundary` references.
- **Done when**: `/gov:analyze` reports no hard-fail and no blocking findings on this spec.

## 9. Run `npx markdownlint-cli2` across all touched files

- [x] Lint `framework/constitution.md`, `specs/021-runtime-boundary/*.md`, and any new markdown the previous tasks introduced.
- **Done when**: `npx markdownlint-cli2` exits 0 against the full repo.

## 10. Cross-spec impact sweep

- [x] Re-read the inline links in `spec.md` and `plan.md` body and confirm no other sibling spec needs an update because of decisions made here. The current expectation is that only 020-code-review is cited (as motivating evidence, not as a behavioral dependency), so no updates are needed elsewhere.
- **Done when**: confirmed in writing in the PR description that no §cross-spec-impact action was triggered.
