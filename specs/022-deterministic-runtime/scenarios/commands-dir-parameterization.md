---
section: "Follow-on scenarios"
---

# Commands-dir parameterization

## Context

The runtime resolves command files at two callsites that both hardcode `.claude/commands/gov/`:

- `runtime/src/main.rs:209-220` — `run_exec`'s candidate list for `gvrn exec <command>`. The middle entry is `repo.join(".claude/commands/gov").join(format!("{command}.md"))`.
- `runtime/src/interpreter/payload.rs:378-390` — `locate_command_file`, used during anchor extraction (resolving `Reference: §<anchor>` lines under the `## Scope Boundaries` section of a command file). Same middle-entry shape.

Both paths bake in two assumptions that hold in this repo but break for adopters:

1. **The host's config dir is `.claude/`.** True for Claude Code, false for Auggie (`.augment/`) and any future host. The constitution already establishes `{cli-config-dir}` as a template variable for exactly this reason (the configure-permission flow uses it), but the runtime's lookup table doesn't honor that variable.
2. **The project's slash-command namespace is `gov/`.** True for this repo (the framework's own dogfooded slash commands live under `/gov:*`), false for any adopter (`/anvil:*` lives under `.claude/commands/anvil/`, `/bark:*` under `.claude/commands/bark/`, etc.). The constitution establishes `{project}` as the namespace variable.

In this repo the bug is invisible because `framework/commands/<name>.md` (the first candidate) always wins the search — the source files are sitting right there. In an adopter project that has only run `/govern` (so they have `.claude/commands/<project>/*.md` but no `framework/commands/` tree), the runtime never finds the command file and `gvrn exec` errors out.

Surfaced 2026-05-24 during the `.govern.session.toml` consolidation sweep on spec 022. Left out of scope at the time because the design space is non-obvious — multiple shapes are viable (env var, `.govern.toml` config block, CLI flag, runtime config file written by `/govern` at bootstrap) — and the bug doesn't bite in the framework repo itself.

## Behavior

The runtime needs to know two values at command-resolution time: the host's config-dir name (`{cli-config-dir}`) and the project's slash-command namespace (`{project}`). The same two values are already substituted into generated command files at install time, so the question is where the *runtime* reads them from at exec time.

### Design picks to evaluate

This scenario is the place to choose the source-of-truth shape during implementation. Three options, ordered by recommended preference:

**Option 1 — `.govern.toml` config block (recommended).** The project root already carries `.govern.toml` (per spec 022.40 the session file consolidated onto `.govern.session.toml`, but `.govern.toml` proper still owns project-level config like `pins`). Add a top-level block:

```toml
[host]
cli-config-dir = ".claude"
project = "gov"
```

`/govern`'s bootstrap writes these values at install time (it already knows both — it substitutes them into every generated file). The runtime reads them at exec time via a small loader. Adopters who hand-roll their `.govern.toml` get a clear, version-controlled config surface. Missing values fall back to `.claude` / the directory name of the repo, preserving today's behavior in the source-of-truth repo.

**Option 2 — environment variables.** `GVRN_CLI_CONFIG_DIR` and `GVRN_PROJECT`, honored by both callsites. Adopters set them in their shell profile or CI env. Lightest-touch implementation but invisible at the repo level — onboarding a new contributor requires knowing to set them, which violates "everything the runtime needs is in the repo."

**Option 3 — CLI flag (`gvrn --cli-config-dir=.augment --project=anvil exec ...`).** Maximally explicit but forces every caller (including `/gov:*` slash command bodies) to pass both flags on every invocation. Heaviest ergonomic cost; rejected.

### Implementation shape (assuming Option 1)

1. Add `Host { cli_config_dir: String, project: String }` to the runtime's config loader. Default values: `cli_config_dir = ".claude"`, `project` = the repo's directory basename.
2. Load `.govern.toml` once at process start (the loader already reads pins from the same file). Surface the `Host` struct through whatever context object both callsites have access to — likely a new field on `Walker` or a parallel argument threaded into `run_exec` and `locate_command_file`.
3. Replace the hardcoded path strings in both callsites with `repo.join(format!("{}/commands/{}/{}.md", host.cli_config_dir, host.project, command_name))`. The interior segment becomes `{cli_config_dir}/commands/{project}/`.
4. `/govern`'s bootstrap (`framework/bootstrap/govern.md`) writes the `[host]` block into the adopter's `.govern.toml` during the install. The block is idempotent — re-runs update existing values rather than appending duplicate sections. Existing `.govern.toml` files in adopter projects (created before this scenario lands) gain the block on their next `/govern` run.
5. Fixtures under `runtime/tests/fixtures/` that exercise `gvrn exec` and anchor resolution gain a `.govern.toml` with explicit `[host]` values so the parity tests cover both Claude (`.claude`/`gov`) and Auggie (`.augment`/`anvil`) shapes. The Auggie fixture is the regression test for this scenario.

### Markdown-only path

The runtime's parity contract requires the markdown-only fallback to keep working. The hardcoded path is a runtime-only concern — the markdown-only walk reads command files via the LLM, which already follows the natural file-system layout of the adopter's project. No change required on the markdown-only side.

## Edge Cases

- **`.govern.toml` missing or `[host]` block absent.** Loader returns defaults (`.claude` for the config dir, repo directory basename for the project). This preserves today's behavior in the framework's own repo where neither value has ever been set.
- **`[host]` values disagree with the actual on-disk layout.** The lookup misses and the existing "command file not found" error fires, listing every candidate path tried (so the user can see the mismatch). No silent fallback to legacy hardcoded paths — fail loud.
- **An adopter migrates from one host to another (Claude → Auggie or vice versa).** They edit `.govern.toml`'s `[host]` block; both callsites pick up the new values on next exec. No re-bootstrap required for the runtime to follow the change. The slash command files themselves still need to be re-emitted by `/govern` for the new host's config dir, but that's an existing concern handled by `/govern` re-runs.
- **Project name contains characters that aren't filesystem-safe.** Out of scope here — `{project}` is already constrained by the bootstrap to lowercase ASCII per the existing template-substitution contract. The runtime can trust it.
- **The framework's source repo (`stonean/govern`) has no `[host]` block.** Defaults pin `cli_config_dir = ".claude"` and `project = "govern"` (the repo directory name). The first candidate (`framework/commands/<name>.md`) keeps winning every lookup, so the default-fallback behavior never gets exercised here in practice — but the path stays correct for any code reading `.claude/commands/govern/` if that ever happens to be the on-disk shape.

## Open Questions

*None — captured during scenario authoring; design picks surfaced as options for the implementation choice.*

## Resolved Questions

**Source-of-truth shape — `.govern.toml` `[host]` block (Option 1).** Resolved 2026-05-24 during task 41 implementation. The runtime loads `Host { cli_config_dir, project }` from a `[host]` block in `.govern.toml`, defaulting to `.claude` / the repo directory basename when the block is missing. `/govern`'s bootstrap writes the block idempotently. Option 2 (env vars) was rejected as invisible at the repo level; Option 3 (per-invocation CLI flags) was rejected for ergonomic cost.

> **Superseded in part for `cli-config-dir`.** `project` still loads from `.govern.toml` `[host]` as above, but `cli_config_dir` is per-contributor (teammates may use different agents) and was relocated to the gitignored `.govern.session.toml` — read with a legacy `.govern.toml` `[host]` fallback. See [cli-config-dir-per-contributor](cli-config-dir-per-contributor.md).
