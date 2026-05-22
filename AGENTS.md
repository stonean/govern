# `govern`

The `govern` framework — a pipeline-driven spec-to-implementation flow scaffolded into adopted projects.

> **Agents:** this file is the committed home for project rules — append durable learnings to the matching section (Gotchas, Workflow, Boundaries, Code Style, Testing). Add a new section only when none fits.

## Constitution

See [constitution.md](constitution.md) — guiding principles, development pipeline, spec lifecycle, and quality standards that govern this project.

## Project Structure

The `framework/` directory is govern's source — everything that ships to adopted projects via `/govern`. It is laid out by IA primary purpose, not by file kind:

- `framework/constitution.md` — the constitution shipped to adopted projects (sync target of root `constitution.md`)
- `framework/rules/` — domain rule sets adopted projects can reference (security-backend, security-frontend, …)
- `framework/templates/spec/` — templates consumed by an agent during the pipeline (spec, plan, tasks, data-model, research, scenario)
- `framework/templates/project/` — project document templates consumed once at adoption (agents, claude-md, system, errors, events, project-readme, gitignore, inbox)
- `framework/commands/` — operational slash command sources only
- `framework/workflows/` — tech-stack-specific workflow files (lint, test, format) plus `registry.json` mapping stack selections to workflows
- `framework/bootstrap/` — the `govern.md` installer plus per-agent permission files at `bootstrap/configure/{key}.md`

When adding files under `framework/`, place them by purpose, not by extension.

## Tech Stack

`govern` is a text-first framework. The implementation is intentionally narrow:

