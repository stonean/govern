# 044 — Relocate the shipped constitution to `.govern/constitution.md` Plan

Implements [044 — Relocate the shipped constitution to `.govern/constitution.md`](spec.md).

## Overview

A markdown-and-registry change with zero runtime code changes: the Shared Files manifest row moves the shipped constitution's destination from the adopter repo root to `.govern/constitution.md`, every shipped artifact that names the adopter-root path is rewritten (seed templates, command bodies, bootstrap prose, framework docs), and a new `constitution-relocate` migration converges existing adopters. The gvrn 0.24.0 cut exists to anchor the migration's `introduced_in` (the 043 precedent for a no-behavior-change version); the parity suite is expected to pass untouched.

## Technical Decisions

### Manifest row and bootstrap prose

The govern-owned Shared Files table maps `framework/constitution.md` → `constitution.md` (`framework/bootstrap/govern.md:614`); the row's destination becomes `.govern/constitution.md`. The `[pinned] files` schema example that illustrates pinning with `"constitution.md"` (`govern.md:432`) is updated to the new destination path so the example stays copy-pasteable. No other `govern.md` reference names the adopter destination — the remaining constitution mentions cite the framework source (`framework/constitution.md`, e.g. `govern.md:520`), which is unchanged.

### The `constitution-relocate` migration

One new registry entry in `framework/migrations.toml`, modeled on `govern-dir-consolidate` (`framework/migrations.toml:68-86`):

- `id = "constitution-relocate"` (the `rule-files-relocate` naming precedent), `introduced_in = "0.24.0"`, **no `sunset_after`** with a comment mirroring `govern-dir-consolidate`'s (`migrations.toml:71-74`): high-impact layout move kept active indefinitely so stragglers converge. Registry ordering by `introduced_in` places it after `govern-dir-consolidate` (0.22.0), so `.govern/` exists by the time it runs; the procedure still creates `.govern/` if absent so it tolerates running standalone.
- `target_paths = ["constitution.md"]`.
- Procedure body at `framework/migrations/constitution-relocate.md`, following the `govern-dir-consolidate.md` template: idempotency check (no root `constitution.md` → exit silently), the same convergence rule verbatim in kind (`govern-dir-consolidate.md:14` — destination exists + identical → silent delete of legacy; divergent → delete with the one-line warning; otherwise `git mv` when tracked, `mv` otherwise), pin re-point (a `[pinned] files` entry `constitution.md` → `.govern/constitution.md`, the `govern-dir-consolidate.md:27` precedent), and the summary line convention (`govern-dir-consolidate.md:31`).
- **Seed-reference rewrites** — the migration's one extension beyond the 042 template. The three adopter-owned (`create`-strategy) seeds carry references in exactly the shipped legacy forms: `@import constitution.md` (`framework/templates/project/claude-md.md:3`), `[constitution.md](constitution.md)` and the `` `constitution.md` `` artifact-list line (`agents.md:9,53`), and `[constitution.md](constitution.md)` / `(constitution.md#development-pipeline)` (`project-readme.md:20,35`). For each seed file that exists, rewrite lines matching the shipped legacy form to the `.govern/constitution.md` path; a file that exists with the reference absent or hand-altered is left unmodified and named in a warning; a wholly absent seed file is silently skipped (non-Claude adopters carry no `CLAUDE.md`).
- **Pinned-command warning** — for each file in `[pinned] files` that is a shipped command body still containing a bare adopter-root `constitution.md` read, emit the `govern-dir-consolidate.md:29`-style warning naming it. Pinning opts out of updates; the warning makes the stale read visible.

### Command-body sweep — bare `constitution.md` means the adopter path

