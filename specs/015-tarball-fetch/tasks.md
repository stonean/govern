# 015 — Tarball Fetch Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Add `tar` and `mktemp` to the Agent Registry settings_template values

- [x] In `framework/bootstrap/govern.md`, edit the Claude row's `settings_template`: append `"Bash(tar *)"` and `"Bash(mktemp *)"` after the existing `"Bash(ls *)"` in the `permissions.allow` array.
- [x] Edit the Auggie row's `settings_template`: append two `launch-process` entries with `"shellInputRegex": "^tar "` and `"shellInputRegex": "^mktemp "` after the existing `"^ls "` entry.
- [x] Verify both rows still parse as inline JSON (no markdown table-cell escaping issues).
- [x] Done when: both rows include the two new entries in `allow`/`toolPermissions` form, and the surrounding table renders correctly in markdown preview.

## 2. Replace the File Fetching section in govern.md

- [x] In `framework/bootstrap/govern.md`, replace the entire **File Fetching** section (lines ~192–202 in the current source) with the archive-fetch + extract + per-file resolution flow described in the spec's **Source**, **Extract**, **Per-file resolution**, and **Cleanup** subsections.
- [x] The replacement must keep the same section heading (`## File Fetching`) so anchor links from the rest of govern.md (and any external references) continue to resolve.
- [x] State the abort message verbatim from the spec, including the `{reason}` placeholder convention.
- [x] State the per-entry warning verbatim: `Source not found in archive: {source-path}; skipping.`
- [x] Note that per-language gitignore fetches against `github.com/github/gitignore` are unchanged and remain separate `curl` calls (the `.gitignore` subsection of **Shared Files** describes them; cross-reference is sufficient).
- [x] Done when: the section is rewritten, the manifest tables in **Shared Files** and **Per-Agent Scaffolding** are unchanged, and a re-read of govern.md from the top still flows naturally into the new section.

## 3. Update the Post-Write Integrity Check wording [simple]

- [x] In `framework/bootstrap/govern.md`'s **Post-Write Integrity Check** section, change "report the error and re-fetch the file" to "report the error and re-read the file from the extracted archive."
- [x] Done when: no remaining wording in the file implies that the integrity-check re-read is a second network call.

## 4. Rewrite the Edge Cases bullet for fetch failures

- [x] In `framework/bootstrap/govern.md`'s **Edge Cases** section, replace the `Curl fails on a single file in the manifest` bullet with two bullets covering: (a) archive fetch/extract failure → clean abort; (b) a required source file absent from the extracted archive → per-entry warning, continue.
- [x] Done when: the Edge Cases section accurately describes both failure modes and no longer mentions per-file curl failures (which are no longer how the manifest pass fails).

## 5. Smoke-test `/govern` end-to-end against a throwaway repo

- [ ] In a scratch directory, `git init` a fresh repo and run the updated `/govern` flow (locally, by reading govern.md's instructions from this checkout — there is no automated test harness for govern.md).
- [ ] Confirm: exactly one `curl` against `github.com/stonean/govern/archive/...`; per-language gitignore fetches still work; `settings.local.json` gains `Bash(tar *)` and `Bash(mktemp *)`; all manifest entries land in their expected destinations; the run summary logs the temp directory path; the temp directory is left in place after the run.
- [ ] Re-run `/govern` against the same repo. Confirm idempotency: `update`-strategy files report `unchanged`; `create`-strategy files are skipped; settings entries are not duplicated.
- [ ] Force an archive failure (e.g., point the URL at a non-existent ref by temporarily editing govern.md, then run). Confirm clean abort with the spec's error message and no partial scaffolding.
- [ ] Done when: all three runs produce the expected behavior and govern.md is restored to the correct ref.
