# constitution-relocate

**Introduced in:** gvrn 0.24.0
**Summary:** Relocate the shipped constitution from the adopter repo root to `.govern/constitution.md`, re-pointing the `CLAUDE.md` import, the `AGENTS.md` / `README.md` links, and any `[pinned] files` entry.

## Background

The Shared Files manifest previously shipped `framework/constitution.md` to the adopter repo root as `constitution.md` — the most prominent govern-owned, rewritten-on-every-`/govern`-run file still landing at the repo root after the `govern-dir-consolidate` migration moved config, session state, and the shipped generators under `.govern/` (spec 042). Spec 044 moves the shipped destination to `.govern/constitution.md`. No runtime fallback pairs with this migration — no runtime primitive reads the constitution; its readers are the shipped command bodies, which the same `/govern` run rescaffolds to the new path, so readers and location cut over atomically.

## Procedure

This migration has no runtime primitive; it is a file move plus reference rewrites the host performs directly. It runs under the batch migration consent (the §Pre-run Migrations "apply N migrations?" prompt) — there is **no** additional per-file prompt. Registry ordering by `introduced_in` places it after `govern-dir-consolidate`, so `.govern/` normally exists already; step 2 still creates it if absent, so the migration tolerates running standalone.

**Convergence rule (for the move in step 2).** The manifest's `update` pass writes `.govern/constitution.md` from this run onward, so a lingering root copy after the move is *dead* — nothing reads it. The move **converges** rather than skip-and-leaves: when the destination already exists, compare it to the legacy root file — if identical, delete the legacy file silently; if they differ, delete the legacy file and emit one line `warning: constitution.md diverged from .govern/constitution.md; removed the stale legacy copy (.govern/constitution.md wins; the legacy content was already ignored — recover from git if needed).` A stale root copy is never left in place. When the destination does not exist, move the file via `git mv` when the source is tracked (so the rename is recorded) or `mv` otherwise, preserving any adopter customization — a pinned, hand-edited constitution moves with its edits intact.

1. **Idempotency check.** If no `constitution.md` exists at the repo root, exit silently — the project is already on the `.govern/` layout (or never had the file; the manifest's `update` strategy lands a fresh `.govern/constitution.md` later in this run either way).

2. **Move the file.** Move root `constitution.md` to `.govern/constitution.md` (creating `.govern/` if absent) under the convergence rule above.

3. **Re-point a pin.** If `.govern/config.toml` `[pinned] files` lists `constitution.md`, rewrite that entry to `.govern/constitution.md` so the pin travels with the file — a customized (pinned) constitution stays both discoverable at the new path and protected from overwrite.

4. **Rewrite seed references.** The three adopter-owned (`create`-strategy) seed files reference the constitution in the shipped legacy forms; scaffolding never overwrites them, so this migration is the only mechanism that updates them. For each file below **that exists**, rewrite lines matching the shipped legacy form; a file that exists but whose reference is absent or hand-altered beyond the forms below is left unmodified and named in one line — `warning: {file} does not carry the expected constitution.md reference; update it to .govern/constitution.md by hand — the constitution has moved.` A wholly absent file is silently skipped (an adopter on a non-Claude agent carries no `CLAUDE.md`; a deleted `README.md` is a legitimate choice — never warn about a file that does not exist).
   - `CLAUDE.md` — the `@import constitution.md` line → `@import .govern/constitution.md`.
   - `AGENTS.md` — inline links and artifact-list mentions of `constitution.md` (the shipped forms `[constitution.md](constitution.md)` and the `` `constitution.md` `` list item) → the `.govern/constitution.md` path.
   - `README.md` — the Documentation link `[constitution.md](constitution.md)` and the pipeline link `(constitution.md#development-pipeline)` → the `.govern/constitution.md` paths.

5. **Pinned-command warning.** The shipped command bodies that read the constitution are normally rewritten to `.govern/constitution.md` by the scaffolding pass. An adopter who has **pinned** a command file keeps its old root-path read, which would look for a now-missing file. Pinning opts out of updates, so this migration does **not** rewrite pinned files; instead, for each file in `.govern/config.toml` `[pinned] files` that is a shipped command body still containing a bare `constitution.md` reference, emit one line: `warning: pinned {file} still references constitution.md at the repo root; update it to .govern/constitution.md — the constitution has moved.` This makes the breakage visible rather than silent.

6. **Summary line.** When the file moved (or a stale legacy copy was removed), report `relocated constitution → .govern/constitution.md{, re-pointed pin}{, updated N seed reference(s)}` in the post-scaffolding output. Omit the line entirely when nothing moved.

## Notes

- The migration is one-way. There is no reverse path.
- Interrupted runs converge: `[migrations].last_applied` advances only after the whole procedure completes, and the move and each rewrite are independently idempotent, so a re-run finishes the remainder without double-moving.
- The gitignore needs no change: `.govern/constitution.md` is committed content, and the session-file entry (`/.govern/session.toml`) never covered it.
- Composes with `govern-dir-consolidate` (0.22.0, runs first): an adopter far enough behind gets `.govern/` created there; this migration only needs the directory and creates it if somehow absent.
