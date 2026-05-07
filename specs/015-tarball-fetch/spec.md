---
title: "015-tarball-fetch — spec"
status: done
dependencies: [007-govern-workflow, 012-multi-agent-govern]
tags: [bootstrap, performance]
---

# 015 — Tarball Fetch

> **Note:** the `framework/templates/project/initialize.md` reference below reflects the layout at the time. The repository was later reorganized so slash command stubs scaffolded at adoption live in `framework/templates/commands/` (currently just `initialize.md`); project document templates remain in `framework/templates/project/`. Adopting projects' destination paths did not change, and the manifest count is unaffected.

Collapse `/govern`'s ~35–50 individual `curl` fetches into a single archive download, extracted once into a temp directory and resolved as local paths. The manifest, strategies (`update`/`create`/`skip`/`merge`/`pinned`), and per-agent scaffolding flow are unchanged — only the **File Fetching** section's transport is replaced.

## Problem

Today `framework/bootstrap/govern.md` issues one `curl` per file in the manifest. A single-agent run touches:

- ~14 governance-owned shared files (constitution, rules, templates, registry)
- ~4 project-specific shared files (system, errors, events, inbox)
- 2–3 conditional shared files (AGENTS.md, CLAUDE.md, gitignore template)
- 1–N per-language gitignore patterns from `github.com/github/gitignore`
- ~16 slash command sources (15 in `framework/commands/` + 1 agent-specific configure)
- 1 `framework/templates/project/initialize.md`
- 1 `framework/bootstrap/govern.md` (self-install)
- 0–N workflow templates (only those the user accepts)

That's ~35 fetches for one agent, ~50+ for two agents on first run, and the same volume on every routine re-run because `update`-strategy files have to be fetched before the content-equality check can decide whether to write. The bottleneck is per-call tool-invocation overhead, not bytes — the entire framework directory is well under a megabyte.

A single archive fetch removes that overhead while keeping every other guarantee `/govern` makes today (idempotency, per-file strategies, pinning, failure granularity within the manifest pass).

## Behavior

### Source

`/govern` issues exactly one `curl` against GitHub's repo-archive endpoint:

```text
https://github.com/stonean/govern/archive/refs/heads/main.tar.gz
```

`curl -fsSL` already follows the 302 redirect to `codeload.github.com`. The archive's top-level directory is `govern-main/`; the framework files live at `govern-main/framework/...` after extraction.

External fetches that are **not** part of the governance repo are unchanged: per-language `.gitignore` patterns continue to come from `https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore` as separate `curl` calls. They are not in the archive, and bundling them is out of scope.

### Extract

After fetching the archive:

1. Create a **new** temp directory on every run: `mktemp -d -t govern-XXXXXX`. On macOS/Linux this lands under `$TMPDIR` or `/tmp`. Never reuse a directory from a prior run, even if one is still on disk — a fresh fetch is the only way `/govern` picks up upstream changes, so the archive must be re-downloaded each invocation.
2. Extract the archive into the temp directory: `tar -xzf {archive} -C {tempdir}`.
3. Compute the framework root: `{tempdir}/govern-main/`. Treat this as the local mirror of the governance repo for the rest of the run.

If the fetch or extraction fails — non-zero exit, missing `govern-main/` directory, or any required manifest entry absent from the extract — abort the run with a clear error:

> Failed to fetch or extract the governance archive ({reason}). Re-run after checking network connectivity, or report this if it persists.

Aborting on archive failure is intentional and a behavior change from the current per-file warning model: a missing archive means **every** file is missing, so there is nothing to scaffold partially. Per-file granularity within the extract is preserved (see **Per-file resolution** below).

### Per-file resolution

The manifest's source paths (e.g., `framework/constitution.md`, `framework/commands/specify.md`) are now resolved against the extracted framework root rather than concatenated with the raw URL prefix. For each manifest entry:

1. Compute the local source path: `{tempdir}/govern-main/{source-path}`.
2. If the local source path does not exist — the file was renamed, removed upstream, or the manifest is out of sync — warn `Source not found in archive: {source-path}; skipping.` and continue with the remaining entries. This preserves the current "do not abort on a single fetch error" guarantee.
3. Apply the existing strategy (`update`, `create`, `skip`, `merge`, `pinned`) using the local file as the new content. Content comparison for `update` strategy is a local file diff against the destination — same semantics as today, just no network round-trip.
4. Apply placeholder substitution after reading the local source, before writing to the destination. Same rules as today (including the `govern.md` self-install exception that keeps `{project}` and `{cli-config-dir}` literal).

### Cleanup

