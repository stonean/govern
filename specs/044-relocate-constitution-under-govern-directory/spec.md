---
status: planned
dependencies: [027-bootstrap-migration-registry, 042-consolidate-govern-per-project-files-under-govern-directory]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 044 — Relocate the shipped constitution to `.govern/constitution.md`

The Shared Files manifest ships `framework/constitution.md` to the adopter repo root as `constitution.md` with `update` strategy — the most prominent govern-owned, rewritten-on-every-`/govern`-run file still landing at an adopter's repo root after [042-consolidate-govern-per-project-files-under-govern-directory](../042-consolidate-govern-per-project-files-under-govern-directory/spec.md) consolidated config, session state, and the shipped generators under `.govern/` (the only other `update`-strategy root file, the `.markdownlint-cli2.jsonc` linter config, stays at root deliberately — markdownlint-cli2 discovers its config from the project root). This feature moves the shipped destination to `.govern/constitution.md`, updates every shipped artifact that names the adopter-root path (templates, command bodies, docs), and migrates existing adopters through a new entry in the [bootstrap migration registry](../027-bootstrap-migration-registry/spec.md).

## Motivation

042's principle was that govern owns one predictable namespace instead of polluting the repo root; its Motivation called the root dotfiles and the `scripts/` intrusion out by name but scoped itself to config, session, and generators. The constitution was left behind: a framework-managed file, overwritten on every `/govern` run unless pinned, sitting at the top level of the adopter's repo as if it were the project's own content. Root placement litters the adopter's directory with a file the adopter does not author, and it is inconsistent with where every other govern-owned artifact now lives (`.govern/` for machinery, `{specs-root}/rules/` and `{specs-root}/templates/` for shipped pipeline content, `.githooks/govern-pre-commit` for the hook).

**Accepted trade-off — human discoverability.** Unlike 042's machinery, the constitution is human-facing: the shipped `README.md` and `AGENTS.md` seeds link to it as the project's top-level governance document. Moving it into a dot-directory buries it slightly for a human browsing the repo root. Agents are unaffected — the `CLAUDE.md` `@import` follows any path, and the pipeline commands are rewritten to the new location in the same `/govern` run that moves the file. The shipped seeds keep linking to the constitution at its new path, so the document stays one click away from the README.

## New destination

| Source Path | Destination Path (old) | Destination Path (new) |
| --- | --- | --- |
| `framework/constitution.md` | `constitution.md` | `.govern/constitution.md` |

- The manifest strategy is unchanged: `update` (rewritten on every `/govern` run), pinnable via `[pinned] files`.
- `.govern/constitution.md` is committed — it joins `config.toml` and `scripts/` as tracked content; the gitignore entry stays scoped to `/.govern/session.toml` and needs no change.
- Nothing else about the file changes: same content, same anchors (`<!-- §name -->` markers), same role as the per-project governance source the pipeline commands load.

## Shipped references to the adopter path

Every shipped artifact that names the constitution's adopter-side location resolves `.govern/constitution.md`:

- **Project seed templates** — the `CLAUDE.md` seed's `@import constitution.md` line, the `AGENTS.md` seed's constitution links and artifact list, and the `README.md` seed's governance links.
- **Command bodies** — the commands that load or name the adopter constitution (`target`'s once-per-session read, `specify`'s not-yet-loaded fallback read, and any other body naming the root path). `analyze`'s anchor-resolution rule keeps its dual-path form with the adopter side updated: `framework/constitution.md` in govern's own repo; `.govern/constitution.md` at the adopter repo root.
- **Bootstrap prose** — the Shared Files manifest row and the `[pinned] files` schema example in `govern.md` that illustrates pinning with `"constitution.md"`.
- **Framework documentation** — `AGENTS.md`'s description of `framework/constitution.md` as the shipped source (its sync-target wording), and any other live-artifact reference to the adopter-root path, per the no-dead-references rule.