- **Markdown** — every primary artifact (constitution, specs, plans, tasks, scenarios, rules, slash command sources, templates). Source of truth per [§text-first-artifacts](framework/constitution.md#text-first-artifacts).
- **Bash scripts** under `scripts/` — generators (`gen-*.sh`), the pre-commit hook installer (`install-hooks.sh`), and lints (`lint-*.sh`). All deterministic; no application logic.
- **GitHub Actions YAML** under `.github/workflows/` — CI configuration only.
- **Node.js / `npx`** — invoked at lint time for `markdownlint-cli2` only; not a build dependency, no `package.json` or `node_modules`.

There is no compiled language, no application runtime, no database, no service binary. An optional deterministic runtime is permitted under [§runtime-boundary](framework/constitution.md#runtime-boundary) but is not yet implemented (deferred to spec 022).

When tooling-language decisions arise (e.g., the runtime spec'd in 022, future binaries the framework might ship), prefer Rust. The `rmcp` crate is the reference MCP SDK, and the recent generation of Rust CLI tooling (`ripgrep`, `fd`, `bat`, `eza`, `helix`, `tokei`, `hyperfine`, etc.) has set the modern baseline for CLI UX patterns (single static binary, fast cold-start, sensible exit codes). Go is the credible alternative if development velocity matters more than production characteristics; other languages typically don't fit (distribution complexity, startup overhead, ecosystem fit).

## Workflow

- Read `framework/commands/{name}.md` before recommending, describing, or disambiguating a slash command — don't guess from the name. Source files are authoritative; the generated `.claude/commands/gov/*.md` copies are produced by the pre-commit hook.
- `.govern.toml` is treated as a shared adopter-side database, not as a schema owned by any single spec. When a new spec adds a section/key (its own "table"), document it in that spec's body and behavior — do not generate a §cross-spec-impact signpost on spec 019 (or any prior `.govern.toml` spec) for the addition. Reason: surfaced 2026-05-10 while clarifying spec 020 — adding `[review] tech-stack-verified` did not warrant reopening 019. How to apply: only treat a `.govern.toml` change as cross-spec impact when it modifies an *existing* key already documented in another spec.
- **No dead references in live artifacts.** When renaming or removing a name (spec slug, capability, command, identifier, parenthetical descriptor, etc.), update every reference across **live artifacts**: `framework/`, `scripts/`, `runtime/` (including `tests/fixtures/`, `tests/golden/`, `tests/parity/`), `.github/`, `docs/`, `README.md`, `AGENTS.md`, and `specs/NNN-*/` (including done-spec bodies). Reason: a reader following a forward-pointer or back-reference in live artifacts must never land on an outdated name; spec bodies are living documents that represent current state. The sweep is mechanical: every change in the diff is the same find-and-replace token substitution applied uniformly across the artifact set above. Because the diff is uniform, it qualifies as a **mechanical sweep** under [§spec-lifecycle](framework/constitution.md#spec-lifecycle) and does NOT trigger the `done → in-progress` back-edge — done specs stay `done` after a rename sweep. How to apply: when renaming X to Y, grep the live-artifact paths above and update every hit in a single commit; do not bundle the sweep with unrelated edits (a non-uniform diff would qualify as a meaningful edit and reopen affected specs). Commit messages and published PR/release notes stay as written (git history is the audit trail of what was named when).
- **Run `/govern` per its spec — no ad-hoc prompts.** When executing `/govern` (here or in any adopter), do not insert confirmation prompts beyond those the spec specifies (project inputs, agent-selection prompts on `--add-agent` / first-run, the registry-driven migration prompts from `framework/migrations.toml`, the per-category workflow prompts). Specifically, do not stop to warn about uncommitted edits to update-strategy files, custom slash commands about to be removed by **Slash command cleanup**, or "data loss" from the `stale` → write-and-abort path. Reason: the spec already encodes safety — `.govern.toml` `[pinned] files` is the documented opt-out; the stale path writes upstream and aborts cleanly (recoverable from git); slash command cleanup is unconditional for unpinned files. Extra prompts duplicate information the spec already gives the user and stall routine runs. How to apply: trust the documented behavior. The canonical statement of this rule lives as a "Procedural fidelity" preamble at the top of §Instructions in `framework/bootstrap/govern.md` so it travels to every adopter via the self-update path; this AGENTS.md entry is the contributor-side mirror. If a real safety concern seems missing from the spec, raise it as a spec change (inbox entry or discussion) — not as a runtime prompt.
- **Use the `Write` tool, not Bash redirects, for `.claude/gov-session.json`.** Spec 023 added explicit `Edit(.claude/gov-session.json)` and `Write(.claude/gov-session.json)` permission entries so pipeline commands stop prompting on session-file updates. Those entries scope the **Edit** and **Write** tools only — they do not cover Bash redirects (`cat > … <<EOF`, `tee`, etc.), which fall under separate `Bash(...)` permissions. Reason: surfaced 2026-05-17 while running `/gov:target 025` — used a Bash heredoc and got a permission prompt despite spec 023's allowlist work. How to apply: when an agent writes the session JSON (or any `govern`-owned state file with a dedicated `Write(...)` allow entry), use the `Write` tool. Do not widen the Bash allowlist with patterns like `Bash(cat > * *)` to compensate — that grants write-anywhere-via-shell and defeats the per-path scoping. A narrow `Bash(cat > .claude/gov-session.json *)` would be safe, but reaching for the right tool is cheaper than maintaining two surfaces.
- **Use repo-relative paths in tool calls.** When invoking `Read`, `Write`, `Edit`, or `Bash` against files in this repo, use repo-relative paths (e.g., `.claude/gov-session.json`, `specs/027-bootstrap-migration-registry/spec.md`) — never `/Users/stonean/src/stonean/govern/...`. Reason: surfaced 2026-05-17 during `/gov:specify` — agent used an absolute path for the session-file write. Absolute paths leak the maintainer's home directory into transcripts, make diffs and error messages noisier, and break path-pattern matching for `permissions.allow` entries that are written relative. How to apply: every framework artifact (rule files, command sources, scripts, spec links) already uses relative paths; mirror that convention in tool calls. The only places absolutes are appropriate are temp files under `/tmp` or `/private/tmp` and explicit references to files in other working directories listed in the harness environment.

## Gotchas

- Use `npx markdownlint-cli2` to run markdown linting — do not suggest installing it globally.
- The command generator substitutes `{project}` → `gov` and `{cli-config-dir}` → `.claude`, and writes the Claude-specific permission file (`framework/bootstrap/configure/claude.md`) as `configure.md` in the gov command directory.
- `framework/workflows/` files ship as-is — they are not generator inputs and have no `govern`-side `gov:workflows:*` counterpart. Adopting projects scaffold them via `/govern` (or `/gov:init`).
- `.claude/commands/gov/init.md` is the one exception to the generator rule — it is `govern`-specific (no source counterpart) and is hand-maintained. The generator leaves it untouched.
- **Claude Code prompts on `.claude/gov-session.json` writes despite the per-path allowlist.** Spec 023 added `Edit(.claude/gov-session.json)` and `Write(.claude/gov-session.json)` to `.claude/settings.local.json`, plus a bare `Write` permission. The session-file write still prompts (observed 2026-05-17 during `/gov:specify` for spec 027 with both absolute and relative path forms, while every other `Write` in the same session went through silently). Working hypothesis: the harness has a built-in `.claude/`-directory protection that fires before `permissions.allow` is consulted. The AGENTS.md note from c179776 instructing agents to use `Write` (not Bash heredocs) and the per-path allow entries are no-ops under that hypothesis. How to apply: when a `/gov:*` command's session-write step prompts, approve it and continue — this is environment friction, not a settings error. Do NOT widen the Bash allowlist, do NOT retry with different path forms, and do NOT file repeat inbox items. If the friction becomes intolerable, the documented escape hatches are (a) raise upstream with Claude Code to scope the protection to `.claude/settings*.json` only, or (b) move `gov-session.json` out of `.claude/` (e.g., `.govern/session.json`) via a new spec amending 023's session-file path.

## Boundaries

- Never edit `.claude/commands/gov/*.md` directly — your changes will be overwritten the next time the generator runs. Edit the source under `framework/commands/` (or `framework/bootstrap/configure/claude.md` for the `configure` command).

## Design Principles

- **Never design framework features that depend on human diligence or discipline.** Any artifact section, frontmatter field, command behavior, or workflow step that requires an author to *remember* to fill it in, set a flag, update a doc alongside code, or otherwise be careful will fail in practice — silently and asymmetrically (the cases where it gets skipped are exactly the cases where it mattered most). When proposing a new input, ask "what happens when an author forgets?" If the answer is "the feature degrades silently," redesign the input as **derived** (extracted from existing artifacts, frontmatter, git history, code analysis) or don't ship it. Reason: surfaced 2026-05-06 when evaluating an optional `## Upgrade Impact` spec section as a way to capture cross-version migration notes; rejected on this principle and the topic was tabled to inbox until a derivable design is found. How to apply: this is a hard filter on framework proposals, not a tiebreaker — if the only viable design relies on author discipline, the right answer is to defer the feature, not to ship the disciplined version "for now."