`/govern` does not delete the temp directory. The path is logged in the run summary (and, on abort, in the error message) so the user can inspect it if needed. Both macOS (`/var/folders/.../T/`) and Linux (`/tmp` on systemd-tmpfiles distros) sweep their temp directories automatically; a few hundred KB of extracted files waiting for the next sweep is acceptable in exchange for not granting an `rm -rf` permission to the bootstrap.

The leftover directory is for inspection only — the next `/govern` run creates its own fresh temp directory via `mktemp` and never reuses a prior extract.

### Permission bootstrap

The agent registry's `settings_template` currently allows `Bash(curl *)` and `Bash(ls *)` (Claude) and equivalent regex entries (Auggie). The tarball flow needs two additional commands plus, on Claude, pre-allowed `Read` globs that cover any `govern-*` temp directory. There is no `rm` addition — the temp directory is left for the OS to sweep (see **Cleanup** above).

Update each registry row's `settings_template`:

- **Claude:** add `Bash(tar *)`, `Bash(mktemp *)`, and six `Read(...)` globs covering both macOS temp roots and Linux `/tmp`, with both single-leading-slash and double-leading-slash forms (see "Why both leading-slash forms" below):
  - `Read(/private/var/folders/**/T/govern-*/**)` and `Read(//private/var/folders/**/T/govern-*/**)` (macOS canonical path)
  - `Read(/var/folders/**/T/govern-*/**)` and `Read(//var/folders/**/T/govern-*/**)` (macOS non-canonical, defensive)
  - `Read(/tmp/govern-*/**)` and `Read(//tmp/govern-*/**)` (Linux)
- **Auggie:** add equivalent `launch-process` entries with `shellInputRegex` patterns matching `tar` and `mktemp`. Auggie's `view` tool is broadly allowed by `configure/auggie.md`, so no per-path read entries are needed.

The merge logic in **Permission Setup** is unchanged — entries are added if missing, never reordered or deduplicated. Existing adopters get the new entries on their next routine `/govern` re-run, before any `tar`, `mktemp`, or extracted-archive read is performed.

#### Why pre-allow the temp-path Read globs

Claude Code's permission system records a `Read(...)` allow with the **exact absolute path** when the agent first reads a file outside the project root and the user accepts the prompt. Combined with the spec's mandate that every `/govern` run gets a fresh `mktemp` directory, that means: without pre-allowed globs, every run prompts once and writes a new `Read(...)` entry into `settings.local.json` for the run's unique path. Those entries accumulate forever and never match a future run.

Pre-allowing the globs at bootstrap solves this in a small fixed set of entries — future Read calls against any `govern-*` temp path match the glob and skip the prompt. The entries are scoped to `govern-*` directories under known temp roots, so they cannot grant Read on unrelated files.

#### Why both leading-slash forms

Empirically, Claude Code's permission rule matcher treats `/private/...` and `//private/...` as **different** prefixes — the matcher is literal, not normalized. Some agent code paths invoke `Read` with a double-leading-slash path (POSIX-permitted, macOS-equivalent to a single slash), and the resulting permission prompt records the path with the double slash preserved. A single-slash glob does not match a double-slash request, so the prompt fires anyway and a per-path entry is added. Including both forms in the bootstrap is the simplest way to cover the observed variation without depending on which path-normalization branch the agent takes on a given run.

#### One-time cleanup of stale per-run entries

