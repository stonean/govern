# govern

**Spec-driven development for AI coding agents.** Describe a feature in plain English; your agent turns it into a spec, a plan, tasks, and reviewed code — and every feature lands with a written record of *why* it was built the way it is.

`govern` is tech-stack agnostic, ships as plain markdown, and works with Claude Code, Auggie, Antigravity, and OpenCode. There's nothing to compile and no dependency to add — you install a single command into your project and drive the rest through a handful of verb-named slash commands.

## Why govern

AI agents are fast, but left to their own devices they're inconsistent: they guess at ambiguous requirements, lose the reasoning behind a change as soon as the chat scrolls away, and reinvent structure on every task. `govern` puts a thin, opinionated pipeline in front of the agent so that:

- **Ambiguity is caught upstream of code.** Open questions get resolved in the spec, not discovered halfway through implementation.
- **Every feature carries its "why."** The spec is a living document that stays accurate after the code ships — not a ticket that gets buried when it closes.
- **The surface area is small.** A few commands map to things you already do: write a ticket, surface unknowns, sketch an approach, build it, audit it.
- **Artifacts stay portable.** Everything is markdown with YAML frontmatter — readable in GitHub, Obsidian, or `cat`, with no proprietary format to escape.

## Quick start

Install `govern` into any project:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/stonean/govern/main/install.sh | sh
```

This installs the `/govern` bootstrap command for Claude Code — see [Installing per agent](#installing-per-agent) to target Auggie, Antigravity, or OpenCode instead. Then, in your agent, run:

```text
/govern my-project
```

That one command scaffolds the `specs/` directory, installs the full set of slash commands, wires up the constitution and agent rules, and prints your next steps. It's idempotent — safe to re-run any time to pull the latest `govern` files.

Now build your first feature by walking it through the pipeline:

```text
/specify   add user login with email and password
/clarify              # resolve open questions the spec surfaced
/plan                 # technical decisions, affected files, tasks
/implement            # work the tasks; code gets written here
/review               # audit the code against the rules
```

Each command advances the feature one step and leaves a durable artifact behind. That's the whole loop.

## How it works

Every feature moves through one pipeline. The status on each spec tracks where it is:

```text
draft ──/clarify──▶ clarified ──/plan──▶ planned ──/implement──▶ in-progress ──/implement──▶ done
```

- **Spec** (`/specify`, `/clarify`) — define *what* the feature does and *why*, with concrete acceptance criteria and a list of open questions. No open questions may remain before planning.
- **Plan** (`/plan`) — turn the spec into technical decisions, affected files, and an ordered task list. Persistence-heavy features also get a data model.
- **Implement** (`/implement`) — work the tasks; this is where code is written. Status moves to `in-progress`, then `done` once the review gate passes.
- **Review** (`/review`) — audit the implementation against the framework's rules (security, reuse, quality, efficiency, simplicity). Blocking violations keep the feature out of `done` until they're fixed or explicitly waived.

`/analyze` can run at any time to check a feature's artifacts against each other — it's a safety check, not a gate.

You don't have to start at `draft`. A brownfield feature can enter with a sparse sketch spec and gain precision as you touch the code; a `done` feature reopens automatically when a bug or change request surfaces. See [docs/introduction.md](docs/introduction.md) for the full mental model, and [framework/constitution.md](framework/constitution.md) for the authoritative rules.

## Commands

Adoption installs a full set of verb-named, session-aware commands. Use `/target` to switch the working feature; `/specify` creates one and targets it automatically.

### Pipeline — advance state

| Command | Purpose |
| --- | --- |
| `/specify` | Create a new feature spec. Accepts rich (greenfield) or sparse (brownfield) input — richness scales with the description |
| `/clarify` | Resolve open questions; advance the spec to `clarified` |
| `/plan` | Create `plan.md` with technical decisions, affected files, and an ordered task list |
| `/implement` | Work through tasks; move the spec to `in-progress`, then `done` |
| `/review` | Audit code against the rules; write `review.md`; block `done` on MUST violations. `--all`, `--fix`, and `--waive <rule-id> --reason "<text>"` supported |
| `/analyze` | Audit a feature's artifacts against each other. `--all` scans every feature; `--fix` auto-corrects checkbox drift |

### Refine — adjust a spec's artifacts

| Command | Purpose |
| --- | --- |
| `/amend` | Add a question or scenario to the targeted spec. Owns the lifecycle back-edges (a new question reopens to `draft`; a new scenario reopens a `done` spec to `in-progress`) |
| `/prune` | Reduce the target's `tasks.md` — drop spent (completed) task sections, or `--reset` to template state. Confirmed, single-artifact; recovery is git history |

### Brownfield — absorb existing reality

| Command | Purpose |
| --- | --- |
| `/log` | Record a raw item to `specs/inbox.md` for later grooming |
| `/groom` | Walk the inbox and route each item to its proper spec or scenario |

### Orient

| Command | Purpose |
| --- | --- |
| `/target` | Set the working feature (or `feature/scenario`) for the session |
| `/status` | Dashboard of every feature's progress, or a focused view of the current target |
| `/link` | Register a service in `.govern/config.toml [services]` so cross-service references resolve to the linked spec's status; `--list` shows registered services and their resolution health |
| `/help` | Project overview and command reference |

### Bootstrap — one-time per project

| Command | Purpose |
| --- | --- |
| `/govern` | Adopt or update `govern` in a project (the installer that placed every other command) |
| `/configure` | Configure agent permissions for `govern` commands |

## Installing (per agent)

`govern` operates a **live-on-main** model — the installer fetches the latest from `main`. Omit the agent to install for Claude Code, or name it explicitly.

### Claude Code

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/stonean/govern/main/install.sh | sh
```

