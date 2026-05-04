---
title: "015-tarball-fetch — plan"
---

# 015 — Tarball Fetch Plan

Implements [015 — Tarball Fetch](spec.md).

## Overview

The change is localized to a single source file: `framework/bootstrap/govern.md`. Three sections need edits — the **Agent Registry** table (two `settings_template` values), the **File Fetching** section (replaced wholesale), and the **Post-Write Integrity Check** plus one **Edge Cases** bullet (wording reconciled with the new transport). No new files; no template changes; no scaffolding-manifest changes; no `configure.md` changes.

The generator (`scripts/gen-claude-commands.sh`) does not regenerate `govern.md` — `govern.md` is hand-maintained per the project's CLAUDE.md ("`framework/bootstrap/govern.md` … the framework is the everything-that-ships portion"). Edits land directly in the source.

## Technical Decisions

### Fetch and extract sequencing

`mktemp -d -t govern-XXXXXX` lands under `$TMPDIR` on macOS and `/tmp` on Linux without hardcoding either. The archive is fetched with `curl -fsSL -o {tempdir}/govern.tar.gz {archive-url}` and extracted with `tar -xzf {tempdir}/govern.tar.gz -C {tempdir}`. Failure is detected by checking `$?` after each command, then verifying `{tempdir}/govern-main/` exists. Any failure of the three (curl, tar, missing dir) triggers the same abort message the spec mandates.

The download-then-extract sequence is deliberate over piping (`curl ... | tar -xzf -`):

- Pipes obscure which command failed — needed for the abort message's `{reason}` field.
- The archive on disk lets the **Post-Write Integrity Check** re-read `govern.md` cheaply if a corrupted write is detected (no second `curl`).
- Disk pressure is trivial (the archive plus the extract is <1 MB).

### Manifest source resolution

Existing manifest tables in govern.md use source paths like `framework/constitution.md`. The replacement **File Fetching** section says: "for each manifest entry, the local source path is `{tempdir}/govern-main/{source-path}`." This requires zero changes to the manifest tables themselves — they continue to list source paths exactly as today, and only the resolution rule changes.

The `framework/bootstrap/govern.md` self-install row resolves the same way: source path is `framework/bootstrap/govern.md`, local path is `{tempdir}/govern-main/framework/bootstrap/govern.md`. The placeholder-substitution exception (keep `{project}` and `{cli-config-dir}` literal in this file) is unchanged.

### Per-language gitignore fetches stay separate

The **.gitignore** subsection of **Shared Files with conflict handling** fetches per-language patterns from `https://raw.githubusercontent.com/github/gitignore/main/{Language}.gitignore`. Those URLs are not in the governance repo and not in the archive. The replacement **File Fetching** section is silent about them (the .gitignore subsection's own text describes its fetch); the new section explicitly notes only governance-repo files are sourced from the archive. No code change to .gitignore handling.

### Agent registry edits

The two `settings_template` values in the registry table are inline JSON. Edits:

- **Claude:** `{ "permissions": { "allow": ["Bash(curl *)", "Bash(ls *)", "Bash(tar *)", "Bash(mktemp *)"], "deny": [] } }` — append two array entries after `Bash(ls *)`.
- **Auggie:** append two `launch-process` entries with `"shellInputRegex": "^tar "` and `"shellInputRegex": "^mktemp "` after the existing `"^ls "` entry.

Order matters for readability but not behavior — the bootstrap merge (govern.md **Permission Setup** step 2) preserves order on both sides.

### Configure surface stays unchanged

`framework/bootstrap/configure/claude.md` already denies `Bash(rm -rf *)`, `Bash(*rm -rf *)`, and four other `rm` patterns. Adding a scoped `rm` allow to the bootstrap registry would be overridden by these denies as soon as `/{project}:configure` runs, independently confirming the spec's "let the OS sweep" decision. No edits to `configure/claude.md` or `configure/auggie.md` are required — they describe the post-adoption full permission set, not bootstrap permissions, and `tar`/`mktemp` are not used by any other slash command.

### Edge Cases reconciliation

The current govern.md has an Edge Case bullet: "Curl fails on a single file in the manifest — report the failure and continue with remaining files. Do not abort the entire scaffolding pass." After this change the bullet has two distinct cases:

- **Archive fetch or extract fails** — clean abort, no partial scaffolding (new behavior).
- **A required source file is absent from the extracted archive** — per-entry warning, continue (preserves the "do not abort on a single fetch error" guarantee, just sourced locally).

The bullet is rewritten to cover both.

### Post-Write Integrity Check wording

Current text: "If it does not, the write was corrupted — report the error and re-fetch the file." With the tarball, "re-fetch" is a re-read from the extracted archive — no second `curl`. Wording becomes: "report the error and re-read the file from the extracted archive."

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/bootstrap/govern.md` | Modify | Update **Agent Registry** `settings_template` values (Claude, Auggie); replace **File Fetching** section with archive-fetch + extract + per-file resolution; update **Post-Write Integrity Check** wording; rewrite the **Edge Cases** bullet for fetch failures. |

No other files change. The generator (`scripts/gen-claude-commands.sh`) does not need to run because govern.md is not in its input set.

## Open Questions Resolved

All four open questions resolved during `/gov:clarify`. See the spec's **Resolved Questions** section for the full record. Summary:

- **Ref pinning** — defer to a later spec.
- **Per-file fallback** — none; archive failure aborts cleanly.
- **`rm` permission scope** — drop `rm` entirely; the OS sweeps the temp directory.
- **Auggie regex for `rm`** — moot, given no `rm` permission.

## Trade-offs

- **Disk vs. memory for the archive** — the download-then-extract approach uses ~1 MB of temp disk. Piping `curl | tar` would skip the disk write but obscure which side failed (needed for the abort message) and force a second `curl` for the integrity-check re-read. The disk cost is negligible; the diagnostic clarity and re-read efficiency are not.
- **Archive ref hardcoded to `main`** — same as the current per-file flow; ref pinning is deferred to a later spec (see Resolved Questions in the spec).
- **One-time bootstrap cost** — existing adopters get the new `tar`/`mktemp` allow entries on their next routine `/govern` re-run. The merge logic is additive and idempotent, so no migration step is needed.
