---
status: draft
dependencies: [012-multi-agent-govern, 026-framework-self-audit]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 027 — Command Source Templating

`/audit` Family 4 ([`scripts/audit/placeholder-roundtrip.sh`](../../scripts/audit/placeholder-roundtrip.sh)) flags command source files under `framework/commands/`, `framework/bootstrap/`, and `.claude/commands/gov/` that contain literal `/gov:` (and bare `gov:`) references where `/{project}:` placeholders belong. The framework's command sources are pre-substituted for the gov project rather than properly templated; adopters scaffolding into a non-gov project (e.g., `anvil`) receive command files that internally reference `/gov:foo` instead of `/anvil:foo`.

Origin: spec [026](../026-framework-self-audit/spec.md)'s `/audit` v1 advisory findings, 2026-05-18. Captured via the inbox during the 2026-05-17 groom pass.

## Motivation

Spec [012](../012-multi-agent-govern/spec.md) establishes the multi-agent contract: command files use `{project}` placeholders and are substituted at bootstrap time so each adopter sees its own command prefix. Family 4's findings reveal that contract is honored in `framework/bootstrap/govern.md`'s substitution pipeline but not in the source files the pipeline rewrites — many backticked command references in command source bodies are already pre-substituted to `/gov:`.

Concrete failure mode: an adopter runs `/govern` against an "anvil" project. The substitution pipeline rewrites the slash-command filenames and the `/{project}:` placeholders in command bodies. Backticked literals like `/gov:plan` survive the rewrite and surface to anvil users as dead references — the anvil host has no `/gov:plan` command, only `/anvil:plan`.

Secondary motivation: the related inbox item to flip `continue-on-error: false` on `.github/workflows/markdown-only-pipeline.yml` step (h) and `.github/workflows/runtime-release.yml`'s `audit` job cannot land until Families 4, 8, and 9 close — this spec is one of the three preconditions for `/audit` becoming a hard CI gate (spec 026 Q4).

## Path Decision

Two viable paths; this spec picks one before scoping the remediation work:

- **Path A — Re-template all sources.** Mechanical sweep across `framework/commands/*.md`, `framework/bootstrap/govern.md`, `framework/bootstrap/configure/*.md`, with `.claude/commands/gov/*.md` regenerated via `scripts/gen-claude-commands.sh`. Restores spec 012's contract end-to-end. Larger surface; preserves substitution semantics.
- **Path B — Declare `/gov:` canonical, tighten Family 4.** Treat `/gov:` references in command sources as the project-local rendering for the gov project and update `scripts/audit/placeholder-roundtrip.sh` to skip those backticks under a documented exception. Smaller surface but contradicts spec 012's multi-agent contract — the contract would need explicit amendment.

This decision is the gating Acceptance Criterion below. The remaining criteria depend on the answer.

## Acceptance Criteria

- [ ] Path decision (A vs. B) is recorded as a Resolved Question in this spec with rationale tied to spec 012's multi-agent contract.
- [ ] All command source files in scope (Path A: re-templated; Path B: deemed canonical) are updated consistently with the chosen path. The chosen path's "in-scope" surface is enumerated explicitly in this spec before tasks are derived.
- [ ] `bash scripts/audit/placeholder-roundtrip.sh` exits 0 with no findings on a clean working tree at HEAD.
- [ ] `bash scripts/audit/run-all.sh` exits 0; Family 4 no longer surfaces advisory findings.
- [ ] Path A only: `scripts/gen-claude-commands.sh` regeneration is run and committed; the regenerated `.claude/commands/gov/*.md` files are byte-identical to a fresh regeneration.
- [ ] Path B only: spec 012's multi-agent contract assertion(s) about `/{project}:` placeholders in command bodies are explicitly amended via a `/gov:ask` cycle on 012 with a corresponding clarification recorded.
- [ ] The related inbox item — flipping `.github/workflows/markdown-only-pipeline.yml` step (h) and `.github/workflows/runtime-release.yml`'s `audit` job `continue-on-error` to `false` — remains tracked for the post-026-Q4 gate flip but is explicitly out of scope here; this spec resolves only the Family 4 precondition.

## Applicable Rules

<!-- No cross-cutting rule governs this remediation. This is framework drift that
     `/audit` was built to surface; resolving it restores adherence to spec 012's
     multi-agent contract rather than a rule-file constraint. -->

## Open Questions

- **Path A vs. Path B.** Default-favored direction is Path A because it preserves spec 012's contract without amendment and matches the bootstrap pipeline's substitution semantics; Path B is the smaller-surface alternative but requires amending 012. Resolve via `/gov:clarify`.
- **Scope of "command source files" under Path A.** Whether Path A's sweep includes `framework/commands/*.md` only, or also `framework/bootstrap/govern.md` and `framework/bootstrap/configure/*.md`. The inbox item lists all three; resolve by enumerating the exact file set the chosen path covers.
