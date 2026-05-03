# 015 — Tarball Fetch Code Locations

## AC: `framework/bootstrap/govern.md`'s **File Fetching** section is replaced with the archive-fetch + extract + local-path-resolution flow above

- `framework/bootstrap/govern.md`

## AC: A successful `/govern` run on a single-agent project issues exactly one `curl` against the governance repo (plus per-language gitignore fetches, which remain unchanged)

- `framework/bootstrap/govern.md`

## AC: All existing manifest strategies (`update`, `create`, `skip`, `merge`, `pinned`) behave identically to today, sourcing files from the extracted archive

- `framework/bootstrap/govern.md`

## AC: A failed archive fetch produces a clean abort with a clear error message and no partial scaffolding

- `framework/bootstrap/govern.md`

## AC: A missing source file within the archive produces a per-entry warning and the remaining manifest continues

- `framework/bootstrap/govern.md`

## AC: The temp directory path is logged in the run summary (and on abort, in the error message); `/govern` does not delete it

- `framework/bootstrap/govern.md`

## AC: Each agent's `settings_template` in the registry adds `tar` and `mktemp` allow entries, applied via the existing **Permission Setup** merge; no `rm` allow is added

- `framework/bootstrap/govern.md`

## AC: The **Post-Write Integrity Check** for `govern.md` works against the local source — no additional `curl`

- `framework/bootstrap/govern.md`

## AC: Per-language gitignore fetches against `github.com/github/gitignore` remain unchanged (separate `curl` calls)

- `framework/bootstrap/govern.md`

## AC: A re-run on an already-adopted project still reports `unchanged` for files whose archive copy matches the destination

- `framework/bootstrap/govern.md`
