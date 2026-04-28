# Governance Specs

Cross-cutting decisions, conventions, and deferred work that span multiple feature specs.

## Design Decisions

- **Lightweight track detection** — the `specify` command determines whether a feature qualifies for lightweight track by prompting the user with qualifying questions (single module? no open questions? trivial data model? under 50 lines?). If all answers indicate "small and clear," it creates `spec-and-plan.md` from a combined template. Pipeline commands (`clarify`, `plan`) detect which file exists and adapt accordingly.
- **Templates live at governance root** — all templates (spec, plan, system, errors, events, project scaffolding) live in `templates/` at the governance root. This is the source. The init command copies spec templates to `{project}/specs/templates/` and system spec templates to `{project}/specs/` during bootstrap. Governance is the source, not an adopting project.

## Future Considerations

- **retire/archive command** — a command to mark abandoned specs. Deferred because projects can manually update status or delete directories. Revisit if a pattern of abandoned specs emerges across adopting projects.
- **minimal flag for init** — a `--minimal` flag that skips system spec templates and events.md for simpler projects. Deferred because empty templates cost nothing to include and remind adopters to think about these patterns. Revisit if adopters consistently delete the same files.
- **interactive tech stack selection during init** — expand init inputs beyond languages to include database options (relational, key/value, etc.), messaging, caching, and other infrastructure. From those selections, populate AGENTS.md with relevant patterns, anti-patterns, and conventions. Deferred to start with languages only and iterate based on experience.
- **hooks** — scaffold a starter `.claude/hooks.json` during init with safe, universal hooks (e.g., run markdownlint on changed `.md` files before commit, block force-push on spec directories). Language-specific hooks could be included as commented-out examples. Deferred to keep init simple; revisit once tech stack selection provides richer project metadata.
- **MCP servers** — recommend useful MCP servers as part of init's "next steps" output based on project language and description (e.g., database MCP for projects with persistence, GitHub MCP for hosted repos). Start with recommendations only; interactive installation deferred until init collects richer project metadata via tech stack selection.
- **subagents in pipeline commands** — refine existing command templates (especially validate, review) to leverage parallel subagents for performance. For example, `/project:validate` could spawn concurrent agents for spec consistency, test execution, and linting. Not an init concern — this is prompt engineering within command templates. Deferred because current sequential execution is adequate; revisit as project complexity grows and validation time becomes a bottleneck.
- **Auggie permissions setup** — resolved. Auggie supports per-project `settings.local.json` with `toolPermissions` format. A separate `commands/setup/auggie.md` command handles Auggie-specific permissions using tool names (`launch-process`, `str-replace-editor`, `save-file`, etc.) and `shellInputRegex` patterns. The unified `govern.md` (see [012](012-multi-agent-govern/spec.md)) selects the per-agent setup source by registry key, picking `commands/setup/auggie.md` when Auggie is adopted. Includes migration logic to remove incompatible `permissions` keys from existing projects.

## Conventions

- **Finish before moving on** — prefer completing a feature through the full pipeline (spec → plan → tasks → implement → done) before starting the next. This keeps context focused and avoids half-finished work scattered across features. Not a hard gate — sometimes planning multiple features in parallel makes sense when they inform each other — but the default should be depth-first.
