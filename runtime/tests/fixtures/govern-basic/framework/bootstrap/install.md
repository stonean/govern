---
description: Fixture-only bootstrap procedure — exercises the seven govern-bootstrap primitives end-to-end.
parity:
  strict-files:
    - "project/framework/commands/specify.md"
    - "project/framework/templates/feature.md"
    - "project/AGENTS.md"
    - "project/.gitignore"
    - "project/.claude/commands/govern.md"
  semantic-fields:
    - completion-message
---

# /install (govern-basic fixture)

A minimal stand-in for the production `framework/bootstrap/govern.md`
procedure. The fixture's parity test stages this file under
`framework/bootstrap/install.md`, sets context keys for the six
primitives in the session JSON, runs `gvrn exec install`, and asserts
the resulting on-disk state matches the golden plus the strict-files
bound above.

The procedure exercises every primitive shape introduced by the
apply-manifest scenario:

- `fetch-archive` / `extract-archive` (back half of the previous
  govern-bootstrap scenario, unchanged).
- `apply-manifest` with five entries that cover every strategy
  (update / create / skip-if-conflict), a pinned dest path, and a
  keep-literals govern.md self-install in the entries list.
- `merge-managed-block` against `.gitignore` with `marker-style:
  "line-prefix"`.
- `enforce-manifest` against the per-agent slash-command directory,
  pruning a pre-seeded legacy file from the project.

The two apply-manifest calls in the production procedure are
collapsed into one here because the walker passes a single context
map to every primitive — splitting them in a fixture would dispatch
twice with identical inputs (the second call observing all
`unchanged` actions). The unit-test layer covers the split-call
semantics directly.

## Instructions

1. The walker context carries every primitive's args: url and sha256-url for the mock-HTTP fetch, archive and dest for the local extract, source-root / target-root / entries / pinned / substitutions for the bulk apply-manifest, path / block / marker / marker-style for the .gitignore merge, and directory / expected for the slash-command cleanup.

2. Invoke `fetch-archive` to download the tarball and verify its sha256 sidecar against the mock-HTTP server. Otherwise, follow the markdown-only path: `curl -LO` the tarball and `shasum -a 256 -c` manually.

3. Invoke `extract-archive` to expand the verified tarball into the staging directory. Otherwise, fall back to `tar -xzf`.

4. Invoke `apply-manifest` to walk the entries list and write the framework files into the project tree. The host's manifest covers every strategy (update / create / skip-if-conflict), the pinned-list short-circuit, and a keep-literals entry for the `.claude/commands/govern.md` self-install.

5. Invoke `merge-managed-block` against `.gitignore` with `marker-style: "line-prefix"` to install or update the framework-managed block. First-run creates the file; subsequent runs update only the region between the `# govern` preamble and the next blank line.

6. Invoke `enforce-manifest` against the slash-command directory to remove any pre-seeded legacy files that aren't in the expected manifest. The fixture pre-seeds `project/framework/commands/legacy-cmd.md`; the enforce step prunes it.

7. Render the completion message (host responsibility): list the framework files installed and the next pipeline step.