Shipped command bodies read or cite the constitution by the bare adopter-root path; each becomes `.govern/constitution.md`: `target.md:19`, `specify.md:25,68`, `groom.md:64`, `clarify.md:100`, and `analyze.md:123,178,216,220,228` plus `analyze.md:48`'s explicit dual-path rule (adopter side only — `framework/constitution.md` in govern's own repo stays). The sweep is grep-driven at implement time (`rg -n '\bconstitution\.md'` over `framework/`, excluding `framework/constitution.md` self-references and relative links that resolve to the framework source, e.g. `framework/migrations/spec-and-plan-sunset.md:26`'s `../constitution.md`) so no hidden reference survives — my scoping grep filtered out lines that mention both forms, so the implement-time sweep must not.

### Seed templates

The four seed-template references listed above (claude-md.md:3, agents.md:9,53, project-readme.md:20,35) are updated to `.govern/constitution.md`. Per the resolved discoverability question, no new governance blurb is added — the README seed's lead Documentation bullet already carries the role description.

### Govern's own docs — including a pre-existing dead link

- `AGENTS.md:9` links `[constitution.md](constitution.md)` — a **dead link today**: govern's repo carries no root `constitution.md` (verified by `ls`; its `CLAUDE.md` imports `framework/constitution.md` directly). Fix it to `framework/constitution.md` as part of this sweep.
- `AGENTS.md:15`'s "sync target of root `constitution.md`" wording is stale for the same reason; reword to name `.govern/constitution.md` as the adopter-side destination of the shipped source.
- `README.md:210,217,284` use `constitution.md` as the pinned-file and `update`-strategy examples; update to `.govern/constitution.md`.
- `docs/introduction.md` references only the framework source — no change (verified by grep).

### Runtime: no code change; 0.24.0 anchors the migration

No runtime primitive reads the constitution (loading it is host responsibility — `target.md:37`), and the parity suite's pinned-destination case uses a synthetic manifest whose pinned dest happens to be `framework/constitution.md` (`runtime/tests/parity.rs:116,183`) — it does not encode the real Shared Files destination, so no fixture or golden changes are expected; the parity suite must pass **without** re-blessing. The 0.24.0 cut is `runtime/Cargo.toml` + a `runtime/CHANGELOG.md` entry stating no runtime behavior changes and that the version exists so `constitution-relocate`'s `introduced_in` has a published release — verbatim the 043 precedent (CHANGELOG 0.23.0 entry). Publish `gvrn-v0.24.0` per the 043 release-task convention.

### Spec accuracy carve-out discovered while grounding

The Shared Files table also ships `.markdownlint-cli2.jsonc` to the adopter root with `update` strategy (`govern.md:630`), and it must stay there — markdownlint-cli2 discovers config from the project root, and relocating it would require `--config` plumbing through every lint invocation on every host path. Criterion 1 and the spec intro were tightened during planning to carve out this one root-anchored tool config rather than claim a constitution-only move empties the root of `update`-strategy files.

### No data model

File relocation and reference rewrites — no domain entities or data structures. `data-model.md` is intentionally absent.

### Test strategy

No new test artifacts. Verification is: the repo-wide stale-reference grep (clean), `npx markdownlint-cli2` over touched markdown (clean), the audit scripts (`scripts/audit/cross-doc-consistency.sh`, `scripts/audit/ssot-invariants.sh` — including Family 10's migration-coverage invariants over the new registry entry), and a green `cargo test` parity run with no golden/fixture diffs.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/bootstrap/govern.md` | Modify | Shared Files row → `.govern/constitution.md`; pinned schema example |
| `framework/migrations.toml` | Modify | Add `constitution-relocate` entry (0.24.0, no sunset) |
| `framework/migrations/constitution-relocate.md` | Create | Migration procedure body |
| `framework/commands/target.md` | Modify | Adopter constitution read path |
| `framework/commands/specify.md` | Modify | Adopter constitution read + cite paths |
| `framework/commands/groom.md` | Modify | Adopter constitution cite path |
| `framework/commands/clarify.md` | Modify | Adopter constitution cite path |
| `framework/commands/analyze.md` | Modify | Dual-path rule + cite paths |
| `framework/templates/project/claude-md.md` | Modify | `@import .govern/constitution.md` |
| `framework/templates/project/agents.md` | Modify | Constitution links |
| `framework/templates/project/project-readme.md` | Modify | Documentation + pipeline links |
| `AGENTS.md` | Modify | Fix dead root link; sync-target wording |
| `README.md` | Modify | Pinned/strategy examples |
| `runtime/Cargo.toml` | Modify | Version 0.24.0 |
| `runtime/CHANGELOG.md` | Modify | 0.24.0 no-behavior-change entry |

## Trade-offs

- **No read fallback (vs 042's indefinite runtime fallback).** Rejected adding one: no runtime primitive reads the constitution, and command bodies are rescaffolded in the same `/govern` run that moves the file, so readers and location cut over atomically. The residual risk — a pinned command body frozen on the old path — is handled by a named warning, not silent breakage.
- **Migration edits adopter-owned seed files.** Rewriting `CLAUDE.md`/`AGENTS.md`/`README.md` lines crosses the create-strategy ownership line, but 042 set the precedent (gitignore supersession), the rewrite matches only the exact shipped legacy forms, and the alternative — a warning telling the adopter to hand-edit three files — leaves every lagging adopter with a broken `@import`.
- **Divergent legacy copy is deleted, not kept.** Inherited from `govern-dir-consolidate`'s convergence rule: after cutover the root copy is dead weight; git recovers it, and the warning names it.
- **`.markdownlint-cli2.jsonc` stays at root.** Moving it under `.govern/` would demand `--config` plumbing in every lint invocation across all host paths for a purely aesthetic gain — out of scope, carved out in the spec.
- **A version cut with zero runtime code changes.** The cost of lockstep versioning; accepted per the 043 precedent so the registry entry's `introduced_in` references a published release.
