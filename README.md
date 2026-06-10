# govern

Standards and conventions for spec-driven software development. This project defines how we run projects — the workflow, spec structure, principles, and quality rules that apply regardless of tech stack.

## TL;DR

`govern` adds a spec-driven pipeline that your AI agent walks for you. You describe a feature in plain English; the agent produces the spec, plan, and tasks in a consistent shape. The surface area you learn is small — a handful of verb-named slash commands (`/specify`, `/clarify`, `/plan`, `/implement`, `/review`, `/analyze`) that map to things you already do: write a ticket, surface unknowns, sketch an approach, build it, audit it, check your work.

The payoff is that ambiguity gets caught upstream of code, and every feature lands with a written record of *why* it's built the way it is.

## Contents

- [framework/](framework/) — Everything that ships to adopting projects
  - [constitution.md](framework/constitution.md) — Guiding principles, development pipeline, spec lifecycle, quality standards. Authoritative.
  - [framework/rules/](framework/rules/) — Domain rule sets adopted by reference
    - [security-backend.md](framework/rules/security-backend.md) — Enforceable backend security rules (RFC 2119)
    - [security-frontend.md](framework/rules/security-frontend.md) — Enforceable frontend security rules (RFC 2119)
  - [framework/templates/](framework/templates/) — Starter files customized per project, split by consumer
    - `templates/spec/` — Templates consumed during the pipeline (spec, plan, tasks, data-model, research, scenario)
    - `templates/project/` — Project document templates consumed during adoption (agents.md, claude-md.md, system.md, errors.md, events.md, project-readme.md, gitignore, inbox.md)
  - [framework/commands/](framework/commands/) — Slash command sources for the operational commands
  - [framework/workflows/](framework/workflows/) — Tech-stack-specific workflow files (lint, test, format, migrate) plus `registry.json` mapping stack selections to workflows
  - [framework/bootstrap/](framework/bootstrap/) — The `govern.md` installer plus per-agent permission files (`configure/{agent}.md`)
- [docs/introduction.md](docs/introduction.md) — Long-form pitch for spec-driven development. The constitution is authoritative for normative rules.
- [specs/](specs/) — Feature specs for `govern` itself (dogfooding the pipeline)
- [scripts/](scripts/) — Maintenance scripts (e.g., regenerate `.claude/commands/gov/` from `framework/commands/`)

## Feature Specs

`govern` uses its own spec-driven pipeline to develop itself.

See [specs/README.md](specs/README.md) for cross-cutting decisions and deferred work.

<!-- generated:feature-specs:start -->

