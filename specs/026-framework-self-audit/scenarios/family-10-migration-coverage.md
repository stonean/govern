---
section: "Follow-on scenarios"
---

# Family-10-migration-coverage

## Context

[`/audit`](../../../framework/commands/audit.md) shipped in spec 026 with nine family checks (1–9). Spec [027 — Bootstrap Migration Registry](../../027-bootstrap-migration-registry/spec.md) extends `/audit` with a tenth family — `scripts/audit/migration-coverage.sh` — that verifies the new `framework/migrations.toml` registry is consistent with its adjacent `framework/migrations/{id}.md` procedure files and with current `framework/` state.

This scenario records the extension on 026 per §cross-spec-impact: spec 026 was at `status: done` when 027 was written, so the Family 10 addition lands as a scenario here rather than a body edit, with a back-link to its driving spec.

Origin: spec 027 plan + tasks, 2026-05-21.

## Behavior

Family 10 follows the same contract as Families 1–9: stdout findings, exit 0/1, read-only, idempotent. Three checks run unconditionally on every `bash scripts/audit/run-all.sh` invocation:

1. **10a no-orphan-procedure-files.** For each file matching `framework/migrations/*.md`, verify a `[[migrations]]` entry in `framework/migrations.toml` has `procedure_file = "framework/migrations/{id}.md"` referencing it. A procedure file with no registry entry is an orphan — either the entry was deleted without removing the file, or the file was added without a registry entry.
2. **10b no-stale-target-paths.** For each active registry entry, iterate `target_paths`. For entries that start with `framework/` (the framework-side artifact that was removed), verify the path does **not** exist in current `framework/`. Adopter-relative paths (paths not prefixed with `framework/`) are skipped — they cannot be verified from this repo. Glob patterns (containing `*`) are expanded and each match is checked. A path that still exists when the registry says "removed" is a real drift.
3. **10c no-broken-procedure-references.** For each `[[migrations]]` entry, verify the `procedure_file` path exists on disk. A dangling reference means the procedure file was renamed or deleted without a TOML update.

Family 10 is invoked from `scripts/audit/run-all.sh` immediately after Family 9 and exits with the OR of the three checks' findings. Its findings format matches the rest: `migration-coverage | {location} | {message} | {suggested-fix}`.

The CHANGELOG.md archived-entry parsing piece of 10b is deferred until the first sunset commit establishes the archive format by example — the script currently carries a TODO for this and only parses the active registry. The bootstrap loop and Family 10's two other checks function fully without it.

## Edge Cases

- **Procedure file with a typo'd id**. Family 10 surfaces both checks: 10a (orphan procedure file) AND 10c (broken procedure_file reference), unless the typo is in both locations consistently — in which case 10b might still catch it if the path doesn't exist. Multiple findings on the same root cause are acceptable; they all point at the same fix.
- **Registry entry with no `procedure_file` field**. Currently treated as `procedure_file = ""`, which fails 10c (the empty string path doesn't exist). Acceptable v1 behavior; a future enhancement could enforce the field's presence schematically.
- **Glob in `target_paths` matches a pinned file in adopter context**. Not Family 10's concern — pinned-file handling lives in the bootstrap loop and procedure files. Family 10 only sees the registry's text.
- **Two registry entries reference the same `procedure_file`**. Not caught by Family 10 as designed (each entry passes 10c individually). The bootstrap loop's duplicate-id guard catches it on read; an additional 10d sub-check could enforce procedure-file uniqueness, captured as future work.
- **A new family check ships before Family 10's flip to a hard gate**. Inherits soft-launch (`continue-on-error: true`) per the [audit-ci-hard-gate](audit-ci-hard-gate.md) scenario's contract. Flip applies only to families whose v1 framework drift is resolved.

## Open Questions

- **Should Family 10 enforce procedure-file uniqueness (no two entries share a `procedure_file`)?** Currently not checked. The bootstrap loop's duplicate-id guard makes shared procedure files inconsistent at runtime, so the practical risk is small. Defer until a real case surfaces.
- **CHANGELOG.md archived-entry format**. The first sunset commit establishes the heading shape and the `target_paths` placement; once that's committed, Family 10's CHANGELOG parser can be implemented against a real example rather than a speculative one. Track via the spec 027 known-limitations section.

## Resolved Questions

*None.*