govern's own repository needs no file move: it carries no root `constitution.md` — its `CLAUDE.md` imports `framework/constitution.md` directly, which remains the canonical source shipped to adopters.

## `/govern` migration: move an existing adopter's constitution

A new entry in the [bootstrap migration registry](../027-bootstrap-migration-registry/spec.md) moves an adopter on the old layout, running under the bootstrap's existing batch migration consent with no additional per-file prompt (the same posture 042 established: govern unambiguously owns the file). It ships under `introduced_in = "0.24.0"` (the next minor cut from the published gvrn 0.23.0) with no `sunset_after` — the directory-reorg posture `govern-dir-consolidate` established: kept active indefinitely so a straggler converges on their next `/govern` run (see Resolved Questions). The migration is idempotent — a no-op when no root `constitution.md` is present — and:

- Moves `constitution.md` → `.govern/constitution.md`, preserving git history (`git mv` when tracked, `mv` otherwise) and preserving the adopter's content as-is — a pinned, hand-customized constitution moves with its customizations intact.
- Rewrites the adopter-owned seed files' references in place: the `CLAUDE.md` `@import constitution.md` line and the `AGENTS.md` / `README.md` constitution links, when they match the shipped legacy form. These files are `create`-strategy (never overwritten by scaffolding), so the migration is the only mechanism that updates them — the same in-place-edit precedent 042 set for superseding the adopter's gitignore session line. In a seed file that exists, a reference line that is absent or hand-altered beyond recognition is left unmodified and named in a warning — never silent breakage. A wholly absent seed file is a silent no-op (see Edge cases).
- Re-points a `[pinned] files` entry naming `constitution.md` to `.govern/constitution.md`, so the pin travels with the file (042's rule: a pinned file moves together with its pin).
- Converges on a destination collision rather than skip-and-leaves: a legacy root file identical to `.govern/constitution.md` is removed silently; a divergent legacy file is removed with a prominent warning naming it (recoverable from git). No stale root copy is left in place.
- Emits a single summary line naming what moved (omitted when nothing moved), and is registered in `framework/migrations.toml` with its procedure body at `framework/migrations/{id}.md`, satisfying the audit's migration-coverage invariants.

**No read fallback is needed** — this is the point where the move is simpler than 042. No runtime primitive reads the constitution (loading it is a host responsibility in the command bodies), so there is no runtime path constant to guard with a legacy fallback. The readers are the shipped command bodies, and they are rescaffolded to the new path in the same `/govern` run that moves the file — readers and location cut over atomically. An adopter who upgrades gvrn but has not re-run `/govern` still holds old command bodies pointing at the still-present root file: consistent either way.

## Edge cases

- **Pinned command bodies referencing the old path.** An adopter who pinned a command file (e.g. `target.md`) keeps its old `constitution.md` read reference after the file moves — pinning opts out of updates, so the migration does not rewrite pinned files; it surfaces a warning naming any pinned command file that still references the root constitution path (042's pinned-invoker precedent).
- **Ordering against the govern-directory migration.** An adopter far enough behind runs 042's govern-directory migration first (creating `.govern/`) and then this one, in registry order; the constitution move also tolerates running standalone by creating `.govern/` if absent.
- **Interrupted run.** `[migrations].last_applied` advances only after the whole procedure completes; the move and each reference rewrite are independently idempotent, so a re-run converges without double-moving.
- **Adopter deleted the root constitution.** The move no-ops; the manifest's `update` strategy lands a fresh copy at `.govern/constitution.md` in the same run, and the reference rewrites still apply.
- **Seed file wholly absent.** An adopter on a non-Claude agent may carry no `CLAUDE.md`, and a deleted `README.md` is a legitimate adopter choice. The rewrite pass silently skips a seed file that does not exist — the missing-reference warning is scoped to files that exist with their constitution reference absent or hand-altered, so an agent lineup that never had the file is not warned about on every run.

## Acceptance Criteria

- [ ] A freshly adopted project (first `/govern` run) has the constitution at `.govern/constitution.md` (committed, `update` strategy, pinnable); no govern-owned `update`-strategy file other than the root-anchored `.markdownlint-cli2.jsonc` linter config (kept at root for markdownlint-cli2's config discovery) lands at the adopter repo root.
- [ ] The shipped `CLAUDE.md` seed imports `.govern/constitution.md`, and the shipped `AGENTS.md` / `README.md` seeds link the constitution at its new path; no shipped template references a root `constitution.md`.
- [ ] Every shipped command body that names the adopter constitution path reads `.govern/constitution.md`; `analyze`'s anchor-resolution rule resolves `framework/constitution.md` in govern's own repo and `.govern/constitution.md` at an adopter root.
- [ ] Running `/govern` in a project on the old layout moves root `constitution.md` → `.govern/constitution.md` with history preserved, under the existing batch migration consent with no per-file prompt, and is a no-op on a project already on the new layout.
- [ ] The migration rewrites the adopter's `CLAUDE.md` `@import` line and `AGENTS.md` / `README.md` constitution links when they match the shipped legacy form; in a seed file that exists, an absent or hand-altered reference it cannot rewrite is left unmodified and named in a warning, while a wholly absent seed file is silently skipped.
- [ ] A `[pinned] files` entry naming `constitution.md` is re-pointed to `.govern/constitution.md` and continues to protect the moved file from overwrite; a pinned command body still referencing the root path is left unmodified but named in a migration warning.
- [ ] On a destination collision the migration converges: an identical legacy root file is removed silently, a divergent one is removed with a prominent warning naming it, and no stale root copy remains.
- [ ] The migration is registered in `framework/migrations.toml` with `introduced_in = "0.24.0"` and no `sunset_after`, with a procedure file at `framework/migrations/{id}.md`, and the audit's migration-coverage invariants pass.
- [ ] The runtime's parity and golden fixtures that encode the shipped constitution destination reflect `.govern/constitution.md`, and the parity suite passes.
- [ ] No stale reference to the adopter-root `constitution.md` path remains in live artifacts (`framework/`, `runtime/`, `docs/`, `README.md`, `AGENTS.md`), per the no-dead-references rule; govern's own repo keeps importing `framework/constitution.md` unchanged.

## Open Questions

*None — all resolved. See Resolved Questions below.*

## Resolved Questions

- **Human discoverability of the relocated constitution.** Resolved: the existing seed links are sufficient — no new governance blurb. The README seed already leads its Documentation section with the constitution *plus* a one-line description of its role, and links it a second time in context from the Development Pipeline section; the `AGENTS.md` seed links it in its opening line and artifact list. Re-pointing those existing references to `.govern/constitution.md` preserves exactly today's discoverability: the README's lead bullet already is the "short governance blurb" the question contemplates, so a dedicated section would duplicate it. The only reader served worse is one browsing the file tree while skipping the README, and that reader is served by `.govern/` being govern's one predictable, greppable namespace (042's own rationale).

- **Release version and sunset.** Resolved: the migration ships under `introduced_in = "0.24.0"` — the next minor cut from the published gvrn 0.23.0 (runtime CHANGELOG) — with **no `sunset_after`**. Every migration to date ships under its own minor cut (`govern-dir-consolidate` @ 0.22.0, `workflows-sunset` @ 0.23.0), and 043 established that a cut whose runtime changes are comment- or fixture-only is legitimate grounds for a version when the migration's `introduced_in` needs a published release — applicable here, since no runtime primitive reads the constitution. The no-sunset posture mirrors `govern-dir-consolidate`'s registry comment verbatim in kind (`framework/migrations.toml`): a high-impact directory reorg is kept active indefinitely so an adopter still on the old layout is converged on their next `/govern` run, however late; `workflows-sunset`'s time-box applies to feature removals, not layout moves. Rejected: `sunset_after` (risks orphaning adopters who never re-run `/govern`) and folding into a later bundled release (delays the cleanup for no benefit).