### Auggie

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/stonean/govern/main/install.sh | sh -s -- auggie
```

Using the optional `gvrn` runtime? Auggie needs a one-time manual registration (`auggie mcp add gvrn …`) — see [Registering the runtime](#registering-the-runtime).

### Antigravity

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/stonean/govern/main/install.sh | sh -s -- antigravity
```

Then run `/govern {project-name}`. The installer creates the right directory for your agent and drops the bootstrap command in place — for Antigravity it's wrapped as a skill under `.agents/skills/govern/`, since Antigravity discovers dir-form skills rather than verbatim command files. It's safe to re-run. (`agy`, the Antigravity CLI command name, works in place of `antigravity`.)

### OpenCode

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/stonean/govern/main/install.sh | sh -s -- opencode
```

OpenCode installs the bootstrap as a verbatim command at `.opencode/command/govern.md` (invoked `/govern`) and reads `AGENTS.md` natively — no `CLAUDE.md`. `/govern` wires the `gvrn` runtime automatically by writing the project's root `opencode.json`; because OpenCode loads config once at startup, restart it after the first wiring (see [Registering the runtime](#registering-the-runtime)).

The same bootstrap supports every agent, so re-run `/govern --add-agent` from any adopted agent later to add others. Once the `gvrn` binary is on your `PATH`, `/govern` wires it on its next run — automatically for Claude and OpenCode (both keep MCP config in a committed repo file), or by surfacing a one-time registration step for Auggie and Antigravity (see [Registering the runtime](#registering-the-runtime)).

## Brownfield adoption

You don't need to clone `govern` or rewrite history to adopt it. Install the command, run `/govern`, then let specs accrete naturally:

- Use `/specify` with a sparse description to stub a skeleton spec for an existing feature — sparse acceptance criteria are valid here.
- Let those specs gain precision incrementally through bug fixes, enhancements, and `/clarify`.
- Drop raw items into `specs/inbox.md` with `/log` without breaking flow, and route them later with `/groom`.

Adoption spreads by feature area, not in a big bang. The goal is for `inbox.md` to eventually disappear.

### Bugs are unwritten scenarios

`govern` treats every bug as evidence that a spec is missing, ambiguous, or violated. When one surfaces, follow the decision tree in order:

1. **No spec exists** — write the spec first, then fix the code.
2. **Spec is ambiguous** — fix the spec, then fix the implementation.
3. **Spec is clear, implementation is wrong** — add a scenario, then fix the code.

A scenario is a spec at a lower level of abstraction — same format, same discipline. Scenarios live in `specs/NNN-feature/scenarios/slug.md`, each gets a linked task in the parent spec, and any can be targeted directly with `/target feature/scenario-slug`.

## The optional runtime

The `govern` runtime (`gvrn`) is an **optional** deterministic execution layer. It parses the prose of each command and runs the mechanical work (reading specs, walking tasks, checking dependencies, atomic checkbox updates, gate handshakes) in native Rust instead of slow LLM tokens — invoking the model only where semantic judgment actually matters (`assessSpecQuality`, `writeCode`, `writeSpecBody`).

The markdown-only path stays first-class per [constitution §runtime-boundary](framework/constitution.md#runtime-boundary): when the runtime is absent, the agent walks the same prose. Install it if you run the pipeline frequently — the wall-clock saving on `/analyze` and `/implement` is significant; skip it if you only use the pipeline occasionally.

### Install the runtime

Download the pre-built binary for your platform from the [latest release](https://github.com/stonean/govern/releases) and verify the checksum:

```bash
# Example for aarch64-apple-darwin; substitute your target triple.
VERSION="0.12.1"
TARGET="aarch64-apple-darwin"
ARCHIVE="gvrn-${TARGET}.tar.gz"
BASE="https://github.com/stonean/govern/releases/download/gvrn-v${VERSION}"

