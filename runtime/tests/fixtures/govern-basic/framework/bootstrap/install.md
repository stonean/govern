---
description: Fixture-only bootstrap procedure — exercises the four govern-bootstrap primitives end-to-end.
parity:
  strict-files:
    - "project/scripts/gen-noop.sh"
    - "project/framework/commands/specify.md"
    - "project/CLAUDE.md"
  semantic-fields:
    - completion-message
---

# /install (govern-basic fixture)

A minimal stand-in for the production `framework/bootstrap/govern.md`
procedure. The fixture's parity test stages this file under
`framework/bootstrap/install.md`, sets context keys for the four
primitives, runs `gvrn exec install`, and asserts the resulting
on-disk state matches a golden plus the strict-files bound above.

## Instructions

1. The walker context carries url, sha256-url, archive, dest, source-dir, target-dir, substitutions, path, and block — all pre-populated by the fixture's session JSON. The mock-HTTP server in the parity harness serves the archive and sidecar; the host substitutes its dynamic URL into the session before launch.

2. Invoke `fetch-archive` to download the framework tarball and verify its sha256 sidecar. Otherwise, follow the markdown-only path: `curl -LO` the tarball and `shasum -a 256 -c` the sidecar manually.

3. Invoke `extract-archive` to expand the verified tarball into the staging directory; path-traversal protection is applied per entry. Otherwise, fall back to `tar -xzf`.

4. Ask the user to approve writing the framework files into the project before any destination-tree changes.

5. Invoke `substitute-templates` to walk the staging tree, apply `{project}` substitutions, and write the result into the project directory.

6. Invoke `merge-claude-md` to install the framework-managed block in CLAUDE.md.

7. Render the completion message (host responsibility): list the framework files installed and the next pipeline step.
