# govern-dir-consolidate

**Introduced in:** gvrn 0.22.0
**Summary:** Consolidate govern's per-project files under a `.govern/` directory — `.govern.toml` → `.govern/config.toml`, `.govern.session.toml` → `.govern/session.toml`, and the adopter-facing generators from root `scripts/` → `.govern/scripts/`.

## Background

govern previously scattered three per-project artifacts across the adopter's repo root: `.govern.toml` (committed config), `.govern.session.toml` (gitignored session state), and the three adopter-facing generators scaffolded into the project's root `scripts/` (`gen-spec-deps.sh`, `gen-cross-service-refs.sh`, `lib/specs-root.sh`). Consolidating them under a single `.govern/` directory keeps govern's footprint out of the repo root and out of a project's own `scripts/` (spec 042). The runtime reads the new locations with a fallback to the legacy root paths, so an adopter who upgrades gvrn before re-running `/govern` is never broken — this migration completes the cutover.

## Procedure

This migration has no runtime primitive; it is file moves the host performs directly. It moves all three concerns in one pass and runs under the batch migration consent (the §Pre-run Migrations "apply N migrations?" prompt) — there is **no** additional per-file prompt.

**Convergence rule (applies to every move below).** Because the runtime reads the new `.govern/` location first and falls back to the legacy root path (spec 042 §Transition and fallback), a lingering legacy file after a write has moved to `.govern/` is *dead* — the new file wins on read. So each move **converges** rather than skip-and-leaves: when the destination already exists, compare it to the legacy source — if identical, delete the legacy file silently; if they differ, delete the legacy file and emit one line `warning: {legacy} diverged from {destination}; removed the stale legacy copy ({destination} wins; the legacy content was already ignored — recover from git if needed).` A stale legacy file is never left in place. When the destination does not exist, move the file via `git mv` when the source is tracked (so the rename is recorded) or `mv` otherwise, preserving any adopter customization.

1. **Idempotency check.** Look for any of these legacy artifacts at the repo root:
   - `.govern.toml`
   - `.govern.session.toml`
   - `scripts/gen-spec-deps.sh`, `scripts/gen-cross-service-refs.sh`, `scripts/lib/specs-root.sh`

   If none is present, exit silently — the project is already on the `.govern/` layout (or never had these files).

2. **Move the config file.** If `.govern.toml` exists, move it to `.govern/config.toml` (creating `.govern/`) under the convergence rule. `.govern/config.toml` becomes the file this and every later step reads and writes — including the `[migrations].last_applied` write that records this migration, so the marker lands in the new location.

3. **Move the session file.** If `.govern.session.toml` exists, move it to `.govern/session.toml` under the convergence rule. (An adopter who ran the older `session-file-consolidate` migration under gvrn ≥ 0.22 already had their session written directly to `.govern/session.toml` by that migration's primitive, so this step finds no root file and is a no-op for them.)

4. **Move the adopter-facing generators.** For each of `scripts/gen-spec-deps.sh`, `scripts/gen-cross-service-refs.sh`, and `scripts/lib/specs-root.sh` that exists, move it to the matching path under `.govern/scripts/` (creating `.govern/scripts/` and `.govern/scripts/lib/`) under the convergence rule. Only these three shipped generators move; any other file under the adopter's `scripts/` is the adopter's own and is left untouched. If a moved generator is listed in `.govern/config.toml` `[pinned] files` under its old `scripts/…` path, rewrite that pin entry to the new `.govern/scripts/…` path so a customized (pinned) generator stays both discoverable at the new location and protected from overwrite.

5. **Pinned-invoker warning.** The generators' invokers — the shipped command bodies and the `govern-pre-commit` hook — are normally rewritten to `.govern/scripts/…` by the scaffolding pass. An adopter who has **pinned** an invoker keeps its old `scripts/…` reference, which would call a now-missing path. Pinning opts out of updates, so this migration does **not** rewrite pinned files; instead, for each file in `.govern/config.toml` `[pinned] files` that still contains a `scripts/gen-spec-deps.sh` or `scripts/gen-cross-service-refs.sh` reference, emit one line: `warning: pinned {file} still references scripts/…; update it to .govern/scripts/… — the generators have moved.` This makes the breakage visible rather than silent.

6. **Summary line.** When at least one artifact moved, report `reorganized govern files → .govern/: {comma-separated list of what moved}` in the post-scaffolding output. Omit the line entirely when nothing moved.

## Notes

- The migration is one-way. There is no reverse path.
- Only the three **adopter-facing** generators move to `.govern/scripts/`; govern's own maintainer-only scripts (audit families, linters, maintainer generators) stay at the framework repo's root `scripts/` and never ship, so `.govern/scripts/` structurally marks "ships to adopters."
- The gitignore entry for the session file is not regenerated here — `/.govern/session.toml` ships via the framework-managed `.gitignore` block, which the `apply-manifest` / `merge-managed-block` step rewrites on every `/govern` run, superseding any legacy `.govern.session.toml` line.
- The migration composes with `session-file-consolidate` (`0.10.0`, runs first): an ancient adopter's per-agent session JSON is translated straight to `.govern/session.toml` by that migration's primitive, so this one no-ops on the session file for them; a mid-era adopter (root `.govern.session.toml`, no JSON) has it moved here.