# Work in a scratch tempdir so the extracted binary lands away from your tree.
tmp="$(mktemp -d)" && cd "${tmp}"

curl -LO "${BASE}/${ARCHIVE}"
curl -LO "${BASE}/${ARCHIVE}.sha256"
shasum -a 256 -c "${ARCHIVE}.sha256"
tar xzf "${ARCHIVE}"
sudo install -m 0755 gvrn /usr/local/bin/gvrn
gvrn --version

# Clean up.
cd - >/dev/null && rm -rf "${tmp}"
```

Binaries are published for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, and `aarch64-unknown-linux-gnu` (a Windows binary appears when cross-compilation succeeds). If a runtime process crashes mid-procedure, just re-run the command — state lives in your markdown, and writes are filesystem-atomic, so the runtime resumes from the next incomplete step.

### Registering the runtime

You install the binary; the next time you run `/govern` after `gvrn` is on your `PATH`, the bootstrap detects it and adds the matching tool permissions. How the server itself is registered depends on where your agent reads MCP config:

- **Claude** — `/govern` writes `.mcp.json` for you; just start a fresh session. Fully automatic.
- **OpenCode** — `/govern` writes the `gvrn` `mcp` block into your committed root `opencode.json` for you; because OpenCode loads config once at startup, quit and restart it so the server loads. No manual `mcp add`.
- **Auggie** — Auggie reads MCP servers from your user-level `~/.augment/settings.json`, which `/govern` does not write. It surfaces a one-line command to run once — `auggie mcp add gvrn --command gvrn --args "mcp"` — then start a fresh session.
- **Antigravity** — Antigravity reads MCP servers only from your home-level `~/.gemini/config/mcp_config.json` (project-local config is ignored), which `/govern` does not write. It surfaces an instruction: add the `gvrn` block to that file, then reload with the in-prompt `/mcp` overlay.

From that session on, the pipeline takes the deterministic path. File writes are additive — an existing MCP config keeps its other servers, and a `gvrn` entry that's already present is left untouched. If `/govern` can't find the binary, it stays on the markdown path and reminds you that installing `gvrn` cuts token use.

## Configuration

`.govern/config.toml` is an optional project file — `/govern` runs fine without it. Create it only if you need one of these behaviors:

- **`[pinned]`** — list destination paths `govern` should never overwrite, even files it normally updates (e.g. a customized `.govern/constitution.md`).
- **`[rules]`** — declare which rule surfaces your project needs: `surfaces = ["backend"]`, `["frontend"]`, or both. `/govern` prompts for this on first run, then installs only the matching rule files (cross-cutting `-cross` rules always apply) and `/review` enforces only those. Leave it unset to let `govern` derive the surface from your stack and install every rule file.
- **`[paths]`** — rename the top-level directory that holds every `govern` artifact: `specs-root = "governance"`. Defaults to `specs`; set it to avoid colliding with a sibling framework's directory (e.g. RSpec's `spec/`). `/govern` prompts for it on first run; once set, every command and the runtime resolve it. A single directory name — no path separators, no `..`, no leading slash.
- **`[services]`** — register sibling services so cross-service reference links resolve to the linked spec's lifecycle status (see [Cross-service references](#cross-service-references)). Add entries with `/link`, not by hand.

```toml
[pinned]
files = [".govern/constitution.md"]

