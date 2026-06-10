---
section: "Generators and Hooks"
---

# Tracked-specs-not-worktree

## Context

§Generators and Hooks / AC12 specify the govern-repo and adopter pre-commit hooks as: *"Runs all generators unconditionally on every commit and stages any changes — trades a fraction of a second per commit for a one-line implementation that can't get the gate logic wrong."* The implementation enumerated specs with a **worktree glob** (`specs/[0-9][0-9][0-9]-*/spec.md`) inside both `gen-spec-deps.sh` and `gen-readme-table.sh`, and the hooks then force-`git add`ed every matched `spec.md` plus `README.md`.

That conflates *what is in the worktree* with *what is being committed*. An untracked, in-progress draft — e.g. a `/specify` spec the author created but has not yet `git add`ed — sits in the worktree. So any commit, on any unrelated change, would: rewrite the draft's frontmatter, regenerate the README to list it, force-`git add` the draft (`A`) and the README, and commit both. A half-written draft with a malformed or circular link would additionally fail `gen-spec-deps.sh`'s cycle check under `set -e` and **block the unrelated commit entirely**.

This is the precise failure the spec exists to prevent. 017's governing principle (AGENTS.md: *"Never design framework features that depend on human diligence or discipline"*) is what motivated auto-staging in the first place — so authors need not *remember* to run generators. But the worktree-scoped implementation imposed a *new* discipline: "don't have a draft spec open when you commit unrelated work." The fix removes that discipline rather than documenting it.

## Behavior

- Both generators (`gen-spec-deps.sh`, `gen-readme-table.sh`) MUST enumerate the feature-spec set from the **git index** (`git ls-files`), not a worktree glob. A spec that is tracked or staged is processed; an untracked draft is not — it is never rewritten, never enters the README table, and never enters the dependency/cycle graph.
- The pre-commit hooks (`.githooks/pre-commit` and the shipped `framework/bootstrap/hooks/govern-pre-commit`) MUST stage only spec files **already part of the commit** (`git diff --cached`). A generator's derived `dependencies:` rewrite on an already-staged spec is captured; untracked drafts and unstaged edits to other tracked specs are not swept in.
- A brand-new spec is processed and staged on the commit that first `git add`s it (it is in the index from that point) — the common "create a spec, then commit it" flow is unchanged.
- Outside a git repo (no index to consult), the generators fall back to the worktree glob. This path is exercised by the existing `gen-spec-deps` fixture tests, which run against plain temp dirs.
- A fixture under the `gen-spec-deps` test surface covers the core invariant: in a git repo with one tracked spec and one untracked draft (both carrying body links), the tracked spec's `dependencies:` are derived and the untracked draft is left byte-for-byte untouched.

## Edge Cases

- **Unstaged edits to a tracked spec while committing something else**: the generator still rewrites that spec's `dependencies:` in the worktree (it is tracked), but the hook does not stage it (it is not in `git diff --cached`). The regenerated line stays unstaged with the author's other unstaged edits — the commit's scope is respected.
- **CI / fresh checkout**: every spec is tracked, so the index scope and the old worktree glob are identical — AC10/AC11's "fresh checkout produces no diff" invariant is preserved.
- **`git rm`'d spec**: removed from the index, so it drops out of the README table and dep graph on the same commit, matching the deletion.