| Spec | Status | Dependencies | Description |
| --- | --- | --- | --- |
| [000-slash-commands](specs/000-slash-commands/spec.md) | done | none | Generic, project-agnostic slash command templates that operationalize the governance development pipeline. |
| [001-system-spec-templates](specs/001-system-spec-templates/spec.md) | done | none | Templates for the cross-cutting system specs that the constitution references but does not provide: `system.md`, `errors.md`, and `events.md`. |
| [002-project-scaffolding](specs/002-project-scaffolding/spec.md) | done | 000, 001 | Templates for the project-level files that every governance-adopting project needs beyond the constitution, AGENTS.md, and spec templates. |
| [003-bootstrap-automation](specs/003-bootstrap-automation/spec.md) | done | 000, 001, 002 | Governance slash commands that dogfood the same pipeline commands adopting projects use (`/gov:about`, `/gov:target`, `/gov:status`, `/gov:setup`, `/gov:specify`, `/gov:clarify`, `/gov:plan`, `/gov:implement`, `/gov:analyze`, `/gov:next`), plus a governance-specific `/gov:init` that scaffolds new projects from templates. |
| [004-tech-stack-selection](specs/004-tech-stack-selection/spec.md) | done | 003 | Interactive tech stack selection during `/gov:init` that collects richer project metadata beyond primary language(s). |
| [005-workflows](specs/005-workflows/spec.md) | done | 004, 010 | Based on project tech stack, recommend and scaffold relevant development workflow files during bootstrap. |
| [006-bug-workflow](specs/006-bug-workflow/spec.md) | done | none | Bugs are unwritten scenarios. |
| [007-govern-workflow](specs/007-govern-workflow/spec.md) | done | 003 | A self-contained slash command file that bootstraps governance in existing (brownfield) projects. |
| [008-security-rules](specs/008-security-rules/spec.md) | done | 007 | Comprehensive, enforceable security rules for backend and frontend development. |
| [009-scenario-targeting](specs/009-scenario-targeting/spec.md) | done | 006 | Promote scenarios to first-class targets in the governance pipeline. |
| [010-agent-autonomy](specs/010-agent-autonomy/spec.md) | done | 000 | Evaluate capabilities found in autonomous agent orchestration tools (e.g., GSD-2) and determine which can be adopted within governance's constraints: zero dependencies, markdown-only artifacts, platform-agnostic, and human-in-the-loop pipeline gates. |
| [011-brownfield-process](specs/011-brownfield-process/spec.md) | done | 007, 023 | A formalized process for initializing and incrementally building out specs in brownfield projects. |
| [012-multi-agent-govern](specs/012-multi-agent-govern/spec.md) | done | 007 | A single `govern.md` command that supports adopting governance for multiple AI coding CLIs in the same project, with the target agent(s) selected at run time rather than baked into the file. |
| [013-text-first-artifacts](specs/013-text-first-artifacts/spec.md) | done | 000, 007, 012 | Declare governance's implicit "all artifacts are markdown" principle in the constitution, formalize spec metadata as YAML frontmatter, and migrate adopted projects to the new format on the next `/govern` run. |
| [014-reclarify-backedge](specs/014-reclarify-backedge/spec.md) | done | 000, 009, 013, 023 | Wire up `/ask` to own the `clarified` / `planned` / `in-progress` → `draft` back-edge so questions surfacing mid-pipeline are captured and the spec's lifecycle invariant is maintained automatically. |
| [015-tarball-fetch](specs/015-tarball-fetch/spec.md) | done | 007, 012 | Collapse `/govern`'s ~35–50 individual `curl` fetches into a single archive download, extracted once into a temp directory and resolved as local paths. |
| [016-cross-cutting-rules](specs/016-cross-cutting-rules/spec.md) | done | 006, 008 | Promote rules to a first-class artifact tier alongside specs and scenarios. |
| [017-derive-dont-ask](specs/017-derive-dont-ask/spec.md) | done | none | Apply the **Design Principles** rule added to `AGENTS.md` on 2026-05-06 ("Never design framework features that depend on human diligence or discipline") to every existing framework input that violates it. |
| [018-adopter-owned-pre-commit](specs/018-adopter-owned-pre-commit/spec.md) | done | 017 | Split the adopter pre-commit hook into two files so `/govern` can keep its generators in sync without ever overwriting code the adopter added to their own pre-commit hook. |
| [019-config-decisions](specs/019-config-decisions/spec.md) | done | 005 | `.govern.toml` is currently a single-purpose pin file: `[pinned] files = [...]` keeps `/govern` from overwriting customized files. |
| [020-code-review](specs/020-code-review/spec.md) | done | none | Adds `/gov:review`, a verb-named slash command that audits implementation code against the framework's rules across five dimensions (reuse, quality, security, efficiency, simplicity), writes a `review.md` artifact alongside the spec, and gates the `in-progress → done` transition via three reinforcing mechanisms. |
| [021-runtime-boundary](specs/021-runtime-boundary/spec.md) | done | 020 | Establish the constitutional scope, eligibility criteria, and opt-in invariant for an optional deterministic runtime that adopters may install alongside the markdown framework. |
| [022-deterministic-runtime](specs/022-deterministic-runtime/spec.md) | done | 021 | The runtime is the deterministic execution layer for govern. |
| [023-govern-refinement](specs/023-govern-refinement/spec.md) | done | 022 | and gov-rt: renames in the audit's RENAMED_TOKENS catalog. |
| [024-rule-loader](specs/024-rule-loader/spec.md) | done | 020, 023 | Generalize `/gov:review`'s rule-file selection so the set of `framework/rules/*.md` files loaded for any given run is derived from each file's declared surface and the project's detected tech stack — not from a hardcoded list of filenames in [`framework/commands/review.md`](../../framework/commands/review.md). |
| [025-rule-opt-out](specs/025-rule-opt-out/spec.md) | done | 020, 024 | Add a narrow `.govern.toml` `[[review.disabled-rule-files]]` opt-out so an adopter whose stack matches a rule file's surface — but whose project is not yet ready to enforce that file's rules — can exclude the file from `/gov:review` loading with a recorded reason. |
| [026-framework-self-audit](specs/026-framework-self-audit/spec.md) | done | 017, 022, 023, 024, 025 | A maintainer-grade slash command that audits `govern`'s own framework artifacts for the kinds of drift [`/gov:analyze`](../../framework/commands/analyze.md) is not scoped to catch. |
| [027-bootstrap-migration-registry](specs/027-bootstrap-migration-registry/spec.md) | done | 026 | Replace the monotonically-growing prose-encoded Pre-run Migrations section in [framework/bootstrap/govern.md](../../framework/bootstrap/govern.md) with a machine-readable registry of convention removals. |
| [028-antigravity-agent](specs/028-antigravity-agent/spec.md) | in-progress | 012, 022 | Generalize the agent registry from "two agents that share one layout" to "N agents across differing per-agent layouts and host conventions," then use that generalization to add Google's **Antigravity CLI** (`agy`) as the third supported agent. |