[rules]
surfaces = ["backend"]

[paths]
specs-root = "governance"
```

For the full schema, see [specs/019-config-decisions/data-model.md](specs/019-config-decisions/data-model.md).

## Cross-service references

When a project spans multiple services — each its own repo with its own `govern` install — a spec can link a spec in another service and see its live lifecycle status. The reference is a standard markdown link to the linked spec's **canonical repo URL**; that URL is identity and navigation only and is **never fetched**. `govern` reads the linked spec's `status` from its **local checkout**, resolved through the `.govern/config.toml [services]` registry.

References are informative, never dependencies: they do not enter `dependencies:`, do not gate completion, and never block a pipeline gate. They are harvested into a derived `references:` frontmatter index — distinct from `dependencies:` — by `.govern/scripts/gen-cross-service-refs.sh`; you never hand-author it.

### Documenting a reference in a spec

You author a reference by writing a **normal inline markdown link** in the spec **body** — nothing goes in the frontmatter, and there is no special syntax. The link's href must be an **absolute `http(s)` URL** whose path contains the target spec's `/specs/NNN-slug/` segment in the other service's repo:

```markdown
Tokens follow the contract in
[api 014-auth-tokens](https://github.com/acme/api/blob/main/specs/014-auth-tokens/spec.md).
```

On the next commit (or any `.govern/scripts/gen-cross-service-refs.sh` run) the generator harvests that link into the frontmatter:

```yaml
references:
  - service: api      # the [services] alias whose repo matches the URL host
    spec: 014-auth-tokens
```

What the generator keys on:

- **`NNN-slug` is the identity.** Everything in the URL before a `/blob/<ref>/` or `/tree/<ref>/` branch segment is the repo, matched against `.govern/config.toml [services]` to resolve the alias; the branch is ignored, so two links to the same spec on different branches collapse to one reference. A URL matching no registered service is still recorded, with `service: null` (the `unregistered` outcome above).
- **Absolute URL, not a sibling link.** `[label](../014-auth-tokens/spec.md)` is a *sibling* link and becomes a **dependency** (a different generator, the blocking `dependencies:` graph) — never a cross-service reference. Use the full canonical URL precisely so the two stay distinct.
- **Opt-outs are honored.** A link is **not** harvested if it sits under a `## See also` heading, inside a fenced code block, wrapped in `` `backticks` `` (inline code reads as an illustrative example, not a live link), or on a blockquote (`>`) line. These are the same navigational opt-outs `dependencies:` honors — use them for example or "see also" links you don't want to register.

Register a service with `/link` (alias, repo URL, local checkout path, optional description):

```toml
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
description = "owns shared data models"
```

The registry is **required for status resolution, optional for referencing** — an unregistered link is just navigation. `/status` shows each reference's resolution outcome (and, on `ok`, the linked status); `/analyze` reports a provably broken one as an Advisory finding. The outcome depends on what can be proven:

| Outcome | Meaning |
| --- | --- |
| `ok` | Registered, checkout reachable, target spec resolves — surfaces the linked lifecycle status |
| `unregistered` | The repo matches no `[services]` entry — a plain navigational link; run `/link` to register the service |
| `not-checked-out` | Registered, but the local `path` is missing or unusable — `unknown`, never reported as broken |
| `broken` | Registered and reachable, but the target spec does not resolve (renamed, moved, deleted, or mistyped) — an `/analyze` finding |
| `status-unreadable` | The target exists but its `status` cannot be read — `unknown`, the defect is upstream's |

Status resolution runs only where the linked service is already checked out locally; `govern` never fetches or clones a repo. For the full schema, see [specs/030-cross-service-references/data-model.md](specs/030-cross-service-references/data-model.md).

## Updating an adopted project

Re-run `/govern` to pull the latest framework files. Each file is handled by one of three strategies:

| Strategy | Behavior | Examples |
| --- | --- | --- |
| `update` | Always overwritten with the latest version | `.govern/constitution.md`, spec templates, slash commands |
| `create` | Created on first run, skipped on re-run | `specs/system.md`, `specs/errors.md`, `specs/events.md` |
| `skip` | Never overwritten | `AGENTS.md`, `CLAUDE.md` |

`.gitignore` uses a `merge` strategy — `govern` patterns are appended below a `# govern` marker. Pin individual files you've customized with `[pinned]` in `.govern/config.toml` (above). `govern` is a reference, not a runtime dependency: if you'd rather not use `/govern`, diff the repo and apply changes at your own pace.

## Security rules

`govern` ships enforceable security rules using RFC 2119 language — **MUST/MUST NOT** are blocking, **SHOULD/SHOULD NOT** are advisory. `/review` loads the rule files for your configured `[rules] surfaces` — or, when that setting is unset, the rule files that match your detected stack.

- [framework/rules/security-backend.md](framework/rules/security-backend.md) — auth, input validation, data protection, API security, logging, dependencies, error handling
- [framework/rules/security-frontend.md](framework/rules/security-frontend.md) — XSS, CSRF, secure storage, auth handling, content security, dependencies

When a MUST violation is intentional, record a waiver instead of silencing the gate:

```bash
/review --waive <rule-id> --reason "<text>"
```

Waivers are anchored to the rule ID and file path — if the file is renamed or the rule stops firing there, the waiver expires and the finding re-blocks. The waiver schema is open, so organizations can layer on their own required fields. See [specs/020-code-review/data-model.md](specs/020-code-review/data-model.md).

## Viewing artifacts

`govern` artifacts are plain markdown with YAML frontmatter, so any markdown viewer or PKM tool can browse them:

- **GitHub** — push `specs/` and browse inline; relative links resolve natively
- **[Obsidian](https://obsidian.md)**, **[Logseq](https://logseq.com)**, **[Foam](https://foambubble.github.io/foam/)** — graph view and backlinks out of the box
- **[Quartz](https://quartz.jzhao.xyz)** or **[MkDocs](https://www.mkdocs.org)** — publish a static site
- Plain `cat`, a GitHub PR review, or any markdown editor — no viewer required

Artifacts stay the portable source of truth; structured viewers are derived views (see [constitution §text-first-artifacts](framework/constitution.md#text-first-artifacts)).

## Repository layout

This repo is the source for everything `govern` ships, plus its own dogfooded specs.

- **[framework/](framework/)** — everything that ships to adopting projects
  - [constitution.md](framework/constitution.md) — guiding principles, pipeline, spec lifecycle, quality standards (authoritative)
  - [rules/](framework/rules/) — domain rule sets adopted by reference
  - [templates/](framework/templates/) — starter files for specs and project scaffolding
  - [commands/](framework/commands/) — slash command sources
  - [bootstrap/](framework/bootstrap/) — the `govern.md` installer and per-agent permission files
- **[install.sh](install.sh)** — the `curl … | sh` installer that places the `/govern` bootstrap command for your agent
- **[docs/introduction.md](docs/introduction.md)** — the long-form pitch for spec-driven development
- **[runtime/](runtime/)** — the optional `gvrn` deterministic runtime (Rust)
- **[specs/](specs/)** — `govern`'s own feature specs; it develops itself with its own pipeline. See [specs/README.md](specs/README.md) for cross-cutting decisions and deferred work.
- **[scripts/](scripts/)** — maintenance and generator scripts

`govern` currently distributes to four AI coding agents: **Claude Code** (`.claude/` paths), **Auggie** (`.augment/` paths), **Antigravity** (`.agents/` paths, installed as a skill), and **OpenCode** (`.opencode/` command tree plus a committed root `opencode.json`). Adding another is a single registry row plus a permission file (or, for a new layout, a derived-values branch) — see [framework/bootstrap/govern.md](framework/bootstrap/govern.md#agent-registry).

## Contributing

All `.md` files must pass `npx markdownlint-cli2` using the project config; see [constitution §markdown-standards](framework/constitution.md#markdown-standards) for the rule set. `govern` dogfoods its own pipeline — changes to the framework go through the same `/specify → /plan → /implement → /review` loop, recorded under [specs/](specs/).

## License

[MIT](LICENSE)
</content>
</invoke>