Adopters who ran the tarball flow before this fix shipped will already have several `Read(/private/var/folders/.../T/govern-XXXXXX.{suffix}/...)` entries in their `settings.local.json`. The bootstrap merge does not delete them (per the spec's "never reorder or deduplicate" rule). Adopters can remove the stale entries manually — they are inert (the new glob covers any future case) but cosmetically noisy.

### Self-update notice

The integrity check that re-fetches `govern.md` on a corrupted write currently re-runs `curl`. With a tarball, re-fetch means reading the same file again from the extracted archive — no second network call. The check itself is unchanged; it just operates on the local source.

The self-update notice (shown when the installed `govern.md` differs from the fetched version) continues to fire identically: the comparison is between the destination file and the local source file from the archive.

## Tradeoffs

- **One archive failure vs. many per-file warnings.** Today a single 404 on `framework/templates/project/inbox.md` produces a warning and the rest of the manifest proceeds. With a tarball, that file would simply not exist in the extract, producing the same per-entry warning. The genuine new failure mode is the archive itself failing — and that's a clean abort, since partial scaffolding from a missing archive is impossible.
- **New permissions.** For Claude, two `Bash` additions (`tar`, `mktemp`) plus six `Read(...)` globs covering `govern-*` temp paths under macOS (`/private/var/folders/**/T/`, `/var/folders/**/T/`) and Linux (`/tmp/`), with both single-leading-slash and double-leading-slash forms because Claude Code's permission matcher treats them as distinct prefixes (see **Permission bootstrap → Why both leading-slash forms**). For Auggie, just the two shell-command entries — `view` is unconditionally allowed by configure. Cost is one-time per adopter, applied on the same `/govern` run that introduces the change. No `rm` allow is needed because the temp directory is left for the OS to sweep.
- **Bytes over the wire.** The full archive is ~hundreds of KB compressed; today's per-file fetches collectively pull a similar volume but spread across ~35 round-trips. Net: fewer bytes once HTTP overhead is counted, fewer tool-call invocations, faster perceived run time.
- **Loss of partial progress.** A network failure mid-fetch today produces ~10 successful files plus warnings on the rest; with a tarball, a network failure produces zero files and a clean abort. Both outcomes leave the project in a recoverable state — re-run resumes idempotently.
- **Pinning at a ref.** Today fetches are hardcoded to `main`. The tarball URL also points at `main`. A `.governance.toml` `[source] ref = "v0.1.0"` option that overrides the archive ref is a natural follow-up but out of scope for this spec — see **Resolved Questions**.

## Acceptance Criteria

- [x] `framework/bootstrap/govern.md`'s **File Fetching** section is replaced with the archive-fetch + extract + local-path-resolution flow above
- [x] A successful `/govern` run on a single-agent project issues exactly one `curl` against the governance repo (plus per-language gitignore fetches, which remain unchanged)
- [x] All existing manifest strategies (`update`, `create`, `skip`, `merge`, `pinned`) behave identically to today, sourcing files from the extracted archive
- [x] A failed archive fetch produces a clean abort with a clear error message and no partial scaffolding
- [x] A missing source file within the archive produces a per-entry warning and the remaining manifest continues
- [x] The temp directory path is logged in the run summary (and on abort, in the error message); `/govern` does not delete it
- [x] Each agent's `settings_template` in the registry adds `tar` and `mktemp` allow entries, applied via the existing **Permission Setup** merge; no `rm` allow is added
- [x] The Claude `settings_template` also adds `Read(...)` globs for `govern-*` temp paths under both macOS temp roots and Linux `/tmp`, so per-run `Read(...)` entries do not accumulate across `/govern` invocations
- [x] The **Post-Write Integrity Check** for `govern.md` works against the local source — no additional `curl`
- [x] The self-update notice continues to fire when the installed `govern.md` differs from the archive's copy
- [x] Per-language gitignore fetches against `github.com/github/gitignore` remain unchanged (separate `curl` calls)
- [x] A re-run on an already-adopted project still reports `unchanged` for files whose archive copy matches the destination
- [x] Each `/govern` invocation creates a fresh temp directory via `mktemp` and re-fetches the archive; a prior run's extracted directory is never reused as the source for the current run

## Open Questions

*None — all resolved.*

## Resolved Questions

- **`.governance.toml` ref pinning** — defer. `.governance.toml` already supports additive sections (currently `[pinned]`), so a `[source] ref = "..."` option can be added in a later spec without migration cost. No adopter has asked for ref pinning, and `main` parity with the current per-file flow keeps this spec's scope to the transport change. v0.1.0 is tagged, but until there is concrete demand, every adopter would still pin to `main` — adding the surface now means documenting and maintaining a feature nobody is using.
- **Fallback to per-file fetch on archive failure** — no fallback. `codeload.github.com` and `raw.githubusercontent.com` are both GitHub-fronted CDNs with correlated availability, so partial outages affecting only one are rare. Maintaining two transport code paths is a permanent tax for a transient failure mode; `/govern` is idempotent and a re-run after the outage clears is the same recovery path used for any transient `curl` failure today. The clean abort already tells the user what happened.
- **`rm` permission scope** — drop the `rm` permission entirely; let the OS sweep the temp directory. macOS (`/var/folders/.../T/`) and Linux (`/tmp` via systemd-tmpfiles) both auto-purge their temp roots, so a few hundred KB of extracted files waiting for the next sweep is acceptable. Granting `rm -rf` — even scoped — is a sharper allow than the rest of govern's bootstrap permissions (`curl`, `ls`, `tar`, `mktemp` are all read-or-create) and adopters with custom `$TMPDIR` would have to edit settings to keep cleanup working. Skipping `rm` removes that friction.
- **Auggie permission regex** — moot. Without an `rm` permission to express, there is no Auggie-regex problem to solve.

## References

Declared dependencies for this spec, surfaced here so the dependency-derivation generator (`scripts/gen-spec-deps.sh`) sees them in the body.

- [007-govern-workflow](../007-govern-workflow/spec.md)
- [012-multi-agent-govern](../012-multi-agent-govern/spec.md)
