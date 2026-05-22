# spec-and-plan-sunset

**Introduced in:** gvrn 0.5.0
**Summary:** Rename `specs/*/spec-and-plan.md` → `specs/*/spec.md` (lightweight-track sunset).

## Procedure

The lightweight track was removed in spec 023. Adopters who scaffolded under the prior dual-template model may still have `spec-and-plan.md` files at any non-`done` status under `specs/`. Pipeline commands now look for `spec.md` only — those files would fail the "spec does not exist" gate on the next command.

1. **Idempotency check.** Walk `specs/*/spec-and-plan.md`. If no matches, exit silently.
2. **Per-file prompt.** For each match, prompt the user with the source path and the proposed destination:

   ```text
   Found legacy spec-and-plan.md: specs/{NNN-feature}/spec-and-plan.md
   Rename to specs/{NNN-feature}/spec.md? (Y/n)
   ```

3. **Per-file action.** On confirm, rename via `mv`. On decline, emit a warning and continue with the next match:

   ```text
   warning: specs/{NNN-feature}/spec-and-plan.md kept; pipeline commands will fail on this feature until renamed manually.
   ```

4. **Summary line.** When N > 0 files were renamed, report `migrated N spec-and-plan.md files` in the post-scaffolding output. Omit the line when N = 0.

Files at `status: done` are also renamed (the rename is just a filename change; body and frontmatter are unchanged, so the frozen-archaeology rule is preserved by the byte-for-byte identity of the file content).
