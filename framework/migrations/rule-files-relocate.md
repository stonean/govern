# rule-files-relocate

**Introduced in:** _set during 027.3 back-fill_
**Summary:** Relocate rule files from `specs/` root to `specs/rules/` (subsumes `configuration.md` → `configuration-cross.md`).

## Procedure

Rule files installed in adopter projects live under `specs/rules/{rule-set}.md` so the top-level `specs/` directory stays focused on feature spec directories (`specs/NNN-*/`), `inbox.md`, `README.md`, and the seed system files. Adopters scaffolded before this change have rule files at `specs/{rule-set}.md`, which `/{project}:review` and `/{project}:analyze` no longer discover (the discovery walk targets `specs/rules/*.md`).

This migration subsumes the earlier `configuration.md` → `configuration-cross.md` rename (closed-suffix policy, spec 024): when the legacy `specs/configuration.md` is found, it is renamed **and** relocated in one move.

1. **Idempotency check.** Walk the top level of `specs/` for files matching either:
   - `configuration.md` (pre-closed-suffix layout)
   - `*-backend.md`, `*-frontend.md`, or `*-cross.md` (closed-suffix rule files at the `specs/` root)

   For each match, compute the destination:
   - `specs/configuration.md` → `specs/rules/configuration-cross.md`
   - otherwise → `specs/rules/{basename}`

   Skip any source listed in `.govern.toml` `pinned.files` (path comparison after placeholder resolution) and emit one line: `warning: {source} is pinned; leaving in place — /{project}:review and /{project}:analyze will not discover it until moved manually.`

   Skip any source whose destination already exists in `specs/rules/` and emit one line: `warning: {destination} already exists; skipping relocation of {source}.`

   If no eligible moves remain, exit silently.

2. **Batch prompt.** If at least one eligible move remains, prompt once:

   ```text
   Found N legacy rule file(s) at the specs/ root.
   Move to specs/rules/? (Y/n)
   ```

3. **Action.** On confirm, create `specs/rules/` (if missing) and rename each eligible file via `mv` (or `git mv` when the source is tracked, so the rename is recorded). On decline, emit one warning per skipped file: `warning: {source} kept; /{project}:review and /{project}:analyze will not discover it until moved manually.`

4. **Summary line.** When N > 0 files were relocated, report `relocated N rule file(s) → specs/rules/` in the post-scaffolding output. Omit the line otherwise.

Rule IDs (`BE-AUTHN-*`, `FE-XSS-*`, `CFG-CONST-*`, etc.) are content-anchored and unchanged by the relocation; every existing citation continues to resolve.
