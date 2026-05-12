---
section: "Follow-on scenarios"
---

# Govern bootstrap

## Context

The `/govern` bootstrap installer at `framework/bootstrap/govern.md` does almost no LLM-judgment work: it resolves the framework source (URL + version), fetches a tarball, verifies its checksum, untars into a staging directory, walks the staged tree applying include/exclude globs, substitutes template variables (`{project}`, `{cli-config-dir}`, etc.) into each file, writes the result to the adopter's project, and optionally merges into `CLAUDE.md`. Every step is deterministic file plumbing. By the spec's own §runtime-boundary principles, this is exactly the kind of procedure the runtime should accelerate — and yet the v0.1.0 runtime can't run it.

Three things kept `/govern` out of the v0.1.0 initial-release scope:

1. **Command-resolution surface.** `gvrn exec <name>` looks for `framework/commands/<name>.md` or `.claude/commands/gov/<name>.md`. The bootstrap doc lives at `framework/bootstrap/govern.md` — the one file that ships outside the project-installable namespace because it's invoked in projects that don't yet have `govern`.
2. **Missing primitives.** The 14 primitives in v0.1.0 are spec/task/git-shaped (`read-spec`, `mark-task`, `derive-boundary`, etc.). The bootstrap needs file-fetch, archive-extract, template-substitute, and CLAUDE.md-merge primitives that don't exist.
3. **Bootstrap paradox.** The first invocation of `/govern` runs in a project that doesn't yet have `govern` files. The runtime may or may not be installed at that moment. The §runtime-boundary opt-in invariant requires the markdown-only path to keep working unaffected.

Item 3 is soluble: adopters who install `gvrn` separately (via `cargo install gvrn` or the release tarball) before running `/govern` get the acceleration; adopters who don't get the existing markdown-only walk. That's the same fallback contract every other runtime-aware command honors.

## Behavior

This scenario extends the runtime to cover the `/govern` bootstrap procedure end-to-end. Concretely:

1. **New primitives ship in `gvrn`**, each a thin pure-Rust function with a `clap` args struct and an MCP tool name:
   - `fetch-archive` — download a URL to a tempfile and verify a sha256 sidecar; returns the local tempfile path.
   - `extract-archive` — untar/unzip a local archive into a staging directory; returns the staging path and a list of extracted files.
   - `substitute-templates` — walk a tree, read each text file, apply a map of `{key}` → value substitutions, write the result to a target tree; returns the count of files written and the count of substitutions applied.
   - `merge-claude-md` — idempotently merge a generated block into an adopter's `CLAUDE.md`, distinguishing first-run (file absent) from update mode (block already present), preserving the user's surrounding edits.
2. **Command-resolution surface extends to bootstrap procedures.** `gvrn exec govern` resolves to `framework/bootstrap/govern.md` after the existing two candidate paths fail. The bootstrap file gains a `## Instructions` section under the parseable conventions; the existing prose moves into a `## Markdown-only reference` section the same way the other six rewrites did.
3. **Frontmatter gains a `parity:` field** for the bootstrap. The natural bound is `strict-files` on the post-run project tree (every adopter-visible file written by `/govern` is byte-equal between the runtime-driven and LLM-driven paths) plus `semantic-fields` for any human-readable completion-message text that varies between runs.
4. **Fixture `runtime/tests/fixtures/govern-basic/`** ships a minimal adopter-project skeleton plus a tarball-shaped staging asset, and a golden JSONL stream capturing the expected envelope sequence. The parity test stages the fixture, runs `gvrn exec govern`, and asserts file-tree equivalence against the expected post-run tree.
5. **Bootstrap completion message** continues to advertise the optional runtime as it does today (per task 22 of the parent spec) — no detect-and-warn elsewhere.

The markdown-only path remains unchanged for adopters without `gvrn` installed: the LLM walks the same prose, performs the same fetch/extract/substitute steps, and produces the same output tree. The §runtime-boundary invariant holds end-to-end.

## Edge Cases

- **Adopter installed `gvrn` after `/govern` started.** Not supported; the runtime is detected at invocation, not mid-procedure. Adopters re-run `/govern` to pick up the acceleration on subsequent passes.
- **First-run vs update-mode CLAUDE.md merge.** The `merge-claude-md` primitive must distinguish the two cases idempotently: appending the framework block on first run, replacing the existing block on update without disturbing the adopter's surrounding edits. Primitive returns whether it inserted, updated, or no-oped.
- **Partial extraction failure.** If `extract-archive` errors mid-extract, the staging tempdir is left behind for inspection; the primitive surfaces the partial state via the `error` envelope (operational failure, not domain finding). The adopter's target tree is never written to with partial state — substitution and write are a separate step on a fully-extracted staging tree.
- **Template substitution with literal `{project}` content.** Adopters may legitimately have `{project}` as literal text in their source files (e.g., docs that describe templating). The primitive operates only on the staging tree the framework just extracted, never the adopter's pre-existing files, so the collision is bounded to framework content — which never contains literal `{project}` outside template variables.
- **Cross-platform tar/zip handling.** The release ships `gvrn-<TARGET>.tar.gz` on Unix and `.zip` on Windows. `extract-archive` chooses the format by extension; the primitive returns an operational error for unknown extensions rather than guessing.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

- **Why not split this into its own spec?** The bootstrap is one usage scenario of the runtime, not a separate architectural concept. It elaborates the parent spec's "Follow-on scenarios" section rather than introducing a new spec lineage.
- **Move `framework/bootstrap/govern.md` into `framework/commands/`?** No. The bootstrap file's location is meaningful — it's the one entry point that lives outside the project-installable namespace. The runtime extends its command-resolution surface to also look at `framework/bootstrap/<name>.md` instead of forcing a file move that would break every existing bootstrap integration.