<!-- generated:feature-specs:end -->

## Runtime

The `govern` runtime is an **optional** deterministic execution layer for slash commands. It parses the prose Instructions section of each `framework/commands/*.md` file directly and dispatches the mechanical work (reading specs, walking tasks, checking dependencies, atomic checkbox updates, gate handshakes) in native Rust instead of having the LLM do it in slow tokens. The LLM is invoked only at named extension points (`assessSpecQuality`, `writeCode`, `writeSpecBody`) where semantic judgment actually matters.

The markdown-only path remains a first-class path per [constitution §runtime-boundary](framework/constitution.md#runtime-boundary). When the runtime is absent, the LLM walks the same prose as today.

### Install

Download the pre-built binary for your platform from the [latest release](https://github.com/stonean/govern/releases) and verify the checksum:

```bash
# Example for aarch64-apple-darwin; substitute your target triple.
VERSION="0.2.1"
TARGET="aarch64-apple-darwin"
ARCHIVE="gvrn-${TARGET}.tar.gz"
BASE="https://github.com/stonean/govern/releases/download/gvrn-v${VERSION}"

# Work in a scratch tempdir so the extracted `gvrn` binary lands away
# from the caller's working tree.
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

Pre-built binaries are published for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, and `aarch64-unknown-linux-gnu`. A Windows binary may also be present when cross-compilation succeeds.

### When to install

Install if you adopt `govern` and run slash commands frequently — the wall-clock saving on `/gov:analyze` and `/gov:implement` is significant. Skip if you only invoke the pipeline occasionally; the markdown-only path is faithful to the same semantics, just slower.

If a runtime process crashes mid-procedure, re-run the slash command — the runtime reads state from your markdown and resumes from the next incomplete step. State-modifying primitives use filesystem-atomic writes (tempfile + rename), so crashes leave coherent markdown. On Windows the rename semantics are weaker; clean up any orphaned tempfile in the spec directory with a manual `rm` if you observe one.

## Adopting in an Existing Project

For brownfield projects, install the `govern` command and run it — no clone required. Once adopted, use `/specify` with a sparse description to initialize skeleton specs for existing features (sparse acceptance criteria are valid for brownfield use), and let them gain precision incrementally through bug fixes, enhancements, and clarification.

`govern` operates a **live-on-main** model — the snippets below fetch the latest from `main`. Tagged releases (`v0.1.0`, etc.) mark milestones for changelogs and release notes, not pinning targets. Adopters who want to lock individual files they've customized use `.govern.toml` (see [Configuring `.govern.toml`](#configuring-governtoml) below).

### Claude Code

```bash
mkdir -p .claude/commands
curl -fsSL https://raw.githubusercontent.com/stonean/govern/main/framework/bootstrap/govern.md \
  > .claude/commands/govern.md
```

Then run `/govern {project-name}`.

### Auggie

```bash
mkdir -p .augment/commands
curl -fsSL https://raw.githubusercontent.com/stonean/govern/main/framework/bootstrap/govern.md \
  > .augment/commands/govern.md
```

Then run `/govern {project-name}`.

The command fetches `govern` files, scaffolds the spec directory, installs slash commands, and displays next steps. It is idempotent — safe to run again to pick up new `govern` files.

The same `govern.md` supports every CLI listed above. Use whichever curl snippet matches the agent you want to start with — adopting additional agents later does not require a second curl. Re-run `/govern --add-agent` from any adopted agent to pick up the others, and the unified file scaffolds them alongside the existing setup.

## Slash Commands

Adoption installs a full set of slash commands that operationalize the pipeline. All commands are verb-named and session-aware — use `/target` to switch to an existing feature; `/specify` creates a new feature and sets it as the session target automatically (accepting both greenfield-rich and brownfield-sparse input).

### Pipeline (advance state)

| Command | Purpose |
| --- | --- |
| `/specify` | Create a new feature spec. Accepts both rich (greenfield) and sparse (brownfield) input — richness scales with the description |
| `/clarify` | Resolve open questions in the current spec, advance status to `clarified` |
| `/plan` | Create plan.md with technical decisions, affected files, and resolved questions |
| `/implement` | Work through tasks, update spec status to `in-progress` then `done` |
| `/review` | Audit code against rules — security, reuse, quality, efficiency, simplicity. Writes `review.md` and the spec's `review:` frontmatter block. Blocks `done` when MUST violations are present. `--all` reviews every `in-progress` or `done` feature. `--fix` applies conservative auto-fixes. Waive MUST findings with `--waive <rule-id> --reason "<text>"`. |
| `/analyze` | Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. `--all` scans every feature. `--fix` auto-corrects fixable checkbox mismatches. Composable: `--all --fix` |

### Refine (add to a spec)

| Command | Purpose |
| --- | --- |
| `/ask` | Add a question or scenario to the targeted spec. The classifier routes the input; the user can `flip` the route at the approval gate. Owns both back-edges (`clarified` / `planned` / `in-progress` → `draft` on a question, `done` → `in-progress` on a scenario). |

### Brownfield (absorb existing reality)

| Command | Purpose |
| --- | --- |
| `/log` | Record a raw item to `specs/inbox.md` for later grooming |
| `/groom` | Walk `specs/inbox.md` and route each item to its proper spec or scenario via the bug decision tree |

### Orient

| Command | Purpose |
| --- | --- |
| `/target` | Set the working feature (or `feature/scenario`) for the session |
| `/status` | Dashboard showing all features' progress, or focused view of the current target |
| `/help` | Display project overview and slash command reference |

### Bootstrap (one-time per project)

| Command | Purpose |
| --- | --- |
| `/govern` | Adopt or update `govern` in an existing project (the installer that placed every other command) |
| `/configure` | Configure agent permissions for `govern` commands |

### Waivers

`/review` blocks the spec from reaching `done` while any MUST violation is unresolved. When a violation is intentional — internal-only endpoint, framework-version constraint, etc. — record a waiver explicitly rather than silencing the gate:

```bash
/review --waive <rule-id> --reason "<text>"
```

The waiver appends a record to the spec's `review.waivers` frontmatter list (`rule`, `file`, `reason`, `waived-at`, `waived-by`). It is anchored to the rule ID and file path: if the file is renamed or the rule no longer fires there, the waiver expires on the next `/review` run and the finding re-blocks.

The waiver list is open-schema — organizations that require additional fields (e.g., `co-waived-by`, `approved-by-team`, `ticket`) can layer them on without `govern` erroring on the unknown keys, then gate those fields in their own CI. See [specs/020-code-review/data-model.md](specs/020-code-review/data-model.md) for the full schema and expiry rules.

## Starting a New Project

The fastest path is `/govern` (see [Adopting in an Existing Project](#adopting-in-an-existing-project) above) — it scaffolds a working setup into a fresh repo. The manual steps below are listed for reference or for agents that don't yet support `/govern`.

### 1. Bootstrap project structure

```bash
mkdir my-project && cd my-project
git init
```

### 2. Copy `govern` files

Copy these files from `govern` into your project root:

| Source | Destination | Purpose |
| --- | --- | --- |
| `framework/constitution.md` | `constitution.md` | Principles, pipeline, spec lifecycle — customize the intro, keep the rest |
| `framework/templates/project/agents.md` | `AGENTS.md` | Agent rules template — fill in every section for your tech stack |
| `.markdownlint-cli2.jsonc` | `.markdownlint-cli2.jsonc` | Markdown linting config — use as-is |

### 3. Fill in AGENTS.md

Open `AGENTS.md` and replace every placeholder section:

- **Tech Stack** — list your languages, frameworks, databases, and versions
- **Commands** — define `dev`, `build`, `test`, `lint` (or your equivalents)
- **Project Structure** — map out your directory layout
- **Code Style** — show idiomatic patterns with code examples
- **Testing** — define test types, file placement, and tooling conventions
- **Gotchas** — document framework quirks and non-obvious behavior
- **Boundaries** — define what agents must never do without asking

### 4. Set up AI agent configuration

Create a `CLAUDE.md` (or equivalent for your agent) that imports the constitution and agent rules:

```text
@import constitution.md
@import AGENTS.md
```

### 5. Create the specs directory

```bash
mkdir specs
```

Write `specs/system.md` describing your architecture — server lifecycle, request flow, shared infrastructure, and module pattern. Add `specs/errors.md` and `specs/events.md` if your project uses structured errors or event-driven communication.

### 6. Add your first feature spec

Run `/specify` to create a numbered feature directory with a spec from template. The command accepts both rich (greenfield) and sparse (brownfield) input — richness scales with the description. Every spec uses the same artifact set (`spec.md`, `plan.md`, `tasks.md`).

Alternatively, create one manually:

```bash
mkdir specs/000-skeleton
cp /path/to/govern/framework/templates/spec/spec.md specs/000-skeleton/spec.md
```

### 7. Work through the pipeline

Follow the pipeline defined in `constitution.md`:

1. **Spec** — resolve all open questions, update status to `clarified`
2. **Plan** — run `/plan` to create plan.md (technical decisions, affected files) and tasks.md (ordered work items) in one step. If the feature involves persistence, also add data-model.md. Updates spec status to `planned`
3. **Implement** — follow the tasks list, update spec status to `in-progress`
4. **Review** — run `/review` to audit the code against rules; resolve MUST violations or record waivers. The `done` transition is gated by `review.blocking: false`
5. **Done** — `/implement` completes the `in-progress → done` transition when the review gate passes

Run `/analyze` any time to audit a feature's artifacts; it is not a pipeline gate, but it is the recommended check before starting `/implement` and before the final `/review`.

## Security Rules

The `govern` framework includes enforceable security rules for backend and frontend code, distributed via `/govern`. Rules use RFC 2119 language: **MUST/MUST NOT** are blocking violations, **SHOULD/SHOULD NOT** are advisory warnings.

- [framework/rules/security-backend.md](framework/rules/security-backend.md) — Authentication, authorization, input validation, data protection, API security, logging, dependency management, and error handling
- [framework/rules/security-frontend.md](framework/rules/security-frontend.md) — XSS prevention, CSRF protection, secure storage, authentication handling, content security, and dependency management

Projects can reference these rules in their AGENTS.md or validate command to enforce security standards during development.

## Templates Reference

Spec-pipeline templates (consumed by an agent during the pipeline):

| Template | When to use |
| --- | --- |
| [spec.md](framework/templates/spec/spec.md) | Starting a new feature — requirements, acceptance criteria, open questions |
| [plan.md](framework/templates/spec/plan.md) | Planning phase — technical decisions, affected files, resolved questions |
| [tasks.md](framework/templates/spec/tasks.md) | Tasks phase — ordered work items derived from the plan |
| [data-model.md](framework/templates/spec/data-model.md) | Plan phase — when the feature involves database persistence |
| [research.md](framework/templates/spec/research.md) | Optional — background research, prior art, references |
| [scenario.md](framework/templates/spec/scenario.md) | Scenario route of `/ask` — capturing specific behavior, edge case, or bug fix as a scenario file under the parent spec |

Project-scaffolding templates (consumed once at adoption):

| Template | Purpose |
| --- | --- |
| [agents.md](framework/templates/project/agents.md) | `AGENTS.md` — tech stack, conventions, code style, boundaries |
| [claude-md.md](framework/templates/project/claude-md.md) | `CLAUDE.md` — Claude Code-specific configuration |
| [project-readme.md](framework/templates/project/project-readme.md) | Starter project README |
| [system.md](framework/templates/project/system.md) | `specs/system.md` — architecture and shared conventions |
| [errors.md](framework/templates/project/errors.md) | `specs/errors.md` — error handling conventions |
| [events.md](framework/templates/project/events.md) | `specs/events.md` — global event catalog |
| [inbox.md](framework/templates/project/inbox.md) | `specs/inbox.md` — temporary inbox for known issues during brownfield adoption |
| [gitignore](framework/templates/project/gitignore) | `govern`-related patterns merged into `.gitignore` |

## Bug Workflow

Bugs are unwritten scenarios. The `govern` framework treats every bug as evidence that a spec is missing, ambiguous, or violated.

### Decision tree

When a bug is reported, follow in order:

1. **No spec exists** — write the spec first, then fix the code
2. **Spec is ambiguous** — fix the spec, then fix the implementation
3. **Spec is clear, implementation is wrong** — add a scenario, then fix the code

### Scenarios

A scenario is a spec at a lower level of abstraction. Scenarios live in `specs/NNN-feature/scenarios/slug.md` and capture edge cases, bug fixes, and detailed behavior. Each scenario gets a linked task in the parent spec's `tasks.md`. Scenarios can be targeted directly with `/target feature/scenario-slug` for focused work.

### Inbox

For brownfield projects, `specs/inbox.md` is a temporary inbox. Items are recorded with `/log` and groomed into specs or scenarios with `/groom`. The goal is for the inbox to eventually be empty.

## Optional CI enforcement

`/govern` installs a local pre-commit hook (`.githooks/pre-commit`) that keeps generated artifacts (currently the spec `dependencies:` frontmatter) in sync on every commit. For contributors who never installed the hook locally, govern ships a GitHub Actions template at [framework/templates/ci/adopter-generators.yml](framework/templates/ci/adopter-generators.yml). Copy it into your project at `.github/workflows/govern-generators.yml` to fail PRs when generators are out of sync. The template is not auto-installed because that would require detecting which CI platform you use (GHA vs. GitLab vs. Buildkite), which is beyond `/govern`'s scope.

## Updating an Adopted Project

Projects that were bootstrapped with `/govern` or adopted `govern` manually can pull the latest `govern` files by running `/govern` again. It uses three strategies to decide how each file is handled:

| Strategy | Behavior | Examples |
| --- | --- | --- |
| `update` | Always overwritten with the latest `govern` version | `constitution.md`, spec templates, slash commands |
| `create` | Created on first run, skipped on re-run | `specs/system.md`, `specs/errors.md`, `specs/events.md` |
| `skip` | Never overwritten | `AGENTS.md`, `CLAUDE.md` |

The `.gitignore` uses a `merge` strategy — `govern` patterns are appended below a `# govern` marker if the marker is not already present.

Re-running `/govern` always pulls from `main` — the project does not pin to a specific tag. To track a tag instead, edit the `https://raw.githubusercontent.com/.../main/` URL inside your installed `govern.md` to point at the tag (e.g., `v0.1.0`). Most adopters won't need to — the per-file pinning below is finer-grained and usually the right tool.

### Configuring `.govern.toml`

`.govern.toml` is the project's optional configuration and persisted-decisions file. Create it at your project root only if you need one of the behaviors below — `/govern` runs without it just fine.

#### `[pinned]` — keep `/govern` from overwriting customized files

If your project has customized a file that `govern` normally overwrites (`update` strategy), pin it:

```toml
[pinned]
# Files listed here use 'skip' instead of 'update'.
# Use destination paths (after placeholder resolution).
files = [
  ".claude/commands/myapp/implement.md",
  "constitution.md",
]
```

Any file listed in `pinned.files` is treated as `skip` instead of `update` when `/govern` runs. Pinned files are reported in the post-scaffolding summary.

#### `[workflows]` — stop being asked about declined workflow categories

When `/govern` offers a workflow category (Linting, Formatting, Testing, Migrations, Code Review, Deployment), the prompt has three options: `Yes, scaffold all in this category`, `Skip this run`, or `Skip and don't ask again`. Picking the third option records the decline here:

```toml
[workflows]
declined_categories = ["Linting"]
```

Categories listed are matched case-insensitively against the canonical category list. `/govern` won't prompt for them on subsequent runs and reports `suppressed (workflow): {Category} (declined in .govern.toml)` in the summary. To re-enable the prompt for a category, remove it from the array (or delete the `[workflows]` section, or delete the file).

For the full schema — allowed values, unrecognized-entry handling, and future sections — see [`specs/019-config-decisions/data-model.md`](specs/019-config-decisions/data-model.md).

### Manual updates

If you prefer not to use `/govern`, `govern` is a reference, not a dependency. Review the `govern` changelog or diff, decide which changes apply to your project, and update your copies at your own pace.

## Platform Support

`govern` currently distributes to two AI coding agents:

- **Claude Code** — `.claude/` paths, `/govern` and `/gov:*` commands
- **Auggie** — `.augment/` paths, `/govern` command variant

Adding a new agent is a single registry row plus an agent-specific `framework/bootstrap/configure/{key}.md` permission file — see [framework/bootstrap/govern.md](framework/bootstrap/govern.md#agent-registry) for the full rules.

## Viewing artifacts

`govern` artifacts are plain markdown with YAML frontmatter, so any markdown viewer or PKM tool can browse them. Pick whichever fits your workflow:

- **GitHub** — push `specs/` and browse inline; relative links resolve natively, no tooling required
- **[Obsidian](https://obsidian.md)** — point at the repo and open; graph view and backlinks out of the box
- **[Logseq](https://logseq.com)** — open-source PKM with a similar graph model
- **[Foam](https://foambubble.github.io/foam/)** — VS Code extension for markdown knowledge graphs
- **[Quartz](https://quartz.jzhao.xyz)** — publish a static graph-style site; see Quartz's docs for setup
- **[MkDocs](https://www.mkdocs.org)** — static documentation site generator
- Plain `cat`, GitHub PR review, or any markdown editor — no viewer required

The principle is that artifacts stay portable and source-of-truth markdown, with structured viewers as derived views (see [constitution.md](framework/constitution.md#text-first-artifacts)).

## Markdown

All `.md` files must pass `npx markdownlint-cli2` using the project config. See [constitution.md](framework/constitution.md#markdown-standards) for the full rule set.
