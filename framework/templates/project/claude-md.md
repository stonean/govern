# CLAUDE.md

@import constitution.md
@import AGENTS.md

## Auto-Memory Routing

> Agent-specific routing for the constitution's *shared knowledge stays in git* principle (Drift Prevention).

Before saving an auto-memory entry, ask: **would this learning help any other contributor to this project?**

- **Yes** → it belongs in a git-tracked artifact, never local auto-memory. Local memory lives under the user's home directory, invisible to everyone else and absent from clones — parking contributor-beneficial guidance there defeats the purpose of a shared, committed codebase. Route a project learning to `AGENTS.md` (matching section: Gotchas, Workflow, Boundaries, Code Style, Testing); route a cross-cutting requirement to the relevant spec, scenario, or rule under `specs/`. Skip the memory entry.
- **No** → auto-memory is correct. Reserve it for facts that are purely personal to this user and carry no value to other contributors: cross-project user facts (role, persistent style preferences) and external reference pointers (Linear/Slack/dashboard bookmarks).
